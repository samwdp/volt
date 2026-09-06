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
    }
    let user_library = shell_user_library(runtime);
    let overlay = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        hover_overlay_draft_for_buffer(
            buffer_id,
            buffer,
            &registry,
            lsp_client.as_ref(),
            lsp_context.as_ref(),
            &*user_library,
        )
    }
    .map(|draft| finalize_hover_overlay(runtime, draft));
    let ui = shell_ui_mut(runtime)?;
    if let Some(mut overlay) = overlay {
        overlay.focused = focused;
        ui.set_hover(overlay);
    } else {
        ui.close_hover();
    }
    Ok(())
}

fn accept_autocomplete(runtime: &mut EditorRuntime) -> Result<(), String> {
    let selected = {
        let ui = shell_ui(runtime)?;
        ui.autocomplete()
            .filter(|autocomplete| autocomplete.is_visible())
            .and_then(|autocomplete| autocomplete.selected().cloned())
    };
    let Some(selected) = selected else {
        return Ok(());
    };

    let buffer_id = active_shell_buffer_id(runtime)?;
    let ui = shell_ui_mut(runtime)?;
    let Some(buffer) = ui.buffer_mut(buffer_id) else {
        ui.close_autocomplete();
        return Ok(());
    };
    if buffer.is_read_only() || buffer.has_input_field() {
        ui.close_autocomplete();
        return Ok(());
    }
    let snapshot = buffer.text.snapshot();
    let Some(query) = autocomplete_query(&snapshot, true) else {
        ui.close_autocomplete();
        return Ok(());
    };
    let replace_range = selected.replace_range.unwrap_or(query.replace_range);
    let replacement =
        normalize_completion_replacement(&snapshot, replace_range, &selected.replacement);
    buffer.replace_range(replace_range, &replacement);
    buffer.mark_syntax_dirty();
    ui.close_autocomplete();
    Ok(())
}
