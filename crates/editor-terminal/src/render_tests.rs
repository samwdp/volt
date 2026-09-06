
use alacritty_terminal::{
    index::{Column, Line, Point},
    term::test::mock_term,
};
use editor_jobs::{JobManager, JobSpec};

use crate::{
    LiveTerminalConfig, LiveTerminalSession, TerminalCursorShape, TerminalKey, TerminalSession,
    TerminalStream, terminal_key_bytes, terminal_render_snapshot,
};

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("unexpected error: {error:?}"),
    }
}

#[test]
fn terminal_session_captures_transcript_lines() {
    let mut jobs = JobManager::new();
    let session = must(TerminalSession::run(
        &mut jobs,
        "Terminal",
        JobSpec::terminal("cargo-version", "cargo", ["--version"]),
    ));

    assert_eq!(session.title(), "Terminal");
    assert_eq!(session.command_label(), "cargo-version");
    assert!(session.transcript().succeeded());
    assert!(session.transcript().line_count() >= 1);
    assert_eq!(
        session.transcript().lines()[0].stream(),
        TerminalStream::Stdout
    );
    assert!(session.transcript().lines()[0].text().contains("cargo"));
}

#[test]
fn terminal_key_sequences_match_common_terminal_controls() {
    assert_eq!(terminal_key_bytes(TerminalKey::Enter), b"\r");
    assert_eq!(terminal_key_bytes(TerminalKey::Backspace), b"\x7f");
    assert_eq!(terminal_key_bytes(TerminalKey::Up), b"\x1b[A");
    assert_eq!(terminal_key_bytes(TerminalKey::PageDown), b"\x1b[6~");
    assert_eq!(terminal_key_bytes(TerminalKey::CtrlC), b"\x03");
}

#[test]
fn live_terminal_session_spawns_and_terminates() {
    let config = if cfg!(target_os = "windows") {
        LiveTerminalConfig::new("Terminal", "cmd", ["/Q".to_owned(), "/K".to_owned()])
    } else {
        LiveTerminalConfig::new("Terminal", "/bin/sh", Vec::<String>::new())
    }
    .with_size(12, 80);
    let mut session = must(LiveTerminalSession::spawn(config));
    if cfg!(not(target_os = "windows")) {
        assert!(session.process_id().is_some());
    }
    must(session.kill());
    assert!(session.has_exited());
}

#[test]
fn terminal_render_snapshot_tracks_visible_cursor() {
    let mut term = mock_term("hello\nworld");
    term.grid_mut().cursor.point = Point::new(Line(1), Column(3));
    let snapshot = terminal_render_snapshot(&term, 2, 5, None);

    assert_eq!(snapshot.rows(), 2);
    assert_eq!(snapshot.cols(), 5);
    assert_eq!(snapshot.lines()[0].runs()[0].text(), "hello");
    assert_eq!(snapshot.lines()[1].runs()[0].text(), "world");
    let cursor = snapshot.cursor().expect("cursor should be visible");
    assert_eq!(cursor.row(), 1);
    assert_eq!(cursor.col(), 3);
    assert_eq!(cursor.width_cells(), 1);
    assert_eq!(cursor.shape(), TerminalCursorShape::Block);
    assert_eq!(cursor.text(), "l");
}

#[test]
fn terminal_render_snapshot_preserves_wide_character_widths() {
    let term = mock_term("界a");
    let snapshot = terminal_render_snapshot(&term, 1, 3, None);

    assert_eq!(snapshot.lines().len(), 1);
    assert_eq!(snapshot.lines()[0].runs().len(), 1);
    let run = &snapshot.lines()[0].runs()[0];
    assert_eq!(run.text(), "界a");
    assert_eq!(run.width_cells(), 3);
}
