use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the undo tree picker package.
pub fn package() -> PluginPackage {
    PluginPackage::new("undotree", true, "Undo tree history navigation.").with_commands(vec![
        PluginCommand::new(
            "undo-tree.open",
            "Opens the undo tree picker.",
            vec![PluginAction::emit_hook("ui.picker.open", Some("undo-tree"))],
        ),
    ])
}
