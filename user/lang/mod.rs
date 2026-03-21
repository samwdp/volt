use editor_syntax::LanguageConfiguration;

/// Markdown language support and theme mappings.
pub mod markdown;
/// Rust language support and theme mappings.
pub mod rust;

/// Returns syntax languages compiled into the user library.
pub fn syntax_languages() -> Vec<LanguageConfiguration> {
    vec![
        rust::syntax_language(),
        markdown::syntax_language(),
        markdown::inline_syntax_language(),
    ]
}
