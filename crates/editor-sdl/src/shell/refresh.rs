fn refresh_pending_syntax(runtime: &mut EditorRuntime) -> Result<SyntaxRefreshStats, String> {
    let mut stats = SyntaxRefreshStats::default();
    if runtime.services().get::<SyntaxRegistry>().is_none() {
        return Ok(stats);
    }
    if let Some(buffer_id) = shell_ui(runtime)?.active_buffer_id()
        && let Some(buffer) = shell_ui_mut(runtime)?.buffer_mut(buffer_id)
    {
        buffer.ensure_visible_syntax_window();
    }
    let syntax_results = shell_ui(runtime)?.syntax_refresh_worker.take_results();
    if !syntax_results.is_empty() {
        let ui = shell_ui_mut(runtime)?;
        for result in syntax_results {
            let Some(buffer) = ui.buffer_mut(result.buffer_id) else {
                continue;
            };
            let current_path = buffer.path().map(Path::to_path_buf);
            let current_language_id = buffer.language_id().map(str::to_owned);
            if buffer.text.revision() != result.buffer_revision
                || current_path.as_deref() != result.path.as_deref()
                || current_language_id.as_deref() != result.buffer_language_id.as_deref()
            {
                continue;
            }
            stats.changed = true;
            stats.worker_compute += result.compute_elapsed;
            stats.result_count = stats.result_count.saturating_add(1);
            stats.highlight_spans = stats
                .highlight_spans
                .saturating_add(result.highlight_span_count);
            buffer.set_language_id(result.language_id.clone());
            match result.syntax_result {
                Some(Ok(syntax_lines)) => {
                    buffer.set_indexed_syntax_lines(Some(syntax_lines), result.syntax_window);
                    buffer.set_syntax_error(None);
                }
                Some(Err(error)) => {
                    let error_label = result
                        .path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .or(result.language_id.clone())
                        .unwrap_or_else(|| "buffer".to_owned());
                    eprintln!("tree-sitter syntax refresh failed for `{error_label}`: {error}");
                    buffer.set_syntax_snapshot(None);
                    buffer.set_syntax_error(Some(error));
                }
                None => {
                    buffer.set_syntax_snapshot(None);
                    buffer.set_syntax_error(None);
                    buffer.set_language_id(None);
                }
            }
        }
    }

    if !shell_ui(runtime)?.syntax_refresh_worker.is_configured() {
        let now = Instant::now();
        let buffer_ids = {
            let ui = shell_ui(runtime)?;
            ui.buffers
                .iter()
                .filter(|buffer| buffer.syntax_refresh_due(now))
                .map(ShellBuffer::id)
                .collect::<Vec<_>>()
        };
        let had_due_buffers = !buffer_ids.is_empty();

        for buffer_id in buffer_ids {
            refresh_buffer_syntax(runtime, buffer_id)?;
        }

        stats.changed = stats.changed || had_due_buffers;
        return Ok(stats);
    }

    let now = Instant::now();
    let default_rainbow_parens_enabled =
        shell_user_library(runtime).rainbow_parens_config().enabled;
    let requests = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .filter(|buffer| buffer.syntax_refresh_due(now))
            .map(|buffer| SyntaxRefreshWorkerRequest {
                buffer_id: buffer.id(),
                buffer_revision: buffer.text.revision(),
                path: buffer.path().map(Path::to_path_buf),
                buffer_language_id: buffer.language_id().map(str::to_owned),
                // Prefer visible-window highlighting so first paint lands fast, then expand on
                // demand as scrolling changes the requested window.
                syntax_window: buffer.worker_syntax_window(),
                rainbow_parens_enabled: buffer
                    .rainbow_parens_enabled(default_rainbow_parens_enabled),
                text: buffer.text.clone(),
            })
            .collect::<Vec<_>>()
    };

    if requests.is_empty() {
        return Ok(stats);
    }

    {
        let ui = shell_ui_mut(runtime)?;
        for request in requests {
            let buffer_id = request.buffer_id;
            let syntax_window = request.syntax_window;
            if ui.syntax_refresh_worker.send(request) {
                if let Some(buffer) = ui.buffer_mut(buffer_id) {
                    buffer.mark_syntax_refresh_requested(syntax_window);
                }
            } else if let Some(buffer) = ui.buffer_mut(buffer_id) {
                buffer.force_syntax_refresh();
            }
        }
    }

    Ok(stats)
}

fn refresh_pending_file_reloads(
    runtime: &mut EditorRuntime,
    _now: Instant,
    force: bool,
) -> Result<bool, String> {
    let mut changed = false;
    if force {
        let buffer_ids = {
            let ui = shell_ui(runtime)?;
            ui.buffers.iter().map(ShellBuffer::id).collect::<Vec<_>>()
        };
        for buffer_id in buffer_ids {
            let did_reload = {
                let ui = shell_ui_mut(runtime)?;
                let Some(buffer) = ui.buffer_mut(buffer_id) else {
                    continue;
                };
                buffer.reload_from_disk_if_changed(true)?
            };
            changed |= did_reload;
        }
        return Ok(changed);
    }

    let mut first_error = None;
    let watcher_errors = shell_ui(runtime)?.file_reload_worker.take_errors();
    if first_error.is_none() {
        first_error = watcher_errors.into_iter().next();
    }
    let changed_paths = shell_ui(runtime)?.file_reload_worker.take_changed_paths();
    if !changed_paths.is_empty() {
        let changed_paths = changed_paths.into_iter().collect::<HashSet<_>>();
        let ui = shell_ui_mut(runtime)?;
        for buffer in &mut ui.buffers {
            let Some(path) = shell_buffer_watch_path(buffer) else {
                continue;
            };
            if changed_paths.contains(&path) {
                buffer.mark_backing_file_reload_pending();
            }
        }
    }
    {
        let ui = shell_ui_mut(runtime)?;
        for buffer in &mut ui.buffers {
            if !buffer.is_pdf_buffer() || !buffer.backing_file_reload_pending {
                continue;
            }
            changed |= buffer.reload_from_disk_if_changed(false)?;
        }
    }
    let results = shell_ui(runtime)?.file_reload_worker.take_results();
    if !results.is_empty() {
        let ui = shell_ui_mut(runtime)?;
        for result in results {
            let Some(buffer) = ui.buffer_mut(result.buffer_id) else {
                continue;
            };
            buffer.finish_file_reload_request();
            let current_path = buffer.path().map(Path::to_path_buf);
            if buffer.text.revision() != result.buffer_revision
                || current_path.as_deref() != Some(result.path.as_path())
            {
                continue;
            }
            match result.outcome {
                Ok(FileReloadWorkerOutcome::Missing) => {}
                Ok(FileReloadWorkerOutcome::Unchanged { fingerprint }) => {
                    buffer.backing_file_fingerprint = Some(fingerprint);
                }
                Ok(FileReloadWorkerOutcome::Reloaded { fingerprint, text }) => {
                    changed |= buffer.apply_reloaded_file_buffer(fingerprint, text);
                }
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }
    }

    let requests = {
        let ui = shell_ui_mut(runtime)?;
        ui.buffers
            .iter_mut()
            .filter_map(ShellBuffer::file_reload_request)
            .collect::<Vec<_>>()
    };
    if !requests.is_empty() {
        let ui = shell_ui(runtime)?;
        for request in requests {
            ui.file_reload_worker.send(request);
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }

    Ok(changed)
}

fn refresh_pending_git(
    runtime: &mut EditorRuntime,
    now: Instant,
    typing_active: bool,
) -> Result<(), String> {
    refresh_pending_git_summary(runtime, now, typing_active)?;
    refresh_pending_git_fringe(runtime, now, typing_active)?;
    Ok(())
}

fn refresh_pending_lsp(runtime: &mut EditorRuntime, typing_active: bool) -> Result<bool, String> {
    schedule_pending_lsp_syncs(runtime, typing_active)?;
    let sync_results = {
        let ui = shell_ui_mut(runtime)?;
        ui.lsp_sync_worker.take_results()
    };
    for result in sync_results {
        if let Some(error) = result.error {
            record_runtime_error(
                runtime,
                "lsp.sync-worker",
                format!(
                    "failed to sync `{}` at revision {}: {error}",
                    result.path.display(),
                    result.revision
                ),
            );
        }
    }
    apply_pending_lsp_state(runtime)
}

fn schedule_pending_lsp_syncs(
    runtime: &mut EditorRuntime,
    typing_active: bool,
) -> Result<(), String> {
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        return Ok(());
    };

    let requests = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .map(|buffer| {
                if !buffer.lsp_enabled() {
                    return Ok(None);
                }
                let Some(path) = buffer.lsp_path().map(Path::to_path_buf) else {
                    return Ok(None);
                };
                if path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_none()
                {
                    return Ok(None);
                }
                let root = lsp_root_for_buffer(runtime, buffer)?;
                apply_sqls_workspace_settings_for_buffer(
                    runtime,
                    buffer.id(),
                    buffer,
                    &path,
                    root.as_deref(),
                    &lsp_client,
                )?;
                let revision = buffer.text.revision();
                if !lsp_client.needs_sync_in_workspace(&path, revision, root.as_deref())
                    || ui.lsp_sync_worker.has_request(&path, revision)
                {
                    return Ok(None);
                }
                Ok(Some(LspSyncWorkerRequest {
                    path: path.clone(),
                    revision,
                    text: buffer.text.snapshot(),
                    root,
                    lsp_client: lsp_client.clone(),
                    preferred_server_id: None,
                    edits: lsp_edits_since_last_sync(&lsp_client, &path, &buffer.text),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    };
    let now = Instant::now();
    let ui = shell_ui_mut(runtime)?;
    for request in requests {
        ui.lsp_sync_worker.schedule(request, typing_active);
    }
    ui.lsp_sync_worker.dispatch_due(now)?;
    Ok(())
}

fn apply_pending_lsp_state(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let now = Instant::now();
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        return Ok(false);
    };
    let has_log_buffers = runtime
        .services()
        .get::<LspLogBufferState>()
        .map(LspLogBufferState::has_buffers)
        .unwrap_or(false);
    let applied_log_revision = runtime
        .services()
        .get::<LspLogBufferState>()
        .map(|state| state.applied_revision)
        .unwrap_or(0);

    let diagnostics_generation = lsp_client.diagnostics_generation();
    let sessions_generation = lsp_client.sessions_generation();
    let last_diagnostics_generation = shell_ui(runtime)?.last_lsp_diagnostics_generation();
    let last_notification_revision = shell_ui(runtime)?.last_lsp_notification_revision();
    let last_label_key = shell_ui(runtime)?.last_attached_lsp_label_key().cloned();
    let diagnostics_changed = last_diagnostics_generation != Some(diagnostics_generation);

    let (
        diagnostic_updates,
        active_workspace_id,
        label_update,
        label_key,
        log_snapshot,
        notification_snapshot,
    ) = {
        let ui = shell_ui(runtime)?;
        let updates = if diagnostics_changed {
            let dirty = lsp_client.take_dirty_diagnostic_paths();
            ui.buffers
                .iter()
                .filter(|buffer| buffer.lsp_enabled())
                .filter_map(|buffer| {
                    let path = buffer.lsp_path()?;
                    if !dirty.is_empty() && !dirty.iter().any(|dirty_path| dirty_path == path) {
                        return None;
                    }
                    Some((buffer.id(), lsp_client.diagnostics_for_path(path)))
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let active_path = ui
            .active_buffer_id()
            .and_then(|buffer_id| ui.buffer(buffer_id))
            .and_then(ShellBuffer::lsp_path)
            .map(Path::to_path_buf);
        let label_key = (
            ui.active_workspace(),
            active_path.clone(),
            sessions_generation,
        );
        let label_update = (last_label_key.as_ref() != Some(&label_key)).then(|| {
            active_path
                .as_ref()
                .map(|path| lsp_client.session_labels_for_path(path))
                .filter(|labels| !labels.is_empty())
                .map(|labels| labels.join(", "))
        });
        (
            updates,
            ui.active_workspace(),
            label_update,
            label_key,
            has_log_buffers
                .then(|| lsp_client.log_snapshot_if_changed(applied_log_revision))
                .flatten(),
            lsp_client.notification_snapshot_if_changed(last_notification_revision),
        )
    };

    let mut changed = false;
    {
        let ui = shell_ui_mut(runtime)?;
        for (buffer_id, diagnostics) in diagnostic_updates {
            if let Some(buffer) = ui.buffer_mut(buffer_id) {
                changed |= buffer.set_lsp_diagnostics(diagnostics);
            }
        }
        if let Some(active_server_label) = label_update {
            changed |= ui.set_attached_lsp_server(active_workspace_id, active_server_label);
            ui.set_last_attached_lsp_label_key(label_key);
        }
        if diagnostics_changed {
            ui.set_last_lsp_diagnostics_generation(diagnostics_generation);
        }
    }
    if let Some(notification_snapshot) = notification_snapshot.as_ref() {
        changed |= apply_lsp_notifications(runtime, notification_snapshot, now)?;
    }
    if let Some(log_snapshot) = log_snapshot.as_ref() {
        changed |= refresh_lsp_log_buffers(runtime, log_snapshot)?;
    }
    Ok(changed)
}

fn notification_severity(level: LspNotificationLevel) -> NotificationSeverity {
    match level {
        LspNotificationLevel::Info => NotificationSeverity::Info,
        LspNotificationLevel::Success => NotificationSeverity::Success,
        LspNotificationLevel::Warning => NotificationSeverity::Warning,
        LspNotificationLevel::Error => NotificationSeverity::Error,
    }
}

fn apply_lsp_notifications(
    runtime: &mut EditorRuntime,
    snapshot: &LspNotificationSnapshot,
    now: Instant,
) -> Result<bool, String> {
    let mut changed = false;
    let last_seen = shell_ui(runtime)?.last_lsp_notification_revision();
    let workspace_id = shell_ui(runtime)?.active_workspace();
    for entry in snapshot.entries() {
        if entry.revision() <= last_seen {
            continue;
        }
        let notification = entry.notification();
        let progress = notification
            .progress()
            .map(|progress| NotificationProgress {
                percentage: progress
                    .percentage()
                    .and_then(|percentage| u8::try_from(percentage.min(u32::from(u8::MAX))).ok()),
            });
        let action = lsp_notification_action(notification);
        if let Some(NotificationAction::OpenBrowserPopup { url }) = action.as_ref() {
            open_browser_buffer_in_popup(runtime, Some(url))?;
        }
        changed |= shell_ui_mut(runtime)?.apply_notification(
            NotificationUpdate {
                key: notification.key().to_owned(),
                severity: notification_severity(notification.level()),
                title: notification.title().to_owned(),
                body_lines: lsp_notification_body_lines(notification),
                progress,
                active: notification.active(),
                action,
                workspace_id: Some(workspace_id),
            },
            now,
        );
    }
    shell_ui_mut(runtime)?.set_last_lsp_notification_revision(snapshot.revision());
    Ok(changed)
}

fn lsp_notification_body_lines(notification: &editor_lsp::LspNotification) -> Vec<String> {
    let mut lines = notification.body_lines().to_vec();
    match notification.action() {
        Some(LspNotificationAction::CopilotSignIn) => {
            lines.push("Click notification to sign in.".to_owned());
        }
        Some(LspNotificationAction::OpenBrowserPopup { .. }) => {
            lines.push("Click notification to reopen browser popup.".to_owned());
        }
        None => {}
    }
    lines
}

fn lsp_notification_action(
    notification: &editor_lsp::LspNotification,
) -> Option<NotificationAction> {
    match notification.action()? {
        LspNotificationAction::CopilotSignIn => Some(NotificationAction::CopilotSignIn {
            root: notification.root().map(Path::to_path_buf),
        }),
        LspNotificationAction::OpenBrowserPopup { url } => {
            Some(NotificationAction::OpenBrowserPopup { url: url.clone() })
        }
    }
}

fn refresh_lsp_log_buffers(
    runtime: &mut EditorRuntime,
    snapshot: &LspLogSnapshot,
) -> Result<bool, String> {
    let (workspace_buffers, applied_revision) = {
        let Some(state) = runtime.services().get::<LspLogBufferState>() else {
            return Ok(false);
        };
        (
            state
                .buffer_ids
                .iter()
                .map(|(workspace_id, buffers)| {
                    (
                        *workspace_id,
                        buffers
                            .iter()
                            .map(|(server_id, buffer_id)| (server_id.clone(), *buffer_id))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
            state.applied_revision,
        )
    };
    if snapshot.revision() == applied_revision {
        return Ok(false);
    }
    if workspace_buffers.is_empty() {
        if let Some(state) = runtime.services_mut().get_mut::<LspLogBufferState>() {
            state.applied_revision = snapshot.revision();
        }
        return Ok(false);
    }
    let had_buffers = !workspace_buffers.is_empty();
    {
        let ui = shell_ui_mut(runtime)?;
        for (_workspace_id, buffers) in &workspace_buffers {
            for (server_id, buffer_id) in buffers {
                let entries = lsp_log_entries_for_server(snapshot.entries(), server_id);
                if let Some(buffer) = ui.buffer_mut(*buffer_id) {
                    buffer.replace_with_lines_follow_output(lsp_log_buffer_lines(
                        server_id, &entries,
                    ));
                }
            }
        }
    }
    if let Some(state) = runtime.services_mut().get_mut::<LspLogBufferState>() {
        state.applied_revision = snapshot.revision();
    }
    Ok(had_buffers || !snapshot.entries().is_empty())
}

fn refresh_pending_git_fringe(
    runtime: &mut EditorRuntime,
    now: Instant,
    typing_active: bool,
) -> Result<(), String> {
    let buffer_ids = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .filter(|buffer| buffer.git_fringe_refresh_due(now, typing_active))
            .map(ShellBuffer::id)
            .collect::<Vec<_>>()
    };

    for buffer_id in buffer_ids {
        refresh_git_fringe(runtime, buffer_id)?;
    }

    Ok(())
}
