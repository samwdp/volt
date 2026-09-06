#![allow(unused_imports)]
use super::*;

#[test]
fn resolve_default_workspace_root_prefers_existing_executable_relative_user_dir() {
    let temp_root = TempTestDir::new("default-workspace-root");
    let exe_dir = temp_root.path().join("target").join("debug").join("deps");
    let bundled_user_dir = temp_root.path().join("target").join("debug").join("user");
    fs::create_dir_all(&exe_dir).expect("create fake executable directory");
    fs::create_dir_all(&bundled_user_dir).expect("create bundled user directory");

    let resolved = resolve_default_workspace_root(Some(&exe_dir.join("volt-tests")), None);
    assert_eq!(resolved, Some(bundled_user_dir));
}

#[test]
fn resolve_default_workspace_root_falls_back_to_executable_user_dir() {
    let temp_root = TempTestDir::new("default-workspace-fallback");
    let exe_dir = temp_root.path().join("bin");
    assert_eq!(
        resolve_default_workspace_root(Some(&exe_dir.join("volt")), Some(temp_root.path())),
        Some(exe_dir.join("user"))
    );
}

#[test]
fn shell_state_uses_default_workspace_root() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    let root = state
        .runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?
        .root()
        .map(Path::to_path_buf);
    assert_eq!(root, default_workspace_root());
    Ok(())
}

#[test]
fn popup_focus_ctrl_n_cycles_popup_buffers_instead_of_marked_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let first = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup-a*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    let second = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup-b*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![first, second], first)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(&state.runtime);
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.ensure_popup_buffer(first, "*popup-a*", BufferKind::Scratch, &*user_library);
        ui.ensure_popup_buffer(second, "*popup-b*", BufferKind::Scratch, &*user_library);
        ui.set_popup_buffer(first);
        ui.set_popup_focus(true);
        ui.enter_normal_mode();
    }

    let handled = state
        .try_runtime_keybinding(Keycode::N, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "popup missing after Ctrl+n".to_owned())?;
    assert_eq!(popup.active_buffer, second);
    assert_eq!(shell_ui(&state.runtime)?.popup_buffer_id, Some(second));
    Ok(())
}

#[test]
fn popup_focus_j_k_do_not_cycle_workspace_dock() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("popup-jk-dock-a");
    let second_root = unique_temp_dir("popup-jk-dock-b");
    let first = open_workspace_from_project(&mut state.runtime, "popup-jk-a", &first_root)?;
    let _second = open_workspace_from_project(&mut state.runtime, "popup-jk-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;

    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let popup_buffer = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![popup_buffer], popup_buffer)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(&state.runtime);
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.ensure_popup_buffer(popup_buffer, "*popup*", BufferKind::Scratch, &*user_library);
        ui.set_popup_buffer(popup_buffer);
        ui.set_popup_focus(true);
        ui.enter_normal_mode();
    }

    let modes = state
        .overlay_minor_modes()
        .map_err(|error| error.to_string())?;
    assert!(
        modes.contains(&KeymapScope::Popup),
        "popup focus must activate Popup Minor Mode: {modes:?}"
    );
    assert!(
        !modes.contains(&KeymapScope::WorkspaceDock),
        "popup focus must not activate Workspace Dock Minor Mode: {modes:?}"
    );
    for chord in ["j", "k"] {
        let overlay = state
            .runtime
            .keymaps()
            .find_in_scopes(&modes, KeymapVimMode::Normal, chord)
            .map(|binding| binding.command_name().to_owned());
        assert_ne!(
            overlay.as_deref(),
            Some("workspace.dock.next"),
            "popup {chord} must not fire workspace dock cycle"
        );
        assert_ne!(
            overlay.as_deref(),
            Some("workspace.dock.previous"),
            "popup {chord} must not fire workspace dock cycle"
        );
    }

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        first,
        "popup j must not cycle the workspace dock"
    );
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        first,
        "popup k must not cycle the workspace dock"
    );
    Ok(())
}

#[test]
fn workspace_dock_focus_j_k_cycle_workspaces() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-jk-a");
    let second_root = unique_temp_dir("workspace-dock-jk-b");
    let first = open_workspace_from_project(&mut state.runtime, "dock-jk-a", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "dock-jk-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_workspace_dock_focus(true);
        ui.enter_normal_mode();
    }

    let modes = state
        .overlay_minor_modes()
        .map_err(|error| error.to_string())?;
    assert!(
        modes.contains(&KeymapScope::WorkspaceDock),
        "dock focus must activate Workspace Dock Minor Mode: {modes:?}"
    );
    assert!(
        !modes.contains(&KeymapScope::Popup),
        "dock focus must not activate Popup Minor Mode: {modes:?}"
    );

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), first);
    Ok(())
}
