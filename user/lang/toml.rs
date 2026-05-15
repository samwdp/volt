use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, Language, LanguageConfiguration};

fn toml_language() -> Language {
    tree_sitter_toml_ng::LANGUAGE.into()
}

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
    LanguageConfiguration::new(
        "toml",
        ["toml"],
        toml_language,
        tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
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
    fn syntax_language_registers_toml_language() {
        let language = syntax_language();

        assert_eq!(language.id(), "toml");
        assert_eq!(language.file_extensions(), ["toml"]);
        assert!(
            language.grammar().is_none(),
            "TOML now uses a pinned static grammar crate"
        );
    }
}
