use super::*;

#[cfg(windows)]
#[test]
fn normalize_git_output_path_converts_git_for_windows_drive_roots() {
    assert_eq!(
        normalize_git_output_path("/p/volt/target/release/user"),
        PathBuf::from(r"P:\volt\target\release\user")
    );
    assert_eq!(
        normalize_git_output_path(r"P:\volt\target\release\user"),
        PathBuf::from(r"P:\volt\target\release\user")
    );
    assert_eq!(
        normalize_git_output_path("w:/w/ftc-ui-web"),
        PathBuf::from(r"W:\w\ftc-ui-web")
    );
    assert_eq!(normalize_git_output_path(".git"), PathBuf::from(".git"));
}

#[cfg(windows)]
#[test]
fn worktree_remove_uses_porcelain_raw_path_not_normalized_windows_path() {
    let entries = parse_git_worktree_list(
        "\
worktree /w/ftc-ui-web
bare

worktree /w/ftc-ui-web/map
HEAD a3dcf8f90bfe54a1bffb3c505ec878c8566986fd
branch refs/heads/feature/TASK-5645-mapchanges

worktree W:\\ftc-ui-web/main
HEAD 3811c5ec536197500efa15290940d47f3f55cff5
branch refs/heads/origin-main
",
    )
    .expect("porcelain parses");

    let invocation =
        worktree_remove_git_invocation_for_entries(&entries, Path::new(r"W:\ftc-ui-web\map"));
    assert_eq!(
        invocation,
        WorktreeRemoveGitInvocation::Remove {
            cli_path: "/w/ftc-ui-web/map".to_owned(),
        },
        "Git only recognizes the registered MSYS spelling"
    );
    assert_eq!(
        worktree_remove_git_args(&invocation),
        vec![
            "worktree".to_owned(),
            "remove".to_owned(),
            "/w/ftc-ui-web/map".to_owned(),
            "--force".to_owned(),
        ]
    );
}

#[cfg(windows)]
#[test]
fn worktree_remove_prunes_when_porcelain_marks_entry_prunable() {
    let entries = parse_git_worktree_list(
        "\
worktree /w/ftc-ui-web/map
HEAD a3dcf8f90bfe54a1bffb3c505ec878c8566986fd
branch refs/heads/feature/TASK-5645-mapchanges
prunable gitdir file points to non-existent location
",
    )
    .expect("porcelain parses");

    let invocation =
        worktree_remove_git_invocation_for_entries(&entries, Path::new(r"W:\ftc-ui-web\map"));
    assert_eq!(invocation, WorktreeRemoveGitInvocation::Prune);
    assert_eq!(
        worktree_remove_git_args(&invocation),
        vec!["worktree".to_owned(), "prune".to_owned(), "-v".to_owned()]
    );
}

#[test]
fn oil_git_worktree_command_opens_branch_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let remote = unique_temp_dir("oil-worktree-remote");
    let repo = init_git_repo_with_commit("oil-worktree-repo")?;

    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &["remote", "add", "origin", remote.to_str().unwrap_or("")],
    )?;
    run_git_in_dir(&repo, &["push", "-u", "origin", "HEAD:master"])?;
    run_git_in_dir(&repo, &["checkout", "-qb", "feature/oil-worktree"])?;
    std::fs::write(repo.join("feature.txt"), "feature\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "feature.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "feature"])?;
    run_git_in_dir(&repo, &["push", "-u", "origin", "feature/oil-worktree"])?;
    run_git_in_dir(&repo, &["checkout", "-q", "master"])?;

    let workspace_root = repo
        .parent()
        .ok_or_else(|| "repo parent missing".to_owned())?
        .to_path_buf();
    open_workspace_from_project(&mut state.runtime, "oil-worktree", &workspace_root)?;
    open_oil_directory(&mut state.runtime, repo.clone())?;
    state
        .runtime
        .execute_command("oil.git-worktree")
        .map_err(|error| error.to_string())?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "oil.git-worktree did not open picker".to_owned())?;
    assert!(
        picker
            .session()
            .matches()
            .iter()
            .any(|entry| entry.item().label() == "New Branch")
    );
    assert!(
        picker
            .session()
            .matches()
            .iter()
            .any(|entry| entry.item().label() == "origin/feature/oil-worktree")
    );

    let _ = std::fs::remove_dir_all(&repo);
    let _ = std::fs::remove_dir_all(&remote);
    Ok(())
}

#[test]
fn oil_git_worktree_new_branch_prompts_for_name_then_directory() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let remote = unique_temp_dir("oil-worktree-new-remote");
    let repo = init_git_repo_with_commit("oil-worktree-new-repo")?;

    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &["remote", "add", "origin", remote.to_str().unwrap_or("")],
    )?;
    run_git_in_dir(&repo, &["push", "-u", "origin", "HEAD:master"])?;

    let workspace_root = repo
        .parent()
        .ok_or_else(|| "repo parent missing".to_owned())?
        .to_path_buf();
    open_workspace_from_project(&mut state.runtime, "oil-worktree-new", &workspace_root)?;
    open_oil_directory(&mut state.runtime, repo.clone())?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    state
        .runtime
        .execute_command("oil.git-worktree")
        .map_err(|error| error.to_string())?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "oil.git-worktree did not open picker".to_owned())?;
    assert_eq!(
        picker
            .session()
            .selected()
            .map(|entry| entry.item().label()),
        Some("New Branch")
    );

    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;

    assert!(
        shell_ui(&state.runtime)?
            .command_line()
            .is_some_and(|command_line| {
                matches!(
                    command_line.purpose(),
                    CommandLinePurpose::GitWorktreeNewBranch { .. }
                )
            }),
        "New Branch should open the branch-name command line"
    );
    assert!(shell_ui(&state.runtime)?.picker().is_none());

    state
        .handle_text_input("feature/new-oil-branch")
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Return, Mod::NOMOD)
        .map_err(|error| error.to_string())?;

    assert!(shell_ui(&state.runtime)?.command_line().is_none());
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    let pending = shell_buffer(&state.runtime, buffer_id)?
        .directory_state()
        .ok_or_else(|| "directory state missing".to_owned())?
        .pending_worktree
        .clone()
        .ok_or_else(|| "pending worktree request missing".to_owned())?;
    assert_eq!(pending.local_branch, "feature/new-oil-branch");
    assert_eq!(pending.remote_branch, "feature/new-oil-branch");
    assert!(pending.create_new_branch);

    let _ = std::fs::remove_dir_all(&repo);
    let _ = std::fs::remove_dir_all(&remote);
    Ok(())
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
fn acp_at_symbol_opens_git_file_picker_and_return_inserts_mention() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = init_git_repo("acp-files")?;
    fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    fs::write(root.join("src").join("main.rs"), "fn main() {}\n")
        .map_err(|error| error.to_string())?;
    run_git_in_dir(&root, &["add", "src/main.rs"])?;
    open_workspace_from_project(&mut state.runtime, "acp-files", &root)
        .map_err(|error| error.to_string())?;

    let buffer_id = install_acp_test_buffer(&mut state, 0, "look at ", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .handle_text_input("@")
        .map_err(|error| error.to_string())?;

    {
        let ui = shell_ui(&state.runtime)?;
        let picker = ui
            .picker()
            .ok_or_else(|| "ACP file picker should open for @".to_owned())?;
        assert_eq!(picker.session().title(), "ACP Files");
        assert!(
            picker
                .session()
                .matches()
                .iter()
                .any(|matched| matched.item().label() == "src/main.rs"),
            "git file picker should list src/main.rs"
        );
        assert_eq!(ui.picker_kind(), Some(PickerKind::AcpFile { buffer_id }));
    }

    state
        .handle_text_input("main.rs")
        .map_err(|error| error.to_string())?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
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
        "look at @src/main.rs "
    );
    let _ = fs::remove_dir_all(root);
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
fn debug_fringe_is_one_cell_when_idle_and_two_when_live() {
    assert_eq!(debug_fringe_cell_count(false), 1);
    assert_eq!(debug_fringe_cell_count(true), 2);
    assert_eq!(editor_fringe_width_px(8, false), 8);
    assert_eq!(editor_fringe_width_px(8, true), 16);
}

#[test]
fn toggle_breakpoint_without_session_shows_idle_fringe_marker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-idle-fringe");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("Program.cs");
    fs::write(&program, "class Program { static void Main() {} }\n").map_err(|e| e.to_string())?;
    let buffer_id = open_workspace_file(&mut state.runtime, &program)?;

    toggle_dap_breakpoint_at_cursor(&mut state.runtime)?;
    sync_active_buffer(&mut state.runtime)?;

    let focused = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        focused, buffer_id,
        "toggling a Breakpoint must not switch away from the editor buffer"
    );
    let focused_name = shell_ui(&state.runtime)?
        .buffer(focused)
        .ok_or_else(|| "focused buffer missing".to_owned())?
        .display_name()
        .to_owned();
    assert_ne!(
        focused_name, DAP_BREAKPOINTS_BUFFER_NAME,
        "toggle must not open `{DAP_BREAKPOINTS_BUFFER_NAME}`"
    );

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "buffer missing".to_owned())?;
    assert!(
        !buffer.dap_fringe_live(),
        "idle Workspace must keep one-cell fringe (no live Session)"
    );
    assert_eq!(
        buffer.dap_fringe_marker(0),
        Some(BreakpointState::Pending),
        "Breakpoint must appear in Debug Fringe before a Session starts"
    );

    let workspace_id = workspace.get();
    let listed = state
        .runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .ok_or_else(|| "dap manager missing".to_owned())?
        .list_breakpoints(workspace_id)
        .map_err(|e| e.to_string())?;
    assert_eq!(listed.len(), 1);
    let _ = fs::remove_dir_all(&root);
    Ok(())
}
