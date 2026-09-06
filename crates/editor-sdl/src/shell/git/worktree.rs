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

pub(crate) fn open_git_worktree_branch_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
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
pub(crate) struct GitWorktreeListEntry {
    /// Exact `worktree` path from `git worktree list --porcelain` (Git identity).
    pub(crate) raw_path: String,
    /// Normalized filesystem path for workspace / picker use.
    pub(crate) path: PathBuf,
    pub(crate) branch: Option<String>,
    pub(crate) head: Option<String>,
    pub(crate) bare: bool,
    /// `prunable` in porcelain — checkout/admin already broken; `remove` cannot succeed.
    pub(crate) prunable: bool,
}

/// Git invocation for Worktree Remove after resolving the registered worktree entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorktreeRemoveGitInvocation {
    /// `git worktree remove <raw_path> --force` using the porcelain path spelling.
    Remove { cli_path: String },
    /// `git worktree prune -v` when the entry is already prunable.
    Prune,
}

/// Picks remove vs prune and the CLI path Git will recognize.
///
/// Git for Windows may register worktrees as `/w/...` while Volt normalizes to `W:\...`.
/// `git worktree remove` matches the registered spelling, so args must use [`GitWorktreeListEntry::raw_path`].
pub(crate) fn worktree_remove_git_invocation_for_entries(
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

pub(crate) fn worktree_remove_git_args(invocation: &WorktreeRemoveGitInvocation) -> Vec<String> {
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
    pub(crate) fn display_name(&self, base_dir: &Path) -> String {
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

    pub(crate) fn detail(&self) -> String {
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

pub(crate) fn git_worktree_dashboard_picker_overlay(
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

pub(crate) fn open_git_worktree_dashboard_create(
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
pub(crate) fn worktree_remove_from_one_shot(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(crate) fn worktree_dashboard_base_dir(runtime: &EditorRuntime) -> Result<PathBuf, String> {
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
pub(crate) fn worktree_remove_repo_cwd(
    runtime: &EditorRuntime,
    selected: &Path,
) -> Result<PathBuf, String> {
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

pub(crate) fn git_common_dir(root: &Path) -> Result<PathBuf, String> {
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

pub(crate) fn git_worktree_list(root: &Path) -> Result<Vec<GitWorktreeListEntry>, String> {
    parse_git_worktree_list(&git_read_command_output(
        root,
        "worktree list",
        &["worktree", "list", "--porcelain"],
    )?)
}

pub(crate) fn parse_git_worktree_list(output: &str) -> Result<Vec<GitWorktreeListEntry>, String> {
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

pub(crate) fn begin_oil_worktree_request(
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

pub(crate) fn open_git_worktree_new_branch_prompt(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.set_command_line(CommandLineOverlay::for_worktree_new_branch(buffer_id));
    Ok(())
}

pub(crate) fn submit_git_worktree_new_branch_name(
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

pub(crate) fn open_git_worktree_path_picker(
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

pub(crate) fn create_git_worktree_from_query(
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

pub(crate) fn finish_oil_worktree_branch_selection(
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

pub(crate) fn create_git_worktree(
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

pub(crate) fn worktree_path_from_name(base_dir: &Path, name: &str) -> Result<PathBuf, String> {
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

pub(crate) fn checkout_git_branch(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "checkout", &["checkout", branch])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}
