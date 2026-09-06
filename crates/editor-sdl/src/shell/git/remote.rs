use super::super::*;

use super::process::*;
use super::status::*;

pub(crate) fn push_git_remote(runtime: &mut EditorRuntime, remote: &str) -> Result<(), String> {
    let branch = {
        let buffer_id = active_shell_buffer_id(runtime)?;
        shell_buffer(runtime, buffer_id)?
            .git_snapshot()
            .and_then(|snapshot| snapshot.branch())
            .map(str::to_owned)
            .ok_or_else(|| "git push requires a current branch".to_owned())?
    };
    run_git_push_in_popup_buffer(
        runtime,
        vec![
            "push".to_owned(),
            "--progress".to_owned(),
            "--set-upstream".to_owned(),
            remote.to_owned(),
            branch,
        ],
    )
}

pub(crate) fn open_git_remote_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    let remotes = git_remote_list(runtime, &root)?;
    if remotes.is_empty() {
        return Err("no git remotes found".to_owned());
    }
    let entries = remotes
        .into_iter()
        .map(|remote| {
            let item_id = format!("git-remote:{remote}");
            let action = PickerAction::GitPushRemote(remote.clone());
            PickerEntry {
                item: PickerItem::new(item_id, remote.clone(), "remote", None::<String>),
                action,
                quickfix: None,
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries("Git Push", entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn open_git_fetch_remote_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    let remotes = git_remote_list(runtime, &root)?;
    if remotes.is_empty() {
        return Err("no git remotes found".to_owned());
    }
    let entries = remotes
        .into_iter()
        .map(|remote| {
            let item_id = format!("git-fetch-remote:{remote}");
            let action = PickerAction::GitFetchRemote(remote.clone());
            PickerEntry {
                item: PickerItem::new(item_id, remote.clone(), "remote", None::<String>),
                action,
                quickfix: None,
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries("Git Fetch", entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn git_snapshot_for_buffer(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<GitStatusSnapshot, String> {
    shell_buffer(runtime, buffer_id)?
        .git_snapshot()
        .cloned()
        .ok_or_else(|| "git status snapshot is missing".to_owned())
}

pub(crate) fn remote_name_from_ref(reference: &str) -> Option<String> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.split('/').next().unwrap_or(trimmed).to_owned())
}

pub(crate) fn remote_and_branch_from_ref(reference: &str) -> Option<(String, String)> {
    let trimmed = reference.trim();
    let (remote, branch) = trimmed.split_once('/')?;
    if remote.is_empty() || branch.is_empty() {
        return None;
    }
    Some((remote.to_owned(), branch.to_owned()))
}

pub(crate) fn git_config_get(root: &Path, key: &str) -> Option<String> {
    git_read_command_output_optional(
        root,
        &format!("config --get {key}"),
        &["config", "--get", key],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
}

pub(crate) fn git_branch_push_remote(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.pushRemote"))
}

pub(crate) fn git_branch_remote(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.remote")).filter(|remote| remote != ".")
}

pub(crate) fn git_branch_merge(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.merge"))
}

pub(crate) fn local_branch_name_from_merge_ref(reference: &str) -> Option<String> {
    reference
        .strip_prefix("refs/heads/")
        .map(str::to_owned)
        .filter(|branch| !branch.is_empty())
}

pub(crate) fn git_push_remote_name(root: &Path, snapshot: &GitStatusSnapshot) -> Option<String> {
    let branch = snapshot.branch()?;
    let upstream_branch = git_branch_merge(root, branch)
        .as_deref()
        .and_then(local_branch_name_from_merge_ref);
    git_branch_push_remote(root, branch)
        .or_else(|| git_config_get(root, "remote.pushDefault"))
        .or_else(|| git_branch_remote(root, branch))
        .or_else(|| {
            upstream_branch
                .as_deref()
                .and_then(|upstream_branch| git_branch_push_remote(root, upstream_branch))
        })
        .or_else(|| {
            upstream_branch
                .as_deref()
                .and_then(|upstream_branch| git_branch_remote(root, upstream_branch))
        })
        .or_else(|| snapshot.upstream().and_then(remote_name_from_ref))
        .or_else(|| snapshot.push_remote().and_then(remote_name_from_ref))
}

pub(crate) fn status_output_upstream(status_output: &str) -> Option<String> {
    status_output.lines().find_map(|line| {
        let line = line.strip_prefix("## ")?;
        let (_, tracking) = line.split_once("...")?;
        let upstream = tracking
            .split_once(" [")
            .map(|(upstream, _)| upstream)
            .unwrap_or(tracking)
            .trim();
        (!upstream.is_empty()).then(|| upstream.to_owned())
    })
}

pub(crate) fn fetch_git_remote(runtime: &mut EditorRuntime, remote: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Fetch",
            vec![
                "fetch".to_owned(),
                "--progress".to_owned(),
                remote.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn fetch_git_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Fetch",
            vec![
                "fetch".to_owned(),
                "--all".to_owned(),
                "--progress".to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn fetch_git_prune(runtime: &mut EditorRuntime, root: &Path) -> Result<(), String> {
    let remotes = git_remote_list(runtime, root)?;
    if remotes.is_empty() {
        return Err("no git remotes found".to_owned());
    }
    for remote in remotes {
        let refspec = format!("+refs/heads/*:refs/remotes/{remote}/*");
        run_command(
            runtime,
            ExternalCommandSpec::git_argv(
                "Git Fetch",
                vec![
                    "fetch".to_owned(),
                    "--prune".to_owned(),
                    remote.clone(),
                    refspec,
                ],
                root.to_path_buf(),
                StreamedCommandExitAction::LeaveOpen,
            )
            .with_stream(false)
            .with_notify(false, false),
        )?;
        // Silent success discards stdout; prune is for side effects.
    }
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn fetch_git_pushremote(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    if let Some(remote) = git_push_remote_name(&root, &snapshot) {
        fetch_git_remote(runtime, &remote)?;
        return Ok(());
    }
    open_git_fetch_remote_picker(runtime)
}

pub(crate) fn fetch_git_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let remote = snapshot
        .upstream()
        .and_then(remote_name_from_ref)
        .ok_or_else(|| "no upstream configured for fetch".to_owned())?;
    fetch_git_remote(runtime, &remote)
}

pub(crate) fn pull_git_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let (remote, branch) = snapshot
        .upstream()
        .and_then(remote_and_branch_from_ref)
        .ok_or_else(|| "no upstream configured for pull".to_owned())?;
    pull_git_remote_branch(runtime, &remote, &branch)
}

pub(crate) fn push_git_to_pushremote(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    if let Some(remote) = git_push_remote_name(&root, &snapshot) {
        push_git_remote(runtime, &remote)?;
        return Ok(());
    }
    open_git_remote_picker(runtime)
}

pub(crate) fn push_git_to_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let (remote, branch) = snapshot
        .upstream()
        .and_then(remote_and_branch_from_ref)
        .ok_or_else(|| "no upstream configured for push".to_owned())?;
    push_git_remote_branch(runtime, &remote, &branch)
}

pub(crate) fn push_git_remote_branch(
    runtime: &mut EditorRuntime,
    remote: &str,
    branch: &str,
) -> Result<(), String> {
    run_git_push_in_popup_buffer(
        runtime,
        vec![
            "push".to_owned(),
            "--progress".to_owned(),
            remote.to_owned(),
            branch.to_owned(),
        ],
    )
}

pub(crate) fn run_git_push_in_popup_buffer(
    runtime: &mut EditorRuntime,
    args: Vec<String>,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Push",
            args,
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(crate) fn pull_git_remote_branch(
    runtime: &mut EditorRuntime,
    remote: &str,
    branch: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Pull",
            vec![
                "pull".to_owned(),
                "--progress".to_owned(),
                remote.to_owned(),
                branch.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}
