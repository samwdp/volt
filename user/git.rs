use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the git workflow package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "git",
        false,
        "Magit-style git workflows surfaced as buffers.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "git.status",
            "Opens the git status buffer.",
            vec![PluginAction::open_buffer(
                "*git-status*",
                "git",
                Some("Git Status"),
            )],
        ),
        PluginCommand::new(
            "git.branches",
            "Opens the git branches popup buffer.",
            vec![PluginAction::open_buffer(
                "*git-branches*",
                "git",
                Some("Git Branches"),
            )],
        ),
    ])
}
