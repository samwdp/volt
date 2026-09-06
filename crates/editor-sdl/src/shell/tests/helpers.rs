use super::*;

#[derive(Debug, Default)]
pub(super) struct CommandLog(pub(super) Vec<String>);

pub(super) fn rust_test_language() -> editor_syntax::Language {
    tree_sitter_rust::LANGUAGE.into()
}

pub(super) fn register_rust_highlight_test_language(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    syntax_registry_mut(runtime)?
        .register(editor_syntax::LanguageConfiguration::new(
            "rust",
            ["rs"],
            rust_test_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [
                editor_syntax::CaptureThemeMapping::new("keyword", "syntax.keyword"),
                editor_syntax::CaptureThemeMapping::new("function", "syntax.function"),
            ],
        ))
        .map_err(|error| error.to_string())
}

pub(super) struct TempTestDir {
    pub(super) path: PathBuf,
}

impl TempTestDir {
    pub(super) fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        Self {
            path: env::temp_dir().join(format!("volt-shell-{name}-{unique}")),
        }
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(super) fn wait_for_buffer_syntax_refresh(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_syntax(runtime)?;
        if !shell_buffer(runtime, buffer_id)?.syntax_dirty {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    let worker_configured = shell_ui(runtime)?.syntax_refresh_worker.is_configured();
    let pending_results = shell_ui(runtime)?
        .syntax_refresh_worker
        .results
        .lock()
        .map(|results| results.len())
        .unwrap_or(usize::MAX);
    let buffer = shell_buffer(runtime, buffer_id)?;
    Err(format!(
        "syntax refresh did not complete for buffer `{}` (worker_configured={worker_configured}, pending_results={pending_results}, language_id={:?}, syntax_error={:?})",
        buffer_id.get(),
        buffer.language_id(),
        buffer.syntax_error
    ))
}

pub(super) fn sync_active_buffer_layout_for_test(state: &mut ShellState) -> Result<(), String> {
    const RENDER_WIDTH: u32 = 960;
    const RENDER_HEIGHT: u32 = 640;
    const CELL_WIDTH: i32 = 8;
    const LINE_HEIGHT: i32 = 16;

    state
        .sync_active_viewport_for_render_size(RENDER_WIDTH, RENDER_HEIGHT, LINE_HEIGHT)
        .map_err(|error| error.to_string())?;
    state
        .sync_visible_buffer_layouts(RENDER_WIDTH, RENDER_HEIGHT, CELL_WIDTH, LINE_HEIGHT)
        .map_err(|error| error.to_string())
}

pub(super) struct HeaderlineTestUserLibrary {
    pub(super) scrolloff: f64,
    pub(super) headerline_lines: Vec<String>,
    pub(super) headerline_requires_scrolled_viewport: bool,
    pub(super) headerline_call_count: Arc<AtomicUsize>,
    pub(super) pdf_open_mode: PdfOpenMode,
    pub(super) markdown_pretty: MarkdownPrettyConfig,
}

impl Default for HeaderlineTestUserLibrary {
    fn default() -> Self {
        Self {
            scrolloff: 1.0,
            headerline_lines: vec!["fn render(value: usize)".to_owned()],
            headerline_requires_scrolled_viewport: false,
            headerline_call_count: Arc::new(AtomicUsize::new(0)),
            pdf_open_mode: PdfOpenMode::Rendered,
            markdown_pretty: MarkdownPrettyConfig::default(),
        }
    }
}

impl HeaderlineTestUserLibrary {
    pub(super) fn with_scrolloff(scrolloff: f64) -> Self {
        Self {
            scrolloff,
            ..Self::default()
        }
    }

    pub(super) fn with_pdf_open_mode(pdf_open_mode: PdfOpenMode) -> Self {
        Self {
            pdf_open_mode,
            ..Self::default()
        }
    }

    pub(super) fn headerline_call_count(&self) -> usize {
        self.headerline_call_count.load(Ordering::Relaxed)
    }
}

impl UserLibrary for HeaderlineTestUserLibrary {
    fn packages(&self) -> Vec<editor_plugin_api::PluginPackage> {
        Vec::new()
    }

    fn themes(&self) -> Vec<Theme> {
        vec![Theme::new("default", "Default").with_option("scrolloff", self.scrolloff)]
    }

    fn syntax_languages(&self) -> Vec<LanguageConfiguration> {
        Vec::new()
    }

    fn language_servers(&self) -> Vec<LanguageServerSpec> {
        Vec::new()
    }

    fn debug_adapters(&self) -> Vec<DebugAdapterSpec> {
        Vec::new()
    }

    fn autocomplete_providers(&self) -> Vec<AutocompleteProvider> {
        Vec::new()
    }

    fn autocomplete_result_limit(&self) -> usize {
        8
    }

    fn autocomplete_token_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_SYMBOL_MISC
    }

    fn hover_providers(&self) -> Vec<HoverProvider> {
        Vec::new()
    }

    fn hover_line_limit(&self) -> usize {
        10
    }

    fn hover_token_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_INFO
    }

    fn hover_signature_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_SIGNATURE
    }

    fn acp_clients(&self) -> Vec<AcpClient> {
        Vec::new()
    }

    fn acp_client_by_id(&self, _id: &str) -> Option<AcpClient> {
        None
    }

    fn workspace_roots(&self) -> Vec<WorkspaceRoot> {
        Vec::new()
    }

    fn terminal_config(&self) -> TerminalConfig {
        TerminalConfig {
            program: "powershell.exe".to_owned(),
            args: vec!["-NoLogo".to_owned()],
        }
    }

    fn commandline_enabled(&self) -> bool {
        true
    }

    fn markdown_pretty_config(&self) -> MarkdownPrettyConfig {
        self.markdown_pretty.clone()
    }

    fn pane_config(&self) -> PaneConfig {
        PaneConfig {
            golden_ratio: false,
        }
    }

    fn ligature_config(&self) -> LigatureConfig {
        LigatureConfig { enabled: false }
    }

    fn rainbow_parens_config(&self) -> RainbowParensConfig {
        RainbowParensConfig { enabled: true }
    }

    fn show_paren_config(&self) -> editor_plugin_api::ShowParenConfig {
        editor_plugin_api::ShowParenConfig { enabled: true }
    }

    fn oil_defaults(&self) -> OilDefaults {
        OilDefaults {
            show_hidden: false,
            sort_mode: editor_plugin_api::OilSortMode::TypeThenName,
            trash_enabled: false,
        }
    }

    fn oil_keybindings(&self) -> OilKeybindings {
        OilKeybindings {
            open_entry: "Enter",
            open_vertical_split: "s",
            open_horizontal_split: "S",
            open_new_pane: "p",
            preview_entry: "-",
            refresh: "gr",
            close: "q",
            prefix: "g",
            open_parent: "..",
            open_workspace_root: "~",
            set_root: "cd",
            show_help: "?",
            cycle_sort: "gs",
            toggle_hidden: "gh",
            toggle_trash: "gt",
            open_external: "gx",
            set_tab_local_root: "gl",
            create_git_worktree: "gwn",
        }
    }

    fn oil_keydown_action(&self, _chord: &str) -> Option<OilKeyAction> {
        None
    }

    fn oil_chord_action(&self, _had_prefix: bool, _chord: &str) -> Option<OilKeyAction> {
        None
    }

    fn oil_help_lines(&self) -> Vec<String> {
        Vec::new()
    }

    fn oil_directory_sections(
        &self,
        _root: &std::path::Path,
        _entries: &[editor_fs::DirectoryEntry],
        _show_hidden: bool,
        _sort_mode: editor_plugin_api::OilSortMode,
        _trash_enabled: bool,
    ) -> editor_core::SectionTree {
        editor_core::SectionTree::default()
    }

    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str {
        label
    }

    fn git_status_sections(
        &self,
        _snapshot: &editor_git::GitStatusSnapshot,
    ) -> editor_core::SectionTree {
        editor_core::SectionTree::default()
    }

    fn git_commit_template(&self, _snapshot: &editor_git::GitStatusSnapshot) -> Vec<String> {
        Vec::new()
    }

    fn git_prefix_for_chord(&self, _chord: &str) -> Option<editor_plugin_api::GitStatusPrefix> {
        None
    }

    fn git_command_for_chord(
        &self,
        _prefix: Option<editor_plugin_api::GitStatusPrefix>,
        _chord: &str,
    ) -> Option<&'static str> {
        None
    }

    fn browser_buffer_lines(&self, _url: Option<&str>) -> Vec<String> {
        Vec::new()
    }

    fn browser_input_hint(&self, _url: Option<&str>) -> String {
        String::new()
    }

    fn browser_url_prompt(&self) -> String {
        "URL > ".to_owned()
    }

    fn browser_url_placeholder(&self) -> String {
        "https://example.com".to_owned()
    }

    fn pdf_open_mode(&self) -> PdfOpenMode {
        self.pdf_open_mode
    }

    fn headerline_lines(&self, context: &GhostTextContext<'_>) -> Vec<String> {
        self.headerline_call_count.fetch_add(1, Ordering::Relaxed);
        if self.headerline_requires_scrolled_viewport && context.viewport_top_line == 0 {
            return Vec::new();
        }
        self.headerline_lines.clone()
    }

    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String {
        format!(
            " {} | {}:{} ",
            context.buffer_name, context.line, context.column
        )
    }

    fn statusline_lsp_connected_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_LAN_CONNECT
    }

    fn statusline_lsp_error_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_ERROR
    }

    fn statusline_lsp_warning_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_WARNING
    }

    fn lsp_diagnostic_icon(&self) -> &'static str {
        "●"
    }

    fn lsp_diagnostic_line_limit(&self) -> usize {
        8
    }

    fn lsp_show_buffer_diagnostics(&self) -> bool {
        true
    }

    fn gitfringe_token_added(&self) -> &'static str {
        "git.fringe.added"
    }

    fn gitfringe_token_modified(&self) -> &'static str {
        "git.fringe.modified"
    }

    fn gitfringe_token_removed(&self) -> &'static str {
        "git.fringe.removed"
    }

    fn gitfringe_symbol(&self) -> &'static str {
        "⏽"
    }

    fn icon_symbols(&self) -> &'static [editor_icons::IconFontSymbol] {
        editor_icons::all_symbols()
    }

    fn run_plugin_buffer_evaluator(&self, _handler: &str, _input: &str) -> Vec<String> {
        Vec::new()
    }

    fn plugin_buffer(&self, _kind: &str) -> Option<PluginBuffer> {
        None
    }

    fn plugin_buffer_sections(&self, _kind: &str) -> Option<PluginBufferSections> {
        None
    }

    fn default_build_command(&self, _language: &str) -> Option<String> {
        None
    }
}

pub(super) fn slice_by_columns(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

pub(super) fn syntax_span_segments(line: &str, spans: &[LineSyntaxSpan]) -> Vec<(String, String)> {
    spans
        .iter()
        .map(|span| {
            (
                span.theme_token.to_string(),
                slice_by_columns(line, span.start, span.end),
            )
        })
        .collect()
}

pub(super) fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!(
        "volt-shell-tests-{label}-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to create temp dir `{}`: {error}", dir.display()));
    dir
}

pub(super) fn write_test_png(path: &Path) -> Result<(), String> {
    let image = image::RgbaImage::from_pixel(40, 20, image::Rgba([255, 0, 0, 255]));
    image.save(path).map_err(|error| error.to_string())
}

pub(super) fn write_test_svg(path: &Path) -> Result<(), String> {
    std::fs::write(
        path,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20" viewBox="0 0 40 20">
  <rect width="40" height="20" fill="#1f6feb"/>
  <circle cx="10" cy="10" r="6" fill="#f2cc60"/>
</svg>"##,
    )
    .map_err(|error| error.to_string())
}

pub(super) fn write_test_pdf(path: &Path, page_texts: &[&str]) -> Result<(), String> {
    use lopdf::content::{Content, Operation};
    use lopdf::dictionary;
    use lopdf::{Document as PdfDocument, Object as PdfObject, Stream};

    let mut document = PdfDocument::with_version("1.5");
    let info_id = document.add_object(lopdf::dictionary! {
        "Title" => PdfObject::string_literal("Volt PDF Test"),
        "Creator" => PdfObject::string_literal("volt"),
    });
    let pages_id = document.new_object_id();
    let font_id = document.add_object(lopdf::dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    });
    let resources_id = document.add_object(lopdf::dictionary! {
        "Font" => lopdf::dictionary! {
            "F1" => font_id,
        },
    });
    let pages = page_texts.iter().map(|text| {
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 24.into()]),
                Operation::new("Td", vec![72.into(), 720.into()]),
                Operation::new("Tj", vec![PdfObject::string_literal(*text)]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = document.add_object(Stream::new(
            lopdf::dictionary! {},
            content.encode().map_err(|error| error.to_string())?,
        ));
        Ok::<PdfObject, String>(
            document
                .add_object(lopdf::dictionary! {
                    "Type" => "Page",
                    "Parent" => pages_id,
                    "Contents" => content_id,
                })
                .into(),
        )
    });
    let kids = pages.collect::<Result<Vec<_>, _>>()?;
    document.objects.insert(
        pages_id,
        PdfObject::Dictionary(lopdf::dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => page_texts.len() as i64,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        }),
    );
    let catalog_id = document.add_object(lopdf::dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    document.trailer.set("Root", catalog_id);
    document.trailer.set("Info", info_id);
    document.compress();
    document.save(path).map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) const MATERIAL_ICONS_FONT: &[u8] = include_bytes!(concat!(
    core::env!("CARGO_MANIFEST_DIR"),
    "/../volt/assets/font/material-design-icons.ttf"
));

pub(super) fn berkeley_mono_font() -> Option<&'static [u8]> {
    static BERKELEY_MONO_FONT: std::sync::OnceLock<Option<Box<[u8]>>> = std::sync::OnceLock::new();
    BERKELEY_MONO_FONT
        .get_or_init(|| {
            let path = std::path::Path::new(core::env!("CARGO_MANIFEST_DIR"))
                .join("../../LigaBerkeleyMono-Regular.ttf");
            std::fs::read(path).ok().map(Vec::into_boxed_slice)
        })
        .as_deref()
}

pub(super) const BERKELEY_MONO_TEST_CELL_WIDTH: i32 = 11;

pub(super) fn berkeley_mono_ligature_test_assets() -> Option<(ShapeFace<'static>, RasterFont)> {
    let bytes = berkeley_mono_font()?;
    Some((
        ShapeFace::from_slice(bytes, 0)?,
        RasterFont::from_bytes(bytes, fontdue::FontSettings::default()).ok()?,
    ))
}

pub(super) fn configure_file_buffer(
    state: &mut ShellState,
    buffer_id: BufferId,
    path: &Path,
) -> Result<(), String> {
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        buffer.kind = BufferKind::File;
        buffer.name = path.display().to_string();
        buffer.text = TextBuffer::load_from_path(path).map_err(|error| error.to_string())?;
        buffer.backing_file_fingerprint = BackingFileFingerprint::read(path).ok();
        buffer.backing_file_reload_pending = false;
        buffer.backing_file_check_in_flight = false;
    }
    shell_ui_mut(&mut state.runtime)?
        .file_reload_worker
        .watch_path(path.to_path_buf());
    Ok(())
}

pub(super) fn active_and_secondary_buffer_ids(
    runtime: &EditorRuntime,
) -> Result<(BufferId, BufferId), String> {
    let ui = shell_ui(runtime)?;
    let active_buffer_id = ui
        .active_buffer_id()
        .ok_or_else(|| "active buffer is missing".to_owned())?;
    let secondary_buffer_id = ui
        .active_workspace_buffer_ids()
        .and_then(|buffer_ids| {
            buffer_ids
                .iter()
                .copied()
                .find(|buffer_id| *buffer_id != active_buffer_id)
        })
        .ok_or_else(|| "secondary buffer is missing".to_owned())?;
    Ok((active_buffer_id, secondary_buffer_id))
}

pub(super) fn wait_for_file_reload_worker(
    state: &mut ShellState,
    buffer_ids: &[BufferId],
) -> Result<(), String> {
    for _ in 0..200 {
        let _ = refresh_pending_file_reloads(&mut state.runtime, Instant::now(), false)?;
        if buffer_ids.iter().copied().all(|buffer_id| {
            shell_buffer(&state.runtime, buffer_id)
                .map(|buffer| !buffer.backing_file_check_in_flight)
                .unwrap_or(true)
        }) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("timed out waiting for the file reload worker".to_owned())
}

pub(super) fn wait_for_file_reload_change(state: &mut ShellState) -> Result<bool, String> {
    for _ in 0..200 {
        if refresh_pending_file_reloads(&mut state.runtime, Instant::now(), false)? {
            return Ok(true);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(false)
}

pub(super) fn record_file_reload_event(state: &ShellState, path: &Path) -> Result<(), String> {
    shell_ui(&state.runtime)?
        .file_reload_worker
        .record_changed_path_for_test(path.to_path_buf());
    Ok(())
}

pub(super) fn install_test_lsp_manager(
    runtime: &mut EditorRuntime,
    server_ids: &[&str],
) -> Result<Arc<LspClientManager>, String> {
    let mut registry = LanguageServerRegistry::new();
    for server_id in server_ids {
        registry
            .register(editor_lsp::LanguageServerSpec::new(
                *server_id,
                "rust",
                ["rs"],
                "dummy-lsp",
                std::iter::empty::<&str>(),
            ))
            .map_err(|error| error.to_string())?;
    }
    let manager = Arc::new(LspClientManager::new(registry));
    runtime.services_mut().insert(Arc::clone(&manager));
    Ok(manager)
}

pub(super) fn install_lsp_enabled_file_buffer(
    state: &mut ShellState,
    name: &str,
    path: &Path,
    lines: Vec<String>,
) -> Result<BufferId, String> {
    let buffer_id = install_text_test_buffer(state, name, lines)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_path(path.to_path_buf());
        buffer.set_lsp_enabled(true);
        buffer.set_lsp_path(Some(path.to_path_buf()));
    }
    Ok(buffer_id)
}

pub(super) fn sample_lsp_diagnostic(message: &str) -> Diagnostic {
    Diagnostic::new(
        "rustc",
        message,
        DiagnosticSeverity::Error,
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 4)),
    )
}

pub(super) fn assert_wrap_cache_matches_cold_build(
    buffer: &ShellBuffer,
    wrap_cols: usize,
    indent_size: usize,
) -> Result<(), String> {
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was missing".to_owned())?;
    let cold = WrapRowCache::build(buffer, wrap_cols, indent_size);
    assert_eq!(cache.line_count, cold.line_count);
    assert_eq!(cache.prefix_rows, cold.prefix_rows);
    assert_eq!(cache.wrap_cols, wrap_cols);
    assert_eq!(cache.indent_size, indent_size);
    Ok(())
}

pub(super) fn install_acp_test_buffer(
    state: &mut ShellState,
    output_lines: usize,
    input_text: &str,
    hint: Option<&str>,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*acp test*",
            BufferKind::Plugin(ACP_BUFFER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP test buffer is missing".to_owned())?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, Vec::new(), &NullUserLibrary);
    shell_buffer.init_acp_view("Test ACP");
    for index in 1..=output_lines {
        shell_buffer.acp_push_system_message(format!("line {index}"));
    }
    if let Some(input) = shell_buffer.input_field_mut() {
        input.set_text(input_text);
    }
    if let Some(footer) = shell_buffer.acp_footer_pane_mut() {
        footer.replace_lines(hint.into_iter().map(str::to_owned).collect(), true);
    }
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

pub(super) fn state_with_user_library() -> Result<ShellState, String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
    ShellState::new_with_user_library(default_error_log_path(), false, user_library)
        .map_err(|error| error.to_string())
}

pub(super) fn focus_input_normal_mode(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn install_user_plugin_buffer(
    state: &mut ShellState,
    name: &str,
    kind: &str,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            name,
            BufferKind::Plugin(kind.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    sync_active_buffer(&mut state.runtime)?;
    Ok(buffer_id)
}

pub(super) fn install_plugin_sections_test_buffer(
    state: &mut ShellState,
    input_lines: &[&str],
    output_lines: &[&str],
) -> Result<BufferId, String> {
    install_plugin_sections_test_buffer_with_update(
        state,
        input_lines,
        output_lines,
        editor_plugin_api::PluginBufferSectionUpdate::Replace,
    )
}

pub(super) fn install_plugin_sections_test_buffer_with_update(
    state: &mut ShellState,
    input_lines: &[&str],
    output_lines: &[&str],
    update: editor_plugin_api::PluginBufferSectionUpdate,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*calculator test*",
            BufferKind::Plugin(buffer_kinds::CALCULATOR.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "plugin test buffer is missing".to_owned())?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(
        buffer,
        input_lines.iter().map(|line| (*line).to_owned()).collect(),
        &NullUserLibrary,
    );
    let output = if output_lines.is_empty() {
        vec!["(press Ctrl+c Ctrl+c to evaluate)".to_owned()]
    } else {
        output_lines.iter().map(|line| (*line).to_owned()).collect()
    };
    shell_buffer.plugin_section_state = PluginSectionBufferState::new(
        PluginBufferSections::new(vec![
            editor_plugin_api::PluginBufferSection::new("Input")
                .with_writable(true)
                .with_initial_lines(input_lines.iter().map(|line| (*line).to_owned()).collect()),
            editor_plugin_api::PluginBufferSection::new("Output")
                .with_min_lines(1)
                .with_initial_lines(output)
                .with_update(update),
        ]),
        Some("Output"),
    );
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

pub(super) fn plugin_section_lines(
    buffer: &ShellBuffer,
    name: &str,
) -> Result<Vec<String>, String> {
    let state = buffer
        .plugin_sections()
        .ok_or_else(|| "plugin section state missing".to_owned())?;
    let index = state
        .section_index_by_name(name)
        .ok_or_else(|| format!("section `{name}` missing"))?;
    if index == 0 {
        return Ok((0..buffer.text.line_count())
            .filter_map(|line_index| buffer.text.line(line_index))
            .collect());
    }
    let pane = state
        .attached_section(index)
        .ok_or_else(|| format!("attached section `{name}` missing"))?;
    Ok((0..pane.line_count())
        .map(|line_index| pane.text.line(line_index).unwrap_or_default())
        .collect())
}

pub(super) fn install_user_acp_test_buffer(
    state: &mut ShellState,
    input_text: &str,
) -> Result<BufferId, String> {
    let buffer_id = install_user_plugin_buffer(state, "*acp*", user::acp::ACP_BUFFER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.init_acp_view("Test ACP");
        let _ = buffer.focus_acp_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .set_text(input_text);
    }
    Ok(buffer_id)
}

pub(super) fn install_scratch_test_buffer(
    state: &mut ShellState,
    name: &str,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, name, BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        name,
        BufferKind::Scratch,
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)?;
    Ok(buffer_id)
}

pub(super) fn install_markdown_test_buffer(
    state: &mut ShellState,
    name: &str,
    text: &str,
) -> Result<BufferId, String> {
    let buffer_id = install_scratch_test_buffer(state, name)?;
    let lines = if text.is_empty() {
        Vec::new()
    } else {
        text.lines().map(str::to_owned).collect()
    };
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(lines);
        buffer.set_language_id(Some("markdown".to_owned()));
    }
    sync_active_buffer(&mut state.runtime)?;
    Ok(buffer_id)
}

pub(super) const PRETTY_CACHE_FIXTURE: &str = "# Title\n- item\nplain\n";

pub(super) fn markdown_pretty_paint_args(buffer: &ShellBuffer) -> MarkdownPrettyPaintArgs {
    MarkdownPrettyPaintArgs {
        visible_start: 0,
        visible_end: buffer.line_count().max(1),
        visual_selection: None,
        input_mode: InputMode::Normal,
        pane_width_px: 640,
        line_height: 16,
    }
}

pub(super) fn park_cursor_on_plain_pretty_line(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 0));
    Ok(())
}

pub(super) fn markdown_table_event_dimensions() -> (u32, u32, i32, i32) {
    (640, 240, 8, 16)
}

pub(super) fn focus_test_buffer(state: &mut ShellState, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)
}

pub(super) fn install_browser_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            BROWSER_BUFFER_NAME,
            BufferKind::Plugin(BROWSER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        BROWSER_BUFFER_NAME,
        BufferKind::Plugin(BROWSER_KIND.to_owned()),
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

pub(super) fn install_terminal_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, "*terminal*", BufferKind::Terminal, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        "*terminal*",
        BufferKind::Terminal,
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

pub(super) fn install_terminal_popup_test_buffer(
    state: &mut ShellState,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*terminal-popup*", BufferKind::Terminal, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Terminal", vec![buffer_id], buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_popup_buffer(
        buffer_id,
        "*terminal-popup*",
        BufferKind::Terminal,
        &NullUserLibrary,
    );
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_popup_buffer(buffer_id);
        ui.set_popup_focus(true);
    }
    Ok(buffer_id)
}

pub(super) fn install_git_status_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*git-status*",
            BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        "*git-status*",
        BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

pub(super) fn run_git_in_dir(root: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run git {:?}: {error}", args))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if stderr.is_empty() {
            format!("git {:?} failed with status {}", args, output.status)
        } else {
            format!("git {:?} failed: {stderr}", args)
        };
        return Err(message);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(super) fn init_git_repo(label: &str) -> Result<std::path::PathBuf, String> {
    let repo = unique_temp_dir(label);
    run_git_in_dir(&repo, &["init", "-q"])?;
    run_git_in_dir(&repo, &["config", "user.email", "volt-tests@example.com"])?;
    run_git_in_dir(&repo, &["config", "user.name", "Volt Tests"])?;
    run_git_in_dir(&repo, &["config", "commit.gpgsign", "false"])?;
    Ok(repo)
}

pub(super) fn init_git_repo_with_commit(label: &str) -> Result<std::path::PathBuf, String> {
    let repo = init_git_repo(label)?;
    std::fs::write(repo.join("README.md"), "seed\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "README.md"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "initial"])?;
    Ok(repo)
}

pub(super) fn install_git_hook(
    repo: &std::path::Path,
    hook_name: &str,
    script: &str,
) -> Result<(), String> {
    let hook_path = repo.join(".git").join("hooks").join(hook_name);
    std::fs::write(&hook_path, script).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        let mut permissions = std::fs::metadata(&hook_path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&hook_path, permissions).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(super) fn open_repo_git_status_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "git-test", root)?;
    let buffer_id = install_git_status_test_buffer(state)?;
    refresh_git_status_buffer(&mut state.runtime, buffer_id)?;
    Ok(buffer_id)
}

pub(super) fn wait_for_streamed_command_output_line(
    state: &mut ShellState,
    buffer_id: BufferId,
    needle: &str,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        let matched = shell_ui(&state.runtime)?
            .buffer(buffer_id)
            .is_some_and(|buffer| {
                (0..buffer.line_count()).any(|line_index| {
                    buffer
                        .text
                        .line(line_index)
                        .unwrap_or_default()
                        .contains(needle)
                })
            });
        if tracked && matched {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "streamed command buffer `{buffer_id}` never emitted `{needle}` while running"
    ))
}

pub(super) fn wait_for_streamed_command_buffer_close(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        let buffered = shell_ui(&state.runtime)?.buffer(buffer_id).is_some();
        if !tracked && !buffered {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    let tracked = terminal_buffer_state(&state.runtime)?.contains(buffer_id);
    let worker_tracked = shell_ui(&state.runtime)?
        .streamed_command_worker
        .contains(buffer_id);
    let buffered = shell_ui(&state.runtime)?.buffer(buffer_id).is_some();
    let popup_visible = active_runtime_popup(&state.runtime)?.is_some();
    Err(format!(
        "temporary streamed command buffer `{buffer_id}` did not close in time (terminal_tracked={tracked}, worker_tracked={worker_tracked}, buffered={buffered}, popup_visible={popup_visible})"
    ))
}

pub(super) fn open_oil_test_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "oil-test", root)?;
    open_oil_directory(&mut state.runtime, root.to_path_buf())?;
    active_shell_buffer_id(&state.runtime)
}

pub(super) fn oil_line_index_containing(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    needle: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|&index| buffer.text.line(index).unwrap_or_default().contains(needle))
        .ok_or_else(|| format!("oil buffer is missing line containing `{needle}`"))
}

pub(super) fn oil_line_index_for_entry_path(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    path: &Path,
) -> Result<usize, String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|&index| {
            buffer
                .section_line_meta(index)
                .and_then(|meta| meta.action.as_ref())
                .filter(|action| action.id() == editor_plugin_api::oil_protocol::ACTION_OIL_ENTRY)
                .and_then(|action| action.detail())
                .is_some_and(|detail| Path::new(detail) == path)
        })
        .ok_or_else(|| format!("oil buffer is missing entry `{}`", path.display()))
}

pub(super) fn oil_type_new_entry_and_leave_insert(
    state: &mut ShellState,
    entry: &str,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let last_line = shell_buffer(&state.runtime, buffer_id)?
        .line_count()
        .saturating_sub(1);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(last_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    if shell_ui(&state.runtime)?.input_mode() != InputMode::Insert {
        return Err(format!(
            "expected insert mode after o, got {:?}",
            shell_ui(&state.runtime)?.input_mode()
        ));
    }
    state
        .handle_text_input(entry)
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn install_text_test_buffer(
    state: &mut ShellState,
    name: &str,
    lines: Vec<String>,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, name, BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "text test buffer is missing".to_owned())?;
    let shell_buffer = ShellBuffer::from_runtime_buffer(buffer, lines, &NullUserLibrary);
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

pub(super) fn screen_point_for_buffer_point(
    state: &mut ShellState,
    buffer_id: BufferId,
    point: TextPoint,
    render_width: u32,
    render_height: u32,
    cell_width: i32,
    line_height: i32,
) -> Result<(f32, f32), String> {
    let original_cursor = shell_buffer(&state.runtime, buffer_id)?.cursor_point();
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(point);
    let anchor = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer_cursor_screen_anchor(
            buffer,
            PixelRectToRect::rect(0, 0, render_width, render_height),
            &*shell_user_library(&state.runtime),
            state.runtime.services().get::<ThemeRegistry>(),
            cell_width,
            line_height,
            false,
        )
        .ok_or_else(|| "buffer cursor screen anchor was missing".to_owned())?
    };
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(original_cursor);
    Ok((
        (anchor.x + (cell_width / 2).max(1)) as f32,
        (anchor.y + (line_height / 2).max(1)) as f32,
    ))
}

pub(super) fn git_status_line_for_action_detail(
    state: &ShellState,
    buffer_id: BufferId,
    action_id: &str,
    detail: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|line_index| {
            git_action_detail(buffer.section_line_meta(*line_index), action_id).as_deref()
                == Some(detail)
        })
        .ok_or_else(|| format!("git status line for `{detail}` and `{action_id}` was not found"))
}

pub(super) fn git_status_header_line(
    state: &ShellState,
    buffer_id: BufferId,
    section_id: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|line_index| {
            buffer.section_line_meta(*line_index).is_some_and(|meta| {
                meta.section_id == section_id
                    && matches!(meta.kind, SectionRenderLineKind::Header { .. })
            })
        })
        .ok_or_else(|| format!("git status header line for section `{section_id}` was not found"))
}

pub(super) fn set_git_status_visual_line_selection(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    let (start_line, end_line) = if start_line <= end_line {
        (start_line, end_line)
    } else {
        (end_line, start_line)
    };
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(start_line, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(end_line, 0));
    Ok(())
}

pub(super) fn set_git_status_visual_block_selection_with_ctrl_v(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    assert!(
        state
            .try_runtime_keybinding(Keycode::V, ctrl_mod())
            .map_err(|error| error.to_string())?
    );

    state
        .handle_text_input("v")
        .map_err(|error| error.to_string())?;

    let motion = if end_line >= start_line { "j" } else { "k" };
    for _ in 0..start_line.abs_diff(end_line) {
        state
            .handle_text_input(motion)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Visual);
    assert_eq!(
        shell_ui(&state.runtime)?.vim().visual_kind,
        VisualSelectionKind::Block
    );
    Ok(())
}

pub(super) fn set_git_status_visual_line_selection_with_shift_v(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;

    let motion = if end_line >= start_line { "j" } else { "k" };
    for _ in 0..start_line.abs_diff(end_line) {
        state
            .handle_text_input(motion)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Visual);
    assert_eq!(
        shell_ui(&state.runtime)?.vim().visual_kind,
        VisualSelectionKind::Line
    );
    Ok(())
}

pub(super) type GitSnapshotPaths = (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>);

pub(super) fn git_status_snapshot_paths(
    state: &ShellState,
    buffer_id: BufferId,
) -> Result<GitSnapshotPaths, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let snapshot = buffer
        .git_snapshot()
        .ok_or_else(|| "git snapshot missing".to_owned())?;
    let staged = snapshot
        .staged()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect();
    let unstaged = snapshot
        .unstaged()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect();
    let untracked = snapshot.untracked().iter().cloned().collect();
    Ok((staged, unstaged, untracked))
}

pub(super) fn install_hover_test_overlay(
    state: &mut ShellState,
    focused: bool,
) -> Result<BufferId, String> {
    let buffer_id = shell_ui(&state.runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let anchor = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    shell_ui_mut(&mut state.runtime)?.set_hover(HoverOverlay {
        buffer_id,
        anchor,
        token: "hover".to_owned(),
        providers: vec![
            HoverProviderContent {
                provider_label: "Alpha".to_owned(),
                provider_icon: "A".to_owned(),
                lines: vec!["first".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
            HoverProviderContent {
                provider_label: "Beta".to_owned(),
                provider_icon: "B".to_owned(),
                lines: vec!["second".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
            HoverProviderContent {
                provider_label: "Gamma".to_owned(),
                provider_icon: "G".to_owned(),
                lines: vec!["third".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
        ],
        provider_index: 0,
        scroll_offset: 0,
        focused,
        line_limit: 8,
        pending_g_prefix: false,
        count: None,
    });
    Ok(buffer_id)
}

pub(super) fn install_scrollable_hover_test_overlay(
    state: &mut ShellState,
    focused: bool,
) -> Result<BufferId, String> {
    let buffer_id = shell_ui(&state.runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let anchor = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    let lines = (1..=12)
        .map(|line| format!("Line {line}"))
        .collect::<Vec<_>>();
    shell_ui_mut(&mut state.runtime)?.set_hover(HoverOverlay {
        buffer_id,
        anchor,
        token: "hover".to_owned(),
        providers: vec![HoverProviderContent {
            provider_label: "Scrollable".to_owned(),
            provider_icon: "S".to_owned(),
            lines,
            syntax_lines: BTreeMap::new(),
        }],
        provider_index: 0,
        scroll_offset: 0,
        focused,
        line_limit: 4,
        pending_g_prefix: false,
        count: None,
    });
    Ok(buffer_id)
}

pub(super) fn hover_scroll_offset(state: &ShellState) -> Result<usize, String> {
    shell_ui(&state.runtime)?
        .hover()
        .map(|hover| hover.scroll_offset)
        .ok_or_else(|| "hover overlay missing".to_owned())
}

pub(super) fn test_notification_update(
    key: &str,
    severity: NotificationSeverity,
    title: &str,
    body_lines: &[&str],
    progress: Option<u8>,
    active: bool,
) -> NotificationUpdate {
    NotificationUpdate {
        key: key.to_owned(),
        severity,
        title: title.to_owned(),
        body_lines: body_lines.iter().map(|line| (*line).to_owned()).collect(),
        progress: progress.map(|percentage| NotificationProgress {
            percentage: Some(percentage),
        }),
        active,
        action: None,
        workspace_id: None,
    }
}

pub(super) fn render_shell_state_scene_with_docked_runtime_popup(
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(Vec<DrawCommand>, Rect), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_popup_test_buffer(&mut state)?;
    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "runtime popup was not opened".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let width = 320;
    let height = 180;
    let cell_width = 8;
    let line_height = 16;
    let popup_height = popup_window_height(height, line_height);
    let popup_rect = PixelRectToRect::rect(
        0,
        height.saturating_sub(popup_height) as i32,
        width,
        popup_height,
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        Some(&popup),
        ShellDockEntries {
            workspace: &[],
            acp: &[],
        },
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize { width, height },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width,
                line_height,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    Ok((scene, popup_rect))
}

pub(super) fn render_shell_state_scene_with_notification_overlay(
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(Vec<DrawCommand>, Rect), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let now = Instant::now();
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "toast".to_owned(),
            severity: NotificationSeverity::Info,
            title: "Overlay".to_owned(),
            body_lines: vec!["Readability check".to_owned()],
            progress: None,
            active: true,
            action: None,
            workspace_id: None,
        },
        now,
    );
    let ui = shell_ui(&state.runtime)?;
    let width = 320;
    let height = 180;
    let cell_width = 8;
    let line_height = 16;
    let rect = notification_overlay_layouts(
        &ui.visible_notifications(now),
        width,
        height,
        cell_width,
        line_height,
    )
    .first()
    .map(|layout| layout.rect)
    .ok_or_else(|| "notification overlay was not created".to_owned())?;

    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        ShellDockEntries {
            workspace: &[],
            acp: &[],
        },
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize { width, height },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width,
                line_height,
                ascent: 12,
            },
            pulse: FramePulse {
                now,
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    Ok((scene, rect))
}

pub(super) fn load_nfm_raster_font() -> Result<RasterFont, String> {
    let font_path = resolve_bundled_icon_font_dir()
        .map_err(|error| error.to_string())?
        .join("NFM.ttf");
    let bytes = fs::read(&font_path).map_err(|error| error.to_string())?;
    RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())
}

pub(super) fn buffer_autocomplete_request(
    buffer_id: BufferId,
    buffer: &TextBuffer,
    query: AutocompleteQuery,
    token_edits_from: Option<u64>,
    token_edits: Option<Vec<TextEdit>>,
) -> AutocompleteWorkerRequest {
    AutocompleteWorkerRequest {
        request_id: 1,
        buffer_id,
        buffer_revision: buffer.revision(),
        text: buffer.snapshot(),
        plugin_kind: None,
        db_candidates: Vec::new(),
        path: None,
        root: None,
        cursor: buffer.cursor(),
        query,
        providers: vec![AutocompleteProviderSpec {
            id: "buffer".to_owned(),
            label: "Buffer".to_owned(),
            icon: "B".to_owned(),
            item_icon: "T".to_owned(),
            or_group: None,
            buffer_kind: None,
            items: Vec::new(),
            kind: AutocompleteProviderKind::Buffer,
        }],
        lsp_client: None,
        edits: None,
        token_edits_from,
        token_edits,
    }
}

pub(super) fn run_gcc_comment_toggle(state: &mut ShellState) -> Result<(), String> {
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())
}

pub(super) fn seed_worktree_remove_one_shot(
    runtime: &mut EditorRuntime,
    path: &Path,
) -> Result<(), String> {
    let path_text = path.display().to_string();
    shell_ui_mut(runtime)?.set_picker_one_shot(PickerOneShotContext::new(
        Some(PickerSelectedRow::new(
            path_text.clone(),
            "worktree",
            Some(path_text),
        )),
        Vec::new(),
    ));
    Ok(())
}

pub(super) fn unique_sibling_path(anchor: &Path, label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    anchor.parent().unwrap_or(anchor).join(format!(
        "volt-shell-tests-{label}-{}-{unique}",
        std::process::id()
    ))
}

pub(super) fn add_linked_worktree(
    main: &Path,
    label: &str,
    branch: &str,
) -> Result<PathBuf, String> {
    let worktree = unique_sibling_path(main, label);
    run_git_in_dir(main, &["branch", "-q", branch])?;
    let path_arg = worktree
        .to_str()
        .ok_or_else(|| format!("non-utf8 worktree path `{}`", worktree.display()))?;
    run_git_in_dir(main, &["worktree", "add", "-q", path_arg, branch])?;
    Ok(worktree)
}

pub(super) fn wait_for_streamed_notification_title(
    state: &mut ShellState,
    needle: &str,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let visible = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
        if visible.iter().any(|entry| entry.title.contains(needle)) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "notification title containing `{needle}` never appeared"
    ))
}

pub(super) fn open_workspace_dashboard(runtime: &mut EditorRuntime) -> Result<(), String> {
    runtime
        .execute_command("workspace.dashboard")
        .map_err(|error| error.to_string())?;
    shell_ui(runtime)?
        .picker()
        .ok_or_else(|| "workspace.dashboard did not open picker".to_owned())?;
    Ok(())
}

pub(super) fn select_dashboard_row_matching_path(
    runtime: &mut EditorRuntime,
    path: &Path,
) -> Result<(), String> {
    let picker = shell_ui_mut(runtime)?
        .picker_mut()
        .ok_or_else(|| "dashboard picker missing".to_owned())?;
    let index = picker
        .session
        .matches()
        .iter()
        .position(|matched| project_roots_equal(Path::new(matched.item().id()), path))
        .ok_or_else(|| format!("dashboard missing worktree row for `{}`", path.display()))?;
    picker.session.set_selected_index(index);
    Ok(())
}

pub(super) fn select_dashboard_create_row(runtime: &mut EditorRuntime) -> Result<(), String> {
    let picker = shell_ui_mut(runtime)?
        .picker_mut()
        .ok_or_else(|| "dashboard picker missing".to_owned())?;
    let index = picker
        .session
        .matches()
        .iter()
        .position(|matched| matched.item().id() == "git-worktree-dashboard:create")
        .ok_or_else(|| "dashboard missing `+ new worktree` row".to_owned())?;
    picker.session.set_selected_index(index);
    Ok(())
}

pub(super) fn prepare_quickfix_workspace_search_picker(
    test_name: &str,
) -> Result<(ShellState, PathBuf, PathBuf, PathBuf), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir(test_name);
    let first = root.join("src").join("main.rs");
    let second = root.join("src").join("lib.rs");
    std::fs::create_dir_all(first.parent().ok_or_else(|| "missing src dir".to_owned())?)
        .map_err(|error| error.to_string())?;
    std::fs::write(&first, "fn alpha() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "fn beta() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, test_name, &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Workspace Search",
            vec![
                workspace_search::workspace_search_match_entry(
                    &root,
                    "src/main.rs",
                    1,
                    4,
                    "fn alpha() {}",
                ),
                workspace_search::workspace_search_match_entry(
                    &root,
                    "src/lib.rs",
                    1,
                    4,
                    "fn beta() {}",
                ),
            ],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new("Ctrl+q", "quickfix.open")]),
    );

    Ok((state, root, first, second))
}

// ---------------------------------------------------------------------------
// LeaveOpen exit action
// ---------------------------------------------------------------------------

pub(super) fn wait_for_streamed_command_worker_done(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        if !tracked {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "streamed command worker for buffer `{buffer_id}` did not finish in time"
    ))
}

// ---------------------------------------------------------------------------
// InputPromptOverlay
// ---------------------------------------------------------------------------

pub(super) fn shell_echo_command(marker: &str) -> String {
    if cfg!(windows) {
        format!("Write-Output {marker}")
    } else {
        format!("printf '{marker}\\n'")
    }
}

pub(super) fn shell_sleep_then_echo_command(seconds: u64, marker: &str) -> String {
    if cfg!(windows) {
        format!("Start-Sleep -Seconds {seconds}; Write-Output {marker}")
    } else {
        format!("sleep {seconds}; printf '{marker}\\n'")
    }
}

pub(super) fn execute_shell_command(state: &mut ShellState, command: &str) -> Result<(), String> {
    state
        .runtime
        .execute_command(command)
        .map_err(|error| error.to_string())
}

pub(super) fn active_input_prompt_text(state: &ShellState) -> Result<Option<String>, String> {
    Ok(shell_ui(&state.runtime)?
        .input_prompt()
        .map(|prompt| prompt.text().to_owned()))
}

pub(super) fn confirm_input_prompt(state: &mut ShellState, text: &str) -> Result<(), String> {
    if !text.is_empty() {
        state
            .handle_text_input(text)
            .map_err(|error| error.to_string())?;
    }
    state
        .try_runtime_keybinding(Keycode::Return, Mod::empty())
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn start_workspace_compile(
    state: &mut ShellState,
    command: &str,
) -> Result<BufferId, String> {
    execute_shell_command(state, "workspace.compile")?;
    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "workspace.compile should open InputPromptOverlay"
    );
    confirm_input_prompt(state, command)?;
    active_runtime_popup(&state.runtime)?
        .map(|popup| popup.active_buffer)
        .ok_or_else(|| "compile confirmation did not open streamed popup".to_owned())
}

pub(super) fn prompt_prefill_for_marker(
    tag: &str,
    marker_name: &str,
    marker_contents: &str,
) -> Result<String, String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir(tag);
    if !marker_name.is_empty() {
        std::fs::write(root.join(marker_name), marker_contents)
            .map_err(|error| error.to_string())?;
    }
    open_workspace_from_project(&mut state.runtime, tag, &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;
    let text = active_input_prompt_text(&state)?.unwrap_or_default();
    std::fs::remove_dir_all(&root).ok();
    Ok(text)
}

#[derive(Debug, Clone, Copy)]
pub(super) struct WorkspaceDockTestUserLibrary {
    config: WorkspaceDockConfig,
}

impl UserLibrary for WorkspaceDockTestUserLibrary {
    fn workspace_dock_config(&self) -> WorkspaceDockConfig {
        self.config
    }
}

pub(super) fn state_with_workspace_dock_config(
    config: WorkspaceDockConfig,
) -> Result<ShellState, String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(WorkspaceDockTestUserLibrary { config });
    ShellState::new_with_user_library(default_error_log_path(), false, user_library)
        .map_err(|error| error.to_string())
}

pub(super) fn install_fake_tcp_dap_manager(
    runtime: &mut EditorRuntime,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    use editor_dap::{DebugAdapterRegistry, DebugAdapterSpec, DebugAdapterTransport};
    use std::io::{BufRead, Read, Write};
    use std::net::TcpListener;

    pub(super) fn write_raw(writer: &mut impl Write, body: &str) {
        write!(writer, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("write");
        writer.flush().expect("flush");
    }

    pub(super) fn read_body(reader: &mut impl BufRead) -> Result<String, String> {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            let read = reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if read == 0 {
                return Err("adapter closed".to_owned());
            }
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                break;
            }
            let Some((key, value)) = trimmed.split_once(':') else {
                continue;
            };
            if key.eq_ignore_ascii_case("Content-Length") {
                content_length = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?);
            }
        }
        let len = content_length.ok_or_else(|| "missing Content-Length".to_owned())?;
        let mut buf = vec![0_u8; len];
        reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
        String::from_utf8(buf).map_err(|e| e.to_string())
    }

    pub(super) fn extract_field(body: &str, key: &str) -> Option<String> {
        let needle = format!("\"{key}\"");
        let start = body.find(&needle)?;
        let after = &body[start + needle.len()..];
        let after = after.trim_start_matches([' ', ':', '\t']);
        if let Some(rest) = after.strip_prefix('"') {
            let end = rest.find('"')?;
            return Some(rest[..end].to_owned());
        }
        let end = after
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after.len());
        Some(after[..end].to_owned())
    }

    pub(super) fn fake_adapter_loop(reader: impl Read, mut writer: impl Write) {
        let mut reader = std::io::BufReader::new(reader);
        let mut seq = 1_u64;
        let mut stopped_line = 1_u64;
        let mut program_path = "main.rs".to_owned();
        while let Ok(body) = read_body(&mut reader) {
            let command = extract_field(&body, "command").unwrap_or_default();
            let request_seq = extract_field(&body, "seq").unwrap_or_else(|| "0".to_owned());
            match command.as_str() {
                "initialize" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"initialize","body":{{"supportsConfigurationDoneRequest":true,"supportTerminateDebuggee":true,"supportsRestartRequest":true}}}}"#
                        ),
                    );
                    seq += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"initialized","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "configurationDone" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"configurationDone","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "launch" | "attach" => {
                    if let Some(program) = extract_field(&body, "program") {
                        program_path = program;
                    }
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"{command}","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line = 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"entry","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "setBreakpoints" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"setBreakpoints","body":{{"breakpoints":[]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "continue" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"continue"}}"#
                        ),
                    );
                    seq += 1;
                    if program_path.contains("exit-on-continue") {
                        write_raw(
                            &mut writer,
                            &format!(
                                r#"{{"seq":{seq},"type":"event","event":"exited","body":{{"exitCode":0}}}}"#
                            ),
                        );
                        seq += 1;
                        write_raw(
                            &mut writer,
                            &format!(
                                r#"{{"seq":{seq},"type":"event","event":"terminated","body":{{}}}}"#
                            ),
                        );
                        break;
                    }
                }
                "next" | "stepIn" | "stepOut" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"{command}","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"step","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "pause" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"pause","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"pause","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "restart" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"restart","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line = 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"entry","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "stackTrace" => {
                    let path_json = program_path.replace('\\', "\\\\");
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"stackTrace","body":{{"stackFrames":[{{"id":1,"name":"main","source":{{"path":"{path_json}"}},"line":{stopped_line},"column":1}},{{"id":2,"name":"caller","source":{{"path":"{path_json}"}},"line":{},"column":1}}],"totalFrames":2}}}}"#,
                            stopped_line + 10
                        ),
                    );
                    seq += 1;
                }
                "threads" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"threads","body":{{"threads":[{{"id":1,"name":"main"}},{{"id":2,"name":"worker"}}]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "evaluate" => {
                    let expression = extract_field(&body, "expression").unwrap_or_default();
                    let frame_id =
                        extract_field(&body, "frameId").unwrap_or_else(|| "1".to_owned());
                    let eval_body = if expression == "person" {
                        r#"{"result":"Person { ... }","type":"Person","variablesReference":2}"#
                            .to_owned()
                    } else {
                        format!(
                            r#"{{"result":"{expression}@{frame_id}={stopped_line}","variablesReference":0}}"#
                        )
                    };
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"evaluate","body":{eval_body}}}"#
                        ),
                    );
                    seq += 1;
                }
                "scopes" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"scopes","body":{{"scopes":[{{"name":"Locals","variablesReference":1,"expensive":false}}]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "variables" => {
                    let reference = extract_field(&body, "variablesReference")
                        .unwrap_or_else(|| "1".to_owned());
                    let variables = match reference.as_str() {
                        "2" => {
                            r#"[{"name":"Name","value":"\"Ada\"","type":"string","variablesReference":0},{"name":"Address","value":"Address { ... }","type":"Address","variablesReference":3}]"#
                        }
                        "3" => {
                            r#"[{"name":"City","value":"\"London\"","type":"string","variablesReference":0}]"#
                        }
                        _ => {
                            r#"[{"name":"x","value":"42","type":"i32","variablesReference":0},{"name":"person","value":"Person { ... }","type":"Person","variablesReference":2}]"#
                        }
                    };
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"variables","body":{{"variables":{variables}}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "disconnect" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"disconnect","body":{{}}}}"#
                        ),
                    );
                    break;
                }
                _ => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":false,"command":"{command}","message":"unsupported"}}"#
                        ),
                    );
                    seq += 1;
                }
            }
        }
    }

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut sessions = 0_u8;
        while sessions < 4 && Instant::now() < deadline {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let reader = stream.try_clone().expect("clone");
                    fake_adapter_loop(reader, stream);
                    sessions += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(_) => break,
            }
        }
    });

    let mut registry = DebugAdapterRegistry::new();
    registry
        .register(
            DebugAdapterSpec::new("fake-dap", "rust", ["rs"], "", [] as [&str; 0])
                .with_transport(DebugAdapterTransport::Tcp {
                    host: "127.0.0.1".to_owned(),
                    port,
                })
                .with_preference(10),
        )
        .map_err(|e| e.to_string())?;
    runtime
        .services_mut()
        .insert(Arc::new(DapClientManager::new(registry)));
    Ok((port, handle))
}
