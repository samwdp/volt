use super::{BreakpointState, BreakpointStore, BreakpointToggle, debug_source_paths_eq};
use std::path::Path;

#[test]
fn debug_source_paths_eq_ignores_slash_style_and_windows_case() {
    assert!(debug_source_paths_eq(
        Path::new(r"P:\Testing\Program.cs"),
        Path::new("P:/Testing/Program.cs"),
    ));
    #[cfg(windows)]
    assert!(debug_source_paths_eq(
        Path::new(r"P:\Testing\Program.cs"),
        Path::new(r"p:\testing\program.cs"),
    ));
    assert!(!debug_source_paths_eq(
        Path::new(r"P:\Testing\Program.cs"),
        Path::new(r"P:\Testing\Other.cs"),
    ));
}

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

#[test]
fn extras_persist_on_stored_breakpoint() {
    let mut store = BreakpointStore::new();
    store.upsert_extras(
        4,
        "main.rs",
        7,
        Some(Some("x > 1".to_owned())),
        Some(Some("5".to_owned())),
        Some(Some("hit {x}".to_owned())),
    );
    let bp = store
        .get(4, std::path::Path::new("main.rs"), 7)
        .expect("bp");
    assert_eq!(bp.condition(), Some("x > 1"));
    assert_eq!(bp.hit_condition(), Some("5"));
    assert_eq!(bp.log_message(), Some("hit {x}"));
    assert!(store.update_extras(
        4,
        std::path::Path::new("main.rs"),
        7,
        Some(None),
        None,
        None,
    ));
    let bp = store
        .get(4, std::path::Path::new("main.rs"), 7)
        .expect("bp");
    assert_eq!(bp.condition(), None);
    assert_eq!(bp.hit_condition(), Some("5"));
}
