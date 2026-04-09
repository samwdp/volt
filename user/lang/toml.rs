use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

/// Returns the metadata for the TOML language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-toml",
        true,
        "TOML language defaults, tree-sitter mapping, and startup hooks.",
    )
    .with_commands(vec![PluginCommand::new(
        "lang-toml.attach",
        "Attaches TOML language defaults to the active workspace.",
        vec![
            PluginAction::log_message("TOML language package attached."),
            PluginAction::emit_hook("workspace.formatter.register", Some("toml|tombi|format")),
            PluginAction::emit_hook("lang.toml.attached", Some("toml")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lang.toml.attached",
        "Runs after the TOML language package attaches to a buffer.",
    )])
    .with_hook_bindings(vec![PluginHookBinding::new(
        "buffer.file-open",
        "lang-toml.auto-attach",
        "lang-toml.attach",
        Some(".toml"),
    )])
}

/// Returns the syntax registration for the TOML tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "toml",
        ["toml"],
        GrammarSource::new(
            "https://github.com/ikatyang/tree-sitter-toml.git",
            ".",
            "src",
            "tree-sitter-toml",
            "tree_sitter_toml",
        ),
        [
            CaptureThemeMapping::new("comment", "syntax.comment"),
            CaptureThemeMapping::new("constant.builtin", "syntax.constant.builtin"),
            CaptureThemeMapping::new("number", "syntax.number"),
            CaptureThemeMapping::new("operator", "syntax.operator"),
            CaptureThemeMapping::new("property", "syntax.property"),
            CaptureThemeMapping::new("punctuation.bracket", "syntax.punctuation.bracket"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("string.special", "syntax.string.special"),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn package_auto_attaches_toml_and_registers_formatter() {
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
                .any(|binding| binding.detail_filter() == Some(".toml"))
        );
        assert_eq!(formatter_details, vec!["toml|tombi|format"]);
    }

    #[test]
    fn syntax_language_registers_toml_grammar() {
        let language = syntax_language();
        let grammar = language.grammar().expect("toml grammar missing");

        assert_eq!(language.id(), "toml");
        assert_eq!(language.file_extensions(), ["toml"]);
        assert_eq!(
            grammar.repository_url(),
            "https://github.com/ikatyang/tree-sitter-toml.git"
        );
        assert_eq!(grammar.grammar_dir(), Path::new("."));
        assert_eq!(grammar.source_dir(), Path::new("src"));
        assert_eq!(grammar.install_dir_name(), "tree-sitter-toml");
        assert_eq!(grammar.symbol_name(), "tree_sitter_toml");
    }
}
