use editor_theme::{Color, Theme};

/// Returns themes compiled into the user library.
pub fn themes() -> Vec<Theme> {
    vec![
        Theme::new("volt-dark", "Volt Dark")
            .with_token("ui.background", Color::rgb(24, 27, 34))
            .with_token("ui.foreground", Color::rgb(215, 221, 232))
            .with_token("syntax.attribute", Color::rgb(97, 175, 239))
            .with_token("syntax.comment", Color::rgb(92, 99, 112))
            .with_token("syntax.constant", Color::rgb(209, 154, 102))
            .with_token("syntax.constant.builtin", Color::rgb(209, 154, 102))
            .with_token("syntax.constructor", Color::rgb(198, 120, 221))
            .with_token("syntax.function", Color::rgb(97, 175, 239))
            .with_token("syntax.function.macro", Color::rgb(86, 182, 194))
            .with_token("syntax.keyword", Color::rgb(198, 120, 221))
            .with_token("syntax.label", Color::rgb(224, 108, 117))
            .with_token("syntax.module", Color::rgb(229, 192, 123))
            .with_token("syntax.operator", Color::rgb(86, 182, 194))
            .with_token("syntax.property", Color::rgb(224, 108, 117))
            .with_token("syntax.punctuation.bracket", Color::rgb(171, 178, 191))
            .with_token("syntax.punctuation.delimiter", Color::rgb(171, 178, 191))
            .with_token("syntax.string", Color::rgb(152, 195, 121))
            .with_token("syntax.type", Color::rgb(229, 192, 123))
            .with_token("syntax.type.builtin", Color::rgb(229, 192, 123))
            .with_token("syntax.variable", Color::rgb(224, 108, 117))
            .with_token("syntax.variable.builtin", Color::rgb(224, 108, 117)),
    ]
}
