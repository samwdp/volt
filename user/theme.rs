use editor_theme::{BlendMode, Color, Theme};

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
/// "token.rgba" = "#rrggbbaa"
///
/// [options]
/// "ui.line-number.relative" = true
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
    let mut section = "header";

    for line in source.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            if line.starts_with("[tokens]") {
                section = "tokens";
            } else if line.starts_with("[options]") {
                section = "options";
            }
            continue;
        }

        let Some((key_raw, value_raw)) = line.split_once('=') else {
            continue;
        };

        let key = key_raw.trim().trim_matches('"');
        let value = value_raw.trim().trim_matches('"');

        match section {
            "header" => match key {
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
                _ => {}
            },
            "tokens" => {
                if theme.is_none() {
                    if let (Some(id), Some(name)) = (id.as_deref(), name.as_deref()) {
                        theme = Some(Theme::new(id, name));
                    }
                }
                if let Some(color) = parse_hex_color(value) {
                    if let Some(t) = theme.take() {
                        theme = Some(t.with_token(key, color));
                    }
                }
            }
            "options" => {
                if theme.is_none() {
                    if let (Some(id), Some(name)) = (id.as_deref(), name.as_deref()) {
                        theme = Some(Theme::new(id, name));
                    }
                }
                let bool_val = match value {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                };
                if let Some(v) = bool_val {
                    if let Some(t) = theme.take() {
                        theme = Some(t.with_option(key, v));
                    }
                }
            }
            _ => {}
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

/// Parses a `#rrggbb` or `#rrggbbaa` hex color string into a [`Color`].
fn parse_hex_color(s: &str) -> Option<Color> {
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::rgba(r, g, b, a))
        }
        _ => None,
    }
}

/// Returns all built-in themes, compiled into the binary via `include_str!`.
///
/// Themes are embedded at compile time from the `user/themes/*.toml` files so
/// the binary works correctly regardless of the working directory or whether
/// a `user/themes` folder is present at the installation path.
pub fn themes() -> Vec<Theme> {
    const BUNDLED: &[&str] = &[
        include_str!("themes/gruvbox-dark.toml"),
        include_str!("themes/gruvbox-light.toml"),
        include_str!("themes/volt-dark.toml"),
        include_str!("themes/volt-light.toml"),
        include_str!("themes/vscode-dark.toml"),
        include_str!("themes/vscode-light.toml"),
    ];
    let mut themes: Vec<Theme> = BUNDLED.iter().filter_map(|s| parse_theme(s)).collect();
    themes.sort_by(|a, b| a.id().cmp(b.id()));
    themes
}

#[cfg(test)]
mod tests {
    use super::{parse_hex_color, parse_theme, themes};
    use editor_theme::{BlendMode, Color};

    #[test]
    fn parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#181b22"), Some(Color::rgb(0x18, 0x1b, 0x22)));
        assert_eq!(parse_hex_color("#ffffff"), Some(Color::rgb(255, 255, 255)));
        assert_eq!(parse_hex_color("#000000"), Some(Color::rgb(0, 0, 0)));
    }

    #[test]
    fn parse_hex_color_rgba_valid() {
        assert_eq!(
            parse_hex_color("#61afef6e"),
            Some(Color::rgba(0x61, 0xaf, 0xef, 0x6e))
        );
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
    fn parse_theme_parses_rgba_token() {
        let src = concat!(
            "id = \"t\"\n",
            "name = \"T\"\n",
            "[tokens]\n",
            "\"ui.yank-flash\" = \"#61afef6e\"\n",
        );
        let theme = parse_theme(src).expect("should parse");
        assert_eq!(theme.color("ui.yank-flash"), Some(Color::rgba(0x61, 0xaf, 0xef, 0x6e)));
    }

    #[test]
    fn parse_theme_parses_options() {
        let src = concat!(
            "id = \"t\"\n",
            "name = \"T\"\n",
            "[options]\n",
            "\"ui.line-number.relative\" = true\n",
        );
        let theme = parse_theme(src).expect("should parse");
        assert_eq!(theme.option("ui.line-number.relative"), Some(true));
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
    fn themes_returns_all_bundled_themes() {
        let t = themes();
        assert_eq!(t.len(), 6);

        // Sorted alphabetically by ID: g < v, "volt" < "vscode".
        let expected_ids = [
            "gruvbox-dark",
            "gruvbox-light",
            "volt-dark",
            "volt-light",
            "vscode-dark",
            "vscode-light",
        ];
        for (theme, expected_id) in t.iter().zip(expected_ids.iter()) {
            assert_eq!(theme.id(), *expected_id);
            assert!(
                theme.color("syntax.keyword").is_some(),
                "theme `{}` missing syntax.keyword token",
                theme.id()
            );
        }

        let volt_dark = t.iter().find(|t| t.id() == "volt-dark").expect("volt-dark");
        assert!(volt_dark.color("ui.yank-flash").is_some());
    }
}
