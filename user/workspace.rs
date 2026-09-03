use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use editor_fs::{ProjectCandidate, ProjectKind, ProjectSearchRoot, project_discovery_for_picker};
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
        hook_command(
            "workspace.mark",
            "Adds the active Project Workspace root to the Mark List.",
            "workspace.mark",
        ),
        hook_command(
            "workspace.unmark",
            "Removes the active Project Workspace root from the Mark List.",
            "workspace.unmark",
        ),
        hook_command(
            "workspace.marks",
            "Opens the app-wide Mark List as an editable buffer.",
            "workspace.marks",
        ),
        hook_command(
            "workspace.marked-1",
            "Jumps to Mark List slot 1 (first Marked Workspace).",
            "workspace.marked-1",
        ),
        hook_command(
            "workspace.marked-2",
            "Jumps to Mark List slot 2 (second Marked Workspace).",
            "workspace.marked-2",
        ),
        hook_command(
            "workspace.marked-3",
            "Jumps to Mark List slot 3 (third Marked Workspace).",
            "workspace.marked-3",
        ),
        hook_command(
            "workspace.marked-4",
            "Jumps to Mark List slot 4 (fourth Marked Workspace).",
            "workspace.marked-4",
        ),
        hook_command(
            "workspace.worktree-remove",
            "Force-removes the selected Worktree from disk after closing matching Project Workspaces.",
            "workspace.worktree-remove",
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

const PROJECT_DISCOVERY_SCANNING_ID: &str = "project-discovery-scanning";

fn project_search_roots_override() -> &'static Mutex<Option<Vec<ProjectSearchRoot>>> {
    static OVERRIDE: OnceLock<Mutex<Option<Vec<ProjectSearchRoot>>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| Mutex::new(None))
}

/// Overrides configured discovery roots. Intended for tests.
pub fn override_project_search_roots_for_test(roots: Option<Vec<ProjectSearchRoot>>) {
    *project_search_roots_override()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = roots;
}

pub fn project_search_roots() -> Vec<ProjectSearchRoot> {
    if let Some(roots) = project_search_roots_override()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
    {
        return roots;
    }
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
    let snapshot = project_discovery_for_picker(&project_search_roots());
    if snapshot.candidates().is_empty() && snapshot.in_progress() {
        return Ok(vec![project_discovery_scanning_item()]);
    }

    let mut projects = snapshot.candidates().to_vec();
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
    let snapshot = project_discovery_for_picker(&project_search_roots());
    let scanning = snapshot.candidates().is_empty() && snapshot.in_progress();
    let mut projects = snapshot
        .candidates()
        .iter()
        .filter(|project| existing_workspace_for_project(context, project).is_none())
        .cloned()
        .collect::<Vec<_>>();
    projects.sort_by_key(|project| project.display_name().to_ascii_lowercase());

    let mut items = context
        .workspaces
        .iter()
        .map(workspace_picker_item)
        .collect::<Vec<_>>();
    if !items.is_empty() && (!projects.is_empty() || scanning) {
        items.push(PickerItemSpec::divider());
    }
    if scanning && projects.is_empty() {
        items.push(project_discovery_scanning_item());
    } else {
        items.extend(
            projects
                .iter()
                .map(|project| project_picker_item(context, project)),
        );
    }
    Ok(items)
}

fn project_discovery_scanning_item() -> PickerItemSpec {
    PickerItemSpec::new(
        PROJECT_DISCOVERY_SCANNING_ID,
        "Scanning for projects...",
        "project discovery in progress",
        PickerActionSpec::no_op(),
    )
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
    use abi_stable::std_types::ROption;
    use editor_fs::{
        ProjectSearchRoot, project_discovery_snapshot, reset_project_discovery_cache,
        set_project_discovery_persist_path_for_test, set_project_discovery_worker_blocked_for_test,
        wait_for_project_discovery,
    };
    use editor_plugin_api::{
        PickerActionSpec, PickerProviderContext, PickerSource, PickerWorkspaceContext,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        sync::{Mutex, MutexGuard},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    fn discovery_test_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct DiscoveryOverrideGuard {
        persist_dir: PathBuf,
    }

    impl Drop for DiscoveryOverrideGuard {
        fn drop(&mut self) {
            override_project_search_roots_for_test(None);
            reset_project_discovery_cache();
            set_project_discovery_persist_path_for_test(None);
            set_project_discovery_worker_blocked_for_test(false);
            let _ = fs::remove_dir_all(&self.persist_dir);
        }
    }

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-user-workspace-{label}-{unique}"))
    }

    fn wait_timeout() -> Duration {
        Duration::from_secs(2)
    }

    fn two_repo_tree() -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
        let root = temp_dir("picker-projects");
        let alpha = root.join("alpha");
        let zeta = root.join("zeta");
        fs::create_dir_all(alpha.join(".git"))?;
        fs::create_dir_all(zeta.join(".git"))?;
        Ok((root, alpha, zeta))
    }

    fn begin_discovery_override(
        roots: Vec<ProjectSearchRoot>,
    ) -> Result<DiscoveryOverrideGuard, Box<dyn std::error::Error>> {
        let persist_dir = temp_dir("discovery-persist");
        fs::create_dir_all(&persist_dir)?;
        set_project_discovery_persist_path_for_test(Some(persist_dir.join("projects.json")));
        reset_project_discovery_cache();
        override_project_search_roots_for_test(Some(roots));
        Ok(DiscoveryOverrideGuard { persist_dir })
    }

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

    #[test]
    fn package_exports_mark_list_commands() {
        let package = package();
        let names: Vec<_> = package
            .commands()
            .iter()
            .map(|command| command.name())
            .collect();

        assert!(names.contains(&"workspace.mark"));
        assert!(names.contains(&"workspace.unmark"));
        assert!(names.contains(&"workspace.marks"));
    }

    #[test]
    fn package_exports_marked_workspace_slot_jump_commands() {
        let package = package();
        let names: Vec<_> = package
            .commands()
            .iter()
            .map(|command| command.name())
            .collect();

        assert!(names.contains(&"workspace.marked-1"));
        assert!(names.contains(&"workspace.marked-2"));
        assert!(names.contains(&"workspace.marked-3"));
        assert!(names.contains(&"workspace.marked-4"));
    }

    #[test]
    fn package_exports_worktree_remove_command() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "workspace.worktree-remove")
        );
    }

    #[test]
    fn workspace_project_picker_items_sort_open_workspace_first()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let (root, alpha, zeta) = two_repo_tree()?;
        let _guard = begin_discovery_override(vec![ProjectSearchRoot::new(&root, 2)])?;
        let scanning = project_discovery_snapshot(&project_search_roots());
        wait_for_project_discovery(scanning.request_id(), wait_timeout())?;

        let mut context = PickerProviderContext::new(
            "workspace.projects",
            "Projects",
            PickerSource::WorkspaceProjects,
        );
        context.workspaces = vec![PickerWorkspaceContext {
            id: 7,
            name: "zeta".into(),
            root: ROption::RSome(zeta.display().to_string().into()),
            is_default: false,
        }]
        .into();

        let items = picker_items(&context).expect("project picker items");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label(), "zeta");
        assert_eq!(items[0].action(), &PickerActionSpec::switch_workspace(7));
        assert_eq!(items[1].label(), "alpha");
        assert!(matches!(
            items[1].action(),
            PickerActionSpec::CreateWorkspace { .. }
        ));
        assert_eq!(items[1].id(), alpha.display().to_string());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn workspace_switch_picker_items_list_open_workspaces_then_projects()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let (root, alpha, zeta) = two_repo_tree()?;
        let _guard = begin_discovery_override(vec![ProjectSearchRoot::new(&root, 2)])?;
        let scanning = project_discovery_snapshot(&project_search_roots());
        wait_for_project_discovery(scanning.request_id(), wait_timeout())?;

        let mut context = PickerProviderContext::new(
            "workspace.switch",
            "Workspaces and Projects",
            PickerSource::WorkspaceSwitch,
        );
        context.workspaces = vec![
            PickerWorkspaceContext {
                id: 1,
                name: "default".into(),
                root: ROption::RNone,
                is_default: true,
            },
            PickerWorkspaceContext {
                id: 7,
                name: "zeta".into(),
                root: ROption::RSome(zeta.display().to_string().into()),
                is_default: false,
            },
        ]
        .into();

        let items = picker_items(&context).expect("switch picker items");
        assert_eq!(items[0].label(), "default");
        assert_eq!(items[1].label(), "zeta");
        assert!(items[2].is_divider());
        assert_eq!(items[3].label(), "alpha");
        assert!(matches!(
            items[3].action(),
            PickerActionSpec::CreateWorkspace { .. }
        ));
        assert!(
            items
                .iter()
                .all(|item| item.id() != alpha.display().to_string()
                    || matches!(item.action(), PickerActionSpec::CreateWorkspace { .. }))
        );
        assert!(items.iter().all(|item| {
            item.label() != "zeta"
                || matches!(item.action(), PickerActionSpec::SwitchWorkspace { .. })
        }));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn workspace_project_picker_items_show_scanning_row_when_cache_empty()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let (root, _, _) = two_repo_tree()?;
        let _guard = begin_discovery_override(vec![ProjectSearchRoot::new(&root, 2)])?;
        set_project_discovery_worker_blocked_for_test(true);

        let context = PickerProviderContext::new(
            "workspace.projects",
            "Projects",
            PickerSource::WorkspaceProjects,
        );
        let items = picker_items(&context).expect("scanning items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id(), PROJECT_DISCOVERY_SCANNING_ID);
        assert_eq!(items[0].action(), &PickerActionSpec::no_op());

        set_project_discovery_worker_blocked_for_test(false);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn workspace_project_picker_items_keep_candidates_while_rescan_runs()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let (root, _, _) = two_repo_tree()?;
        let _guard = begin_discovery_override(vec![ProjectSearchRoot::new(&root, 2)])?;
        let scanning = project_discovery_snapshot(&project_search_roots());
        wait_for_project_discovery(scanning.request_id(), wait_timeout())?;
        editor_fs::set_project_discovery_ttl_for_test(Duration::ZERO);
        set_project_discovery_worker_blocked_for_test(true);

        let context = PickerProviderContext::new(
            "workspace.projects",
            "Projects",
            PickerSource::WorkspaceProjects,
        );
        let items = picker_items(&context).expect("stale candidates");
        assert!(
            items
                .iter()
                .all(|item| item.id() != PROJECT_DISCOVERY_SCANNING_ID)
        );
        assert_eq!(items.len(), 2);

        set_project_discovery_worker_blocked_for_test(false);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    fn git_available() -> bool {
        Command::new("git").arg("--version").output().is_ok()
    }

    fn run_git(root: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new("git").args(args).current_dir(root).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("git {:?} failed with status {status}", args).into())
        }
    }

    #[test]
    fn workspace_file_picker_items_report_no_project_root() {
        let context = PickerProviderContext::new(
            "workspace.files",
            "Workspace Files",
            PickerSource::WorkspaceFiles,
        );
        let items = picker_items(&context).expect("file picker items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label(), "Workspace has no project root");
        assert!(items[0].preview().is_some());
    }

    #[test]
    fn workspace_file_picker_items_list_paths_without_previews()
    -> Result<(), Box<dyn std::error::Error>> {
        if !git_available() {
            return Ok(());
        }

        let root = temp_dir("files-lazy-preview");
        let nested = root.join("src").join("deep");
        fs::create_dir_all(&nested)?;
        fs::write(root.join(".gitignore"), "ignored.txt\n")?;
        fs::write(nested.join("nested.rs"), "fn nested() {}\n")?;
        fs::write(root.join("ignored.txt"), "ignored\n")?;
        fs::write(root.join("notes.txt"), "notes\n")?;
        run_git(&root, &["init", "-q"])?;
        run_git(&root, &["add", ".gitignore", "src/deep/nested.rs"])?;

        let mut context = PickerProviderContext::new(
            "workspace.files",
            "Workspace Files",
            PickerSource::WorkspaceFiles,
        );
        context.workspace_root = ROption::RSome(root.display().to_string().into());
        let items = picker_items(&context).expect("file picker items");

        assert!(
            items.iter().all(|item| item.preview().is_none()),
            "Workspace Files rows must not read file bodies in the user library"
        );
        assert!(items.iter().any(|item| {
            item.label() == "src/deep/nested.rs"
                && item.search_text() == Some("src/deep/nested.rs")
                && item.detail().is_empty()
                && item.fringe() == Some(editor_icons::seti_file_icon(&nested.join("nested.rs")))
        }));
        assert!(items.iter().any(|item| item.label() == "notes.txt"));
        assert!(items.iter().all(|item| item.label() != "ignored.txt"));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
