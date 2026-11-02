use editor_plugin_api::LanguageConfiguration;
use editor_plugin_api::PluginPackage;

use super::common::{self, GrammarSourceSpec};

const EXTENSIONS: &[&str] = &["tf", "tfvars", "tofu"];

/// Terraform language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("terraform", "Terraform", EXTENSIONS, &[])
}

/// Returns the syntax registration for the Terraform tree-sitter dialect.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language_with_source_and_path_matchers(
        "terraform",
        EXTENSIONS,
        &[],
        &[],
        GrammarSourceSpec::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-hcl.git",
            "tree-sitter-terraform",
            "tree_sitter_terraform",
        )
        .with_source_paths("dialects/terraform", "src"),
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn terraform_package_metadata() {
        let pkg = package();
        assert_eq!(pkg.name(), "lang-terraform");
    }

    #[test]
    fn terraform_package_no_formatter() {
        let pkg = package();
        let formatter_details: Vec<&str> = pkg
            .commands()
            .iter()
            .flat_map(|cmd| cmd.actions())
            .filter_map(|action| action.hook())
            .filter(|hook| hook.hook_name() == "workspace.formatter.register")
            .filter_map(|hook| hook.detail())
            .collect();
        assert!(formatter_details.is_empty());
    }

    #[test]
    fn terraform_package_auto_attaches_all_extensions() {
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
    fn terraform_syntax_language_metadata() {
        let lang = syntax_language();
        assert_eq!(lang.id(), "terraform");
        let exts: Vec<&str> = lang.file_extensions().iter().map(String::as_str).collect();
        assert_eq!(exts, EXTENSIONS);
        let grammar = lang.grammar().expect("grammar metadata missing");
        assert_eq!(
            grammar.repository_url(),
            "https://github.com/tree-sitter-grammars/tree-sitter-hcl.git"
        );
        assert_eq!(grammar.install_dir_name(), "tree-sitter-terraform");
        assert_eq!(grammar.symbol_name(), "tree_sitter_terraform");
        assert_eq!(grammar.grammar_dir(), Path::new("dialects/terraform"));
        assert_eq!(grammar.source_dir(), Path::new("src"));
    }
}
