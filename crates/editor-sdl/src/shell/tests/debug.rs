use super::*;

#[test]
fn debug_layout_installs_three_panes_and_disables_golden_ratio() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    install_debug_layout(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert!(ui.is_debug_layout_active());
    assert_eq!(view.golden_ratio_override, Some(false));
    assert_eq!(
        view.pane_size_weights.as_deref(),
        Some(
            [
                DEBUG_LAYOUT_BREAKPOINTS_WEIGHT,
                DEBUG_LAYOUT_EDITOR_WEIGHT,
                DEBUG_LAYOUT_LOCALS_WEIGHT
            ]
            .as_slice()
        )
    );
    assert_eq!(view.panes.len(), 3);
    assert_eq!(view.split_direction, Some(PaneSplitDirection::Vertical));
    let left = ui
        .buffer(view.panes[0].buffer_id)
        .ok_or_else(|| "breakpoints pane missing".to_owned())?;
    let right = ui
        .buffer(view.panes[2].buffer_id)
        .ok_or_else(|| "locals pane missing".to_owned())?;
    assert!(matches!(
        &left.kind,
        BufferKind::Plugin(kind) if kind == DAP_BREAKPOINTS_KIND
    ));
    assert!(matches!(
        &right.kind,
        BufferKind::Plugin(kind) if kind == DAP_LOCALS_KIND
    ));
    assert!(
        right.plugin_section_state.is_some(),
        "locals pane should expose Locals/Expressions sections"
    );
    let rects = workspace_pane_rects(&*shell_user_library(&state.runtime), ui, 600, 200, 3);
    assert_eq!(rects.len(), 3);
    assert!(
        rects[0].width < rects[1].width && rects[2].width < rects[1].width,
        "editor pane should be widest: {rects:?}"
    );
    Ok(())
}

#[test]
fn debug_layout_blocks_user_splits() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    install_debug_layout(&mut state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 3);
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    assert_eq!(
        shell_ui(&state.runtime)?.pane_count(),
        3,
        "user splits must be blocked while Debug Layout is active"
    );
    Ok(())
}

#[test]
fn debug_layout_teardown_restores_golden_ratio() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    assert!(
        shell_user_library(&state.runtime)
            .pane_config()
            .golden_ratio
    );
    install_debug_layout(&mut state.runtime)?;
    teardown_debug_layout(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert!(!ui.is_debug_layout_active());
    assert_eq!(view.golden_ratio_override, None);
    assert!(view.pane_size_weights.is_none());
    assert_eq!(view.panes.len(), 1);
    Ok(())
}

#[test]
fn dap_start_installs_debug_layout_and_stop_restores() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-layout-start-stop");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 3);

    stop_dap_for_active_workspace(&mut state.runtime)?;
    assert!(!shell_ui(&state.runtime)?.is_debug_layout_active());
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_start_saves_dirty_workspace_files_before_launch() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-start-saves-dirty");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let buffer_id = open_workspace_file(&mut state.runtime, &program)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// dirty\n");
        assert!(buffer.is_dirty());
    }
    assert_eq!(
        fs::read_to_string(&program).map_err(|e| e.to_string())?,
        "fn main() {}\n",
        "disk must stay stale until dap.start"
    );

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;

    assert_eq!(
        fs::read_to_string(&program).map_err(|e| e.to_string())?,
        "// dirty\nfn main() {}\n",
        "dap.start must save dirty Workspace files before compile-before-debug / launch"
    );
    assert!(
        !shell_buffer(&state.runtime, buffer_id)?.is_dirty(),
        "saved buffer must clear dirty"
    );
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_stopped_jumps_to_source_refreshes_locals_and_steps() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-step-jump-locals");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(
        &program,
        "fn main() {\n    let x = 1;\n    let y = 2;\n    let z = 3;\n}\n",
    )
    .map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    toggle_dap_breakpoint_at_cursor(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    {
        let dap = state
            .runtime
            .services()
            .get::<Arc<DapClientManager>>()
            .ok_or_else(|| "dap manager missing".to_owned())?;
        assert!(
            dap.session_info(workspace_id.get())
                .map_err(|e| e.to_string())?
                .is_some(),
            "session must be live after start"
        );
    }

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DAP stopped UI".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }

    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert_eq!(view.active_pane, 1, "center editor pane should be focused");
    let editor = ui
        .buffer(view.panes[1].buffer_id)
        .ok_or_else(|| "center editor missing".to_owned())?;
    assert_eq!(editor.cursor_row(), 0, "should jump to stop line 1");
    assert!(
        editor.dap_fringe_live(),
        "live Session must widen Debug Fringe in the center pane"
    );
    assert_eq!(editor.dap_execution_line(), Some(0));
    assert!(
        editor.dap_fringe_marker(0).is_some(),
        "Breakpoint on the stopped line must stay in the Debug Fringe"
    );
    let locals = ui
        .buffer(view.panes[2].buffer_id)
        .ok_or_else(|| "locals pane missing".to_owned())?;
    let locals_text = locals.text.text();
    assert!(
        locals_text.contains("x: 42"),
        "locals should refresh on stop: {locals_text}"
    );
    assert!(
        locals_text.contains(&format!("{DAP_VAR_COLLAPSED_GLYPH} person:")),
        "structured Locals must show a collapsed chevron: {locals_text}"
    );
    let breakpoints = ui
        .buffer(view.panes[0].buffer_id)
        .ok_or_else(|| "breakpoints pane missing".to_owned())?;
    let bp_text = breakpoints.text.text();
    assert!(
        bp_text.contains("main.rs:1"),
        "Breakpoints pane should list the source line: {bp_text}"
    );
    assert!(
        !bp_text.contains("Breakpoints:"),
        "Breakpoints pane should not repeat the title as a header: {bp_text}"
    );

    dap_control_for_active_workspace(&mut state.runtime, DapControl::StepOver)?;
    {
        let ui = shell_ui(&state.runtime)?;
        let editor = ui
            .buffer(
                ui.workspace_view()
                    .ok_or_else(|| "view missing".to_owned())?
                    .panes[1]
                    .buffer_id,
            )
            .ok_or_else(|| "center editor missing".to_owned())?;
        assert_eq!(
            editor.dap_execution_line(),
            None,
            "step must clear the execution highlight until the next stop"
        );
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for step stop".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }
    let ui = shell_ui(&state.runtime)?;
    let editor = ui
        .buffer(
            ui.workspace_view()
                .ok_or_else(|| "view missing".to_owned())?
                .panes[1]
                .buffer_id,
        )
        .ok_or_else(|| "center editor missing".to_owned())?;
    assert_eq!(editor.cursor_row(), 1);
    assert_eq!(editor.dap_execution_line(), Some(1));

    restart_dap_for_active_workspace(&mut state.runtime)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for restart stop".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }
    let ui = shell_ui(&state.runtime)?;
    let editor = ui
        .buffer(
            ui.workspace_view()
                .ok_or_else(|| "view missing".to_owned())?
                .panes[1]
                .buffer_id,
        )
        .ok_or_else(|| "center editor missing".to_owned())?;
    assert_eq!(editor.cursor_row(), 0);
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_locals_and_watches_expand_structured_variables() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-expand-vars");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {\n    let person = Person;\n}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DAP stopped UI".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }

    focus_debug_layout_pane(&mut state.runtime, 2)?;
    {
        let buffer = active_shell_buffer_mut(&mut state.runtime)?;
        buffer.plugin_focus_section_named(DAP_LOCALS_SECTION);
        let line = (0..buffer.line_count())
            .find(|&index| {
                buffer
                    .text
                    .line(index)
                    .is_some_and(|line| line.contains("person:"))
            })
            .ok_or_else(|| "person Locals row missing".to_owned())?;
        buffer.set_cursor(TextPoint::new(line, 0));
    }
    state
        .runtime
        .execute_command("dap.toggle-variable")
        .map_err(|error| error.to_string())?;

    let locals_id = shell_ui(&state.runtime)?
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?
        .panes[2]
        .buffer_id;
    let locals_text = {
        let locals = shell_buffer(&state.runtime, locals_id)?;
        plugin_section_lines(locals, DAP_LOCALS_SECTION)?.join("\n")
    };
    assert!(
        locals_text.contains(DAP_WATCHES_HEADER),
        "Locals section must keep Watch Expressions header: {locals_text}"
    );
    assert!(
        locals_text.contains(&format!("{DAP_VAR_EXPANDED_GLYPH} person:")),
        "person should expand: {locals_text}"
    );
    assert!(
        locals_text.contains("Name:") && locals_text.contains("Address:"),
        "expanded person must show members: {locals_text}"
    );

    add_dap_expression(&mut state.runtime, "person")?;
    {
        let buffer = active_shell_buffer_mut(&mut state.runtime)?;
        buffer.plugin_focus_section_named(DAP_EXPRESSIONS_SECTION);
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    state
        .runtime
        .execute_command("dap.toggle-variable")
        .map_err(|error| error.to_string())?;
    let watch_text = {
        let locals = shell_buffer(&state.runtime, locals_id)?;
        plugin_section_lines(locals, DAP_EXPRESSIONS_SECTION)?.join("\n")
    };
    assert!(
        watch_text.contains(&format!("{DAP_VAR_EXPANDED_GLYPH} person:")),
        "Watch Expression should expand: {watch_text}"
    );
    assert!(
        watch_text.contains("Name:"),
        "expanded watch must show members: {watch_text}"
    );

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_locals_insert_watch_expression_evaluates_while_stopped() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-insert-watch");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {\n    let x = 1;\n}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DAP stopped UI".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }

    focus_debug_layout_pane(&mut state.runtime, 2)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    {
        let buffer = active_shell_buffer_mut(&mut state.runtime)?;
        buffer.plugin_focus_section_named(DAP_LOCALS_SECTION);
        let mut lines: Vec<String> = (0..buffer.line_count())
            .filter_map(|index| buffer.text.line(index))
            .collect();
        let header = lines
            .iter()
            .position(|line| line == DAP_WATCHES_HEADER)
            .ok_or_else(|| format!("Watch Expressions header missing: {lines:?}"))?;
        lines.insert(header + 1, "x".to_owned());
        buffer.replace_with_lines_preserve_view(lines);
    }
    apply_dap_locals_edits(&mut state.runtime, buffer_id)?;

    let locals_text = {
        let locals = shell_buffer(&state.runtime, buffer_id)?;
        plugin_section_lines(locals, DAP_LOCALS_SECTION)?.join("\n")
    };
    assert!(
        locals_text.contains(DAP_WATCHES_HEADER),
        "header must remain: {locals_text}"
    );
    assert!(
        locals_text.lines().any(|line| line.contains("x@")),
        "inserted Watch Expression must evaluate while stopped: {locals_text}"
    );
    let watch_text = {
        let locals = shell_buffer(&state.runtime, buffer_id)?;
        plugin_section_lines(locals, DAP_EXPRESSIONS_SECTION)?.join("\n")
    };
    assert!(
        watch_text.contains("x@"),
        "Expressions section must mirror the new watch: {watch_text}"
    );

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_continue_to_exit_tears_down_debug_layout() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-continue-exit");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("exit-on-continue.rs");
    fs::write(&program, "fn main() {\n    let x = 1;\n}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DAP stopped UI".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());

    dap_control_for_active_workspace(&mut state.runtime, DapControl::Continue)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let _ = refresh_pending_dap(&mut state.runtime)?;
        if !shell_ui(&state.runtime)?.is_debug_layout_active() {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for Debug Stop cleanup after continue".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    {
        let dap = state
            .runtime
            .services()
            .get::<Arc<DapClientManager>>()
            .ok_or_else(|| "dap manager missing".to_owned())?;
        assert!(
            dap.session_info(workspace_id.get())
                .map_err(|e| e.to_string())?
                .is_none(),
            "Session must end after process exit"
        );
    }
    let ui = shell_ui(&state.runtime)?;
    let editor = ui
        .buffer(
            ui.workspace_view()
                .ok_or_else(|| "view missing".to_owned())?
                .panes[0]
                .buffer_id,
        )
        .ok_or_else(|| "editor missing".to_owned())?;
    assert!(
        !editor.dap_fringe_live(),
        "Debug Fringe must not stay live after Debug Stop cleanup"
    );

    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_mode_function_keys_continue_and_step() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let idle_modes = state.overlay_minor_modes().map_err(|e| e.to_string())?;
    assert!(
        !idle_modes.contains(&KeymapScope::Dap),
        "DAP Mode must stay off without a Session"
    );
    let start = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&idle_modes, KeymapVimMode::Any, "F5")
        .ok_or_else(|| "expected Global F5".to_owned())?;
    assert_eq!(start.command_name(), "dap.start");

    let root = unique_temp_dir("dap-mode-keys");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;
    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;

    let live_modes = state.overlay_minor_modes().map_err(|e| e.to_string())?;
    assert!(
        live_modes.contains(&KeymapScope::Dap),
        "live Session must activate DAP Mode: {live_modes:?}"
    );
    let continue_binding = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&live_modes, KeymapVimMode::Any, "F5")
        .ok_or_else(|| "expected DAP F5".to_owned())?;
    assert_eq!(continue_binding.command_name(), "dap.continue");
    let step = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&live_modes, KeymapVimMode::Any, "F10")
        .ok_or_else(|| "expected DAP F10".to_owned())?;
    assert_eq!(step.command_name(), "dap.step");
    let into = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&live_modes, KeymapVimMode::Any, "F11")
        .ok_or_else(|| "expected DAP F11".to_owned())?;
    assert_eq!(into.command_name(), "dap.step-into");

    let toggle = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&[KeymapScope::Workspace], KeymapVimMode::Any, "Space d a")
        .ok_or_else(|| "expected <leader> da".to_owned())?;
    assert_eq!(toggle.command_name(), "dap.toggle-breakpoint");

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_watches_eval_repl_switch_and_breakpoint_extras() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-polish");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() { let x = 1; }\n").map_err(|e| e.to_string())?;
    let buffer_id = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for stop".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }

    add_dap_expression(&mut state.runtime, "x")?;
    let workspace_id = workspace.get();
    let (locals, expressions) = dap_locals_and_expression_lines(&state.runtime, workspace_id)?;
    assert!(
        locals.iter().any(|line| line == DAP_WATCHES_HEADER),
        "locals must keep Watch Expressions header: {locals:?}"
    );
    assert!(
        locals.iter().any(|line| line.contains("x")),
        "locals rows: {locals:?}"
    );
    assert!(
        expressions.iter().any(|line| line.contains("x:")),
        "expression rows: {expressions:?}"
    );

    show_dap_eval_result(&mut state.runtime, "y", DapEvaluateContext::Repl)?;
    open_dap_repl(&mut state.runtime)?;
    submit_dap_repl_expression(&mut state.runtime, "z")?;
    assert!(
        shell_ui(&state.runtime)?
            .input_prompt()
            .is_some_and(|prompt| prompt.id == DAP_REPL_PROMPT_ID),
        "REPL should reopen prompt"
    );

    switch_dap_thread(&mut state.runtime, 2)?;
    switch_dap_stack_frame(&mut state.runtime, 2)?;

    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.focus_buffer_in_active_pane(buffer_id);
        if let Some(buffer) = ui.buffer_mut(buffer_id) {
            buffer.text.set_cursor(TextPoint::new(0, 0));
        }
    }
    apply_dap_breakpoint_extra(
        &mut state.runtime,
        DapBreakpointExtraKind::Condition,
        "x > 0",
    )?;
    apply_dap_breakpoint_extra(
        &mut state.runtime,
        DapBreakpointExtraKind::HitCondition,
        "2",
    )?;
    apply_dap_breakpoint_extra(
        &mut state.runtime,
        DapBreakpointExtraKind::LogMessage,
        "hit",
    )?;
    let bps = dap_client_manager(&state.runtime)?
        .list_breakpoints(workspace_id)
        .map_err(|e| e.to_string())?;
    let bp = bps
        .iter()
        .find(|bp| bp.line() == 1)
        .ok_or_else(|| "breakpoint missing".to_owned())?;
    assert_eq!(bp.condition(), Some("x > 0"));
    assert_eq!(bp.hit_condition(), Some("2"));
    assert_eq!(bp.log_message(), Some("hit"));

    open_dap_log_buffer(&mut state.runtime)?;
    remove_dap_expression(&mut state.runtime, "x")?;

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn debug_layout_hides_on_workspace_switch_and_rebuilds_on_return() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("dap-layout-ws-a");
    let second_root = unique_temp_dir("dap-layout-ws-b");
    let first = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let program = first_root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());

    let second = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    assert_ne!(first, second);
    assert!(
        !shell_ui(&state.runtime)?.is_debug_layout_active(),
        "leaving Workspace must tear down Debug Layout"
    );
    let dap = state
        .runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .ok_or_else(|| "dap manager missing".to_owned())?;
    assert!(
        dap.session_info(first.get())
            .map_err(|e| e.to_string())?
            .is_some(),
        "Debug Session must survive Workspace switch"
    );

    switch_runtime_workspace(&mut state.runtime, first)?;
    assert!(
        shell_ui(&state.runtime)?.is_debug_layout_active(),
        "returning to Workspace with live Session must rebuild Debug Layout"
    );
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 3);

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = fs::remove_dir_all(&first_root);
    let _ = fs::remove_dir_all(&second_root);
    Ok(())
}

#[test]
fn dap_start_opens_adapter_picker_ordered_by_preference() -> Result<(), String> {
    use editor_dap::{DebugAdapterRegistry, DebugAdapterSpec, DebugAdapterTransport};

    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-adapter-picker");
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let mut registry = DebugAdapterRegistry::new();
    registry
        .register(
            DebugAdapterSpec::new("gdb-fake", "rust", ["rs"], "", [] as [&str; 0])
                .with_transport(DebugAdapterTransport::Tcp {
                    host: "127.0.0.1".to_owned(),
                    port: 1,
                })
                .with_preference(50),
        )
        .map_err(|e| e.to_string())?;
    registry
        .register(
            DebugAdapterSpec::new("codelldb-fake", "rust", ["rs"], "", [] as [&str; 0])
                .with_transport(DebugAdapterTransport::Tcp {
                    host: "127.0.0.1".to_owned(),
                    port: 2,
                })
                .with_preference(100),
        )
        .map_err(|e| e.to_string())?;
    state
        .runtime
        .services_mut()
        .insert(Arc::new(DapClientManager::new(registry)));

    start_dap_for_active_workspace(&mut state.runtime, None)?;
    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "expected Debug Adapter picker".to_owned())?;
    assert_eq!(picker.session().title(), "Choose Debug Adapter");
    let ids: Vec<_> = picker
        .session()
        .matches()
        .iter()
        .map(|entry| entry.item().label().to_owned())
        .collect();
    assert_eq!(ids, ["codelldb-fake", "gdb-fake"]);
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_start_lists_project_configurations_in_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-project-configs");
    let volt_dir = root.join(".volt");
    fs::create_dir_all(&volt_dir).map_err(|e| e.to_string())?;
    fs::write(
        volt_dir.join("debug.json"),
        r#"{
          "configurations": [
            {
              "name": "Project Launch",
              "adapter": "fake-dap",
              "request": "launch",
              "program": "main.rs"
            }
          ]
        }"#,
    )
    .map_err(|e| e.to_string())?;
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "expected Debug Configuration picker".to_owned())?;
    assert_eq!(picker.session().title(), "Choose Debug Configuration");
    let labels: Vec<_> = picker
        .session()
        .matches()
        .iter()
        .map(|entry| entry.item().label().to_owned())
        .collect();
    assert!(
        labels.iter().any(|label| label.contains("Project Launch")),
        "project config missing from {labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label.contains("Debug (current file)")),
        "inferred/compiled default missing from {labels:?}"
    );
    let _ = fake.join();
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_start_last_replays_prior_configuration() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-start-last");
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    stop_dap_for_active_workspace(&mut state.runtime)?;

    start_dap_last(&mut state.runtime)?;
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());
    let dap = state
        .runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .ok_or_else(|| "dap manager missing".to_owned())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    assert!(
        dap.session_info(workspace_id.get())
            .map_err(|e| e.to_string())?
            .is_some()
    );
    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_default_workspace_skips_deep_inference() -> Result<(), String> {
    let state = state_with_user_library()?;
    let ctx = dap_start_context(&state.runtime)?;
    assert!(
        !ctx.allow_deep_inference,
        "Default Workspace must not deep-infer Debug Configurations"
    );
    Ok(())
}
