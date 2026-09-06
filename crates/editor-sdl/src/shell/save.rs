fn save_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
) -> Result<(), String> {
    let path = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let buffer = workspace
            .buffer(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        buffer.path().map(Path::to_path_buf)
    };

    let supports_text_file_actions = shell_buffer(runtime, buffer_id)?.supports_text_file_actions();
    let is_pdf_buffer = shell_buffer(runtime, buffer_id)?.is_pdf_buffer();
    if !supports_text_file_actions && !is_pdf_buffer {
        return Ok(());
    }

    let path = path.ok_or_else(|| "buffer.save requires a file path".to_owned())?;
    if is_pdf_buffer {
        return save_buffer_inner(runtime, workspace_id, buffer_id, &path);
    }
    let language_id = language_id_for_path(runtime, &path).ok();
    if theme_lang_format_on_save(
        runtime.services().get::<ThemeRegistry>(),
        language_id.as_deref(),
    ) && let Err(error) = format_buffer_on_save(runtime, workspace_id, buffer_id, &path)
    {
        record_runtime_error(
            runtime,
            "buffer.save.format-on-save",
            format!(
                "format-on-save failed for `{}`: {error}; saving without formatting",
                path.display()
            ),
        );
    }
    normalize_mark_list_buffer_before_save(runtime, buffer_id, &path)?;
    save_buffer_inner(runtime, workspace_id, buffer_id, &path)?;
    reload_mark_list_after_save(runtime, &path)
}

fn save_buffer_inner(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
) -> Result<(), String> {
    runtime
        .emit_hook(
            builtins::BEFORE_SAVE,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_buffer(buffer_id)
                .with_detail(path.display().to_string()),
        )
        .map_err(|error| error.to_string())?;

    {
        let buffer = shell_ui_mut(runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))?;
        buffer
            .save_to_path(path)
            .map_err(|error| format!("failed to save `{}`: {error}", path.display()))?;
    }

    runtime
        .emit_hook(
            builtins::AFTER_SAVE,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_buffer(buffer_id)
                .with_detail(path.display().to_string()),
        )
        .map_err(|error| error.to_string())?;
    if let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>() {
        lsp_client
            .save_buffer(path)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn close_buffer_with_prompt(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let (is_dirty, name) = {
        let ui = shell_ui(runtime)?;
        let buffer = ui
            .buffer(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))?;
        (buffer.is_dirty(), buffer.display_name().to_owned())
    };
    if is_dirty {
        let picker = picker::buffer_close_confirm_overlay(buffer_id, &name);
        shell_ui_mut(runtime)?.set_picker(picker);
        return Ok(());
    }
    close_buffer_immediate(runtime, buffer_id)
}

fn close_buffer_save(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    save_buffer(runtime, workspace_id, buffer_id)?;
    close_buffer_immediate(runtime, buffer_id)
}

fn close_buffer_discard(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    close_buffer_immediate(runtime, buffer_id)
}

fn close_buffer_immediate(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let path = shell_ui(runtime)?
        .buffer(buffer_id)
        .and_then(|buffer| buffer.lsp_path().map(Path::to_path_buf));
    let lsp_client = runtime.services().get::<Arc<LspClientManager>>().cloned();
    if let (Some(path), Some(lsp_client)) = (path, lsp_client) {
        cancel_lsp_sync_for_path(runtime, &path)?;
        lsp_client
            .close_buffer(&path)
            .map_err(|error| error.to_string())?;
    }
    acp::close_acp_buffer(runtime, buffer_id)?;
    close_terminal_buffer(runtime, buffer_id)?;
    if let Some(db) = runtime.services_mut().get_mut::<DbService>() {
        db.detach_buffer(buffer_id.get());
    }
    runtime
        .model_mut()
        .close_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.remove_buffer(buffer_id);
    sync_active_buffer(runtime)?;
    Ok(())
}

fn close_popup_buffer_and_restore_focus(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    if shell_ui(runtime)?.buffer(buffer_id).is_none() {
        return Ok(());
    }
    close_buffer_immediate(runtime, buffer_id)?;
    if let Some(popup) = active_runtime_popup(runtime)? {
        let popup_focus_allowed = {
            let ui = shell_ui(runtime)?;
            ui.popup_focus_allowed(&popup)
        };
        {
            let ui = shell_ui_mut(runtime)?;
            ui.set_popup_buffer(popup.active_buffer);
            if !popup_focus_allowed {
                ui.set_popup_focus(false);
            }
        }
        ensure_shell_buffer(runtime, popup.active_buffer)?;
    } else {
        let ui = shell_ui_mut(runtime)?;
        ui.set_popup_focus(false);
        ui.clear_popup_buffer();
    }
    Ok(())
}

fn close_lsp_buffers_for_workspace(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let lsp_client = runtime.services().get::<Arc<LspClientManager>>().cloned();
    let Some(lsp_client) = lsp_client else {
        return Ok(());
    };
    let (paths, workspace_root) = {
        let ui = shell_ui(runtime)?;
        let paths = ui
            .workspace_views
            .get(&workspace_id)
            .map(|view| {
                view.buffer_ids
                    .iter()
                    .filter_map(|buffer_id| {
                        ui.buffer(*buffer_id)
                            .and_then(|buffer| buffer.lsp_path().map(Path::to_path_buf))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let workspace_root = runtime
            .model()
            .workspace(workspace_id)
            .ok()
            .and_then(|workspace| workspace.root().map(Path::to_path_buf));
        (paths, workspace_root)
    };
    for path in paths {
        cancel_lsp_sync_for_path(runtime, &path)?;
        lsp_client
            .close_buffer(&path)
            .map_err(|error| error.to_string())?;
    }
    lsp_client
        .stop_sessions_for_root(workspace_root.as_deref())
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn save_workspace(runtime: &mut EditorRuntime, workspace_id: WorkspaceId) -> Result<(), String> {
    let buffer_ids = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        workspace.buffers().map(Buffer::id).collect::<Vec<_>>()
    };

    for buffer_id in buffer_ids {
        let path = {
            let workspace = runtime
                .model()
                .workspace(workspace_id)
                .map_err(|error| error.to_string())?;
            let buffer = workspace
                .buffer(buffer_id)
                .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
            buffer.path().map(Path::to_path_buf)
        };

        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer.supports_text_file_actions() && !buffer.is_pdf_buffer() {
            continue;
        }

        let is_dirty = {
            let ui = shell_ui(runtime)?;
            let buffer = ui
                .buffer(buffer_id)
                .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))?;
            buffer.is_dirty()
        };

        if !is_dirty {
            continue;
        }

        let path =
            path.ok_or_else(|| format!("text-editable buffer `{buffer_id}` is missing a path"))?;
        save_buffer(runtime, workspace_id, buffer_id)
            .map_err(|error| format!("failed to save `{}`: {error}", path.display()))?;
    }

    Ok(())
}

fn format_workspace(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (path, extension, original_cursor, selection, supports_text_actions) = {
        let ui = shell_ui(runtime)?;
        let buffer = ui
            .buffer(buffer_id)
            .ok_or_else(|| "active buffer is missing".to_owned())?;
        let path = buffer
            .path()
            .ok_or_else(|| "active buffer does not have a file path".to_owned())?;
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_owned);
        let original_cursor = buffer.cursor_point();
        let selection = if ui.input_mode() == InputMode::Visual {
            let anchor = ui
                .vim()
                .visual_anchor
                .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
            let kind = ui.vim().visual_kind;
            let selection = visual_selection(buffer, anchor, kind)
                .ok_or_else(|| "visual selection is empty".to_owned())?;
            Some((selection, anchor, original_cursor, kind))
        } else {
            None
        };
        (
            path.to_path_buf(),
            extension,
            original_cursor,
            selection,
            buffer.supports_text_file_actions(),
        )
    };

    if !supports_text_actions {
        return Err("workspace.format only supports file buffers".to_owned());
    }

    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .or_else(|| active_workspace_root(runtime).ok().flatten());

    start_change_recording(runtime)?;

    if let Some((selection, anchor, head, kind)) = selection {
        store_last_visual_selection(runtime, anchor, head, kind)?;
        if try_format_visual_selection_with_lsp(
            runtime,
            workspace_id,
            buffer_id,
            &path,
            selection,
            original_cursor,
        )? {
            finish_format_command(runtime)?;
            return Ok(());
        }
        let formatter = formatter_for_path(runtime, &path)?;
        format_visual_selection_with_formatter(
            runtime,
            &formatter,
            selection,
            extension.as_deref(),
            cwd.as_deref(),
            original_cursor,
        )?;
    } else {
        if try_format_buffer_entire_with_lsp(
            runtime,
            workspace_id,
            buffer_id,
            &path,
            original_cursor,
        )? {
            finish_format_command(runtime)?;
            return Ok(());
        }
        let formatter = formatter_for_path(runtime, &path)?;
        format_entire_buffer_with_formatter(
            runtime,
            &formatter,
            extension.as_deref(),
            cwd.as_deref(),
            original_cursor,
        )?;
    }

    Ok(())
}

fn format_buffer_on_save(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
) -> Result<(), String> {
    let original_cursor = shell_buffer(runtime, buffer_id)?.cursor_point();
    if try_format_buffer_entire_with_lsp(runtime, workspace_id, buffer_id, path, original_cursor)? {
        return Ok(());
    }

    let formatter = formatter_for_path(runtime, path)?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_owned);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .or_else(|| active_workspace_root(runtime).ok().flatten());
    format_buffer_entire_with_formatter(
        runtime,
        buffer_id,
        &formatter,
        extension.as_deref(),
        cwd.as_deref(),
        original_cursor,
    )
}

fn formatter_for_path(runtime: &EditorRuntime, path: &Path) -> Result<FormatterSpec, String> {
    let language_id = language_id_for_path(runtime, path)?;
    let formatter = formatter_registry(runtime)?
        .formatter_for_language(&language_id)
        .ok_or_else(|| format!("no formatter registered for language `{language_id}`"))?;
    Ok(formatter.clone())
}

fn language_id_for_path(runtime: &EditorRuntime, path: &Path) -> Result<String, String> {
    let syntax = runtime
        .services()
        .get::<SyntaxRegistry>()
        .ok_or_else(|| "syntax registry service missing".to_owned())?;
    let language = syntax
        .language_for_path(path)
        .ok_or_else(|| format!("no syntax language registered for `{}`", path.display()))?;
    Ok(language.id().to_owned())
}

fn finish_format_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.enter_normal_mode();
    schedule_finish_change(runtime)
}

fn format_entire_buffer_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    extension: Option<&str>,
    cwd: Option<&Path>,
    original_cursor: TextPoint,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    format_buffer_entire_with_formatter(
        runtime,
        buffer_id,
        formatter,
        extension,
        cwd,
        original_cursor,
    )?;
    shell_ui_mut(runtime)?.enter_normal_mode();
    schedule_finish_change(runtime)?;
    Ok(())
}

fn format_buffer_entire_with_formatter(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    formatter: &FormatterSpec,
    extension: Option<&str>,
    cwd: Option<&Path>,
    original_cursor: TextPoint,
) -> Result<(), String> {
    let input = { shell_buffer(runtime, buffer_id)?.text.text() };
    let formatted = format_text_with_formatter(runtime, formatter, &input, extension, cwd)?;
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    if formatted != input {
        let end = buffer.text.point_from_char_index(buffer.text.char_count());
        buffer.replace_range(TextRange::new(TextPoint::default(), end), &formatted);
        buffer.mark_syntax_dirty();
    }
    buffer.set_cursor(original_cursor);
    Ok(())
}

fn format_visual_selection_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    selection: VisualSelection,
    extension: Option<&str>,
    cwd: Option<&Path>,
    original_cursor: TextPoint,
) -> Result<(), String> {
    match selection {
        VisualSelection::Range(range) => {
            format_range_with_formatter(runtime, formatter, range, extension, cwd)?;
        }
        VisualSelection::Block(block) => {
            format_block_with_formatter(runtime, formatter, block, extension, cwd)?;
        }
    }
    let buffer = active_shell_buffer_mut(runtime)?;
    buffer.set_cursor(original_cursor);
    buffer.mark_syntax_dirty();
    shell_ui_mut(runtime)?.enter_normal_mode();
    schedule_finish_change(runtime)?;
    Ok(())
}

fn try_format_buffer_entire_with_lsp(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    original_cursor: TextPoint,
) -> Result<bool, String> {
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        return Ok(false);
    };
    let context = lsp_buffer_context(runtime, workspace_id, buffer_id)?;
    let language_id = language_id_for_path(runtime, path).ok();
    let options = lsp_formatting_options(runtime, language_id.as_deref());
    cancel_lsp_sync_for_path(runtime, &context.path)?;
    apply_sqls_workspace_settings_for_active_buffer_context(runtime, &lsp_client, &context)?;
    let (labels, edits) = {
        if !lsp_client.supports_path_in_workspace(&context.path, context.root.as_deref()) {
            return Ok(false);
        }
        let labels = lsp_client
            .sync_buffer(
                &context.path,
                &context.text,
                context.revision,
                context.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        let edits = lsp_client
            .formatting(&context.path, options)
            .map_err(|error| error.to_string())?;
        (labels, edits)
    };
    sync_lsp_buffer_state(runtime, workspace_id, buffer_id, &labels)?;
    let Some(edits) = edits else {
        return Ok(false);
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    apply_lsp_text_edits(buffer, &edits);
    buffer.set_cursor(original_cursor);
    Ok(true)
}

fn try_format_visual_selection_with_lsp(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    selection: VisualSelection,
    original_cursor: TextPoint,
) -> Result<bool, String> {
    let VisualSelection::Range(range) = selection else {
        return Ok(false);
    };
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        return Ok(false);
    };
    let context = lsp_buffer_context(runtime, workspace_id, buffer_id)?;
    let language_id = language_id_for_path(runtime, path).ok();
    let options = lsp_formatting_options(runtime, language_id.as_deref());
    cancel_lsp_sync_for_path(runtime, &context.path)?;
    apply_sqls_workspace_settings_for_active_buffer_context(runtime, &lsp_client, &context)?;
    let (labels, edits) = {
        if !lsp_client.supports_path_in_workspace(&context.path, context.root.as_deref()) {
            return Ok(false);
        }
        let labels = lsp_client
            .sync_buffer(
                &context.path,
                &context.text,
                context.revision,
                context.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        let edits = lsp_client
            .range_formatting(&context.path, range, options)
            .map_err(|error| error.to_string())?;
        (labels, edits)
    };
    sync_lsp_buffer_state(runtime, workspace_id, buffer_id, &labels)?;
    let Some(edits) = edits else {
        return Ok(false);
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    apply_lsp_text_edits(buffer, &edits);
    buffer.set_cursor(original_cursor);
    Ok(true)
}

fn sync_lsp_buffer_state(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    labels: &[String],
) -> Result<(), String> {
    let active_buffer_id = active_shell_buffer_id(runtime).ok();
    let ui = shell_ui_mut(runtime)?;
    if let Some(buffer) = ui.buffer_mut(buffer_id) {
        buffer.set_lsp_enabled(!labels.is_empty());
    }
    if active_buffer_id == Some(buffer_id) {
        ui.set_attached_lsp_server(
            workspace_id,
            (!labels.is_empty()).then(|| labels.join(", ")),
        );
    }
    Ok(())
}

fn apply_lsp_text_edits(buffer: &mut ShellBuffer, edits: &[LspTextEdit]) {
    let mut ordered = edits.to_vec();
    ordered.sort_by(|left, right| {
        right
            .range()
            .start()
            .cmp(&left.range().start())
            .then_with(|| right.range().end().cmp(&left.range().end()))
    });
    for edit in ordered {
        buffer.replace_range(edit.range(), edit.new_text());
    }
    if !edits.is_empty() {
        buffer.mark_syntax_dirty();
    }
}

fn format_range_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    range: TextRange,
    extension: Option<&str>,
    cwd: Option<&Path>,
) -> Result<(), String> {
    let input = {
        let buffer = active_shell_buffer_mut(runtime)?;
        buffer.slice(range)
    };
    let formatted = format_text_with_formatter(runtime, formatter, &input, extension, cwd)?;
    active_shell_buffer_mut(runtime)?.replace_range(range, &formatted);
    Ok(())
}

fn format_block_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    selection: BlockSelection,
    extension: Option<&str>,
    cwd: Option<&Path>,
) -> Result<(), String> {
    let (ranges, snippets) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let ranges = block_selection_ranges(buffer, selection);
        let snippets = ranges
            .iter()
            .map(|range| buffer.slice(*range))
            .collect::<Vec<_>>();
        (ranges, snippets)
    };

    if ranges.is_empty() {
        return Ok(());
    }

    let mut replacements = Vec::with_capacity(snippets.len());
    for snippet in snippets {
        let formatted = format_text_with_formatter(runtime, formatter, &snippet, extension, cwd)?;
        let formatted = normalize_block_output(&formatted)?;
        replacements.push(formatted);
    }

    let buffer = active_shell_buffer_mut(runtime)?;
    for index in (0..ranges.len()).rev() {
        buffer.replace_range(ranges[index], &replacements[index]);
    }

    Ok(())
}

fn normalize_block_output(formatted: &str) -> Result<String, String> {
    let trimmed = formatted.trim_end_matches(['\n', '\r']);
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err("formatter output spans multiple lines for a block selection".to_owned());
    }
    Ok(trimmed.to_owned())
}

fn format_text_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    input: &str,
    extension: Option<&str>,
    cwd: Option<&Path>,
) -> Result<String, String> {
    let temp_path = formatter_temp_path(extension);
    fs::write(&temp_path, input).map_err(|error| {
        format!(
            "failed to write formatter input `{}`: {error}",
            temp_path.display()
        )
    })?;

    let mut args = formatter.args.clone();
    args.push(temp_path.to_string_lossy().into_owned());
    let mut spec = JobSpec::command(
        format!("format-{}", formatter.language_id),
        formatter.program.clone(),
        args,
    );
    if let Some(cwd) = cwd {
        spec = spec.with_cwd(cwd.to_path_buf());
    }

    let manager = runtime
        .services()
        .get::<Mutex<JobManager>>()
        .ok_or_else(|| "job manager service missing".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "job manager lock poisoned".to_owned())?;
    let handle = manager.spawn(spec).map_err(|error| error.to_string())?;
    drop(manager);
    let result = handle.wait().map_err(|error| error.to_string())?;

    if !result.succeeded() {
        cleanup_formatter_temp(&temp_path);
        return Err(format!(
            "formatter `{}` failed: {}",
            formatter.program,
            result.transcript()
        ));
    }

    let formatted = fs::read_to_string(&temp_path).map_err(|error| {
        format!(
            "failed to read formatter output `{}`: {error}",
            temp_path.display()
        )
    })?;
    cleanup_formatter_temp(&temp_path);
    Ok(formatted)
}

fn formatter_temp_path(extension: Option<&str>) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    let mut filename = format!("volt-format-{}-{unique}", std::process::id());
    if let Some(extension) = extension.filter(|extension| !extension.is_empty()) {
        filename.push('.');
        filename.push_str(extension);
    }
    std::env::temp_dir().join(filename)
}

fn cleanup_formatter_temp(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        eprintln!(
            "failed to remove formatter temp file `{}`: {error}",
            path.display()
        );
    }
}
