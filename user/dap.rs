use editor_dap::DebugAdapterSpec;
use editor_plugin_api::{PluginAction, PluginCommand, PluginHookDeclaration, PluginPackage};

/// Returns the metadata for the DAP integration package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "dap",
        true,
        "Debug adapter integration and Rust debugging session defaults.",
    )
    .with_commands(vec![PluginCommand::new(
        "dap.start-codelldb",
        "Prepares a Rust debug session through the compiled user package.",
        vec![
            PluginAction::open_buffer("*dap-sessions*", "dap", Some("Debug Sessions")),
            PluginAction::emit_hook("dap.session-start", Some("codelldb")),
        ],
    )])
    .with_hook_declarations(vec![PluginHookDeclaration::new(
        "dap.session-start",
        "Runs after a debug adapter session plan is prepared.",
    )])
}

/// Returns DAP adapter specifications compiled into the user library.
pub fn debug_adapters() -> Vec<DebugAdapterSpec> {
    vec![DebugAdapterSpec::new(
        "codelldb",
        "rust",
        ["rs"],
        "codelldb",
        ["--port", "13000"],
    )]
}
