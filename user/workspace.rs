use std::path::{Path, PathBuf};

use editor_fs::{ProjectCandidate, ProjectKind, ProjectSearchRoot, discover_projects};
use editor_git::list_repository_files;
use editor_plugin_api::{
    PickerActionSpec, PickerItemSpec, PickerProviderContext, PickerSource, PluginAction,
    PluginCommand, PluginPackage,
};

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
            "workspace.dashboard",
            "Shows checked-out git worktrees for the active workspace.",
            "workspace.dashboard",
        ),
        picker_command(
            "workspace.list-files",
            "Lists the current workspace files that are visible to Git.",
            "workspace.files",
        ),
        picker_command(
            "workspace.search",
            "Searches text across files in the active workspace.",
            "workspace.search",
        ),
        PluginCommand::new(
            "workspace.save",
            "Saves all modified file buffers in the active workspace.",
            vec![PluginAction::emit_hook("workspace.save", None::<&str>)],
        ),
        PluginCommand::new(
            "workspace.format",
            "Formats the active file buffer, preferring LSP formatting when available.",
            vec![PluginAction::emit_hook("workspace.format", None::<&str>)],
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
        ProjectSearchRoot::new(r"C:\Users\sam\", 4),
    ]
    .into_iter()
    .filter(|search_root| search_root.root().exists())
    .collect()
}

pub fn picker_items(context: &PickerProviderContext) -> Option<Vec<PickerItemSpec>> {
    match context.source {
        PickerSource::WorkspaceProjects => workspace_project_picker_items(context).ok(),
        PickerSource::WorkspaceSwitch => Some(workspace_switch_picker_items(context)),
        PickerSource::WorkspaceDelete => Some(workspace_delete_picker_items(context)),
        PickerSource::WorkspaceFiles => Some(workspace_file_picker_items(context)),
        _ => None,
    }
}

fn workspace_project_picker_items(
    context: &PickerProviderContext,
) -> Result<Vec<PickerItemSpec>, String> {
    let roots = project_search_roots();
    let projects = discover_projects(&roots).map_err(|error| error.to_string())?;
    Ok(projects
        .into_iter()
        .map(|project| {
            let existing_workspace = context.workspaces.iter().find(|workspace| {
                workspace
                    .root
                    .as_ref()
                    .into_option()
                    .is_some_and(|root| Path::new(root.as_str()) == project.root())
            });
            let workspace_name = project.display_name();
            let detail = workspace_project_picker_detail(&project, existing_workspace.is_some());
            let action = existing_workspace.map_or_else(
                || {
                    PickerActionSpec::create_workspace(
                        workspace_name.clone(),
                        project.root().display().to_string(),
                    )
                },
                |workspace| PickerActionSpec::switch_workspace(workspace.id),
            );
            PickerItemSpec::new(
                project.root().display().to_string(),
                workspace_name,
                detail,
                action,
            )
            .with_preview(workspace_project_picker_preview(&project))
        })
        .collect())
}

fn workspace_project_picker_detail(project: &ProjectCandidate, is_open: bool) -> String {
    let mut parts = vec![project.kind().label().to_owned()];
    if project.kind() == ProjectKind::GitWorktree && project.repository_root() != project.root() {
        let context = project
            .worktree_parent_name()
            .unwrap_or_else(|| project.repository_display_name());
        parts.push(format!("project {context}"));
    }
    if is_open {
        parts.push("open workspace".to_owned());
    }
    parts.join(" | ")
}

fn workspace_project_picker_preview(project: &ProjectCandidate) -> String {
    if project.kind() == ProjectKind::GitWorktree && project.repository_root() != project.root() {
        return format!(
            "worktree {} | repo {}",
            project.root().display(),
            project.repository_root().display(),
        );
    }
    project.root().display().to_string()
}

fn workspace_switch_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .workspaces
        .iter()
        .map(|workspace| {
            let detail = workspace
                .root
                .as_ref()
                .into_option()
                .map(|root| root.to_string())
                .unwrap_or_else(|| "default workspace".to_owned());
            PickerItemSpec::new(
                workspace.id.to_string(),
                workspace.name.clone(),
                detail.clone(),
                PickerActionSpec::switch_workspace(workspace.id),
            )
            .with_preview(detail)
        })
        .collect()
}

fn workspace_delete_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .workspaces
        .iter()
        .filter(|workspace| !workspace.is_default)
        .map(|workspace| {
            let detail = workspace
                .root
                .as_ref()
                .into_option()
                .map(|root| root.to_string())
                .unwrap_or_else(|| "workspace".to_owned());
            PickerItemSpec::new(
                workspace.id.to_string(),
                workspace.name.clone(),
                detail,
                PickerActionSpec::delete_workspace(workspace.id),
            )
            .with_preview("Deletes the selected workspace.")
        })
        .collect()
}

fn workspace_file_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    let Some(root) = context.workspace_root.as_ref().into_option() else {
        return vec![message_item(
            "Workspace has no project root",
            "Open a project-backed workspace before listing files.",
            "workspace.list-files works from a project workspace created by workspace.new.",
        )];
    };
    let root = PathBuf::from(root.as_str());
    let files = match list_repository_files(&root) {
        Ok(files) => files,
        Err(error) => {
            return vec![message_item(
                "Unable to read workspace files",
                error.to_string(),
                root.display().to_string(),
            )];
        }
    };
    if files.is_empty() {
        return vec![message_item(
            "No visible files found",
            "Git did not report any tracked or unignored files for this workspace.",
            root.display().to_string(),
        )];
    }
    files
        .into_iter()
        .map(|relative_path| {
            let path = root.join(&relative_path);
            let search_text = relative_path.display().to_string();
            let label = relative_path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| search_text.clone());
            let detail = relative_path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(|parent| parent.display().to_string())
                .unwrap_or_else(|| "workspace root".to_owned());
            PickerItemSpec::new(
                path.display().to_string(),
                label,
                detail,
                PickerActionSpec::open_file(path.display().to_string()),
            )
            .with_preview(path.display().to_string())
            .with_search_text(search_text)
            .with_fringe(editor_icons::seti_file_icon(&path))
        })
        .collect()
}

fn message_item(
    label: impl Into<String>,
    detail: impl Into<String>,
    preview: impl Into<String>,
) -> PickerItemSpec {
    let label = label.into();
    PickerItemSpec::new(
        label.clone(),
        label,
        detail.into(),
        PickerActionSpec::no_op(),
    )
    .with_preview(preview.into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_format_command() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "workspace.format")
        );
    }
}
