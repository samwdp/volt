use super::{Color, Theme, ThemeOption, ThemeRegistry, ThemeStyle};

fn volt_dark() -> Theme {
    Theme::new("volt-dark", "Volt Dark")
        .with_token("syntax.keyword", Color::rgb(198, 120, 221))
        .with_token("syntax.string", Color::rgb(152, 195, 121))
        .with_token_style(
            "syntax.markup.heading",
            Color::rgb(224, 108, 117),
            ThemeStyle::new(true, true),
        )
        .with_option("ui.line-number.relative", true)
        .with_option("cursor_roundness", 3.0)
}

fn amber() -> Theme {
    Theme::new("amber", "Amber")
        .with_token("syntax.keyword", Color::rgb(255, 191, 105))
        .with_token("syntax.string", Color::rgb(255, 221, 128))
}

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("unexpected error: {error:?}"),
    }
}

#[test]
fn registry_resolves_tokens_from_active_theme() {
    let mut registry = ThemeRegistry::new();
    must(registry.register_all([volt_dark(), amber()]));
    must(registry.activate("amber"));

    assert_eq!(registry.len(), 2);
    assert_eq!(
        registry.active_theme().map(|theme| theme.id()),
        Some("amber")
    );
    assert_eq!(
        registry.resolve("syntax.keyword"),
        Some(Color::rgb(255, 191, 105))
    );
}

#[test]
fn registry_resolves_option_values() {
    let mut registry = ThemeRegistry::new();
    must(registry.register(volt_dark()));

    assert_eq!(
        registry.resolve_option("cursor_roundness"),
        Some(&ThemeOption::Number(3.0))
    );
    assert_eq!(registry.resolve_bool("ui.line-number.relative"), Some(true));
    assert_eq!(registry.resolve_number("cursor_roundness"), Some(3.0));
}

#[test]
fn registry_resolves_token_styles() {
    let mut registry = ThemeRegistry::new();
    must(registry.register(volt_dark()));

    let style = registry
        .resolve_style("syntax.markup.heading")
        .unwrap_or_else(|| panic!("missing token style"));
    assert_eq!(style.color, Color::rgb(224, 108, 117));
    assert!(style.style.bold);
    assert!(style.style.italic);
    assert!(
        registry
            .resolve_style("syntax.keyword")
            .is_some_and(|style| style.style.is_plain())
    );
}
