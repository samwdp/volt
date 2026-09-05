pub(super) fn merge_git_continue(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn merge_git_abort(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn rebase_git_onto(runtime: &mut EditorRuntime, target: &str) -> Result<(), String> {
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

pub(super) fn rebase_git_interactive_onto(
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

pub(super) fn rebase_git_onto_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let upstream = snapshot
        .upstream()
        .ok_or_else(|| "no upstream configured for rebase".to_owned())?;
    rebase_git_onto(runtime, upstream)
}

pub(super) fn rebase_git_onto_pushremote(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let push_remote = snapshot
        .push_remote()
        .ok_or_else(|| "no push-remote configured for rebase".to_owned())?;
    rebase_git_onto(runtime, push_remote)
}

pub(super) fn rebase_git_continue(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn rebase_git_skip(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn rebase_git_edit_todo(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn rebase_git_abort(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn open_git_cherry_buffer(
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

pub(super) fn push_git_remote(runtime: &mut EditorRuntime, remote: &str) -> Result<(), String> {
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

pub(super) fn open_git_remote_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn open_git_fetch_remote_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn git_snapshot_for_buffer(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<GitStatusSnapshot, String> {
    shell_buffer(runtime, buffer_id)?
        .git_snapshot()
        .cloned()
        .ok_or_else(|| "git status snapshot is missing".to_owned())
}

pub(super) fn remote_name_from_ref(reference: &str) -> Option<String> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.split('/').next().unwrap_or(trimmed).to_owned())
}

pub(super) fn remote_and_branch_from_ref(reference: &str) -> Option<(String, String)> {
    let trimmed = reference.trim();
    let (remote, branch) = trimmed.split_once('/')?;
    if remote.is_empty() || branch.is_empty() {
        return None;
    }
    Some((remote.to_owned(), branch.to_owned()))
}

fn git_config_get(root: &Path, key: &str) -> Option<String> {
    git_read_command_output_optional(
        root,
        &format!("config --get {key}"),
        &["config", "--get", key],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
}

fn git_branch_push_remote(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.pushRemote"))
}

fn git_branch_remote(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.remote")).filter(|remote| remote != ".")
}

fn git_branch_merge(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.merge"))
}

fn local_branch_name_from_merge_ref(reference: &str) -> Option<String> {
    reference
        .strip_prefix("refs/heads/")
        .map(str::to_owned)
        .filter(|branch| !branch.is_empty())
}

fn git_push_remote_name(root: &Path, snapshot: &GitStatusSnapshot) -> Option<String> {
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

fn status_output_upstream(status_output: &str) -> Option<String> {
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

pub(super) fn fetch_git_remote(runtime: &mut EditorRuntime, remote: &str) -> Result<(), String> {
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

pub(super) fn fetch_git_all(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn fetch_git_prune(runtime: &mut EditorRuntime, root: &Path) -> Result<(), String> {
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

pub(super) fn fetch_git_pushremote(
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

pub(super) fn fetch_git_upstream(
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

pub(super) fn pull_git_upstream(
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

pub(super) fn push_git_to_pushremote(
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

pub(super) fn push_git_to_upstream(
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

pub(super) fn push_git_remote_branch(
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

fn run_git_push_in_popup_buffer(
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

pub(super) fn pull_git_remote_branch(
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

pub(super) fn git_branch_list(
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

pub(super) fn git_remote_worktree_branch_list(
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

pub(super) fn git_commit_list(
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

pub(super) fn open_git_commit_picker_with_action(
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

pub(super) fn open_git_branch_picker_with_action(
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

pub(super) fn open_git_branch_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(runtime, "Git Branches", GitBranchActionKind::Checkout)
}

pub(super) fn open_git_worktree_branch_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    let entries = git_remote_worktree_branch_list(runtime, &root)?
        .into_iter()
        .map(|(remote_branch, local_branch)| {
            let item_id = format!("git-worktree-branch:{remote_branch}");
            let action = PickerAction::GitWorktreeBranch {
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
    if entries.is_empty() {
        return Err("no remote-only branches found".to_owned());
    }
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries("Git Worktree Branch", entries));
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GitWorktreeListEntry {
    /// Exact `worktree` path from `git worktree list --porcelain` (Git identity).
    raw_path: String,
    /// Normalized filesystem path for workspace / picker use.
    path: PathBuf,
    branch: Option<String>,
    head: Option<String>,
    bare: bool,
    /// `prunable` in porcelain — checkout/admin already broken; `remove` cannot succeed.
    prunable: bool,
}

/// Git invocation for Worktree Remove after resolving the registered worktree entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WorktreeRemoveGitInvocation {
    /// `git worktree remove <raw_path> --force` using the porcelain path spelling.
    Remove { cli_path: String },
    /// `git worktree prune -v` when the entry is already prunable.
    Prune,
}

/// Picks remove vs prune and the CLI path Git will recognize.
///
/// Git for Windows may register worktrees as `/w/...` while Volt normalizes to `W:\...`.
/// `git worktree remove` matches the registered spelling, so args must use [`GitWorktreeListEntry::raw_path`].
pub(super) fn worktree_remove_git_invocation_for_entries(
    entries: &[GitWorktreeListEntry],
    selected: &Path,
) -> WorktreeRemoveGitInvocation {
    let Some(entry) = entries
        .iter()
        .find(|entry| !entry.bare && project_roots_equal(&entry.path, selected))
    else {
        return WorktreeRemoveGitInvocation::Remove {
            cli_path: selected.display().to_string(),
        };
    };
    if entry.prunable {
        WorktreeRemoveGitInvocation::Prune
    } else {
        WorktreeRemoveGitInvocation::Remove {
            cli_path: entry.raw_path.clone(),
        }
    }
}

pub(super) fn worktree_remove_git_args(invocation: &WorktreeRemoveGitInvocation) -> Vec<String> {
    match invocation {
        WorktreeRemoveGitInvocation::Remove { cli_path } => vec![
            "worktree".to_owned(),
            "remove".to_owned(),
            cli_path.clone(),
            "--force".to_owned(),
        ],
        WorktreeRemoveGitInvocation::Prune => {
            vec!["worktree".to_owned(), "prune".to_owned(), "-v".to_owned()]
        }
    }
}

impl GitWorktreeListEntry {
    fn display_name(&self, base_dir: &Path) -> String {
        if let Some(path) = self
            .path
            .strip_prefix(base_dir)
            .ok()
            .and_then(|path| path.to_str())
            .filter(|path| !path.is_empty())
        {
            return path.to_owned();
        }
        if let Some(name) = self.path.file_name().and_then(|name| name.to_str()) {
            return name.to_owned();
        }
        self.path.to_string_lossy().into_owned()
    }

    fn detail(&self) -> String {
        let mut parts = Vec::new();
        if let Some(branch) = &self.branch {
            parts.push(branch.clone());
        }
        if let Some(head) = &self.head {
            parts.push(head.chars().take(12).collect::<String>());
        }
        if self.bare {
            parts.push("bare".to_owned());
        }
        parts.join(" | ")
    }
}

pub(super) fn git_worktree_dashboard_picker_overlay(
    runtime: &EditorRuntime,
) -> Result<PickerOverlay, String> {
    let base_dir = worktree_dashboard_base_dir(runtime)?;
    let worktrees = git_worktree_list(&base_dir)?;
    let mut entries = worktrees
        .into_iter()
        .filter(|entry| !entry.bare)
        .map(|entry| {
            let existing_workspace = find_workspace_by_root(runtime, &entry.path)?;
            let name = entry.display_name(&base_dir);
            let detail = {
                let mut detail = entry.detail();
                if existing_workspace.is_some() {
                    if !detail.is_empty() {
                        detail.push_str(" | ");
                    }
                    detail.push_str("open workspace");
                }
                detail
            };
            let action = existing_workspace.map_or(
                PickerAction::CreateWorkspace {
                    name: name.clone(),
                    root: entry.path.clone(),
                },
                PickerAction::SwitchWorkspace,
            );
            Ok(PickerEntry {
                item: PickerItem::new(
                    entry.path.display().to_string(),
                    name,
                    detail,
                    Some(entry.path.display().to_string()),
                ),
                action,
                quickfix: None,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    entries.insert(
        0,
        PickerEntry {
            item: PickerItem::new(
                "git-worktree-dashboard:create",
                "+ new worktree",
                base_dir.display().to_string(),
                Some("Open oil at the bare repo and choose a branch.".to_owned()),
            ),
            action: PickerAction::GitWorktreeDashboardCreate { base_dir },
            quickfix: None,
        },
    );

    Ok(PickerOverlay::from_entries("Workspace Dashboard", entries))
}

pub(super) fn open_git_worktree_dashboard_create(
    runtime: &mut EditorRuntime,
    base_dir: &Path,
) -> Result<(), String> {
    split_runtime_pane(runtime, PaneSplitDirection::Vertical)?;
    open_oil_directory(runtime, base_dir.to_path_buf())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    begin_oil_worktree_request(runtime, buffer_id)
}

/// Force-removes a Worktree from disk using one-shot picker context.
///
/// Closes matching Project Workspaces first, then streams
/// `git worktree remove <path> --force`. Missing path / create affordance → no-op.
pub(super) fn worktree_remove_from_one_shot(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(context) = shell_ui_mut(runtime)?.take_picker_one_shot() else {
        return Ok(());
    };
    let selected_path = context
        .selected()
        .and_then(PickerSelectedRow::path)
        .map(PathBuf::from);
    let open = open_project_workspaces_with_roots(runtime)?;
    let Some(plan) = plan_worktree_remove(selected_path.as_deref(), &open) else {
        return Ok(());
    };

    project_discovery_forget_candidate(&plan.request.path);

    for workspace_id in plan.workspace_ids_to_close {
        delete_runtime_workspace(runtime, workspace_id)?;
    }

    let cwd = worktree_remove_repo_cwd(runtime, &plan.request.path)?;
    let entries = git_worktree_list(&cwd)?;
    let invocation = worktree_remove_git_invocation_for_entries(&entries, &plan.request.path);
    if matches!(invocation, WorktreeRemoveGitInvocation::Prune)
        && plan.request.path.exists()
        && let Err(error) = std::fs::remove_dir_all(&plan.request.path)
    {
        return Err(format!(
            "failed to delete leftover worktree `{}`: {error}",
            plan.request.path.display()
        ));
    }
    let args = worktree_remove_git_args(&invocation);
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Worktree Remove",
            args,
            cwd,
            StreamedCommandExitAction::RefreshGitStatusCloseAndRescanProjects,
        ),
    )?;
    Ok(())
}

fn worktree_dashboard_base_dir(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_directory_root(runtime)? {
        return git_common_dir(&root).or(Ok(root));
    }
    if let Some(root) = active_workspace_root(runtime)? {
        return git_common_dir(&root).or_else(|_| {
            root.parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| format!("workspace root `{}` has no parent", root.display()))
        });
    }
    env::current_dir().map_err(|error| format!("workspace dashboard requires a root: {error}"))
}

/// Resolves a git cwd that can list/remove worktrees for `selected`.
///
/// Broken/prunable checkouts may lack `.git`, so fall back to open Project
/// Workspace roots and parent directories before giving up.
fn worktree_remove_repo_cwd(runtime: &EditorRuntime, selected: &Path) -> Result<PathBuf, String> {
    if let Ok(cwd) = git_common_dir(selected) {
        return Ok(cwd);
    }

    for (_, root) in open_project_workspaces_with_roots(runtime)? {
        if let Ok(cwd) = git_common_dir(&root) {
            return Ok(cwd);
        }
    }

    let mut cursor = selected.parent();
    while let Some(dir) = cursor {
        if let Ok(cwd) = git_common_dir(dir) {
            return Ok(cwd);
        }
        cursor = dir.parent();
    }

    selected.parent().map(Path::to_path_buf).ok_or_else(|| {
        format!(
            "cannot resolve git repository for worktree `{}`",
            selected.display()
        )
    })
}

fn git_common_dir(root: &Path) -> Result<PathBuf, String> {
    let output = git_read_command_output(
        root,
        "rev-parse --git-common-dir",
        &["rev-parse", "--git-common-dir"],
    )?;
    let common_dir = output.trim();
    if common_dir.is_empty() {
        return Err(format!(
            "git rev-parse --git-common-dir returned no path for {}",
            root.display()
        ));
    }
    let path = normalize_git_output_path(common_dir);
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    Ok(path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| *name == ".git")
        .and_then(|_| path.parent().map(Path::to_path_buf))
        .unwrap_or(path))
}

fn git_worktree_list(root: &Path) -> Result<Vec<GitWorktreeListEntry>, String> {
    parse_git_worktree_list(&git_read_command_output(
        root,
        "worktree list",
        &["worktree", "list", "--porcelain"],
    )?)
}

pub(super) fn parse_git_worktree_list(output: &str) -> Result<Vec<GitWorktreeListEntry>, String> {
    let mut entries = Vec::new();
    let mut current: Option<GitWorktreeListEntry> = None;
    for line in output.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            continue;
        }
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(GitWorktreeListEntry {
                raw_path: path.to_owned(),
                path: normalize_git_output_path(path),
                branch: None,
                head: None,
                bare: false,
                prunable: false,
            });
        } else if let Some(entry) = current.as_mut() {
            if let Some(branch) = line.strip_prefix("branch ") {
                entry.branch = Some(branch.trim_start_matches("refs/heads/").to_owned());
            } else if let Some(head) = line.strip_prefix("HEAD ") {
                entry.head = Some(head.to_owned());
            } else if line == "bare" {
                entry.bare = true;
            } else if line.starts_with("prunable") {
                entry.prunable = true;
            }
        } else {
            return Err(format!(
                "git worktree list returned unexpected line `{line}`"
            ));
        }
    }
    if let Some(entry) = current {
        entries.push(entry);
    }
    Ok(entries)
}
