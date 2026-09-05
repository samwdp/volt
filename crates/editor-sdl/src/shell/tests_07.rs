#[test]
fn git_status_stash_spans_handle_compact_stash_names() {
    let line = SectionRenderLine {
        text: format!(
            "{} stash[0] WIP on master: overnight todo",
            editor_icons::symbols::cod::COD_HISTORY
        ),
        depth: 1,
        section_id: GIT_SECTION_STASHES.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_STASH_NAME.to_owned(),
                editor_icons::symbols::cod::COD_HISTORY.to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_STASH_NAME.to_owned(),
                "stash[0]".to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_STASH_SUMMARY.to_owned(),
                "WIP on master: overnight todo".to_owned(),
            ),
        ]
    );
}

#[test]
fn git_status_uppercase_f_starts_pull_prefix() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_git_status_test_buffer(&mut state)?;

    assert!(handle_git_status_chord(&mut state.runtime, "F")?);
    assert_eq!(take_git_prefix(&mut state.runtime)?, Some(GitPrefix::Pull));
    Ok(())
}

#[test]
fn git_status_sequence_commands_are_registered() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;

    for &(name, _, _) in GIT_STATUS_COMMANDS {
        assert!(
            state.runtime.commands().contains(name),
            "missing command `{name}`"
        );
    }
    for name in ["git.diff", "git.log", "git.stash-list"] {
        assert!(
            state.runtime.commands().contains(name),
            "missing command `{name}`"
        );
    }

    Ok(())
}

#[test]
fn git_status_command_name_maps_sequences_to_picker_commands() {
    let user_library = user::UserLibraryImpl;
    assert_eq!(
        git_status_command_name(&user_library, None, "S"),
        Some("git.status.stage-all")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Pull), "u"),
        Some("git.status.pull-upstream")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Branch), "b"),
        Some("git.status.branches")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Diff), "w"),
        Some("git.diff")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Log), "l"),
        Some("git.log")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Stash), "l"),
        Some("git.stash-list")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Rebase), "f"),
        Some("git.status.rebase-autosquash")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Reset), "f"),
        Some("git.status.checkout-file")
    );
}

#[test]
fn git_status_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-visual-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection(&mut state, buffer_id, alpha, beta)?;

    assert!(handle_git_status_chord(&mut state.runtime, "s")?);

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-visual-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection(&mut state, buffer_id, alpha, beta)?;

    assert!(handle_git_status_chord(&mut state.runtime, "u")?);

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("s")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("u")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_x_deletes_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-delete")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(staged.is_empty());
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(!repo.join("alpha.txt").exists());
    assert!(!repo.join("beta.txt").exists());
    assert!(repo.join("gamma.txt").exists());
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("s")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("u")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_x_deletes_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-delete")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(staged.is_empty());
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(!repo.join("alpha.txt").exists());
    assert!(!repo.join("beta.txt").exists());
    assert!(repo.join("gamma.txt").exists());
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_buffer_supports_first_commit_on_fresh_repo() -> Result<(), String> {
    let repo = init_git_repo("git-status-fresh-repo")?;
    let branch = run_git_in_dir(&repo, &["symbolic-ref", "--short", "HEAD"])?
        .trim()
        .to_owned();
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let snapshot = buffer
        .git_snapshot()
        .ok_or_else(|| "git snapshot missing".to_owned())?;
    let has_commit_action = (0..buffer.line_count()).any(|line_index| {
        buffer
            .section_line_meta(line_index)
            .and_then(|meta| meta.action.as_ref())
            .is_some_and(|action| action.id() == editor_plugin_api::git_actions::COMMIT_OPEN)
    });

    assert_eq!(snapshot.branch(), Some(branch.as_str()));
    assert!(snapshot.head().is_none());
    assert!(has_commit_action);
    assert_eq!(staged, BTreeSet::from(["alpha.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert!(untracked.is_empty());

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn buffer_save_does_not_synchronously_refresh_git_status_buffers() -> Result<(), String> {
    // Regression: buffer.save used to block the UI on a full sync `git status` snapshot
    // whenever a git-status buffer was open (:w / <leader>w). workspace.save did not.
    let repo = init_git_repo_with_commit("buffer-save-no-sync-git-status")?;
    let mut state = state_with_user_library()?;
    let status_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let (_, _, untracked_before) = git_status_snapshot_paths(&state, status_id)?;
    assert!(untracked_before.is_empty());

    let path = repo.join("note.txt");
    std::fs::write(&path, "seed\n").map_err(|error| error.to_string())?;
    let file_id = open_workspace_file(&mut state.runtime, &path)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, file_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("edit\n");
        assert!(buffer.is_dirty());
    }

    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;

    let started = Instant::now();
    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;
    let buffer_save_elapsed = started.elapsed();

    let (_, _, untracked_after_save) = git_status_snapshot_paths(&state, status_id)?;
    assert!(
        !untracked_after_save.contains("beta.txt"),
        "buffer.save must not synchronously refresh open git-status buffers (elapsed {buffer_save_elapsed:?}); got {untracked_after_save:?}"
    );
    assert!(!shell_buffer(&state.runtime, file_id)?.is_dirty());
    assert!(
        buffer_save_elapsed < Duration::from_millis(500),
        "buffer.save blocked too long with git-status open: {buffer_save_elapsed:?}"
    );

    refresh_git_status_buffer(&mut state.runtime, status_id)?;
    let (_, _, untracked_after_refresh) = git_status_snapshot_paths(&state, status_id)?;
    assert_eq!(
        untracked_after_refresh,
        BTreeSet::from(["beta.txt".to_owned(), "note.txt".to_owned()])
    );

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_focus_refresh_reuses_recent_snapshot() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-focus-refresh-cache")?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let (_, _, untracked_before) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(untracked_before.is_empty());

    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;

    refresh_git_status_if_active_if_due(&mut state.runtime)?;
    let (_, _, untracked_throttled) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(untracked_throttled.is_empty());

    refresh_git_status_buffer(&mut state.runtime, buffer_id)?;
    let (_, _, untracked_after) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(untracked_after, BTreeSet::from(["beta.txt".to_owned()]));

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_tab_on_unstaged_file_opens_diff_buffer() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-tab-unstaged")?;
    std::fs::write(repo.join("alpha.txt"), "before\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "add alpha"])?;
    std::fs::write(repo.join("alpha.txt"), "after\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let status_buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha_line = git_status_line_for_action_detail(
        &state,
        status_buffer_id,
        GIT_ACTION_STAGE_FILE,
        "alpha.txt",
    )?;
    shell_buffer_mut(&mut state.runtime, status_buffer_id)?
        .set_cursor(TextPoint::new(alpha_line, 0));

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    let diff_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let diff_buffer = shell_buffer(&state.runtime, diff_buffer_id)?;
    assert_ne!(diff_buffer_id, status_buffer_id);
    assert!(matches!(
        &diff_buffer.kind,
        BufferKind::Plugin(kind) if kind == GIT_DIFF_KIND
    ));
    assert_eq!(diff_buffer.language_id(), Some("diff"));
    assert!((0..diff_buffer.line_count()).any(|line_index| {
        diff_buffer
            .text
            .line(line_index)
            .unwrap_or_default()
            .contains("diff --git")
    }));
    assert!((0..diff_buffer.line_count()).any(|line_index| {
        diff_buffer
            .text
            .line(line_index)
            .unwrap_or_default()
            .contains("alpha.txt")
    }));

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_tab_on_staged_file_opens_diff_buffer() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-tab-staged")?;
    std::fs::write(repo.join("alpha.txt"), "before\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "add alpha"])?;
    std::fs::write(repo.join("alpha.txt"), "after\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;

    let mut state = state_with_user_library()?;
    let status_buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha_line = git_status_line_for_action_detail(
        &state,
        status_buffer_id,
        GIT_ACTION_UNSTAGE_FILE,
        "alpha.txt",
    )?;
    shell_buffer_mut(&mut state.runtime, status_buffer_id)?
        .set_cursor(TextPoint::new(alpha_line, 0));

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    let diff_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let diff_buffer = shell_buffer(&state.runtime, diff_buffer_id)?;
    assert_ne!(diff_buffer_id, status_buffer_id);
    assert!(matches!(
        &diff_buffer.kind,
        BufferKind::Plugin(kind) if kind == GIT_DIFF_KIND
    ));
    assert_eq!(diff_buffer.language_id(), Some("diff"));
    assert!((0..diff_buffer.line_count()).any(|line_index| {
        diff_buffer
            .text
            .line(line_index)
            .unwrap_or_default()
            .contains("diff --git")
    }));
    assert!((0..diff_buffer.line_count()).any(|line_index| {
        diff_buffer
            .text
            .line(line_index)
            .unwrap_or_default()
            .contains("alpha.txt")
    }));

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_tab_on_header_still_toggles_section() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-tab-header")?;
    std::fs::write(repo.join("alpha.txt"), "before\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "add alpha"])?;
    std::fs::write(repo.join("alpha.txt"), "after\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let header_line = git_status_header_line(&state, buffer_id, GIT_SECTION_UNSTAGED)?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(header_line, 0));
    assert!(
        !shell_buffer(&state.runtime, buffer_id)?
            .section_state()
            .is_some_and(|state| state.collapsed.is_collapsed(GIT_SECTION_UNSTAGED))
    );

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);
    assert!(
        buffer
            .section_state()
            .is_some_and(|state| state.collapsed.is_collapsed(GIT_SECTION_UNSTAGED))
    );

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_push_upstream_streams_into_popup_buffer_and_refreshes_status() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-push-upstream-popup")?;
    let remote = unique_temp_dir("git-push-upstream-popup-remote");
    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            remote
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", remote.display()))?,
        ],
    )?;
    let branch = run_git_in_dir(&repo, &["symbolic-ref", "--short", "HEAD"])?
        .trim()
        .to_owned();
    run_git_in_dir(
        &repo,
        &["push", "-q", "--set-upstream", "origin", branch.as_str()],
    )?;
    install_git_hook(
        &repo,
        "pre-push",
        "#!/bin/sh\necho \"pre-push hook starting\"\nsleep 1\necho \"pre-push hook finishing\"\n",
    )?;
    std::fs::write(repo.join("feature.txt"), "feature\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "feature.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "feature"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let snapshot = shell_buffer(&state.runtime, buffer_id)?
        .git_snapshot()
        .cloned()
        .ok_or_else(|| "git snapshot missing before push".to_owned())?;
    assert_eq!(snapshot.ahead(), 1);
    assert!(snapshot.upstream().is_some());

    push_git_to_upstream(&mut state.runtime, buffer_id)?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed popup was not opened for git push".to_owned())?;
    assert!(shell_ui(&state.runtime)?.popup_focus);
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert!(matches!(
        &shell_buffer(&state.runtime, popup.active_buffer)?.kind,
        BufferKind::Plugin(kind) if kind == INTERACTIVE_READONLY_KIND
    ));
    assert!(!terminal_buffer_state(&state.runtime)?.contains(popup.active_buffer));

    wait_for_streamed_command_output_line(
        &mut state,
        popup.active_buffer,
        "pre-push hook starting",
    )?;
    wait_for_streamed_command_buffer_close(&mut state, popup.active_buffer)?;
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.popup_focus);
    assert_eq!(ui.popup_buffer_id, None);
    assert!(ui.buffer(popup.active_buffer).is_none());
    assert!(!ui.streamed_command_worker.contains(popup.active_buffer));
    assert!(!terminal_buffer_state(&state.runtime)?.contains(popup.active_buffer));
    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);

    let refreshed = shell_buffer(&state.runtime, buffer_id)?
        .git_snapshot()
        .cloned()
        .ok_or_else(|| "git snapshot missing after push".to_owned())?;
    assert_eq!(refreshed.ahead(), 0);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&remote).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_pull_upstream_streams_into_popup_buffer() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-pull-upstream-popup")?;
    let remote = unique_temp_dir("git-pull-upstream-popup-remote");
    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            remote
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", remote.display()))?,
        ],
    )?;
    let branch = run_git_in_dir(&repo, &["symbolic-ref", "--short", "HEAD"])?
        .trim()
        .to_owned();
    run_git_in_dir(
        &repo,
        &["push", "-q", "--set-upstream", "origin", branch.as_str()],
    )?;
    install_git_hook(
        &repo,
        "pre-merge-commit",
        "#!/bin/sh\necho \"pre-merge hook starting\"\nsleep 1\necho \"pre-merge hook finishing\"\n",
    )?;

    // Create a second commit on remote via a clone so pull has work.
    let clone = unique_temp_dir("git-pull-upstream-clone");
    std::fs::remove_dir_all(&clone).ok();
    run_git_in_dir(
        repo.parent().unwrap_or(&repo),
        &[
            "clone",
            "-q",
            remote
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", remote.display()))?,
            clone
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", clone.display()))?,
        ],
    )?;
    run_git_in_dir(&clone, &["config", "user.email", "volt-tests@example.com"])?;
    run_git_in_dir(&clone, &["config", "user.name", "Volt Tests"])?;
    run_git_in_dir(&clone, &["config", "commit.gpgsign", "false"])?;
    std::fs::write(clone.join("remote.txt"), "from-remote\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&clone, &["add", "--", "remote.txt"])?;
    run_git_in_dir(&clone, &["commit", "-qm", "remote change"])?;
    run_git_in_dir(&clone, &["push", "-q", "origin", "HEAD"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    pull_git_upstream(&mut state.runtime, buffer_id)?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed popup was not opened for git pull".to_owned())?;
    assert!(
        shell_buffer(&state.runtime, popup.active_buffer)?
            .text
            .text()
            .contains("git pull")
    );
    wait_for_streamed_command_buffer_close(&mut state, popup.active_buffer)?;
    assert!(active_runtime_popup(&state.runtime)?.is_none());

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&remote).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&clone).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn fetch_git_prune_is_silent_command_without_popup() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-fetch-prune-silent")?;
    let remote = unique_temp_dir("git-fetch-prune-silent-remote");
    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            remote
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", remote.display()))?,
        ],
    )?;
    let branch = run_git_in_dir(&repo, &["symbolic-ref", "--short", "HEAD"])?
        .trim()
        .to_owned();
    run_git_in_dir(
        &repo,
        &["push", "-q", "--set-upstream", "origin", branch.as_str()],
    )?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    fetch_git_prune(&mut state.runtime, &repo)?;
    assert!(
        active_runtime_popup(&state.runtime)?.is_none(),
        "Silent Command must not open a Command Stream popup"
    );

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&remote).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_editor_confirm_writes_file_and_signals_stub() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let mut env = Vec::new();
    inject_git_editor_env(&mut state.runtime, &mut env)?;
    let dir = env
        .iter()
        .find(|(key, _)| key == VOLT_GIT_EDITOR_DIR_ENV)
        .map(|(_, value)| PathBuf::from(value))
        .ok_or_else(|| "VOLT_GIT_EDITOR_DIR missing".to_owned())?;
    let edit_path = dir.join("todo.txt");
    std::fs::write(&edit_path, "pick abc hello\n").map_err(|error| error.to_string())?;
    let request_id = "test-confirm";
    std::fs::write(
        dir.join(format!("request-{request_id}")),
        format!("{}\n", edit_path.display()),
    )
    .map_err(|error| error.to_string())?;

    assert!(refresh_pending_git_editor(&mut state.runtime)?);
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert!(matches!(
        &shell_buffer(&state.runtime, buffer_id)?.kind,
        BufferKind::Plugin(kind) if kind == GIT_EDITOR_KIND
    ));
    shell_buffer_mut(&mut state.runtime, buffer_id)?
        .replace_with_lines(vec!["pick abc hello edited".to_owned()]);
    confirm_git_editor_buffer(&mut state.runtime, buffer_id)?;

    let written = std::fs::read_to_string(&edit_path).map_err(|error| error.to_string())?;
    assert!(written.contains("edited"));
    let result = std::fs::read_to_string(dir.join(format!("result-{request_id}")))
        .map_err(|error| error.to_string())?;
    assert_eq!(result.trim(), "0");
    Ok(())
}

#[test]
fn git_line_is_untracked_uses_section_metadata() {
    let meta = SectionLineMeta {
        section_id: GIT_SECTION_UNTRACKED.to_owned(),
        kind: SectionRenderLineKind::Item,
        action: None,
    };
    let staged_meta = SectionLineMeta {
        section_id: GIT_SECTION_UNSTAGED.to_owned(),
        kind: SectionRenderLineKind::Item,
        action: None,
    };

    assert!(git_line_is_untracked(Some(&meta)));
    assert!(!git_line_is_untracked(Some(&staged_meta)));
    assert!(!git_line_is_untracked(None));
}

#[test]
fn git_status_commit_message_spans_use_command_token_with_icon_prefix() {
    let line = SectionRenderLine {
        text: format!(
            "{} Press c to commit staged changes.",
            editor_icons::symbols::cod::COD_GIT_COMMIT
        ),
        depth: 1,
        section_id: GIT_SECTION_COMMIT.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![(
            TOKEN_GIT_STATUS_COMMAND.to_owned(),
            format!(
                "{} Press c to commit staged changes.",
                editor_icons::symbols::cod::COD_GIT_COMMIT
            ),
        )]
    );
}

#[test]
fn hover_registry_includes_signature_help_provider() {
    let user_library = editor_plugin_host::NullUserLibrary;
    let registry = HoverRegistry::from_user_config(&user_library);
    assert!(matches!(registry.providers[0].kind, HoverProviderKind::Lsp));
    assert!(matches!(
        registry.providers[1].kind,
        HoverProviderKind::SignatureHelp
    ));
    assert_eq!(registry.providers[1].label, "Signature");
    assert_eq!(
        registry.providers[1].icon,
        user_library.hover_signature_icon()
    );
    assert!(matches!(
        registry.providers[2].kind,
        HoverProviderKind::Diagnostics
    ));
}

#[test]
fn statusline_icon_segments_split_acp_and_lsp_icons() {
    let user_library = user::UserLibraryImpl;
    let acp_icon = editor_icons::symbols::fa::FA_CONNECTDEVELOP;
    let lsp_icon = user_library.statusline_lsp_connected_icon();
    let statusline = format!("NORMAL | {acp_icon} | Ln 3, Col 9 | {lsp_icon} rust-analyzer");
    assert_eq!(
        statusline_icon_segments(&statusline, &[acp_icon, lsp_icon]),
        vec![
            ("NORMAL | ", false),
            (acp_icon, true),
            (" | Ln 3, Col 9 | ", false),
            (lsp_icon, true),
            (" rust-analyzer", false),
        ]
    );
}

#[test]
fn statusline_icon_segments_split_diagnostic_icons() {
    let user_library = user::UserLibraryImpl;
    let lsp_icon = user_library.statusline_lsp_connected_icon();
    let error_icon = user_library.statusline_lsp_error_icon();
    let warning_icon = user_library.statusline_lsp_warning_icon();
    let prefix = format!("NORMAL | {lsp_icon} rust-analyzer ");
    let statusline = format!("NORMAL | {lsp_icon} rust-analyzer {error_icon} 2 {warning_icon} 4");
    assert_eq!(
        statusline_icon_segments(&statusline, &[error_icon, warning_icon]),
        vec![
            (prefix.as_str(), false),
            (error_icon, true),
            (" 2 ", false),
            (warning_icon, true),
            (" 4", false),
        ]
    );
}

#[test]
fn notification_center_updates_entries_and_expires_completed_toasts() {
    let now = Instant::now();
    let mut center = NotificationCenter::default();
    assert!(center.apply(
        test_notification_update(
            "progress",
            NotificationSeverity::Info,
            "LSP · rust-analyzer",
            &["Indexing", "Scanning workspace"],
            Some(24),
            true,
        ),
        now,
    ));
    assert_eq!(center.visible(now).len(), 1);
    assert!(center.visible(now)[0].active);

    assert!(center.apply(
        test_notification_update(
            "progress",
            NotificationSeverity::Success,
            "LSP · rust-analyzer",
            &["Indexed workspace"],
            Some(100),
            false,
        ),
        now + Duration::from_millis(25),
    ));
    let visible = center.visible(now + Duration::from_millis(25));
    assert_eq!(visible.len(), 1);
    assert!(!visible[0].active);
    assert_eq!(visible[0].severity, NotificationSeverity::Success);

    assert!(!center.prune_expired(now + NOTIFICATION_AUTO_DISMISS - Duration::from_millis(1)));
    assert!(center.prune_expired(now + NOTIFICATION_AUTO_DISMISS + Duration::from_millis(50)));
    assert!(
        center
            .visible(now + NOTIFICATION_AUTO_DISMISS + Duration::from_millis(50))
            .is_empty()
    );
}

#[test]
fn notification_center_prioritizes_active_toasts_with_visible_limit() {
    let now = Instant::now();
    let mut center = NotificationCenter::default();
    assert!(center.apply(
        test_notification_update(
            "old-complete",
            NotificationSeverity::Success,
            "Done",
            &["Completed task"],
            None,
            false,
        ),
        now,
    ));
    assert!(center.apply(
        test_notification_update(
            "active-a",
            NotificationSeverity::Info,
            "Active A",
            &["Working"],
            Some(10),
            true,
        ),
        now + Duration::from_millis(10),
    ));
    assert!(center.apply(
        test_notification_update(
            "active-b",
            NotificationSeverity::Info,
            "Active B",
            &["Working"],
            Some(40),
            true,
        ),
        now + Duration::from_millis(20),
    ));
    assert!(center.apply(
        test_notification_update(
            "active-c",
            NotificationSeverity::Warning,
            "Active C",
            &["Working"],
            None,
            true,
        ),
        now + Duration::from_millis(30),
    ));
    assert!(center.apply(
        test_notification_update(
            "new-complete",
            NotificationSeverity::Success,
            "Done",
            &["Completed task"],
            None,
            false,
        ),
        now + Duration::from_millis(40),
    ));

    let visible = center.visible(now + Duration::from_millis(40));
    assert_eq!(visible.len(), NOTIFICATION_VISIBLE_LIMIT);
    assert!(visible.iter().all(|notification| notification.active));
    assert_eq!(visible[0].key, "active-c");
    assert_eq!(visible[1].key, "active-b");
    assert_eq!(visible[2].key, "active-a");
}

#[test]
fn notification_action_at_point_returns_acp_permission_action() -> Result<(), String> {
    let now = Instant::now();
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "acp.permission.42".to_owned(),
            severity: NotificationSeverity::Warning,
            title: "project Read file is requesting permission".to_owned(),
            body_lines: vec!["Allow once".to_owned(), "Reject once".to_owned()],
            progress: None,
            active: true,
            action: Some(NotificationAction::OpenAcpPermissionPicker { request_id: 42 }),
            workspace_id: None,
        },
        now,
    );

    let ui = shell_ui(&state.runtime)?;
    let layouts = notification_overlay_layouts(
        &ui.visible_notifications(now),
        render_width,
        render_height,
        cell_width,
        line_height,
    );
    let rect = layouts
        .first()
        .map(|layout| layout.rect)
        .ok_or_else(|| "notification layout missing".to_owned())?;
    let action = notification_action_at_point(
        ui,
        render_width,
        render_height,
        cell_width,
        line_height,
        now,
        (rect.x() + 4, rect.y() + 4),
    );

    assert_eq!(
        action,
        Some(NotificationAction::OpenAcpPermissionPicker { request_id: 42 })
    );
    Ok(())
}

#[test]
fn notification_action_at_point_returns_copilot_sign_in_action() -> Result<(), String> {
    let now = Instant::now();
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "copilot.sign-in".to_owned(),
            severity: NotificationSeverity::Error,
            title: "Copilot authentication required".to_owned(),
            body_lines: vec!["Click notification to sign in.".to_owned()],
            progress: None,
            active: true,
            action: Some(NotificationAction::CopilotSignIn {
                root: Some(PathBuf::from(r"P:\volt")),
            }),
            workspace_id: None,
        },
        now,
    );

    let ui = shell_ui(&state.runtime)?;
    let layouts = notification_overlay_layouts(
        &ui.visible_notifications(now),
        render_width,
        render_height,
        cell_width,
        line_height,
    );
    let rect = layouts
        .first()
        .map(|layout| layout.rect)
        .ok_or_else(|| "notification layout missing".to_owned())?;
    let action = notification_action_at_point(
        ui,
        render_width,
        render_height,
        cell_width,
        line_height,
        now,
        (rect.x() + 4, rect.y() + 4),
    );

    assert_eq!(
        action,
        Some(NotificationAction::CopilotSignIn {
            root: Some(PathBuf::from(r"P:\volt")),
        })
    );
    Ok(())
}

#[test]
fn copilot_auth_notification_shows_device_code_and_stays_active() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let key = copilot_status_notification_key(Some(Path::new(r"P:\volt")));
    apply_copilot_auth_notification(
        &mut state.runtime,
        &key,
        NotificationSeverity::Info,
        "Copilot sign-in started",
        vec![
            "Device code: ABCD-EFGH".to_owned(),
            "Code copied to clipboard.".to_owned(),
            "Enter code in GitHub browser flow.".to_owned(),
        ],
        true,
    )?;

    let now = Instant::now();
    let ui = shell_ui(&state.runtime)?;
    let notification = ui
        .visible_notifications(now)
        .into_iter()
        .find(|notification| notification.key == key)
        .ok_or_else(|| "copilot auth notification missing".to_owned())?;

    assert_eq!(notification.body_lines[0], "Device code: ABCD-EFGH");
    assert!(notification.active);
    Ok(())
}
