#[test]
fn markdown_pretty_paint_plan_visual_anti_conceal_then_restores() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*pretty-anti-conceal-visual*",
        PRETTY_CACHE_FIXTURE,
    )?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let visual = VisualSelection::Range(TextRange::new(TextPoint::new(0, 0), TextPoint::new(1, 6)));
    let visual_args = MarkdownPrettyPaintArgs {
        visual_selection: Some(visual),
        input_mode: InputMode::Visual,
        ..markdown_pretty_paint_args(buffer)
    };
    let visual_paint = markdown_pretty_paint_plan(buffer, &*user_library, visual_args);
    assert!(
        !visual_paint.text_overrides.contains_key(&0),
        "Visual selection should paint Markdown Raw: {:?}",
        visual_paint.text_overrides
    );
    assert!(
        !visual_paint.text_overrides.contains_key(&1),
        "Visual selection should paint Markdown Raw on selected lines: {:?}",
        visual_paint.text_overrides
    );
    let visual_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing plan during visual")?;
    let normal_paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let normal_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing plan after visual")?;
    assert!(std::sync::Arc::ptr_eq(&visual_plan, &normal_plan));
    assert!(normal_paint.text_overrides.contains_key(&0));
    assert!(normal_paint.text_overrides.contains_key(&1));
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_toggle_off_is_raw() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-toggle-off*", "# Title\n- item\n")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.toggle_markdown_pretty(true);
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    assert!(paint.text_overrides.is_empty());
    assert!(paint.images.is_empty());
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_kill_switch_skips() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        markdown_pretty: MarkdownPrettyConfig {
            kill_switch_enabled: true,
            kill_switch_max_lines: 0,
            ..MarkdownPrettyConfig::default()
        },
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-kill-switch*", "# Title\n- item\n")?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let first =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let first_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing kill-switch sentinel")?;
    assert!(first.text_overrides.is_empty());
    assert!(first_plan.skipped_by_kill_switch);
    let second =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let second_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing reused sentinel")?;
    assert!(std::sync::Arc::ptr_eq(&first_plan, &second_plan));
    assert_eq!(first.text_overrides, second.text_overrides);
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_forced_language_caches() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*pretty-forced-language*")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "# Title".to_owned(),
            "- item".to_owned(),
            "plain".to_owned(),
        ]);
        buffer.set_forced_language_id("markdown");
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let first =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let first_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing Forced Language plan")?;
    let second =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let second_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing reused Forced Language plan")?;
    assert!(std::sync::Arc::ptr_eq(&first_plan, &second_plan));
    assert_eq!(first.text_overrides, second.text_overrides);
    assert!(
        first
            .text_overrides
            .get(&0)
            .is_some_and(|line| line.contains("Title") && !line.starts_with("# ")),
        "Forced Language markdown should Pretty: {:?}",
        first.text_overrides
    );
    Ok(())
}

fn markdown_table_event_dimensions() -> (u32, u32, i32, i32) {
    (640, 240, 8, 16)
}

fn focus_test_buffer(state: &mut ShellState, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)
}

fn install_browser_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
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
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        BROWSER_BUFFER_NAME,
        BufferKind::Plugin(BROWSER_KIND.to_owned()),
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn install_terminal_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, "*terminal*", BufferKind::Terminal, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        "*terminal*",
        BufferKind::Terminal,
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn install_terminal_popup_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*terminal-popup*", BufferKind::Terminal, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Terminal", vec![buffer_id], buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_popup_buffer(
        buffer_id,
        "*terminal-popup*",
        BufferKind::Terminal,
        &NullUserLibrary,
    );
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_popup_buffer(buffer_id);
        ui.set_popup_focus(true);
    }
    Ok(buffer_id)
}

fn install_git_status_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
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
            "*git-status*",
            BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        "*git-status*",
        BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn run_git_in_dir(root: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run git {:?}: {error}", args))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if stderr.is_empty() {
            format!("git {:?} failed with status {}", args, output.status)
        } else {
            format!("git {:?} failed: {stderr}", args)
        };
        return Err(message);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn init_git_repo(label: &str) -> Result<std::path::PathBuf, String> {
    let repo = unique_temp_dir(label);
    run_git_in_dir(&repo, &["init", "-q"])?;
    run_git_in_dir(&repo, &["config", "user.email", "volt-tests@example.com"])?;
    run_git_in_dir(&repo, &["config", "user.name", "Volt Tests"])?;
    run_git_in_dir(&repo, &["config", "commit.gpgsign", "false"])?;
    Ok(repo)
}

fn init_git_repo_with_commit(label: &str) -> Result<std::path::PathBuf, String> {
    let repo = init_git_repo(label)?;
    std::fs::write(repo.join("README.md"), "seed\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "README.md"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "initial"])?;
    Ok(repo)
}

fn install_git_hook(repo: &std::path::Path, hook_name: &str, script: &str) -> Result<(), String> {
    let hook_path = repo.join(".git").join("hooks").join(hook_name);
    std::fs::write(&hook_path, script).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        let mut permissions = std::fs::metadata(&hook_path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&hook_path, permissions).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn open_repo_git_status_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "git-test", root)?;
    let buffer_id = install_git_status_test_buffer(state)?;
    refresh_git_status_buffer(&mut state.runtime, buffer_id)?;
    Ok(buffer_id)
}

fn wait_for_streamed_command_output_line(
    state: &mut ShellState,
    buffer_id: BufferId,
    needle: &str,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        let matched = shell_ui(&state.runtime)?
            .buffer(buffer_id)
            .is_some_and(|buffer| {
                (0..buffer.line_count()).any(|line_index| {
                    buffer
                        .text
                        .line(line_index)
                        .unwrap_or_default()
                        .contains(needle)
                })
            });
        if tracked && matched {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "streamed command buffer `{buffer_id}` never emitted `{needle}` while running"
    ))
}

fn wait_for_streamed_command_buffer_close(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        let buffered = shell_ui(&state.runtime)?.buffer(buffer_id).is_some();
        if !tracked && !buffered {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    let tracked = terminal_buffer_state(&state.runtime)?.contains(buffer_id);
    let worker_tracked = shell_ui(&state.runtime)?
        .streamed_command_worker
        .contains(buffer_id);
    let buffered = shell_ui(&state.runtime)?.buffer(buffer_id).is_some();
    let popup_visible = active_runtime_popup(&state.runtime)?.is_some();
    Err(format!(
        "temporary streamed command buffer `{buffer_id}` did not close in time (terminal_tracked={tracked}, worker_tracked={worker_tracked}, buffered={buffered}, popup_visible={popup_visible})"
    ))
}

fn open_oil_test_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "oil-test", root)?;
    open_oil_directory(&mut state.runtime, root.to_path_buf())?;
    active_shell_buffer_id(&state.runtime)
}

fn oil_line_index_containing(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    needle: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|&index| buffer.text.line(index).unwrap_or_default().contains(needle))
        .ok_or_else(|| format!("oil buffer is missing line containing `{needle}`"))
}

fn oil_line_index_for_entry_path(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    path: &Path,
) -> Result<usize, String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|&index| {
            buffer
                .section_line_meta(index)
                .and_then(|meta| meta.action.as_ref())
                .filter(|action| action.id() == editor_plugin_api::oil_protocol::ACTION_OIL_ENTRY)
                .and_then(|action| action.detail())
                .is_some_and(|detail| Path::new(detail) == path)
        })
        .ok_or_else(|| format!("oil buffer is missing entry `{}`", path.display()))
}

fn oil_type_new_entry_and_leave_insert(state: &mut ShellState, entry: &str) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let last_line = shell_buffer(&state.runtime, buffer_id)?
        .line_count()
        .saturating_sub(1);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(last_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    if shell_ui(&state.runtime)?.input_mode() != InputMode::Insert {
        return Err(format!(
            "expected insert mode after o, got {:?}",
            shell_ui(&state.runtime)?.input_mode()
        ));
    }
    state
        .handle_text_input(entry)
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn install_text_test_buffer(
    state: &mut ShellState,
    name: &str,
    lines: Vec<String>,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, name, BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "text test buffer is missing".to_owned())?;
    let shell_buffer = ShellBuffer::from_runtime_buffer(buffer, lines, &NullUserLibrary);
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn screen_point_for_buffer_point(
    state: &mut ShellState,
    buffer_id: BufferId,
    point: TextPoint,
    render_width: u32,
    render_height: u32,
    cell_width: i32,
    line_height: i32,
) -> Result<(f32, f32), String> {
    let original_cursor = shell_buffer(&state.runtime, buffer_id)?.cursor_point();
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(point);
    let anchor = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer_cursor_screen_anchor(
            buffer,
            PixelRectToRect::rect(0, 0, render_width, render_height),
            &*shell_user_library(&state.runtime),
            state.runtime.services().get::<ThemeRegistry>(),
            cell_width,
            line_height,
            false,
        )
        .ok_or_else(|| "buffer cursor screen anchor was missing".to_owned())?
    };
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(original_cursor);
    Ok((
        (anchor.x + (cell_width / 2).max(1)) as f32,
        (anchor.y + (line_height / 2).max(1)) as f32,
    ))
}

fn git_status_line_for_action_detail(
    state: &ShellState,
    buffer_id: BufferId,
    action_id: &str,
    detail: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|line_index| {
            git_action_detail(buffer.section_line_meta(*line_index), action_id).as_deref()
                == Some(detail)
        })
        .ok_or_else(|| format!("git status line for `{detail}` and `{action_id}` was not found"))
}

fn git_status_header_line(
    state: &ShellState,
    buffer_id: BufferId,
    section_id: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|line_index| {
            buffer.section_line_meta(*line_index).is_some_and(|meta| {
                meta.section_id == section_id
                    && matches!(meta.kind, SectionRenderLineKind::Header { .. })
            })
        })
        .ok_or_else(|| format!("git status header line for section `{section_id}` was not found"))
}

fn set_git_status_visual_line_selection(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    let (start_line, end_line) = if start_line <= end_line {
        (start_line, end_line)
    } else {
        (end_line, start_line)
    };
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(start_line, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(end_line, 0));
    Ok(())
}

fn set_git_status_visual_block_selection_with_ctrl_v(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    assert!(
        state
            .try_runtime_keybinding(Keycode::V, ctrl_mod())
            .map_err(|error| error.to_string())?
    );

    state
        .handle_text_input("v")
        .map_err(|error| error.to_string())?;

    let motion = if end_line >= start_line { "j" } else { "k" };
    for _ in 0..start_line.abs_diff(end_line) {
        state
            .handle_text_input(motion)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Visual);
    assert_eq!(
        shell_ui(&state.runtime)?.vim().visual_kind,
        VisualSelectionKind::Block
    );
    Ok(())
}

fn set_git_status_visual_line_selection_with_shift_v(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;

    let motion = if end_line >= start_line { "j" } else { "k" };
    for _ in 0..start_line.abs_diff(end_line) {
        state
            .handle_text_input(motion)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Visual);
    assert_eq!(
        shell_ui(&state.runtime)?.vim().visual_kind,
        VisualSelectionKind::Line
    );
    Ok(())
}

type GitSnapshotPaths = (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>);

fn git_status_snapshot_paths(
    state: &ShellState,
    buffer_id: BufferId,
) -> Result<GitSnapshotPaths, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let snapshot = buffer
        .git_snapshot()
        .ok_or_else(|| "git snapshot missing".to_owned())?;
    let staged = snapshot
        .staged()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect();
    let unstaged = snapshot
        .unstaged()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect();
    let untracked = snapshot.untracked().iter().cloned().collect();
    Ok((staged, unstaged, untracked))
}

fn install_hover_test_overlay(state: &mut ShellState, focused: bool) -> Result<BufferId, String> {
    let buffer_id = shell_ui(&state.runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let anchor = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    shell_ui_mut(&mut state.runtime)?.set_hover(HoverOverlay {
        buffer_id,
        anchor,
        token: "hover".to_owned(),
        providers: vec![
            HoverProviderContent {
                provider_label: "Alpha".to_owned(),
                provider_icon: "A".to_owned(),
                lines: vec!["first".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
            HoverProviderContent {
                provider_label: "Beta".to_owned(),
                provider_icon: "B".to_owned(),
                lines: vec!["second".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
            HoverProviderContent {
                provider_label: "Gamma".to_owned(),
                provider_icon: "G".to_owned(),
                lines: vec!["third".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
        ],
        provider_index: 0,
        scroll_offset: 0,
        focused,
        line_limit: 8,
        pending_g_prefix: false,
        count: None,
    });
    Ok(buffer_id)
}

fn install_scrollable_hover_test_overlay(
    state: &mut ShellState,
    focused: bool,
) -> Result<BufferId, String> {
    let buffer_id = shell_ui(&state.runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let anchor = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    let lines = (1..=12)
        .map(|line| format!("Line {line}"))
        .collect::<Vec<_>>();
    shell_ui_mut(&mut state.runtime)?.set_hover(HoverOverlay {
        buffer_id,
        anchor,
        token: "hover".to_owned(),
        providers: vec![HoverProviderContent {
            provider_label: "Scrollable".to_owned(),
            provider_icon: "S".to_owned(),
            lines,
            syntax_lines: BTreeMap::new(),
        }],
        provider_index: 0,
        scroll_offset: 0,
        focused,
        line_limit: 4,
        pending_g_prefix: false,
        count: None,
    });
    Ok(buffer_id)
}

fn hover_scroll_offset(state: &ShellState) -> Result<usize, String> {
    shell_ui(&state.runtime)?
        .hover()
        .map(|hover| hover.scroll_offset)
        .ok_or_else(|| "hover overlay missing".to_owned())
}

fn test_notification_update(
    key: &str,
    severity: NotificationSeverity,
    title: &str,
    body_lines: &[&str],
    progress: Option<u8>,
    active: bool,
) -> NotificationUpdate {
    NotificationUpdate {
        key: key.to_owned(),
        severity,
        title: title.to_owned(),
        body_lines: body_lines.iter().map(|line| (*line).to_owned()).collect(),
        progress: progress.map(|percentage| NotificationProgress {
            percentage: Some(percentage),
        }),
        active,
        action: None,
        workspace_id: None,
    }
}

#[test]
fn parse_rg_workspace_search_line_extracts_location() {
    let parsed = parse_rg_workspace_search_line(r"src\main.rs:12:7:let answer = compute();")
        .expect("rg output should parse into a workspace search match");
    assert_eq!(parsed.0, r"src\main.rs");
    assert_eq!(parsed.1, 12);
    assert_eq!(parsed.2, 7);
    assert_eq!(parsed.3, "let answer = compute();");
}

#[test]
fn parse_grep_workspace_search_line_finds_case_insensitive_column() {
    let parsed = parse_grep_workspace_search_line(r"src\lib.rs:3:Hello Workspace", "workspace")
        .expect("grep output should parse into a workspace search match");
    assert_eq!(parsed.0, r"src\lib.rs");
    assert_eq!(parsed.1, 3);
    assert_eq!(parsed.2, 7);
    assert_eq!(parsed.3, "Hello Workspace");
}

#[test]
fn workspace_search_char_column_handles_utf8_offsets() {
    assert_eq!(workspace_search_char_column("aébc", 0), 0);
    assert_eq!(workspace_search_char_column("aébc", 1), 1);
    assert_eq!(workspace_search_char_column("aébc", 3), 2);
}

#[test]
fn collect_search_output_stops_after_limit() {
    let (output, reached_limit) =
        collect_search_output(std::io::Cursor::new("one\ntwo\nthree\n"), 2)
            .expect("search output should be collected");
    assert_eq!(output, "one\ntwo\n");
    assert!(reached_limit);
}

#[test]
fn frame_pacing_remaining_clamps_to_120fps_budget() {
    let now = Instant::now();
    let remaining = frame_pacing_remaining(now - Duration::from_millis(2), now);
    assert!(remaining >= Duration::from_micros(6_000));
    assert_eq!(
        frame_pacing_remaining(now - Duration::from_millis(10), now),
        Duration::from_secs(0)
    );
}

#[test]
fn git_refresh_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(git_refresh_deferred_for_typing(Some(now), now));
    assert!(git_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!git_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!git_refresh_deferred_for_typing(None, now));
}

#[test]
fn secondary_refresh_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(secondary_refresh_deferred_for_typing(Some(now), now));
    assert!(secondary_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!secondary_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!secondary_refresh_deferred_for_typing(None, now));
}

#[test]
fn frame_pacing_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(frame_pacing_deferred_for_typing(Some(now), now));
    assert!(frame_pacing_deferred_for_typing(
        Some(now - FRAME_PACING_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!frame_pacing_deferred_for_typing(
        Some(now - FRAME_PACING_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!frame_pacing_deferred_for_typing(None, now));
}

#[test]
fn idle_wait_timeout_equals_next_deadline_when_idle() {
    let now = Instant::now();
    let deadline = now + Duration::from_millis(40);
    assert_eq!(
        idle_wait_timeout(now, &[deadline], false, false),
        Some(Duration::from_millis(40))
    );
}

#[test]
fn idle_wait_timeout_caps_and_skips_when_interacting() {
    let now = Instant::now();
    assert_eq!(
        idle_wait_timeout(now, &[], false, false),
        Some(IDLE_WAIT_CAP)
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_secs(5)], false, false),
        Some(IDLE_WAIT_CAP)
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_millis(40)], true, false),
        None
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_millis(40)], false, true),
        None
    );
}

#[test]
fn normal_mode_text_input_does_not_activate_typing_budget() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;

    assert!(!state.secondary_refresh_deferred_for_typing(Instant::now()));
    assert!(!state.typing_refresh_budget_active(Instant::now()));
    Ok(())
}

#[test]
fn insert_mode_text_input_activates_typing_budget() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    assert!(state.secondary_refresh_deferred_for_typing(Instant::now()));
    assert!(state.typing_refresh_budget_active(Instant::now()));
    Ok(())
}

#[test]
fn context_overlay_cache_reuses_stale_snapshot_while_typing() {
    let cached = Arc::new(BufferContextOverlaySnapshot {
        key: BufferContextOverlayCacheKey {
            buffer_revision: 41,
            buffer_name: "demo.rs".to_owned(),
            language_id: Some("rust".to_owned()),
            viewport_top_line: 10,
            cursor_line: 20,
            cursor_column: 4,
        },
        headerline_lines: vec!["fn demo".to_owned()],
        ghost_text_by_line: BTreeMap::new(),
    });
    let key = BufferContextOverlayCacheKey {
        buffer_revision: 42,
        buffer_name: "demo.rs".to_owned(),
        language_id: Some("rust".to_owned()),
        viewport_top_line: 11,
        cursor_line: 21,
        cursor_column: 5,
    };

    let snapshot =
        cached_context_overlay_snapshot(Some(&cached), &key, true).expect("stale snapshot");

    assert!(Arc::ptr_eq(&snapshot, &cached));
    assert_eq!(snapshot.key.buffer_revision, 41);
    assert_eq!(snapshot.headerline_lines, vec!["fn demo".to_owned()]);
}

#[test]
fn context_overlay_cache_requires_matching_buffer_identity() {
    let cached = Arc::new(BufferContextOverlaySnapshot {
        key: BufferContextOverlayCacheKey {
            buffer_revision: 1,
            buffer_name: "demo.rs".to_owned(),
            language_id: Some("rust".to_owned()),
            viewport_top_line: 0,
            cursor_line: 0,
            cursor_column: 0,
        },
        headerline_lines: vec!["fn demo".to_owned()],
        ghost_text_by_line: BTreeMap::new(),
    });
    let key = BufferContextOverlayCacheKey {
        buffer_revision: 2,
        buffer_name: "demo.py".to_owned(),
        language_id: Some("python".to_owned()),
        viewport_top_line: 0,
        cursor_line: 0,
        cursor_column: 0,
    };

    assert!(cached_context_overlay_snapshot(Some(&cached), &key, false).is_none());
    assert!(cached_context_overlay_snapshot(Some(&cached), &key, true).is_none());
}

#[test]
fn context_overlay_snapshot_reuses_same_arc_when_key_matches() -> Result<(), String> {
    let user_library = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline-arc-reuse*",
        vec!["alpha".to_owned()],
    )?;
    let first = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer.context_overlay_snapshot(&*user_library, false)
    };
    let second = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer.context_overlay_snapshot(&*user_library, false)
    };
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(user_library.headerline_call_count(), 1);
    Ok(())
}

#[test]
fn typing_event_batches_yield_once_budget_is_exhausted() {
    let now = Instant::now();
    assert!(!should_yield_after_typing_batch(
        0,
        TYPING_EVENT_BATCH_LIMIT,
        now
    ));
    assert!(!should_yield_after_typing_batch(
        1,
        TYPING_EVENT_BATCH_LIMIT - 1,
        now
    ));
    assert!(should_yield_after_typing_batch(
        1,
        TYPING_EVENT_BATCH_LIMIT,
        now
    ));
    assert!(should_yield_after_typing_batch(
        1,
        1,
        now - TYPING_EVENT_BATCH_TIME_BUDGET
    ));
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
fn git_status_header_spans_skip_leading_icons() {
    let line = SectionRenderLine {
        text: format!(
            "{} Head: master f9d8c15 Added some more keybinds",
            editor_icons::symbols::dev::DEV_GIT_BRANCH
        ),
        depth: 1,
        section_id: GIT_SECTION_HEADERS.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(),
                editor_icons::symbols::dev::DEV_GIT_BRANCH.to_owned(),
            ),
            (TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(), "Head:".to_owned()),
            (
                TOKEN_GIT_STATUS_HEADER_VALUE.to_owned(),
                "master".to_owned()
            ),
            (
                TOKEN_GIT_STATUS_HEADER_HASH.to_owned(),
                "f9d8c15".to_owned()
            ),
            (
                TOKEN_GIT_STATUS_HEADER_SUMMARY.to_owned(),
                "Added some more keybinds".to_owned(),
            ),
        ]
    );
}

#[test]
fn git_status_merge_header_spans_keep_tracking_counts() {
    let line = SectionRenderLine {
        text: format!(
            "{} Merge: origin/main (ahead 2, behind 1)",
            editor_icons::symbols::cod::COD_ARROW_DOWN
        ),
        depth: 1,
        section_id: GIT_SECTION_HEADERS.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(),
                editor_icons::symbols::cod::COD_ARROW_DOWN.to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(),
                "Merge:".to_owned()
            ),
            (
                TOKEN_GIT_STATUS_HEADER_VALUE.to_owned(),
                "origin/main".to_owned(),
            ),
            (TOKEN_GIT_STATUS_SECTION_COUNT.to_owned(), "2".to_owned()),
            (TOKEN_GIT_STATUS_SECTION_COUNT.to_owned(), "1".to_owned()),
        ]
    );
}

#[test]
fn git_status_entry_spans_skip_leading_icons() {
    let line = SectionRenderLine {
        text: format!(
            "{} crates/editor-sdl/src/shell.rs",
            editor_icons::symbols::cod::COD_DIFF_MODIFIED
        ),
        depth: 1,
        section_id: GIT_SECTION_UNSTAGED.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_ENTRY_MODIFIED.to_owned(),
                editor_icons::symbols::cod::COD_DIFF_MODIFIED.to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_ENTRY_PATH.to_owned(),
                "crates/editor-sdl/src/shell.rs".to_owned(),
            ),
        ]
    );
}
