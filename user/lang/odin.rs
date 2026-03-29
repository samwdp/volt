use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// Odin language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("odin", "Odin", &["odin"], &["odin|odin|fmt"])
}

/// Returns the syntax registration for the Odin tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "odin",
        &["odin"],
        "https://github.com/tree-sitter-grammars/tree-sitter-odin.git",
        "tree-sitter-odin",
        "tree_sitter_odin",
    )
}
