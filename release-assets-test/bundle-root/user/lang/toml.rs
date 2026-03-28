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
