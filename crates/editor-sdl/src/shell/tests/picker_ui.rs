use super::*;

#[test]
fn picker_open_file_save_clears_dirty_state_and_closes_cleanly() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("picker-open-file-save");
    let path = root.join("sample.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "picker-save", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Files",
        vec![PickerEntry {
            item: PickerItem::new(
                path.display().to_string(),
                "sample.rs",
                "workspace root",
                Some(path.display().to_string()),
            ),
            action: PickerAction::OpenFile(path.clone()),
            quickfix: None,
        }],
    ));

    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;

    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.path(),
        Some(path.as_path())
    );

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nfn main() {}\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    close_buffer_with_prompt(&mut state.runtime, buffer_id)?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(shell_ui(&state.runtime)?.buffer(buffer_id).is_none());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn picker_open_file_location_save_clears_dirty_state_and_closes_cleanly() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("picker-open-location-save");
    let path = root.join("mod.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "picker-location-save", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Search",
        vec![PickerEntry {
            item: PickerItem::new(
                format!("{}:1:1", path.display()),
                "fn main() {}",
                "mod.rs | Ln 1, Col 1",
                Some(path.display().to_string()),
            ),
            action: PickerAction::OpenFileLocation {
                path: path.clone(),
                target: TextPoint::new(0, 0),
            },
            quickfix: None,
        }],
    ));

    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;

    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.path(),
        Some(path.as_path())
    );

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nfn main() {}\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    close_buffer_with_prompt(&mut state.runtime, buffer_id)?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(shell_ui(&state.runtime)?.buffer(buffer_id).is_none());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn truncate_text_to_width_uses_cell_budget() {
    assert_eq!(truncate_text_to_width("abcdef", 24, 4), "abcdef");
    assert_eq!(truncate_text_to_width("abcdef", 20, 4), "ab...");
    assert_eq!(truncate_text_to_width("abcdef", 8, 4), "...");
}

#[test]
fn truncate_picker_label_shrink_directories_preserves_filename() {
    use editor_plugin_api::PickerTruncateStrategy;

    let path = "src/dir1/dir2/test.rs";
    assert_eq!(
        truncate_picker_label(path, 240, 4, PickerTruncateStrategy::ShrinkDirectories),
        "s/d/d/test.rs"
    );
}

#[test]
fn truncate_picker_label_start_ellipsis_preserves_tail() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label(
            "src/dir1/dir2/test.rs",
            56,
            4,
            PickerTruncateStrategy::StartEllipsis
        ),
        "...ir2/test.rs"
    );
}

#[test]
fn truncate_picker_label_middle_ellipsis_preserves_both_ends() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label(
            "src/dir1/dir2/test.rs",
            56,
            4,
            PickerTruncateStrategy::MiddleEllipsis
        ),
        "src...test.rs"
    );
}

#[test]
fn truncate_picker_label_auto_falls_back_to_start_ellipsis() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label("src/dir1/dir2/test.rs", 56, 4, PickerTruncateStrategy::Auto),
        "...ir2/test.rs"
    );
}

#[test]
fn truncate_picker_label_file_name_with_parent() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label(
            "src/dir1/dir2/test.rs",
            240,
            4,
            PickerTruncateStrategy::FileNameWithParent
        ),
        "dir2/test.rs"
    );
}

#[test]
fn truncate_picker_label_shrink_all_includes_stem() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label(
            "src/dir1/dir2/test.rs",
            240,
            4,
            PickerTruncateStrategy::ShrinkAll
        ),
        "s/d/d/t.rs"
    );
}

#[test]
fn paste_text_into_active_input_buffer_closes_acp_picker_for_multiline_text() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/fix", None)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "ACP Slash Commands",
        Vec::new(),
    ));

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        "\nmore context"
    )?);

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "/fix\nmore context"
    );
    Ok(())
}

#[test]
fn acp_slash_picker_text_input_updates_acp_input() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries("ACP Slash Commands", Vec::new())
            .with_kind(PickerKind::AcpSlash { buffer_id }),
    );

    state
        .handle_text_input("fix")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "/fix"
    );
    Ok(())
}

#[test]
fn acp_slash_picker_backspace_can_delete_leading_slash() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries("ACP Slash Commands", Vec::new())
            .with_kind(PickerKind::AcpSlash { buffer_id }),
    );
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Backspace),
                scancode: None,
                keymod: Mod::NOMOD,
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

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        ""
    );
    Ok(())
}

#[test]
fn f7_keydown_opens_keybinding_picker_from_user_binding() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let binding = state
        .runtime
        .keymaps()
        .get(&editor_core::KeymapScope::Global, "F7")
        .ok_or_else(|| "F7 global binding is missing".to_owned())?;
    assert_eq!(binding.command_name(), "picker.open-keybindings");

    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::F7),
                scancode: None,
                keymod: Mod::NOMOD,
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
    let picker_title = shell_ui(&state.runtime)?
        .picker()
        .map(|picker| picker.session.title().to_owned())
        .ok_or_else(|| "F7 binding did not open the keybinding picker".to_owned())?;
    assert_eq!(picker_title, "Keybindings");
    Ok(())
}

#[test]
fn workspace_dashboard_command_opens_fallback_picker_outside_git() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-dashboard-non-git");
    open_workspace_from_project(&mut state.runtime, "non-git", &root)?;

    state
        .runtime
        .execute_command("workspace.dashboard")
        .map_err(|error| error.to_string())?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "workspace dashboard fallback picker did not open".to_owned())?;
    assert_eq!(picker.session.title(), "Worktrees");
    assert_eq!(picker.session.item_count(), 1);
    assert!(
        picker
            .session
            .selected()
            .is_some_and(|selected| selected.item().label() == "Workspace dashboard unavailable")
    );
    Ok(())
}

#[test]
fn picker_extra_keybind_snapshots_context_closes_and_runs_command() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.picker-extra",
            "Consumes picker one-shot context",
            CommandSource::Core,
            |runtime| {
                let context = shell_ui_mut(runtime)?
                    .take_picker_one_shot()
                    .ok_or_else(|| "picker one-shot missing".to_owned())?;
                let selected = context
                    .selected()
                    .ok_or_else(|| "selected row missing".to_owned())?;
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push(format!(
                    "{}|{}|{}",
                    selected.id(),
                    selected.label(),
                    selected.path().unwrap_or("")
                ));
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;

    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Worktrees",
            vec![PickerEntry {
                item: PickerItem::new(
                    r"P:\repo\feature",
                    "feature",
                    "branch",
                    Some(r"P:\repo\feature"),
                ),
                action: PickerAction::NoOp,
                quickfix: None,
            }],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new(
            "Ctrl+d",
            "tests.picker-extra",
        )]),
    );

    let handled = state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(
        shell_ui_mut(&mut state.runtime)?
            .take_picker_one_shot()
            .is_none()
    );
    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec![r"P:\repo\feature|feature|P:\repo\feature".to_string()]
    );
    Ok(())
}

#[test]
fn lsp_session_lifecycle_picker_labels_sessions_and_wires_stop_action() {
    let root = {
        #[cfg(windows)]
        {
            PathBuf::from(r"p:\volt")
        }
        #[cfg(not(windows))]
        {
            PathBuf::from("/volt")
        }
    };
    let session = LspLiveSession::new("rust-analyzer", Some(root.clone()));
    let picker = lsp_session_lifecycle_picker_overlay(LspSessionPickerAction::Stop, &[session]);
    assert_eq!(picker.session().title(), "Stop Language Server Session");
    assert_eq!(picker.session().item_count(), 1);
    let selected = picker.session().selected().expect("one row");
    assert_eq!(
        selected.item().label(),
        format!("rust-analyzer — {}", root.display())
    );
    let action = picker
        .actions
        .get(selected.item().id())
        .expect("stop action");
    assert!(matches!(
        action,
        PickerAction::StopLspSession {
            server_id,
            root: action_root
        } if server_id == "rust-analyzer" && action_root.as_deref() == Some(root.as_path())
    ));
}
