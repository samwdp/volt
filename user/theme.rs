use editor_theme::{Color, Theme, ThemeOption};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const THEME_DIRECTORY_PARTS: [&str; 2] = ["user", "themes"];
const THEME_EXTENSION: &str = "toml";
const DEFAULT_THEME_ID: &str = "volt-dark";
// Maximum number of ancestor directories to check when resolving user/themes from current_exe.
const THEME_SEARCH_DEPTH: usize = 6;

/// Returns themes loaded from the executable-relative themes directory.
pub fn themes() -> Vec<Theme> {
    let Some(themes_dir) = themes_dir() else {
        return Vec::new();
    };
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
            Ok(contents) => match parse_theme(&path, &contents) {
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

fn themes_dir() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    for ancestor in exe_dir.ancestors().take(THEME_SEARCH_DEPTH) {
        let mut candidate = PathBuf::from(ancestor);
        for part in THEME_DIRECTORY_PARTS {
            candidate = candidate.join(part);
        }
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
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
        {
            files.push(path);
        }
    }
    Ok(files)
}

fn parse_theme(path: &Path, source: &str) -> Result<Theme, String> {
    let value: toml::Value =
        toml::from_str(source).map_err(|error| format!("toml parse error: {error}"))?;
    let table = value
        .as_table()
        .ok_or_else(|| "theme root must be a table".to_owned())?;
    let id = theme_id(path, table)?;
    let name = table
        .get("name")
        .and_then(toml::Value::as_str)
        .unwrap_or(&id)
        .to_owned();

    let mut theme = Theme::new(id, name);
    if let Some(tokens) = table.get("tokens").and_then(toml::Value::as_table) {
        for (token, value) in tokens {
            let color = parse_color(token, value)?;
            theme = theme.with_token(token, color);
        }
    }

    if let Some(options) = table.get("options").and_then(toml::Value::as_table) {
        for (option, value) in options {
            let parsed = parse_option(option, value)?;
            theme = theme.with_option(option, parsed);
        }
    }

    Ok(theme)
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

fn parse_color(token: &str, value: &toml::Value) -> Result<Color, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("token `{token}` must be a string"))?;
    let hex = raw.trim().trim_start_matches('#');
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
        _ => Err(format!("token `{token}` must be a 6 or 8 digit hex value")),
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
    use super::parse_theme;

    #[test]
    fn parses_theme_tokens_and_options() {
        let source = r##"
id = "test-theme"
name = "Test Theme"

[tokens]
"ui.background" = "#112233"

[options]
font = "Example Mono"
font_size = 18
"ui.line-number.relative" = true
"##;
        let theme = parse_theme(std::path::Path::new("test.toml"), source)
            .unwrap_or_else(|error| panic!("unexpected error: {error}"));

        assert_eq!(theme.id(), "test-theme");
        assert!(theme.color("ui.background").is_some());
        assert!(theme.option_string("font").is_some());
        assert!(theme.option_number("font_size").is_some());
        assert_eq!(theme.option_bool("ui.line-number.relative"), Some(true));
    }
}
