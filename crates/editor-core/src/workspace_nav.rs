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

    #[test]
    fn slot_returns_first_four_marked_workspaces_and_empty_beyond_list() {
        let marks = MarkList::parse("P:\\a\nP:\\b\nP:\\c\nP:\\d\nP:\\e\n");

        assert_eq!(marks.slot(0), Some(Path::new(r"P:\a")));
        assert_eq!(marks.slot(1), Some(Path::new(r"P:\b")));
        assert_eq!(marks.slot(2), Some(Path::new(r"P:\c")));
        assert_eq!(marks.slot(3), Some(Path::new(r"P:\d")));
        assert_eq!(marks.slot(4), None);

        let short = MarkList::parse("P:\\only\n");
        assert_eq!(short.slot(0), Some(Path::new(r"P:\only")));
        assert_eq!(short.slot(1), None);
        assert_eq!(short.slot(2), None);
        assert_eq!(short.slot(3), None);
    }

    #[test]
    fn marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing() {
        let open = [PathBuf::from(r"P:\open-a"), PathBuf::from(r"P:\open-b")];

        assert_eq!(
            marked_workspace_jump(Path::new(r"P:\open-a"), &open, true),
            MarkedWorkspaceJump::Switch
        );
        assert_eq!(
            marked_workspace_jump(Path::new(r"P:\closed"), &open, true),
            MarkedWorkspaceJump::OpenThenSwitch
        );
        assert_eq!(
            marked_workspace_jump(Path::new(r"P:\gone"), &open, false),
            MarkedWorkspaceJump::NotifyMissing
        );
    }

    #[test]
    fn normalize_project_root_path_strips_windows_verbatim_prefix() {
        assert_eq!(
            normalize_project_root_path(Path::new(r"\\?\P:\volt")),
            PathBuf::from(r"P:\volt")
        );
        assert_eq!(
            normalize_project_root_path(Path::new(r"\\?\UNC\server\share\repo")),
            PathBuf::from(r"\\server\share\repo")
        );
        assert_eq!(
            normalize_project_root_path(Path::new(r"P:\volt")),
            PathBuf::from(r"P:\volt")
        );
    }

    #[test]
    fn project_roots_equal_treats_verbatim_and_plain_spellings_as_same_identity() {
        assert!(project_roots_equal(
            Path::new(r"\\?\P:\volt"),
            Path::new(r"P:\volt")
        ));
        assert!(!project_roots_equal(
            Path::new(r"P:\volt"),
            Path::new(r"P:\other")
        ));
    }

    #[test]
    fn mark_and_jump_use_normalized_path_identity() {
        let mut marks = MarkList::parse("P:\\volt\n");
        assert!(!marks.mark(Path::new(r"\\?\P:\volt")));
        assert!(marks.unmark(Path::new(r"\\?\P:\volt")));
        assert!(marks.roots().is_empty());

        let open = [PathBuf::from(r"\\?\P:\open-a")];
        assert_eq!(
            marked_workspace_jump(Path::new(r"P:\open-a"), &open, true),
            MarkedWorkspaceJump::Switch
        );
    }
}
