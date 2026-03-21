use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the buffer management package.
pub fn package() -> PluginPackage {
    PluginPackage::new("buffer", true, "Buffer save and management commands.").with_commands(vec![
        PluginCommand::new(
            "buffer.save",
            "Saves the active file buffer to disk.",
            vec![PluginAction::emit_hook("buffer.save", None::<&str>)],
        ),
    ])
}
