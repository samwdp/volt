use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

use super::web_queries;

/// Returns the metadata for the JavaScript and JSX language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-javascript",
        true,
        "JavaScript and JSX defaults, tree-sitter mapping, and startup hooks.",
    )
    .with_commands(vec![PluginCommand::new(
        "lang-javascript.attach",
        "Attaches JavaScript and JSX language defaults to the active workspace.",
        vec![
            PluginAction::log_message("JavaScript language package attached."),
            PluginAction::emit_hook(
                "workspace.formatter.register",
                Some("javascript|prettier|--write"),
            ),
            PluginAction::emit_hook("workspace.formatter.register", Some("jsx|prettier|--write")),
            PluginAction::emit_hook("lang.javascript.attached", Some("javascript")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lang.javascript.attached",
        "Runs after the JavaScript language package attaches to a buffer.",
    )])
    .with_hook_bindings(vec![
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-javascript.auto-attach-js",
            "lang-javascript.attach",
            Some(".js"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-javascript.auto-attach-jsx",
            "lang-javascript.attach",
            Some(".jsx"),
        ),
    ])
}

pub(crate) fn capture_mappings(include_jsx: bool) -> Vec<CaptureThemeMapping> {
    let mut mappings = vec![
        CaptureThemeMapping::new("comment", "syntax.comment"),
        CaptureThemeMapping::new("constant", "syntax.constant"),
        CaptureThemeMapping::new("constant.builtin", "syntax.constant.builtin"),
        CaptureThemeMapping::new("constructor", "syntax.constructor"),
        CaptureThemeMapping::new("embedded", "syntax.none"),
        CaptureThemeMapping::new("function", "syntax.function"),
        CaptureThemeMapping::new("function.builtin", "syntax.function.builtin"),
        CaptureThemeMapping::new("function.method", "syntax.function.method"),
        CaptureThemeMapping::new("keyword", "syntax.keyword"),
        CaptureThemeMapping::new("number", "syntax.number"),
        CaptureThemeMapping::new("operator", "syntax.operator"),
        CaptureThemeMapping::new("property", "syntax.property"),
        CaptureThemeMapping::new("punctuation.bracket", "syntax.punctuation.bracket"),
        CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
        CaptureThemeMapping::new("punctuation.special", "syntax.punctuation.special"),
        CaptureThemeMapping::new("string", "syntax.string"),
        CaptureThemeMapping::new("string.special", "syntax.string.special"),
        CaptureThemeMapping::new("variable", "syntax.variable"),
        CaptureThemeMapping::new("variable.builtin", "syntax.variable.builtin"),
        CaptureThemeMapping::new("variable.parameter", "syntax.variable.parameter"),
    ];
    if include_jsx {
        mappings.extend([
            CaptureThemeMapping::new("attribute", "syntax.tag.attribute"),
            CaptureThemeMapping::new("tag", "syntax.tag"),
        ]);
    }
    mappings
}

/// Returns the syntax registration for the JavaScript tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "javascript",
        ["js"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-javascript.git",
            ".",
            "src",
            "tree-sitter-javascript",
            "tree_sitter_javascript",
        ),
        capture_mappings(false),
    )
    .with_extra_highlight_query(web_queries::javascript_extra_highlight_query(false))
}

/// Returns the syntax registration for the JSX tree-sitter language.
pub fn jsx_syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "jsx",
        ["jsx"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-javascript.git",
            ".",
            "src",
            "tree-sitter-javascript",
            "tree_sitter_javascript",
        ),
        capture_mappings(true),
    )
    .with_extra_highlight_query(web_queries::javascript_extra_highlight_query(true))
}
