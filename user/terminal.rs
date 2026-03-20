use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage,
};

/// Returns the metadata for the builtin terminal package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "terminal",
        true,
        "Builtin terminal buffers and shell integration.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "terminal.open",
            "Opens the builtin terminal buffer.",
            vec![
                PluginAction::open_buffer("*terminal*", "terminal", None::<&str>),
                PluginAction::log_message("Terminal buffer opened from the user package."),
            ],
        ),
        PluginCommand::new(
            "terminal.popup",
            "Opens a popup-hosted terminal buffer.",
            vec![PluginAction::open_buffer(
                "*terminal-popup*",
                "terminal",
                Some("Terminal"),
            )],
        ),
    ])
    .with_key_bindings(vec![PluginKeyBinding::new(
        "Ctrl+`",
        "terminal.open",
        PluginKeymapScope::Global,
    )])
}
