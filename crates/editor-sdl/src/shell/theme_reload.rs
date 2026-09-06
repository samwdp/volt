fn load_next_deferred_icon_font<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    settings: &ThemeRuntimeSettings,
    user_library: &dyn UserLibrary,
    fonts: &mut FontSet<'ttf>,
    deferred_icon_font_paths: &mut Option<VecDeque<PathBuf>>,
    deferred_icon_fonts_complete: &mut bool,
) -> Result<bool, ShellError> {
    if *deferred_icon_fonts_complete {
        return Ok(false);
    }

    if deferred_icon_font_paths.is_none() {
        *deferred_icon_font_paths = Some(resolve_icon_font_paths()?.into());
    }
    let Some(paths) = deferred_icon_font_paths.as_mut() else {
        return Ok(false);
    };
    let Some(path) = paths.pop_front() else {
        validate_bundled_icon_fonts(fonts, user_library)?;
        *deferred_icon_fonts_complete = true;
        return Ok(false);
    };

    let effective_font_size = scaled_font_size(settings.font_size, settings.display_scale);
    let primary_line_height = fonts.primary().height().max(1);
    let (name, font, raster_font, pixel_size) =
        load_icon_font(ttf, &path, effective_font_size, primary_line_height)?;
    fonts.push_icon_font(name, font, raster_font, pixel_size);

    if paths.is_empty() {
        validate_bundled_icon_fonts(fonts, user_library)?;
        *deferred_icon_fonts_complete = true;
    }

    Ok(true)
}

fn load_deferred_emoji_font<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    settings: &ThemeRuntimeSettings,
    fonts: &mut FontSet<'ttf>,
    deferred_emoji_font_loaded: &mut bool,
) -> bool {
    if *deferred_emoji_font_loaded {
        return false;
    }
    *deferred_emoji_font_loaded = true;
    let Some((font, raster_font, pixel_size, shape_face)) = load_emoji_font(ttf, settings) else {
        return false;
    };
    fonts.set_emoji_font(font, raster_font, pixel_size, shape_face);
    true
}

fn update_theme_runtime<'ttf, 'texture>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    state: &ShellState,
    config: &ShellConfig,
    display_scale: f32,
    slots: ThemeRuntimeSlots<'_, 'ttf, 'texture>,
) -> Result<bool, ShellError> {
    let ThemeRuntimeSlots {
        theme_settings,
        fonts,
        font_path,
        text_texture_cache,
        line_height,
        ascent,
        cell_width,
    } = slots;
    let updated = theme_runtime_settings(
        state.runtime.services().get::<ThemeRegistry>(),
        config,
        display_scale,
    );
    if &updated == theme_settings {
        return Ok(false);
    }

    let mut fonts_changed = false;
    if updated.font_size != theme_settings.font_size
        || updated.font_request != theme_settings.font_request
        || updated.emoji_font_request != theme_settings.emoji_font_request
        || updated.emoji_font_size != theme_settings.emoji_font_size
        || updated.display_scale != theme_settings.display_scale
    {
        let (next_fonts, next_font_path) = load_font_set_with_mode(
            ttf,
            &updated,
            &*shell_user_library(&state.runtime),
            OptionalFontLoadMode::Eager,
        )?;
        *font_path = next_font_path;
        *fonts = next_fonts;
        text_texture_cache.clear();
        *line_height = fonts.primary().height().max(1) as usize;
        *ascent = fonts.primary().ascent();
        *cell_width = fonts.cell_width();
        fonts_changed = true;
    }

    *theme_settings = updated;
    Ok(fonts_changed)
}

fn refresh_theme_registry_if_needed(
    runtime: &mut EditorRuntime,
    reload_state: &mut ThemeReloadState,
    now: Instant,
) -> Result<bool, String> {
    if now
        .checked_duration_since(reload_state.last_checked_at)
        .unwrap_or_else(|| Duration::from_secs(0))
        < THEME_SOURCE_POLL_INTERVAL
    {
        return Ok(false);
    }
    reload_state.last_checked_at = now;

    let next_fingerprint = current_theme_source_fingerprint();
    if next_fingerprint == reload_state.fingerprint {
        return Ok(false);
    }
    reload_state.fingerprint = next_fingerprint;

    let active_theme_id = runtime
        .services()
        .get::<ThemeRegistry>()
        .and_then(|registry| registry.active_theme().map(|theme| theme.id().to_owned()));
    let reloaded = rebuild_theme_registry(
        shell_user_library(runtime).themes(),
        active_theme_id.as_deref(),
    )?;
    runtime.services_mut().insert(reloaded);
    Ok(true)
}

fn refresh_user_config_if_needed(reload_state: &mut UserConfigReloadState, now: Instant) -> bool {
    if now
        .checked_duration_since(reload_state.last_checked_at)
        .unwrap_or_else(|| Duration::from_secs(0))
        < THEME_SOURCE_POLL_INTERVAL
    {
        return false;
    }
    reload_state.last_checked_at = now;
    let next_fingerprint = current_user_config_source_fingerprint();
    if next_fingerprint == reload_state.fingerprint {
        return false;
    }
    reload_state.fingerprint = next_fingerprint;
    true
}

fn rebuild_theme_registry<I>(
    themes: I,
    active_theme_id: Option<&str>,
) -> Result<ThemeRegistry, String>
where
    I: IntoIterator<Item = editor_theme::Theme>,
{
    let mut registry = ThemeRegistry::new();
    registry
        .register_all(themes)
        .map_err(|error| error.to_string())?;
    if let Some(theme_id) = active_theme_id {
        let _ = registry.activate(theme_id);
    }
    Ok(registry)
}

fn current_theme_source_fingerprint() -> Option<ThemeSourceFingerprint> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    let themes_dir = theme_sources_dir_from_exe_dir(exe_dir)?;
    theme_source_fingerprint_from_dir(&themes_dir)
}

fn current_user_config_source_fingerprint() -> Option<UserConfigSourceFingerprint> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    let config_root = user_config_root_dir_from_exe_dir(exe_dir)?;
    user_config_source_fingerprint_from_root(&config_root)
}

fn theme_sources_dir_from_exe_dir(exe_dir: &Path) -> Option<PathBuf> {
    let mut fallback = None;
    for ancestor in exe_dir.ancestors().take(THEME_SOURCE_SEARCH_DEPTH) {
        let mut candidate = PathBuf::from(ancestor);
        for part in THEME_DIRECTORY_PARTS {
            candidate = candidate.join(part);
        }
        if !candidate.is_dir() {
            continue;
        }
        if ancestor.join("Cargo.toml").is_file() {
            return Some(candidate);
        }
        fallback.get_or_insert(candidate);
    }
    fallback
}

fn user_config_root_dir_from_exe_dir(exe_dir: &Path) -> Option<PathBuf> {
    let mut fallback = None;
    for ancestor in exe_dir.ancestors().take(THEME_SOURCE_SEARCH_DEPTH) {
        let mut candidate = PathBuf::from(ancestor);
        for part in USER_CONFIG_DIRECTORY_PARTS {
            candidate = candidate.join(part);
        }
        if !candidate.is_dir() {
            continue;
        }
        if ancestor.join("Cargo.toml").is_file() {
            return Some(candidate);
        }
        fallback.get_or_insert(candidate);
    }
    fallback
}

fn theme_source_fingerprint_from_dir(themes_dir: &Path) -> Option<ThemeSourceFingerprint> {
    if !themes_dir.is_dir() {
        return None;
    }

    let mut files = fs::read_dir(themes_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case(THEME_FILE_EXTENSION))
        })
        .map(|path| {
            let metadata = fs::metadata(&path).ok();
            ThemeSourceFile {
                path,
                size: metadata.as_ref().map_or(0, |metadata| metadata.len()),
                modified_at: metadata.and_then(|metadata| metadata.modified().ok()),
            }
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Some(ThemeSourceFingerprint { files })
}

fn user_config_source_fingerprint_from_files<I>(files: I) -> Option<UserConfigSourceFingerprint>
where
    I: IntoIterator<Item = PathBuf>,
{
    let files = files
        .into_iter()
        .filter(|path| path.is_file())
        .map(|path| {
            let metadata = fs::metadata(&path).ok();
            UserConfigSourceFile {
                path,
                size: metadata.as_ref().map_or(0, |metadata| metadata.len()),
                modified_at: metadata.and_then(|metadata| metadata.modified().ok()),
            }
        })
        .collect::<Vec<_>>();
    if files.is_empty() {
        return None;
    }
    let mut files = files;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Some(UserConfigSourceFingerprint { files })
}

fn user_config_source_fingerprint_from_root(
    root_dir: &Path,
) -> Option<UserConfigSourceFingerprint> {
    let master_path = root_dir.join(USER_CONFIG_FILE_NAME);
    let mut files = Vec::new();
    if master_path.is_file() {
        files.push(master_path.clone());
    }
    if let Ok(contents) = fs::read_to_string(&master_path) {
        for relative in user_config_child_paths(&contents) {
            let path = root_dir.join(relative);
            if path.is_file() {
                files.push(path);
            }
        }
    }
    user_config_source_fingerprint_from_files(files)
}

fn user_config_child_paths(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once(':')?;
            matches!(key.trim(), "workspace" | "acp" | "ui" | "oil")
                .then(|| value.trim().trim_matches('"').trim_matches('\'').to_owned())
        })
        .filter(|value| !value.is_empty())
        .collect()
}

fn theme_runtime_settings(
    theme_registry: Option<&ThemeRegistry>,
    config: &ShellConfig,
    display_scale: f32,
) -> ThemeRuntimeSettings {
    let font_request = theme_registry
        .and_then(|registry| registry.resolve_string(OPTION_FONT))
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("default"))
        .map(str::to_owned);
    let emoji_font_request = theme_registry
        .and_then(|registry| registry.resolve_string(OPTION_EMOJI_FONT))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let font_size = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_FONT_SIZE))
        .map(|value| value.max(1.0).round() as u32)
        .unwrap_or(config.font_size);
    let emoji_font_size = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_EMOJI_FONT_SIZE))
        .map(|value| value.max(1.0).round() as u32)
        .unwrap_or(font_size);
    ThemeRuntimeSettings {
        font_request,
        emoji_font_request,
        font_size,
        emoji_font_size,
        display_scale: normalize_display_scale(display_scale),
        window_effects: current_window_effect_settings(theme_registry),
    }
}

fn resolve_font_path(request: Option<&str>) -> Result<PathBuf, ShellError> {
    if let Some(request) = request.and_then(|value| (!value.is_empty()).then_some(value))
        && let Some(path) = resolve_font_request(request)
    {
        return Ok(path);
    }
    find_system_monospace_font().map_err(ShellError::from)
}

fn resolve_font_request(request: &str) -> Option<PathBuf> {
    let trimmed = request.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return path.exists().then(|| path.to_path_buf());
    }
    if path.extension().is_some() || trimmed.contains('/') || trimmed.contains('\\') {
        if let Ok(exe_path) = env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            let candidate = exe_dir.join(path);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        if path.exists() {
            return Some(path.to_path_buf());
        }
        return None;
    }
    find_font_by_name(trimmed)
}

fn asset_path_from_parts(base: &Path, parts: &[&str]) -> PathBuf {
    parts
        .iter()
        .fold(base.to_path_buf(), |candidate, part| candidate.join(part))
}

fn resolve_default_workspace_root(exe_path: Option<&Path>, cwd: Option<&Path>) -> Option<PathBuf> {
    if let Some(exe_dir) = exe_path.and_then(Path::parent) {
        for ancestor in exe_dir
            .ancestors()
            .take(DEFAULT_WORKSPACE_ROOT_SEARCH_DEPTH)
        {
            let candidate = ancestor.join("user");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
        // Fall back to the executable-relative user directory even before it exists so the
        // default workspace targets the bundled customization path consistently.
        return Some(exe_dir.join("user"));
    }
    cwd.map(Path::to_path_buf)
}

fn default_workspace_root() -> Option<PathBuf> {
    resolve_default_workspace_root(
        env::current_exe().ok().as_deref(),
        env::current_dir().ok().as_deref(),
    )
}

fn resolve_bundled_icon_font_dir() -> Result<PathBuf, ShellError> {
    let mut roots = Vec::new();
    if let Ok(exe_path) = env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        roots.extend(
            exe_dir
                .ancestors()
                .take(BUNDLED_ICON_FONT_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }
    if let Ok(cwd) = env::current_dir() {
        roots.extend(
            cwd.ancestors()
                .take(BUNDLED_ICON_FONT_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }
    for root in roots {
        for parts in BUNDLED_ICON_FONT_DIR_CANDIDATES {
            let candidate = asset_path_from_parts(&root, parts);
            if candidate.is_dir() {
                return Ok(candidate);
            }
        }
    }
    let candidates = BUNDLED_ICON_FONT_DIR_CANDIDATES
        .iter()
        .map(|parts| parts.join("\\"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(ShellError::Runtime(format!(
        "bundled icon font directory not found; looked for {candidates}"
    )))
}

fn resolve_bundled_icon_font_paths() -> Result<Vec<PathBuf>, ShellError> {
    let font_dir = resolve_bundled_icon_font_dir()?;
    BUNDLED_ICON_FONT_FILES
        .iter()
        .map(|name| {
            let path = font_dir.join(name);
            if path.is_file() {
                Ok(path)
            } else {
                Err(ShellError::Runtime(format!(
                    "bundled icon font `{name}` is missing from `{}`",
                    font_dir.display()
                )))
            }
        })
        .collect()
}

fn resolve_system_icon_font_paths() -> Vec<PathBuf> {
    SYSTEM_ICON_FONT_CANDIDATES
        .iter()
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .collect()
}

fn resolve_system_emoji_font_paths() -> Vec<PathBuf> {
    SYSTEM_EMOJI_FONT_CANDIDATES
        .iter()
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .collect()
}

fn resolve_emoji_font_path(request: Option<&str>) -> Option<PathBuf> {
    if let Some(request) = request.and_then(|value| (!value.is_empty()).then_some(value))
        && let Some(path) = resolve_font_request(request)
    {
        return Some(path);
    }
    resolve_system_emoji_font_paths().first().cloned()
}

fn resolve_icon_font_paths() -> Result<Vec<PathBuf>, ShellError> {
    let mut paths = resolve_bundled_icon_font_paths()?;
    let mut seen = paths.iter().cloned().collect::<BTreeSet<_>>();
    for path in resolve_system_icon_font_paths() {
        if seen.insert(path.clone()) {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn resolve_startup_icon_font_paths() -> Result<Vec<PathBuf>, ShellError> {
    let bundled_dir = resolve_bundled_icon_font_dir()?;
    let nfm_path = bundled_dir.join("NFM.ttf");
    if !nfm_path.is_file() {
        return resolve_icon_font_paths();
    }

    let mut paths = vec![nfm_path];
    let mut seen = paths.iter().cloned().collect::<BTreeSet<_>>();
    for path in resolve_system_icon_font_paths() {
        if seen.insert(path.clone()) {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn validate_bundled_icon_fonts(
    fonts: &FontSet<'_>,
    user_library: &dyn UserLibrary,
) -> Result<(), ShellError> {
    let mut missing_count = 0usize;
    let mut examples = Vec::new();
    for symbol in user_library.icon_symbols() {
        let supported = symbol
            .glyph
            .chars()
            .all(|character| fonts.icon_font_index_for_char(character).is_some());
        if supported {
            continue;
        }
        missing_count += 1;
        if examples.len() < 12 {
            examples.push(format!("{} ({})", symbol.id(), symbol.codepoint_label()));
        }
    }
    if missing_count == 0 {
        return Ok(());
    }
    let loaded_fonts = fonts
        .icon_fonts()
        .iter()
        .map(|font| font.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(ShellError::Runtime(format!(
        "bundled icon font validation failed: {missing_count} exported icons are missing from the startup icon-font stack ({loaded_fonts}). examples: {}",
        examples.join(", ")
    )))
}

fn load_emoji_font<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    settings: &ThemeRuntimeSettings,
) -> Option<(Font<'ttf>, RasterFont, f32, ShapeFace<'static>)> {
    let emoji_effective_font_size =
        scaled_font_size(settings.emoji_font_size, settings.display_scale);
    let emoji_path = resolve_emoji_font_path(settings.emoji_font_request.as_deref())?;
    let emoji_font_data = fs::read(&emoji_path).ok()?;
    let emoji_font_data: &'static [u8] = Box::leak(emoji_font_data.into_boxed_slice());
    if emoji_font_data.is_empty() {
        return None;
    }
    let emoji_raster_font = RasterFont::from_bytes(
        emoji_font_data,
        fontdue::FontSettings {
            scale: emoji_effective_font_size,
            ..fontdue::FontSettings::default()
        },
    )
    .ok()?;
    let emoji_shape_face = ShapeFace::from_slice(emoji_font_data, 0)?;
    let font = ttf.load_font(&emoji_path, emoji_effective_font_size).ok()?;
    let pixel_size = normalized_raster_pixel_size(
        emoji_effective_font_size,
        font.height().max(1),
        emoji_raster_font.horizontal_line_metrics(emoji_effective_font_size),
    );
    Some((font, emoji_raster_font, pixel_size, emoji_shape_face))
}

fn load_icon_font<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    path: &Path,
    effective_font_size: f32,
    primary_line_height: i32,
) -> Result<(String, Font<'ttf>, RasterFont, f32), ShellError> {
    let name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or("icon-font")
        .to_owned();
    let bytes = fs::read(path).map_err(|error| {
        ShellError::Runtime(format!(
            "failed to read bundled icon font `{}`: {error}",
            path.display()
        ))
    })?;
    let raster_font =
        RasterFont::from_bytes(bytes, fontdue::FontSettings::default()).map_err(|error| {
            ShellError::Runtime(format!(
                "failed to parse bundled icon font `{}`: {error}",
                path.display()
            ))
        })?;
    let font = ttf
        .load_font(path, effective_font_size)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let pixel_size = normalized_raster_pixel_size(
        effective_font_size,
        primary_line_height,
        raster_font.horizontal_line_metrics(effective_font_size),
    );
    Ok((name, font, raster_font, pixel_size))
}

fn load_icon_fonts_for_paths<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    paths: Vec<PathBuf>,
    effective_font_size: f32,
    primary_line_height: i32,
) -> Result<Vec<(String, Font<'ttf>, RasterFont, f32)>, ShellError> {
    paths
        .into_iter()
        .map(|path| load_icon_font(ttf, &path, effective_font_size, primary_line_height))
        .collect()
}

struct LoadedPrimaryFont<'ttf> {
    font: Font<'ttf>,
    synthetic_bold: bool,
}

fn load_primary_font<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    path: &Path,
    effective_font_size: f32,
    style: TextStyle,
) -> Result<LoadedPrimaryFont<'ttf>, ShellError> {
    let font_path = styled_primary_font_path(path, style);
    let synthetic_bold = style.bold && font_path == path;
    let mut font = ttf
        .load_font(&font_path, effective_font_size)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    if let Some(hinting) = preferred_primary_font_hinting() {
        font.set_hinting(hinting);
    }
    if style.italic && font_path == path {
        font.set_style(FontStyle::ITALIC);
    }
    Ok(LoadedPrimaryFont {
        font,
        synthetic_bold,
    })
}

fn styled_primary_font_path(path: &Path, style: TextStyle) -> PathBuf {
    if style == TextStyle::plain() {
        return path.to_path_buf();
    }
    styled_font_candidates(path, style)
        .into_iter()
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| path.to_path_buf())
}

fn styled_font_candidates(path: &Path, style: TextStyle) -> Vec<PathBuf> {
    let Some(parent) = path.parent() else {
        return Vec::new();
    };
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Vec::new();
    };
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    let suffix = match (style.bold, style.italic) {
        (true, true) => "BoldItalic",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (false, false) => return Vec::new(),
    };
    let mut stems = Vec::new();
    for regular_suffix in ["-Regular", " Regular", "_Regular"] {
        if let Some(base) = stem.strip_suffix(regular_suffix) {
            stems.push(format!("{base}-{suffix}"));
            stems.push(format!("{base} {suffix}"));
            stems.push(format!("{base}_{suffix}"));
        }
    }
    if style.bold && style.italic {
        for italic_suffix in ["-Italic", " Italic", "_Italic"] {
            if let Some(base) = stem.strip_suffix(italic_suffix) {
                stems.push(format!("{base}-BoldItalic"));
                stems.push(format!("{base} BoldItalic"));
                stems.push(format!("{base}_BoldItalic"));
            }
        }
    }
    stems
        .into_iter()
        .map(|stem| {
            if extension.is_empty() {
                parent.join(stem)
            } else {
                parent.join(format!("{stem}.{extension}"))
            }
        })
        .collect()
}

#[cfg(test)]
fn load_font_set<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    settings: &ThemeRuntimeSettings,
    user_library: &dyn UserLibrary,
) -> Result<(FontSet<'ttf>, PathBuf), ShellError> {
    load_font_set_with_mode(ttf, settings, user_library, OptionalFontLoadMode::Eager)
}

fn load_font_set_with_mode<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    settings: &ThemeRuntimeSettings,
    user_library: &dyn UserLibrary,
    optional_font_load_mode: OptionalFontLoadMode,
) -> Result<(FontSet<'ttf>, PathBuf), ShellError> {
    let mut startup_trace = StartupTrace::new();
    let effective_font_size = scaled_font_size(settings.font_size, settings.display_scale);
    let primary_path = resolve_font_path(settings.font_request.as_deref())?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.resolve-primary");
    }
    let primary_font_data: &'static [u8] = Box::leak(
        fs::read(&primary_path)
            .map_err(|error| {
                ShellError::Runtime(format!(
                    "failed to read primary font `{}`: {error}",
                    primary_path.display()
                ))
            })?
            .into_boxed_slice(),
    );
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.read-primary");
    }
    let primary_raster_font = RasterFont::from_bytes(
        primary_font_data,
        fontdue::FontSettings {
            scale: effective_font_size,
            ..fontdue::FontSettings::default()
        },
    )
    .map_err(|error| {
        ShellError::Runtime(format!(
            "failed to parse primary font `{}`: {error}",
            primary_path.display()
        ))
    })?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.raster-primary");
    }
    let primary_shape_face = ShapeFace::from_slice(primary_font_data, 0).ok_or_else(|| {
        ShellError::Runtime(format!(
            "failed to parse shaping data for primary font `{}`",
            primary_path.display()
        ))
    })?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.shape-primary");
    }
    let primary = load_primary_font(ttf, &primary_path, effective_font_size, TextStyle::plain())?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.load-primary");
    }
    let primary_bold = load_primary_font(
        ttf,
        &primary_path,
        effective_font_size,
        TextStyle::new(true, false),
    )?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.load-primary-bold");
    }
    let primary_italic = load_primary_font(
        ttf,
        &primary_path,
        effective_font_size,
        TextStyle::new(false, true),
    )?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.load-primary-italic");
    }
    let primary_bold_italic = load_primary_font(
        ttf,
        &primary_path,
        effective_font_size,
        TextStyle::new(true, true),
    )?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.load-primary-bold-italic");
    }
    let primary_line_height = primary.font.height().max(1);
    let primary_pixel_size = normalized_raster_pixel_size(
        effective_font_size,
        primary_line_height,
        primary_raster_font.horizontal_line_metrics(effective_font_size),
    );
    let cell_width = primary
        .font
        .size_of_char('M')
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0
        .max(1) as i32;

    let emoji_font = match optional_font_load_mode {
        OptionalFontLoadMode::StartupPrimaryOnly => None,
        OptionalFontLoadMode::Eager => load_emoji_font(ttf, settings),
    };
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.load-emoji");
    }

    let icon_chars = user_library
        .icon_symbols()
        .iter()
        .flat_map(|symbol| symbol.glyph.chars())
        .collect();
    let icon_fonts = match optional_font_load_mode {
        OptionalFontLoadMode::StartupPrimaryOnly => load_icon_fonts_for_paths(
            ttf,
            resolve_startup_icon_font_paths()?,
            effective_font_size,
            primary_line_height,
        )?,
        OptionalFontLoadMode::Eager => load_icon_fonts_for_paths(
            ttf,
            resolve_icon_font_paths()?,
            effective_font_size,
            primary_line_height,
        )?,
    };
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.load-icons");
    }
    let mut fonts = FontSet::new(FontSetInit {
        primary: primary.font,
        primary_bold: primary_bold.font,
        primary_italic: primary_italic.font,
        primary_bold_italic: primary_bold_italic.font,
        primary_bold_is_synthetic: primary_bold.synthetic_bold,
        primary_bold_italic_is_synthetic: primary_bold_italic.synthetic_bold,
        primary_raster_font,
        primary_shape_face,
        primary_pixel_size,
        emoji_font,
        ligatures_enabled: user_library.ligature_config().enabled,
        icon_fonts,
        icon_chars,
        cell_width,
    });
    if optional_font_load_mode == OptionalFontLoadMode::StartupPrimaryOnly
        && validate_bundled_icon_fonts(&fonts, user_library).is_err()
    {
        fonts = FontSet::new(FontSetInit {
            primary: fonts.primary,
            primary_bold: fonts.primary_bold,
            primary_italic: fonts.primary_italic,
            primary_bold_italic: fonts.primary_bold_italic,
            primary_bold_is_synthetic: fonts.primary_bold_is_synthetic,
            primary_bold_italic_is_synthetic: fonts.primary_bold_italic_is_synthetic,
            primary_raster_font: fonts.primary_raster_font,
            primary_shape_face: fonts.primary_shape_face,
            primary_pixel_size: fonts.primary_pixel_size,
            emoji_font: fonts.emoji_font.map(|emoji| {
                (
                    emoji.font,
                    emoji.raster_font,
                    emoji.pixel_size,
                    emoji.shape_face,
                )
            }),
            ligatures_enabled: fonts.ligatures_enabled,
            icon_fonts: load_icon_fonts_for_paths(
                ttf,
                resolve_icon_font_paths()?,
                effective_font_size,
                primary_line_height,
            )?,
            icon_chars: fonts.icon_chars,
            cell_width: fonts.cell_width,
        });
    }
    if optional_font_load_mode == OptionalFontLoadMode::Eager
        || optional_font_load_mode == OptionalFontLoadMode::StartupPrimaryOnly
    {
        validate_bundled_icon_fonts(&fonts, user_library)?;
    }
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.fonts.validate-icons");
    }
    Ok((fonts, primary_path))
}
