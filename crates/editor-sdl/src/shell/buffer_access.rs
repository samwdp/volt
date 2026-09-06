fn register_hook(runtime: &mut EditorRuntime, name: &str, description: &str) -> Result<(), String> {
    runtime
        .register_hook(name, description)
        .map_err(|error| error.to_string())
}

fn shell_ui(runtime: &EditorRuntime) -> Result<&ShellUiState, String> {
    runtime
        .services()
        .get::<ShellUiState>()
        .ok_or_else(|| "shell UI state service missing".to_owned())
}

fn shell_ui_mut(runtime: &mut EditorRuntime) -> Result<&mut ShellUiState, String> {
    runtime
        .services_mut()
        .get_mut::<ShellUiState>()
        .ok_or_else(|| "shell UI state service missing".to_owned())
}

fn vim_count_digit(chord: &str, has_existing_count: bool) -> Option<usize> {
    let mut characters = chord.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }
    (character.is_ascii_digit() && (character != '0' || has_existing_count))
        .then(|| character.to_digit(10))
        .flatten()
        .map(|digit| digit as usize)
}

fn active_shell_buffer_id(runtime: &EditorRuntime) -> Result<BufferId, String> {
    if let Some(popup) = active_runtime_popup(runtime)? {
        if let Ok(ui) = shell_ui(runtime) {
            if ui.popup_focus_active(&popup) {
                return Ok(popup.active_buffer);
            }
        } else {
            return Ok(popup.active_buffer);
        }
    }

    shell_ui(runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active shell buffer is missing".to_owned())
}

fn active_shell_workspace_id(runtime: &EditorRuntime) -> Option<WorkspaceId> {
    shell_ui(runtime).ok().map(ShellUiState::active_workspace)
}

fn active_shell_buffer_mut(runtime: &mut EditorRuntime) -> Result<&mut ShellBuffer, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    shell_ui_mut(runtime)?
        .buffer_mut(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())
}

fn active_shell_buffer_read_only(runtime: &EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    Ok(shell_buffer(runtime, buffer_id)?.is_read_only())
}

fn active_shell_buffer_has_input(runtime: &EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    Ok(shell_buffer(runtime, buffer_id)?.has_input_field())
}

fn zoom_active_image_buffer_in(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.image_zoom_in();
    Ok(())
}

fn zoom_active_image_buffer_out(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.image_zoom_out();
    Ok(())
}

fn reset_active_image_buffer_zoom(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.reset_image_zoom();
    Ok(())
}

fn toggle_active_image_buffer_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let switched_to_source = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if !buffer.toggle_svg_image_mode()? {
            return Ok(());
        }
        buffer.is_svg_source_mode()
    };
    if switched_to_source {
        queue_buffer_syntax_refresh(runtime, buffer_id)?;
    }
    Ok(())
}

fn toggle_active_markdown_pretty(runtime: &mut EditorRuntime) -> Result<(), String> {
    let default_enabled = shell_user_library(runtime).markdown_pretty_config().enabled;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    if buffer.language_id() != Some("markdown") && !buffer.forced_language {
        // Still allow toggle when Forced Language may be set next; only skip pure non-md.
        if buffer.language_id().is_none() {
            return Ok(());
        }
    }
    buffer.toggle_markdown_pretty(default_enabled);
    Ok(())
}

fn toggle_active_rainbow_parens(runtime: &mut EditorRuntime) -> Result<(), String> {
    let default_enabled = shell_user_library(runtime).rainbow_parens_config().enabled;
    let buffer_id = active_shell_buffer_id(runtime)?;
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.toggle_rainbow_parens(default_enabled);
    }
    queue_buffer_syntax_refresh(runtime, buffer_id)?;
    Ok(())
}

fn toggle_active_show_paren(runtime: &mut EditorRuntime) -> Result<(), String> {
    let default_enabled = shell_user_library(runtime).show_paren_config().enabled;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.toggle_show_paren(default_enabled);
    Ok(())
}

fn enter_insert_mode_for_input_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let has_input = shell_ui(runtime)?
        .buffer(buffer_id)
        .map(ShellBuffer::has_input_field)
        .unwrap_or(false);
    if has_input {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if buffer_is_acp(&buffer.kind) {
            let _ = buffer.focus_acp_input();
        } else if buffer_is_browser(&buffer.kind) {
            let _ = buffer.focus_browser_input();
        }
        let ui = shell_ui_mut(runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    Ok(())
}

fn active_shell_buffer_vim_targets_input(runtime: &EditorRuntime) -> Result<bool, String> {
    Ok(shell_ui(runtime)?.active_buffer_targets_input())
}

fn vim_edit_targets_input(runtime: &EditorRuntime, action: VimEditAction) -> Result<bool, String> {
    if !active_shell_buffer_has_input(runtime)? {
        return Ok(false);
    }
    Ok(match action {
        VimEditAction::DeleteChar
        | VimEditAction::DeleteCharBefore
        | VimEditAction::DeleteLineEnd
        | VimEditAction::ChangeLineEnd
        | VimEditAction::SubstituteChar
        | VimEditAction::SubstituteLine
        | VimEditAction::ReplaceChar
        | VimEditAction::EnterReplaceMode
        | VimEditAction::Append
        | VimEditAction::AppendLineEnd
        | VimEditAction::InsertLineStart
        | VimEditAction::OpenLineBelow
        | VimEditAction::OpenLineAbove
        | VimEditAction::StartDeleteOperator
        | VimEditAction::StartChangeOperator
        | VimEditAction::PutAfter
        | VimEditAction::PutBefore
        | VimEditAction::VisualDelete
        | VimEditAction::VisualChange
        | VimEditAction::VisualReplaceChar => active_shell_buffer_vim_targets_input(runtime)?,
        _ => false,
    })
}

fn active_buffer_event_context(
    runtime: &EditorRuntime,
) -> Result<ActiveBufferEventContext, String> {
    let ui = shell_ui(runtime)?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())?;
    Ok(ActiveBufferEventContext {
        buffer_id,
        has_input: buffer.has_input_field(),
        vim_targets_input: ui.active_buffer_targets_input(),
        is_read_only: buffer.is_read_only(),
        is_git_status: buffer_is_git_status(&buffer.kind),
        is_git_commit: buffer_is_git_commit(&buffer.kind),
        is_git_editor: buffer_is_git_editor(&buffer.kind),
        is_acp: buffer_is_acp(&buffer.kind),
        is_directory: buffer_is_directory(&buffer.kind),
        is_browser: buffer_is_browser(&buffer.kind),
        is_terminal: buffer_is_terminal(&buffer.kind),
        is_db_query: buffer_is_db_query(&buffer.kind),
        is_plugin_evaluatable: plugin_evaluatable_kind(&buffer.kind, runtime),
    })
}

fn active_buffer_revision_key(runtime: &EditorRuntime) -> Option<(BufferId, u64)> {
    let buffer_id = active_shell_buffer_id(runtime).ok()?;
    let revision = shell_buffer(runtime, buffer_id).ok()?.text.revision();
    Some((buffer_id, revision))
}

fn active_lsp_buffer_context(runtime: &EditorRuntime) -> Result<ActiveLspBufferContext, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    lsp_buffer_context(runtime, workspace_id, buffer_id)
}

fn lsp_root_for_buffer(
    runtime: &EditorRuntime,
    buffer: &ShellBuffer,
) -> Result<Option<PathBuf>, String> {
    if buffer_is_db_query(&buffer.kind) {
        return Ok(buffer
            .lsp_path()
            .and_then(Path::parent)
            .map(Path::to_path_buf));
    }
    let Some(path) = buffer.lsp_path() else {
        return Ok(None);
    };
    workspace_root_for_path(runtime, path)
}

fn lsp_buffer_context(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
) -> Result<ActiveLspBufferContext, String> {
    let ui = shell_ui(runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())?;
    let path = buffer
        .lsp_path()
        .map(Path::to_path_buf)
        .ok_or_else(|| "active buffer does not have a file path for LSP".to_owned())?;
    Ok(ActiveLspBufferContext {
        workspace_id,
        buffer_id,
        path: path.clone(),
        text: buffer.text.text(),
        revision: buffer.text.revision(),
        root: lsp_root_for_buffer(runtime, buffer)?,
    })
}

fn apply_sqls_workspace_settings_for_active_buffer_context(
    runtime: &EditorRuntime,
    manager: &LspClientManager,
    context: &ActiveLspBufferContext,
) -> Result<(), String> {
    let buffer = shell_buffer(runtime, context.buffer_id)?;
    apply_sqls_workspace_settings_for_buffer(
        runtime,
        context.buffer_id,
        buffer,
        &context.path,
        context.root.as_deref(),
        manager,
    )
}

fn apply_sqls_workspace_settings_for_buffer(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    buffer: &ShellBuffer,
    path: &Path,
    root: Option<&Path>,
    manager: &LspClientManager,
) -> Result<(), String> {
    if !manager
        .registered_server_ids_for_path_in_workspace(path, root)
        .into_iter()
        .any(|server_id| server_id == SQLS_SERVER_ID)
    {
        return Ok(());
    }
    let (settings, initialization_options) = runtime
        .services()
        .get::<DbService>()
        .map(|db| {
            if buffer_is_db_query(&buffer.kind) {
                (
                    db.sqls_workspace_settings_for_query_buffer(buffer_id.get()),
                    db.sqls_initialization_options_for_query_buffer(buffer_id.get()),
                )
            } else {
                (db.sqls_workspace_settings_for_active_session(), None)
            }
        })
        .unwrap_or((None, None));
    match initialization_options {
        Some(initialization_options) => manager
            .set_server_initialization_options_override(
                SQLS_SERVER_ID,
                root,
                initialization_options,
            )
            .map(|_| ())
            .map_err(|error| error.to_string())?,
        None => manager
            .clear_server_initialization_options_override(SQLS_SERVER_ID, root)
            .map(|_| ())
            .map_err(|error| error.to_string())?,
    }
    match settings {
        Some(settings) => manager
            .set_server_settings_override(SQLS_SERVER_ID, root, settings)
            .map(|_| ())
            .map_err(|error| error.to_string()),
        None => manager
            .clear_server_settings_override(SQLS_SERVER_ID, root)
            .map(|_| ())
            .map_err(|error| error.to_string()),
    }
}

fn cancel_lsp_sync_for_path(runtime: &mut EditorRuntime, path: &Path) -> Result<(), String> {
    shell_ui_mut(runtime)?.lsp_sync_worker.cancel_path(path);
    Ok(())
}

fn buffer_is_git_status(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STATUS_KIND)
}

fn buffer_is_git_commit(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_COMMIT_KIND)
}

fn buffer_is_git_editor(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_EDITOR_KIND)
}

fn buffer_is_acp(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND)
}

fn buffer_is_browser(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == BROWSER_KIND)
}

fn buffer_is_db_connect(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == DB_CONNECT_KIND)
}

fn buffer_is_db_query(kind: &BufferKind) -> bool {
    matches!(
        kind,
        BufferKind::Plugin(plugin_kind)
            if plugin_kind == DB_QUERY_KIND || plugin_kind == DB_DASHBOARD_KIND
    )
}

fn buffer_is_db_dashboard(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == DB_DASHBOARD_KIND)
}

fn buffer_is_db_sidebar(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == DB_SIDEBAR_KIND)
}

fn buffer_is_db_browser(kind: &BufferKind) -> bool {
    matches!(
        kind,
        BufferKind::Plugin(plugin_kind)
            if matches!(
                plugin_kind.as_str(),
                DB_CONNECTIONS_KIND
                    | DB_SCHEMA_KIND
                    | DB_HISTORY_KIND
                    | DB_SNIPPETS_KIND
                    | DB_DASHBOARD_KIND
                    | DB_SIDEBAR_KIND
            )
    )
}

fn buffer_is_quickfix(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Quickfix)
}

/// Returns `true` when the user library has an evaluator for the given buffer
/// kind.  Used to decide whether Ctrl+c Ctrl+c should trigger evaluation.
fn plugin_evaluatable_kind(kind: &BufferKind, runtime: &EditorRuntime) -> bool {
    if let BufferKind::Plugin(plugin_kind) = kind {
        shell_user_library(runtime).supports_plugin_evaluate(plugin_kind)
    } else {
        false
    }
}

fn active_shell_buffer_is_terminal(runtime: &EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    Ok(buffer_is_terminal(&shell_buffer(runtime, buffer_id)?.kind))
}

fn buffer_is_directory(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Directory)
}

fn buffer_is_oil_preview(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == OIL_PREVIEW_KIND)
}

fn report_read_only(runtime: &mut EditorRuntime, action: &str) {
    let message = match active_shell_buffer_id(runtime)
        .ok()
        .and_then(|buffer_id| shell_buffer(runtime, buffer_id).ok())
    {
        Some(buffer) => format!("buffer `{}` is read-only; {action}", buffer.display_name()),
        None => format!("read-only buffer; {action}"),
    };
    record_runtime_error(runtime, "buffer.read-only", message);
}

fn vim_edit_requires_write(action: VimEditAction) -> bool {
    matches!(
        action,
        VimEditAction::DeleteChar
            | VimEditAction::DeleteCharBefore
            | VimEditAction::DeleteLineEnd
            | VimEditAction::ChangeLineEnd
            | VimEditAction::SubstituteChar
            | VimEditAction::SubstituteLine
            | VimEditAction::ReplaceChar
            | VimEditAction::EnterReplaceMode
            | VimEditAction::ToggleCase
            | VimEditAction::ToggleLineComment
            | VimEditAction::Append
            | VimEditAction::AppendLineEnd
            | VimEditAction::InsertLineStart
            | VimEditAction::OpenLineBelow
            | VimEditAction::OpenLineAbove
            | VimEditAction::Undo
            | VimEditAction::Redo
            | VimEditAction::StartDeleteOperator
            | VimEditAction::StartChangeOperator
            | VimEditAction::StartFormatOperator
            | VimEditAction::PutAfter
            | VimEditAction::PutBefore
            | VimEditAction::VisualDelete
            | VimEditAction::VisualChange
            | VimEditAction::VisualReplaceChar
            | VimEditAction::VisualFormat
            | VimEditAction::VisualToggleComment
            | VimEditAction::VisualToggleCase
            | VimEditAction::VisualLowercase
            | VimEditAction::VisualUppercase
            | VimEditAction::VisualIndent
            | VimEditAction::VisualOutdent
            | VimEditAction::VisualJoin
            | VimEditAction::VisualMoveDown
            | VimEditAction::VisualMoveUp
            | VimEditAction::VisualBlockInsert
            | VimEditAction::VisualBlockAppend
    )
}

fn handle_terminal_vim_edit(
    runtime: &mut EditorRuntime,
    action: VimEditAction,
) -> Result<bool, String> {
    if !active_shell_buffer_is_terminal(runtime)? {
        return Ok(false);
    }
    match action {
        VimEditAction::Append
        | VimEditAction::AppendLineEnd
        | VimEditAction::InsertLineStart
        | VimEditAction::OpenLineBelow
        | VimEditAction::OpenLineAbove
        | VimEditAction::SubstituteChar
        | VimEditAction::SubstituteLine => {
            shell_ui_mut(runtime)?.enter_insert_mode();
            Ok(true)
        }
        VimEditAction::EnterReplaceMode | VimEditAction::ReplaceChar => {
            shell_ui_mut(runtime)?.enter_replace_mode();
            Ok(true)
        }
        VimEditAction::PutAfter => {
            put_yank(runtime, true)?;
            Ok(true)
        }
        VimEditAction::PutBefore => {
            put_yank(runtime, false)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn shell_buffer(runtime: &EditorRuntime, buffer_id: BufferId) -> Result<&ShellBuffer, String> {
    shell_ui(runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))
}

fn shell_buffer_mut(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<&mut ShellBuffer, String> {
    shell_ui_mut(runtime)?
        .buffer_mut(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))
}
