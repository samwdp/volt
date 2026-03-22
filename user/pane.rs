use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the pane management package.
pub fn package() -> PluginPackage {
    PluginPackage::new("pane", true, "Pane layout and split commands.").with_commands(vec![
        hook_command(
            "pane.split-horizontal",
            "Splits the active workspace horizontally.",
            "ui.pane.split-horizontal",
        ),
        hook_command(
            "pane.split-vertical",
            "Splits the active workspace vertically.",
            "ui.pane.split-vertical",
        ),
    ])
}

fn hook_command(name: &str, description: &str, hook_name: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, None::<&str>)],
    )
}
