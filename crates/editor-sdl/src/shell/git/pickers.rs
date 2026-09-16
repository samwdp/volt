use super::super::*;

use super::commands::*;
use super::process::*;
use super::remote::*;

pub(crate) fn git_branch_list(
    _runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<Vec<String>, String> {
    git_branch_list_at(root)
}

pub(crate) fn git_branch_list_at(root: &Path) -> Result<Vec<String>, String> {
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

/// Fetch + branch list for worktree pickers. Safe off the UI thread (direct git).
///
/// Returns `(branches, optional_fetch_error)`. Fetch failure is soft when any
/// branch refs remain readable.
type RemoteWorktreeBranchList = (Vec<(String, String)>, Option<String>);

fn collect_remote_worktree_branches(root: &Path) -> Result<Vec<(String, String)>, String> {
    let local = git_branch_list_at(root)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let output = git_read_command_output(
        root,
        "branch -r --format",
        &["branch", "-r", "--format=%(refname:short)"],
    )?;
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
        branches = local
            .iter()
            .filter(|branch| branch.as_str() != "HEAD")
            .map(|branch| (branch.clone(), branch.clone()))
            .collect::<Vec<_>>();
    }
    branches.sort();
    branches.dedup();
    Ok(branches)
}

fn finish_remote_worktree_branch_list(
    branches: Vec<(String, String)>,
    fetch_error: Option<String>,
) -> Result<RemoteWorktreeBranchList, String> {
    if branches.is_empty()
        && let Some(error) = fetch_error
    {
        return Err(error);
    }
    Ok((branches, fetch_error))
}

pub(crate) fn git_remote_worktree_branch_list_at(
    root: &Path,
) -> Result<RemoteWorktreeBranchList, String> {
    let fetch_error = fetch_git_prune_at(root).err();
    let branches = match collect_remote_worktree_branches(root) {
        Ok(branches) => branches,
        Err(error) => return Err(fetch_error.unwrap_or(error)),
    };
    finish_remote_worktree_branch_list(branches, fetch_error)
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
    let branches = match collect_remote_worktree_branches(root) {
        Ok(branches) => branches,
        Err(error) => return Err(fetch_error.unwrap_or(error)),
    };
    let (branches, fetch_error) = finish_remote_worktree_branch_list(branches, fetch_error)?;
    if let Some(error) = fetch_error {
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
