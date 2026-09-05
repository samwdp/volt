#[test]
fn terminal_mode_insert_hook_allows_reentering_insert_for_terminals() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_INSERT, HookEvent::new())
        .map_err(|error| error.to_string())?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn terminal_mode_normal_hook_uses_live_terminal_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.set_viewport_lines(2);
        buffer.replace_with_lines_follow_output(vec![
            "zero".to_owned(),
            "one".to_owned(),
            "two".to_owned(),
            "three456".to_owned(),
        ]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            8,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        3,
                        "two",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        8,
                        "three456",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                1,
                5,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 5));
    Ok(())
}

#[test]
fn terminal_popup_mode_normal_hook_uses_live_terminal_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_popup_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal popup test buffer missing".to_owned())?;
        buffer.set_viewport_lines(2);
        buffer.replace_with_lines_follow_output(vec![
            "zero".to_owned(),
            "one".to_owned(),
            "two".to_owned(),
            "three456".to_owned(),
        ]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            8,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        3,
                        "two",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        8,
                        "three456",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                1,
                4,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 4));
    Ok(())
}

#[test]
fn terminal_vim_edit_shortcuts_enter_insert_mode_instead_of_read_only_errors() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("substitute-char"),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn popup_terminal_event_context_prefers_popup_buffer_when_focused() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let pane_buffer = install_scratch_test_buffer(&mut state, "*popup-pane*")?;
    let popup_buffer = install_terminal_popup_test_buffer(&mut state)?;

    let context = active_buffer_event_context(&state.runtime)?;
    assert_eq!(context.buffer_id, popup_buffer);
    assert!(context.is_terminal);
    assert_ne!(context.buffer_id, pane_buffer);

    shell_ui_mut(&mut state.runtime)?.set_popup_focus(false);

    let context = active_buffer_event_context(&state.runtime)?;
    assert_eq!(context.buffer_id, pane_buffer);
    assert!(!context.is_terminal);
    Ok(())
}

#[test]
fn terminal_put_shortcuts_paste_yanks_in_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
    {
        let vim = shell_ui_mut(&mut state.runtime)?.vim_mut();
        vim.active_register = Some('a');
        vim.registers.insert(
            'a',
            YankRegister::Character("volt terminal paste".to_owned()),
        );
    }

    assert!(handle_terminal_vim_edit(
        &mut state.runtime,
        VimEditAction::PutAfter
    )?);
    assert!(terminal_buffer_state(&state.runtime)?.contains(buffer_id));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert_eq!(shell_ui(&state.runtime)?.vim().pending, None);
    Ok(())
}

#[test]
fn terminal_popup_bootstraps_session_and_enters_insert_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_popup_test_buffer(&mut state)?;

    let popup = state
        .runtime_popup()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;

    assert_eq!(popup.active_buffer, buffer_id);
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    assert!(terminal_buffer_state(&state.runtime)?.contains(buffer_id));
    Ok(())
}

#[test]
fn terminal_popup_command_focuses_the_popup_surface() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let pane_buffer = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("terminal.popup")
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.popup_buffer_id, Some(popup.active_buffer));
    assert_eq!(active_shell_buffer_id(&state.runtime)?, popup.active_buffer);
    assert_ne!(popup.active_buffer, pane_buffer);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(terminal_buffer_state(&state.runtime)?.contains(popup.active_buffer));
    Ok(())
}

#[test]
fn dismissed_popup_toggle_restores_terminal_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    state
        .runtime
        .execute_command("terminal.popup")
        .map_err(|error| error.to_string())?;

    let first_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;
    let terminal_buffer = first_popup.active_buffer;
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));

    state
        .runtime
        .execute_command("picker.toggle-popup-window")
        .map_err(|error| error.to_string())?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.popup_buffer_id, None);
    assert!(shell_buffer(&state.runtime, terminal_buffer).is_ok());
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));

    state
        .runtime
        .execute_command("picker.toggle-popup-window")
        .map_err(|error| error.to_string())?;

    let restored_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not restored".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(restored_popup.active_buffer, terminal_buffer);
    assert_eq!(ui.popup_buffer_id, Some(terminal_buffer));
    assert!(ui.popup_focus);
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));
    Ok(())
}

#[test]
fn browser_popup_command_focuses_the_popup_surface() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let pane_buffer = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("browser.open-popup")
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "browser popup was not opened".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.popup_buffer_id, Some(popup.active_buffer));
    assert_eq!(active_shell_buffer_id(&state.runtime)?, popup.active_buffer);
    assert_ne!(popup.active_buffer, pane_buffer);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(matches!(
        shell_buffer(&state.runtime, popup.active_buffer)?.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
    Ok(())
}

#[test]
fn workspace_dashboard_command_opens_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    state
        .runtime
        .execute_command("workspace.dashboard")
        .map_err(|error| error.to_string())?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "workspace dashboard picker did not open".to_owned())?;
    assert_eq!(picker.session.title(), "Worktrees");
    assert!(picker.session.item_count() > 0);
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

fn seed_worktree_remove_one_shot(runtime: &mut EditorRuntime, path: &Path) -> Result<(), String> {
    let path_text = path.display().to_string();
    shell_ui_mut(runtime)?.set_picker_one_shot(PickerOneShotContext::new(
        Some(PickerSelectedRow::new(
            path_text.clone(),
            "worktree",
            Some(path_text),
        )),
        Vec::new(),
    ));
    Ok(())
}

fn unique_sibling_path(anchor: &Path, label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    anchor.parent().unwrap_or(anchor).join(format!(
        "volt-shell-tests-{label}-{}-{unique}",
        std::process::id()
    ))
}

fn add_linked_worktree(main: &Path, label: &str, branch: &str) -> Result<PathBuf, String> {
    let worktree = unique_sibling_path(main, label);
    run_git_in_dir(main, &["branch", "-q", branch])?;
    let path_arg = worktree
        .to_str()
        .ok_or_else(|| format!("non-utf8 worktree path `{}`", worktree.display()))?;
    run_git_in_dir(main, &["worktree", "add", "-q", path_arg, branch])?;
    Ok(worktree)
}

fn wait_for_streamed_notification_title(
    state: &mut ShellState,
    needle: &str,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let visible = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
        if visible.iter().any(|entry| entry.title.contains(needle)) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "notification title containing `{needle}` never appeared"
    ))
}

#[test]
fn worktree_remove_missing_one_shot_is_silent_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let before = shell_ui(&state.runtime)?.notification_revision();

    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);
    Ok(())
}

#[test]
fn worktree_remove_create_affordance_one_shot_is_silent_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    shell_ui_mut(&mut state.runtime)?.set_picker_one_shot(PickerOneShotContext::new(
        Some(PickerSelectedRow::new(
            "git-worktree-dashboard:create",
            "+ new worktree",
            None::<&str>,
        )),
        Vec::new(),
    ));
    let before = shell_ui(&state.runtime)?.notification_revision();

    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);
    Ok(())
}

#[test]
fn worktree_remove_closes_matching_workspaces_streams_and_closes_on_success() -> Result<(), String>
{
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("worktree-remove-success-marks");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;

    let main = init_git_repo_with_commit("worktree-remove-success-main")?;
    let feature = add_linked_worktree(&main, "worktree-remove-success-feature", "feature-remove")?;
    let main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    let feature_ws = open_workspace_from_project(&mut state.runtime, "feature", &feature)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), feature_ws);

    state
        .runtime
        .execute_command("workspace.mark")
        .map_err(|error| error.to_string())?;
    let marks_before =
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?;
    assert!(!marks_before.trim().is_empty());

    seed_worktree_remove_one_shot(&mut state.runtime, &feature)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    assert!(
        find_workspace_by_root(&state.runtime, &feature)?.is_none(),
        "matching Project Workspace should close before git starts"
    );
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), main_ws);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed Worktree Remove popup was not opened".to_owned())?;
    let buffer_id = popup.active_buffer;
    assert!(shell_ui(&state.runtime)?.popup_focus);
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let contents = (0..buffer.line_count())
        .map(|line_index| buffer.text.line(line_index).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        contents.contains("git worktree remove") && contents.contains("--force"),
        "popup should show force remove command, got `{contents}`"
    );
    let feature_name = feature
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "feature worktree name".to_owned())?;
    assert!(
        contents.contains(feature_name),
        "popup should include worktree path, got `{contents}`"
    );

    wait_for_streamed_notification_title(&mut state, "Worktree Remove succeeded")?;
    wait_for_streamed_command_buffer_close(&mut state, buffer_id)?;
    assert!(
        !feature.exists(),
        "worktree path should be removed from disk"
    );
    assert_eq!(
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?,
        marks_before,
        "Mark List must stay untouched"
    );

    let branch_list = run_git_in_dir(&main, &["branch", "--list", "feature-remove"])?;
    assert!(
        branch_list.contains("feature-remove"),
        "Worktree Remove must not delete the branch"
    );

    let _ = std::fs::remove_dir_all(&main);
    let _ = std::fs::remove_dir_all(&state_dir);
    Ok(())
}

#[test]
fn worktree_remove_prunable_checkout_streams_prune_and_clears_registration() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("worktree-remove-prunable-main")?;
    let feature = add_linked_worktree(&main, "worktree-remove-prunable-feature", "feature-prune")?;
    let _main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    // Break the checkout so porcelain marks it prunable (matches stale `/w/...` trees).
    std::fs::remove_file(feature.join(".git")).map_err(|error| error.to_string())?;

    seed_worktree_remove_one_shot(&mut state.runtime, &feature)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed Worktree Remove popup was not opened".to_owned())?;
    let buffer_id = popup.active_buffer;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let contents = (0..buffer.line_count())
        .map(|line_index| buffer.text.line(line_index).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        contents.contains("git worktree prune"),
        "prunable worktree should prune, got `{contents}`"
    );

    wait_for_streamed_notification_title(&mut state, "Worktree Remove succeeded")?;
    wait_for_streamed_command_buffer_close(&mut state, buffer_id)?;
    assert!(
        !feature.exists(),
        "leftover prunable checkout path should be deleted"
    );
    let list = run_git_in_dir(&main, &["worktree", "list", "--porcelain"])?;
    assert!(
        !list.contains("feature-prune")
            && !list.contains(
                feature
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("feature-prune")
            ),
        "pruned worktree must not remain registered, got `{list}`"
    );

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn worktree_remove_failure_notifies_and_keeps_buffer_after_closing_workspaces() -> Result<(), String>
{
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("worktree-remove-fail-main")?;
    let main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    let default_workspace = shell_ui(&state.runtime)?.default_workspace();

    seed_worktree_remove_one_shot(&mut state.runtime, &main)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    assert!(
        find_workspace_by_root(&state.runtime, &main)?.is_none(),
        "Project Workspace should stay closed after git failure"
    );
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        default_workspace
    );
    assert_ne!(main_ws, default_workspace);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed Worktree Remove popup was not opened".to_owned())?;
    let buffer_id = popup.active_buffer;
    wait_for_streamed_notification_title(&mut state, "Worktree Remove failed")?;
    assert!(
        shell_ui(&state.runtime)?.buffer(buffer_id).is_some(),
        "failure must keep the streamed popup buffer"
    );
    assert!(
        !shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "worker should finish even when buffer is kept"
    );
    assert!(main.exists(), "main worktree should remain on disk");

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn worktree_remove_second_invocation_opens_distinct_streamed_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("worktree-remove-concurrent-main")?;
    let first = add_linked_worktree(&main, "worktree-remove-concurrent-a", "feature-a")?;
    let second = add_linked_worktree(&main, "worktree-remove-concurrent-b", "feature-b")?;
    open_workspace_from_project(&mut state.runtime, "main", &main)?;

    seed_worktree_remove_one_shot(&mut state.runtime, &first)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;
    let first_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "first Worktree Remove popup missing".to_owned())?;
    let first_buffer = first_popup.active_buffer;

    seed_worktree_remove_one_shot(&mut state.runtime, &second)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;
    let second_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "second Worktree Remove popup missing".to_owned())?;
    let second_buffer = second_popup.active_buffer;

    assert_ne!(first_buffer, second_buffer);
    assert!(
        shell_ui(&state.runtime)?.buffer(first_buffer).is_some()
            || shell_ui(&state.runtime)?
                .streamed_command_worker
                .contains(first_buffer),
        "first remove buffer should still exist or still be tracked"
    );
    assert!(
        shell_ui(&state.runtime)?.buffer(second_buffer).is_some()
            || shell_ui(&state.runtime)?
                .streamed_command_worker
                .contains(second_buffer),
        "second remove buffer should exist or be tracked"
    );

    wait_for_streamed_command_buffer_close(&mut state, first_buffer)?;
    wait_for_streamed_command_buffer_close(&mut state, second_buffer)?;

    let _ = std::fs::remove_dir_all(&main);
    let _ = std::fs::remove_dir_all(&first);
    let _ = std::fs::remove_dir_all(&second);
    Ok(())
}

fn open_workspace_dashboard(runtime: &mut EditorRuntime) -> Result<(), String> {
    runtime
        .execute_command("workspace.dashboard")
        .map_err(|error| error.to_string())?;
    shell_ui(runtime)?
        .picker()
        .ok_or_else(|| "workspace.dashboard did not open picker".to_owned())?;
    Ok(())
}

fn select_dashboard_row_matching_path(
    runtime: &mut EditorRuntime,
    path: &Path,
) -> Result<(), String> {
    let picker = shell_ui_mut(runtime)?
        .picker_mut()
        .ok_or_else(|| "dashboard picker missing".to_owned())?;
    let index = picker
        .session
        .matches()
        .iter()
        .position(|matched| project_roots_equal(Path::new(matched.item().id()), path))
        .ok_or_else(|| format!("dashboard missing worktree row for `{}`", path.display()))?;
    picker.session.set_selected_index(index);
    Ok(())
}

fn select_dashboard_create_row(runtime: &mut EditorRuntime) -> Result<(), String> {
    let picker = shell_ui_mut(runtime)?
        .picker_mut()
        .ok_or_else(|| "dashboard picker missing".to_owned())?;
    let index = picker
        .session
        .matches()
        .iter()
        .position(|matched| matched.item().id() == "git-worktree-dashboard:create")
        .ok_or_else(|| "dashboard missing `+ new worktree` row".to_owned())?;
    picker.session.set_selected_index(index);
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
fn workspace_dashboard_ctrl_d_on_worktree_runs_remove_ux() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-ctrl-d-remove-main")?;
    let feature = add_linked_worktree(
        &main,
        "dashboard-ctrl-d-remove-feature",
        "feature-dash-remove",
    )?;
    let main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    let feature_ws = open_workspace_from_project(&mut state.runtime, "feature", &feature)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), feature_ws);

    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &feature)?;

    let handled = state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);
    assert!(
        shell_ui(&state.runtime)?.picker().is_none(),
        "Ctrl+d should close the Workspace Dashboard picker"
    );
    assert!(
        find_workspace_by_root(&state.runtime, &feature)?.is_none(),
        "matching Project Workspace should close"
    );
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), main_ws);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed Worktree Remove popup was not opened".to_owned())?;
    let buffer_id = popup.active_buffer;
    assert!(shell_ui(&state.runtime)?.popup_focus);
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let contents = (0..buffer.line_count())
        .map(|line_index| buffer.text.line(line_index).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        contents.contains("git worktree remove") && contents.contains("--force"),
        "popup should show force remove command, got `{contents}`"
    );

    wait_for_streamed_notification_title(&mut state, "Worktree Remove succeeded")?;
    wait_for_streamed_command_buffer_close(&mut state, buffer_id)?;
    assert!(
        !feature.exists(),
        "worktree path should be removed from disk"
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
fn workspace_dashboard_enter_still_switches_and_creates() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-enter-unchanged-main")?;
    let feature = add_linked_worktree(&main, "dashboard-enter-unchanged-feature", "feature-enter")?;
    let main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    let feature_ws = open_workspace_from_project(&mut state.runtime, "feature", &feature)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), feature_ws);

    // Switch: Enter on an already-open Worktree row.
    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &main)?;
    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), main_ws);

    // Open/create: Enter on a Worktree that is not yet a Project Workspace.
    let closed = add_linked_worktree(&main, "dashboard-enter-unchanged-closed", "feature-closed")?;
    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &closed)?;
    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    let opened = find_workspace_by_root(&state.runtime, &closed)?
        .ok_or_else(|| "Enter should open Project Workspace for worktree".to_owned())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), opened);

    // Create affordance: Enter on `+ new worktree` still starts create flow.
    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_create_row(&mut state.runtime)?;
    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .directory_state()
            .is_some(),
        "`+ new worktree` Enter should open oil directory"
    );
    assert_eq!(
        shell_ui(&state.runtime)?
            .picker()
            .map(|picker| picker.session.title().to_owned()),
        Some("Git Worktree Branch".to_owned()),
        "`+ new worktree` Enter should open the branch picker"
    );

    let _ = std::fs::remove_dir_all(&main);
    let _ = std::fs::remove_dir_all(&feature);
    let _ = std::fs::remove_dir_all(&closed);
    Ok(())
}

#[test]
fn split_runtime_pane_switches_focus_to_the_new_pane() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let original_pane_id = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .active_pane_id()
        .ok_or_else(|| "initial pane is missing".to_owned())?;

    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;

    let runtime_workspace = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let active_pane_id = runtime_workspace
        .active_pane_id()
        .ok_or_else(|| "split pane is missing".to_owned())?;
    assert_ne!(active_pane_id, original_pane_id);
    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(active_pane_id)
    );
    Ok(())
}

#[test]
fn pane_close_hook_closes_the_focused_split() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let initial_pane_id = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .active_pane_id()
        .ok_or_else(|| "initial pane is missing".to_owned())?;

    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Horizontal)?;
    cycle_runtime_pane(&mut state.runtime)?;
    state
        .runtime
        .emit_hook(HOOK_PANE_CLOSE, HookEvent::new())
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?
            .pane_count(),
        1
    );
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    assert_eq!(
        state
            .runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?
            .active_pane_id(),
        Some(initial_pane_id)
    );
    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(initial_pane_id)
    );
    Ok(())
}

#[test]
fn switch_split_hook_reverses_pane_order_and_preserves_the_active_pane() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;

    let (active_pane_id, before) = {
        let ui = shell_ui(&state.runtime)?;
        assert_eq!(ui.active_pane_index(), 0);
        let active_pane_id = ui
            .active_pane_id()
            .ok_or_else(|| "active pane is missing".to_owned())?;
        let before = ui
            .panes()
            .ok_or_else(|| "pane list is missing".to_owned())?
            .iter()
            .map(|pane| pane.buffer_id)
            .collect::<Vec<_>>();
        (active_pane_id, before)
    };

    state
        .runtime
        .emit_hook(HOOK_PANE_SWITCH_SPLIT, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let after = ui
        .panes()
        .ok_or_else(|| "pane list is missing after switch".to_owned())?
        .iter()
        .map(|pane| pane.buffer_id)
        .collect::<Vec<_>>();

    assert_eq!(after, before.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(ui.active_pane_id(), Some(active_pane_id));
    assert_eq!(ui.active_pane_index(), 1);
    Ok(())
}

#[test]
fn render_terminal_buffer_prefers_terminal_render_snapshot() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            12,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        11,
                        "echo hello",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                0,
                0,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Normal,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    let rendered_text = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(rendered_text.contains(&"echo hello"));
    assert!(
        !rendered_text
            .iter()
            .any(|text| text.contains("launching the configured shell"))
    );
    assert!(
        scene
            .iter()
            .any(|command| matches!(command, DrawCommand::FillRoundedRect { .. }))
    );
    Ok(())
}

#[test]
fn terminal_box_drawing_chars_render_as_strokes() -> Result<(), String> {
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let color = Color::RGB(200, 205, 210);

    draw_terminal_text_run(&mut target, 10, 20, "a│b", color, 8, 16)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        scene
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        vec!["a", "b"]
    );
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 21
                && rect.y == 20
                && rect.height == 16
                && *color == to_render_color(Color::RGB(200, 205, 210))
    )));
    Ok(())
}

#[test]
fn render_terminal_buffer_draws_visual_selection_highlight() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let selection_color = Color::RGBA(55, 71, 99, 255);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec!["echo hello".to_owned(), String::new()]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            12,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        10,
                        "echo hello",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![]),
            ],
            None,
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Visual,
            visual_selection: Some(VisualSelection::Range(TextRange::new(
                TextPoint::new(0, 0),
                TextPoint::new(0, 4),
            ))),
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: selection_color,
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. } if *color == to_render_color(selection_color)
    )));
    Ok(())
}
