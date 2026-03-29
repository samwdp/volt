use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// HTML language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("html", "HTML", &["html", "htm"], &["html|prettier|--write"])
}

/// Returns the syntax registration for the HTML tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "html",
        &["html", "htm"],
        "https://github.com/tree-sitter/tree-sitter-html.git",
        "tree-sitter-html",
        "tree_sitter_html",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_registers_expected_html_bindings_and_formatter() {
        let package = package();
        let formatter_details = package
            .commands()
            .iter()
            .flat_map(|command| command.actions())
            .filter_map(|action| action.hook())
            .filter(|hook| hook.hook_name() == "workspace.formatter.register")
            .filter_map(|hook| hook.detail())
            .collect::<Vec<_>>();

        assert!(
            package
                .hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some(".html"))
        );
        assert!(
            package
                .hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some(".htm"))
        );
        assert_eq!(formatter_details, vec!["html|prettier|--write"]);
    }
}
