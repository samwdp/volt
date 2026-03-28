use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

use super::{javascript, web_queries};

/// Returns the metadata for the TypeScript and TSX language package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-typescript",
        true,
        "TypeScript and TSX defaults, tree-sitter mapping, and startup hooks.",
    )
    .with_commands(vec![PluginCommand::new(
        "lang-typescript.attach",
        "Attaches TypeScript and TSX language defaults to the active workspace.",
        vec![
            PluginAction::log_message("TypeScript language package attached."),
            PluginAction::emit_hook("workspace.formatter.register", Some("typescript|prettier")),
            PluginAction::emit_hook("workspace.formatter.register", Some("tsx|prettier")),
            PluginAction::emit_hook("lang.typescript.attached", Some("typescript")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lang.typescript.attached",
        "Runs after the TypeScript language package attaches to a buffer.",
    )])
    .with_hook_bindings(vec![
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-typescript.auto-attach-ts",
            "lang-typescript.attach",
            Some(".ts"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-typescript.auto-attach-tsx",
            "lang-typescript.attach",
            Some(".tsx"),
        ),
    ])
}

fn capture_mappings(include_jsx: bool) -> Vec<CaptureThemeMapping> {
    let mut mappings = javascript::capture_mappings(include_jsx);
    mappings.extend([
        CaptureThemeMapping::new("type", "syntax.type"),
        CaptureThemeMapping::new("type.builtin", "syntax.type.builtin"),
    ]);
    mappings
}

/// Returns the syntax registration for the TypeScript tree-sitter language.
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "typescript",
        ["ts"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-typescript.git",
            ".",
            "typescript/src",
            "tree-sitter-typescript",
            "tree_sitter_typescript",
        ),
        capture_mappings(false),
    )
    .with_extra_highlight_query(web_queries::typescript_extra_highlight_query(false))
}

/// Returns the syntax registration for the TSX tree-sitter language.
pub fn tsx_syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "tsx",
        ["tsx"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-typescript.git",
            ".",
            "tsx/src",
            "tree-sitter-tsx",
            "tree_sitter_tsx",
        ),
        capture_mappings(true),
    )
    .with_extra_highlight_query(web_queries::typescript_extra_highlight_query(true))
}
