use alacritty_terminal::{
    event::VoidListener,
    grid::{Dimensions, Scroll as GridScroll},
    index::{Column, Line, Point},
    term::{
        Config, Term,
        test::{TermSize, mock_term},
    },
    vte::ansi::Handler,
};

use crate::{
    TerminalNormalMotion, apply_terminal_normal_motion, begin_terminal_normal_mode,
    terminal_normal_cursor_viewport,
};

#[test]
fn terminal_normal_mode_moves_cursor_before_scrolling_with_scrolloff() {
    let size = TermSize::new(16, 6);
    let config = Config {
        scrolling_history: 64,
        ..Config::default()
    };
    let mut term = Term::new(config, &size, VoidListener);
    for index in 0..24 {
        for character in format!("line-{index:02}\n").chars() {
            term.input(character);
        }
    }

    begin_terminal_normal_mode(&mut term);
    term.vi_goto_point(Point::new(term.topmost_line(), Column(0)));
    let start_offset = term.grid().display_offset();
    assert!(start_offset > 0, "fixture should have scrollback history");

    let mut viewport_rows = Vec::new();
    for _ in 0..3 {
        assert!(apply_terminal_normal_motion(
            &mut term,
            TerminalNormalMotion::Down,
            None,
            2,
        ));
        let (row, _) =
            terminal_normal_cursor_viewport(&term).expect("cursor should stay in viewport");
        viewport_rows.push(row);
    }

    assert!(
        viewport_rows.windows(2).any(|pair| pair[1] > pair[0]),
        "cursor should move down the screen before scrollback advances: {viewport_rows:?}"
    );
    assert_eq!(
        term.grid().display_offset(),
        start_offset,
        "display should not scroll until the cursor reaches scrolloff"
    );

    while term.grid().display_offset() == start_offset {
        assert!(apply_terminal_normal_motion(
            &mut term,
            TerminalNormalMotion::Down,
            None,
            2,
        ));
        let (row, _) =
            terminal_normal_cursor_viewport(&term).expect("cursor should stay in viewport");
        assert!(
            row <= size.screen_lines().saturating_sub(1).saturating_sub(2),
            "cursor should remain within bottom scrolloff once scrolling starts (row={row})"
        );
    }
}

#[test]
fn terminal_normal_mode_paragraph_motion_moves_across_blank_lines() {
    let mut term = mock_term("alpha\n\nbeta\n\ngamma");
    begin_terminal_normal_mode(&mut term);
    term.vi_mode_cursor.point = Point::new(Line(0), Column(0));
    let start = term.vi_mode_cursor.point.line;

    assert!(apply_terminal_normal_motion(
        &mut term,
        TerminalNormalMotion::ParagraphForward,
        None,
        0,
    ));
    assert!(
        term.vi_mode_cursor.point.line > start,
        "paragraph motion should advance through blank-line separators"
    );
}

#[test]
fn terminal_normal_mode_gg_jumps_to_top_of_scrollback() {
    let size = TermSize::new(12, 4);
    let config = Config {
        scrolling_history: 32,
        ..Config::default()
    };
    let mut term = Term::new(config, &size, VoidListener);
    for index in 0..12 {
        for character in format!("row{index}\n").chars() {
            term.input(character);
        }
    }
    term.scroll_display(GridScroll::Bottom);
    begin_terminal_normal_mode(&mut term);

    assert!(apply_terminal_normal_motion(
        &mut term,
        TerminalNormalMotion::FirstLine,
        None,
        1,
    ));
    assert_eq!(term.vi_mode_cursor.point.line, term.topmost_line());
    assert_eq!(
        term.grid().display_offset(),
        term.history_size(),
        "gg should reveal the top of scrollback"
    );
}
