use editor_fs::ProjectSearchRoot;
use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the workspace management package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "workspace",
        true,
        "Workspace creation, switching, deletion, file listing, and project discovery.",
    )
    .with_commands(vec![
        picker_command(
            "workspace.new",
            "Creates or focuses a workspace from the project picker.",
            "workspace.projects",
        ),
        picker_command(
            "workspace.switch",
            "Switches to one of the open workspaces.",
            "workspace.switch",
        ),
        picker_command(
            "workspace.delete",
            "Deletes one of the open workspaces.",
            "workspace.delete",
        ),
        picker_command(
            "workspace.list-files",
            "Lists the current workspace files that are visible to Git.",
            "workspace.files",
        ),
        PluginCommand::new(
            "workspace.save",
            "Saves all modified file buffers in the active workspace.",
            vec![PluginAction::emit_hook("workspace.save", None::<&str>)],
        ),
        hook_command(
            "workspace.window-left",
            "Moves focus to the window on the left (wraps).",
            "ui.workspace.window-left",
        ),
        hook_command(
            "workspace.window-down",
            "Moves focus to the window below (wraps).",
            "ui.workspace.window-down",
        ),
        hook_command(
            "workspace.window-up",
            "Moves focus to the window above (wraps).",
            "ui.workspace.window-up",
        ),
        hook_command(
            "workspace.window-right",
            "Moves focus to the window on the right (wraps).",
            "ui.workspace.window-right",
        ),
    ])
}

/// Returns the configured project discovery roots.
///
/// Users can edit this list to control which directories are scanned and how
/// deep the project search should traverse from each root.
pub fn project_search_roots() -> Vec<ProjectSearchRoot> {
    vec![
        ProjectSearchRoot::new(r"P:\", 4),
        ProjectSearchRoot::new(r"W:\", 4),
    ]
    .into_iter()
    .filter(|search_root| search_root.root().exists())
    .collect()
}

fn picker_command(name: &str, description: &str, provider: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook("ui.picker.open", Some(provider))],
    )
}

fn hook_command(name: &str, description: &str, hook_name: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, None::<&str>)],
    )
}
