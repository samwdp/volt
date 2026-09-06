#![allow(unused_imports)]
use super::super::*;

#[allow(unused_imports)]
use super::commands::*;
#[allow(unused_imports)]
use super::commit::*;
#[allow(unused_imports)]
use super::diff::*;
#[allow(unused_imports)]
use super::fringe::*;
#[allow(unused_imports)]
use super::log::*;
#[allow(unused_imports)]
use super::pickers::*;
#[allow(unused_imports)]
use super::process::*;
#[allow(unused_imports)]
use super::remote::*;
#[allow(unused_imports)]
use super::staging::*;
#[allow(unused_imports)]
use super::stash::*;
#[allow(unused_imports)]
use super::status::*;
#[allow(unused_imports)]
use super::worktree::*;

pub(crate) fn git_merge_in_progress(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let root = git_root(runtime)?;
    let Some(git_dir) = git_dir_path(runtime, &root) else {
        return Ok(false);
    };
    Ok(git_dir.join("MERGE_HEAD").is_file())
}

pub(crate) fn git_rebase_in_progress(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let root = git_root(runtime)?;
    let Some(git_dir) = git_dir_path(runtime, &root) else {
        return Ok(false);
    };
    Ok(git_dir.join("rebase-apply").is_dir() || git_dir.join("rebase-merge").is_dir())
}

pub(crate) fn git_sequence_in_progress(
    runtime: &mut EditorRuntime,
) -> Result<Option<GitSequenceKind>, String> {
    let root = git_root(runtime)?;
    let Some(git_dir) = git_dir_path(runtime, &root) else {
        return Ok(None);
    };
    if git_dir.join("CHERRY_PICK_HEAD").is_file() {
        return Ok(Some(GitSequenceKind::CherryPick));
    }
    if git_dir.join("REVERT_HEAD").is_file() {
        return Ok(Some(GitSequenceKind::Revert));
    }
    Ok(None)
}

pub(crate) fn sequence_git_continue(
    runtime: &mut EditorRuntime,
    kind: GitSequenceKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let (title, args) = match kind {
        GitSequenceKind::CherryPick => (
            "Git Cherry-pick",
            vec!["cherry-pick".to_owned(), "--continue".to_owned()],
        ),
        GitSequenceKind::Revert => (
            "Git Revert",
            vec!["revert".to_owned(), "--continue".to_owned()],
        ),
    };
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            title,
            args,
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn sequence_git_skip(
    runtime: &mut EditorRuntime,
    kind: GitSequenceKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let (title, args) = match kind {
        GitSequenceKind::CherryPick => (
            "Git Cherry-pick",
            vec!["cherry-pick".to_owned(), "--skip".to_owned()],
        ),
        GitSequenceKind::Revert => ("Git Revert", vec!["revert".to_owned(), "--skip".to_owned()]),
    };
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            title,
            args,
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn sequence_git_abort(
    runtime: &mut EditorRuntime,
    kind: GitSequenceKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let (title, args) = match kind {
        GitSequenceKind::CherryPick => (
            "Git Cherry-pick",
            vec!["cherry-pick".to_owned(), "--abort".to_owned()],
        ),
        GitSequenceKind::Revert => (
            "Git Revert",
            vec!["revert".to_owned(), "--abort".to_owned()],
        ),
    };
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            title,
            args,
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn git_commit_at_point(meta: Option<&SectionLineMeta>) -> Option<String> {
    git_action_detail(meta, GIT_ACTION_SHOW_COMMIT)
}

pub(crate) fn cherry_pick_git_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Cherry-pick",
            vec!["cherry-pick".to_owned(), commit.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn cherry_pick_git_commit_no_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Cherry-pick",
            vec![
                "cherry-pick".to_owned(),
                "--no-commit".to_owned(),
                commit.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn revert_git_commit(runtime: &mut EditorRuntime, commit: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Revert",
            vec!["revert".to_owned(), commit.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn revert_git_commit_no_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Revert",
            vec![
                "revert".to_owned(),
                "--no-commit".to_owned(),
                commit.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn cherry_pick_commit_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return cherry_pick_git_commit(runtime, &commit);
    }
    open_git_commit_picker_with_action(runtime, "Git Cherry-Pick", GitCommitActionKind::CherryPick)
}

pub(crate) fn cherry_pick_apply_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return cherry_pick_git_commit_no_commit(runtime, &commit);
    }
    open_git_commit_picker_with_action(
        runtime,
        "Git Cherry-Pick (Apply)",
        GitCommitActionKind::CherryPickNoCommit,
    )
}

pub(crate) fn revert_commit_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return revert_git_commit(runtime, &commit);
    }
    open_git_commit_picker_with_action(runtime, "Git Revert", GitCommitActionKind::Revert)
}

pub(crate) fn revert_no_commit_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return revert_git_commit_no_commit(runtime, &commit);
    }
    open_git_commit_picker_with_action(
        runtime,
        "Git Revert (No Commit)",
        GitCommitActionKind::RevertNoCommit,
    )
}

pub(crate) fn reset_git_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
    mode: GitResetMode,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let (label, args) = match mode {
        GitResetMode::Mixed => ("reset --mixed", vec!["reset", "--mixed", commit]),
        GitResetMode::Soft => ("reset --soft", vec!["reset", "--soft", commit]),
        GitResetMode::Hard => ("reset --hard", vec!["reset", "--hard", commit]),
        GitResetMode::Keep => ("reset --keep", vec!["reset", "--keep", commit]),
    };
    git_command_output(runtime, &root, label, &args)?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn reset_commit_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
    mode: GitResetMode,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return reset_git_commit(runtime, &commit, mode);
    }
    let (title, action) = match mode {
        GitResetMode::Mixed => ("Git Reset (Mixed)", GitCommitActionKind::ResetMixed),
        GitResetMode::Soft => ("Git Reset (Soft)", GitCommitActionKind::ResetSoft),
        GitResetMode::Hard => ("Git Reset (Hard)", GitCommitActionKind::ResetHard),
        GitResetMode::Keep => ("Git Reset (Keep)", GitCommitActionKind::ResetKeep),
    };
    open_git_commit_picker_with_action(runtime, title, action)
}

pub(crate) fn merge_git_plain(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec!["merge".to_owned(), branch.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn merge_git_edit(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec![
                "merge".to_owned(),
                "--no-commit".to_owned(),
                "--edit".to_owned(),
                branch.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusCloseAndOpenCommitBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn merge_git_no_commit(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec![
                "merge".to_owned(),
                "--no-commit".to_owned(),
                branch.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn merge_git_squash(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec!["merge".to_owned(), "--squash".to_owned(), branch.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusCloseAndOpenCommitBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn merge_git_preview(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let args = vec![
        "--no-pager".to_owned(),
        "diff".to_owned(),
        "--no-color".to_owned(),
        format!("HEAD...{branch}"),
    ];
    let view = GitViewState::new("diff", args, "No changes to merge.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn merge_git_continue(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec!["merge".to_owned(), "--continue".to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn merge_git_abort(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec!["merge".to_owned(), "--abort".to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn rebase_git_onto(runtime: &mut EditorRuntime, target: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Rebase",
            vec!["rebase".to_owned(), target.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn rebase_git_interactive_onto(
    runtime: &mut EditorRuntime,
    target: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Rebase",
            vec!["rebase".to_owned(), "-i".to_owned(), target.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn rebase_git_onto_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let upstream = snapshot
        .upstream()
        .ok_or_else(|| "no upstream configured for rebase".to_owned())?;
    rebase_git_onto(runtime, upstream)
}

pub(crate) fn rebase_git_onto_pushremote(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let push_remote = snapshot
        .push_remote()
        .ok_or_else(|| "no push-remote configured for rebase".to_owned())?;
    rebase_git_onto(runtime, push_remote)
}

pub(crate) fn rebase_git_continue(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Rebase",
            vec!["rebase".to_owned(), "--continue".to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn rebase_git_skip(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Rebase",
            vec!["rebase".to_owned(), "--skip".to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn rebase_git_edit_todo(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Rebase",
            vec!["rebase".to_owned(), "--edit-todo".to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(crate) fn rebase_git_abort(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Rebase",
            vec!["rebase".to_owned(), "--abort".to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn open_git_cherry_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let upstream = snapshot
        .upstream()
        .ok_or_else(|| "no upstream configured for cherry".to_owned())?;
    let args = git_args_with_no_pager("cherry", &["-v", upstream]);
    let view = GitViewState::new("cherry", args, "No cherry commits.", &[0]);
    open_git_log_buffer(runtime, view)
}
