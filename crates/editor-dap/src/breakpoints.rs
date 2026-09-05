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
    condition: Option<String>,
    hit_condition: Option<String>,
    log_message: Option<String>,
}

impl StoredBreakpoint {
    /// Creates a pending Breakpoint at `path`:`line` (1-based).
    pub fn new(path: impl Into<PathBuf>, line: u32) -> Self {
        Self {
            path: path.into(),
            line,
            state: BreakpointState::Pending,
            condition: None,
            hit_condition: None,
            log_message: None,
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

    /// Conditional expression, when set.
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }

    /// Hit-condition expression, when set.
    pub fn hit_condition(&self) -> Option<&str> {
        self.hit_condition.as_deref()
    }

    /// Logpoint message, when set.
    pub fn log_message(&self) -> Option<&str> {
        self.log_message.as_deref()
    }

    /// Sets verification state after a `setBreakpoints` response.
    pub fn set_state(&mut self, state: BreakpointState) {
        self.state = state;
    }

    /// Sets or clears the Breakpoint condition.
    pub fn set_condition(&mut self, condition: Option<String>) {
        self.condition = normalize_optional_text(condition);
    }

    /// Sets or clears the hit condition.
    pub fn set_hit_condition(&mut self, hit_condition: Option<String>) {
        self.hit_condition = normalize_optional_text(hit_condition);
    }

    /// Sets or clears the logpoint message.
    pub fn set_log_message(&mut self, log_message: Option<String>) {
        self.log_message = normalize_optional_text(log_message);
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
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

    /// Updates Breakpoint extras at `path`:`line`. Returns false when missing.
    pub fn update_extras(
        &mut self,
        workspace_id: u64,
        path: &Path,
        line: u32,
        condition: Option<Option<String>>,
        hit_condition: Option<Option<String>>,
        log_message: Option<Option<String>>,
    ) -> bool {
        let Some(entries) = self.by_workspace.get_mut(&workspace_id) else {
            return false;
        };
        let Some(bp) = entries
            .iter_mut()
            .find(|bp| paths_equal(bp.path(), path) && bp.line() == line)
        else {
            return false;
        };
        if let Some(condition) = condition {
            bp.set_condition(condition);
        }
        if let Some(hit_condition) = hit_condition {
            bp.set_hit_condition(hit_condition);
        }
        if let Some(log_message) = log_message {
            bp.set_log_message(log_message);
        }
        true
    }

    /// Ensures a Breakpoint exists at `path`:`line` (pending), then updates extras.
    pub fn upsert_extras(
        &mut self,
        workspace_id: u64,
        path: impl Into<PathBuf>,
        line: u32,
        condition: Option<Option<String>>,
        hit_condition: Option<Option<String>>,
        log_message: Option<Option<String>>,
    ) {
        let path = path.into();
        if self.get(workspace_id, &path, line).is_none() {
            let _ = self.toggle(workspace_id, path.clone(), line);
        }
        let _ = self.update_extras(
            workspace_id,
            &path,
            line,
            condition,
            hit_condition,
            log_message,
        );
    }
}

/// Compares Debug Adapter source paths, ignoring slash style and Windows case.
pub fn debug_source_paths_eq(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    normalize_debug_source_path(left) == normalize_debug_source_path(right)
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    debug_source_paths_eq(left, right)
}

fn normalize_debug_source_path(path: &Path) -> String {
    let mut text = path.to_string_lossy().replace('/', "\\");
    if let Some(stripped) = text.strip_prefix("\\\\?\\") {
        text = stripped.to_owned();
    }
    let mut normalized = String::with_capacity(text.len());
    let mut prev_slash = false;
    for ch in text.chars() {
        if ch == '\\' {
            if !prev_slash || normalized.is_empty() {
                normalized.push('\\');
            }
            prev_slash = true;
            continue;
        }
        prev_slash = false;
        normalized.push(ch);
    }
    #[cfg(windows)]
    {
        normalized.make_ascii_lowercase();
    }
    normalized
}

#[cfg(test)]
#[path = "breakpoints_tests.rs"]
mod tests;
