use editor_syntax::LanguageConfiguration;

/// Rust language support and theme mappings.
pub mod rust;
/// Markdown language support and theme mappings.
pub mod markdown;

/// Returns syntax languages compiled into the user library.
pub fn syntax_languages() -> Vec<LanguageConfiguration> {
    vec![
        rust::syntax_language(),
        markdown::syntax_language(),
        markdown::inline_syntax_language(),
    ]
}
