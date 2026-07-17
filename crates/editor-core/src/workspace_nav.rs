//! Workspace navigation domain seam (cycle among Project Workspaces).
//!
//! Pure inputs/outputs — no SDL. Callers supply Project Workspaces already in open
//! order (Default Workspace excluded). Fewer than two → [`None`] (silent no-op).

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
}
