use editor_lsp::LanguageServerSpec;
use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};

/// Returns the metadata for the LSP integration package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lsp",
        true,
        "Language server integration and startup hooks.",
    )
    .with_commands(vec![PluginCommand::new(
        "lsp.start-rust-analyzer",
        "Starts the Rust language server through the host runtime.",
        vec![
            PluginAction::log_message("Starting rust-analyzer from the user LSP package."),
            PluginAction::emit_hook("lsp.server-start", Some("rust-analyzer")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "lsp.server-start",
        "Runs after an LSP server start command is triggered.",
    )])
    .with_hook_bindings(vec![PluginHookBinding::new(
        "buffer.file-open",
        "lsp.auto-start-rust",
        "lsp.start-rust-analyzer",
        Some(".rs"),
    )])
}

/// Returns LSP server specifications compiled into the user library.
pub fn language_servers() -> Vec<LanguageServerSpec> {
    vec![
        LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "rust-analyzer",
            ["--stdio"],
        )
        .with_root_markers(["Cargo.toml", "rust-project.json"]),
    ]
}
