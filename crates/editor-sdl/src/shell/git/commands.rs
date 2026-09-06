use super::super::*;

use super::commit::*;
use super::diff::*;
use super::log::*;
use super::merge_rebase::*;
use super::pickers::*;
use super::remote::*;
use super::staging::*;
use super::stash::*;
use super::status::*;
use super::worktree::*;

#[derive(Debug, Clone)]
pub(crate) struct GitStatusCommandContext {
    pub(crate) buffer_id: BufferId,
    pub(crate) meta: Option<SectionLineMeta>,
    pub(crate) staged_empty: bool,
    pub(crate) has_stage_candidates: bool,
}

pub(crate) const GIT_STATUS_COMMANDS: &[(&str, &str, ShellCommandHandler)] = &[
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

pub(crate) fn register_git_status_commands(runtime: &mut EditorRuntime) -> Result<(), String> {
    for &(name, description, handler) in GIT_STATUS_COMMANDS {
        runtime
            .register_command(name, description, CommandSource::Core, handler)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn active_git_status_command_context(
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

pub(crate) fn ensure_no_rebase_in_progress(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_rebase_in_progress(runtime)? {
        return Err("rebase already in progress".to_owned());
    }
    Ok(())
}

pub(crate) fn ensure_rebase_in_progress(
    runtime: &mut EditorRuntime,
    message: &str,
) -> Result<(), String> {
    if !git_rebase_in_progress(runtime)? {
        return Err(message.to_owned());
    }
    Ok(())
}

pub(crate) fn unsupported_git_status_command(message: &str) -> Result<(), String> {
    Err(message.to_owned())
}

pub(crate) fn git_status_refresh_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    refresh_git_status_buffer(runtime, context.buffer_id)
}

pub(crate) fn git_status_next_section_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    move_git_section(runtime, true).map(|_| ())
}

pub(crate) fn git_status_previous_section_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    move_git_section(runtime, false).map(|_| ())
}

pub(crate) fn git_status_stage_command(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(crate) fn git_status_stage_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if !context.has_stage_candidates {
        return Err("no unstaged changes to stage".to_owned());
    }
    stage_git_all(runtime)
}

pub(crate) fn git_status_unstage_command(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(crate) fn git_status_unstage_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if context.staged_empty {
        return Err("no staged changes to unstage".to_owned());
    }
    unstage_git_all(runtime)
}

pub(crate) fn git_status_commit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if context.staged_empty {
        return Err("no staged changes to commit".to_owned());
    }
    open_git_commit_buffer(runtime)
}

pub(crate) fn git_status_push_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    push_git_to_pushremote(runtime, context.buffer_id)
}

pub(crate) fn git_status_push_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    push_git_to_upstream(runtime, context.buffer_id)
}

pub(crate) fn git_status_fetch_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    fetch_git_pushremote(runtime, context.buffer_id)
}

pub(crate) fn git_status_fetch_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    fetch_git_upstream(runtime, context.buffer_id)
}

pub(crate) fn git_status_fetch_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    fetch_git_all(runtime)
}

pub(crate) fn git_status_pull_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    pull_git_upstream(runtime, context.buffer_id)
}

pub(crate) fn git_status_branches_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker(runtime)
}

pub(crate) fn git_worktree_create_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_worktree_branch_picker(runtime)
}

pub(crate) fn trace_oil_worktree(runtime: &mut EditorRuntime, message: impl Into<String>) {
    let message = message.into();
    eprintln!("[oil.git-worktree] {message}");
    record_runtime_error(runtime, "oil.git-worktree.trace", message);
}

pub(crate) fn oil_git_worktree_command(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(crate) fn git_status_merge_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_merge_in_progress(runtime)? {
        return merge_git_continue(runtime);
    }
    open_git_branch_picker_with_action(runtime, "Git Merge", GitBranchActionKind::MergePlain)
}

pub(crate) fn git_status_merge_edit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Edit Message)",
        GitBranchActionKind::MergeEdit,
    )
}

pub(crate) fn git_status_merge_no_commit_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (No Commit)",
        GitBranchActionKind::MergeNoCommit,
    )
}

pub(crate) fn git_status_merge_squash_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Squash)",
        GitBranchActionKind::MergeSquash,
    )
}

pub(crate) fn git_status_merge_preview_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Preview)",
        GitBranchActionKind::MergePreview,
    )
}

pub(crate) fn git_status_merge_abort_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    merge_git_abort(runtime)
}

pub(crate) fn git_status_rebase_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    let context = active_git_status_command_context(runtime)?;
    rebase_git_onto_pushremote(runtime, context.buffer_id)
}

pub(crate) fn git_status_rebase_upstream_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    let context = active_git_status_command_context(runtime)?;
    rebase_git_onto_upstream(runtime, context.buffer_id)
}

pub(crate) fn git_status_rebase_onto_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_rebase_in_progress(runtime)? {
        return rebase_git_edit_todo(runtime);
    }
    open_git_branch_picker_with_action(runtime, "Git Rebase", GitBranchActionKind::RebaseOnto)
}

pub(crate) fn git_status_rebase_interactive_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    open_git_branch_picker_with_action(
        runtime,
        "Git Rebase (Interactive)",
        GitBranchActionKind::RebaseInteractive,
    )
}

pub(crate) fn git_status_rebase_continue_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "no rebase in progress")?;
    rebase_git_continue(runtime)
}

pub(crate) fn git_status_rebase_skip_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "rebase subset is not supported yet")?;
    rebase_git_skip(runtime)
}

pub(crate) fn git_status_rebase_abort_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "no rebase in progress")?;
    rebase_git_abort(runtime)
}

pub(crate) fn git_status_rebase_autosquash_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase autosquash is not supported yet")
}

pub(crate) fn git_status_rebase_edit_commit_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase edit-commit is not supported yet")
}

pub(crate) fn git_status_rebase_reword_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase reword is not supported yet")
}

pub(crate) fn git_status_rebase_remove_commit_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase remove-commit is not supported yet")
}

pub(crate) fn git_status_diff_dwim_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_dwim(runtime, context.buffer_id, context.meta.as_ref(), "")
}

pub(crate) fn git_status_diff_staged_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_diff_staged(runtime)
}

pub(crate) fn git_status_diff_unstaged_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_diff_unstaged(runtime)
}

pub(crate) fn git_status_diff_commit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_commit_at_point(runtime, context.buffer_id, context.meta.as_ref())
}

pub(crate) fn git_status_diff_stash_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_stash_at_point(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_diff_range_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git diff range is not supported yet")
}

pub(crate) fn git_status_diff_paths_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git diff paths is not supported yet")
}

pub(crate) fn git_status_log_head_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_head(runtime)
}

pub(crate) fn git_status_log_related_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    open_git_log_related(runtime, context.buffer_id)
}

pub(crate) fn git_status_log_other_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git log other is not supported yet")
}

pub(crate) fn git_status_log_branches_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_branches(runtime)
}

pub(crate) fn git_status_log_all_branches_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    open_git_log_all_branches(runtime)
}

pub(crate) fn git_status_log_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_all(runtime)
}

pub(crate) fn git_status_stash_both_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_both(runtime)
}

pub(crate) fn git_status_stash_index_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_index(runtime)
}

pub(crate) fn git_status_stash_worktree_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_worktree(runtime)
}

pub(crate) fn git_status_stash_keep_index_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    stash_git_keep_index(runtime)
}

pub(crate) fn git_status_stash_apply_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    stash_git_apply_at_point(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_stash_pop_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    stash_git_pop_at_point(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_stash_drop_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    stash_git_drop_at_point(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_stash_show_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    stash_git_show_at_point(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_cherry_open_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    open_git_cherry_buffer(runtime, context.buffer_id)
}

pub(crate) fn git_status_cherry_pick_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if let Some(kind) = git_sequence_in_progress(runtime)? {
        return sequence_git_continue(runtime, kind);
    }
    cherry_pick_commit_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_cherry_pick_apply_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if let Some(kind) = git_sequence_in_progress(runtime)? {
        return sequence_git_abort(runtime, kind);
    }
    cherry_pick_apply_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_cherry_pick_skip_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let kind =
        git_status_sequence_kind(runtime, "cherry-pick move commands are not supported yet")?;
    sequence_git_skip(runtime, kind)
}

pub(crate) fn git_status_revert_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if let Some(kind) = git_sequence_in_progress(runtime)? {
        return sequence_git_continue(runtime, kind);
    }
    revert_commit_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_revert_no_commit_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if let Some(kind) = git_sequence_in_progress(runtime)? {
        return sequence_git_abort(runtime, kind);
    }
    revert_no_commit_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_revert_skip_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let kind = git_status_sequence_kind(runtime, "no cherry-pick or revert in progress")?;
    sequence_git_skip(runtime, kind)
}

pub(crate) fn git_status_revert_abort_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let kind = git_status_sequence_kind(runtime, "no cherry-pick or revert in progress")?;
    sequence_git_abort(runtime, kind)
}

pub(crate) fn git_status_apply_commit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    cherry_pick_apply_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(crate) fn git_status_reset_mixed_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Mixed)
}

pub(crate) fn git_status_reset_soft_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Soft)
}

pub(crate) fn git_status_reset_hard_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Hard)
}

pub(crate) fn git_status_reset_keep_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Keep)
}

pub(crate) fn git_status_reset_index_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("reset index is not supported yet")
}

pub(crate) fn git_status_reset_worktree_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("reset worktree is not supported yet")
}

pub(crate) fn git_status_checkout_file_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("file checkout is not supported yet")
}

pub(crate) fn git_status_discard_or_reset_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    let (targets, is_visual) = git_status_delete_targets(runtime, context.buffer_id)?;
    if !targets.is_empty() {
        delete_git_status_targets(runtime, &targets)?;
        if is_visual {
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        return Ok(());
    }
    if is_visual {
        return Err("no deletable files selected".to_owned());
    }
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Mixed)
}

pub(crate) fn take_directory_prefix(runtime: &mut EditorRuntime) -> Result<Option<String>, String> {
    const PREFIX_TIMEOUT: Duration = Duration::from_millis(1200);
    let now = Instant::now();
    let ui = shell_ui_mut(runtime)?;
    let pending = match ui.pending_directory_prefix.take() {
        Some(state) if now.duration_since(state.started_at) <= PREFIX_TIMEOUT => Some(state.chord),
        _ => None,
    };
    Ok(pending)
}

pub(crate) fn set_directory_prefix(runtime: &mut EditorRuntime, chord: &str) -> Result<(), String> {
    shell_ui_mut(runtime)?.pending_directory_prefix = Some(DirectoryPrefixState {
        chord: chord.to_owned(),
        started_at: Instant::now(),
    });
    Ok(())
}

pub(crate) enum TakeKeySequence {
    /// Live pending tokens for this scope.
    Live(PendingKeySequence),
    /// Ambiguous short already timed out — fire it before handling the new token.
    FireShort {
        chord: String,
        vim_mode: KeymapVimMode,
    },
    /// No pending sequence for this scope.
    None,
}

pub(crate) fn take_key_sequence(
    runtime: &mut EditorRuntime,
    scope: &KeymapScope,
    options: &KeySequenceOptions,
) -> Result<TakeKeySequence, String> {
    let now = Instant::now();
    let ui = shell_ui_mut(runtime)?;
    let state = ui.pending_key_sequence.take();
    match state {
        Some(state) if &state.scope == scope => {
            let elapsed_ms =
                u64::try_from(now.duration_since(state.started_at).as_millis()).unwrap_or(u64::MAX);
            let pending = PendingKeySequence {
                tokens: state.tokens,
                started_at_ms: 0,
                ambiguous_short: state.ambiguous_short,
            };
            match tick_key_sequence(&pending, elapsed_ms, options) {
                KeySequenceTick::Pending => Ok(TakeKeySequence::Live(pending)),
                KeySequenceTick::Execute { chord } => Ok(TakeKeySequence::FireShort {
                    chord,
                    vim_mode: state.vim_mode,
                }),
                KeySequenceTick::Expired => Ok(TakeKeySequence::None),
            }
        }
        Some(state) => {
            ui.pending_key_sequence = Some(state);
            Ok(TakeKeySequence::None)
        }
        None => Ok(TakeKeySequence::None),
    }
}

pub(crate) fn set_key_sequence(
    runtime: &mut EditorRuntime,
    scope: KeymapScope,
    vim_mode: KeymapVimMode,
    pending: PendingKeySequence,
) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    ui.pending_key_sequence = Some(KeySequenceState {
        scope,
        vim_mode,
        tokens: pending.tokens,
        started_at: Instant::now(),
        ambiguous_short: pending.ambiguous_short,
    });
    Ok(())
}

pub(crate) fn clear_key_sequence(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.pending_key_sequence = None;
    Ok(())
}

pub(crate) fn peek_key_sequence_tick(
    runtime: &EditorRuntime,
    options: &KeySequenceOptions,
) -> Result<Option<(KeymapScope, KeymapVimMode, KeySequenceTick)>, String> {
    let ui = shell_ui(runtime)?;
    let Some(state) = ui.pending_key_sequence.as_ref() else {
        return Ok(None);
    };
    let now = Instant::now();
    let elapsed_ms =
        u64::try_from(now.duration_since(state.started_at).as_millis()).unwrap_or(u64::MAX);
    let pending = PendingKeySequence {
        tokens: state.tokens.clone(),
        started_at_ms: 0,
        ambiguous_short: state.ambiguous_short.clone(),
    };
    Ok(Some((
        state.scope.clone(),
        state.vim_mode,
        tick_key_sequence(&pending, elapsed_ms, options),
    )))
}

pub(crate) fn move_git_section(runtime: &mut EditorRuntime, forward: bool) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (start_line, line_count) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_git_status(&buffer.kind) {
            return Ok(false);
        }
        (buffer.cursor_point().line, buffer.line_count())
    };
    if line_count == 0 {
        return Ok(false);
    }
    if forward {
        for line in start_line.saturating_add(1)..line_count {
            if let Some(meta) = shell_buffer(runtime, buffer_id)?.section_line_meta(line)
                && matches!(meta.kind, SectionRenderLineKind::Header { .. })
            {
                shell_buffer_mut(runtime, buffer_id)?.goto_line(line);
                return Ok(true);
            }
        }
    } else {
        let mut line = start_line;
        while line > 0 {
            line = line.saturating_sub(1);
            if let Some(meta) = shell_buffer(runtime, buffer_id)?.section_line_meta(line)
                && matches!(meta.kind, SectionRenderLineKind::Header { .. })
            {
                shell_buffer_mut(runtime, buffer_id)?.goto_line(line);
                return Ok(true);
            }
            if line == 0 {
                break;
            }
        }
    }
    Ok(false)
}

pub(crate) fn toggle_git_section(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (section_id, snapshot) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_git_status(&buffer.kind) {
            return Ok(false);
        }
        let meta = buffer
            .section_line_meta(buffer.cursor_point().line)
            .cloned();
        let section_id = match meta.as_ref().map(|meta| &meta.kind) {
            Some(SectionRenderLineKind::Header { id, .. }) => id.clone(),
            _ => return Ok(false),
        };
        let snapshot = buffer
            .git_snapshot()
            .cloned()
            .ok_or_else(|| "git status snapshot is missing".to_owned())?;
        (section_id, snapshot)
    };
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let state = buffer.ensure_section_state();
        state.collapsed.toggle(&section_id);
    }
    apply_git_status_snapshot(runtime, buffer_id, snapshot)?;
    Ok(true)
}

pub(crate) fn handle_git_status_tab(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let meta = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_git_status(&buffer.kind) {
            return Ok(false);
        }
        buffer
            .section_line_meta(buffer.cursor_point().line)
            .cloned()
    };
    if matches!(
        meta.as_ref().map(|meta| &meta.kind),
        Some(SectionRenderLineKind::Header { .. })
    ) {
        return toggle_git_section(runtime);
    }
    diff_git_dwim(runtime, buffer_id, meta.as_ref(), "")?;
    Ok(true)
}

pub(crate) fn handle_git_status_chord(
    runtime: &mut EditorRuntime,
    chord: &str,
) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_git_status(&buffer.kind) {
            return Ok(false);
        }
    }

    let prefix = take_git_prefix(runtime)?;
    let user_library = shell_user_library(runtime);
    if let Some(command_name) = git_status_command_name(&*user_library, prefix, chord)
        .or_else(|| git_status_command_name(&*user_library, None, chord))
    {
        runtime
            .execute_command(command_name)
            .map_err(|error| error.to_string())?;
        return Ok(true);
    }

    if let Some(prefix) = user_library.git_prefix_for_chord(chord) {
        set_git_prefix(runtime, prefix)?;
        return Ok(true);
    }
    Ok(false)
}

pub(crate) fn handle_git_view_chord(
    runtime: &mut EditorRuntime,
    chord: &str,
) -> Result<bool, String> {
    if chord != "g" {
        return Ok(false);
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let view = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let is_git_view = matches!(
            &buffer.kind,
            BufferKind::Plugin(plugin_kind)
                if plugin_kind == GIT_DIFF_KIND
                    || plugin_kind == GIT_LOG_KIND
                    || plugin_kind == GIT_STASH_KIND
        );
        if !is_git_view {
            return Ok(false);
        }
        buffer
            .git_view()
            .cloned()
            .ok_or_else(|| "git view state is missing".to_owned())?
    };
    apply_git_view(runtime, buffer_id, view)?;
    Ok(true)
}
