use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

/// Returns the metadata for the Rust language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-rust",
        true,
        "Rust language defaults, tree-sitter mapping, and startup hooks.",
    )
    .with_commands(vec![PluginCommand::new(
        "lang-rust.attach",
        "Attaches Rust language defaults to the active workspace.",
        vec![
            PluginAction::log_message("Rust language package attached."),
            PluginAction::emit_hook("workspace.formatter.register", Some("rust|rustfmt")),
            PluginAction::emit_hook("lang.rust.attached", Some("rust")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lang.rust.attached",
        "Runs after the Rust language package attaches to a buffer.",
    )])
    .with_hook_bindings(vec![PluginHookBinding::new(
        "buffer.file-open",
        "lang-rust.auto-attach",
        "lang-rust.attach",
        Some(".rs"),
    )])
}

/// Returns the syntax registration for the Rust tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "rust",
        ["rs"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-rust.git",
            ".",
            "src",
            "tree-sitter-rust",
            "tree_sitter_rust",
        ),
        [
            CaptureThemeMapping::new("attribute", "syntax.attribute"),
            CaptureThemeMapping::new("comment", "syntax.comment"),
            CaptureThemeMapping::new("constant", "syntax.constant"),
            CaptureThemeMapping::new("constant.builtin", "syntax.constant.builtin"),
            CaptureThemeMapping::new("constructor", "syntax.constructor"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("function.macro", "syntax.function.macro"),
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("label", "syntax.label"),
            CaptureThemeMapping::new("module", "syntax.module"),
            CaptureThemeMapping::new("operator", "syntax.operator"),
            CaptureThemeMapping::new("property", "syntax.property"),
            CaptureThemeMapping::new("punctuation.bracket", "syntax.punctuation.bracket"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("type", "syntax.type"),
            CaptureThemeMapping::new("type.builtin", "syntax.type.builtin"),
            CaptureThemeMapping::new("variable", "syntax.variable"),
            CaptureThemeMapping::new("variable.builtin", "syntax.variable.builtin"),
        ],
    )
}
