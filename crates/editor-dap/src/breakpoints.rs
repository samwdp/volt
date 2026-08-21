//! Workspace-scoped in-memory Breakpoint store.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Verification state for a stored Breakpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakpointState {
    /// Not yet confirmed by a live Debug Adapter (or Session idle).
    #[default]
    Pending,
    /// Adapter reported the Breakpoint as verified.
    Verified,
    /// Adapter rejected or could not bind the Breakpoint.
    Unverified,
}

/// One line Breakpoint stored for a Workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredBreakpoint {
    path: PathBuf,
    /// 1-based DAP line number.
    line: u32,
    state: BreakpointState,
}

impl StoredBreakpoint {
    /// Creates a pending Breakpoint at `path`:`line` (1-based).
    pub fn new(path: impl Into<PathBuf>, line: u32) -> Self {
        Self {
            path: path.into(),
            line,
            state: BreakpointState::Pending,
        }
    }

    /// Returns the source path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the 1-based DAP line.
    pub const fn line(&self) -> u32 {
        self.line
    }

    /// Returns verification state.
    pub const fn state(&self) -> BreakpointState {
        self.state
    }

    /// Returns true when the adapter verified this Breakpoint.
    pub const fn is_verified(&self) -> bool {
        matches!(self.state, BreakpointState::Verified)
    }

    /// Sets verification state after a `setBreakpoints` response.
    pub fn set_state(&mut self, state: BreakpointState) {
        self.state = state;
    }
}

/// Result of toggling a Breakpoint at a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointToggle {
    /// Breakpoint was added (pending until sync).
    Added,
    /// Existing Breakpoint was removed.
    Removed,
}

/// Workspace-scoped in-memory Breakpoint store for the app lifetime.
#[derive(Debug, Default, Clone)]
pub struct BreakpointStore {
    by_workspace: BTreeMap<u64, Vec<StoredBreakpoint>>,
}

impl BreakpointStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists Breakpoints for a Workspace (sorted by path, then line).
    pub fn list(&self, workspace_id: u64) -> Vec<StoredBreakpoint> {
        self.by_workspace
            .get(&workspace_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Returns Breakpoints for one source path inside a Workspace.
    pub fn for_path(&self, workspace_id: u64, path: &Path) -> Vec<StoredBreakpoint> {
        self.list(workspace_id)
            .into_iter()
            .filter(|bp| paths_equal(bp.path(), path))
            .collect()
    }

    /// Returns the Breakpoint at `path`:`line` if present.
    pub fn get(&self, workspace_id: u64, path: &Path, line: u32) -> Option<&StoredBreakpoint> {
        self.by_workspace
            .get(&workspace_id)?
            .iter()
            .find(|bp| paths_equal(bp.path(), path) && bp.line() == line)
    }

    /// Toggles a Breakpoint at `path`:`line` (1-based). Survives buffer close.
    pub fn toggle(
        &mut self,
        workspace_id: u64,
        path: impl Into<PathBuf>,
        line: u32,
    ) -> BreakpointToggle {
        let path = path.into();
        let entries = self.by_workspace.entry(workspace_id).or_default();
        if let Some(index) = entries
            .iter()
            .position(|bp| paths_equal(bp.path(), &path) && bp.line() == line)
        {
            entries.remove(index);
            if entries.is_empty() {
                self.by_workspace.remove(&workspace_id);
            }
            BreakpointToggle::Removed
        } else {
            entries.push(StoredBreakpoint::new(path, line));
            entries.sort_by(|left, right| {
                left.path()
                    .cmp(right.path())
                    .then_with(|| left.line().cmp(&right.line()))
            });
            BreakpointToggle::Added
        }
    }

    /// Deletes a Breakpoint at `path`:`line`. Returns whether one was removed.
    pub fn delete(&mut self, workspace_id: u64, path: &Path, line: u32) -> bool {
        let Some(entries) = self.by_workspace.get_mut(&workspace_id) else {
            return false;
        };
        let Some(index) = entries
            .iter()
            .position(|bp| paths_equal(bp.path(), path) && bp.line() == line)
        else {
            return false;
        };
        entries.remove(index);
        if entries.is_empty() {
            self.by_workspace.remove(&workspace_id);
        }
        true
    }

    /// Distinct source paths that have Breakpoints in a Workspace.
    pub fn source_paths(&self, workspace_id: u64) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();
        for bp in self.list(workspace_id) {
            if !paths
                .iter()
                .any(|existing| paths_equal(existing, bp.path()))
            {
                paths.push(bp.path().to_path_buf());
            }
        }
        paths
    }

    /// Applies adapter verification results for one source (same order as sent).
    pub fn apply_verification(&mut self, workspace_id: u64, path: &Path, results: &[(u32, bool)]) {
        let Some(entries) = self.by_workspace.get_mut(&workspace_id) else {
            return;
        };
        for (line, verified) in results {
            if let Some(bp) = entries
                .iter_mut()
                .find(|bp| paths_equal(bp.path(), path) && bp.line() == *line)
            {
                bp.set_state(if *verified {
                    BreakpointState::Verified
                } else {
                    BreakpointState::Unverified
                });
            }
        }
    }

    /// Marks every Breakpoint in a Workspace as pending (Session ended / not yet synced).
    pub fn mark_all_pending(&mut self, workspace_id: u64) {
        let Some(entries) = self.by_workspace.get_mut(&workspace_id) else {
            return;
        };
        for bp in entries {
            bp.set_state(BreakpointState::Pending);
        }
    }
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    // Compare lossless display strings so Windows drive letters stay case-insensitive enough
    // via PathBuf equality while still matching absolute paths used by the shell.
    left == right
}

#[cfg(test)]
mod tests {
    use super::{BreakpointState, BreakpointStore, BreakpointToggle};

    #[test]
    fn toggle_adds_and_removes_without_session() {
        let mut store = BreakpointStore::new();
        assert_eq!(
            store.toggle(1, "P:/demo/main.rs", 12),
            BreakpointToggle::Added
        );
        assert_eq!(store.list(1).len(), 1);
        assert_eq!(
            store.toggle(1, "P:/demo/main.rs", 12),
            BreakpointToggle::Removed
        );
        assert!(store.list(1).is_empty());
    }

    #[test]
    fn delete_removes_current_line_breakpoint() {
        let mut store = BreakpointStore::new();
        store.toggle(2, "src/lib.rs", 4);
        store.toggle(2, "src/lib.rs", 8);
        assert!(store.delete(2, std::path::Path::new("src/lib.rs"), 4));
        let remaining = store.list(2);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].line(), 8);
    }

    #[test]
    fn store_survives_conceptually_across_buffer_close() {
        // Buffer close does not touch the store; listing after "close" still returns entries.
        let mut store = BreakpointStore::new();
        store.toggle(9, "a.rs", 1);
        store.toggle(9, "b.rs", 2);
        assert_eq!(store.list(9).len(), 2);
        assert_eq!(store.for_path(9, std::path::Path::new("a.rs")).len(), 1);
    }

    #[test]
    fn verification_updates_state_from_adapter() {
        let mut store = BreakpointStore::new();
        store.toggle(3, "main.rs", 10);
        store.toggle(3, "main.rs", 20);
        store.apply_verification(
            3,
            std::path::Path::new("main.rs"),
            &[(10, true), (20, false)],
        );
        let bps = store.for_path(3, std::path::Path::new("main.rs"));
        assert_eq!(bps[0].state(), BreakpointState::Verified);
        assert_eq!(bps[1].state(), BreakpointState::Unverified);
    }

    #[test]
    fn workspaces_are_isolated() {
        let mut store = BreakpointStore::new();
        store.toggle(1, "a.rs", 1);
        store.toggle(2, "a.rs", 1);
        assert_eq!(store.list(1).len(), 1);
        store.delete(1, std::path::Path::new("a.rs"), 1);
        assert!(store.list(1).is_empty());
        assert_eq!(store.list(2).len(), 1);
    }
}
