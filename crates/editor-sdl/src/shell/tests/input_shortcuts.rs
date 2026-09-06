#![allow(unused_imports)]
use super::*;

#[test]
fn keydown_chord_maps_ctrl_tab() {
    assert_eq!(
        keydown_chord(Keycode::Tab, ctrl_mod()).as_deref(),
        Some("Ctrl+Tab")
    );
}

#[test]
fn calculator_ctrl_tab_switches_sections_without_changing_workspace_pane() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(
        &mut state,
        user::calculator::BUFFER_NAME,
        user::calculator::CALCULATOR_KIND,
    )?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    let active_pane_id = shell_ui(&state.runtime)?
        .active_pane_id()
        .ok_or_else(|| "active pane is missing".to_owned())?;

    let handled = state
        .try_runtime_keybinding(Keycode::Tab, ctrl_mod())
        .map_err(|error| error.to_string())?;

    assert!(handled);
    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(active_pane_id)
    );
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.plugin_active_section_index(), Some(1));
    assert!(buffer.is_read_only());
    Ok(())
}

#[test]
fn acp_input_field_dd_deletes_current_line() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta\ngamma")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char("alpha\n".chars().count());
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "alpha\ngamma");
    assert_eq!(ui.vim().yank, Some(YankRegister::Line("beta\n".to_owned())));
    Ok(())
}

#[test]
fn acp_input_field_dw_deletes_motion_range() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha beta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("w")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "beta");
    assert_eq!(
        ui.vim().yank,
        Some(YankRegister::Character("alpha ".to_owned()))
    );
    Ok(())
}

#[test]
fn acp_input_field_cw_enters_insert_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha beta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("w")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(input.text(), "beta");

    state
        .handle_text_input("zeta ")
        .map_err(|error| error.to_string())?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "zeta beta");
    Ok(())
}

#[test]
fn acp_input_field_o_and_o_open_new_lines() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char("alpha\n".chars().count());
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    state
        .handle_text_input("middle")
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("O")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    state
        .handle_text_input("above")
        .map_err(|error| error.to_string())?;

    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "alpha\nbeta\nabove\nmiddle");
    Ok(())
}

#[test]
fn acp_input_field_yy_and_p_work_linewise() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("p")
        .map_err(|error| error.to_string())?;

    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "alpha\nalpha\nbeta");
    Ok(())
}

#[test]
fn ctrl_enter_variants_match_manual_lsp_code_action_command() -> Result<(), String> {
    let root = unique_temp_dir("lsp-code-action-binding");
    let path = root.join("main.rs");
    fs::write(
        &path,
        "fn main() {\n    let value = 1;\n    let _ = value;\n}\n",
    )
    .map_err(|error| error.to_string())?;

    let manual_title = {
        let mut state = state_with_user_library()?;
        open_workspace_from_project(&mut state.runtime, "lsp-code-actions-manual", &root)?;
        open_workspace_file(&mut state.runtime, &path)?;
        shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
        state
            .runtime
            .execute_command("lsp.code-action")
            .map_err(|error| error.to_string())?;
        shell_ui(&state.runtime)?
            .picker()
            .map(|picker| picker.session.title().to_owned())
            .ok_or_else(|| "manual lsp code-action did not open a picker".to_owned())?
    };

    for (name, keycode) in [
        ("return", Keycode::Return),
        ("kp-enter", Keycode::KpEnter),
        ("return2", Keycode::Return2),
    ] {
        let mut state = state_with_user_library()?;
        open_workspace_from_project(
            &mut state.runtime,
            &format!("lsp-code-actions-binding-{name}"),
            &root,
        )?;
        open_workspace_file(&mut state.runtime, &path)?;
        shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

        let binding = state
            .runtime
            .keymaps()
            .get_for_mode(
                &editor_core::KeymapScope::Workspace,
                editor_core::KeymapVimMode::Normal,
                "Ctrl+Enter",
            )
            .ok_or_else(|| "Ctrl+Enter workspace binding is missing".to_owned())?;
        assert_eq!(binding.command_name(), "lsp.code-actions");

        let (render_width, render_height, cell_width, line_height) =
            markdown_table_event_dimensions();
        let handled = state
            .handle_event(
                Event::KeyDown {
                    timestamp: 0,
                    window_id: 0,
                    keycode: Some(keycode),
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
        let binding_title = shell_ui(&state.runtime)?
            .picker()
            .map(|picker| picker.session.title().to_owned())
            .ok_or_else(|| format!("Ctrl+Enter variant `{name}` did not open an LSP picker"))?;
        assert_eq!(binding_title, manual_title);
    }

    fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_dashboard_provider_extras_copy_ctrl_d_onto_instance() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-ctrl-d-extra-main")?;
    open_workspace_from_project(&mut state.runtime, "dashboard-ctrl-d-extra", &main)?;

    let overlay = picker::picker_overlay(&state.runtime, "workspace.dashboard")?;
    assert!(
        overlay.extra_keybinds().iter().any(|binding| {
            binding.chord() == "Ctrl+d" && binding.command_name() == "workspace.worktree-remove"
        }),
        "workspace.dashboard provider extras should land on the open picker instance"
    );

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn workspace_dashboard_ctrl_d_on_create_row_is_silent_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-ctrl-d-create-noop-main")?;
    open_workspace_from_project(&mut state.runtime, "main", &main)?;

    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_create_row(&mut state.runtime)?;
    let before = shell_ui(&state.runtime)?.notification_revision();

    let handled = state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled, "Ctrl+d extra should still fire and close picker");
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);
    assert!(main.exists(), "primary worktree must stay on disk");

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn workspace_dashboard_ctrl_d_second_remove_while_first_runs() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-ctrl-d-concurrent-main")?;
    let first = add_linked_worktree(&main, "dashboard-ctrl-d-concurrent-a", "feature-a")?;
    let second = add_linked_worktree(&main, "dashboard-ctrl-d-concurrent-b", "feature-b")?;
    open_workspace_from_project(&mut state.runtime, "main", &main)?;

    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &first)?;
    state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    let first_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "first Worktree Remove popup missing".to_owned())?;
    let first_buffer = first_popup.active_buffer;

    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &second)?;
    state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    let second_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "second Worktree Remove popup missing".to_owned())?;
    let second_buffer = second_popup.active_buffer;

    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert_ne!(first_buffer, second_buffer);
    assert!(
        shell_ui(&state.runtime)?.buffer(first_buffer).is_some()
            || shell_ui(&state.runtime)?
                .streamed_command_worker
                .contains(first_buffer),
        "first remove buffer should still exist or still be tracked"
    );

    wait_for_streamed_command_buffer_close(&mut state, first_buffer)?;
    wait_for_streamed_command_buffer_close(&mut state, second_buffer)?;

    let _ = std::fs::remove_dir_all(&main);
    let _ = std::fs::remove_dir_all(&first);
    let _ = std::fs::remove_dir_all(&second);
    Ok(())
}

#[test]
fn input_field_paste_shortcut_requested_recognizes_ctrl_shift_v_only() {
    assert!(input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod() | shift_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        shift_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod() | shift_mod() | Mod::LALTMOD
    ));
}

#[test]
fn workspace_dock_ctrl_h_enters_focus_from_panes_when_left_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_LEFT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(
        shell_ui(&state.runtime)?.workspace_dock_focus_active(&*shell_user_library(&state.runtime))
    );
    Ok(())
}

#[test]
fn workspace_dock_ctrl_l_exits_focus_back_to_panes_when_left_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    shell_ui_mut(&mut state.runtime)?.set_workspace_dock_focus(true);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_RIGHT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());
    Ok(())
}

#[test]
fn workspace_dock_ctrl_l_enters_focus_when_right_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Right,
        docked: true,
    })?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_RIGHT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(
        shell_ui(&state.runtime)?.workspace_dock_focus_active(&*shell_user_library(&state.runtime))
    );
    Ok(())
}

#[test]
fn acp_dock_ctrl_l_enters_focus_when_open() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    toggle_acp_dock(&mut state.runtime)?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_focus());

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_RIGHT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(shell_ui(&state.runtime)?.acp_dock_focus_active());
    Ok(())
}

#[test]
fn acp_dock_ctrl_h_exits_focus_back_to_panes() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    toggle_acp_dock(&mut state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.set_acp_dock_focus(true);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_LEFT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_focus());
    Ok(())
}
