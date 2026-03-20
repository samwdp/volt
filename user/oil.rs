use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the directory editing package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "oil",
        false,
        "Directory manipulation buffers inspired by oil.nvim.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "oil.open-directory",
            "Opens an editable directory buffer.",
            vec![PluginAction::open_buffer(
                "*oil*",
                "directory",
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "oil.open-parent",
            "Opens a parent-directory view in a popup.",
            vec![PluginAction::open_buffer(
                "*oil-parent*",
                "directory",
                Some("Parent Directory"),
            )],
        ),
    ])
}
