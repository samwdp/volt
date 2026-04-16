use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

const EXTENSIONS: &[&str] = &["sol"];

/// Solidity language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package(
        "solidity",
        "Solidity",
        EXTENSIONS,
        &["solidity|prettier|--plugin=prettier-plugin-solidity|--write"],
    )
}

/// Returns the syntax registration for the Solidity tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "solidity",
        EXTENSIONS,
        "https://github.com/JoranHonig/tree-sitter-solidity.git",
        "tree-sitter-solidity",
        "tree_sitter_solidity",
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn solidity_package_metadata() {
        let pkg = package();
        assert_eq!(pkg.name(), "lang-solidity");
        assert!(pkg.auto_load());
    }

    #[test]
    fn solidity_package_registers_formatter() {
        let pkg = package();
        let formatter_details: Vec<&str> = pkg
            .commands()
            .iter()
            .flat_map(|cmd| cmd.actions())
            .filter_map(|action| action.hook())
            .filter(|hook| hook.hook_name() == "workspace.formatter.register")
            .filter_map(|hook| hook.detail())
            .collect();
        assert_eq!(
            formatter_details,
            &["solidity|prettier|--plugin=prettier-plugin-solidity|--write"]
        );
    }

    #[test]
    fn solidity_package_auto_attaches_all_extensions() {
        let pkg = package();
        for ext in EXTENSIONS {
            let expected = format!(".{ext}");
            assert!(
                pkg.hook_bindings()
                    .iter()
                    .any(|b| b.detail_filter() == Some(expected.as_str())),
                "missing auto-attach binding for {expected}",
            );
        }
    }

    #[test]
    fn solidity_syntax_language_metadata() {
        let lang = syntax_language();
        assert_eq!(lang.id(), "solidity");
        let exts: Vec<&str> = lang.file_extensions().iter().map(String::as_str).collect();
        assert_eq!(exts, EXTENSIONS);
        let grammar = lang.grammar().expect("grammar metadata missing");
        assert_eq!(
            grammar.repository_url(),
            "https://github.com/JoranHonig/tree-sitter-solidity.git"
        );
        assert_eq!(grammar.install_dir_name(), "tree-sitter-solidity");
        assert_eq!(grammar.symbol_name(), "tree_sitter_solidity");
        assert_eq!(grammar.grammar_dir(), Path::new("."));
        assert_eq!(grammar.source_dir(), Path::new("src"));
    }
}
