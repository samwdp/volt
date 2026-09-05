pub(super) fn begin_oil_worktree_request(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    trace_oil_worktree(
        runtime,
        format!("begin oil worktree request for buffer `{buffer_id}`"),
    );
    let root = git_root(runtime)?;
    trace_oil_worktree(runtime, format!("resolved git root `{}`", root.display()));
    let branches = git_remote_worktree_branch_list(runtime, &root)?;
    let mut entries = branches
        .into_iter()
        .map(|(remote_branch, local_branch)| {
            let item_id = format!("git-worktree-oil-branch:{remote_branch}");
            let action = PickerAction::GitWorktreeOilBranch {
                buffer_id,
                remote_branch: remote_branch.clone(),
                local_branch: local_branch.clone(),
            };
            PickerEntry {
                item: PickerItem::new(
                    item_id,
                    remote_branch,
                    format!("create local branch {local_branch}"),
                    None::<String>,
                ),
                action,
                quickfix: None,
            }
        })
        .collect::<Vec<_>>();
    entries.insert(
        0,
        PickerEntry {
            item: PickerItem::new(
                "git-worktree-oil-branch:new",
                "New Branch",
                "create a new local branch and worktree",
                Some("Enter a branch name, then choose the worktree directory in oil.".to_owned()),
            ),
            action: PickerAction::GitWorktreeOilNewBranch { buffer_id },
            quickfix: None,
        },
    );
    let entry_count = entries.len();
    shell_ui_mut(runtime)?.set_picker(
        PickerOverlay::from_entries("Git Worktree Branch", entries)
            .with_result_order(PickerResultOrder::Source),
    );
    trace_oil_worktree(
        runtime,
        format!("set git worktree picker with {entry_count} entries"),
    );
    Ok(())
}

pub(super) fn open_git_worktree_new_branch_prompt(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.set_command_line(CommandLineOverlay::for_worktree_new_branch(buffer_id));
    Ok(())
}

pub(super) fn submit_git_worktree_new_branch_name(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    branch: &str,
) -> Result<(), String> {
    let branch = branch.trim();
    if branch.is_empty() {
        return Err("branch name is required".to_owned());
    }
    if branch.chars().any(char::is_whitespace) {
        return Err("branch name must not contain whitespace".to_owned());
    }
    finish_oil_worktree_branch_selection(runtime, buffer_id, branch, branch, true)?;
    sync_active_buffer(runtime)
}

pub(super) fn open_git_worktree_path_picker(
    runtime: &mut EditorRuntime,
    remote_branch: &str,
    local_branch: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let base_dir = root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.clone());
    let mut picker = PickerOverlay::from_entries(
        format!("Worktree directory for {remote_branch}"),
        Vec::new(),
    );
    picker.submit_action = Some(PickerAction::GitWorktreeCreate {
        remote_branch: remote_branch.to_owned(),
        local_branch: local_branch.to_owned(),
        base_dir,
    });
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(super) fn create_git_worktree_from_query(
    runtime: &mut EditorRuntime,
    remote_branch: &str,
    local_branch: &str,
    base_dir: &Path,
    query: &str,
) -> Result<(), String> {
    let name = query.trim();
    if name.is_empty() {
        return Err("worktree directory name is required".to_owned());
    }
    let worktree_path = worktree_path_from_name(base_dir, name)?;
    create_git_worktree(runtime, remote_branch, local_branch, &worktree_path, false)
}

pub(super) fn finish_oil_worktree_branch_selection(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    remote_branch: &str,
    local_branch: &str,
    create_new_branch: bool,
) -> Result<(), String> {
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.move_line_end();
    buffer.insert_text("\n");
    let line = buffer.cursor_point().line;
    buffer.set_cursor(TextPoint::new(line, buffer.line_len_chars(line)));
    let state = buffer
        .directory_state_mut()
        .ok_or_else(|| "directory state is missing".to_owned())?;
    state.pending_worktree = Some(PendingWorktreeRequest {
        line,
        remote_branch: remote_branch.to_owned(),
        local_branch: local_branch.to_owned(),
        create_new_branch,
    });
    shell_ui_mut(runtime)?.enter_insert_mode();
    Ok(())
}

pub(super) fn create_git_worktree(
    runtime: &mut EditorRuntime,
    remote_branch: &str,
    local_branch: &str,
    worktree_path: &Path,
    create_new_branch: bool,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    if worktree_path.exists() {
        return Err(format!(
            "worktree path already exists: {}",
            worktree_path.display()
        ));
    }
    let path_arg = worktree_path.display().to_string();
    let args = if create_new_branch {
        vec![
            "worktree".to_owned(),
            "add".to_owned(),
            "-b".to_owned(),
            local_branch.to_owned(),
            path_arg,
        ]
    } else if remote_branch == local_branch {
        vec![
            "worktree".to_owned(),
            "add".to_owned(),
            path_arg,
            local_branch.to_owned(),
        ]
    } else {
        vec![
            "worktree".to_owned(),
            "add".to_owned(),
            "--track".to_owned(),
            "-b".to_owned(),
            local_branch.to_owned(),
            path_arg,
            remote_branch.to_owned(),
        ]
    };
    let name = worktree_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(local_branch)
        .to_owned();
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Worktree Add",
            args,
            root,
            StreamedCommandExitAction::RefreshGitStatusCloseAndOpenWorkspace {
                name,
                path: worktree_path.to_path_buf(),
            },
        ),
    )?;
    Ok(())
}

pub(super) fn worktree_path_from_name(base_dir: &Path, name: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(name);
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("worktree path must not contain `..`".to_owned());
    }
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(base_dir.join(path))
    }
}

pub(super) fn checkout_git_branch(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "checkout", &["checkout", branch])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn take_git_prefix(runtime: &mut EditorRuntime) -> Result<Option<GitPrefix>, String> {
    const PREFIX_TIMEOUT: Duration = Duration::from_millis(1200);
    let now = Instant::now();
    let ui = shell_ui_mut(runtime)?;
    let prefix = match ui.pending_git_prefix.take() {
        Some(state) if now.duration_since(state.started_at) <= PREFIX_TIMEOUT => {
            Some(state.prefix())
        }
        _ => None,
    };
    Ok(prefix)
}

pub(super) fn set_git_prefix(runtime: &mut EditorRuntime, prefix: GitPrefix) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    ui.pending_git_prefix = Some(GitPrefixState {
        prefix,
        started_at: Instant::now(),
    });
    Ok(())
}

pub(super) type ShellCommandHandler = fn(&mut EditorRuntime) -> Result<(), String>;

#[derive(Debug, Clone)]
pub(super) struct GitStatusCommandContext {
    buffer_id: BufferId,
    meta: Option<SectionLineMeta>,
    staged_empty: bool,
    has_stage_candidates: bool,
}

pub(super) const GIT_STATUS_COMMANDS: &[(&str, &str, ShellCommandHandler)] = &[
    (
        "git.status.refresh",
        "Refresh the active git status buffer.",
        git_status_refresh_command,
    ),
    (
        "git.status.next-section",
        "Move to the next git status section.",
        git_status_next_section_command,
    ),
    (
        "git.status.previous-section",
        "Move to the previous git status section.",
        git_status_previous_section_command,
    ),
    (
        "git.status.stage",
        "Stage the selected file or all unstaged changes.",
        git_status_stage_command,
    ),
    (
        "git.status.stage-all",
        "Stage all unstaged changes.",
        git_status_stage_all_command,
    ),
    (
        "git.status.unstage",
        "Unstage the selected file.",
        git_status_unstage_command,
    ),
    (
        "git.status.unstage-all",
        "Unstage all staged changes.",
        git_status_unstage_all_command,
    ),
    (
        "git.status.commit",
        "Open the git commit buffer for staged changes.",
        git_status_commit_command,
    ),
    (
        "git.status.push-pushremote",
        "Push to the configured push-remote from the git status buffer.",
        git_status_push_pushremote_command,
    ),
    (
        "git.status.push-upstream",
        "Push to the configured upstream from the git status buffer.",
        git_status_push_upstream_command,
    ),
    (
        "git.status.fetch-pushremote",
        "Fetch from the configured push-remote from the git status buffer.",
        git_status_fetch_pushremote_command,
    ),
    (
        "git.status.fetch-upstream",
        "Fetch from the configured upstream from the git status buffer.",
        git_status_fetch_upstream_command,
    ),
    (
        "git.status.fetch-all",
        "Fetch all remotes.",
        git_status_fetch_all_command,
    ),
    (
        "git.status.pull-upstream",
        "Pull from the configured upstream from the git status buffer.",
        git_status_pull_upstream_command,
    ),
    (
        "git.status.branches",
        "Open the git branch picker.",
        git_status_branches_command,
    ),
    (
        "git.worktree.create",
        "Create a worktree from a remote-only branch.",
        git_worktree_create_command,
    ),
    (
        "git.status.merge",
        "Merge a branch from the git status buffer.",
        git_status_merge_command,
    ),
    (
        "git.status.merge-edit",
        "Merge a branch and edit the merge message.",
        git_status_merge_edit_command,
    ),
    (
        "git.status.merge-no-commit",
        "Merge a branch without committing.",
        git_status_merge_no_commit_command,
    ),
    (
        "git.status.merge-squash",
        "Squash-merge a branch from the git status buffer.",
        git_status_merge_squash_command,
    ),
    (
        "git.status.merge-preview",
        "Preview the diff for a branch merge.",
        git_status_merge_preview_command,
    ),
    (
        "git.status.merge-abort",
        "Abort the current git merge.",
        git_status_merge_abort_command,
    ),
    (
        "git.status.rebase-pushremote",
        "Rebase onto the configured push-remote from the git status buffer.",
        git_status_rebase_pushremote_command,
    ),
    (
        "git.status.rebase-upstream",
        "Rebase onto the configured upstream from the git status buffer.",
        git_status_rebase_upstream_command,
    ),
    (
        "git.status.rebase-onto",
        "Rebase onto a selected branch or edit the current rebase todo.",
        git_status_rebase_onto_command,
    ),
    (
        "git.status.rebase-interactive",
        "Start an interactive rebase from the git status buffer.",
        git_status_rebase_interactive_command,
    ),
    (
        "git.status.rebase-continue",
        "Continue the current git rebase.",
        git_status_rebase_continue_command,
    ),
    (
        "git.status.rebase-skip",
        "Skip the current git rebase commit.",
        git_status_rebase_skip_command,
    ),
    (
        "git.status.rebase-abort",
        "Abort the current git rebase.",
        git_status_rebase_abort_command,
    ),
    (
        "git.status.rebase-autosquash",
        "Autosquash the current git rebase.",
        git_status_rebase_autosquash_command,
    ),
    (
        "git.status.rebase-edit-commit",
        "Edit a commit during the current git rebase.",
        git_status_rebase_edit_commit_command,
    ),
    (
        "git.status.rebase-reword",
        "Reword a commit during the current git rebase.",
        git_status_rebase_reword_command,
    ),
    (
        "git.status.rebase-remove-commit",
        "Remove a commit during the current git rebase.",
        git_status_rebase_remove_commit_command,
    ),
    (
        "git.status.diff-dwim",
        "Open the most relevant git diff for the current status line.",
        git_status_diff_dwim_command,
    ),
    (
        "git.status.diff-staged",
        "Open the staged git diff buffer.",
        git_status_diff_staged_command,
    ),
    (
        "git.status.diff-unstaged",
        "Open the unstaged git diff buffer.",
        git_status_diff_unstaged_command,
    ),
    (
        "git.status.diff-commit",
        "Open a git diff for the commit at point or HEAD.",
        git_status_diff_commit_command,
    ),
    (
        "git.status.diff-stash",
        "Open a git diff for the stash at point.",
        git_status_diff_stash_command,
    ),
    (
        "git.status.diff-range",
        "Diff a git range from the git status buffer.",
        git_status_diff_range_command,
    ),
    (
        "git.status.diff-paths",
        "Diff selected paths from the git status buffer.",
        git_status_diff_paths_command,
    ),
    (
        "git.status.log-head",
        "Open the git log for HEAD.",
        git_status_log_head_command,
    ),
    (
        "git.status.log-related",
        "Open the git log for the branch, upstream, and push-remote related to the status buffer.",
        git_status_log_related_command,
    ),
    (
        "git.status.log-other",
        "Open another git log view from the git status buffer.",
        git_status_log_other_command,
    ),
    (
        "git.status.log-branches",
        "Open the git log for local branches.",
        git_status_log_branches_command,
    ),
    (
        "git.status.log-all-branches",
        "Open the git log for local and remote branches.",
        git_status_log_all_branches_command,
    ),
    (
        "git.status.log-all",
        "Open the git log for all refs.",
        git_status_log_all_command,
    ),
    (
        "git.status.stash-both",
        "Stash both index and worktree changes.",
        git_status_stash_both_command,
    ),
    (
        "git.status.stash-index",
        "Stash staged changes.",
        git_status_stash_index_command,
    ),
    (
        "git.status.stash-worktree",
        "Stash worktree changes.",
        git_status_stash_worktree_command,
    ),
    (
        "git.status.stash-keep-index",
        "Stash changes while keeping the index.",
        git_status_stash_keep_index_command,
    ),
    (
        "git.status.stash-apply",
        "Apply the stash at point.",
        git_status_stash_apply_command,
    ),
    (
        "git.status.stash-pop",
        "Pop the stash at point.",
        git_status_stash_pop_command,
    ),
    (
        "git.status.stash-drop",
        "Drop the stash at point.",
        git_status_stash_drop_command,
    ),
    (
        "git.status.stash-show",
        "Show the stash diff at point.",
        git_status_stash_show_command,
    ),
    (
        "git.status.cherry-open",
        "Open the git cherry buffer for the current upstream.",
        git_status_cherry_open_command,
    ),
    (
        "git.status.cherry-pick",
        "Cherry-pick the commit at point or continue an active sequence.",
        git_status_cherry_pick_command,
    ),
    (
        "git.status.cherry-pick-apply",
        "Apply the commit at point without committing, or abort an active sequence.",
        git_status_cherry_pick_apply_command,
    ),
    (
        "git.status.cherry-pick-skip",
        "Skip the current cherry-pick sequence step.",
        git_status_cherry_pick_skip_command,
    ),
    (
        "git.status.revert",
        "Revert the commit at point or continue an active revert sequence.",
        git_status_revert_command,
    ),
    (
        "git.status.revert-no-commit",
        "Revert the commit at point without committing, or abort an active revert sequence.",
        git_status_revert_no_commit_command,
    ),
    (
        "git.status.revert-skip",
        "Skip the current cherry-pick or revert sequence step.",
        git_status_revert_skip_command,
    ),
    (
        "git.status.revert-abort",
        "Abort the current cherry-pick or revert sequence.",
        git_status_revert_abort_command,
    ),
    (
        "git.status.apply-commit",
        "Apply the commit at point without committing, or open the commit picker.",
        git_status_apply_commit_command,
    ),
    (
        "git.status.reset-mixed",
        "Reset to the selected commit with --mixed.",
        git_status_reset_mixed_command,
    ),
    (
        "git.status.reset-soft",
        "Reset to the selected commit with --soft.",
        git_status_reset_soft_command,
    ),
    (
        "git.status.reset-hard",
        "Reset to the selected commit with --hard.",
        git_status_reset_hard_command,
    ),
    (
        "git.status.reset-keep",
        "Reset to the selected commit with --keep.",
        git_status_reset_keep_command,
    ),
    (
        "git.status.reset-index",
        "Reset the git index from the git status buffer.",
        git_status_reset_index_command,
    ),
    (
        "git.status.reset-worktree",
        "Reset the git worktree from the git status buffer.",
        git_status_reset_worktree_command,
    ),
    (
        "git.status.checkout-file",
        "Check out a file from the git status buffer.",
        git_status_checkout_file_command,
    ),
    (
        "git.status.discard-or-reset",
        "Delete selected git status targets or reset the commit at point.",
        git_status_discard_or_reset_command,
    ),
];

pub(super) fn register_git_status_commands(runtime: &mut EditorRuntime) -> Result<(), String> {
    for &(name, description, handler) in GIT_STATUS_COMMANDS {
        runtime
            .register_command(name, description, CommandSource::Core, handler)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(super) fn active_git_status_command_context(
    runtime: &EditorRuntime,
) -> Result<GitStatusCommandContext, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_git_status(&buffer.kind) {
        return Err("git status buffer is not active".to_owned());
    }
    let snapshot = buffer.git_snapshot();
    Ok(GitStatusCommandContext {
        buffer_id,
        meta: buffer
            .section_line_meta(buffer.cursor_point().line)
            .cloned(),
        staged_empty: snapshot
            .map(|snapshot| snapshot.staged().is_empty())
            .unwrap_or(true),
        has_stage_candidates: snapshot
            .map(|snapshot| !(snapshot.unstaged().is_empty() && snapshot.untracked().is_empty()))
            .unwrap_or(false),
    })
}

pub(super) fn ensure_no_rebase_in_progress(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_rebase_in_progress(runtime)? {
        return Err("rebase already in progress".to_owned());
    }
    Ok(())
}

pub(super) fn ensure_rebase_in_progress(
    runtime: &mut EditorRuntime,
    message: &str,
) -> Result<(), String> {
    if !git_rebase_in_progress(runtime)? {
        return Err(message.to_owned());
    }
    Ok(())
}

pub(super) fn git_status_sequence_kind(
    runtime: &mut EditorRuntime,
    message: &str,
) -> Result<GitSequenceKind, String> {
    git_sequence_in_progress(runtime)?.ok_or_else(|| message.to_owned())
}

pub(super) fn unsupported_git_status_command(message: &str) -> Result<(), String> {
    Err(message.to_owned())
}

pub(super) fn git_status_refresh_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    refresh_git_status_buffer(runtime, context.buffer_id)
}

pub(super) fn git_status_next_section_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    move_git_section(runtime, true).map(|_| ())
}

pub(super) fn git_status_previous_section_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    move_git_section(runtime, false).map(|_| ())
}

pub(super) fn git_status_stage_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    let (targets, is_visual) =
        git_status_action_targets(runtime, context.buffer_id, GIT_ACTION_STAGE_FILE)?;
    if !targets.is_empty() {
        stage_git_files(runtime, &targets)?;
        if is_visual {
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        return Ok(());
    }
    if is_visual {
        return Err("no stageable files selected".to_owned());
    }
    if !context.has_stage_candidates {
        return Err("no unstaged changes to stage".to_owned());
    }
    stage_git_all(runtime)
}

pub(super) fn git_status_stage_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if !context.has_stage_candidates {
        return Err("no unstaged changes to stage".to_owned());
    }
    stage_git_all(runtime)
}

pub(super) fn git_status_unstage_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    let (targets, is_visual) =
        git_status_action_targets(runtime, context.buffer_id, GIT_ACTION_UNSTAGE_FILE)?;
    if !targets.is_empty() {
        unstage_git_files(runtime, &targets)?;
        if is_visual {
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        return Ok(());
    }
    if is_visual {
        return Err("no staged files selected".to_owned());
    }
    Ok(())
}

pub(super) fn git_status_unstage_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if context.staged_empty {
        return Err("no staged changes to unstage".to_owned());
    }
    unstage_git_all(runtime)
}

pub(super) fn git_status_commit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if context.staged_empty {
        return Err("no staged changes to commit".to_owned());
    }
    open_git_commit_buffer(runtime)
}

pub(super) fn git_status_push_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    push_git_to_pushremote(runtime, context.buffer_id)
}

pub(super) fn git_status_push_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    push_git_to_upstream(runtime, context.buffer_id)
}

pub(super) fn git_status_fetch_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    fetch_git_pushremote(runtime, context.buffer_id)
}

pub(super) fn git_status_fetch_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    fetch_git_upstream(runtime, context.buffer_id)
}

pub(super) fn git_status_fetch_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    fetch_git_all(runtime)
}

pub(super) fn git_status_pull_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    pull_git_upstream(runtime, context.buffer_id)
}

pub(super) fn git_status_branches_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker(runtime)
}

pub(super) fn git_worktree_create_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_worktree_branch_picker(runtime)
}

fn trace_oil_worktree(runtime: &mut EditorRuntime, message: impl Into<String>) {
    let message = message.into();
    eprintln!("[oil.git-worktree] {message}");
    record_runtime_error(runtime, "oil.git-worktree.trace", message);
}

pub(super) fn oil_git_worktree_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    trace_oil_worktree(runtime, "oil.git-worktree command invoked");
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (buffer_name, buffer_kind, is_directory) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        (
            buffer.name.clone(),
            buffer_kind_label(&buffer.kind),
            buffer_is_directory(&buffer.kind),
        )
    };
    trace_oil_worktree(
        runtime,
        format!("active buffer `{buffer_name}` kind `{buffer_kind}`"),
    );
    if !is_directory {
        return Err("oil.git-worktree requires an oil buffer".to_owned());
    }
    begin_oil_worktree_request(runtime, buffer_id)
}

pub(super) fn git_status_merge_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_merge_in_progress(runtime)? {
        return merge_git_continue(runtime);
    }
    open_git_branch_picker_with_action(runtime, "Git Merge", GitBranchActionKind::MergePlain)
}

pub(super) fn git_status_merge_edit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Edit Message)",
        GitBranchActionKind::MergeEdit,
    )
}

pub(super) fn git_status_merge_no_commit_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (No Commit)",
        GitBranchActionKind::MergeNoCommit,
    )
}

pub(super) fn git_status_merge_squash_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Squash)",
        GitBranchActionKind::MergeSquash,
    )
}

pub(super) fn git_status_merge_preview_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Preview)",
        GitBranchActionKind::MergePreview,
    )
}

pub(super) fn git_status_merge_abort_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    merge_git_abort(runtime)
}

pub(super) fn git_status_rebase_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    let context = active_git_status_command_context(runtime)?;
    rebase_git_onto_pushremote(runtime, context.buffer_id)
}

pub(super) fn git_status_rebase_upstream_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    let context = active_git_status_command_context(runtime)?;
    rebase_git_onto_upstream(runtime, context.buffer_id)
}

pub(super) fn git_status_rebase_onto_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_rebase_in_progress(runtime)? {
        return rebase_git_edit_todo(runtime);
    }
    open_git_branch_picker_with_action(runtime, "Git Rebase", GitBranchActionKind::RebaseOnto)
}

pub(super) fn git_status_rebase_interactive_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    open_git_branch_picker_with_action(
        runtime,
        "Git Rebase (Interactive)",
        GitBranchActionKind::RebaseInteractive,
    )
}

pub(super) fn git_status_rebase_continue_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "no rebase in progress")?;
    rebase_git_continue(runtime)
}

pub(super) fn git_status_rebase_skip_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "rebase subset is not supported yet")?;
    rebase_git_skip(runtime)
}

pub(super) fn git_status_rebase_abort_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "no rebase in progress")?;
    rebase_git_abort(runtime)
}

pub(super) fn git_status_rebase_autosquash_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase autosquash is not supported yet")
}

pub(super) fn git_status_rebase_edit_commit_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase edit-commit is not supported yet")
}

pub(super) fn git_status_rebase_reword_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase reword is not supported yet")
}

pub(super) fn git_status_rebase_remove_commit_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase remove-commit is not supported yet")
}

pub(super) fn git_status_diff_dwim_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_dwim(runtime, context.buffer_id, context.meta.as_ref(), "")
}

pub(super) fn git_status_diff_staged_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_diff_staged(runtime)
}

pub(super) fn git_status_diff_unstaged_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_diff_unstaged(runtime)
}

pub(super) fn git_status_diff_commit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_commit_at_point(runtime, context.buffer_id, context.meta.as_ref())
}

pub(super) fn git_status_diff_stash_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_stash_at_point(runtime, context.meta.as_ref())
}

pub(super) fn git_status_diff_range_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git diff range is not supported yet")
}

pub(super) fn git_status_diff_paths_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git diff paths is not supported yet")
}

pub(super) fn git_status_log_head_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_head(runtime)
}

pub(super) fn git_status_log_related_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    open_git_log_related(runtime, context.buffer_id)
}

pub(super) fn git_status_log_other_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git log other is not supported yet")
}

pub(super) fn git_status_log_branches_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_branches(runtime)
}

pub(super) fn git_status_log_all_branches_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    open_git_log_all_branches(runtime)
}

pub(super) fn git_status_log_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_all(runtime)
}

pub(super) fn git_status_stash_both_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_both(runtime)
}

pub(super) fn git_status_stash_index_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_index(runtime)
}

pub(super) fn git_status_stash_worktree_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_worktree(runtime)
}
