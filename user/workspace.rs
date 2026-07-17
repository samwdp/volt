use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use editor_fs::{ProjectCandidate, ProjectKind, ProjectSearchRoot, discover_projects};
use editor_git::list_repository_files;
use editor_plugin_api::{
    PickerActionSpec, PickerItemSpec, PickerProviderContext, PickerSource, PickerWorkspaceContext,
    PluginAction, PluginCommand, PluginPackage,
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
            "Switches to an open workspace or opens a discovered project.",
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
        hook_command(
            "workspace.next",
            "Switches to the next open Project Workspace in open order.",
            "workspace.next",
        ),
        hook_command(
            "workspace.previous",
            "Switches to the previous open Project Workspace in open order.",
            "workspace.previous",
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

pub fn project_search_roots() -> Vec<ProjectSearchRoot> {
    crate::config::load().workspace.project_search_roots()
}

pub fn picker_items(context: &PickerProviderContext) -> Option<Vec<PickerItemSpec>> {
    match context.source {
        PickerSource::WorkspaceProjects => workspace_project_picker_items(context).ok(),
        PickerSource::WorkspaceSwitch => {
            Some(workspace_switch_picker_items(context).unwrap_or_else(|_| {
                context
                    .workspaces
                    .iter()
                    .map(workspace_picker_item)
                    .collect()
            }))
        }
        PickerSource::WorkspaceDelete => Some(workspace_delete_picker_items(context)),
        PickerSource::WorkspaceFiles => Some(workspace_file_picker_items(context)),
        _ => None,
    }
}

fn workspace_project_picker_items(
    context: &PickerProviderContext,
) -> Result<Vec<PickerItemSpec>, String> {
    let roots = project_search_roots();
    let mut projects = discover_projects(&roots).map_err(|error| error.to_string())?;
    projects.sort_by_key(|project| {
        (
            existing_workspace_for_project(context, project).is_none(),
            project.display_name().to_ascii_lowercase(),
        )
    });

    Ok(projects
        .iter()
        .map(|project| project_picker_item(context, project))
        .collect())
}

fn workspace_switch_picker_items(
    context: &PickerProviderContext,
) -> Result<Vec<PickerItemSpec>, String> {
    let roots = project_search_roots();
    let mut projects = discover_projects(&roots)
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|project| existing_workspace_for_project(context, project).is_none())
        .collect::<Vec<_>>();
    projects.sort_by_key(|project| project.display_name().to_ascii_lowercase());

    let mut items = context
        .workspaces
        .iter()
        .map(workspace_picker_item)
        .collect::<Vec<_>>();
    if !items.is_empty() && !projects.is_empty() {
        items.push(PickerItemSpec::divider());
    }
    items.extend(
        projects
            .iter()
            .map(|project| project_picker_item(context, project)),
    );
    Ok(items)
}

fn project_picker_item(
    context: &PickerProviderContext,
    project: &ProjectCandidate,
) -> PickerItemSpec {
    let existing_workspace = existing_workspace_for_project(context, project);
    let workspace_name = project.display_name();
    let detail = workspace_project_picker_detail(project, existing_workspace.is_some());
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
    .with_preview(workspace_project_picker_preview(project))
}

fn existing_workspace_for_project<'a>(
    context: &'a PickerProviderContext,
    project: &ProjectCandidate,
) -> Option<&'a PickerWorkspaceContext> {
    context.workspaces.iter().find(|workspace| {
        workspace
            .root
            .as_ref()
            .into_option()
            .is_some_and(|root| Path::new(root.as_str()) == project.root())
    })
}

fn workspace_picker_item(workspace: &PickerWorkspaceContext) -> PickerItemSpec {
    let location = workspace
        .root
        .as_ref()
        .into_option()
        .map(|root| root.to_string())
        .unwrap_or_else(|| "default workspace".to_owned());
    let detail = format!("open workspace | {location}");
    PickerItemSpec::new(
        workspace.id.to_string(),
        workspace.name.clone(),
        detail.clone(),
        PickerActionSpec::switch_workspace(workspace.id),
    )
    .with_preview(detail)
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
            PickerItemSpec::new(
                path.display().to_string(),
                search_text.clone(),
                "",
                PickerActionSpec::open_file(path.display().to_string()),
            )
            .with_preview(file_picker_preview(&path))
            .with_search_text(search_text)
            .with_fringe(editor_icons::seti_file_icon(&path))
        })
        .collect()
}

fn file_picker_preview(path: &Path) -> String {
    const MAX_BYTES: u64 = 16 * 1024;
    const MAX_LINES: usize = 24;

    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return path.display().to_string(),
    };
    let mut buffer = String::new();
    if file.take(MAX_BYTES).read_to_string(&mut buffer).is_err() {
        return path.display().to_string();
    }
    let mut lines = Vec::new();
    lines.push(path.display().to_string());
    lines.extend(
        buffer
            .lines()
            .take(MAX_LINES)
            .map(|line| line.trim_end().to_owned()),
    );
    lines.join("\n")
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

    #[test]
    fn package_exports_cycle_project_workspace_commands() {
        let package = package();
        let names: Vec<_> = package.commands().iter().map(|c| c.name()).collect();
        assert!(names.contains(&"workspace.next"));
        assert!(names.contains(&"workspace.previous"));
    }
}
