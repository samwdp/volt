use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

/// Returns the metadata for the JSON language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-json",
        true,
        "JSON language defaults, tree-sitter mapping, and startup hooks.",
    )
    .with_commands(vec![PluginCommand::new(
        "lang-json.attach",
        "Attaches JSON language defaults to the active workspace.",
        vec![
            PluginAction::log_message("JSON language package attached."),
            PluginAction::emit_hook(
                "workspace.formatter.register",
                Some("json|prettier|--write|--tab-width 2"),
            ),
            PluginAction::emit_hook("lang.json.attached", Some("json")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lang.json.attached",
        "Runs after the JSON language package attaches to a buffer.",
    )])
    .with_hook_bindings(vec![PluginHookBinding::new(
        "buffer.file-open",
        "lang-json.auto-attach",
        "lang-json.attach",
        Some(".json"),
    )])
}

/// Returns the syntax registration for the JSON tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "json",
        ["json"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-json.git",
            ".",
            "src",
            "tree-sitter-json",
            "tree_sitter_json",
        ),
        [
            CaptureThemeMapping::new("comment", "syntax.comment"),
            CaptureThemeMapping::new("constant.builtin", "syntax.constant.builtin"),
            CaptureThemeMapping::new("escape", "syntax.string.escape"),
            CaptureThemeMapping::new("number", "syntax.number"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("string.special.key", "syntax.property"),
        ],
    )
}
