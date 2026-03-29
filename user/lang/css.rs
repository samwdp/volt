use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// CSS language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("css", "CSS", &["css"], &["css|prettier|--write"])
}

/// Returns the syntax registration for the CSS tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "css",
        &["css"],
        "https://github.com/tree-sitter/tree-sitter-css.git",
        "tree-sitter-css",
        "tree_sitter_css",
    )
}
