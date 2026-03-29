use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// Zig language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("zig", "Zig", &["zig"], &["zig|zig|fmt"])
}

/// Returns the syntax registration for the Zig tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "zig",
        &["zig"],
        "https://github.com/tree-sitter-grammars/tree-sitter-zig.git",
        "tree-sitter-zig",
        "tree_sitter_zig",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_language_registers_zig_grammar() {
        let language = syntax_language();
        let grammar = language.grammar().expect("zig grammar missing");

        assert_eq!(language.id(), "zig");
        assert_eq!(language.file_extensions(), ["zig"]);
        assert_eq!(grammar.install_dir_name(), "tree-sitter-zig");
        assert_eq!(grammar.symbol_name(), "tree_sitter_zig");
    }
}
