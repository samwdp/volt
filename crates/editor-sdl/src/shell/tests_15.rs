#[test]
fn browser_url_command_opens_split_browser_with_detected_url() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("Docs: https://example.com/docs.");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 8));

    open_detected_browser_url(&mut state.runtime)?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "browser split buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    Ok(())
}

#[test]
fn browser_open_buffer_command_opens_split_with_file_url() -> Result<(), String> {
    let root = unique_temp_dir("browser-open-buffer");
    let html_path = root.join("page.html");
    std::fs::write(&html_path, "<html><body>preview</body></html>")
        .map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("<html><body>preview</body></html>");
        buffer.text.set_path(html_path.clone());
    }

    open_active_buffer_in_browser_split(&mut state.runtime)?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "browser split buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    let expected_url = path_to_file_url(&html_path);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some(expected_url.as_str())
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn browser_open_buffer_command_uses_existing_split_pane() -> Result<(), String> {
    let root = unique_temp_dir("browser-open-buffer-split");
    let html_path = root.join("preview.html");
    std::fs::write(&html_path, "<html></html>").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let source_buffer_id = active_shell_buffer_id(&state.runtime)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("<html></html>");
        buffer.text.set_path(html_path.clone());
    }
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 2);
    focus_test_buffer(&mut state, source_buffer_id)?;

    open_active_buffer_in_browser_split(&mut state.runtime)?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let browser_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(browser_buffer_id)
        .ok_or_else(|| "browser buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    assert!(
        ui.panes()
            .is_some_and(|panes| panes.iter().any(|pane| pane.buffer_id == source_buffer_id)),
        "source file buffer should remain open in the other pane"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn sync_active_browser_buffer_enters_insert_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            BROWSER_BUFFER_NAME,
            BufferKind::Plugin(BROWSER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;

    sync_active_buffer(&mut state.runtime)?;
    state
        .handle_text_input("example.com")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_id));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .text(),
        "example.com"
    );
    Ok(())
}

#[test]
fn browser_host_focus_parent_event_returns_to_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .enter_insert_mode();

    state
        .apply_browser_host_events(&[BrowserHostEvent::FocusParentRequested { buffer_id }])
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state.ui().map_err(|error| error.to_string())?.input_mode(),
        InputMode::Normal
    );
    Ok(())
}

#[test]
fn browser_host_new_window_event_routes_into_browser_popup() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    state
        .apply_browser_host_events(&[BrowserHostEvent::NewWindowRequested {
            buffer_id,
            url: "https://example.com/oauth/callback?code=test".to_owned(),
        }])
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "browser popup was not opened from new-window event".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    let popup_buffer = ui
        .buffer(popup.active_buffer)
        .ok_or_else(|| "popup browser buffer missing".to_owned())?;
    assert!(ui.popup_focus);
    assert!(matches!(
        popup_buffer.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
    assert_eq!(
        popup_buffer
            .browser_state
            .as_ref()
            .and_then(|browser| browser.requested_url.as_deref()),
        Some("https://example.com/oauth/callback?code=test")
    );
    Ok(())
}

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
fn db_query_buffer_receives_sql_highlighting_without_blocking() -> Result<(), String> {
    let state_dir = TempTestDir::new("db-query-syntax-refresh");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("query.sqlite3");
    let mut state = state_with_user_library()?;
    let connection_string = format!("sqlite://{}", db_path.display());
    db_service_mut(&mut state.runtime)?
        .connect_raw(&connection_string, Some("query"))
        .map_err(|error| error.to_string())?;

    open_db_query_buffer(&mut state.runtime)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert!(buffer_is_db_query(&buffer.kind));
    assert_eq!(buffer.language_id(), Some("sql"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(3).is_some_and(|spans| {
            spans
                .iter()
                .any(|span| span.theme_token.starts_with("syntax.keyword"))
        }),
        "DB query starter SQL should receive keyword highlighting"
    );
    Ok(())
}

#[test]
fn opened_sql_file_survives_layout_and_syntax_refresh() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-sql-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("query.sql");
    fs::write(&path, "SELECT *\nFROM widgets\nWHERE id = 1;\n")
        .map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;
    sync_active_buffer_layout_for_test(&mut state)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.language_id(), Some("sql"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(0).is_some_and(|spans| {
            spans
                .iter()
                .any(|span| span.theme_token.starts_with("syntax.keyword"))
        }),
        "opened SQL file should receive keyword highlight spans"
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

#[test]
fn opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-toml-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("volt.toml");
    fs::write(&path, "title = \"Volt\"\n[editor]\nmode = \"vim\"\n")
        .map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;
    sync_active_buffer_layout_for_test(&mut state)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.language_id(), Some("toml"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(0).is_some(),
        "opened TOML file should receive syntax spans"
    );
    Ok(())
}

#[test]
fn opened_file_receives_tree_sitter_highlighting() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("main.rs");
    fs::write(&path, "fn main() {\n    let value = 1;\n}\n").map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;

    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .line_syntax_spans(0)
            .is_some_and(|spans| {
                spans
                    .iter()
                    .any(|span| span.theme_token.starts_with("syntax.keyword"))
            }),
        "opened file should receive syntax highlight spans"
    );
    Ok(())
}

#[test]
fn insert_mode_is_buffer_local_across_buffer_switches() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    let buffer_b = install_scratch_test_buffer(&mut state, "*vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);

    focus_test_buffer(&mut state, buffer_a)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn insert_mode_is_buffer_local_across_split_focus_changes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*split-vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    cycle_runtime_pane(&mut state.runtime)?;

    let buffer_b = install_scratch_test_buffer(&mut state, "*split-vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);

    cycle_runtime_pane(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Insert);

    cycle_runtime_pane(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn same_buffer_split_keeps_independent_cursor_and_scroll() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let lines = (0..64)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>();
    let buffer_id = install_text_test_buffer(&mut state, "*split-shared-buffer*", lines)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(buffer.id(), buffer_id);
        buffer.set_cursor(TextPoint::new(2, 3));
        buffer.scroll_row = 1;
    }

    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let pane_id = state
        .runtime
        .model_mut()
        .split_pane(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.split_pane(pane_id, buffer_id, PaneSplitDirection::Vertical);
    shell_ui_mut(&mut state.runtime)?.focus_pane(pane_id);
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(buffer.id(), buffer_id);
        buffer.set_cursor(TextPoint::new(20, 2));
        buffer.scroll_row = 18;
    }

    cycle_runtime_pane(&mut state.runtime)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(buffer.id(), buffer_id);
        assert_eq!(buffer.cursor_point(), TextPoint::new(2, 3));
        assert_eq!(buffer.scroll_row, 1);
    }

    cycle_runtime_pane(&mut state.runtime)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(buffer.id(), buffer_id);
        assert_eq!(buffer.cursor_point(), TextPoint::new(20, 2));
        assert_eq!(buffer.scroll_row, 18);
    }
    Ok(())
}

#[test]
fn inactive_split_render_reads_saved_buffer_input_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*render-vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    cycle_runtime_pane(&mut state.runtime)?;

    let buffer_b = install_scratch_test_buffer(&mut state, "*render-vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode_for_buffer(buffer_b, true), InputMode::Normal);
    assert_eq!(ui.input_mode_for_buffer(buffer_a, false), InputMode::Insert);
    Ok(())
}

#[test]
fn popup_terminal_focus_restores_its_own_vim_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let split_buffer = install_scratch_test_buffer(&mut state, "*popup-split*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let popup_buffer = install_terminal_popup_test_buffer(&mut state)?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(
        ui.input_mode_for_buffer(split_buffer, false),
        InputMode::Insert
    );

    let anchor = TextPoint::new(0, 0);
    shell_ui_mut(&mut state.runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(anchor));

    shell_ui_mut(&mut state.runtime)?.set_popup_focus(false);

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(
        ui.input_mode_for_buffer(popup_buffer, false),
        InputMode::Visual
    );

    shell_ui_mut(&mut state.runtime)?.set_popup_focus(true);

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(anchor));
    Ok(())
}

#[test]
fn visual_mode_is_buffer_local_across_buffer_switches() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*visual-a*")?;
    let anchor = TextPoint::new(0, 0);
    shell_ui_mut(&mut state.runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);

    let buffer_b = install_scratch_test_buffer(&mut state, "*visual-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(ui.vim().visual_anchor, None);

    focus_test_buffer(&mut state, buffer_a)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(anchor));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Character);
    Ok(())
}

#[test]
fn terminal_scroll_for_motion_maps_terminal_viewport_navigation() {
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::Down, None),
        Some(TerminalViewportScroll::LineDelta(-1))
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::Up, Some(3)),
        Some(TerminalViewportScroll::LineDelta(3))
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::FirstLine, Some(42)),
        Some(TerminalViewportScroll::Top)
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::LastLine, None),
        Some(TerminalViewportScroll::Bottom)
    );
    assert_eq!(terminal_scroll_for_motion(ShellMotion::Left, None), None);
}

#[test]
fn repeated_keydown_events_move_the_cursor() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(&mut state, "*repeat*", vec!["abcd".to_owned()])?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 3));

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Left),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: true,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(0, 2)
    );
    Ok(())
}

#[test]
fn undo_tree_root_cursor_tracks_last_root_cursor_across_undo_redo() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*undo-tree-root-redo*",
        vec!["alpha".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;

    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    buffer.record_undo_snapshot();

    assert!(buffer.undo_tree_undo());
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 5));

    buffer.set_cursor(TextPoint::new(0, 2));
    assert!(buffer.undo_tree_redo());
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha!"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 6));

    assert!(buffer.undo_tree_undo());
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 2));
    Ok(())
}

#[test]
fn undo_tree_select_restores_latest_root_cursor_without_changing_child_cursor() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*undo-tree-root-select*",
        vec!["alpha".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;

    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    buffer.record_undo_snapshot();

    assert!(buffer.undo_tree_undo());
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 5));

    buffer.set_cursor(TextPoint::new(0, 3));
    assert!(buffer.undo_tree_select(1));
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha!"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 6));

    assert!(buffer.undo_tree_select(0));
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 3));
    Ok(())
}

#[test]
fn undo_tree_picker_entries_use_fringe_indent_and_diff_preview() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*undo-tree-picker*", vec!["alpha".to_owned()])?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;

    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    buffer.record_undo_snapshot();

    let (entries, selected_index) = buffer.undo_tree_entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(selected_index, 1);
    assert!(!entries[0].label.starts_with(' '));
    assert!(!entries[1].label.starts_with(' '));
    assert!(entries[0].fringe.contains('*') || entries[0].fringe.contains('○'));
    assert!(entries[1].fringe.contains('├') || entries[1].fringe.contains('└'));
    let preview = entries[1]
        .preview
        .as_deref()
        .ok_or_else(|| "child preview missing".to_owned())?;
    assert!(
        preview.contains("-alpha") && preview.contains("+alpha!"),
        "preview should show parent→node diff, got {preview}"
    );
    Ok(())
}

#[test]
fn mouse_wheel_scrolls_the_buffer_under_the_pointer() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-scroll*",
        (0..20).map(|index| format!("line {index}")).collect(),
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;

    let handled = state
        .handle_event(
            Event::MouseWheel {
                timestamp: 0,
                window_id: 0,
                which: 0,
                x: 0.0,
                y: -1.0,
                direction: MouseWheelDirection::Normal,
                mouse_x: 24.0,
                mouse_y: 24.0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.scroll_row, MOUSE_WHEEL_SCROLL_LINES as usize);
    assert_eq!(buffer.cursor_row(), MOUSE_WHEEL_SCROLL_LINES as usize);
    Ok(())
}

#[test]
fn scroll_by_uses_wrapped_max_for_line_wrap() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*wrapped-scroll-max*",
        (0..30).map(|index| format!("line {index}")).collect(),
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.line_wrap = true;
        buffer.set_viewport_lines(8);
        buffer.set_scroll_layout(8, 40, 4);
        let expected = buffer.max_scroll_row_for_wrapped_rows(8, 40, 4);
        assert_eq!(buffer.max_scroll_row(), expected);
        assert!(buffer.max_scroll_row() < buffer.line_count().saturating_sub(1));
        buffer.scroll_row = buffer.max_scroll_row();
        assert_eq!(buffer.line_at_viewport_offset(7), 29);
    }
    Ok(())
}

#[test]
fn scroll_by_uses_content_viewport_rows_after_layout_sync() -> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*content-viewport-scroll*",
        (0..80).map(|index| format!("line {index}")).collect(),
    )?;
    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    assert!(buffer.content_viewport_lines < buffer.viewport_lines);
    assert_eq!(
        buffer.max_scroll_row(),
        buffer
            .line_count()
            .saturating_sub(buffer.content_viewport_lines)
    );
    buffer.scroll_row = buffer.max_scroll_row();
    assert_eq!(
        buffer.line_at_viewport_offset(buffer.content_viewport_lines.saturating_sub(1)),
        79
    );
    Ok(())
}

#[test]
fn mouse_drag_creates_a_character_visual_selection() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-drag*",
        vec!["alpha beta".to_owned(), "gamma delta".to_owned()],
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;
    let start = TextPoint::new(0, 1);
    let end = TextPoint::new(1, 3);
    let (start_x, start_y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        start,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;
    let (end_x, end_y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        end,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;

    state
        .handle_event(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: start_x,
                y: start_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseMotion {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mousestate: MouseState::from_sdl_state(0),
                x: end_x,
                y: end_y,
                xrel: end_x - start_x,
                yrel: end_y - start_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseButtonUp {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: end_x,
                y: end_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(start));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Character);
    assert_eq!(buffer.cursor_point(), end);
    assert_eq!(
        visual_selection(buffer, start, VisualSelectionKind::Character),
        Some(VisualSelection::Range(TextRange::new(
            start,
            buffer.point_after(end).unwrap_or(end)
        )))
    );
    assert!(state.mouse_drag.is_none());
    Ok(())
}

#[test]
fn mouse_double_click_selects_the_whole_line() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-double-click*",
        vec!["alpha beta".to_owned(), "gamma delta".to_owned()],
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;
    let point = TextPoint::new(1, 2);
    let (x, y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        point,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;

    state
        .handle_event(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 2,
                x,
                y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseButtonUp {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 2,
                x,
                y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(point));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(buffer.cursor_point(), point);
    assert_eq!(
        visual_selection(buffer, point, VisualSelectionKind::Line),
        buffer.line_span_range(1, 1).map(VisualSelection::Range)
    );
    assert!(state.mouse_drag.is_none());
    Ok(())
}
