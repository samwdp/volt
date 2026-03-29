use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

const EXTRA_HIGHLIGHT_QUERY: &str = r#"
(comment) @comment

[
  (yaml_directive)
  (tag_directive)
  (reserved_directive)
  (directive_name)
] @keyword

[
  (boolean_scalar)
  (null_scalar)
] @constant.builtin

[
  (integer_scalar)
  (float_scalar)
  (yaml_version)
] @number

(block_mapping_pair key: (_) @property)
(flow_pair key: (_) @property)

[
  (double_quote_scalar)
  (single_quote_scalar)
  (string_scalar)
  (block_scalar)
  (directive_parameter)
] @string

(escape_sequence) @string.escape

[
  (anchor)
  (anchor_name)
  (alias)
  (alias_name)
] @string.special

[
  (tag)
  (tag_handle)
  (tag_prefix)
] @type

[
  "[" "]" "{" "}"
] @punctuation.bracket

[
  ":" "-" "," "?" "|" ">" "---" "..."
] @punctuation.delimiter
"#;

/// Returns the metadata for the YAML language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-yaml",
        true,
        "YAML language defaults, tree-sitter mapping, and startup hooks.",
    )
    .with_commands(vec![PluginCommand::new(
        "lang-yaml.attach",
        "Attaches YAML language defaults to the active workspace.",
        vec![
            PluginAction::log_message("YAML language package attached."),
            PluginAction::emit_hook(
                "workspace.formatter.register",
                Some("yaml|prettier|--write"),
            ),
            PluginAction::emit_hook("lang.yaml.attached", Some("yaml")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lang.yaml.attached",
        "Runs after the YAML language package attaches to a buffer.",
    )])
    .with_hook_bindings(vec![
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-yaml.auto-attach-yaml",
            "lang-yaml.attach",
            Some(".yaml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-yaml.auto-attach-yml",
            "lang-yaml.attach",
            Some(".yml"),
        ),
    ])
}

/// Returns the syntax registration for the YAML tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "yaml",
        ["yaml", "yml"],
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-yaml.git",
            ".",
            "src",
            "tree-sitter-yaml",
            "tree_sitter_yaml",
        ),
        [
            CaptureThemeMapping::new("comment", "syntax.comment"),
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("constant.builtin", "syntax.constant.builtin"),
            CaptureThemeMapping::new("number", "syntax.number"),
            CaptureThemeMapping::new("property", "syntax.property"),
            CaptureThemeMapping::new("punctuation.bracket", "syntax.punctuation.bracket"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("string.escape", "syntax.string.escape"),
            CaptureThemeMapping::new("string.special", "syntax.string.special"),
            CaptureThemeMapping::new("type", "syntax.type"),
        ],
    )
    .with_extra_highlight_query(EXTRA_HIGHLIGHT_QUERY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_auto_attaches_for_yaml_extensions() {
        let package = package();
        let bindings = package.hook_bindings();
        let formatter_details = package
            .commands()
            .iter()
            .flat_map(|command| command.actions())
            .filter_map(|action| action.hook())
            .filter(|hook| hook.hook_name() == "workspace.formatter.register")
            .filter_map(|hook| hook.detail())
            .collect::<Vec<_>>();

        assert!(
            bindings
                .iter()
                .any(|binding| binding.detail_filter() == Some(".yaml"))
        );
        assert!(
            bindings
                .iter()
                .any(|binding| binding.detail_filter() == Some(".yml"))
        );
        assert_eq!(formatter_details, vec!["yaml|prettier|--write"]);
    }

    #[test]
    fn syntax_language_registers_yaml_grammar() {
        let language = syntax_language();
        let grammar = language.grammar().expect("yaml grammar metadata missing");

        assert_eq!(language.id(), "yaml");
        assert_eq!(language.file_extensions(), ["yaml", "yml"]);
        assert_eq!(grammar.source_dir(), std::path::Path::new("src"));
        assert_eq!(grammar.install_dir_name(), "tree-sitter-yaml");
        assert_eq!(grammar.symbol_name(), "tree_sitter_yaml");
        assert!(language.extra_highlight_query().is_some());
    }
}
