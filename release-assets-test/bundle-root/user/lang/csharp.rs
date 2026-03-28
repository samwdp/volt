use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

/// Returns the metadata for the C# language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-csharp",
        true,
        "C# language defaults, tree-sitter mapping, and startup hooks.",
    )
    .with_commands(vec![PluginCommand::new(
        "lang-csharp.attach",
        "Attaches C# language defaults to the active workspace.",
        vec![
            PluginAction::log_message("C# language package attached."),
            PluginAction::emit_hook("workspace.formatter.register", Some("csharp|csharpier")),
            PluginAction::emit_hook("lang.csharp.attached", Some("csharp")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lang.csharp.attached",
        "Runs after the C# language package attaches to a buffer.",
    )])
    .with_hook_bindings(vec![PluginHookBinding::new(
        "buffer.file-open",
        "lang-csharp.auto-attach",
        "lang-csharp.attach",
        Some(".cs"),
    )])
}

/// Returns the syntax registration for the C# tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "csharp",
        ["cs"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-c-sharp.git",
            ".",
            "src",
            "tree-sitter-c-sharp",
            "tree_sitter_c_sharp",
        ),
        [
            CaptureThemeMapping::new("attribute", "syntax.attribute"),
            CaptureThemeMapping::new("comment", "syntax.comment"),
            CaptureThemeMapping::new("constant.builtin", "syntax.constant.builtin"),
            CaptureThemeMapping::new("constructor", "syntax.constructor"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("module", "syntax.module"),
            CaptureThemeMapping::new("number", "syntax.number"),
            CaptureThemeMapping::new("operator", "syntax.operator"),
            CaptureThemeMapping::new("property.definition", "syntax.property"),
            CaptureThemeMapping::new("punctuation.bracket", "syntax.punctuation.bracket"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("string.escape", "syntax.string.escape"),
            CaptureThemeMapping::new("type", "syntax.type"),
            CaptureThemeMapping::new("type.builtin", "syntax.type.builtin"),
            CaptureThemeMapping::new("variable", "syntax.variable"),
            CaptureThemeMapping::new("variable.parameter", "syntax.variable.parameter"),
        ],
    )
}
