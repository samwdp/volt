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
            PluginAction::emit_hook(
                "workspace.formatter.register",
                Some("markdown|prettier|--write"),
            ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_auto_attaches_markdown_extensions_and_formatter() {
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
                .any(|binding| binding.detail_filter() == Some(".md"))
        );
        assert!(
            package
                .hook_bindings()
                .iter()
                .any(|binding| binding.detail_filter() == Some(".markdown"))
        );
        assert_eq!(formatter_details, vec!["markdown|prettier|--write"]);
    }

    #[test]
    fn syntax_languages_register_markdown_grammars() {
        let markdown = syntax_language();
        let markdown_grammar = markdown.grammar().expect("markdown grammar missing");
        let inline = inline_syntax_language();
        let inline_grammar = inline.grammar().expect("markdown inline grammar missing");

        assert_eq!(markdown.id(), "markdown");
        assert_eq!(markdown.file_extensions(), ["md", "markdown"]);
        assert_eq!(markdown_grammar.install_dir_name(), "tree-sitter-markdown");
        assert_eq!(inline.id(), "markdown-inline");
        assert!(inline.file_extensions().is_empty());
        assert_eq!(
            inline_grammar.install_dir_name(),
            "tree-sitter-markdown-inline"
        );
        assert_eq!(
            markdown.additional_highlight_languages(),
            ["markdown-inline"]
        );
    }
}
