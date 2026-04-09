use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

const MAKE_FILE_NAMES: &[&str] = &["Makefile", "GNUmakefile", "makefile"];

/// Make language support and theme mappings.
pub fn package() -> PluginPackage {
    common::package_with_path_matchers(
        "make",
        "Make",
        &["mk", "mak", "make"],
        MAKE_FILE_NAMES,
        &[],
        &[],
    )
}

/// Returns the syntax registration for the Make tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language_with_path_matchers(
        "make",
        &["mk", "mak", "make"],
        MAKE_FILE_NAMES,
        &[],
        "https://github.com/tree-sitter-grammars/tree-sitter-make.git",
        "tree-sitter-make",
        "tree_sitter_make",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_auto_attaches_make_paths_without_formatter() {
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
        assert!(
            package
                .hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some("Makefile"))
        );
        assert!(
            package
                .hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some("GNUmakefile"))
        );
        assert!(
            package
                .hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some("makefile"))
        );
        assert!(formatter_details.is_empty());
    }

    #[test]
    fn syntax_language_matches_makefile_basenames() {
        let language = syntax_language();

        assert_eq!(language.file_names(), MAKE_FILE_NAMES);
        assert!(language.file_globs().is_empty());
    }
}
