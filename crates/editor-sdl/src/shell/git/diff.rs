#![allow(unused_imports)]
use super::super::*;

#[allow(unused_imports)]
use super::commands::*;
#[allow(unused_imports)]
use super::commit::*;
#[allow(unused_imports)]
use super::fringe::*;
#[allow(unused_imports)]
use super::log::*;
#[allow(unused_imports)]
use super::merge_rebase::*;
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

pub(crate) fn git_view_lines(
    runtime: &mut EditorRuntime,
    view: &GitViewState,
) -> Result<Vec<String>, String> {
    let root = git_root(runtime)?;
    let args = view.args.iter().map(String::as_str).collect::<Vec<_>>();
    let output = git_read_command_output_allow_exit_codes(
        &root,
        &view.label,
        &args,
        &view.allowed_exit_codes,
    )?;
    let mut lines = output
        .lines()
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push(view.empty_message.clone());
    }
    Ok(lines)
}

pub(crate) fn git_view_lines_or_error(
    runtime: &mut EditorRuntime,
    view: &GitViewState,
) -> Vec<String> {
    match git_view_lines(runtime, view) {
        Ok(lines) => lines,
        Err(error) => {
            record_runtime_error(runtime, &format!("git.{}", view.label), error.clone());
            vec![format!("Git {} unavailable.", view.label), error]
        }
    }
}

pub(crate) fn git_view_language_id(kind: &str) -> Option<&'static str> {
    match kind {
        GIT_DIFF_KIND => Some("diff"),
        _ => None,
    }
}

pub(crate) fn open_git_view_buffer(
    runtime: &mut EditorRuntime,
    kind: &str,
    name: &str,
    view: GitViewState,
) -> Result<(), String> {
    let lines = git_view_lines_or_error(runtime, &view);
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_kind = BufferKind::Plugin(kind.to_owned());
    let language_id = git_view_language_id(kind).map(str::to_owned);
    let existing = find_workspace_named_buffer(runtime, workspace_id, name, &buffer_kind)?;
    if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.focus_buffer_in_active_pane(existing);
        {
            let buffer = shell_buffer_mut(runtime, existing)?;
            buffer.set_git_view(view);
            buffer.replace_with_lines(lines);
            buffer.set_language_id(language_id.clone());
        }
        if language_id.is_some() {
            queue_buffer_syntax_refresh(runtime, existing)?;
        }
        return Ok(());
    }

    let buffer_id = runtime
        .model_mut()
        .create_buffer(workspace_id, name, buffer_kind, None)
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, lines, &*user_library);
    shell_buffer.set_git_view(view);
    shell_buffer.set_language_id(language_id.clone());
    let ui = shell_ui_mut(runtime)?;
    ui.insert_buffer(shell_buffer);
    ui.focus_buffer_in_active_pane(buffer_id);
    if language_id.is_some() {
        queue_buffer_syntax_refresh(runtime, buffer_id)?;
    }
    Ok(())
}

pub(crate) fn apply_git_view(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    view: GitViewState,
) -> Result<(), String> {
    let lines = git_view_lines_or_error(runtime, &view);
    let refresh_syntax = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let language_id = match &buffer.kind {
            BufferKind::Plugin(kind) if kind == GIT_DIFF_KIND => Some("diff".to_owned()),
            _ => None,
        };
        buffer.set_git_view(view);
        buffer.replace_with_lines(lines);
        buffer.set_language_id(language_id.clone());
        language_id.is_some()
    };
    if refresh_syntax {
        queue_buffer_syntax_refresh(runtime, buffer_id)?;
    }
    Ok(())
}

pub(crate) fn open_git_diff_buffer(
    runtime: &mut EditorRuntime,
    view: GitViewState,
) -> Result<(), String> {
    open_git_view_buffer(runtime, GIT_DIFF_KIND, "*git-diff*", view)
}

pub(crate) fn open_git_diff_worktree(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_args_with_no_pager("diff", &["--no-color", "HEAD"]);
    let view = GitViewState::new("diff", args, "No working tree changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn open_git_diff_staged(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_args_with_no_pager("diff", &["--no-color", "--cached"]);
    let view = GitViewState::new("diff", args, "No staged changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn open_git_diff_unstaged(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_args_with_no_pager("diff", &["--no-color"]);
    let view = GitViewState::new("diff", args, "No unstaged changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn open_git_diff_staged_file(
    runtime: &mut EditorRuntime,
    path: &str,
) -> Result<(), String> {
    let mut args = git_args_with_no_pager("diff", &["--no-color", "--cached", "--"]);
    args.push(path.to_owned());
    let view = GitViewState::new("diff", args, "No staged changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn open_git_diff_unstaged_file(
    runtime: &mut EditorRuntime,
    path: &str,
) -> Result<(), String> {
    let mut args = git_args_with_no_pager("diff", &["--no-color", "--"]);
    args.push(path.to_owned());
    let view = GitViewState::new("diff", args, "No unstaged changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn open_git_diff_untracked_file(
    runtime: &mut EditorRuntime,
    path: &str,
) -> Result<(), String> {
    let mut args = git_args_with_no_pager("diff", &["--no-color", "--no-index", "--", "/dev/null"]);
    args.push(path.to_owned());
    let view = GitViewState::new("diff", args, "No untracked diff.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn open_git_diff_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let args = git_args_with_no_pager("show", &["--no-color", commit]);
    let view = GitViewState::new("show", args, "No commit diff.", &[0]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn open_git_diff_stash(runtime: &mut EditorRuntime, stash: &str) -> Result<(), String> {
    let args = git_args_with_no_pager("stash", &["show", "--no-color", "-p", stash]);
    let view = GitViewState::new("stash", args, "No stash diff.", &[0]);
    open_git_diff_buffer(runtime, view)
}

pub(crate) fn diff_git_dwim(
    runtime: &mut EditorRuntime,
    _buffer_id: BufferId,
    meta: Option<&SectionLineMeta>,
    _line_text: &str,
) -> Result<(), String> {
    if let Some(commit) = git_action_detail(meta, GIT_ACTION_SHOW_COMMIT) {
        return open_git_diff_commit(runtime, &commit);
    }
    if let Some(stash) = git_action_detail(meta, GIT_ACTION_SHOW_STASH) {
        return open_git_diff_stash(runtime, &stash);
    }
    if let Some(path) = git_action_detail(meta, GIT_ACTION_UNSTAGE_FILE) {
        return open_git_diff_staged_file(runtime, &path);
    }
    if let Some(path) = git_action_detail(meta, GIT_ACTION_STAGE_FILE) {
        if git_line_is_untracked(meta) {
            return open_git_diff_untracked_file(runtime, &path);
        }
        return open_git_diff_unstaged_file(runtime, &path);
    }
    if let Some(meta) = meta
        && let SectionRenderLineKind::Header { id, .. } = &meta.kind
    {
        if id == GIT_SECTION_STAGED {
            return open_git_diff_staged(runtime);
        }
        if id == GIT_SECTION_UNSTAGED || id == GIT_SECTION_UNTRACKED {
            return open_git_diff_unstaged(runtime);
        }
    }
    open_git_diff_worktree(runtime)
}

pub(crate) fn diff_git_commit_at_point(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_action_detail(meta, GIT_ACTION_SHOW_COMMIT) {
        return open_git_diff_commit(runtime, &commit);
    }
    let commit = git_snapshot_for_buffer(runtime, buffer_id)
        .ok()
        .and_then(|snapshot| snapshot.head().map(|head| head.hash().to_owned()))
        .unwrap_or_else(|| "HEAD".to_owned());
    open_git_diff_commit(runtime, &commit)
}

pub(crate) fn diff_git_stash_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    open_git_diff_stash(runtime, &stash)
}
