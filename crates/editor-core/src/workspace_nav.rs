//! Workspace navigation and Mark List domain seam.
//!
//! Pure inputs/outputs — no SDL. Cycling callers supply Project Workspaces already
//! in open order (Default Workspace excluded). Mark List operations preserve path order.

use std::path::{Path, PathBuf};

/// Ordered app-wide project roots recorded as Marked Workspaces.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkList {
    roots: Vec<PathBuf>,
}

impl MarkList {
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

    /// Appends a root when absent, returning whether the Mark List changed.
    pub fn mark(&mut self, root: &Path) -> bool {
        if self.roots.iter().any(|marked| marked == root) {
            return false;
        }
        self.roots.push(root.to_path_buf());
        true
    }

    /// Removes all occurrences of a root, returning whether the Mark List changed.
    pub fn unmark(&mut self, root: &Path) -> bool {
        let original_len = self.roots.len();
        self.roots.retain(|marked| marked != root);
        self.roots.len() != original_len
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn next_moves_forward_in_open_order() {
        let open = ["a", "b", "c"];
        assert_eq!(
            cycle_project_workspace(&open, &"a", CycleDirection::Next),
            Some("b")
        );
        assert_eq!(
            cycle_project_workspace(&open, &"b", CycleDirection::Next),
            Some("c")
        );
    }

    #[test]
    fn previous_moves_backward_in_open_order() {
        let open = ["a", "b", "c"];
        assert_eq!(
            cycle_project_workspace(&open, &"c", CycleDirection::Previous),
            Some("b")
        );
        assert_eq!(
            cycle_project_workspace(&open, &"b", CycleDirection::Previous),
            Some("a")
        );
    }

    #[test]
    fn next_and_previous_wrap_at_ends() {
        let open = ["a", "b", "c"];
        assert_eq!(
            cycle_project_workspace(&open, &"c", CycleDirection::Next),
            Some("a")
        );
        assert_eq!(
            cycle_project_workspace(&open, &"a", CycleDirection::Previous),
            Some("c")
        );
    }

    #[test]
    fn fewer_than_two_project_workspaces_yields_none() {
        assert_eq!(
            cycle_project_workspace(&["only"], &"only", CycleDirection::Next),
            None
        );
        assert_eq!(
            cycle_project_workspace::<&str>(&[], &"x", CycleDirection::Previous),
            None
        );
    }

    #[test]
    fn default_workspace_not_in_list_enters_cycle_at_ends() {
        // Caller already skipped Default Workspace from `open`; active may still be it.
        let open = ["a", "b"];
        assert_eq!(
            cycle_project_workspace(&open, &"default", CycleDirection::Next),
            Some("a")
        );
        assert_eq!(
            cycle_project_workspace(&open, &"default", CycleDirection::Previous),
            Some("b")
        );
    }

    #[test]
    fn mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order() {
        let marks =
            MarkList::parse(" P:\\alpha \n\nP:\\beta\n  \nP:\\gamma\nP:\\delta\nP:\\extra\n");

        assert_eq!(
            marks.roots(),
            &[
                PathBuf::from(r"P:\alpha"),
                PathBuf::from(r"P:\beta"),
                PathBuf::from(r"P:\gamma"),
                PathBuf::from(r"P:\delta"),
                PathBuf::from(r"P:\extra"),
            ]
        );
        assert_eq!(
            marks.serialize(),
            "P:\\alpha\nP:\\beta\nP:\\gamma\nP:\\delta\nP:\\extra\n"
        );
    }

    #[test]
    fn mark_appends_absent_root_and_duplicate_is_no_op() {
        let mut marks = MarkList::parse("P:\\alpha\n");

        assert!(marks.mark(Path::new(r"P:\beta")));
        assert!(!marks.mark(Path::new(r"P:\alpha")));
        assert_eq!(
            marks.roots(),
            &[PathBuf::from(r"P:\alpha"), PathBuf::from(r"P:\beta")]
        );
    }

    #[test]
    fn unmark_removes_root_without_reordering_remaining_marks() {
        let mut marks = MarkList::parse("P:\\alpha\nP:\\beta\nP:\\gamma\n");

        assert!(marks.unmark(Path::new(r"P:\beta")));
        assert!(!marks.unmark(Path::new(r"P:\missing")));
        assert_eq!(
            marks.roots(),
            &[PathBuf::from(r"P:\alpha"), PathBuf::from(r"P:\gamma")]
        );
    }
}
