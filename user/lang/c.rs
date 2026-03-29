use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// C language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("c", "C", &["c", "h"], &["c|clang-format|--style=file|-i"])
}

/// Returns the syntax registration for the C tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "c",
        &["c", "h"],
        "https://github.com/tree-sitter/tree-sitter-c.git",
        "tree-sitter-c",
        "tree_sitter_c",
    )
}
