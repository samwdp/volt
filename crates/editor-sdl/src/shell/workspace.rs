fn active_window_id(runtime: &EditorRuntime) -> Result<editor_core::WindowId, String> {
    runtime
        .model()
        .active_window_id()
        .ok_or_else(|| "active window is missing".to_owned())
}

fn active_workspace_root(runtime: &EditorRuntime) -> Result<Option<PathBuf>, String> {
    Ok(runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?
        .root()
        .map(Path::to_path_buf))
}

fn workspace_root_for_path(
    runtime: &EditorRuntime,
    path: &Path,
) -> Result<Option<PathBuf>, String> {
    let window = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?;
    Ok(window
        .workspaces()
        .filter_map(|workspace| workspace.root())
        .filter(|root| path.starts_with(root))
        .max_by_key(|root| root.components().count())
        .map(Path::to_path_buf))
}

fn workspace_root_readme_path(root: &Path) -> Result<Option<PathBuf>, String> {
    let mut candidates = fs::read_dir(root)
        .map_err(|error| {
            format!(
                "failed to read workspace root `{}`: {error}",
                root.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_file() {
                return None;
            }

            let path = entry.path();
            let stem = path.file_stem()?.to_str()?;
            stem.eq_ignore_ascii_case("readme").then_some(path)
        })
        .collect::<Vec<_>>();
    candidates.sort_by_cached_key(|path| readme_path_priority(path));
    Ok(candidates.into_iter().next())
}

fn readme_path_priority(path: &Path) -> (u8, String) {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("<invalid-utf8-readme-name>")
        .to_ascii_lowercase();
    let priority = match file_name.as_str() {
        "readme.md" => 0,
        "readme" => 1,
        _ => 2,
    };
    (priority, file_name)
}

fn git_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_directory_root(runtime)? {
        return resolve_git_root_from_path(&root).or(Ok(root));
    }
    if let Some(root) = active_workspace_root(runtime)? {
        return resolve_git_root_from_path(&root).or(Ok(root));
    }
    env::current_dir().map_err(|error| format!("git status requires a workspace root: {error}"))
}

fn normalize_git_output_path(raw_path: &str) -> PathBuf {
    let trimmed = raw_path.trim();
    #[cfg(windows)]
    if let Some(converted) = normalize_git_output_path_windows(trimmed) {
        return converted;
    }
    PathBuf::from(trimmed)
}

#[cfg(windows)]
fn normalize_git_output_path_windows(trimmed: &str) -> Option<PathBuf> {
    let trimmed = trimmed.replace('\\', "/");
    let trimmed = trimmed.strip_prefix('/').unwrap_or(trimmed.as_str());
    let (drive, rest) = if let Some((drive, rest)) = trimmed.split_once(":/") {
        (drive, rest)
    } else {
        trimmed.split_once('/').unwrap_or((trimmed.trim(), ""))
    };
    if drive.len() != 1 || !drive.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    let drive = drive.chars().next()?.to_ascii_uppercase();
    let rest = rest.replace('/', "\\");
    Some(if rest.is_empty() {
        PathBuf::from(format!("{drive}:\\"))
    } else {
        PathBuf::from(format!("{drive}:\\{rest}"))
    })
}

fn resolve_git_root_from_path(path: &Path) -> Result<PathBuf, String> {
    let mut command = Command::new("git");
    configure_background_command(&mut command);
    let output = command
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .map_err(|error| {
            format!(
                "failed to resolve git root from {}: {error}",
                path.display()
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if stderr.is_empty() {
            format!(
                "git rev-parse --show-toplevel failed in {} with status {}",
                path.display(),
                output.status
            )
        } else {
            stderr
        });
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let root = stdout.trim();
    if root.is_empty() {
        return Err(format!(
            "git rev-parse --show-toplevel returned no root for {}",
            path.display()
        ));
    }
    Ok(normalize_git_output_path(root))
}

fn find_workspace_by_root(
    runtime: &EditorRuntime,
    root: &std::path::Path,
) -> Result<Option<WorkspaceId>, String> {
    let identity = canonicalize_project_root_path(root);
    let window = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?;
    Ok(window.workspaces().find_map(|workspace| {
        workspace.root().and_then(|workspace_root| {
            project_roots_equal(&canonicalize_project_root_path(workspace_root), &identity)
                .then_some(workspace.id())
        })
    }))
}

fn find_workspace_file_buffer(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
    path: &Path,
) -> Result<Option<BufferId>, String> {
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    Ok(workspace
        .buffers()
        .find(|buffer| {
            buffer
                .path()
                .is_some_and(|existing| editor_dap::debug_source_paths_eq(existing, path))
        })
        .map(Buffer::id))
}

fn find_workspace_named_buffer(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
    name: &str,
    kind: &BufferKind,
) -> Result<Option<BufferId>, String> {
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    Ok(workspace
        .buffers()
        .find(|buffer| buffer.name() == name && buffer.kind() == kind)
        .map(Buffer::id))
}

pub(crate) fn switch_runtime_workspace(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    prepare_workspace_leave_for_debug_layout(runtime)?;
    runtime
        .model_mut()
        .switch_workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.switch_workspace(workspace_id);
    prepare_workspace_enter_for_debug_layout(runtime)?;
    let window_id = active_window_id(runtime)?;
    let workspace_name = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .name()
        .to_owned();
    runtime
        .emit_hook(
            builtins::WORKSPACE_SWITCH,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_detail(workspace_name),
        )
        .map_err(|error| error.to_string())?;
    sync_active_buffer(runtime)
}

fn toggle_workspace_dock(runtime: &mut EditorRuntime) -> Result<(), String> {
    let user_library = shell_user_library(runtime);
    if user_library.workspace_dock_config().docked {
        shell_ui_mut(runtime)?.set_workspace_dock_open(true);
        return Ok(());
    }
    shell_ui_mut(runtime)?.toggle_workspace_dock_open();
    let visible = workspace_dock_visible(&*user_library, shell_ui(runtime)?);
    if !visible {
        shell_ui_mut(runtime)?.set_workspace_dock_focus(false);
    }
    Ok(())
}

fn toggle_acp_dock(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.toggle_acp_dock_open();
    Ok(())
}

fn workspace_dock_enter_direction(side: WorkspaceDockSide) -> WindowMoveDirection {
    match side {
        WorkspaceDockSide::Left => WindowMoveDirection::Left,
        WorkspaceDockSide::Right => WindowMoveDirection::Right,
    }
}

fn workspace_dock_exit_direction(side: WorkspaceDockSide) -> WindowMoveDirection {
    match side {
        WorkspaceDockSide::Left => WindowMoveDirection::Right,
        WorkspaceDockSide::Right => WindowMoveDirection::Left,
    }
}

fn cycle_workspace_dock(runtime: &mut EditorRuntime, forward: bool) -> Result<(), String> {
    let entries = collect_workspace_dock_entries(runtime)?;
    if entries.len() <= 1 {
        return Ok(());
    }
    let active = shell_ui(runtime)?.active_workspace();
    let current_index = entries
        .iter()
        .position(|entry| entry.workspace_id == active)
        .unwrap_or(0);
    let next_index = if forward {
        (current_index + 1) % entries.len()
    } else if current_index == 0 {
        entries.len() - 1
    } else {
        current_index - 1
    };
    switch_runtime_workspace(runtime, entries[next_index].workspace_id)
}

fn cycle_acp_dock(runtime: &mut EditorRuntime, forward: bool) -> Result<(), String> {
    let entries = collect_acp_dock_entries(runtime)?;
    if entries.is_empty() {
        return Ok(());
    }
    if entries.len() == 1 {
        acp::focus_acp_buffer(runtime, entries[0].buffer_id)?;
        shell_ui_mut(runtime)?.set_acp_dock_focus(true);
        return Ok(());
    }
    let active = shell_ui(runtime)?.active_buffer_id();
    let current_index = entries
        .iter()
        .position(|entry| Some(entry.buffer_id) == active)
        .unwrap_or(0);
    let next_index = if forward {
        (current_index + 1) % entries.len()
    } else if current_index == 0 {
        entries.len() - 1
    } else {
        current_index - 1
    };
    acp::focus_acp_buffer(runtime, entries[next_index].buffer_id)?;
    shell_ui_mut(runtime)?.set_acp_dock_focus(true);
    Ok(())
}

fn collect_workspace_dock_entries(
    runtime: &EditorRuntime,
) -> Result<Vec<WorkspaceDockEntry>, String> {
    let ui = shell_ui(runtime)?;
    let active = ui.active_workspace();
    let default_workspace = ui.default_workspace();
    let branch_cache = ui.workspace_dock_branches().clone();
    let window = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?;
    let mut entries = Vec::new();
    for workspace in window.workspaces() {
        if workspace.id() == default_workspace {
            entries.push(WorkspaceDockEntry {
                workspace_id: workspace.id(),
                name: workspace.name().to_owned(),
                buffer_count: workspace.buffer_count(),
                branch: None,
                active: workspace.id() == active,
                unread: ui.workspace_unread_count(workspace.id()),
            });
            break;
        }
    }
    for workspace in window.workspaces() {
        if workspace.id() == default_workspace {
            continue;
        }
        let branch = workspace
            .root()
            .and_then(|root| branch_cache.branch_for_root(root));
        entries.push(WorkspaceDockEntry {
            workspace_id: workspace.id(),
            name: workspace.name().to_owned(),
            buffer_count: workspace.buffer_count(),
            branch,
            active: workspace.id() == active,
            unread: ui.workspace_unread_count(workspace.id()),
        });
    }
    Ok(entries)
}

fn collect_acp_dock_entries(runtime: &EditorRuntime) -> Result<Vec<AcpDockEntry>, String> {
    let ui = shell_ui(runtime)?;
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let active_buffer = ui.active_buffer_id();
    let manager = runtime
        .services()
        .get::<Arc<Mutex<acp::AcpManager>>>()
        .cloned();
    let manager = match manager.as_ref() {
        Some(manager) => Some(
            manager
                .lock()
                .map_err(|_| "acp manager lock was poisoned".to_owned())?,
        ),
        None => None,
    };
    let mut entries = Vec::new();
    for buffer in workspace.buffers() {
        let BufferKind::Plugin(plugin_kind) = buffer.kind() else {
            continue;
        };
        if plugin_kind != ACP_BUFFER_KIND {
            continue;
        }
        let shell = ui.buffer(buffer.id());
        let session = shell
            .and_then(|buffer| buffer.acp_state.as_ref())
            .and_then(|state| state.session_title.clone())
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| "New session".to_owned());
        let client = manager
            .as_ref()
            .and_then(|manager| manager.client_id_for_buffer(buffer.id()))
            .and_then(|client_id| {
                shell_user_library(runtime)
                    .acp_client_by_id(&client_id)
                    .map(|client| client.label)
                    .or(Some(client_id))
            })
            .unwrap_or_else(|| "ACP".to_owned());
        let name = acp_dock_buffer_label(buffer.name());
        entries.push(AcpDockEntry {
            buffer_id: buffer.id(),
            name,
            session,
            client,
            active: active_buffer == Some(buffer.id()),
        });
    }
    Ok(entries)
}

fn acp_dock_buffer_label(name: &str) -> String {
    let trimmed = name
        .strip_prefix("*acp ")
        .and_then(|rest| rest.strip_suffix('*'))
        .unwrap_or(name);
    if trimmed.is_empty() {
        "ACP".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn workspace_dock_project_roots(runtime: &EditorRuntime) -> Result<Vec<PathBuf>, String> {
    let window = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?;
    Ok(window
        .workspaces()
        .filter_map(|workspace| workspace.root().map(Path::to_path_buf))
        .collect())
}

fn open_project_workspace_ids(runtime: &EditorRuntime) -> Result<Vec<WorkspaceId>, String> {
    let default_workspace = shell_ui(runtime)?.default_workspace();
    let window = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?;
    Ok(window
        .workspaces()
        .filter(|workspace| workspace.id() != default_workspace && workspace.root().is_some())
        .map(|workspace| workspace.id())
        .collect())
}

fn open_project_workspaces_with_roots(
    runtime: &EditorRuntime,
) -> Result<Vec<(WorkspaceId, PathBuf)>, String> {
    let default_workspace = shell_ui(runtime)?.default_workspace();
    let window = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?;
    Ok(window
        .workspaces()
        .filter(|workspace| workspace.id() != default_workspace)
        .filter_map(|workspace| {
            workspace
                .root()
                .map(|root| (workspace.id(), root.to_path_buf()))
        })
        .collect())
}

fn cycle_runtime_project_workspace(
    runtime: &mut EditorRuntime,
    direction: CycleDirection,
) -> Result<(), String> {
    let project_workspaces = open_project_workspace_ids(runtime)?;
    let active = shell_ui(runtime)?.active_workspace();
    let Some(target) = cycle_project_workspace(&project_workspaces, &active, direction) else {
        return Ok(());
    };
    switch_runtime_workspace(runtime, target)
}

fn mark_list_state(runtime: &EditorRuntime) -> Result<&MarkListState, String> {
    runtime
        .services()
        .get::<MarkListState>()
        .ok_or_else(|| "Mark List state service missing".to_owned())
}

fn mark_list_state_mut(runtime: &mut EditorRuntime) -> Result<&mut MarkListState, String> {
    runtime
        .services_mut()
        .get_mut::<MarkListState>()
        .ok_or_else(|| "Mark List state service missing".to_owned())
}

/// Canonical absolute project root for Mark List identity (strips Windows `\\?\`).
fn canonicalize_project_root_path(path: &Path) -> PathBuf {
    match fs::canonicalize(path) {
        Ok(canonical) => normalize_project_root_path(&canonical),
        Err(_) => normalize_project_root_path(path),
    }
}

/// Parses Mark List text, canonicalizing existing roots and keeping missing paths as written.
fn mark_list_from_persisted_text(text: &str) -> MarkList {
    let mut roots: Vec<PathBuf> = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let path = PathBuf::from(line);
        let identity = if path.exists() {
            canonicalize_project_root_path(&path)
        } else {
            path
        };
        if !roots
            .iter()
            .any(|root| project_roots_equal(root, &identity))
        {
            roots.push(identity);
        }
    }
    MarkList::from_roots(roots)
}

#[cfg(test)]
fn install_mark_list_state_for_test(
    runtime: &mut EditorRuntime,
    path: PathBuf,
) -> Result<(), String> {
    runtime.services_mut().insert(MarkListState::load(path)?);
    Ok(())
}

fn active_project_workspace_root(runtime: &EditorRuntime) -> Result<Option<PathBuf>, String> {
    let ui = shell_ui(runtime)?;
    let workspace_id = ui.active_workspace();
    if workspace_id == ui.default_workspace() {
        return Ok(None);
    }
    runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())
        .map(|workspace| workspace.root().map(Path::to_path_buf))
}

fn persist_mark_list(state: &MarkListState) -> Result<(), String> {
    let parent = state.path.parent().ok_or_else(|| {
        format!(
            "Mark List path `{}` does not have a parent directory",
            state.path.display()
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create Mark List state directory `{}`: {error}",
            parent.display()
        )
    })?;
    fs::write(&state.path, state.list.serialize()).map_err(|error| {
        format!(
            "failed to write Mark List `{}`: {error}",
            state.path.display()
        )
    })
}

fn persist_mark_list_and_refresh_open_buffers(runtime: &mut EditorRuntime) -> Result<(), String> {
    let path = {
        let state = mark_list_state(runtime)?;
        persist_mark_list(state)?;
        state.path.clone()
    };
    let buffer_ids = shell_ui(runtime)?
        .buffers
        .iter()
        .filter(|buffer| buffer.path() == Some(path.as_path()) && !buffer.is_dirty())
        .map(ShellBuffer::id)
        .collect::<Vec<_>>();
    for buffer_id in buffer_ids {
        let text = TextBuffer::load_from_path(&path)
            .map_err(|error| format!("failed to reload Mark List `{}`: {error}", path.display()))?;
        let fingerprint = BackingFileFingerprint::read(&path)
            .map_err(|error| format!("failed to stat Mark List `{}`: {error}", path.display()))?;
        shell_buffer_mut(runtime, buffer_id)?.apply_reloaded_file_buffer(fingerprint, text);
    }
    Ok(())
}

fn notify_default_workspace_has_no_project_root(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.apply_notification(
        NotificationUpdate {
            key: "workspace.mark.no-project-root".to_owned(),
            severity: NotificationSeverity::Warning,
            title: "Default Workspace has no project root".to_owned(),
            body_lines: vec![
                "Switch to a Project Workspace before changing the Mark List.".to_owned(),
            ],
            progress: None,
            active: false,
            action: None,
            workspace_id: None,
        },
        Instant::now(),
    );
    Ok(())
}

fn mark_active_project_workspace(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(root) = active_project_workspace_root(runtime)? else {
        return notify_default_workspace_has_no_project_root(runtime);
    };
    let root = canonicalize_project_root_path(&root);
    if mark_list_state_mut(runtime)?.list.mark(&root) {
        persist_mark_list_and_refresh_open_buffers(runtime)?;
    }
    Ok(())
}

fn unmark_active_project_workspace(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(root) = active_project_workspace_root(runtime)? else {
        return notify_default_workspace_has_no_project_root(runtime);
    };
    let root = canonicalize_project_root_path(&root);
    if mark_list_state_mut(runtime)?.list.unmark(&root) {
        persist_mark_list_and_refresh_open_buffers(runtime)?;
    }
    Ok(())
}

fn open_mark_list(runtime: &mut EditorRuntime) -> Result<(), String> {
    let path = mark_list_state(runtime)?.path.clone();
    if !path.exists() {
        persist_mark_list(mark_list_state(runtime)?)?;
    }
    open_workspace_file(runtime, &path)?;
    Ok(())
}

fn open_project_workspace_roots(runtime: &EditorRuntime) -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();
    for workspace_id in open_project_workspace_ids(runtime)? {
        let Some(root) = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?
            .root()
            .map(Path::to_path_buf)
        else {
            continue;
        };
        roots.push(canonicalize_project_root_path(&root));
    }
    Ok(roots)
}

fn marked_workspace_display_name(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| root.display().to_string())
}

fn notify_marked_workspace_missing(runtime: &mut EditorRuntime, root: &Path) -> Result<(), String> {
    shell_ui_mut(runtime)?.apply_notification(
        NotificationUpdate {
            key: "workspace.marked.missing-path".to_owned(),
            severity: NotificationSeverity::Warning,
            title: "Marked Workspace path missing".to_owned(),
            body_lines: vec![format!(
                "Mark List entry `{}` does not exist on disk.",
                root.display()
            )],
            progress: None,
            active: false,
            action: None,
            workspace_id: None,
        },
        Instant::now(),
    );
    Ok(())
}

fn jump_to_marked_workspace_slot(
    runtime: &mut EditorRuntime,
    slot_index: usize,
) -> Result<(), String> {
    let Some(root) = mark_list_state(runtime)?
        .list
        .slot(slot_index)
        .map(Path::to_path_buf)
    else {
        return Ok(());
    };
    let root = canonicalize_project_root_path(&root);
    let open_roots = open_project_workspace_roots(runtime)?;
    match marked_workspace_jump(&root, &open_roots, root.exists()) {
        MarkedWorkspaceJump::NotifyMissing => notify_marked_workspace_missing(runtime, &root),
        MarkedWorkspaceJump::Switch | MarkedWorkspaceJump::OpenThenSwitch => {
            let name = marked_workspace_display_name(&root);
            open_workspace_from_project(runtime, &name, &root)?;
            Ok(())
        }
    }
}

pub(crate) fn open_workspace_from_project(
    runtime: &mut EditorRuntime,
    name: &str,
    root: &std::path::Path,
) -> Result<WorkspaceId, String> {
    if let Some(workspace_id) = find_workspace_by_root(runtime, root)? {
        switch_runtime_workspace(runtime, workspace_id)?;
        return Ok(workspace_id);
    }

    prepare_workspace_leave_for_debug_layout(runtime)?;

    let initial_readme_path = workspace_root_readme_path(root)?;
    let window_id = active_window_id(runtime)?;
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, name, Some(root.to_path_buf()))
        .map_err(|error| error.to_string())?;
    let notes_id = runtime
        .model_mut()
        .create_buffer(workspace_id, "*notes*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    let scratch_id = runtime
        .model_mut()
        .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;

    let (scratch, notes, primary_pane_id) = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let pane_id = workspace
            .active_pane_id()
            .ok_or_else(|| "new workspace has no active pane".to_owned())?;
        let scratch = workspace
            .buffer(scratch_id)
            .ok_or_else(|| "new workspace scratch buffer is missing".to_owned())?;
        let notes = workspace
            .buffer(notes_id)
            .ok_or_else(|| "new workspace notes buffer is missing".to_owned())?;
        (
            ShellBuffer::from_runtime_buffer(
                scratch,
                workspace_scratch_lines(workspace.name(), workspace.root()),
                &*shell_user_library(runtime),
            ),
            ShellBuffer::from_runtime_buffer(
                notes,
                workspace_notes_lines(workspace.name(), workspace.root()),
                &*shell_user_library(runtime),
            ),
            pane_id,
        )
    };

    {
        let ui = shell_ui_mut(runtime)?;
        ui.add_workspace(workspace_id, primary_pane_id, scratch, notes, notes_id);
        ui.switch_workspace(workspace_id);
    }
    prepare_workspace_enter_for_debug_layout(runtime)?;

    invalidate_repository_file_list_cache_for(root);
    queue_workspace_syntax_prewarm(runtime, root);
    // Queue prewarm immediately so the shared worker starts loading grammars. Do not
    // wait: first paint of a cold language may be uncolored until the worker finishes.
    while refresh_pending_syntax_prewarm(runtime).unwrap_or(false) {}

    if let Some(readme_path) = initial_readme_path {
        queue_workspace_readme_open(runtime, readme_path);
    }

    runtime
        .emit_hook(
            builtins::WORKSPACE_OPEN,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_detail(name),
        )
        .map_err(|error| error.to_string())?;

    Ok(workspace_id)
}

fn workspace_language_path_signature(path: &Path) -> String {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        return extension.to_ascii_lowercase();
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn collect_workspace_language_ids(
    registry: &SyntaxRegistry,
    root: &Path,
    files: &[PathBuf],
) -> BTreeSet<String> {
    let mut language_ids = BTreeSet::new();
    let mut seen_signatures = BTreeSet::new();
    for relative_path in files {
        let path = root.join(relative_path);
        let signature = workspace_language_path_signature(&path);
        if !seen_signatures.insert(signature) {
            continue;
        }
        if let Some(language_id) = registry
            .language_for_path(&path)
            .map(|language| language.id().to_owned())
        {
            language_ids.insert(language_id);
        }
    }
    if let Ok(Some(readme_path)) = workspace_root_readme_path(root) {
        let signature = workspace_language_path_signature(&readme_path);
        if seen_signatures.insert(signature)
            && let Some(language_id) = registry
                .language_for_path(&readme_path)
                .map(|language| language.id().to_owned())
        {
            language_ids.insert(language_id);
        }
    }
    language_ids
}

fn queue_workspace_syntax_prewarm(runtime: &mut EditorRuntime, root: &Path) {
    let Ok(ui) = shell_ui_mut(runtime) else {
        return;
    };
    let root = root.to_path_buf();
    if ui
        .pending_syntax_prewarm_roots
        .iter()
        .any(|pending| pending == &root)
    {
        return;
    }
    ui.pending_syntax_prewarm_roots.push_back(root);
}

fn queue_workspace_readme_open(runtime: &mut EditorRuntime, path: PathBuf) {
    let Ok(ui) = shell_ui_mut(runtime) else {
        return;
    };
    if ui
        .pending_workspace_readme_opens
        .iter()
        .any(|pending| pending == &path)
    {
        return;
    }
    ui.pending_workspace_readme_opens.push_back(path);
}

fn refresh_pending_workspace_readme_opens(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let path = shell_ui_mut(runtime)?
        .pending_workspace_readme_opens
        .pop_front();
    let Some(path) = path else {
        return Ok(false);
    };
    open_workspace_file(runtime, &path)?;
    Ok(true)
}

fn refresh_pending_syntax_prewarm(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let root = shell_ui_mut(runtime)?
        .pending_syntax_prewarm_roots
        .pop_front();
    let Some(root) = root else {
        return Ok(false);
    };
    prewarm_workspace_syntax_languages(runtime, &root);
    Ok(true)
}

fn prewarm_workspace_syntax_languages(runtime: &mut EditorRuntime, root: &Path) {
    let language_ids = {
        let Some(registry) = runtime.services().get::<SyntaxRegistry>() else {
            return;
        };
        let candidates = if let Ok(files) = list_repository_files(root) {
            collect_workspace_language_ids(registry, root, &files)
        } else if let Ok(Some(readme_path)) = workspace_root_readme_path(root) {
            let mut language_ids = BTreeSet::new();
            if let Some(language_id) = registry
                .language_for_path(&readme_path)
                .map(|language| language.id().to_owned())
            {
                language_ids.insert(language_id);
            }
            language_ids
        } else {
            BTreeSet::new()
        };
        let mut preloadable = Vec::new();
        for language_id in candidates {
            match registry.is_installed(&language_id) {
                Ok(true) => preloadable.push(language_id),
                Ok(false) => {}
                Err(error) => eprintln!("tree-sitter prewarm skipped `{language_id}`: {error}"),
            }
        }
        preloadable
    };
    if language_ids.is_empty() {
        return;
    }
    // Shared highlight worker loads grammars in the background. UI stays interactive.
    if let Ok(ui) = shell_ui_mut(runtime) {
        ui.syntax_refresh_worker.preload_languages(language_ids);
    }
}

fn picker_preview_syntax_lines(
    runtime: &mut EditorRuntime,
    preview: &str,
) -> Option<IndexedSyntaxLines> {
    let mut lines = preview.lines();
    let path = PathBuf::from(lines.next()?);
    if !path.is_absolute() {
        return None;
    }
    let source_lines = lines.collect::<Vec<_>>();
    if source_lines.is_empty()
        || source_lines.iter().any(|line| {
            line.strip_prefix('>')
                .or_else(|| line.strip_prefix(' '))
                .is_some_and(|rest| {
                    rest.trim_start()
                        .chars()
                        .next()
                        .is_some_and(|ch| ch.is_ascii_digit())
                })
        })
    {
        return None;
    }
    let text = TextBuffer::from_text(source_lines.join("\n"));
    let registry = runtime.services_mut().get_mut::<SyntaxRegistry>()?;
    let snapshot = registry.highlight_buffer_for_path(&path, &text).ok()?;
    Some(index_syntax_lines(snapshot, &text))
}

pub(crate) fn delete_runtime_workspace(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let next_workspace = {
        let ui = shell_ui(runtime)?;
        if workspace_id == ui.default_workspace() {
            return Err("the default workspace cannot be deleted".to_owned());
        }

        if ui.active_workspace() != workspace_id {
            ui.active_workspace()
        } else {
            ui.previous_workspace()
                .filter(|candidate| ui.has_workspace(*candidate) && *candidate != workspace_id)
                .unwrap_or(ui.default_workspace())
        }
    };

    let window_id = active_window_id(runtime)?;
    close_lsp_buffers_for_workspace(runtime, workspace_id)?;
    close_terminal_buffers_for_workspace(runtime, workspace_id)?;
    acp::close_acp_workspace_buffers(runtime, workspace_id)?;
    let removed = runtime
        .model_mut()
        .close_workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.remove_workspace(workspace_id);
    if let Some(state) = runtime.services_mut().get_mut::<LspLogBufferState>() {
        state.remove_workspace(workspace_id);
    }
    runtime
        .emit_hook(
            builtins::WORKSPACE_CLOSE,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_detail(removed.name()),
        )
        .map_err(|error| error.to_string())?;

    switch_runtime_workspace(runtime, next_workspace)
}

fn active_runtime_popup(runtime: &EditorRuntime) -> Result<Option<RuntimePopupSnapshot>, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let Some(popup) = workspace.popups().next() else {
        return Ok(None);
    };

    Ok(Some(RuntimePopupSnapshot {
        active_buffer: popup.active_buffer(),
    }))
}

fn toggle_runtime_popup(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let popup = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .popups()
        .next()
        .map(|popup| {
            (
                popup.id(),
                popup.title().to_owned(),
                popup.buffer_ids().to_vec(),
                popup.active_buffer(),
            )
        });

    if let Some((popup_id, title, buffers, active_buffer)) = popup {
        runtime
            .model_mut()
            .close_popup(workspace_id, popup_id)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.stash_dismissed_popup(workspace_id, title, buffers, active_buffer);
        ui.set_popup_focus(false);
        ui.clear_popup_buffer();
        return Ok(());
    }

    let dismissed_popup = shell_ui(runtime)?.dismissed_popup(workspace_id).cloned();
    if let Some(dismissed_popup) = dismissed_popup {
        let (buffers, active_buffer) = {
            let workspace = runtime
                .model()
                .workspace(workspace_id)
                .map_err(|error| error.to_string())?;
            let buffers = dismissed_popup
                .buffers
                .into_iter()
                .filter(|buffer_id| workspace.buffer(*buffer_id).is_some())
                .collect::<Vec<_>>();
            let active_buffer = if buffers.contains(&dismissed_popup.active_buffer) {
                dismissed_popup.active_buffer
            } else if let Some(active_buffer) = buffers.first().copied() {
                active_buffer
            } else {
                shell_ui_mut(runtime)?.clear_dismissed_popup(workspace_id);
                return Ok(());
            };
            (buffers, active_buffer)
        };
        runtime
            .model_mut()
            .open_popup(workspace_id, dismissed_popup.title, buffers, active_buffer)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.clear_dismissed_popup(workspace_id);
        ui.set_popup_buffer(active_buffer);
        ui.set_popup_focus(true);
        return Ok(());
    }

    let buffer_id = runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup*", BufferKind::Diagnostics, None)
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![buffer_id], buffer_id)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_popup_buffer(
            buffer_id,
            "*popup*",
            BufferKind::Diagnostics,
            &*user_library,
        );
        ui.set_popup_buffer(buffer_id);
    }
    shell_ui_mut(runtime)?.set_popup_focus(true);
    Ok(())
}

fn cycle_runtime_popup_buffer(runtime: &mut EditorRuntime, forward: bool) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = runtime
        .model_mut()
        .cycle_popup_buffer(workspace_id, forward)
        .map_err(|error| error.to_string())?;
    let Some(buffer_id) = buffer_id else {
        return Ok(());
    };
    ensure_shell_buffer(runtime, buffer_id)?;
    shell_ui_mut(runtime)?.set_popup_buffer(buffer_id);
    Ok(())
}

fn split_runtime_pane(
    runtime: &mut EditorRuntime,
    direction: PaneSplitDirection,
) -> Result<(), String> {
    let split_buffer_id = {
        let ui = shell_ui(runtime)?;
        if ui.is_debug_layout_active() {
            return Ok(());
        }
        if ui.pane_count() > 1 {
            return Ok(());
        }
        ui.split_buffer_id()
            .ok_or_else(|| "active workspace view is missing".to_owned())?
    };
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let pane_id = runtime
        .model_mut()
        .split_pane(workspace_id, split_buffer_id)
        .map_err(|error| error.to_string())?;
    let (buffer_name, buffer_kind) = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let buffer = workspace
            .buffer(split_buffer_id)
            .ok_or_else(|| format!("buffer `{split_buffer_id}` is missing"))?;
        (buffer.name().to_owned(), buffer.kind().clone())
    };
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(split_buffer_id, &buffer_name, buffer_kind, &*user_library);
        ui.split_pane(pane_id, split_buffer_id, direction);
        ui.focus_pane(pane_id);
    }
    let window_id = active_window_id(runtime)?;
    let hook_name = match direction {
        PaneSplitDirection::Horizontal => builtins::PANE_SPLIT_HORIZONTAL,
        PaneSplitDirection::Vertical => builtins::PANE_SPLIT_VERTICAL,
    };
    runtime
        .emit_hook(
            hook_name,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_pane(pane_id)
                .with_buffer(split_buffer_id),
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn close_db_multiview(runtime: &mut EditorRuntime) -> Result<(), String> {
    let sidebar_pane_id = shell_ui(runtime)?.db_multiview_sidebar_pane_id();
    shell_ui_mut(runtime)?.set_db_multiview_layout(false);
    if let Some(pane_id) = sidebar_pane_id {
        close_runtime_pane_by_id(runtime, pane_id)?;
    }
    Ok(())
}

fn close_runtime_pane(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let pane_id = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .active_pane_id()
        .ok_or_else(|| format!("workspace `{workspace_id}` has no active pane"))?;
    close_runtime_pane_by_id(runtime, pane_id)
}

fn close_runtime_pane_by_id(runtime: &mut EditorRuntime, pane_id: PaneId) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .close_pane(workspace_id, pane_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.close_pane(pane_id);
    sync_active_buffer(runtime)?;

    let (active_pane_id, active_buffer_id) = active_runtime_buffer(runtime)?
        .map(|(active_pane_id, active_buffer_id, _, _)| (active_pane_id, active_buffer_id))
        .ok_or_else(|| "active runtime surface is missing after closing pane".to_owned())?;
    let window_id = active_window_id(runtime)?;
    runtime
        .emit_hook(
            builtins::PANE_SWITCH,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_pane(active_pane_id)
                .with_buffer(active_buffer_id),
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn switch_runtime_split(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui_mut(runtime)?.switch_split() {
        return Ok(());
    }
    Err("switch split requires an active split".to_owned())
}

fn cycle_runtime_pane(runtime: &mut EditorRuntime) -> Result<(), String> {
    let pane_id = shell_ui_mut(runtime)?.cycle_active_pane();
    let Some(pane_id) = pane_id else {
        return Ok(());
    };
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_pane(workspace_id, pane_id)
        .map_err(|error| error.to_string())?;
    let window_id = active_window_id(runtime)?;
    let buffer_id = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .pane(pane_id)
        .and_then(|pane| pane.active_buffer());
    let mut event = HookEvent::new()
        .with_window(window_id)
        .with_workspace(workspace_id)
        .with_pane(pane_id);
    if let Some(buffer_id) = buffer_id {
        event = event.with_buffer(buffer_id);
    }
    runtime
        .emit_hook(builtins::PANE_SWITCH, event)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn move_workspace_window(
    runtime: &mut EditorRuntime,
    direction: WindowMoveDirection,
) -> Result<(), String> {
    let user_library = shell_user_library(runtime);
    let dock_config = user_library.workspace_dock_config();
    let dock_visible = workspace_dock_visible(&*user_library, shell_ui(runtime)?);
    let dock_focused = shell_ui(runtime)?.workspace_dock_focus_active(&*user_library);
    let acp_visible = acp_dock_visible(shell_ui(runtime)?);
    let acp_focused = shell_ui(runtime)?.acp_dock_focus_active();
    if dock_focused {
        if direction == workspace_dock_exit_direction(dock_config.side) {
            shell_ui_mut(runtime)?.set_workspace_dock_focus(false);
            if dock_config.side == WorkspaceDockSide::Right && acp_visible {
                shell_ui_mut(runtime)?.set_acp_dock_focus(true);
            }
            return Ok(());
        }
        return Ok(());
    }
    if acp_focused {
        if direction == WindowMoveDirection::Left {
            shell_ui_mut(runtime)?.set_acp_dock_focus(false);
            return Ok(());
        }
        if direction == WindowMoveDirection::Right
            && dock_visible
            && dock_config.side == WorkspaceDockSide::Right
        {
            shell_ui_mut(runtime)?.set_acp_dock_focus(false);
            shell_ui_mut(runtime)?.set_workspace_dock_focus(true);
            return Ok(());
        }
        return Ok(());
    }
    if acp_visible && direction == WindowMoveDirection::Right {
        shell_ui_mut(runtime)?.set_acp_dock_focus(true);
        return Ok(());
    }
    if dock_visible && direction == workspace_dock_enter_direction(dock_config.side) {
        shell_ui_mut(runtime)?.set_workspace_dock_focus(true);
        return Ok(());
    }

    if let Some(popup) = active_runtime_popup(runtime)? {
        let (focus_allowed, focus_active) = {
            let ui = shell_ui(runtime)?;
            (
                ui.popup_focus_allowed(&popup),
                ui.popup_focus_active(&popup),
            )
        };
        if focus_allowed {
            let emit_switch = |runtime: &mut EditorRuntime| -> Result<(), String> {
                let window_id = active_window_id(runtime)?;
                let workspace_id = runtime
                    .model()
                    .active_workspace_id()
                    .map_err(|error| error.to_string())?;
                runtime
                    .emit_hook(
                        builtins::PANE_SWITCH,
                        HookEvent::new()
                            .with_window(window_id)
                            .with_workspace(workspace_id),
                    )
                    .map_err(|error| error.to_string())
            };
            if !focus_active && direction == WindowMoveDirection::Down {
                let ui = shell_ui_mut(runtime)?;
                ui.set_popup_buffer(popup.active_buffer);
                ui.set_popup_focus(true);
                emit_switch(runtime)?;
                return Ok(());
            }
            if focus_active
                && matches!(
                    direction,
                    WindowMoveDirection::Up | WindowMoveDirection::Left
                )
            {
                shell_ui_mut(runtime)?.set_popup_focus(false);
                emit_switch(runtime)?;
                return Ok(());
            }
            if focus_active {
                return Ok(());
            }
        }
    }

    let delta = match direction {
        WindowMoveDirection::Left | WindowMoveDirection::Up => -1,
        WindowMoveDirection::Right | WindowMoveDirection::Down => 1,
    };
    let pane_id = shell_ui_mut(runtime)?.shift_active_pane(delta);
    let Some(pane_id) = pane_id else {
        return Ok(());
    };
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_pane(workspace_id, pane_id)
        .map_err(|error| error.to_string())?;
    let window_id = active_window_id(runtime)?;
    let buffer_id = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .pane(pane_id)
        .and_then(|pane| pane.active_buffer());
    let mut event = HookEvent::new()
        .with_window(window_id)
        .with_workspace(workspace_id)
        .with_pane(pane_id);
    if let Some(buffer_id) = buffer_id {
        event = event.with_buffer(buffer_id);
    }
    runtime
        .emit_hook(builtins::PANE_SWITCH, event)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn active_runtime_buffer(
    runtime: &EditorRuntime,
) -> Result<Option<(PaneId, BufferId, String, BufferKind)>, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let Some(pane_id) = workspace.active_pane_id() else {
        return Ok(None);
    };
    let pane = workspace
        .pane(pane_id)
        .ok_or_else(|| format!("pane `{pane_id}` is missing"))?;
    let Some(buffer_id) = pane.active_buffer() else {
        return Ok(None);
    };
    let buffer = workspace
        .buffer(buffer_id)
        .ok_or_else(|| format!("runtime buffer `{buffer_id}` is missing"))?;
    Ok(Some((
        pane_id,
        buffer_id,
        buffer.name().to_owned(),
        buffer.kind().clone(),
    )))
}
