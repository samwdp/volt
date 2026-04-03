use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the buffer management package.
pub fn package() -> PluginPackage {
    PluginPackage::new("buffer", true, "Buffer save and management commands.").with_commands(vec![
        PluginCommand::new(
            "buffer.save",
            "Saves the active file-backed buffer to disk.",
            vec![PluginAction::emit_hook("buffer.save", None::<&str>)],
        ),
        PluginCommand::new(
            "buffer.close",
            "Closes the active buffer.",
            vec![PluginAction::emit_hook("buffer.close", None::<&str>)],
        ),
        PluginCommand::new(
            "buffer.close-picker",
            "Opens the buffer close picker.",
            vec![PluginAction::emit_hook(
                "ui.picker.open",
                Some("buffers.close"),
            )],
        ),
    ])
}
