use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

/// Returns the metadata for the Markdown language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-markdown",
        true,
        "Markdown language defaults and tree-sitter mapping.",
    )
    .with_commands(vec![PluginCommand::new(
        "lang-markdown.attach",
        "Attaches Markdown language defaults to the active workspace.",
        vec![
            PluginAction::log_message("Markdown language package attached."),
            PluginAction::emit_hook("lang.markdown.attached", Some("markdown")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lang.markdown.attached",
        "Runs after the Markdown language package attaches to a buffer.",
    )])
    .with_hook_bindings(vec![
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-markdown.auto-attach",
            "lang-markdown.attach",
            Some(".md"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-markdown.auto-attach-markdown",
            "lang-markdown.attach",
            Some(".markdown"),
        ),
    ])
}

/// Returns the syntax registration for the Markdown block grammar.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "markdown",
        ["md", "markdown"],
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
            ".",
            "tree-sitter-markdown/src",
            "tree-sitter-markdown",
            "tree_sitter_markdown",
        ),
        [
            CaptureThemeMapping::new("text.title", "syntax.text.title"),
            CaptureThemeMapping::new("text.literal", "syntax.text.literal"),
            CaptureThemeMapping::new("text.uri", "syntax.text.uri"),
            CaptureThemeMapping::new("text.reference", "syntax.text.reference"),
            CaptureThemeMapping::new("punctuation.special", "syntax.punctuation.special"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("string.escape", "syntax.string.escape"),
        ],
    )
    .with_additional_highlight_languages(["markdown-inline"])
}

/// Returns the syntax registration for the Markdown inline grammar.
pub fn inline_syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "markdown-inline",
        [] as [&str; 0],
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
            ".",
            "tree-sitter-markdown-inline/src",
            "tree-sitter-markdown-inline",
            "tree_sitter_markdown_inline",
        ),
        [
            CaptureThemeMapping::new("text.literal", "syntax.text.literal"),
            CaptureThemeMapping::new("text.emphasis", "syntax.text.emphasis"),
            CaptureThemeMapping::new("text.strong", "syntax.text.strong"),
            CaptureThemeMapping::new("text.uri", "syntax.text.uri"),
            CaptureThemeMapping::new("text.reference", "syntax.text.reference"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("string.escape", "syntax.string.escape"),
        ],
    )
}
