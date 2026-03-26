use editor_syntax::LanguageConfiguration;

/// Git commit language support and theme mappings.
/// C# language support and theme mappings.
pub mod csharp;
/// Git commit language support and theme mappings.
pub mod gitcommit;
/// JavaScript language support and theme mappings.
pub mod javascript;
/// JSON language support and theme mappings.
pub mod json;
/// Markdown language support and theme mappings.
pub mod markdown;
/// Rust language support and theme mappings.
pub mod rust;
/// TOML language support and theme mappings.
pub mod toml;
/// TypeScript language support and theme mappings.
pub mod typescript;
/// YAML language support and theme mappings.
pub mod yaml;

mod web_queries;

/// Returns syntax languages compiled into the user library.
pub fn syntax_languages() -> Vec<LanguageConfiguration> {
    vec![
        csharp::syntax_language(),
        rust::syntax_language(),
        gitcommit::syntax_language(),
        javascript::syntax_language(),
        javascript::jsx_syntax_language(),
        json::syntax_language(),
        markdown::syntax_language(),
        markdown::inline_syntax_language(),
        toml::syntax_language(),
        typescript::syntax_language(),
        typescript::tsx_syntax_language(),
        yaml::syntax_language(),
    ]
}
