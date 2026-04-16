use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common::{self, GrammarSourceSpec};

const EXTENSIONS: &[&str] = &["xml", "svg", "xsd", "xslt", "xsl", "rng"];

/// XML language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package(
        "xml",
        "XML",
        EXTENSIONS,
        &["xml|prettier|--plugin=@prettier/plugin-xml|--write"],
    )
}

/// Returns the syntax registration for the XML tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language_with_source_and_path_matchers(
        "xml",
        EXTENSIONS,
        &[],
        &[],
        GrammarSourceSpec::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-xml.git",
            "tree-sitter-xml",
            "tree_sitter_xml",
        )
        .with_source_paths("xml", "src"),
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn xml_package_metadata() {
        let pkg = package();
        assert_eq!(pkg.name(), "lang-xml");
        assert!(pkg.auto_load());
    }

    #[test]
    fn xml_package_registers_formatter() {
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
            &["xml|prettier|--plugin=@prettier/plugin-xml|--write"]
        );
    }

    #[test]
    fn xml_package_auto_attaches_all_extensions() {
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
    fn xml_syntax_language_metadata() {
        let lang = syntax_language();
        assert_eq!(lang.id(), "xml");
        let exts: Vec<&str> = lang.file_extensions().iter().map(String::as_str).collect();
        assert_eq!(exts, EXTENSIONS);
        let grammar = lang.grammar().expect("grammar metadata missing");
        assert_eq!(
            grammar.repository_url(),
            "https://github.com/tree-sitter-grammars/tree-sitter-xml.git"
        );
        assert_eq!(grammar.install_dir_name(), "tree-sitter-xml");
        assert_eq!(grammar.symbol_name(), "tree_sitter_xml");
        assert_eq!(grammar.grammar_dir(), Path::new("xml"));
        assert_eq!(grammar.source_dir(), Path::new("src"));
    }
}
