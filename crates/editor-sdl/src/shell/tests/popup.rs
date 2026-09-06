use super::*;

#[test]
fn render_shell_state_uses_theme_background_for_docked_runtime_popup_surface() -> Result<(), String>
{
    let base_background = Color::RGB(15, 16, 20);
    let (scene, popup_rect) = render_shell_state_scene_with_docked_runtime_popup(None)?;
    let popup_surface_fills = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::FillRoundedRect { rect, color, .. }
                if rect.x == popup_rect.x()
                    && rect.y == popup_rect.y()
                    && rect.width == popup_rect.width()
                    && rect.height == popup_rect.height() =>
            {
                Some(*color)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(popup_surface_fills, vec![to_render_color(base_background)]);
    Ok(())
}

#[test]
fn calculator_switch_pane_command_targets_workspace_buffer_when_popup_has_focus()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(
        &mut state,
        user::calculator::BUFFER_NAME,
        user::calculator::CALCULATOR_KIND,
    )?;
    let _popup_buffer_id = install_terminal_popup_test_buffer(&mut state)?;

    state
        .runtime
        .execute_command("calculator.switch-pane")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.plugin_active_section_index(), Some(1));
    assert!(buffer.is_read_only());
    Ok(())
}

#[test]
fn ctrl_q_with_workspace_search_picker_exports_quickfix_instead_of_quitting() -> Result<(), String>
{
    let (mut state, _root, _first, _second) =
        prepare_quickfix_workspace_search_picker("quickfix-ctrl-q-export")?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
                repeat: false,
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
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "Ctrl+Q did not open quickfix popup".to_owned())?;
    assert_eq!(
        shell_ui(&state.runtime)?.popup_buffer_id,
        Some(popup.active_buffer)
    );
    assert!(shell_ui(&state.runtime)?.popup_focus);
    let buffer = shell_buffer(&state.runtime, popup.active_buffer)?;
    assert!(buffer_is_quickfix(&buffer.kind));
    let first_line = buffer
        .text
        .line(0)
        .ok_or_else(|| "quickfix first line missing".to_owned())?;
    assert!(first_line.contains("main.rs:1:4 | fn alpha() {}"));
    assert!(first_line.contains("[ ] "));
    let second_line = buffer
        .text
        .line(1)
        .ok_or_else(|| "quickfix second line missing".to_owned())?;
    assert!(second_line.contains("lib.rs:1:4 | fn beta() {}"));
    assert!(second_line.contains("[ ] "));
    Ok(())
}

#[test]
fn ctrl_q_with_non_quickfix_picker_does_not_quit() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Buffers",
        vec![PickerEntry {
            item: PickerItem::new("buffer:alpha", "alpha", "scratch", None::<&str>),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    ));
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
                repeat: false,
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
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    Ok(())
}

#[test]
fn ctrl_q_without_quickfix_extra_does_not_export_popup_global() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("quickfix-ctrl-q-no-extra");
    let first = root.join("src").join("main.rs");
    std::fs::create_dir_all(first.parent().ok_or_else(|| "missing src dir".to_owned())?)
        .map_err(|error| error.to_string())?;
    std::fs::write(&first, "fn alpha() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "quickfix-ctrl-q-no-extra", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Search",
        vec![workspace_search::workspace_search_match_entry(
            &root,
            "src/main.rs",
            1,
            4,
            "fn alpha() {}",
        )],
    ));
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
                repeat: false,
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
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    Ok(())
}

#[test]
fn popup_buffer_escape_enters_normal_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let popup_buffer = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup-escape*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![popup_buffer], popup_buffer)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(&state.runtime);
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.ensure_popup_buffer(
            popup_buffer,
            "*popup-escape*",
            BufferKind::Scratch,
            &*user_library,
        );
        ui.set_popup_buffer(popup_buffer);
        ui.set_popup_focus(true);
        ui.enter_insert_mode();
    }

    let handled = state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    assert!(handled, "Escape must be handled in a focused popup");
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn picker_extra_keybind_falls_through_for_shared_popup_navigation() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Buffers",
            vec![
                PickerEntry {
                    item: PickerItem::new("a", "alpha", "scratch", None::<&str>),
                    action: PickerAction::NoOp,
                    quickfix: None,
                },
                PickerEntry {
                    item: PickerItem::new("b", "beta", "scratch", None::<&str>),
                    action: PickerAction::NoOp,
                    quickfix: None,
                },
            ],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new(
            "Ctrl+d",
            "tests.picker-extra",
        )]),
    );

    let handled = state
        .try_runtime_keybinding(Keycode::N, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    let selected = shell_ui(&state.runtime)?
        .picker()
        .and_then(|picker| picker.session().selected())
        .map(|matched| matched.item().id().to_owned());
    assert_eq!(selected.as_deref(), Some("b"));
    Ok(())
}

#[test]
#[ignore = "enable once quickfix picker export command lands"]
fn quickfix_picker_export_opens_popup_and_renders_workspace_search_results() -> Result<(), String> {
    let (state, root, first, second) =
        prepare_quickfix_workspace_search_picker("quickfix-export-popup")?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "picker missing before quickfix export".to_owned())?;
    assert_eq!(picker.session().matches().len(), 2);
    let _ = (first, second);
    let _ = active_runtime_popup(&state.runtime)?;
    let _ = shell_ui(&state.runtime)?.popup_buffer_id;
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = root;
    unimplemented!("invoke quickfix export and assert popup buffer renders exported rows");
}

#[test]
#[ignore = "enable once quickfix enter handler lands"]
fn quickfix_enter_opens_target_and_moves_focus_back_to_workspace() -> Result<(), String> {
    let (state, root, first, _) = prepare_quickfix_workspace_search_picker("quickfix-enter-focus")?;

    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let _ = (root, first);
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = original_buffer_id;
    unimplemented!(
        "export picker to quickfix, press Enter on quickfix row, assert workspace focus"
    );
}

#[test]
#[ignore = "enable once quickfix next and previous commands land"]
fn quickfix_next_previous_wraparound_tracks_current_list() -> Result<(), String> {
    let (state, root, first, second) =
        prepare_quickfix_workspace_search_picker("quickfix-wraparound")?;

    let _ = active_shell_buffer_id(&state.runtime)?;
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = (root, first, second);
    unimplemented!("export picker, drive quickfix.next/previous, assert wraparound navigation");
}

#[test]
#[ignore = "enable once quickfix export command lands"]
fn quickfix_export_from_unsupported_picker_is_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Buffers",
        vec![PickerEntry {
            item: PickerItem::new("buffer:alpha", "alpha", "scratch", None::<&str>),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    ));

    let _ = original_buffer_id;
    let _ = active_runtime_popup(&state.runtime)?;
    unimplemented!("invoke quickfix export and assert picker closes or no-ops without popup");
}

#[test]
fn leave_open_keeps_popup_buffer_after_process_exits() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = open_streamed_command_popup(
        &mut state.runtime,
        StreamedCommandSpec {
            popup_title: "Leave Open Test".to_owned(),
            buffer_name: "*leave-open-test*".to_owned(),
            command_label: "true".to_owned(),
            #[cfg(unix)]
            program: "true".to_owned(),
            #[cfg(windows)]
            program: "cmd".to_owned(),
            #[cfg(unix)]
            args: vec![],
            #[cfg(windows)]
            args: vec!["/C".to_owned(), "exit 0".to_owned()],
            env: Vec::new(),
            cwd: std::env::temp_dir(),
            on_exit: StreamedCommandExitAction::LeaveOpen,
            notify_on_success: false,
            notify_on_failure: false,
        },
    )?;

    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;

    let ui = shell_ui(&state.runtime)?;
    assert!(
        ui.buffer(buffer_id).is_some(),
        "LeaveOpen: popup buffer must remain open after process exits"
    );
    assert!(
        !ui.streamed_command_worker.contains(buffer_id),
        "LeaveOpen: worker should be done"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Cancel flag: closing a popup buffer kills its worker
// ---------------------------------------------------------------------------

#[test]
fn closing_streamed_command_popup_kills_worker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = open_streamed_command_popup(
        &mut state.runtime,
        StreamedCommandSpec {
            popup_title: "Cancel Test".to_owned(),
            buffer_name: "*cancel-test*".to_owned(),
            command_label: "sleep".to_owned(),
            #[cfg(unix)]
            program: "sleep".to_owned(),
            #[cfg(windows)]
            program: "cmd".to_owned(),
            #[cfg(unix)]
            args: vec!["60".to_owned()],
            #[cfg(windows)]
            args: vec!["/C".to_owned(), "timeout /T 60 /NOBREAK".to_owned()],
            env: Vec::new(),
            cwd: std::env::temp_dir(),
            on_exit: StreamedCommandExitAction::LeaveOpen,
            notify_on_success: false,
            notify_on_failure: false,
        },
    )?;

    assert!(
        shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "worker should be active before close"
    );

    close_buffer_immediate(&mut state.runtime, buffer_id).map_err(|error| error.to_string())?;

    assert!(
        !shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "worker should be removed immediately after buffer close"
    );

    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;
    Ok(())
}

#[test]
fn workspace_compile_closing_popup_mid_build_stops_tracking_worker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-close-popup");
    open_workspace_from_project(&mut state.runtime, "compile-close-popup", &root)?;

    let buffer_id = start_workspace_compile(
        &mut state,
        &shell_sleep_then_echo_command(60, "compile-stop"),
    )?;
    assert!(
        shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "compile worker should be tracked before popup close"
    );

    close_buffer_immediate(&mut state.runtime, buffer_id).map_err(|error| error.to_string())?;
    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;
    assert!(
        !shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "closing compile popup should stop tracking the worker within the poll timeout"
    );

    std::fs::remove_dir_all(&root).ok();
    Ok(())
}
