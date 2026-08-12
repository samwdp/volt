use std::collections::BTreeMap;

use editor_plugin_api::{
    MarkdownPrettyConfig, MarkdownPrettyIcon, PluginAction, PluginCommand, PluginHookBinding,
    PluginHookDeclaration, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

/// Returns the metadata for the Markdown language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-markdown",
        true,
        "Markdown language defaults, Pretty icons, and tree-sitter mapping.",
    )
    .with_commands(vec![
        PluginCommand::new(
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
        ),
        PluginCommand::new(
            "markdown.pretty.toggle",
            "Toggles Markdown Pretty for the active buffer.",
            vec![PluginAction::emit_hook(
                "markdown.pretty.toggle",
                None::<&str>,
            )],
        ),
    ])
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
    .with_key_bindings(vec![
        PluginKeyBinding::new(
            "<leader>mp",
            "markdown.pretty.toggle",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
    ])
}

/// Default Markdown Pretty configuration (enable on, kill-switch off, icon map).
pub fn pretty_config() -> MarkdownPrettyConfig {
    MarkdownPrettyConfig {
        enabled: true,
        kill_switch_enabled: false,
        kill_switch_max_lines: 20_000,
        kill_switch_max_bytes: 2_000_000,
        image_max_bytes: 10_000_000,
        image_max_rows: 24,
        icons: default_pretty_icons(),
    }
}

/// treesitter node kind → icon map shipped as the default Pretty style.
pub fn default_pretty_icons() -> Vec<MarkdownPrettyIcon> {
    use editor_icons::symbols::{fa, md, oct};
    let entries: [(&str, &str); 22] = [
        ("atx_h1_marker", fa::FA_CIRCLE_DOT),
        ("atx_h2_marker", fa::FA_CIRCLE_THIN),
        ("atx_h3_marker", fa::FA_DIAMOND),
        ("atx_h4_marker", oct::OCT_DIAMOND),
        ("atx_h5_marker", md::MD_PAN_RIGHT),
        ("atx_h6_marker", fa::FA_ANGLES_RIGHT),
        ("list_marker_minus", md::MD_MINUS),
        ("list_marker_plus", md::MD_PLUS),
        ("list_marker_star", md::MD_STAR),
        ("list_marker_dot", md::MD_CIRCLE_SMALL),
        ("list_marker_parenthesis", md::MD_FORMAT_LIST_NUMBERED),
        ("task_list_marker_unchecked", md::MD_CHECKBOX_BLANK_OUTLINE),
        ("task_list_marker_checked", md::MD_CHECKBOX_MARKED),
        ("thematic_break", oct::OCT_HORIZONTAL_RULE),
        ("image", md::MD_IMAGE),
        ("inline_link", md::MD_LINK),
        ("full_reference_link", md::MD_LINK),
        ("collapsed_reference_link", md::MD_LINK),
        ("shortcut_link", md::MD_LINK),
        ("uri_autolink", md::MD_LINK),
        ("email_autolink", md::MD_LINK),
        ("pipe_table", "|"),
    ];
    entries
        .into_iter()
        .map(|(node_kind, icon)| MarkdownPrettyIcon {
            node_kind: node_kind.to_owned(),
            icon: icon.to_owned(),
        })
        .collect()
}

/// Icon map as a BTreeMap for host planners.
pub fn pretty_icon_map() -> BTreeMap<String, String> {
    default_pretty_icons()
        .into_iter()
        .map(|entry| (entry.node_kind, entry.icon))
        .collect()
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
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "markdown.pretty.toggle")
        );
    }

    #[test]
    fn pretty_config_ships_consistent_icon_map() {
        let config = pretty_config();
        assert!(config.enabled);
        assert!(!config.kill_switch_enabled);
        assert!(
            config
                .icons
                .iter()
                .any(|entry| entry.node_kind == "atx_h1_marker")
        );
        assert!(
            config
                .icons
                .iter()
                .any(|entry| entry.node_kind == "inline_link")
        );
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
