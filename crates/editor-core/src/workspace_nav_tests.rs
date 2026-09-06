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
    let marks = MarkList::parse(" P:\\alpha \n\nP:\\beta\n  \nP:\\gamma\nP:\\delta\nP:\\extra\n");

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

#[test]
fn worktree_remove_missing_path_is_noop() {
    let open = [("ws-a", PathBuf::from(r"P:\repo\feature"))];
    assert!(plan_worktree_remove::<&str>(None, &open).is_none());
}

#[test]
fn worktree_remove_plans_matching_workspaces_and_force_remove_args() {
    let path = Path::new(r"P:\repo\feature");
    let open = [
        ("ws-main", PathBuf::from(r"P:\repo\main")),
        ("ws-feature", PathBuf::from(r"P:\repo\feature")),
        ("ws-feature-dup", PathBuf::from(r"\\?\P:\repo\feature")),
        ("ws-other", PathBuf::from(r"P:\other")),
    ];

    let plan = plan_worktree_remove(Some(path), &open).expect("path present");

    assert_eq!(
        plan.workspace_ids_to_close,
        vec!["ws-feature", "ws-feature-dup"]
    );
    assert_eq!(plan.request.path, PathBuf::from(r"P:\repo\feature"));
    assert_eq!(
        plan.request.args,
        vec![
            "worktree".to_owned(),
            "remove".to_owned(),
            r"P:\repo\feature".to_owned(),
            "--force".to_owned(),
        ]
    );
}

#[test]
fn worktree_remove_with_no_matching_workspaces_still_plans_git_remove() {
    let path = Path::new(r"P:\repo\stale");
    let open = [("ws-main", PathBuf::from(r"P:\repo\main"))];

    let plan = plan_worktree_remove(Some(path), &open).expect("path present");

    assert!(plan.workspace_ids_to_close.is_empty());
    assert_eq!(
        plan.request.args,
        vec![
            "worktree".to_owned(),
            "remove".to_owned(),
            r"P:\repo\stale".to_owned(),
            "--force".to_owned(),
        ]
    );
}
