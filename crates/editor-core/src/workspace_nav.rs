//! Workspace navigation and Mark List domain seam.
//!
//! Pure inputs/outputs — no SDL. Cycling callers supply Project Workspaces already
//! in open order (Default Workspace excluded). Mark List operations preserve path order.

use std::path::{Path, PathBuf};

/// Strips Windows verbatim (`\\?\`) prefixes so canonical roots round-trip cleanly.
///
/// Non-Windows paths and already-plain Windows paths are returned unchanged.
pub fn normalize_project_root_path(path: &Path) -> PathBuf {
    let Some(raw) = path.to_str() else {
        return path.to_path_buf();
    };
    if let Some(unc) = raw.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{unc}"));
    }
    if let Some(rest) = raw.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path.to_path_buf()
}

/// Returns whether two project roots share Mark List path identity after normalization.
pub fn project_roots_equal(left: &Path, right: &Path) -> bool {
    normalize_project_root_path(left) == normalize_project_root_path(right)
}

/// Ordered app-wide project roots recorded as Marked Workspaces.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkList {
    roots: Vec<PathBuf>,
}

impl MarkList {
    /// Number of Mark List entries with dedicated jump bindings.
    pub const SLOT_COUNT: usize = 4;

    /// Parses one project root per non-blank line, preserving order.
    pub fn parse(text: &str) -> Self {
        Self {
            roots: text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(PathBuf::from)
                .collect(),
        }
    }

    /// Serializes one project root per line with a trailing newline when non-empty.
    pub fn serialize(&self) -> String {
        let mut text = self
            .roots
            .iter()
            .map(|root| root.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        if !text.is_empty() {
            text.push('\n');
        }
        text
    }

    /// Returns Marked Workspace roots in Mark List order.
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Builds a Mark List from already-ordered roots (caller owns normalization).
    pub fn from_roots(roots: Vec<PathBuf>) -> Self {
        Self { roots }
    }

    /// Appends a root when absent, returning whether the Mark List changed.
    pub fn mark(&mut self, root: &Path) -> bool {
        if self
            .roots
            .iter()
            .any(|marked| project_roots_equal(marked, root))
        {
            return false;
        }
        self.roots.push(normalize_project_root_path(root));
        true
    }

    /// Removes all occurrences of a root, returning whether the Mark List changed.
    pub fn unmark(&mut self, root: &Path) -> bool {
        let original_len = self.roots.len();
        self.roots
            .retain(|marked| !project_roots_equal(marked, root));
        self.roots.len() != original_len
    }

    /// Returns the Marked Workspace root for a dedicated jump slot.
    ///
    /// `index` is 0-based among the first [`Self::SLOT_COUNT`] entries. Empty
    /// slots and indexes outside the dedicated set yield [`None`] (silent no-op).
    pub fn slot(&self, index: usize) -> Option<&Path> {
        if index >= Self::SLOT_COUNT {
            return None;
        }
        self.roots.get(index).map(PathBuf::as_path)
    }
}

/// Intent for jumping to a Marked Workspace root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkedWorkspaceJump {
    /// Root is already an open Project Workspace — switch to it.
    Switch,
    /// Root exists on disk but is closed — open/create then switch.
    OpenThenSwitch,
    /// Root missing on disk — notify; leave Mark List unchanged.
    NotifyMissing,
}

/// Resolves jump intent for a filled Mark List slot.
///
/// Callers supply whether `root` exists on disk and the open Project Workspace
/// roots (Default Workspace excluded). Empty slots are handled by [`MarkList::slot`].
pub fn marked_workspace_jump(
    root: &Path,
    open_project_roots: &[impl AsRef<Path>],
    exists_on_disk: bool,
) -> MarkedWorkspaceJump {
    if !exists_on_disk {
        return MarkedWorkspaceJump::NotifyMissing;
    }
    if open_project_roots
        .iter()
        .any(|open| project_roots_equal(open.as_ref(), root))
    {
        MarkedWorkspaceJump::Switch
    } else {
        MarkedWorkspaceJump::OpenThenSwitch
    }
}

/// Direction to move when cycling Project Workspaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleDirection {
    /// Next Project Workspace in open order (wraps).
    Next,
    /// Previous Project Workspace in open order (wraps).
    Previous,
}

/// Returns the next/previous Project Workspace target in open order.
///
/// `project_workspaces` must already exclude the Default Workspace and be ordered
/// by open order. Returns [`None`] when fewer than two Project Workspaces are open
/// (caller should silently no-op). When `active` is not in the list (e.g. the
/// Default Workspace is focused), Next yields the first entry and Previous the last.
pub fn cycle_project_workspace<T: PartialEq + Copy>(
    project_workspaces: &[T],
    active: &T,
    direction: CycleDirection,
) -> Option<T> {
    if project_workspaces.len() < 2 {
        return None;
    }
    let Some(index) = project_workspaces.iter().position(|id| id == active) else {
        return Some(match direction {
            CycleDirection::Next => project_workspaces[0],
            CycleDirection::Previous => *project_workspaces.last()?,
        });
    };
    let next_index = match direction {
        CycleDirection::Next => (index + 1) % project_workspaces.len(),
        CycleDirection::Previous => {
            if index == 0 {
                project_workspaces.len() - 1
            } else {
                index - 1
            }
        }
    };
    Some(project_workspaces[next_index])
}

/// Streamed `git worktree remove` request shape (no Mark List mutation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeRemoveRequest {
    /// Absolute Worktree path to remove from disk.
    pub path: PathBuf,
    /// Args after `git`: `worktree remove <path> --force`.
    pub args: Vec<String>,
}

/// Planned Worktree Remove: matching Project Workspaces to close + git request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeRemovePlan<Id: Clone> {
    /// Every open Project Workspace whose root matches the Worktree path (incl. active).
    pub workspace_ids_to_close: Vec<Id>,
    /// Streamed remove command request.
    pub request: WorktreeRemoveRequest,
}

/// Plans Worktree Remove from a snapshotted path and open Project Workspace roots.
///
/// Missing path (create affordance / empty selection) → [`None`] (silent no-op).
/// Mark List is not part of the plan.
pub fn plan_worktree_remove<Id: Clone>(
    selected_path: Option<&Path>,
    open_project_workspaces: &[(Id, impl AsRef<Path>)],
) -> Option<WorktreeRemovePlan<Id>> {
    let path = selected_path?;
    let workspace_ids_to_close = open_project_workspaces
        .iter()
        .filter(|(_, root)| project_roots_equal(root.as_ref(), path))
        .map(|(id, _)| id.clone())
        .collect();
    let path_display = path.display().to_string();
    Some(WorktreeRemovePlan {
        workspace_ids_to_close,
        request: WorktreeRemoveRequest {
            path: normalize_project_root_path(path),
            args: vec![
                "worktree".to_owned(),
                "remove".to_owned(),
                path_display,
                "--force".to_owned(),
            ],
        },
    })
}

#[cfg(test)]
#[path = "workspace_nav_tests.rs"]
mod tests;
