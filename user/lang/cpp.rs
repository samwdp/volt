use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// C++ language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package(
        "cpp",
        "C++",
        &["cc", "cpp", "cxx", "hpp", "hh", "hxx"],
        &["cpp|clang-format|--style=file|-i"],
    )
}

/// Returns the syntax registration for the C++ tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "cpp",
        &["cc", "cpp", "cxx", "hpp", "hh", "hxx"],
        "https://github.com/tree-sitter/tree-sitter-cpp.git",
        "tree-sitter-cpp",
        "tree_sitter_cpp",
    )
}
