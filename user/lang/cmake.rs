use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

const EXTENSIONS: &[&str] = &["cmake"];
const FILE_NAMES: &[&str] = &["CMakeLists.txt"];

pub fn package() -> PluginPackage {
    common::package_with_path_matchers("cmake", "CMake", EXTENSIONS, FILE_NAMES, &[], &[])
}

pub fn syntax_language() -> LanguageConfiguration {
    common::syntax_language_with_path_matchers(
        "cmake",
        EXTENSIONS,
        FILE_NAMES,
        &[],
        "https://github.com/uyha/tree-sitter-cmake.git",
        "tree-sitter-cmake",
        "tree_sitter_cmake",
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn cmake_package_metadata() {
        let pkg = package();
        assert_eq!(pkg.name(), "lang-cmake");
        assert!(pkg.auto_load());
    }

    #[test]
    fn cmake_package_no_formatter() {
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
    fn cmake_package_auto_attaches_extension() {
        let pkg = package();
        assert!(
            pkg.hook_bindings()
                .iter()
                .any(|b| b.detail_filter() == Some(".cmake")),
            "missing auto-attach binding for .cmake",
        );
    }

    #[test]
    fn cmake_package_auto_attaches_cmakelists() {
        let pkg = package();
        assert!(
            pkg.hook_bindings()
                .iter()
                .any(|b| b.detail_filter() == Some("CMakeLists.txt")),
            "missing auto-attach binding for CMakeLists.txt",
        );
    }

    #[test]
    fn cmake_syntax_language_metadata() {
        let lang = syntax_language();
        assert_eq!(lang.id(), "cmake");
        let exts: Vec<&str> = lang.file_extensions().iter().map(String::as_str).collect();
        assert_eq!(exts, EXTENSIONS);
        let file_names: Vec<&str> = lang.file_names().iter().map(String::as_str).collect();
        assert_eq!(file_names, FILE_NAMES);
        let grammar = lang.grammar().expect("grammar metadata missing");
        assert_eq!(
            grammar.repository_url(),
            "https://github.com/uyha/tree-sitter-cmake.git"
        );
        assert_eq!(grammar.install_dir_name(), "tree-sitter-cmake");
        assert_eq!(grammar.symbol_name(), "tree_sitter_cmake");
        assert_eq!(grammar.grammar_dir(), Path::new("."));
        assert_eq!(grammar.source_dir(), Path::new("src"));
    }
}
