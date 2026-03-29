use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

/// Make language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package("make", "Make", &["mk", "mak", "make"], &[])
}

/// Returns the syntax registration for the Make tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language(
        "make",
        &["mk", "mak", "make"],
        "https://github.com/tree-sitter-grammars/tree-sitter-make.git",
        "tree-sitter-make",
        "tree_sitter_make",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_auto_attaches_make_extensions_without_formatter() {
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
                .any(|binding| binding.detail_filter() == Some(".mk"))
        );
        assert!(
            package
                .hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some(".mak"))
        );
        assert!(
            package
                .hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some(".make"))
        );
        assert!(formatter_details.is_empty());
    }
}
