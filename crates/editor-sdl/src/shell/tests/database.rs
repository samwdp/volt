use super::*;

#[test]
fn db_table_preview_buffer_exposes_hidden_sqls_path_without_file_open_hooks() -> Result<(), String>
{
    let state_dir = TempTestDir::new("db-preview-no-file-open-hooks");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("preview.sqlite3");
    let mut state = state_with_user_library()?;
    let connection_string = format!("sqlite://{}", db_path.display());
    let session = db_service_mut(&mut state.runtime)?
        .connect_raw(&connection_string, Some("preview"))
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .attach_query_buffer(99, Some(session.id), None)
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .execute_sql_for_buffer(
            99,
            "CREATE TABLE widgets (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
        )
        .map_err(|error| error.to_string())?;

    open_db_query_for_table_preview(
        &mut state.runtime,
        session.id,
        &QualifiedName {
            schema: None,
            name: "widgets".to_owned(),
        },
    )?;

    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert!(buffer_is_db_query(&buffer.kind));
    assert_eq!(buffer.language_id(), Some("sql"));
    assert!(
        buffer.lsp_enabled(),
        "DB scratch query buffers should opt into sqls syncs"
    );
    assert!(
        buffer.desired_syntax_window().is_some(),
        "DB scratch query buffers should be queued for tree-sitter highlighting"
    );
    assert!(
        buffer.path().is_none(),
        "DB scratch query buffers should not masquerade as file-backed workspace buffers",
    );
    assert!(
        buffer
            .lsp_path()
            .is_some_and(|path| path.extension().and_then(|value| value.to_str()) == Some("sql")),
        "DB scratch query buffers should expose a hidden .sql path for sqls",
    );
    assert!(
        formatter_registry(&state.runtime)?
            .formatter_for_language("sql")
            .is_none(),
        "DB scratch query buffers should not trigger generic file-open formatter hooks",
    );
    assert_eq!(
        syntax_indent_for_buffer(&mut state.runtime, buffer_id, 0, 2, false)?,
        None,
        "DB scratch query buffers should use text-only indentation"
    );
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.open_line_below();
    }
    format_current_line_indent(&mut state.runtime, buffer_id, 2, false)?;
    assert!(
        shell_user_library(&state.runtime)
            .plugin_buffer_key_bindings(DB_QUERY_KIND)
            .iter()
            .any(|binding| binding.chord() == "Ctrl+c Ctrl+c"
                && binding
                    .command_names()
                    .iter()
                    .any(|command| command.as_str() == "db.execute-sql")),
        "DB query buffers should expose the execute SQL chord"
    );
    Ok(())
}

#[test]
fn db_dashboard_layout_places_sidebar_left_and_editor_output_right() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    open_db_dashboard(&mut state.runtime)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let panes = plugin_section_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "dashboard section layout missing".to_owned())?;
    assert_eq!(panes.panes.len(), 4);
    let editor = panes.panes[0].rect;
    let connections = panes.panes[1].rect;
    let tables = panes.panes[2].rect;
    let output = panes.panes[3].rect;
    assert!(
        connections.x() < editor.x(),
        "Connections should sit left of Editor"
    );
    assert!(tables.x() < output.x(), "Tables should sit left of Output");
    assert!(
        tables.y() > connections.y(),
        "Tables should sit below Connections"
    );
    assert!(output.y() > editor.y(), "Output should sit below Editor");
    Ok(())
}

#[test]
fn db_dashboard_execute_replaces_output_and_concatenates_multiple_queries() -> Result<(), String> {
    let state_dir = TempTestDir::new("db-dashboard-execute");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("dashboard.sqlite3");
    let mut state = state_with_user_library()?;
    let connection_string = format!("sqlite://{}", db_path.display());
    db_service_mut(&mut state.runtime)?
        .connect_raw(&connection_string, Some("dashboard"))
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .attach_query_buffer(99, None, None)
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .execute_sql_for_buffer(
            99,
            "CREATE TABLE widgets (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
        )
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .execute_sql_for_buffer(99, "INSERT INTO widgets(name) VALUES ('Ada'), ('Grace');")
        .map_err(|error| error.to_string())?;

    open_db_dashboard(&mut state.runtime)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec!["SELECT name FROM widgets WHERE id = 1;".to_owned()]);
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    }
    state
        .runtime
        .execute_command("db.execute-sql")
        .map_err(|error| error.to_string())?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let output = plugin_section_lines(buffer, DB_OUTPUT_SECTION)?;
        assert!(
            output.iter().any(|line| line.contains("Ada")),
            "first execute should write Ada into Output: {output:?}"
        );
        assert!(
            !output.iter().any(|line| line.contains("Grace")),
            "first execute should not include Grace: {output:?}"
        );
        assert_eq!(buffer.plugin_active_section_name(), Some(DB_EDITOR_SECTION));
    }

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec!["SELECT name FROM widgets WHERE id = 2;".to_owned()]);
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    }
    state
        .runtime
        .execute_command("db.execute-sql")
        .map_err(|error| error.to_string())?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let output = plugin_section_lines(buffer, DB_OUTPUT_SECTION)?;
        assert!(
            output.iter().any(|line| line.contains("Grace")),
            "second execute should replace Output with Grace: {output:?}"
        );
        assert!(
            !output.iter().any(|line| line.contains("Ada")),
            "second execute should overwrite Ada: {output:?}"
        );
    }

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "SELECT name FROM widgets WHERE id = 1;".to_owned(),
            "SELECT name FROM widgets WHERE id = 2;".to_owned(),
        ]);
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    }
    state
        .runtime
        .execute_command("db.execute-sql")
        .map_err(|error| error.to_string())?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let output = plugin_section_lines(buffer, DB_OUTPUT_SECTION)?;
    assert!(
        output.iter().any(|line| line.contains("-- Query 1")),
        "batch execute should label first query: {output:?}"
    );
    assert!(
        output.iter().any(|line| line.contains("-- Query 2")),
        "batch execute should label second query: {output:?}"
    );
    assert!(output.iter().any(|line| line.contains("Ada")));
    assert!(output.iter().any(|line| line.contains("Grace")));
    Ok(())
}

#[test]
fn db_dashboard_opens_and_writes_files_through_editor_section() -> Result<(), String> {
    let root = TempTestDir::new("db-dashboard-file-open");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("query.sql");
    fs::write(&path, "SELECT 1;\n").map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;
    open_db_dashboard(&mut state.runtime)?;
    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        assert!(buffer_is_db_dashboard(&buffer.kind));
        assert_eq!(buffer.text.text().trim(), "SELECT 1;");
        assert_eq!(buffer.plugin_active_section_name(), Some(DB_EDITOR_SECTION));
        assert_eq!(buffer.path(), Some(path.as_path()));
    }
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec!["SELECT 2;".to_owned()]);
    }
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    save_buffer(&mut state.runtime, workspace_id, buffer_id)?;
    let saved = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    assert_eq!(saved.trim(), "SELECT 2;");
    Ok(())
}

#[test]
fn db_multiview_disables_golden_ratio_and_narrows_left_sidebar() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    open_db_multiview(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert_eq!(view.golden_ratio_override, Some(false));
    assert_eq!(
        view.pane_size_weights.as_deref(),
        Some([DB_MULTIVIEW_LEFT_WEIGHT, DB_MULTIVIEW_RIGHT_WEIGHT].as_slice())
    );
    assert_eq!(view.panes.len(), 2);
    let left = ui
        .buffer(view.panes[0].buffer_id)
        .ok_or_else(|| "left pane buffer missing".to_owned())?;
    let right = ui
        .buffer(view.panes[1].buffer_id)
        .ok_or_else(|| "right pane buffer missing".to_owned())?;
    assert!(buffer_is_db_sidebar(&left.kind));
    assert!(buffer_is_db_query(&right.kind));
    assert!(!buffer_is_db_dashboard(&right.kind));
    let rects = workspace_pane_rects(&*shell_user_library(&state.runtime), ui, 400, 200, 2);
    assert!(
        rects[0].width < rects[1].width,
        "multiview left split should be narrower: {rects:?}"
    );
    Ok(())
}

#[test]
fn db_multiview_toggle_restores_golden_ratio() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    open_db_multiview(&mut state.runtime)?;
    open_db_multiview(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert_eq!(view.golden_ratio_override, None);
    assert!(view.pane_size_weights.is_none());
    assert_eq!(view.panes.len(), 1);
    assert!(
        shell_user_library(&state.runtime)
            .pane_config()
            .golden_ratio,
        "default pane config should keep golden ratio enabled"
    );
    Ok(())
}

#[test]
fn db_connect_enter_submits_pasted_connection_string() -> Result<(), String> {
    let state_dir = TempTestDir::new("db-connect-enter");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("connect.sqlite3");
    let connection_string = format!("sqlite://{}", db_path.display());
    let mut state = state_with_user_library()?;

    state
        .runtime
        .execute_command("db.connect")
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui(&state.runtime)?;
        assert!(
            ui.popup_focus,
            "db.connect prompt should take popup focus so paste and Enter target the prompt"
        );
        assert_eq!(ui.input_mode(), InputMode::Insert);
        assert_eq!(ui.vim().target, VimTarget::Input);
    }
    assert!(
        paste_text_into_active_input_buffer(&mut state.runtime, &connection_string)
            .map_err(|error| error.to_string())?,
        "paste should land in the DB connect input"
    );
    let handled = state
        .try_runtime_keybinding(Keycode::Return, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    assert!(handled, "Enter should submit the DB connect prompt");
    let session = db_service(&state.runtime)?
        .active_session_summary()
        .ok_or_else(|| "Enter did not create a database session".to_owned())?;
    assert_eq!(session.engine.label(), "SQLite");
    Ok(())
}
