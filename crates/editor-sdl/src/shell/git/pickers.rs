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
use super::merge_rebase::*;
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

pub(crate) fn git_branch_list(
    _runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<Vec<String>, String> {
    let output = git_read_command_output(
        root,
        "branch --format",
        &["branch", "--format=%(refname:short)"],
    )?;
    let mut branches = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();
    branches.sort();
    branches.dedup();
    Ok(branches)
}

pub(crate) fn git_remote_worktree_branch_list(
    runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<Vec<(String, String)>, String> {
    trace_oil_worktree(
        runtime,
        format!("listing remote branches from `{}`", root.display()),
    );
    let fetch_error = fetch_git_prune(runtime, root).err();
    match &fetch_error {
        Some(error) => trace_oil_worktree(runtime, format!("git fetch --prune failed: {error}")),
        None => trace_oil_worktree(runtime, "git fetch --prune succeeded"),
    }
    let local = git_branch_list(runtime, root)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    trace_oil_worktree(runtime, format!("found {} local branches", local.len()));
    let output = match git_read_command_output(
        root,
        "branch -r --format",
        &["branch", "-r", "--format=%(refname:short)"],
    ) {
        Ok(output) => output,
        Err(error) => return Err(fetch_error.unwrap_or(error)),
    };
    let mut branches = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.ends_with("/HEAD"))
        .filter_map(|remote_branch| {
            let (_, local_branch) = remote_and_branch_from_ref(remote_branch)?;
            let local_branch = if local.contains(&local_branch) {
                remote_branch.replace('/', "-")
            } else {
                local_branch
            };
            Some((remote_branch.to_owned(), local_branch))
        })
        .collect::<Vec<_>>();
    if branches.is_empty() {
        trace_oil_worktree(
            runtime,
            "no remote branches found; falling back to local branch refs",
        );
        branches = local
            .iter()
            .filter(|branch| branch.as_str() != "HEAD")
            .map(|branch| (branch.clone(), branch.clone()))
            .collect::<Vec<_>>();
    }
    branches.sort();
    branches.dedup();
    if branches.is_empty() {
        if let Some(error) = fetch_error {
            return Err(error);
        }
    } else if let Some(error) = fetch_error {
        record_runtime_error(runtime, "git.worktree.fetch", error);
    }
    trace_oil_worktree(
        runtime,
        format!(
            "found {} remote branches for worktree picker",
            branches.len()
        ),
    );
    Ok(branches)
}

pub(crate) fn git_commit_list(
    _runtime: &mut EditorRuntime,
    root: &Path,
    limit: usize,
) -> Result<Vec<GitLogEntry>, String> {
    let output = git_read_command_output(
        root,
        "log --oneline",
        &["log", "-n", &limit.to_string(), "--oneline"],
    )?;
    Ok(parse_log_oneline(&output))
}

pub(crate) fn open_git_commit_picker_with_action(
    runtime: &mut EditorRuntime,
    title: &str,
    action: GitCommitActionKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let commits = git_commit_list(runtime, &root, GIT_LOG_VIEW_LIMIT)?;
    if commits.is_empty() {
        return Err("no commits found".to_owned());
    }
    let entries = commits
        .into_iter()
        .map(|commit| {
            let label = format!("{} {}", commit.hash(), commit.summary());
            let item_id = format!("git-commit:{}", commit.hash());
            let action = PickerAction::GitCommitAction {
                action,
                commit: commit.hash().to_owned(),
            };
            PickerEntry {
                item: PickerItem::new(item_id, label, "commit", None::<String>),
                action,
                quickfix: None,
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries(title, entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn open_git_branch_picker_with_action(
    runtime: &mut EditorRuntime,
    title: &str,
    action: GitBranchActionKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let branches = git_branch_list(runtime, &root)?;
    if branches.is_empty() {
        return Err("no git branches found".to_owned());
    }
    let entries = branches
        .into_iter()
        .map(|branch| {
            let item_id = format!("git-branch:{branch}");
            let action = PickerAction::GitBranchAction {
                action,
                branch: branch.clone(),
            };
            PickerEntry {
                item: PickerItem::new(item_id, branch.clone(), "branch", None::<String>),
                action,
                quickfix: None,
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries(title, entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn open_git_branch_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(runtime, "Git Branches", GitBranchActionKind::Checkout)
}
