use editor_syntax::LanguageConfiguration;

/// Bash language support and theme mappings.
pub mod bash;
/// C language support and theme mappings.
pub mod c;
/// Clojure language support and theme mappings.
pub mod clojure;
/// CMake language support and theme mappings.
pub mod cmake;
/// Shared helpers for simple declarative language packages.
mod common;
/// C++ language support and theme mappings.
pub mod cpp;
/// C# language support and theme mappings.
pub mod csharp;
/// CSS language support and theme mappings.
pub mod css;
/// Elixir language support and theme mappings.
pub mod elixir;
/// Git commit language support and theme mappings.
pub mod gitcommit;
/// Go language support and theme mappings.
pub mod go;
/// GraphQL language support and theme mappings.
pub mod graphql;
/// HCL language support and theme mappings.
pub mod hcl;
/// HTML language support and theme mappings.
pub mod html;
/// Java language support and theme mappings.
pub mod java;
/// JavaScript language support and theme mappings.
pub mod javascript;
/// JSON language support and theme mappings.
pub mod json;
/// Kotlin language support and theme mappings.
pub mod kotlin;
/// LaTeX language support and theme mappings.
pub mod latex;
/// Lua language support and theme mappings.
pub mod lua;
/// Make language support and theme mappings.
pub mod make;
/// Markdown language support and theme mappings.
pub mod markdown;
/// Nix language support and theme mappings.
pub mod nix;
/// Odin language support and theme mappings.
pub mod odin;
/// Perl language support and theme mappings.
pub mod perl;
/// PHP language support and theme mappings.
pub mod php;
/// Protocol Buffers language support and theme mappings.
pub mod proto;
/// Python language support and theme mappings.
pub mod python;
/// R language support and theme mappings.
pub mod r;
/// Ruby language support and theme mappings.
pub mod ruby;
/// Rust language support and theme mappings.
pub mod rust;
/// Scala language support and theme mappings.
pub mod scala;
/// SCSS language support and theme mappings.
pub mod scss;
/// Solidity language support and theme mappings.
pub mod solidity;
/// SQL language support and theme mappings.
pub mod sql;
/// Swift language support and theme mappings.
pub mod swift;
/// TOML language support and theme mappings.
pub mod toml;
/// TypeScript language support and theme mappings.
pub mod typescript;
/// Vim script language support and theme mappings.
pub mod vim;
/// XML language support and theme mappings.
pub mod xml;
/// YAML language support and theme mappings.
pub mod yaml;
/// Zig language support and theme mappings.
pub mod zig;

mod web_queries;

/// Returns plugin packages for all compiled-in languages.
pub fn packages() -> Vec<editor_plugin_api::PluginPackage> {
    vec![
        bash::package(),
        c::package(),
        clojure::package(),
        cmake::package(),
        cpp::package(),
        csharp::package(),
        css::package(),
        elixir::package(),
        go::package(),
        graphql::package(),
        hcl::package(),
        html::package(),
        java::package(),
        javascript::package(),
        json::package(),
        kotlin::package(),
        latex::package(),
        lua::package(),
        make::package(),
        markdown::package(),
        nix::package(),
        odin::package(),
        perl::package(),
        php::package(),
        proto::package(),
        python::package(),
        r::package(),
        ruby::package(),
        rust::package(),
        scala::package(),
        scss::package(),
        solidity::package(),
        sql::package(),
        swift::package(),
        toml::package(),
        typescript::package(),
        vim::package(),
        xml::package(),
        yaml::package(),
        zig::package(),
    ]
}

/// Returns syntax languages compiled into the user library.
pub fn syntax_languages() -> Vec<LanguageConfiguration> {
    vec![
        bash::syntax_language(),
        c::syntax_language(),
        clojure::syntax_language(),
        cmake::syntax_language(),
        cpp::syntax_language(),
        csharp::syntax_language(),
        css::syntax_language(),
        elixir::syntax_language(),
        gitcommit::syntax_language(),
        go::syntax_language(),
        graphql::syntax_language(),
        hcl::syntax_language(),
        html::syntax_language(),
        java::syntax_language(),
        javascript::syntax_language(),
        javascript::jsx_syntax_language(),
        json::syntax_language(),
        kotlin::syntax_language(),
        latex::syntax_language(),
        lua::syntax_language(),
        make::syntax_language(),
        markdown::syntax_language(),
        markdown::inline_syntax_language(),
        nix::syntax_language(),
        odin::syntax_language(),
        perl::syntax_language(),
        php::syntax_language(),
        proto::syntax_language(),
        python::syntax_language(),
        r::syntax_language(),
        ruby::syntax_language(),
        rust::syntax_language(),
        scala::syntax_language(),
        scss::syntax_language(),
        solidity::syntax_language(),
        sql::syntax_language(),
        swift::syntax_language(),
        toml::syntax_language(),
        typescript::syntax_language(),
        typescript::tsx_syntax_language(),
        vim::syntax_language(),
        xml::syntax_language(),
        yaml::syntax_language(),
        zig::syntax_language(),
    ]
}
