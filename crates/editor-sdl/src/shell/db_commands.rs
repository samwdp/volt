fn emit_workspace_format(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    runtime
        .emit_hook(
            HOOK_WORKSPACE_FORMAT,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_buffer(buffer_id),
        )
        .map_err(|error| error.to_string())
}

fn submit_input_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (prompt, text, kind, name) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let Some(input) = buffer.input_field() else {
            return Ok(());
        };
        (
            input.prompt().to_owned(),
            input.text().to_owned(),
            buffer.kind.clone(),
            buffer.display_name().to_owned(),
        )
    };
    if text.trim().is_empty() {
        return Ok(());
    }
    if buffer_is_acp(&kind) {
        return acp::submit_acp_prompt(runtime, buffer_id, &prompt, &text);
    }
    if buffer_is_browser(&kind) {
        return browser::submit_browser_input(runtime);
    }
    if buffer_is_db_connect(&kind) {
        return submit_db_connect_prompt(runtime, buffer_id, &text);
    }
    if buffer_is_command_output(&kind, &name) {
        return run_shell_command_in_buffer(runtime, buffer_id, &text);
    }
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!("{prompt}{text}")]);
        buffer.clear_input();
    }
    Ok(())
}

fn db_service(runtime: &EditorRuntime) -> Result<&DbService, String> {
    runtime
        .services()
        .get::<DbService>()
        .ok_or_else(|| "database service is missing".to_owned())
}

fn db_service_mut(runtime: &mut EditorRuntime) -> Result<&mut DbService, String> {
    runtime
        .services_mut()
        .get_mut::<DbService>()
        .ok_or_else(|| "database service is missing".to_owned())
}

fn open_or_focus_workspace_plugin_buffer(
    runtime: &mut EditorRuntime,
    name: &str,
    kind: &'static str,
) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_kind = BufferKind::Plugin(kind.to_owned());
    let buffer_id = if let Some(existing) =
        find_workspace_named_buffer(runtime, workspace_id, name, &buffer_kind)?
    {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        existing
    } else {
        runtime
            .model_mut()
            .create_buffer(workspace_id, name, buffer_kind.clone(), None)
            .map_err(|error| error.to_string())?
    };
    ensure_shell_buffer(runtime, buffer_id)?;
    shell_ui_mut(runtime)?.focus_buffer_in_active_pane(buffer_id);
    Ok(buffer_id)
}

fn open_or_focus_popup_plugin_buffer(
    runtime: &mut EditorRuntime,
    name: &str,
    kind: &'static str,
    title: &str,
) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_kind = BufferKind::Plugin(kind.to_owned());
    let buffer_id = if let Some(existing) =
        find_workspace_named_buffer(runtime, workspace_id, name, &buffer_kind)?
    {
        existing
    } else {
        runtime
            .model_mut()
            .create_popup_buffer(workspace_id, name, buffer_kind.clone(), None)
            .map_err(|error| error.to_string())?
    };
    runtime
        .model_mut()
        .open_popup_buffer(workspace_id, title, buffer_id)
        .map_err(|error| error.to_string())?;
    ensure_shell_buffer(runtime, buffer_id)?;
    {
        let ui = shell_ui_mut(runtime)?;
        ui.set_popup_focus(false);
        ui.set_popup_buffer(buffer_id);
    }
    Ok(buffer_id)
}

fn apply_db_browser_view(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    view: DbBrowserBufferView,
) -> Result<(), String> {
    apply_db_browser_view_to_section(runtime, buffer_id, 0, view)
}

fn apply_db_browser_view_to_section(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    section_index: usize,
    view: DbBrowserBufferView,
) -> Result<(), String> {
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    let syntax_lines = view
        .lines
        .iter()
        .zip(view.kinds_by_line.iter())
        .enumerate()
        .filter_map(|(index, (line, kind))| {
            let spans = db_browser_line_spans(line, *kind);
            (!spans.is_empty()).then_some((index, spans))
        })
        .collect();
    if section_index == 0 {
        buffer.replace_with_lines_preserve_view(view.lines);
        buffer.set_indexed_syntax_lines(Some(syntax_lines), None);
    } else if let Some(pane) = buffer
        .plugin_section_state
        .as_mut()
        .and_then(|state| state.attached_section_mut(section_index))
    {
        pane.replace_lines(view.lines, false);
    }
    Ok(())
}

fn refresh_db_browser_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let user_library = shell_user_library(runtime);
    let view = db_service_mut(runtime)?
        .rerender_browser_buffer_with(buffer_id.get(), &|context| {
            user_library.db_browser_items(context)
        })?;
    apply_db_browser_view(runtime, buffer_id, view)
}

fn refresh_all_db_browser_buffers(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_ids = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .filter_map(|buffer| buffer_is_db_browser(&buffer.kind).then_some(buffer.id()))
            .collect::<Vec<_>>()
    };
    for buffer_id in buffer_ids {
        let kind = shell_buffer(runtime, buffer_id)?.kind.clone();
        if buffer_is_db_dashboard(&kind) || buffer_is_db_sidebar(&kind) {
            let _ = refresh_db_layout_browsers(runtime, buffer_id);
        } else {
            let _ = refresh_db_browser_buffer(runtime, buffer_id);
        }
    }
    Ok(())
}

fn refresh_db_layout_browsers(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let sections = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let Some(state) = buffer.plugin_sections() else {
            return Ok(());
        };
        (0..state.section_count())
            .filter_map(|index| {
                state
                    .browser_kind_for_section(index)
                    .map(|kind| (index, state.section_title(index).to_owned(), kind))
            })
            .collect::<Vec<_>>()
    };
    let user_library = shell_user_library(runtime);
    for (index, name, kind) in sections {
        let view = match kind {
            editor_plugin_api::DbBrowserKind::Connections => db_service_mut(runtime)?
                .render_connections_section_with(buffer_id.get(), &name, &|context| {
                    user_library.db_browser_items(context)
                })?,
            editor_plugin_api::DbBrowserKind::Schema => db_service_mut(runtime)?
                .render_schema_section_with(buffer_id.get(), &name, None, &|context| {
                    user_library.db_browser_items(context)
                })?,
            editor_plugin_api::DbBrowserKind::History
            | editor_plugin_api::DbBrowserKind::Snippets => {
                continue;
            }
        };
        apply_db_browser_view_to_section(runtime, buffer_id, index, view)?;
    }
    Ok(())
}

fn open_db_connect_prompt(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = open_or_focus_popup_plugin_buffer(
        runtime,
        DB_CONNECT_BUFFER_NAME,
        DB_CONNECT_KIND,
        "DB Connect",
    )?;
    db_service_mut(runtime)?.attach_prompt_buffer(buffer_id.get());
    {
        let ui = shell_ui_mut(runtime)?;
        ui.set_popup_focus(true);
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    Ok(())
}

fn submit_db_connect_prompt(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    text: &str,
) -> Result<(), String> {
    let (remember_alias, connection_string) = parse_db_connect_prompt(text)?;
    let persistence_available = db_service(runtime)?.secret_persistence_available();
    let session =
        db_service_mut(runtime)?.connect_raw(&connection_string, remember_alias.as_deref())?;
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!(
            "Connected {} session `{}`.",
            session.engine.label(),
            session.alias
        )]);
        if remember_alias.is_some() && !persistence_available {
            buffer.append_output_lines(&[
                "Remembered connections are unavailable here; session kept in memory only."
                    .to_owned(),
            ]);
        }
        buffer.clear_input();
    }
    let _ = close_popup_buffer_and_restore_focus(runtime, buffer_id);
    refresh_all_db_browser_buffers(runtime)?;
    open_db_schema_buffer(runtime, Some(session.id))
}

fn open_db_connections_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = open_or_focus_workspace_plugin_buffer(
        runtime,
        DB_CONNECTIONS_BUFFER_NAME,
        DB_CONNECTIONS_KIND,
    )?;
    let user_library = shell_user_library(runtime);
    let view = db_service_mut(runtime)?
        .render_connections_buffer_with(buffer_id.get(), &|context| {
            user_library.db_browser_items(context)
        })?;
    apply_db_browser_view(runtime, buffer_id, view)
}

fn open_db_schema_buffer(
    runtime: &mut EditorRuntime,
    session_id: Option<DbSessionId>,
) -> Result<(), String> {
    let buffer_id =
        open_or_focus_workspace_plugin_buffer(runtime, DB_SCHEMA_BUFFER_NAME, DB_SCHEMA_KIND)?;
    let user_library = shell_user_library(runtime);
    let view = db_service_mut(runtime)?.render_schema_buffer_with(
        buffer_id.get(),
        session_id,
        &|context| user_library.db_browser_items(context),
    )?;
    apply_db_browser_view(runtime, buffer_id, view)
}

fn open_db_history_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id =
        open_or_focus_workspace_plugin_buffer(runtime, DB_HISTORY_BUFFER_NAME, DB_HISTORY_KIND)?;
    let user_library = shell_user_library(runtime);
    let view = db_service_mut(runtime)?
        .render_history_buffer_with(buffer_id.get(), &|context| {
            user_library.db_browser_items(context)
        })?;
    apply_db_browser_view(runtime, buffer_id, view)
}

fn open_db_snippets_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id =
        open_or_focus_workspace_plugin_buffer(runtime, DB_SNIPPETS_BUFFER_NAME, DB_SNIPPETS_KIND)?;
    let user_library = shell_user_library(runtime);
    let view = db_service_mut(runtime)?
        .render_snippets_buffer_with(buffer_id.get(), &|context| {
            user_library.db_browser_items(context)
        })?;
    apply_db_browser_view(runtime, buffer_id, view)
}

fn create_db_query_buffer(
    runtime: &mut EditorRuntime,
    session_id: Option<DbSessionId>,
    sql: Option<&str>,
    requested_name: Option<&str>,
) -> Result<BufferId, String> {
    create_db_query_like_buffer(
        runtime,
        requested_name.unwrap_or("*db-query*"),
        DB_QUERY_KIND,
        session_id,
        sql,
        requested_name,
    )
}

fn open_db_query_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    if active_shell_buffer_id(runtime)
        .ok()
        .and_then(|buffer_id| shell_buffer(runtime, buffer_id).ok())
        .is_some_and(|buffer| buffer_is_db_dashboard(&buffer.kind))
    {
        return reset_db_dashboard_editor(runtime);
    }
    create_db_query_buffer(runtime, None, None, None).map(|_| ())
}

fn open_db_dashboard(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.is_db_multiview_active() {
        close_db_multiview(runtime)?;
    }
    let buffer_id = create_db_query_like_buffer(
        runtime,
        DB_DASHBOARD_BUFFER_NAME,
        DB_DASHBOARD_KIND,
        None,
        None,
        Some(DB_DASHBOARD_BUFFER_NAME),
    )?;
    focus_db_editor_section(runtime, buffer_id)?;
    refresh_db_layout_browsers(runtime, buffer_id)
}

fn open_db_multiview(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.is_db_multiview_active() {
        return close_db_multiview(runtime);
    }
    let sidebar_id =
        open_or_focus_workspace_plugin_buffer(runtime, DB_SIDEBAR_BUFFER_NAME, DB_SIDEBAR_KIND)?;
    refresh_db_layout_browsers(runtime, sidebar_id)?;
    let query_id = if let Some(existing) = first_db_query_buffer(runtime, sidebar_id) {
        existing
    } else {
        create_db_query_buffer(runtime, None, None, None)?
    };
    ensure_vertical_split_with_sidebar(runtime, sidebar_id, query_id)?;
    shell_ui_mut(runtime)?.set_db_multiview_layout(true);
    Ok(())
}

fn first_db_query_buffer(runtime: &EditorRuntime, exclude: BufferId) -> Option<BufferId> {
    shell_ui(runtime).ok()?.buffers.iter().find_map(|buffer| {
        (buffer.id() != exclude
            && buffer_is_db_query(&buffer.kind)
            && !buffer_is_db_dashboard(&buffer.kind))
        .then_some(buffer.id())
    })
}

fn ensure_vertical_split_with_sidebar(
    runtime: &mut EditorRuntime,
    sidebar_id: BufferId,
    query_id: BufferId,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let pane_count = shell_ui(runtime)?
        .workspace_view()
        .map(|view| view.panes.len())
        .unwrap_or(1);
    if pane_count < 2 {
        split_runtime_pane(runtime, PaneSplitDirection::Vertical)?;
    }
    let (left_pane, right_pane) = {
        let ui = shell_ui(runtime)?;
        let view = ui
            .workspace_view()
            .ok_or_else(|| "workspace view is missing".to_owned())?;
        let left = view
            .panes
            .first()
            .map(|pane| pane.pane_id)
            .ok_or_else(|| "left pane is missing".to_owned())?;
        let right = view
            .panes
            .get(1)
            .map(|pane| pane.pane_id)
            .ok_or_else(|| "right pane is missing".to_owned())?;
        (left, right)
    };
    runtime
        .model_mut()
        .focus_pane(workspace_id, left_pane)
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, sidebar_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_buffer_in_active_pane(sidebar_id);
    if let Some(view) = shell_ui_mut(runtime)?.workspace_view_mut()
        && let Some(pane) = view.panes.get_mut(0)
    {
        pane.buffer_id = sidebar_id;
    }
    runtime
        .model_mut()
        .focus_pane(workspace_id, right_pane)
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, query_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_buffer_in_active_pane(query_id);
    if let Some(view) = shell_ui_mut(runtime)?.workspace_view_mut()
        && let Some(pane) = view.panes.get_mut(1)
    {
        pane.buffer_id = query_id;
    }
    Ok(())
}

fn create_db_query_like_buffer(
    runtime: &mut EditorRuntime,
    initial_name: &str,
    kind: &str,
    session_id: Option<DbSessionId>,
    sql: Option<&str>,
    requested_name: Option<&str>,
) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            initial_name,
            BufferKind::Plugin(kind.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    let meta = db_service_mut(runtime)?
        .attach_query_buffer(buffer_id.get(), session_id, requested_name)
        .ok();
    if let Some((sql, path)) = sql.zip(meta.as_ref().map(|meta| meta.temp_path.clone())) {
        fs::write(&path, sql).map_err(|error| {
            format!(
                "failed to seed DB query buffer `{}`: {error}",
                path.display()
            )
        })?;
    }
    let title = meta
        .as_ref()
        .map(|meta| meta.title.clone())
        .unwrap_or_else(|| requested_name.unwrap_or(initial_name).to_owned());
    runtime
        .model_mut()
        .set_buffer_name(workspace_id, buffer_id, title)
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let text = sql
        .map(TextBuffer::from_text)
        .or_else(|| {
            meta.as_ref()
                .and_then(|meta| fs::read_to_string(&meta.temp_path).ok())
                .map(TextBuffer::from_text)
        })
        .unwrap_or_else(|| TextBuffer::from_text(query_starter_sql()));
    let user_library = shell_user_library(runtime);
    let mut shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
    if let Some(meta) = meta {
        shell_buffer.set_language_id(Some(meta.dialect_id));
        shell_buffer.set_lsp_path(Some(meta.temp_path));
        shell_buffer.set_lsp_enabled(true);
    } else {
        shell_buffer.set_language_id(Some("sql".to_owned()));
    }
    shell_buffer.force_syntax_refresh();
    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
    }
    Ok(buffer_id)
}

fn query_starter_sql() -> &'static str {
    "-- SQL query\n-- Ctrl+c Ctrl+c  execute statement or selection\n\nSELECT *\nFROM ;\n"
}

fn reset_db_dashboard_editor(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.replace_with_lines(query_starter_sql().lines().map(str::to_owned).collect());
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    }
    runtime
        .model_mut()
        .set_buffer_path(workspace_id, buffer_id, None)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn focus_db_editor_section(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    Ok(())
}

fn open_db_query_from_sql(
    runtime: &mut EditorRuntime,
    session_id: DbSessionId,
    sql: &str,
    requested_name: Option<&str>,
) -> Result<(), String> {
    if let Some(dashboard_id) = active_or_open_dashboard_buffer(runtime) {
        load_sql_into_db_editor(runtime, dashboard_id, session_id, sql, requested_name)?;
        return Ok(());
    }
    create_db_query_buffer(runtime, Some(session_id), Some(sql), requested_name).map(|_| ())
}

fn active_or_open_dashboard_buffer(runtime: &EditorRuntime) -> Option<BufferId> {
    let ui = shell_ui(runtime).ok()?;
    ui.buffers
        .iter()
        .find_map(|buffer| buffer_is_db_dashboard(&buffer.kind).then_some(buffer.id()))
}

fn load_sql_into_db_editor(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    session_id: DbSessionId,
    sql: &str,
    requested_name: Option<&str>,
) -> Result<(), String> {
    if db_service(runtime)?
        .query_buffer_session_id(buffer_id.get())
        .is_none()
    {
        db_service_mut(runtime)?.attach_query_buffer(
            buffer_id.get(),
            Some(session_id),
            requested_name,
        )?;
    }
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if let Some(name) = requested_name {
        runtime
            .model_mut()
            .set_buffer_name(workspace_id, buffer_id, name.to_owned())
            .map_err(|error| error.to_string())?;
    }
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.replace_with_lines(sql.lines().map(str::to_owned).collect());
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
        buffer.force_syntax_refresh();
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_buffer_in_active_pane(buffer_id);
    Ok(())
}

fn open_db_query_for_table_preview(
    runtime: &mut EditorRuntime,
    session_id: DbSessionId,
    table: &QualifiedName,
) -> Result<(), String> {
    if let Some(dashboard_id) = active_or_open_dashboard_buffer(runtime) {
        let sql = db_service(runtime)?.preview_sql_for_table(session_id, table)?;
        return load_sql_into_db_editor(
            runtime,
            dashboard_id,
            session_id,
            &sql,
            Some(&format!("*db-query {}*", table.display())),
        );
    }
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*db-query*",
            BufferKind::Plugin(DB_QUERY_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    let (meta, sql) = db_service_mut(runtime)?.attach_table_preview_query_buffer(
        buffer_id.get(),
        session_id,
        table,
    )?;
    runtime
        .model_mut()
        .set_buffer_name(workspace_id, buffer_id, meta.title.clone())
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let text = TextBuffer::from_text(sql);
    let user_library = shell_user_library(runtime);
    let mut shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
    shell_buffer.set_language_id(Some(meta.dialect_id.clone()));
    shell_buffer.set_lsp_path(Some(meta.temp_path.clone()));
    shell_buffer.set_lsp_enabled(true);
    shell_buffer.force_syntax_refresh();
    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
    }
    Ok(())
}

fn db_query_scope_sql(runtime: &EditorRuntime, buffer_id: BufferId) -> Result<String, String> {
    let ui = shell_ui(runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "active DB query buffer is missing".to_owned())?;
    let text = buffer.text.text();
    let cursor_char_index = buffer.text.point_to_char_index(buffer.cursor_point());
    match ui.visual_selection_for_buffer(buffer, true) {
        Some(VisualSelection::Range(range)) => {
            let start = buffer.text.point_to_char_index(range.start());
            let end = buffer.text.point_to_char_index(range.end());
            sql_scope_from_text(&text, cursor_char_index, Some((start, end)))
                .ok_or_else(|| "no SQL selected for execution".to_owned())
        }
        Some(VisualSelection::Block(block)) => {
            let sql = block_selection_ranges(buffer, block)
                .into_iter()
                .map(|range| buffer.slice(range))
                .collect::<Vec<_>>()
                .join("\n");
            let sql = sql.trim().to_owned();
            if sql.is_empty() {
                Err("no SQL selected for execution".to_owned())
            } else {
                Ok(sql)
            }
        }
        None => {
            if buffer_is_db_dashboard(&buffer.kind) {
                let sql = text.trim().to_owned();
                if sql.is_empty() {
                    Err("no SQL selected for execution".to_owned())
                } else {
                    Ok(sql)
                }
            } else {
                sql_scope_from_text(&text, cursor_char_index, None)
                    .ok_or_else(|| "no SQL selected for execution".to_owned())
            }
        }
    }
}

fn open_db_results_popup(
    runtime: &mut EditorRuntime,
    output: Result<DbExecutionOutput, String>,
) -> Result<(), String> {
    let buffer_id = open_or_focus_popup_plugin_buffer(
        runtime,
        DB_RESULTS_BUFFER_NAME,
        DB_RESULTS_KIND,
        "DB Results",
    )?;
    let (lines, is_error) = match output {
        Ok(output) => {
            let mut lines = vec![output.title];
            if !output.lines.is_empty() {
                lines.push(String::new());
                lines.extend(output.lines);
            }
            (lines, false)
        }
        Err(error) => (vec!["Query failed".to_owned(), String::new(), error], true),
    };
    let syntax_lines = db_results_syntax_lines(&lines, is_error);
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.replace_with_lines_preserve_view(lines);
        buffer.set_indexed_syntax_lines(Some(syntax_lines), None);
    }
    Ok(())
}

fn execute_db_sql(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let kind = shell_buffer(runtime, buffer_id)?.kind.clone();
    if !buffer_is_db_query(&kind) {
        return Err("db.execute-sql requires an active DB query buffer".to_owned());
    }
    if buffer_is_db_dashboard(&kind) {
        let section = shell_buffer(runtime, buffer_id)?
            .plugin_active_section_name()
            .unwrap_or(DB_EDITOR_SECTION)
            .to_owned();
        if section != DB_EDITOR_SECTION {
            return Err("db.execute-sql requires the Editor section".to_owned());
        }
    }
    let sql = db_query_scope_sql(runtime, buffer_id)?;
    if db_service(runtime)?
        .query_buffer_session_id(buffer_id.get())
        .is_none()
    {
        match db_service_mut(runtime)?.attach_query_buffer(buffer_id.get(), None, None) {
            Ok(_) => {}
            Err(error) if buffer_is_db_dashboard(&kind) => {
                apply_db_results_to_output_section(runtime, buffer_id, Err(error))?;
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    }
    let output = if buffer_is_db_dashboard(&kind) && split_sql_statements(&sql).len() > 1 {
        db_service_mut(runtime)?.execute_sql_batch_for_buffer(buffer_id.get(), &sql)
    } else {
        db_service_mut(runtime)?.execute_sql_for_buffer(buffer_id.get(), &sql)
    };
    if buffer_is_db_dashboard(&kind) {
        apply_db_results_to_output_section(runtime, buffer_id, output)?;
    } else {
        open_db_results_popup(runtime, output)?;
    }
    refresh_all_db_browser_buffers(runtime)
}

fn apply_db_results_to_output_section(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    output: Result<DbExecutionOutput, String>,
) -> Result<(), String> {
    let lines = match output {
        Ok(output) => {
            let mut lines = vec![output.title];
            if !output.lines.is_empty() {
                lines.push(String::new());
                lines.extend(output.lines);
            }
            lines
        }
        Err(error) => vec!["Query failed".to_owned(), String::new(), error],
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    let _ = buffer.plugin_focus_section_named(DB_OUTPUT_SECTION);
    buffer.set_plugin_output_lines(lines);
    let _ = buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    Ok(())
}

fn snippet_name_from_sql(sql: &str) -> String {
    let compact = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        "snippet".to_owned()
    } else if compact.chars().count() <= 48 {
        compact
    } else {
        let mut prefix = compact.chars().take(45).collect::<String>();
        prefix.push_str("...");
        prefix
    }
}

fn save_db_snippet(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let kind = shell_buffer(runtime, buffer_id)?.kind.clone();
    if !buffer_is_db_query(&kind) {
        return Err("db.save-snippet requires an active DB query buffer".to_owned());
    }
    let sql = db_query_scope_sql(runtime, buffer_id)?;
    let name = snippet_name_from_sql(&sql);
    db_service_mut(runtime)?.save_snippet(buffer_id.get(), &name, &sql)?;
    refresh_all_db_browser_buffers(runtime)
}

fn refresh_db_schema(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let session_id = {
        let db = db_service(runtime)?;
        db.query_buffer_session_id(buffer_id.get())
            .or_else(|| db.active_session_summary().map(|summary| summary.id))
            .ok_or_else(|| "no active database session".to_owned())?
    };
    db_service_mut(runtime)?.refresh_schema_cache(session_id)?;
    refresh_all_db_browser_buffers(runtime)
}

fn activate_db_browser_line(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let kind = shell_buffer(runtime, buffer_id)?.kind.clone();
    if !buffer_is_db_browser(&kind) {
        return Err("db.activate-line requires an active DB browser buffer".to_owned());
    }
    let (section, line_index) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if buffer_is_db_dashboard(&kind) || buffer_is_db_sidebar(&kind) {
            let section = buffer.plugin_active_section_name().unwrap_or("").to_owned();
            if section != DB_CONNECTIONS_SECTION && section != DB_TABLES_SECTION {
                return Err("db.activate-line requires a Connections or Tables section".to_owned());
            }
            let line_index = buffer
                .plugin_attached_pane_state()
                .map(|pane| pane.cursor().line)
                .unwrap_or_else(|| buffer.cursor_row());
            (section, line_index)
        } else {
            (String::new(), buffer.cursor_row())
        }
    };
    let action = db_service(runtime)?
        .browser_action_in(buffer_id.get(), &section, line_index)
        .ok_or_else(|| "no action is attached to the current database browser line".to_owned())?;
    match db_service_mut(runtime)?.activate_browser_action(action)? {
        DbActionOutcome::ActivatedSession(_) => {
            refresh_all_db_browser_buffers(runtime)?;
            refresh_db_browser_surface(runtime, buffer_id)
        }
        DbActionOutcome::Disconnected => {
            refresh_all_db_browser_buffers(runtime)?;
            refresh_db_browser_surface(runtime, buffer_id)
        }
        DbActionOutcome::OpenPreviewQuery { session_id, table } => {
            open_db_query_for_table_preview(runtime, session_id, &table)
        }
        DbActionOutcome::ExploreRows { session_id, table } => {
            open_db_query_for_table_preview(runtime, session_id, &table)
        }
        DbActionOutcome::SchemaRefreshed(_) => refresh_db_browser_surface(runtime, buffer_id),
        DbActionOutcome::OpenSql { session_id, sql } => {
            open_db_query_from_sql(runtime, session_id, &sql, None)
        }
        DbActionOutcome::SnippetDeleted | DbActionOutcome::RememberedDeleted => {
            refresh_all_db_browser_buffers(runtime)?;
            refresh_db_browser_surface(runtime, buffer_id)
        }
    }
}

fn refresh_db_browser_surface(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let kind = shell_buffer(runtime, buffer_id)?.kind.clone();
    if buffer_is_db_dashboard(&kind) || buffer_is_db_sidebar(&kind) {
        refresh_db_layout_browsers(runtime, buffer_id)
    } else {
        refresh_db_browser_buffer(runtime, buffer_id)
    }
}

fn clear_input_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer_is_acp(&buffer.kind)
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.clear_input();
    if is_acp {
        acp::refresh_acp_input_hint(runtime, buffer_id)?;
    }
    Ok(())
}

fn focus_acp_input_section(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    if shell_buffer_mut(runtime, buffer_id)?.focus_acp_input() {
        start_change_recording(runtime)?;
        mark_change_finish_on_normal(runtime)?;
        let ui = shell_ui_mut(runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    Ok(())
}

fn normalize_mark_list_buffer_before_save(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    path: &Path,
) -> Result<(), String> {
    if mark_list_state(runtime)?.path != path {
        return Ok(());
    }
    let (current, cursor) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        (buffer.text.text(), buffer.cursor_point())
    };
    let normalized = mark_list_from_persisted_text(&current).serialize();
    if current == normalized {
        return Ok(());
    }
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    let end = buffer.text.point_from_char_index(buffer.text.char_count());
    buffer.replace_range(TextRange::new(TextPoint::default(), end), &normalized);
    buffer.set_cursor(cursor);
    Ok(())
}

fn reload_mark_list_after_save(runtime: &mut EditorRuntime, path: &Path) -> Result<(), String> {
    if mark_list_state(runtime)?.path != path {
        return Ok(());
    }
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to reload Mark List `{}`: {error}", path.display()))?;
    mark_list_state_mut(runtime)?.list = mark_list_from_persisted_text(&text);
    Ok(())
}
