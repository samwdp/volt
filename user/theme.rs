use editor_theme::{Color, Theme, ThemeOption};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

const THEME_DIRECTORY_PARTS: [&str; 2] = ["user", "themes"];
const THEME_EXTENSION: &str = "toml";
const GLOBAL_THEME_FILE_NAME: &str = "global.toml";
const DEFAULT_THEME_ID: &str = "gruvbox-dark";
const PALETTE_SECTION_NAMES: [&str; 2] = ["palette", "pallet"];
const PALETTE_REFERENCE_PREFIXES: [&str; 2] = ["palette.", "pallet."];
// Maximum number of ancestor directories to check when resolving user/themes from current_exe.
const THEME_SEARCH_DEPTH: usize = 6;

#[derive(Debug, Default)]
struct SharedThemeConfig {
    options: BTreeMap<String, ThemeOption>,
    language_options: BTreeMap<String, BTreeMap<String, ThemeOption>>,
}

impl SharedThemeConfig {
    fn apply_to_theme(&self, mut theme: Theme) -> Theme {
        for (option, value) in &self.options {
            theme = theme.with_option(option.clone(), value.clone());
        }
        for (language_id, options) in &self.language_options {
            for (option, value) in options {
                theme = theme.with_option(format!("langs.{language_id}.{option}"), value.clone());
            }
        }
        theme
    }
}

/// Returns themes loaded from the executable-relative themes directory.
pub fn themes() -> Vec<Theme> {
    let Some(themes_dir) = themes_dir() else {
        return Vec::new();
    };
    let shared_config = load_shared_theme_config(&themes_dir);
    let mut theme_files = match list_theme_files(&themes_dir) {
        Ok(files) => files,
        Err(error) => {
            eprintln!(
                "failed to read themes directory `{}`: {error}",
                themes_dir.display()
            );
            return Vec::new();
        }
    };
    theme_files.sort();

    let mut themes = Vec::new();
    for path in theme_files {
        match fs::read_to_string(&path) {
            Ok(contents) => match parse_theme(&path, &contents, shared_config.as_ref()) {
                Ok(theme) => themes.push(theme),
                Err(error) => {
                    eprintln!("failed to parse theme `{}`: {error}", path.display());
                }
            },
            Err(error) => {
                eprintln!("failed to read theme `{}`: {error}", path.display());
            }
        }
    }

    if let Some(index) = themes
        .iter()
        .position(|theme| theme.id() == DEFAULT_THEME_ID)
    {
        let theme = themes.remove(index);
        themes.insert(0, theme);
    }

    themes
}

fn load_shared_theme_config(themes_dir: &Path) -> Option<SharedThemeConfig> {
    let path = themes_dir.join(GLOBAL_THEME_FILE_NAME);
    if !path.is_file() {
        return None;
    }
    match fs::read_to_string(&path) {
        Ok(contents) => match parse_shared_theme_config(&path, &contents) {
            Ok(config) => Some(config),
            Err(error) => {
                eprintln!(
                    "failed to parse shared theme config `{}`: {error}",
                    path.display()
                );
                None
            }
        },
        Err(error) => {
            eprintln!(
                "failed to read shared theme config `{}`: {error}",
                path.display()
            );
            None
        }
    }
}

fn themes_dir() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    themes_dir_from_exe_dir(exe_dir)
}

fn themes_dir_from_exe_dir(exe_dir: &Path) -> Option<PathBuf> {
    let mut fallback = None;
    for ancestor in exe_dir.ancestors().take(THEME_SEARCH_DEPTH) {
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

fn list_theme_files(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let entries = fs::read_dir(path)?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case(THEME_EXTENSION))
            && !path.file_name().is_some_and(|name| {
                name.to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(GLOBAL_THEME_FILE_NAME))
            })
        {
            files.push(path);
        }
    }
    Ok(files)
}

fn parse_theme(
    path: &Path,
    source: &str,
    shared_config: Option<&SharedThemeConfig>,
) -> Result<Theme, String> {
    let table = parse_toml_table(source)?;
    let id = theme_id(path, &table)?;
    let name = table
        .get("name")
        .and_then(toml::Value::as_str)
        .unwrap_or(&id)
        .to_owned();
    let palette = parse_palette(&table)?;

    let mut theme = Theme::new(id, name);
    if let Some(shared_config) = shared_config {
        theme = shared_config.apply_to_theme(theme);
    }
    if let Some(tokens) = table.get("tokens").and_then(toml::Value::as_table) {
        for (token, value) in tokens {
            let color = parse_color(token, value, &palette)?;
            theme = theme.with_token(token, color);
        }
    }

    theme = apply_options_table(theme, &table)?;
    theme = apply_language_options_table(theme, &table)?;

    Ok(theme)
}

fn parse_shared_theme_config(path: &Path, source: &str) -> Result<SharedThemeConfig, String> {
    let table = parse_toml_table(source)?;
    let options = parse_options_table(&table)?;
    let language_options = parse_language_options_table(&table)?;
    if table.get("tokens").is_some() {
        return Err(format!(
            "shared theme config `{}` cannot define [tokens]",
            path.display()
        ));
    }
    for section_name in PALETTE_SECTION_NAMES {
        if table.get(section_name).is_some() {
            return Err(format!(
                "shared theme config `{}` cannot define [{section_name}]",
                path.display()
            ));
        }
    }
    Ok(SharedThemeConfig {
        options,
        language_options,
    })
}

fn parse_toml_table(source: &str) -> Result<toml::value::Table, String> {
    let value: toml::Value =
        toml::from_str(source).map_err(|error| format!("toml parse error: {error}"))?;
    value
        .as_table()
        .cloned()
        .ok_or_else(|| "theme root must be a table".to_owned())
}

fn apply_options_table(mut theme: Theme, table: &toml::value::Table) -> Result<Theme, String> {
    for (option, value) in parse_options_table(table)? {
        theme = theme.with_option(option, value);
    }
    Ok(theme)
}

fn parse_options_table(
    table: &toml::value::Table,
) -> Result<BTreeMap<String, ThemeOption>, String> {
    let mut parsed = BTreeMap::new();
    let Some(options) = table.get("options").and_then(toml::Value::as_table) else {
        return Ok(parsed);
    };
    for (option, value) in options {
        let parsed_value = parse_option(option, value)?;
        parsed.insert(option.clone(), parsed_value);
    }
    Ok(parsed)
}

fn apply_language_options_table(
    mut theme: Theme,
    table: &toml::value::Table,
) -> Result<Theme, String> {
    for (language_id, options) in parse_language_options_table(table)? {
        for (option, value) in options {
            theme = theme.with_option(format!("langs.{language_id}.{option}"), value);
        }
    }
    Ok(theme)
}

fn parse_language_options_table(
    table: &toml::value::Table,
) -> Result<BTreeMap<String, BTreeMap<String, ThemeOption>>, String> {
    let mut parsed = BTreeMap::new();
    let Some(langs) = table.get("langs").and_then(toml::Value::as_table) else {
        return Ok(parsed);
    };
    for (language_id, value) in langs {
        let language_table = value
            .as_table()
            .ok_or_else(|| format!("langs.{language_id} must be a table of language options"))?;
        let mut options = BTreeMap::new();
        for (option, option_value) in language_table {
            let key = format!("langs.{language_id}.{option}");
            let parsed_value = parse_option(&key, option_value)?;
            options.insert(option.clone(), parsed_value);
        }
        parsed.insert(language_id.clone(), options);
    }
    Ok(parsed)
}

fn parse_palette(table: &toml::value::Table) -> Result<BTreeMap<String, Color>, String> {
    let mut entries = BTreeMap::new();
    for section_name in PALETTE_SECTION_NAMES {
        let Some(section) = table.get(section_name) else {
            continue;
        };
        let section = section
            .as_table()
            .ok_or_else(|| format!("{section_name} must be a table of palette colors"))?;
        for (name, value) in section {
            if entries.insert(name.as_str(), value).is_some() {
                return Err(format!("palette entry `{name}` is declared more than once"));
            }
        }
    }

    let mut resolved = BTreeMap::new();
    let mut stack = Vec::new();
    for name in entries.keys().copied().collect::<Vec<_>>() {
        resolve_palette_color(name, &entries, &mut resolved, &mut stack)?;
    }
    Ok(resolved)
}

fn theme_id(path: &Path, table: &toml::value::Table) -> Result<String, String> {
    let explicit = table
        .get("id")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    if let Some(id) = explicit {
        return Ok(id);
    }
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_owned);
    stem.ok_or_else(|| format!("theme `{}` is missing an id", path.display()))
}

fn resolve_palette_color<'a>(
    name: &'a str,
    entries: &BTreeMap<&'a str, &'a toml::Value>,
    resolved: &mut BTreeMap<String, Color>,
    stack: &mut Vec<&'a str>,
) -> Result<Color, String> {
    if let Some(color) = resolved.get(name).copied() {
        return Ok(color);
    }
    if let Some(start) = stack.iter().position(|entry| *entry == name) {
        let mut cycle = stack[start..].to_vec();
        cycle.push(name);
        return Err(format!(
            "palette references form a cycle: {}",
            cycle.join(" -> ")
        ));
    }
    let value = entries
        .get(name)
        .copied()
        .ok_or_else(|| format!("palette entry `{name}` was not found"))?;
    stack.push(name);
    let raw = value
        .as_str()
        .ok_or_else(|| format!("palette entry `{name}` must be a string"))?;
    let color = if let Some(color) =
        parse_hex_color(raw).map_err(|error| format!("palette entry `{name}` {error}"))?
    {
        color
    } else {
        let Some(reference) = parse_palette_reference(raw) else {
            return Err(format!(
                "palette entry `{name}` must be a 6 or 8 digit hex value or a palette reference"
            ));
        };
        if !entries.contains_key(reference) {
            return Err(format!(
                "palette entry `{name}` references unknown palette color `{reference}`"
            ));
        }
        resolve_palette_color(reference, entries, resolved, stack)?
    };
    stack.pop();
    resolved.insert(name.to_owned(), color);
    Ok(color)
}

fn parse_color(
    token: &str,
    value: &toml::Value,
    palette: &BTreeMap<String, Color>,
) -> Result<Color, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("token `{token}` must be a string"))?;
    if let Some(color) = parse_hex_color(raw).map_err(|error| format!("token `{token}` {error}"))? {
        return Ok(color);
    }
    let Some(reference) = parse_palette_reference(raw) else {
        return Err(format!(
            "token `{token}` must be a 6 or 8 digit hex value or a palette reference"
        ));
    };
    palette
        .get(reference)
        .copied()
        .ok_or_else(|| format!("token `{token}` references unknown palette color `{reference}`"))
}

fn parse_palette_reference(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    PALETTE_REFERENCE_PREFIXES
        .into_iter()
        .find_map(|prefix| trimmed.strip_prefix(prefix))
}

fn parse_hex_color(raw: &str) -> Result<Option<Color>, String> {
    let trimmed = raw.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        return Ok(Some(parse_hex_color_value(hex)?));
    }
    let is_plain_hex = matches!(trimmed.len(), 6 | 8)
        && trimmed
            .chars()
            .all(|character| character.is_ascii_hexdigit());
    if is_plain_hex {
        return Ok(Some(parse_hex_color_value(trimmed)?));
    }
    Ok(None)
}

fn parse_hex_color_value(hex: &str) -> Result<Color, String> {
    match hex.len() {
        6 => {
            let r = parse_hex_channel(&hex[0..2])?;
            let g = parse_hex_channel(&hex[2..4])?;
            let b = parse_hex_channel(&hex[4..6])?;
            Ok(Color::rgb(r, g, b))
        }
        8 => {
            let r = parse_hex_channel(&hex[0..2])?;
            let g = parse_hex_channel(&hex[2..4])?;
            let b = parse_hex_channel(&hex[4..6])?;
            let a = parse_hex_channel(&hex[6..8])?;
            Ok(Color::rgba(r, g, b, a))
        }
        _ => Err("must be a 6 or 8 digit hex value".to_owned()),
    }
}

fn parse_hex_channel(hex: &str) -> Result<u8, String> {
    u8::from_str_radix(hex, 16).map_err(|_| format!("invalid hex channel `{hex}`"))
}

fn parse_option(option: &str, value: &toml::Value) -> Result<ThemeOption, String> {
    match value {
        toml::Value::Boolean(value) => Ok(ThemeOption::Bool(*value)),
        toml::Value::Integer(value) => {
            const MAX_THEME_OPTION_INTEGER: i64 = 9_007_199_254_740_992; // 2^53 in IEEE 754 f64.
            if *value < -MAX_THEME_OPTION_INTEGER || *value > MAX_THEME_OPTION_INTEGER {
                return Err(format!(
                    "option `{option}` integer value is too large for a number"
                ));
            }
            Ok(ThemeOption::Number(*value as f64))
        }
        toml::Value::Float(value) => Ok(ThemeOption::Number(*value)),
        toml::Value::String(value) => Ok(ThemeOption::Text(value.clone())),
        _ => Err(format!(
            "option `{option}` must be a boolean, number, or string"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GLOBAL_THEME_FILE_NAME, SharedThemeConfig, list_theme_files, parse_shared_theme_config,
        parse_theme, themes_dir_from_exe_dir,
    };
    use editor_theme::{Color, Theme};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn bundled_theme_sources() -> Vec<(PathBuf, String)> {
        let themes_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("themes");
        let mut theme_files = list_theme_files(&themes_dir)
            .unwrap_or_else(|error| panic!("failed to list bundled themes: {error}"));
        theme_files.sort();
        theme_files
            .into_iter()
            .map(|path| {
                let source = fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
                (path, source)
            })
            .collect()
    }

    fn bundled_shared_theme_config() -> SharedThemeConfig {
        let themes_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("themes");
        let path = themes_dir.join(GLOBAL_THEME_FILE_NAME);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        parse_shared_theme_config(&path, &source)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
    }

    fn assert_bundled_theme_uses_pallet_colors(path: &Path, source: &str) {
        let table: toml::Table = toml::from_str(source)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()));
        assert!(
            table
                .get("pallet")
                .and_then(toml::Value::as_table)
                .is_some(),
            "theme {} must declare a [pallet] section",
            path.display()
        );
        let tokens = table
            .get("tokens")
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("theme {} missing [tokens] section", path.display()));
        for (token, value) in tokens {
            let raw = value.as_str().unwrap_or_else(|| {
                panic!("theme {} token `{token}` must be a string", path.display())
            });
            assert!(
                raw.trim().starts_with("pallet."),
                "theme {} token `{token}` must use a pallet.* reference, found `{raw}`",
                path.display()
            );
        }
    }

    fn assert_bundled_theme_omits_shared_sections(path: &Path, source: &str) {
        let table: toml::Table = toml::from_str(source)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()));
        assert!(
            table.get("options").is_none(),
            "theme {} must not declare [options]",
            path.display()
        );
        assert!(
            table.get("langs").is_none(),
            "theme {} must not declare [langs.*]",
            path.display()
        );
    }

    #[test]
    fn parse_theme_resolves_palette_references_in_tokens_and_options() {
        let source = r##"
id = "test-theme"
name = "Test Theme"

[palette]
background = "#112233"
accent = "palette.background"

[tokens]
"ui.background" = "palette.background"
"ui.cursor" = "palette.accent"

[options]
font = "Example Mono"
font_size = 18
"ui.line-number.relative" = true

[langs.rust]
indent = 4
format_on_save = true
use_tabs = false
"##;
        let theme = parse_theme(std::path::Path::new("test.toml"), source, None)
            .unwrap_or_else(|error| panic!("unexpected error: {error}"));

        assert_eq!(theme.id(), "test-theme");
        assert_eq!(
            theme.color("ui.background"),
            Some(Color::rgb(0x11, 0x22, 0x33))
        );
        assert_eq!(theme.color("ui.cursor"), Some(Color::rgb(0x11, 0x22, 0x33)));
        assert!(theme.option_string("font").is_some());
        assert!(theme.option_number("font_size").is_some());
        assert_eq!(theme.option_bool("ui.line-number.relative"), Some(true));
        assert_eq!(theme.option_number("langs.rust.indent"), Some(4.0));
        assert_eq!(theme.option_bool("langs.rust.format_on_save"), Some(true));
        assert_eq!(theme.option_bool("langs.rust.use_tabs"), Some(false));
    }

    #[test]
    fn parse_theme_accepts_pallet_section_alias() {
        let source = r##"
[pallet]
blue = "#83a598"

[tokens]
"ui.background" = "pallet.blue"
"##;
        let theme = parse_theme(std::path::Path::new("test.toml"), source, None)
            .unwrap_or_else(|error| panic!("unexpected error: {error}"));

        assert_eq!(
            theme.color("ui.background"),
            Some(Color::rgb(0x83, 0xa5, 0x98))
        );
    }

    #[test]
    fn parse_theme_accepts_inline_hex_token_colors() {
        let source = r##"
[tokens]
"ui.background" = "#112233"
"ui.cursor" = "44556677"
"##;
        let theme = parse_theme(std::path::Path::new("test.toml"), source, None)
            .unwrap_or_else(|error| panic!("unexpected error: {error}"));

        assert_eq!(
            theme.color("ui.background"),
            Some(Color::rgb(0x11, 0x22, 0x33))
        );
        assert_eq!(
            theme.color("ui.cursor"),
            Some(Color::rgba(0x44, 0x55, 0x66, 0x77))
        );
    }

    #[test]
    fn bundled_themes_use_pallet_sections_and_token_references() {
        for (path, source) in bundled_theme_sources() {
            assert_bundled_theme_uses_pallet_colors(&path, &source);
            assert_bundled_theme_omits_shared_sections(&path, &source);
        }
    }

    #[test]
    fn bundled_themes_define_defaults_for_all_compiled_languages() {
        let shared = bundled_shared_theme_config();
        let expected_language_ids = crate::syntax_languages()
            .into_iter()
            .map(|language| language.id().to_owned())
            .filter(|language_id| language_id != "markdown-inline")
            .collect::<std::collections::BTreeSet<_>>();

        for (path, source) in bundled_theme_sources() {
            let theme = parse_theme(&path, &source, Some(&shared))
                .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()));

            for language_id in &expected_language_ids {
                let indent_key = format!("langs.{language_id}.indent");
                let format_key = format!("langs.{language_id}.format_on_save");
                let tabs_key = format!("langs.{language_id}.use_tabs");
                assert!(
                    theme.option_number(&indent_key).is_some(),
                    "theme {} missing {indent_key}",
                    path.display()
                );
                assert!(
                    theme.option_bool(&format_key).is_some(),
                    "theme {} missing {format_key}",
                    path.display()
                );
                assert!(
                    theme.option_bool(&tabs_key).is_some(),
                    "theme {} missing {tabs_key}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn bundled_shared_theme_config_includes_window_effect_defaults() {
        let shared = bundled_shared_theme_config();
        let theme = shared.apply_to_theme(Theme::new("test-theme", "Test Theme"));
        let corner_radius = theme
            .option_number("corner_radius")
            .unwrap_or_else(|| panic!("shared config missing corner_radius"));
        let opacity = theme
            .option_number("window.opacity")
            .unwrap_or_else(|| panic!("shared config missing window.opacity"));
        let blur = theme
            .option_number("window.blur")
            .unwrap_or_else(|| panic!("shared config missing window.blur"));

        assert!(corner_radius >= 0.0);
        assert!((0.0..=1.0).contains(&opacity));
        assert!(blur >= 0.0);
        assert_eq!(corner_radius, 16.0);
        assert_eq!(opacity, 0.1);
        assert_eq!(blur, 1.0);
    }

    #[test]
    fn parse_theme_applies_shared_options_and_languages() {
        let shared = parse_shared_theme_config(
            Path::new(GLOBAL_THEME_FILE_NAME),
            r##"
[options]
font = "Example Mono"
font_size = 14
corner_radius = 10
"window.opacity" = 0.75
"window.blur" = 12.0

[langs.rust]
indent = 4
format_on_save = true
use_tabs = false
"##,
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
        let theme = parse_theme(
            Path::new("test.toml"),
            r##"
id = "test-theme"

[pallet]
background = "#112233"

[tokens]
"ui.background" = "pallet.background"
"##,
            Some(&shared),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

        assert_eq!(theme.option_string("font"), Some("Example Mono"));
        assert_eq!(theme.option_number("font_size"), Some(14.0));
        assert_eq!(theme.option_number("corner_radius"), Some(10.0));
        assert_eq!(theme.option_number("window.opacity"), Some(0.75));
        assert_eq!(theme.option_number("window.blur"), Some(12.0));
        assert_eq!(theme.option_number("langs.rust.indent"), Some(4.0));
        assert_eq!(theme.option_bool("langs.rust.format_on_save"), Some(true));
        assert_eq!(theme.option_bool("langs.rust.use_tabs"), Some(false));
    }

    #[test]
    fn parse_theme_specific_options_override_shared_values() {
        let shared = parse_shared_theme_config(
            Path::new(GLOBAL_THEME_FILE_NAME),
            r##"
[options]
font = "Global Font"
font_size = 14

[langs.rust]
indent = 4
format_on_save = false
use_tabs = false
"##,
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
        let theme = parse_theme(
            Path::new("test.toml"),
            r##"
id = "test-theme"

[pallet]
background = "#112233"

[tokens]
"ui.background" = "pallet.background"

[options]
font = "Local Font"

[langs.rust]
format_on_save = true
"##,
            Some(&shared),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

        assert_eq!(theme.option_string("font"), Some("Local Font"));
        assert_eq!(theme.option_number("font_size"), Some(14.0));
        assert_eq!(theme.option_number("langs.rust.indent"), Some(4.0));
        assert_eq!(theme.option_bool("langs.rust.format_on_save"), Some(true));
        assert_eq!(theme.option_bool("langs.rust.use_tabs"), Some(false));
    }

    #[test]
    fn list_theme_files_excludes_shared_theme_config() {
        let temp_root = std::env::temp_dir().join(format!(
            "volt-theme-files-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_millis()
        ));
        fs::create_dir_all(&temp_root).expect("create temp theme dir");
        fs::write(temp_root.join("alpha.toml"), "id = \"alpha\"\n").expect("write alpha theme");
        fs::write(temp_root.join(GLOBAL_THEME_FILE_NAME), "[options]\n")
            .expect("write global config");

        let mut files = list_theme_files(&temp_root).expect("list theme files");
        files.sort();

        assert_eq!(files, vec![temp_root.join("alpha.toml")]);

        fs::remove_dir_all(&temp_root).expect("cleanup temp theme dir");
    }

    #[test]
    fn themes_dir_prefers_workspace_source_over_staged_target_copy() {
        let temp_root = std::env::temp_dir().join(format!(
            "volt-theme-dir-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_millis()
        ));
        let exe_dir = temp_root.join("target").join("debug").join("deps");
        let staged_themes = temp_root
            .join("target")
            .join("debug")
            .join("user")
            .join("themes");
        let source_themes = temp_root.join("user").join("themes");

        fs::create_dir_all(&exe_dir).expect("create exe dir");
        fs::create_dir_all(&staged_themes).expect("create staged themes dir");
        fs::create_dir_all(&source_themes).expect("create source themes dir");
        fs::write(temp_root.join("Cargo.toml"), "[workspace]\n").expect("write workspace manifest");

        let resolved =
            themes_dir_from_exe_dir(&exe_dir).expect("resolve themes directory from exe dir");
        assert_eq!(resolved, source_themes);

        fs::remove_dir_all(&temp_root).expect("cleanup temp theme dir");
    }
}
