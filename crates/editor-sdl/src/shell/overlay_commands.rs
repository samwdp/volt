fn trigger_autocomplete(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let active_buffer_is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer_is_acp(&buffer.kind) && buffer.has_input_field()
    };
    if active_buffer_is_acp {
        return acp::acp_complete_slash(runtime);
    }

    let registry = runtime
        .services()
        .get::<AutocompleteRegistry>()
        .cloned()
        .ok_or_else(|| "autocomplete registry service missing".to_owned())?;
    let lsp_client = runtime.services().get::<Arc<LspClientManager>>().cloned();
    let request = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        if buffer.is_read_only() || buffer.has_input_field() {
            return Ok(());
        }
        let root = lsp_root_for_buffer(runtime, buffer)?;
        if let (Some(lsp_client), Some(path)) = (lsp_client.as_ref(), buffer.lsp_path())
            && let Err(error) = apply_sqls_workspace_settings_for_buffer(
                runtime,
                buffer_id,
                buffer,
                path,
                root.as_deref(),
                lsp_client,
            )
        {
            return Err(error);
        }
        let token_map_key = ui.autocomplete_worker.token_map_key();
        autocomplete_request_for_buffer(
            runtime, buffer_id, buffer, root, &registry, lsp_client, true,
        )
        .map(|mut request| {
            attach_token_count_edits(&mut request, &buffer.text, token_map_key);
            request
        })
    };
    let Some(request) = request else {
        shell_ui_mut(runtime)?.close_autocomplete();
        return Ok(());
    };
    let overlay =
        AutocompleteOverlay::new(buffer_id, request.buffer_revision, request.query.clone());
    let ui = shell_ui_mut(runtime)?;
    ui.set_autocomplete(overlay);
    ui.autocomplete_worker.schedule(request);
    Ok(())
}

fn trigger_hover_toggle(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let same_anchor = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        if buffer.has_input_field() {
            return Ok(());
        }
        let cursor = buffer.cursor_point();
        ui.hover()
            .filter(|hover| hover.buffer_id == buffer_id && hover.anchor == cursor)
            .is_some()
    };
    if same_anchor {
        shell_ui_mut(runtime)?.close_hover();
        return Ok(());
    }
    show_hover_overlay(runtime, false)
}

fn trigger_hover_focus(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let same_anchor_focus = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        if buffer.has_input_field() {
            return Ok(());
        }
        let cursor = buffer.cursor_point();
        ui.hover()
            .filter(|hover| hover.buffer_id == buffer_id && hover.anchor == cursor)
            .map(|hover| hover.focused)
    };
    match same_anchor_focus {
        Some(true) => return Ok(()),
        Some(false) => {
            if let Some(hover) = shell_ui_mut(runtime)?.hover_mut() {
                hover.focused = true;
            }
            return Ok(());
        }
        None => {}
    }

    show_hover_overlay(runtime, true)
}

fn cycle_hover_provider(runtime: &mut EditorRuntime, next: bool) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let Some(hover) = shell_ui_mut(runtime)?.hover_mut() else {
        return Ok(());
    };
    if next {
        hover.select_next_provider();
    } else {
        hover.select_previous_provider();
    }
    Ok(())
}

fn show_hover_overlay(runtime: &mut EditorRuntime, focused: bool) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let registry = runtime
        .services()
        .get::<HoverRegistry>()
        .cloned()
        .ok_or_else(|| "hover registry service missing".to_owned())?;
    let lsp_client = runtime.services().get::<Arc<LspClientManager>>().cloned();
    let lsp_context = active_lsp_buffer_context(runtime).ok();
    if let (Some(lsp_client), Some(lsp_context)) = (lsp_client.as_ref(), lsp_context.as_ref()) {
        apply_sqls_workspace_settings_for_active_buffer_context(runtime, lsp_client, lsp_context)?;
        schedule_lsp_ui_hover(runtime, focused)?;
    }
    let user_library = shell_user_library(runtime);
    let overlay = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        hover_overlay_draft_for_buffer(buffer_id, buffer, &registry, None, &*user_library)
    }
    .map(|draft| finalize_hover_overlay(runtime, draft));
    let ui = shell_ui_mut(runtime)?;
    if let Some(mut overlay) = overlay {
        overlay.focused = focused;
        ui.set_hover(overlay);
    }
    Ok(())
}

fn schedule_lsp_ui_hover(runtime: &mut EditorRuntime, focused: bool) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        return Ok(());
    };
    let (cursor, signature_point, edits) = {
        let buffer = shell_buffer(runtime, context.buffer_id)?;
        (
            buffer.cursor_point(),
            hover_signature_request_point(buffer),
            None,
        )
    };
    let ui = shell_ui_mut(runtime)?;
    ui.lsp_ui_worker.schedule(LspUiWorkerRequest {
        buffer_id: context.buffer_id,
        buffer_revision: context.revision,
        path: context.path,
        text: context.text,
        root: context.root,
        cursor,
        kind: LspUiRequestKind::Hover {
            signature_point,
            focused,
        },
        lsp_client,
        edits,
    });
    Ok(())
}

fn apply_lsp_ui_worker_results(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let results = {
        let ui = shell_ui_mut(runtime)?;
        ui.lsp_ui_worker.take_results()
    };
    if results.is_empty() {
        return Ok(false);
    }
    let mut changed = false;
    for result in results {
        if let Some(error) = result.error {
            record_runtime_error(runtime, "lsp.ui-worker", error);
        }
        if let Some((payload, focused)) = result.hover {
            changed |= apply_lsp_ui_hover_result(runtime, result.buffer_id, result.cursor, payload, focused)?;
        }
        if let Some((title, locations)) = result.locations {
            open_lsp_locations(runtime, &title, locations)?;
            changed = true;
        }
        if let Some(format) = result.format {
            changed |= apply_lsp_ui_format_result(
                runtime,
                result.buffer_id,
                result.buffer_revision,
                format,
            )?;
        }
        if let Some(code_actions) = result.code_actions {
            changed |= apply_lsp_ui_code_actions_result(runtime, result.buffer_id, code_actions)?;
        }
    }
    Ok(changed)
}

fn apply_lsp_ui_format_result(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    buffer_revision: u64,
    payload: LspUiFormatPayload,
) -> Result<bool, String> {
    let revision_matches = shell_buffer(runtime, buffer_id)
        .ok()
        .is_some_and(|buffer| buffer.text.revision() == buffer_revision);
    match (
        revision_matches,
        payload.edits.as_ref(),
        payload.then_save,
    ) {
        (true, Some(edits), _) => {
            let buffer = shell_buffer_mut(runtime, buffer_id)?;
            apply_lsp_text_edits(buffer, edits);
            buffer.set_cursor(payload.original_cursor);
            finish_format_command(runtime)?;
        }
        (true, None, false) => {
            if let Ok(formatter) = formatter_for_path(runtime, &payload.path) {
                if let Some(range) = payload.range {
                    format_range_with_formatter(
                        runtime,
                        &formatter,
                        range,
                        payload.extension.as_deref(),
                        payload.cwd.as_deref(),
                    )?;
                    let buffer = shell_buffer_mut(runtime, buffer_id)?;
                    buffer.set_cursor(payload.original_cursor);
                    finish_format_command(runtime)?;
                } else {
                    format_buffer_entire_with_formatter(
                        runtime,
                        buffer_id,
                        &formatter,
                        payload.extension.as_deref(),
                        payload.cwd.as_deref(),
                        payload.original_cursor,
                    )?;
                    finish_format_command(runtime)?;
                }
            }
        }
        _ => {}
    }
    if payload.then_save {
        save_buffer_inner(runtime, payload.workspace_id, buffer_id, &payload.path)?;
    }
    Ok(true)
}

fn apply_lsp_ui_code_actions_result(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    payload: LspUiCodeActionsPayload,
) -> Result<bool, String> {
    sync_lsp_buffer_state(runtime, payload.workspace_id, buffer_id, &payload.labels)?;
    if payload.actions.is_empty() {
        let picker = lsp_code_actions_status_picker_overlay(
            "No code actions available",
            "The active cursor position does not expose any LSP code actions.",
            Some(payload.path.display().to_string()),
        );
        shell_ui_mut(runtime)?.set_picker(picker);
        return Ok(true);
    }
    let picker = lsp_code_actions_picker_overlay(
        payload.workspace_id,
        buffer_id,
        &payload.path,
        &payload.actions,
    );
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(true)
}

fn apply_lsp_ui_hover_result(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    cursor: TextPoint,
    payload: LspUiHoverPayload,
    focused: bool,
) -> Result<bool, String> {
    let still_valid = {
        let ui = shell_ui(runtime)?;
        ui.buffer(buffer_id)
            .is_some_and(|buffer| buffer.cursor_point() == cursor)
    };
    if !still_valid {
        return Ok(false);
    }
    let registry = runtime
        .services()
        .get::<HoverRegistry>()
        .cloned()
        .ok_or_else(|| "hover registry service missing".to_owned())?;
    let user_library = shell_user_library(runtime);
    let overlay = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(false);
        };
        hover_overlay_draft_for_buffer(
            buffer_id,
            buffer,
            &registry,
            Some(&payload),
            &*user_library,
        )
    }
    .map(|draft| finalize_hover_overlay(runtime, draft));
    let ui = shell_ui_mut(runtime)?;
    if let Some(mut overlay) = overlay {
        overlay.focused = focused;
        ui.set_hover(overlay);
        Ok(true)
    } else {
        Ok(false)
    }
}

fn accept_autocomplete(runtime: &mut EditorRuntime) -> Result<(), String> {
    let selected = {
        let ui = shell_ui(runtime)?;
        ui.autocomplete()
            .filter(|autocomplete| autocomplete.is_visible())
            .and_then(|autocomplete| {
                autocomplete.selected().map(|entry| {
                    (
                        entry.replacement.clone(),
                        entry.replace_range,
                        autocomplete.query.clone(),
                    )
                })
            })
    };
    let Some((replacement, selected_range, query)) = selected else {
        return Ok(());
    };

    let buffer_id = active_shell_buffer_id(runtime)?;
    let ui = shell_ui_mut(runtime)?;
    ui.close_autocomplete();
    let Some(buffer) = ui.buffer_mut(buffer_id) else {
        return Ok(());
    };
    if buffer.is_read_only() || buffer.has_input_field() {
        return Ok(());
    }
    let snapshot = buffer.text.snapshot();
    let replace_range = selected_range.unwrap_or_else(|| {
        autocomplete_query(&snapshot, true)
            .map(|live| live.replace_range)
            .unwrap_or(query.replace_range)
    });
    let replacement = normalize_completion_replacement(&snapshot, replace_range, &replacement);
    buffer.replace_range(replace_range, &replacement);
    buffer.mark_syntax_dirty();
    Ok(())
}
