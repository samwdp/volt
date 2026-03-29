use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// Go language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("go", "Go", &["go"], &["go|gofmt|-w"])
}

/// Returns the syntax registration for the Go tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "go",
        &["go"],
        "https://github.com/tree-sitter/tree-sitter-go.git",
        "tree-sitter-go",
        "tree_sitter_go",
    )
}
