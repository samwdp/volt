use std::path::{Path, PathBuf};

use editor_theme::{BlendMode, Color, Theme};

/// Returns the themes directory.
///
/// Tries `<cwd>/user/themes` first (runtime layout), then falls back to
/// `<cwd>/themes` (package-test layout where CWD is already `user/`).
fn themes_dir() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let with_user = cwd.join("user").join("themes");
    if with_user.is_dir() {
        with_user
    } else {
        cwd.join("themes")
    }
}

/// Loads all `.toml` theme files found in `dir`.
///
/// Files that cannot be read or that fail to parse are silently skipped so
/// that a single bad file never prevents the rest from loading.
fn load_themes_from_dir(dir: &Path) -> Vec<Theme> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut themes = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Some(theme) = parse_theme(&source) {
            themes.push(theme);
        }
    }
    themes.sort_by(|a, b| a.id().cmp(b.id()));
    themes
}

/// Parses a theme from a TOML-like source text.
///
/// The format is:
/// ```toml
/// id = "theme-id"
/// name = "Theme Name"
/// opacity = 0.9
/// blend_mode = "blur"   # "opacity" (default) or "blur"
/// font = "JetBrains Mono"
/// font_size = 18
///
/// [tokens]
/// "token.name" = "#rrggbb"
/// ```
///
/// Unrecognised lines and invalid color values are silently skipped.
fn parse_theme(source: &str) -> Option<Theme> {
    let mut id: Option<String> = None;
    let mut name: Option<String> = None;
    let mut opacity: Option<f32> = None;
    let mut blend_mode: Option<BlendMode> = None;
    let mut font: Option<String> = None;
    let mut font_size: Option<u32> = None;
    let mut theme = None;

    for line in source.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('[') || line.starts_with('#') {
            continue;
        }

        let Some((key_raw, value_raw)) = line.split_once('=') else {
            continue;
        };

        let key = key_raw.trim().trim_matches('"');
        let value = value_raw.trim().trim_matches('"');

        match key {
            "id" => id = Some(value.to_owned()),
            "name" => name = Some(value.to_owned()),
            "opacity" => opacity = value.parse::<f32>().ok(),
            "blend_mode" => {
                blend_mode = match value {
                    "blur" => Some(BlendMode::Blur),
                    _ => Some(BlendMode::Opacity),
                };
            }
            "font" => font = Some(value.to_owned()),
            "font_size" => font_size = value.parse::<u32>().ok(),
            token => {
                if theme.is_none() {
                    if let (Some(id), Some(name)) = (id.as_deref(), name.as_deref()) {
                        theme = Some(Theme::new(id, name));
                    }
                }
                if let Some(color) = parse_hex_color(value) {
                    if let Some(t) = theme.take() {
                        theme = Some(t.with_token(token, color));
                    }
                }
            }
        }
    }

    let mut built = theme.or_else(|| {
        id.as_deref()
            .zip(name.as_deref())
            .map(|(i, n)| Theme::new(i, n))
    })?;

    if let Some(v) = opacity {
        built = built.with_opacity(v);
    }
    if let Some(bm) = blend_mode {
        built = built.with_blend_mode(bm);
    }
    if let Some(f) = font {
        built = built.with_font(f);
    }
    if let Some(fs) = font_size {
        built = built.with_font_size(fs);
    }
    Some(built)
}

/// Parses a `#rrggbb` hex color string into a [`Color`].
fn parse_hex_color(s: &str) -> Option<Color> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::rgb(r, g, b))
}

/// Returns themes loaded from the `user/themes` directory at runtime.
///
/// Themes are discovered by scanning the directory for `.toml` files and
/// parsing each one. Files that fail to parse are silently skipped.
pub fn themes() -> Vec<Theme> {
    load_themes_from_dir(&themes_dir())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{load_themes_from_dir, parse_hex_color, parse_theme};
    use editor_theme::{BlendMode, Color};

    #[test]
    fn parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#181b22"), Some(Color::rgb(0x18, 0x1b, 0x22)));
        assert_eq!(parse_hex_color("#ffffff"), Some(Color::rgb(255, 255, 255)));
        assert_eq!(parse_hex_color("#000000"), Some(Color::rgb(0, 0, 0)));
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("181b22"), None);
        assert_eq!(parse_hex_color("#gg0000"), None);
        assert_eq!(parse_hex_color("#12345"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn parse_theme_returns_none_for_missing_id() {
        assert!(parse_theme("name = \"No ID\"\n[tokens]\n").is_none());
    }

    #[test]
    fn parse_theme_parses_tokens_correctly() {
        let src = concat!(
            "id = \"test\"\n",
            "name = \"Test Theme\"\n",
            "\n",
            "[tokens]\n",
            "\"syntax.keyword\" = \"#c678dd\"\n",
            "\"ui.background\" = \"#181b22\"\n",
        );
        let theme = parse_theme(src).expect("should parse");
        assert_eq!(theme.id(), "test");
        assert_eq!(theme.name(), "Test Theme");
        assert_eq!(theme.color("syntax.keyword"), Some(Color::rgb(0xc6, 0x78, 0xdd)));
        assert_eq!(theme.color("ui.background"), Some(Color::rgb(0x18, 0x1b, 0x22)));
        assert_eq!(theme.opacity(), 1.0);
        assert_eq!(theme.blend_mode(), BlendMode::Opacity);
        assert_eq!(theme.font(), None);
        assert_eq!(theme.font_size(), None);
    }

    #[test]
    fn parse_theme_parses_render_settings() {
        let src = concat!(
            "id = \"t\"\n",
            "name = \"T\"\n",
            "opacity = 0.85\n",
            "blend_mode = \"blur\"\n",
            "font = \"JetBrains Mono\"\n",
            "font_size = 16\n",
            "[tokens]\n",
        );
        let theme = parse_theme(src).expect("should parse");
        assert!((theme.opacity() - 0.85).abs() < 0.001);
        assert_eq!(theme.blend_mode(), BlendMode::Blur);
        assert_eq!(theme.font(), Some("JetBrains Mono"));
        assert_eq!(theme.font_size(), Some(16));
    }

    #[test]
    fn load_themes_from_dir_reads_toml_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path();

        fs::write(
            path.join("alpha.toml"),
            "id = \"alpha\"\nname = \"Alpha\"\n[tokens]\n\"syntax.keyword\" = \"#ff0000\"\n",
        )
        .expect("write");
        fs::write(
            path.join("beta.toml"),
            "id = \"beta\"\nname = \"Beta\"\n[tokens]\n\"syntax.keyword\" = \"#00ff00\"\n",
        )
        .expect("write");
        fs::write(path.join("not-a-theme.txt"), "ignored").expect("write");

        let themes = load_themes_from_dir(path);
        assert_eq!(themes.len(), 2);
        assert_eq!(themes[0].id(), "alpha");
        assert_eq!(themes[1].id(), "beta");
    }

    #[test]
    fn load_themes_from_dir_returns_empty_for_missing_dir() {
        let themes = load_themes_from_dir(std::path::Path::new("/nonexistent/path/themes"));
        assert!(themes.is_empty());
    }
}
