fn start_lsp_for_active_buffer(
    runtime: &mut EditorRuntime,
    preferred_server_id: Option<&str>,
) -> Result<(), String> {
    run_lsp_start_for_active_buffer(runtime, preferred_server_id)
}

fn copilot_sign_in_for_active_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    validate_lsp_server_request(
        &lsp_client,
        &context.path,
        context.root.as_deref(),
        Some(COPILOT_LANGUAGE_SERVER),
    )?;
    execute_lsp_start_for_buffer(
        runtime,
        &lsp_client,
        &context,
        Some(COPILOT_LANGUAGE_SERVER),
    )?;
    begin_copilot_sign_in(runtime, context.root.as_deref())
}

fn copilot_sign_out_for_active_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    validate_lsp_server_request(
        &lsp_client,
        &context.path,
        context.root.as_deref(),
        Some(COPILOT_LANGUAGE_SERVER),
    )?;
    execute_lsp_start_for_buffer(
        runtime,
        &lsp_client,
        &context,
        Some(COPILOT_LANGUAGE_SERVER),
    )?;
    let signed_out = lsp_client
        .copilot_sign_out(context.root.as_deref())
        .map_err(|error| error.to_string())?;
    if !signed_out {
        return Err("Copilot language server is not running".to_owned());
    }
    apply_copilot_auth_notification(
        runtime,
        "copilot.sign-out",
        NotificationSeverity::Info,
        "Copilot sign-out requested",
        vec!["Copilot session sign-out sent to language server.".to_owned()],
        false,
    )?;
    Ok(())
}

fn begin_copilot_sign_in(runtime: &mut EditorRuntime, root: Option<&Path>) -> Result<(), String> {
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let prompt = lsp_client
        .copilot_sign_in(root)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Copilot language server is not running".to_owned())?;
    write_system_clipboard(prompt.user_code());
    apply_copilot_auth_notification(
        runtime,
        &copilot_status_notification_key(root),
        NotificationSeverity::Info,
        "Copilot sign-in started",
        vec![
            format!("Device code: {}", prompt.user_code()),
            "Code copied to clipboard.".to_owned(),
            "Enter code in GitHub browser flow.".to_owned(),
        ],
        true,
    )?;
    lsp_client
        .execute_server_command(COPILOT_LANGUAGE_SERVER, root, prompt.command())
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn copilot_status_notification_key(root: Option<&Path>) -> String {
    match root {
        Some(root) => format!("status:{COPILOT_LANGUAGE_SERVER}:{}", root.display()),
        None => format!("status:{COPILOT_LANGUAGE_SERVER}:global"),
    }
}

fn apply_copilot_auth_notification(
    runtime: &mut EditorRuntime,
    key: &str,
    severity: NotificationSeverity,
    title: &str,
    body_lines: Vec<String>,
    active: bool,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.apply_notification(
        NotificationUpdate {
            key: key.to_owned(),
            severity,
            title: title.to_owned(),
            body_lines,
            progress: None,
            active,
            action: None,
            workspace_id: None,
        },
        Instant::now(),
    );
    Ok(())
}

fn open_lsp_session_stop_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_lsp_session_lifecycle_picker(runtime, LspSessionPickerAction::Stop)
}

fn open_lsp_session_restart_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_lsp_session_lifecycle_picker(runtime, LspSessionPickerAction::Restart)
}

#[derive(Debug, Clone, Copy)]
enum LspSessionPickerAction {
    Stop,
    Restart,
}

fn open_lsp_session_lifecycle_picker(
    runtime: &mut EditorRuntime,
    action: LspSessionPickerAction,
) -> Result<(), String> {
    let sessions = live_lsp_sessions_for_active_workspace(runtime)?;
    if sessions.is_empty() {
        return Err("no running Language Server Sessions".to_owned());
    }
    let picker = lsp_session_lifecycle_picker_overlay(action, &sessions);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

fn live_lsp_sessions_for_active_workspace(
    runtime: &EditorRuntime,
) -> Result<Vec<LspLiveSession>, String> {
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let project_workspace_root = active_workspace_root(runtime)?;
    let open_buffer_paths = active_workspace_open_buffer_paths(runtime)?;
    lsp_client
        .live_sessions_for_workspace(&open_buffer_paths, project_workspace_root.as_deref())
        .map_err(|error| error.to_string())
}

fn active_workspace_open_buffer_paths(runtime: &EditorRuntime) -> Result<Vec<PathBuf>, String> {
    let ui = shell_ui(runtime)?;
    Ok(ui
        .active_workspace_buffer_ids()
        .into_iter()
        .flatten()
        .filter_map(|buffer_id| ui.buffer(*buffer_id))
        .filter_map(|buffer| buffer.path().map(Path::to_path_buf))
        .collect())
}

fn lsp_session_lifecycle_picker_overlay(
    action: LspSessionPickerAction,
    sessions: &[LspLiveSession],
) -> PickerOverlay {
    let title = match action {
        LspSessionPickerAction::Stop => "Stop Language Server Session",
        LspSessionPickerAction::Restart => "Restart Language Server Session",
    };
    let entries = sessions
        .iter()
        .map(|session| lsp_session_lifecycle_picker_entry(action, session))
        .collect();
    PickerOverlay::from_entries(title, entries)
}

fn lsp_session_lifecycle_picker_entry(
    action: LspSessionPickerAction,
    session: &LspLiveSession,
) -> PickerEntry {
    let root = session.root().map(Path::to_path_buf);
    let picker_action = match action {
        LspSessionPickerAction::Stop => PickerAction::StopLspSession {
            server_id: session.server_id().to_owned(),
            root: root.clone(),
        },
        LspSessionPickerAction::Restart => PickerAction::RestartLspSession {
            server_id: session.server_id().to_owned(),
            root: root.clone(),
        },
    };
    let id = match root.as_deref() {
        Some(path) => format!("lsp-session:{}:{}", session.server_id(), path.display()),
        None => format!("lsp-session:{}:", session.server_id()),
    };
    PickerEntry {
        item: PickerItem::new(id, session.picker_label(), String::new(), None::<String>),
        action: picker_action,
        quickfix: None,
    }
}

fn stop_lsp_session(
    runtime: &mut EditorRuntime,
    server_id: &str,
    root: Option<&Path>,
) -> Result<(), String> {
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let paths = lsp_client
        .stop_session(server_id, root)
        .map_err(|error| error.to_string())?;
    clear_lsp_ui_for_stopped_paths(runtime, &paths)
}

fn restart_lsp_session(
    runtime: &mut EditorRuntime,
    server_id: &str,
    root: Option<&Path>,
) -> Result<(), String> {
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let paths = lsp_client
        .restart_session(server_id, root)
        .map_err(|error| error.to_string())?;
    clear_lsp_ui_for_stopped_paths(runtime, &paths)?;
    for path in paths {
        let Some(context) = lsp_buffer_context_for_path(runtime, &path)? else {
            continue;
        };
        cancel_lsp_sync_for_path(runtime, &path)?;
        lsp_client
            .sync_buffer_onto_session(
                &context.path,
                &context.text,
                context.revision,
                server_id,
                root,
            )
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(context.buffer_id) {
            buffer.set_lsp_enabled(true);
            buffer.set_lsp_diagnostics(Vec::new());
        }
        ui.set_attached_lsp_server(context.workspace_id, Some(server_id.to_owned()));
    }
    Ok(())
}

fn clear_lsp_ui_for_stopped_paths(
    runtime: &mut EditorRuntime,
    paths: &[PathBuf],
) -> Result<(), String> {
    for path in paths {
        cancel_lsp_sync_for_path(runtime, path)?;
    }
    let path_set = paths.iter().cloned().collect::<HashSet<_>>();
    let ui = shell_ui_mut(runtime)?;
    let mut touched_workspaces = BTreeSet::new();
    for buffer in &mut ui.buffers {
        let Some(path) = buffer.path().map(Path::to_path_buf) else {
            continue;
        };
        if !path_set.contains(&path) {
            continue;
        }
        buffer.set_lsp_diagnostics(Vec::new());
        let buffer_id = buffer.id();
        for (workspace_id, view) in &ui.workspace_views {
            if view.buffer_ids.contains(&buffer_id) {
                touched_workspaces.insert(*workspace_id);
            }
        }
    }
    for workspace_id in touched_workspaces {
        ui.set_attached_lsp_server(workspace_id, None);
    }
    Ok(())
}

fn lsp_buffer_context_for_path(
    runtime: &EditorRuntime,
    path: &Path,
) -> Result<Option<ActiveLspBufferContext>, String> {
    let ui = shell_ui(runtime)?;
    let Some(buffer_id) = ui
        .buffers
        .iter()
        .find(|buffer| buffer.path() == Some(path))
        .map(ShellBuffer::id)
    else {
        return Ok(None);
    };
    let Some(workspace_id) = ui
        .workspace_views
        .iter()
        .find(|(_, view)| view.buffer_ids.contains(&buffer_id))
        .map(|(workspace_id, _)| *workspace_id)
    else {
        return Ok(None);
    };
    Ok(Some(lsp_buffer_context(runtime, workspace_id, buffer_id)?))
}

fn run_lsp_start_for_active_buffer(
    runtime: &mut EditorRuntime,
    preferred_server_id: Option<&str>,
) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    validate_lsp_server_request(
        &lsp_client,
        &context.path,
        context.root.as_deref(),
        preferred_server_id,
    )?;
    execute_lsp_start_for_buffer(runtime, &lsp_client, &context, preferred_server_id)
}

fn validate_lsp_server_request(
    manager: &LspClientManager,
    path: &Path,
    workspace_root: Option<&Path>,
    preferred_server_id: Option<&str>,
) -> Result<(), String> {
    if let Some(server_id) = preferred_server_id {
        let supported = manager
            .registered_server_ids_for_path_in_workspace(path, workspace_root)
            .into_iter()
            .any(|registered| registered == server_id);
        if !supported {
            return Err(format!(
                "language server `{server_id}` is not registered for `{}`",
                path.display()
            ));
        }
        return Ok(());
    }
    if !manager.supports_path_in_workspace(path, workspace_root) {
        return Err(format!(
            "no language server is registered for `{}`",
            path.display()
        ));
    }
    Ok(())
}

fn execute_lsp_start_for_buffer(
    runtime: &mut EditorRuntime,
    manager: &LspClientManager,
    context: &ActiveLspBufferContext,
    preferred_server_id: Option<&str>,
) -> Result<(), String> {
    if tool_install::queue_missing_language_server_installs(
        runtime,
        manager,
        context,
        preferred_server_id,
    )? {
        return Ok(());
    }
    cancel_lsp_sync_for_path(runtime, &context.path)?;
    apply_sqls_workspace_settings_for_active_buffer_context(runtime, manager, context)?;
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(context.buffer_id) {
            buffer.set_lsp_enabled(true);
        }
    }
    schedule_immediate_lsp_sync(runtime, context, preferred_server_id)
}

fn lsp_edits_since_last_sync(
    lsp_client: &LspClientManager,
    path: &Path,
    buffer: &TextBuffer,
) -> Option<Vec<TextEdit>> {
    lsp_client
        .last_synced_revision(path)
        .and_then(|revision| buffer.edits_since(revision))
}

fn schedule_immediate_lsp_sync(
    runtime: &mut EditorRuntime,
    context: &ActiveLspBufferContext,
    preferred_server_id: Option<&str>,
) -> Result<(), String> {
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let edits = shell_ui(runtime)?
        .buffer(context.buffer_id)
        .and_then(|buffer| lsp_edits_since_last_sync(&lsp_client, &context.path, &buffer.text));
    let request = LspSyncWorkerRequest {
        path: context.path.clone(),
        revision: context.revision,
        text: TextBuffer::from_text(&context.text).snapshot(),
        root: context.root.clone(),
        lsp_client,
        preferred_server_id: preferred_server_id.map(str::to_owned),
        edits,
    };
    let ui = shell_ui_mut(runtime)?;
    ui.lsp_sync_worker.schedule(request, false);
    ui.lsp_sync_worker.dispatch_due(Instant::now())
}

fn open_lsp_log_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let server_id = preferred_lsp_log_server(runtime, workspace_id)?
        .ok_or_else(|| "no active LSP server log is available".to_owned())?;
    let buffer_id = ensure_lsp_log_buffer(runtime, workspace_id, &server_id)?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let ui = shell_ui_mut(runtime)?;
    ui.focus_buffer_in_active_pane(buffer_id);
    ui.enter_normal_mode();
    Ok(())
}

fn goto_lsp_definition(runtime: &mut EditorRuntime) -> Result<(), String> {
    navigate_to_lsp_locations(runtime, "Definitions", LspClientManager::definitions)
}

fn goto_lsp_references(runtime: &mut EditorRuntime) -> Result<(), String> {
    navigate_to_lsp_locations(runtime, "References", LspClientManager::references)
}

fn goto_lsp_implementation(runtime: &mut EditorRuntime) -> Result<(), String> {
    navigate_to_lsp_locations(
        runtime,
        "Implementations",
        LspClientManager::implementations,
    )
}

fn lsp_diagnostics_for_active_workspace(
    runtime: &EditorRuntime,
    diagnostics: Vec<LspWorkspaceDiagnostic>,
) -> Result<Vec<LspWorkspaceDiagnostic>, String> {
    let active_root = active_workspace_root(runtime)?;
    let active_buffer_paths = {
        let ui = shell_ui(runtime)?;
        ui.active_workspace_buffer_ids()
            .into_iter()
            .flatten()
            .filter_map(|buffer_id| ui.buffer(*buffer_id))
            .filter_map(|buffer| buffer.path().map(Path::to_path_buf))
            .collect::<HashSet<_>>()
    };
    let mut filtered = Vec::new();
    for diagnostic in diagnostics {
        let resolved_root = workspace_root_for_path(runtime, diagnostic.path())?;
        if lsp_diagnostic_belongs_to_workspace(
            active_root.as_deref(),
            resolved_root.as_deref(),
            &active_buffer_paths,
            diagnostic.path(),
        ) {
            filtered.push(diagnostic);
        }
    }
    Ok(filtered)
}

fn lsp_diagnostic_belongs_to_workspace(
    active_root: Option<&Path>,
    resolved_root: Option<&Path>,
    active_buffer_paths: &HashSet<PathBuf>,
    diagnostic_path: &Path,
) -> bool {
    active_buffer_paths.contains(diagnostic_path)
        || matches!(
            (active_root, resolved_root),
            (Some(active_root), Some(resolved_root)) if resolved_root == active_root
        )
}

fn open_lsp_diagnostics(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        let picker = lsp_diagnostics_status_picker_overlay(
            "Diagnostics unavailable",
            "LSP client manager service is missing.",
            None,
        );
        shell_ui_mut(runtime)?.set_picker(picker);
        return Ok(());
    };
    let diagnostics =
        lsp_diagnostics_for_active_workspace(runtime, lsp_client.workspace_diagnostics())?;
    if diagnostics.is_empty() {
        let picker = lsp_diagnostics_status_picker_overlay(
            "No diagnostics available",
            "No live LSP diagnostics are currently available from active language servers.",
            active_workspace_root(runtime)
                .ok()
                .flatten()
                .map(|root| root.display().to_string()),
        );
        shell_ui_mut(runtime)?.set_picker(picker);
        return Ok(());
    }
    let picker = lsp_diagnostics_picker_overlay(runtime, &diagnostics);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

#[cfg(test)]
mod lsp_diagnostic_scope_tests;

fn open_lsp_code_actions(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let range = active_lsp_code_action_range(runtime, context.buffer_id)?;
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    cancel_lsp_sync_for_path(runtime, &context.path)?;
    apply_sqls_workspace_settings_for_active_buffer_context(runtime, &lsp_client, &context)?;
    let load_code_actions = || -> Result<(Vec<String>, Vec<LspCodeAction>), String> {
        let labels = lsp_client
            .sync_buffer(
                &context.path,
                &context.text,
                context.revision,
                context.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        let code_actions = lsp_client
            .code_actions(&context.path, range)
            .map_err(|error| error.to_string())?;
        Ok((labels, code_actions))
    };
    let (labels, code_actions) = match load_code_actions() {
        Ok(result) => result,
        Err(error) => {
            record_runtime_error(
                runtime,
                "lsp.code-actions",
                format!(
                    "failed to load code actions for `{}`: {error}",
                    context.path.display()
                ),
            );
            let picker = lsp_code_actions_status_picker_overlay(
                "Code actions unavailable",
                &error,
                Some(format!("Path: {}", context.path.display())),
            );
            shell_ui_mut(runtime)?.set_picker(picker);
            return Ok(());
        }
    };
    sync_lsp_buffer_state(runtime, context.workspace_id, context.buffer_id, &labels)?;
    if code_actions.is_empty() {
        let picker = lsp_code_actions_status_picker_overlay(
            "No code actions available",
            "The active cursor position does not expose any LSP code actions.",
            Some(context.path.display().to_string()),
        );
        shell_ui_mut(runtime)?.set_picker(picker);
        return Ok(());
    }
    let picker = lsp_code_actions_picker_overlay(
        context.workspace_id,
        context.buffer_id,
        &context.path,
        &code_actions,
    );
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

fn active_lsp_code_action_range(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<TextRange, String> {
    let ui = shell_ui(runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())?;
    match ui.visual_selection_for_buffer(buffer, true) {
        Some(VisualSelection::Range(range)) => Ok(range),
        Some(VisualSelection::Block(_)) => {
            Err("LSP code actions do not support block selections".to_owned())
        }
        None => {
            let cursor = buffer.cursor_point();
            Ok(TextRange::new(cursor, cursor))
        }
    }
}

fn navigate_to_lsp_locations(
    runtime: &mut EditorRuntime,
    title: &str,
    request: fn(&LspClientManager, &Path, TextPoint) -> Result<Vec<LspLocation>, LspClientError>,
) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let position = shell_buffer(runtime, context.buffer_id)?.cursor_point();
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    cancel_lsp_sync_for_path(runtime, &context.path)?;
    apply_sqls_workspace_settings_for_active_buffer_context(runtime, &lsp_client, &context)?;
    let (labels, locations) = {
        let labels = lsp_client
            .sync_buffer(
                &context.path,
                &context.text,
                context.revision,
                context.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        let locations =
            request(&lsp_client, &context.path, position).map_err(|error| error.to_string())?;
        (labels, locations)
    };
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(context.buffer_id) {
            buffer.set_lsp_enabled(true);
        }
        ui.set_attached_lsp_server(
            context.workspace_id,
            (!labels.is_empty()).then(|| labels.join(", ")),
        );
    }
    open_lsp_locations(runtime, title, locations)
}

fn open_lsp_locations(
    runtime: &mut EditorRuntime,
    title: &str,
    locations: Vec<LspLocation>,
) -> Result<(), String> {
    let Some(location) = locations.first() else {
        return Err(format!("no {} found at cursor", title.to_ascii_lowercase()));
    };
    if locations.len() == 1 {
        open_lsp_location(runtime, location)?;
        sync_active_buffer(runtime)?;
        return Ok(());
    }
    let picker = lsp_locations_picker_overlay(runtime, title, &locations);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

fn open_lsp_location(runtime: &mut EditorRuntime, location: &LspLocation) -> Result<(), String> {
    if let Some(path) = location.file_path() {
        return open_workspace_file_at(runtime, path, location.range().start());
    }
    Err(format!(
        "unsupported LSP location URI `{}` returned by `{}`",
        location.uri(),
        location.server_id()
    ))
}

fn quickfix_state(runtime: &EditorRuntime) -> Result<&QuickfixState, String> {
    runtime
        .services()
        .get::<QuickfixState>()
        .ok_or_else(|| "quickfix state service missing".to_owned())
}

fn quickfix_state_mut(runtime: &mut EditorRuntime) -> Result<&mut QuickfixState, String> {
    runtime
        .services_mut()
        .get_mut::<QuickfixState>()
        .ok_or_else(|| "quickfix state service missing".to_owned())
}

fn quickfix_buffer_exists(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
) -> bool {
    runtime
        .model()
        .workspace(workspace_id)
        .ok()
        .and_then(|workspace| workspace.buffer(buffer_id))
        .is_some()
}

fn sync_quickfix_popup_buffer(
    runtime: &mut EditorRuntime,
    focus_popup: bool,
) -> Result<Option<BufferId>, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let (entries, selected_index, existing_buffer_id) = {
        let state = quickfix_state(runtime)?;
        if state.is_empty() {
            return Ok(None);
        }
        (
            state.render_lines(),
            state.selected_index(),
            state.buffer_id(),
        )
    };
    let buffer_id = if let Some(existing) = existing_buffer_id {
        if quickfix_buffer_exists(runtime, workspace_id, existing) {
            existing
        } else {
            let created = runtime
                .model_mut()
                .create_popup_buffer(
                    workspace_id,
                    QUICKFIX_BUFFER_NAME,
                    BufferKind::Quickfix,
                    None,
                )
                .map_err(|error| error.to_string())?;
            quickfix_state_mut(runtime)?.set_buffer_id(created);
            created
        }
    } else {
        let created = runtime
            .model_mut()
            .create_popup_buffer(
                workspace_id,
                QUICKFIX_BUFFER_NAME,
                BufferKind::Quickfix,
                None,
            )
            .map_err(|error| error.to_string())?;
        quickfix_state_mut(runtime)?.set_buffer_id(created);
        created
    };
    runtime
        .model_mut()
        .open_popup_buffer(workspace_id, QUICKFIX_POPUP_TITLE, buffer_id)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        let buffer = ui.ensure_popup_buffer(
            buffer_id,
            QUICKFIX_BUFFER_NAME,
            BufferKind::Quickfix,
            &*user_library,
        );
        buffer.replace_with_lines_preserve_view(entries);
        let _ = buffer.goto_line(selected_index);
    }
    {
        let ui = shell_ui_mut(runtime)?;
        ui.set_popup_buffer(buffer_id);
        ui.set_popup_focus(focus_popup);
    }
    Ok(Some(buffer_id))
}

fn quickfix_entries_from_one_shot(rows: &[PickerExportableRow]) -> Vec<QuickfixEntry> {
    rows.iter()
        .map(|row| {
            QuickfixEntry::new(
                row.id(),
                PathBuf::from(row.path()),
                TextPoint::new(row.line(), row.column()),
                row.label(),
            )
        })
        .collect()
}

fn quickfix_open_from_one_shot(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let Some(context) = shell_ui_mut(runtime)?.take_picker_one_shot() else {
        return Ok(false);
    };
    let entries = quickfix_entries_from_one_shot(context.exportable_quickfix());
    if entries.is_empty() {
        return Ok(false);
    }
    quickfix_state_mut(runtime)?.set_entries(entries);
    sync_quickfix_popup_buffer(runtime, true)?;
    Ok(true)
}

fn quickfix_open_picker_matches(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let entries = {
        let ui = shell_ui(runtime)?;
        let Some(picker) = ui.picker() else {
            return Ok(false);
        };
        picker.exportable_quickfix_entries()
    };
    if entries.is_empty() {
        return Ok(false);
    }
    {
        let ui = shell_ui_mut(runtime)?;
        ui.close_picker();
    }
    quickfix_state_mut(runtime)?.set_entries(entries);
    sync_quickfix_popup_buffer(runtime, true)?;
    Ok(true)
}

fn quickfix_entry_for_cursor(runtime: &mut EditorRuntime) -> Result<Option<QuickfixEntry>, String> {
    let row = {
        let buffer_id = active_shell_buffer_id(runtime)?;
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_quickfix(&buffer.kind) {
            return Ok(None);
        }
        buffer.cursor_row()
    };
    let state = quickfix_state_mut(runtime)?;
    state.set_selected_index(row);
    Ok(state.selected_entry())
}

fn quickfix_open_entry(
    runtime: &mut EditorRuntime,
    entry: &QuickfixEntry,
    focus_popup: bool,
) -> Result<(), String> {
    open_workspace_file_at(runtime, entry.path(), entry.target())?;
    if !focus_popup {
        shell_ui_mut(runtime)?.set_popup_focus(false);
    }
    sync_active_buffer(runtime)?;
    Ok(())
}

fn quickfix_open_selected_entry(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let Some(entry) = quickfix_entry_for_cursor(runtime)? else {
        return Ok(false);
    };
    quickfix_open_entry(runtime, &entry, false)?;
    Ok(true)
}

fn quickfix_open_current_list(runtime: &mut EditorRuntime) -> Result<(), String> {
    if quickfix_open_from_one_shot(runtime)? {
        return Ok(());
    }
    if quickfix_open_picker_matches(runtime)? {
        return Ok(());
    }
    let _ = sync_quickfix_popup_buffer(runtime, true)?;
    Ok(())
}

fn quickfix_select_next(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(entry) = quickfix_state_mut(runtime)?.select_next() else {
        return Ok(());
    };
    let _ = sync_quickfix_popup_buffer(runtime, false)?;
    quickfix_open_entry(runtime, &entry, false)
}

fn quickfix_select_previous(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(entry) = quickfix_state_mut(runtime)?.select_previous() else {
        return Ok(());
    };
    let _ = sync_quickfix_popup_buffer(runtime, false)?;
    quickfix_open_entry(runtime, &entry, false)
}

fn quickfix_toggle_mark(runtime: &mut EditorRuntime) -> Result<(), String> {
    let row = {
        let Some(entry) = quickfix_entry_for_cursor(runtime)? else {
            return Ok(());
        };
        let state = quickfix_state(runtime)?;
        state
            .entries()
            .iter()
            .position(|candidate| candidate.id() == entry.id())
            .unwrap_or(0)
    };
    if quickfix_state_mut(runtime)?.toggle_mark_at(row) {
        let _ = sync_quickfix_popup_buffer(runtime, true)?;
    }
    Ok(())
}

fn quickfix_clear_marks(runtime: &mut EditorRuntime) -> Result<(), String> {
    quickfix_state_mut(runtime)?.clear_marks();
    let _ = sync_quickfix_popup_buffer(runtime, true)?;
    Ok(())
}

fn quickfix_mark_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    quickfix_state_mut(runtime)?.mark_all();
    let _ = sync_quickfix_popup_buffer(runtime, true)?;
    Ok(())
}

fn ensure_lsp_log_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    server_id: &str,
) -> Result<BufferId, String> {
    if let Some(buffer_id) = runtime
        .services()
        .get::<LspLogBufferState>()
        .and_then(|state| state.buffer_id(workspace_id, server_id))
    {
        return Ok(buffer_id);
    }

    let snapshot = current_lsp_log_snapshot(runtime)?;
    let entries = lsp_log_entries_for_server(snapshot.entries(), server_id);
    let buffer_name = lsp_log_buffer_name(server_id);
    let buffer_id = runtime
        .model_mut()
        .create_popup_buffer(workspace_id, &buffer_name, BufferKind::Diagnostics, None)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(
            buffer_id,
            &buffer_name,
            BufferKind::Diagnostics,
            &*user_library,
        )
        .replace_with_lines_follow_output(lsp_log_buffer_lines(server_id, &entries));
    }
    runtime
        .services_mut()
        .get_mut::<LspLogBufferState>()
        .ok_or_else(|| "LSP log buffer service missing".to_owned())?
        .insert_buffer(workspace_id, server_id.to_owned(), buffer_id);
    Ok(buffer_id)
}

fn current_lsp_log_snapshot(runtime: &EditorRuntime) -> Result<LspLogSnapshot, String> {
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>() else {
        return Ok(LspLogSnapshot::default());
    };
    Ok(lsp_client.log_snapshot())
}

fn preferred_lsp_log_server(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<Option<String>, String> {
    if let Ok(context) = active_lsp_buffer_context(runtime)
        && let Some(manager) = runtime.services().get::<Arc<LspClientManager>>()
        && let Some(server_id) = manager
            .session_labels_for_path(&context.path)
            .into_iter()
            .next()
    {
        return Ok(Some(server_id));
    }
    if let Some(server_id) = shell_ui(runtime)?
        .attached_lsp_servers
        .get(&workspace_id)
        .and_then(|labels| labels.split(", ").next())
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
    {
        return Ok(Some(server_id));
    }
    Ok(runtime
        .services()
        .get::<LspLogBufferState>()
        .and_then(|state| state.buffers_for_workspace(workspace_id).into_iter().next())
        .map(|(server_id, _)| server_id))
}

fn lsp_log_buffer_name(server_id: &str) -> String {
    format!("{LSP_LOG_BUFFER_PREFIX}{server_id}*")
}
