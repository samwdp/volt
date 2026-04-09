use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;

use super::common;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SimpleLanguageDefinition {
    id: &'static str,
    display_name: &'static str,
    extensions: &'static [&'static str],
    file_names: &'static [&'static str],
    file_globs: &'static [&'static str],
    repository: &'static str,
    grammar_dir: &'static str,
    source_dir: &'static str,
    install_dir_name: &'static str,
    symbol_name: &'static str,
    formatters: &'static [&'static str],
}

impl SimpleLanguageDefinition {
    const fn new(
        id: &'static str,
        display_name: &'static str,
        extensions: &'static [&'static str],
        repository: &'static str,
        install_dir_name: &'static str,
        symbol_name: &'static str,
        formatters: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            display_name,
            extensions,
            file_names: &[],
            file_globs: &[],
            repository,
            grammar_dir: ".",
            source_dir: "src",
            install_dir_name,
            symbol_name,
            formatters,
        }
    }

    const fn with_file_names(mut self, file_names: &'static [&'static str]) -> Self {
        self.file_names = file_names;
        self
    }

    const fn with_source_paths(
        mut self,
        grammar_dir: &'static str,
        source_dir: &'static str,
    ) -> Self {
        self.grammar_dir = grammar_dir;
        self.source_dir = source_dir;
        self
    }

    fn package(self) -> PluginPackage {
        common::package_with_path_matchers(
            self.id,
            self.display_name,
            self.extensions,
            self.file_names,
            self.file_globs,
            self.formatters,
        )
    }

    fn syntax_language(self) -> LanguageConfiguration {
        common::syntax_language_with_source_and_path_matchers(
            self.id,
            self.extensions,
            self.file_names,
            self.file_globs,
            self.repository,
            self.grammar_dir,
            self.source_dir,
            self.install_dir_name,
            self.symbol_name,
        )
    }
}

const SIMPLE_LANGUAGES: &[SimpleLanguageDefinition] = &[
    SimpleLanguageDefinition::new(
        "bash",
        "Bash",
        &["sh", "bash", "zsh", "ksh", "ash", "dash", "mksh"],
        "https://github.com/tree-sitter/tree-sitter-bash.git",
        "tree-sitter-bash",
        "tree_sitter_bash",
        &["bash|shfmt|-w"],
    ),
    SimpleLanguageDefinition::new(
        "clojure",
        "Clojure",
        &["clj", "cljs", "cljc", "edn"],
        "https://github.com/sogaiu/tree-sitter-clojure.git",
        "tree-sitter-clojure",
        "tree_sitter_clojure",
        &[],
    ),
    SimpleLanguageDefinition::new(
        "cmake",
        "CMake",
        &["cmake"],
        "https://github.com/uyha/tree-sitter-cmake.git",
        "tree-sitter-cmake",
        "tree_sitter_cmake",
        &[],
    )
    .with_file_names(&["CMakeLists.txt"]),
    SimpleLanguageDefinition::new(
        "elixir",
        "Elixir",
        &["ex", "exs"],
        "https://github.com/elixir-lang/tree-sitter-elixir.git",
        "tree-sitter-elixir",
        "tree_sitter_elixir",
        &["elixir|mix|format"],
    ),
    SimpleLanguageDefinition::new(
        "graphql",
        "GraphQL",
        &["gql", "graphql", "graphqls"],
        "https://github.com/bkegley/tree-sitter-graphql.git",
        "tree-sitter-graphql",
        "tree_sitter_graphql",
        &["graphql|prettier|--write"],
    ),
    SimpleLanguageDefinition::new(
        "hcl",
        "HCL",
        &["hcl", "tf", "nomad"],
        "https://github.com/tree-sitter-grammars/tree-sitter-hcl.git",
        "tree-sitter-hcl",
        "tree_sitter_hcl",
        &[],
    ),
    SimpleLanguageDefinition::new(
        "java",
        "Java",
        &["java", "jav", "pde"],
        "https://github.com/tree-sitter/tree-sitter-java.git",
        "tree-sitter-java",
        "tree_sitter_java",
        &["java|google-java-format|-i"],
    ),
    SimpleLanguageDefinition::new(
        "kotlin",
        "Kotlin",
        &["kt", "kts"],
        "https://github.com/fwcd/tree-sitter-kotlin.git",
        "tree-sitter-kotlin",
        "tree_sitter_kotlin",
        &["kotlin|ktlint|-F"],
    ),
    SimpleLanguageDefinition::new(
        "latex",
        "LaTeX",
        &["tex", "dtx", "ins", "sty", "cls", "rd", "bbx", "cbx"],
        "https://github.com/latex-lsp/tree-sitter-latex.git",
        "tree-sitter-latex",
        "tree_sitter_latex",
        &["latex|latexindent|-w"],
    ),
    SimpleLanguageDefinition::new(
        "lua",
        "Lua",
        &["lua", "rockspec"],
        "https://github.com/tree-sitter-grammars/tree-sitter-lua.git",
        "tree-sitter-lua",
        "tree_sitter_lua",
        &["lua|stylua"],
    ),
    SimpleLanguageDefinition::new(
        "nix",
        "Nix",
        &["nix"],
        "https://github.com/nix-community/tree-sitter-nix.git",
        "tree-sitter-nix",
        "tree_sitter_nix",
        &["nix|nixfmt"],
    ),
    SimpleLanguageDefinition::new(
        "perl",
        "Perl",
        &["pl", "pm", "t", "psgi"],
        "https://github.com/tree-sitter-perl/tree-sitter-perl.git",
        "tree-sitter-perl",
        "tree_sitter_perl",
        &["perl|perltidy|-b"],
    ),
    SimpleLanguageDefinition::new(
        "php",
        "PHP",
        &["php", "inc", "php4", "php5", "phtml", "ctp"],
        "https://github.com/tree-sitter/tree-sitter-php.git",
        "tree-sitter-php",
        "tree_sitter_php",
        &[],
    ),
    SimpleLanguageDefinition::new(
        "proto",
        "Protocol Buffers",
        &["proto"],
        "https://github.com/mitchellh/tree-sitter-proto.git",
        "tree-sitter-proto",
        "tree_sitter_proto",
        &["proto|buf|format|-w"],
    ),
    SimpleLanguageDefinition::new(
        "r",
        "R",
        &["r"],
        "https://github.com/r-lib/tree-sitter-r.git",
        "tree-sitter-r",
        "tree_sitter_r",
        &[],
    ),
    SimpleLanguageDefinition::new(
        "ruby",
        "Ruby",
        &[
            "rb", "rake", "irb", "gemspec", "rabl", "jbuilder", "jb", "podspec", "rjs", "rbi",
            "rbs",
        ],
        "https://github.com/tree-sitter/tree-sitter-ruby.git",
        "tree-sitter-ruby",
        "tree_sitter_ruby",
        &[],
    ),
    SimpleLanguageDefinition::new(
        "scala",
        "Scala",
        &["scala", "sbt", "sc"],
        "https://github.com/tree-sitter/tree-sitter-scala.git",
        "tree-sitter-scala",
        "tree_sitter_scala",
        &["scala|scalafmt|-i"],
    ),
    SimpleLanguageDefinition::new(
        "solidity",
        "Solidity",
        &["sol"],
        "https://github.com/JoranHonig/tree-sitter-solidity.git",
        "tree-sitter-solidity",
        "tree_sitter_solidity",
        &["solidity|prettier|--plugin=prettier-plugin-solidity|--write"],
    ),
    SimpleLanguageDefinition::new(
        "swift",
        "Swift",
        &["swift", "swiftinterface"],
        "https://github.com/alex-pinkus/tree-sitter-swift.git",
        "tree-sitter-swift",
        "tree_sitter_swift",
        &["swift|swift-format|-i"],
    ),
    SimpleLanguageDefinition::new(
        "vim",
        "Vim",
        &["vim"],
        "https://github.com/tree-sitter-grammars/tree-sitter-vim.git",
        "tree-sitter-vim",
        "tree_sitter_vim",
        &[],
    ),
    SimpleLanguageDefinition::new(
        "xml",
        "XML",
        &["xml", "svg", "xsd", "xslt", "xsl", "rng"],
        "https://github.com/tree-sitter-grammars/tree-sitter-xml.git",
        "tree-sitter-xml",
        "tree_sitter_xml",
        &["xml|prettier|--plugin=@prettier/plugin-xml|--write"],
    )
    .with_source_paths("xml", "src"),
];

pub(super) fn packages() -> Vec<PluginPackage> {
    SIMPLE_LANGUAGES
        .iter()
        .copied()
        .map(SimpleLanguageDefinition::package)
        .collect()
}

pub(super) fn syntax_languages() -> Vec<LanguageConfiguration> {
    SIMPLE_LANGUAGES
        .iter()
        .copied()
        .map(SimpleLanguageDefinition::syntax_language)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn formatter_details(package: &PluginPackage) -> Vec<&str> {
        package
            .commands()
            .iter()
            .flat_map(|command| command.actions())
            .filter_map(|action| action.hook())
            .filter(|hook| hook.hook_name() == "workspace.formatter.register")
            .filter_map(|hook| hook.detail())
            .collect()
    }

    #[test]
    fn packages_register_expected_bindings_and_formatters() {
        for definition in SIMPLE_LANGUAGES {
            let package = definition.package();

            assert_eq!(package.name(), format!("lang-{}", definition.id));
            assert!(package.auto_load());
            assert_eq!(
                formatter_details(&package).as_slice(),
                definition.formatters
            );
            for extension in definition.extensions {
                let expected_filter = format!(".{extension}");
                assert!(
                    package
                        .hook_bindings()
                        .iter()
                        .any(|binding| binding.detail_filter() == Some(expected_filter.as_str())),
                    "missing auto-attach binding for {expected_filter}",
                );
            }
            for file_name in definition.file_names {
                assert!(
                    package
                        .hook_bindings()
                        .iter()
                        .any(|binding| binding.detail_filter() == Some(*file_name)),
                    "missing auto-attach binding for {file_name}",
                );
            }
            for file_glob in definition.file_globs {
                assert!(
                    package
                        .hook_bindings()
                        .iter()
                        .any(|binding| binding.detail_filter() == Some(*file_glob)),
                    "missing auto-attach binding for {file_glob}",
                );
            }
        }
    }

    #[test]
    fn syntax_languages_register_expected_grammar_metadata() {
        for definition in SIMPLE_LANGUAGES {
            let language = definition.syntax_language();
            let grammar = language
                .grammar()
                .expect("simple language grammar metadata missing");
            let extensions = language
                .file_extensions()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let file_names = language
                .file_names()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let file_globs = language
                .file_globs()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();

            assert_eq!(language.id(), definition.id);
            assert_eq!(extensions.as_slice(), definition.extensions);
            assert_eq!(file_names.as_slice(), definition.file_names);
            assert_eq!(file_globs.as_slice(), definition.file_globs);
            assert_eq!(grammar.repository_url(), definition.repository);
            assert_eq!(grammar.grammar_dir(), Path::new(definition.grammar_dir));
            assert_eq!(grammar.source_dir(), Path::new(definition.source_dir));
            assert_eq!(grammar.install_dir_name(), definition.install_dir_name);
            assert_eq!(grammar.symbol_name(), definition.symbol_name);
        }
    }

    #[test]
    fn registry_exports_all_curated_simple_languages_once() {
        let packages = packages();
        let package_names = packages.iter().map(PluginPackage::name).collect::<Vec<_>>();
        let syntax_languages = syntax_languages();
        let language_ids = syntax_languages
            .iter()
            .map(LanguageConfiguration::id)
            .collect::<Vec<_>>();
        let expected_package_names = SIMPLE_LANGUAGES
            .iter()
            .map(|definition| format!("lang-{}", definition.id))
            .collect::<Vec<_>>();
        let expected_language_ids = SIMPLE_LANGUAGES
            .iter()
            .map(|definition| definition.id)
            .collect::<Vec<_>>();

        assert_eq!(package_names, expected_package_names);
        assert_eq!(language_ids, expected_language_ids);
    }
}
