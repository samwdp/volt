use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// Python language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("python", "Python", &["py"], &["python|black"])
}

/// Returns the syntax registration for the Python tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "python",
        &["py"],
        "https://github.com/tree-sitter/tree-sitter-python.git",
        "tree-sitter-python",
        "tree_sitter_python",
    )
}
