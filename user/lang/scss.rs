use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// SCSS language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("scss", "SCSS", &["scss"], &["scss|prettier|--write"])
}

/// Returns the syntax registration for the SCSS tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "scss",
        &["scss"],
        "https://github.com/serenadeai/tree-sitter-scss.git",
        "tree-sitter-scss",
        "tree_sitter_scss",
    )
}
