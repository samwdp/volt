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

pub(crate) fn open_git_log_buffer(
    runtime: &mut EditorRuntime,
    view: GitViewState,
) -> Result<(), String> {
    open_git_view_buffer(runtime, GIT_LOG_KIND, "*git-log*", view)
}

pub(crate) fn git_log_args(extra: &[String]) -> Vec<String> {
    let mut args = vec![
        "--no-pager".to_owned(),
        "log".to_owned(),
        "--no-color".to_owned(),
        "--oneline".to_owned(),
        "--decorate".to_owned(),
        "--graph".to_owned(),
        "-n".to_owned(),
        GIT_LOG_VIEW_LIMIT.to_string(),
    ];
    args.extend(extra.iter().cloned());
    args
}

pub(crate) fn open_git_log_current(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&[]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(crate) fn open_git_log_head(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&["HEAD".to_owned()]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(crate) fn open_git_log_related(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let mut refs = Vec::new();
    if let Some(branch) = snapshot.branch() {
        refs.push(branch.to_owned());
    } else {
        refs.push("HEAD".to_owned());
    }
    if let Some(upstream) = snapshot.upstream() {
        refs.push(upstream.to_owned());
    }
    if let Some(push_remote) = snapshot.push_remote() {
        refs.push(push_remote.to_owned());
    }
    refs.sort();
    refs.dedup();
    let args = git_log_args(&refs);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(crate) fn open_git_log_branches(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&["--branches".to_owned()]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(crate) fn open_git_log_all_branches(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&["--branches".to_owned(), "--remotes".to_owned()]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(crate) fn open_git_log_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&["--all".to_owned()]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}
