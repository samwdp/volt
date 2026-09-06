use super::super::*;

use super::diff::*;
use super::process::*;
use super::staging::*;
use super::status::*;

pub(crate) fn stash_git_both(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "stash push", &["stash", "push"])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn stash_git_index(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "stash push --staged",
        &["stash", "push", "--staged"],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn stash_git_worktree(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "stash push --keep-index",
        &["stash", "push", "--keep-index"],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn stash_git_keep_index(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "stash push --keep-index",
        &["stash", "push", "--keep-index"],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn stash_git_apply_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "stash apply", &["stash", "apply", &stash])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn stash_git_pop_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "stash pop", &["stash", "pop", &stash])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn stash_git_drop_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "stash drop", &["stash", "drop", &stash])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn stash_git_show_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    open_git_diff_stash(runtime, &stash)
}
