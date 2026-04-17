use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// Razor language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package(
        "razor",
        "Razor",
        &["cshtml", "razor"],
        &["html|prettier|--write|--tab-width 2"],
    )
}

/// Returns the syntax registration for the Razor tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "razor",
        &["cshtml", "razor"],
        "https://github.com/tris203/tree-sitter-razor",
        "tree-sitter-razor",
        "tree_sitter_razor",
    )
}
