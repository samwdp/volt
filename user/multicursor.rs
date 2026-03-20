use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the multiple cursor package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "multicursor",
        false,
        "Multiple cursor editing commands and selections.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "multicursor.add-next-match",
            "Adds a new cursor at the next match.",
            vec![PluginAction::log_message(
                "Multiple cursor expansion requested by the user package.",
            )],
        ),
        PluginCommand::new(
            "multicursor.select-all-matches",
            "Adds cursors at every remaining match in the buffer.",
            vec![PluginAction::log_message(
                "Multiple cursor select-all requested by the user package.",
            )],
        ),
    ])
}
