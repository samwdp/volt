/// [`InputPromptOverlay`] id for the `workspace.clone` remote URL prompt.
const WORKSPACE_CLONE_URL_PROMPT_ID: &str = "workspace.clone.url";

fn open_workspace_clone_prompt(runtime: &mut EditorRuntime) -> Result<(), String> {
    let overlay = InputPromptOverlay::new(WORKSPACE_CLONE_URL_PROMPT_ID, "Clone URL: ", "");
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn confirm_workspace_clone_url(runtime: &mut EditorRuntime, text: &str) -> Result<(), String> {
    let url = text.trim();
    if url.is_empty() {
        return Err("clone URL is required".to_owned());
    }
    open_workspace_clone_mode_picker(runtime, url)
}

fn open_workspace_clone_mode_picker(runtime: &mut EditorRuntime, url: &str) -> Result<(), String> {
    let entries = vec![
        PickerEntry {
            item: PickerItem::new(
                "workspace-clone:bare",
                "Bare repo",
                "git clone --bare, then Workspace Dashboard",
                Some(
                    "Clone as a Bare Repo, then open Workspace Dashboard to create a Worktree."
                        .to_owned(),
                ),
            ),
            action: PickerAction::WorkspaceCloneMode {
                url: url.to_owned(),
                bare: true,
            },
            quickfix: None,
        },
        PickerEntry {
            item: PickerItem::new(
                "workspace-clone:full",
                "Full clone",
                "git clone, then open as Project Workspace",
                Some("Clone a full working tree and open it as a Project Workspace.".to_owned()),
            ),
            action: PickerAction::WorkspaceCloneMode {
                url: url.to_owned(),
                bare: false,
            },
            quickfix: None,
        },
    ];
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries("Clone mode", entries));
    Ok(())
}

fn first_project_discovery_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    let roots = project_search_roots_from_user_library(&*shell_user_library(runtime));
    if let Some(root) = roots.into_iter().next() {
        return Ok(root.root().to_path_buf());
    }
    oil_default_root(runtime)
}

fn begin_oil_clone_request(
    runtime: &mut EditorRuntime,
    url: &str,
    bare: bool,
) -> Result<(), String> {
    let start_root = first_project_discovery_root(runtime)?;
    shell_ui_mut(runtime)?.pending_workspace_clone = Some(PendingWorkspaceClone {
        url: url.to_owned(),
        bare,
    });
    open_oil_directory(runtime, start_root)?;
    Ok(())
}

fn run_workspace_clone(
    runtime: &mut EditorRuntime,
    url: &str,
    bare: bool,
    clone_path: &Path,
) -> Result<(), String> {
    if clone_path.exists() {
        return Err(format!(
            "clone path already exists: {}",
            clone_path.display()
        ));
    }
    let parent = clone_path.parent().map(Path::to_path_buf).ok_or_else(|| {
        format!(
            "clone path `{}` has no parent directory",
            clone_path.display()
        )
    })?;
    let path_arg = clone_path.display().to_string();
    let args = if bare {
        vec![
            "clone".to_owned(),
            "--bare".to_owned(),
            url.to_owned(),
            path_arg,
        ]
    } else {
        vec!["clone".to_owned(), url.to_owned(), path_arg]
    };
    let name = clone_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("clone")
        .to_owned();
    let on_exit = if bare {
        StreamedCommandExitAction::CloseAndOpenWorkspaceDashboard {
            base_dir: clone_path.to_path_buf(),
        }
    } else {
        StreamedCommandExitAction::RefreshGitStatusCloseAndOpenWorkspace {
            name,
            path: clone_path.to_path_buf(),
        }
    };
    let title = if bare {
        "Clone Bare".to_owned()
    } else {
        "Clone".to_owned()
    };
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(title, args, parent, on_exit),
    )?;
    Ok(())
}

fn close_active_oil_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let Some(buffer_id) = find_oil_buffer(runtime, workspace_id) else {
        return Ok(());
    };
    shell_ui_mut(runtime)?.pending_workspace_clone = None;
    close_buffer_immediate(runtime, buffer_id)
}
