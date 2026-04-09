use editor_syntax::LanguageConfiguration;

/// C language support and theme mappings.
pub mod c;
/// Shared helpers for simple declarative language packages.
mod common;
/// C++ language support and theme mappings.
pub mod cpp;
/// C# language support and theme mappings.
pub mod csharp;
/// CSS language support and theme mappings.
pub mod css;
/// Git commit language support and theme mappings.
pub mod gitcommit;
/// Go language support and theme mappings.
pub mod go;
/// HTML language support and theme mappings.
pub mod html;
/// JavaScript language support and theme mappings.
pub mod javascript;
/// JSON language support and theme mappings.
pub mod json;
/// Make language support and theme mappings.
pub mod make;
/// Markdown language support and theme mappings.
pub mod markdown;
/// Odin language support and theme mappings.
pub mod odin;
/// Python language support and theme mappings.
pub mod python;
/// Rust language support and theme mappings.
pub mod rust;
/// SCSS language support and theme mappings.
pub mod scss;
/// Curated declarative grammar-backed languages.
mod simple;
/// SQL language support and theme mappings.
pub mod sql;
/// TOML language support and theme mappings.
pub mod toml;
/// TypeScript language support and theme mappings.
pub mod typescript;
/// YAML language support and theme mappings.
pub mod yaml;
/// Zig language support and theme mappings.
pub mod zig;

mod web_queries;

/// Returns plugin packages for all compiled-in languages.
/// Custom languages keep their own modules, while curated simple grammars live in
/// `user/lang/simple.rs` and are merged here without touching `user/lib.rs`.
pub fn packages() -> Vec<editor_plugin_api::PluginPackage> {
    let mut packages = vec![
        c::package(),
        cpp::package(),
        csharp::package(),
        css::package(),
        go::package(),
        html::package(),
        javascript::package(),
        json::package(),
        make::package(),
        markdown::package(),
        odin::package(),
        python::package(),
        rust::package(),
        scss::package(),
        sql::package(),
        toml::package(),
        typescript::package(),
        yaml::package(),
        zig::package(),
    ];
    packages.extend(simple::packages());
    packages
}

/// Returns syntax languages compiled into the user library.
pub fn syntax_languages() -> Vec<LanguageConfiguration> {
    let mut languages = vec![
        c::syntax_language(),
        cpp::syntax_language(),
        csharp::syntax_language(),
        css::syntax_language(),
        gitcommit::syntax_language(),
        go::syntax_language(),
        html::syntax_language(),
        javascript::syntax_language(),
        javascript::jsx_syntax_language(),
        json::syntax_language(),
        make::syntax_language(),
        markdown::syntax_language(),
        markdown::inline_syntax_language(),
        odin::syntax_language(),
        python::syntax_language(),
        rust::syntax_language(),
        scss::syntax_language(),
        sql::syntax_language(),
        toml::syntax_language(),
        typescript::syntax_language(),
        typescript::tsx_syntax_language(),
        yaml::syntax_language(),
        zig::syntax_language(),
    ];
    languages.extend(simple::syntax_languages());
    languages
}
