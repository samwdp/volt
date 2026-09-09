//! Normal-mode navigation for live terminals.
//!
//! Uses Alacritty vi-mode cursor motion on the full scrollback grid, then applies
//! editor-style scrolloff so the cursor moves through the viewport before the
//! display scrolls — matching regular buffer behavior.

use alacritty_terminal::{
    event::EventListener,
    grid::{Dimensions, Scroll as GridScroll},
    index::{Column, Line, Point},
    term::{Term, TermMode},
    vi_mode::ViMotion,
};

/// Motions supported while navigating a terminal in normal mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalNormalMotion {
    Up,
    Down,
    Left,
    Right,
    WordForward,
    WordBackward,
    WordEnd,
    WordEndBackward,
    BigWordForward,
    BigWordBackward,
    BigWordEnd,
    BigWordEndBackward,
    ParagraphForward,
    ParagraphBackward,
    LineStart,
    LineFirstNonBlank,
    LineEnd,
    ScreenTop,
    ScreenMiddle,
    ScreenBottom,
    FirstLine,
    LastLine,
    MatchPair,
}

/// Enable Alacritty vi mode for grid-aware cursor navigation.
pub fn begin_terminal_normal_mode<T: EventListener>(term: &mut Term<T>) {
    if !term.mode().contains(TermMode::VI) {
        term.toggle_vi_mode();
    }
}

/// Leave vi mode and jump the viewport back to the live prompt.
pub fn end_terminal_normal_mode<T: EventListener>(term: &mut Term<T>) {
    if term.mode().contains(TermMode::VI) {
        term.toggle_vi_mode();
    }
    term.scroll_display(GridScroll::Bottom);
}

/// Apply one normal-mode motion, then enforce scrolloff against the viewport.
pub fn apply_terminal_normal_motion<T: EventListener>(
    term: &mut Term<T>,
    motion: TerminalNormalMotion,
    count: Option<usize>,
    scrolloff: usize,
) -> bool {
    begin_terminal_normal_mode(term);
    let before = (term.vi_mode_cursor.point, term.grid().display_offset());
    let repeat = count.unwrap_or(1).max(1);
    match motion {
        TerminalNormalMotion::FirstLine => {
            let line = match count {
                Some(number) => absolute_line_for_number(term, number),
                None => term.topmost_line(),
            };
            term.vi_goto_point(Point::new(line, Column(0)));
        }
        TerminalNormalMotion::LastLine => {
            let line = match count {
                Some(number) => absolute_line_for_number(term, number),
                None => last_occupied_line(term).unwrap_or_else(|| term.bottommost_line()),
            };
            term.vi_goto_point(Point::new(line, Column(0)));
        }
        TerminalNormalMotion::WordEndBackward | TerminalNormalMotion::BigWordEndBackward => {
            for _ in 0..repeat {
                term.vi_motion(ViMotion::WordLeftEnd);
            }
        }
        other => {
            let Some(vi_motion) = vi_motion_for(other) else {
                return false;
            };
            for _ in 0..repeat {
                term.vi_motion(vi_motion);
            }
        }
    }
    apply_terminal_scrolloff(term, scrolloff);
    let after = (term.vi_mode_cursor.point, term.grid().display_offset());
    before != after
}

/// Page/half-page style scroll that keeps the vi cursor fixed on screen.
///
/// `lines` uses Alacritty's scroll delta sign: positive looks further up into
/// history, negative moves toward the live prompt.
pub fn scroll_terminal_normal_view<T: EventListener>(
    term: &mut Term<T>,
    lines: i32,
    scrolloff: usize,
) -> bool {
    begin_terminal_normal_mode(term);
    let before = (term.vi_mode_cursor.point, term.grid().display_offset());
    term.scroll_display(GridScroll::Delta(lines));
    term.vi_mode_cursor = term.vi_mode_cursor.scroll(term, lines);
    apply_terminal_scrolloff(term, scrolloff);
    let after = (term.vi_mode_cursor.point, term.grid().display_offset());
    before != after
}

/// Map the vi cursor into the current viewport (row, col).
pub fn terminal_normal_cursor_viewport<T: EventListener>(term: &Term<T>) -> Option<(usize, usize)> {
    let display_offset = term.grid().display_offset() as i32;
    let point = term.vi_mode_cursor.point;
    let row = point.line.0 + display_offset;
    if row < 0 || row as usize >= term.screen_lines() {
        return None;
    }
    Some((row as usize, point.column.0))
}

fn vi_motion_for(motion: TerminalNormalMotion) -> Option<ViMotion> {
    Some(match motion {
        TerminalNormalMotion::Up => ViMotion::Up,
        TerminalNormalMotion::Down => ViMotion::Down,
        TerminalNormalMotion::Left => ViMotion::Left,
        TerminalNormalMotion::Right => ViMotion::Right,
        TerminalNormalMotion::WordForward | TerminalNormalMotion::BigWordForward => {
            ViMotion::WordRight
        }
        TerminalNormalMotion::WordBackward | TerminalNormalMotion::BigWordBackward => {
            ViMotion::WordLeft
        }
        TerminalNormalMotion::WordEnd | TerminalNormalMotion::BigWordEnd => ViMotion::WordRightEnd,
        TerminalNormalMotion::ParagraphForward => ViMotion::ParagraphDown,
        TerminalNormalMotion::ParagraphBackward => ViMotion::ParagraphUp,
        TerminalNormalMotion::LineStart => ViMotion::First,
        TerminalNormalMotion::LineFirstNonBlank => ViMotion::FirstOccupied,
        TerminalNormalMotion::LineEnd => ViMotion::Last,
        TerminalNormalMotion::ScreenTop => ViMotion::High,
        TerminalNormalMotion::ScreenMiddle => ViMotion::Middle,
        TerminalNormalMotion::ScreenBottom => ViMotion::Low,
        TerminalNormalMotion::MatchPair => ViMotion::Bracket,
        TerminalNormalMotion::FirstLine
        | TerminalNormalMotion::LastLine
        | TerminalNormalMotion::WordEndBackward
        | TerminalNormalMotion::BigWordEndBackward => return None,
    })
}

fn absolute_line_for_number<T: EventListener>(term: &Term<T>, number: usize) -> Line {
    let top = term.topmost_line().0;
    let bottom = term.bottommost_line().0;
    let target = top + number.saturating_sub(1) as i32;
    Line(target.clamp(top, bottom))
}

fn last_occupied_line<T: EventListener>(term: &Term<T>) -> Option<Line> {
    let top = term.topmost_line().0;
    let bottom = term.bottommost_line().0;
    (top..=bottom)
        .rev()
        .map(Line)
        .find(|line| !term.grid()[*line].is_clear())
}

fn apply_terminal_scrolloff<T: EventListener>(term: &mut Term<T>, scrolloff: usize) {
    let screen_lines = term.screen_lines().max(1) as i32;
    let max_off = scrolloff.min(screen_lines.saturating_sub(1) as usize / 2) as i32;
    if max_off == 0 {
        return;
    }
    let display_offset = term.grid().display_offset() as i32;
    let history = term.history_size() as i32;
    let point = term.vi_mode_cursor.point;
    let viewport_row = point.line.0 + display_offset;
    if viewport_row < max_off {
        let desired = (display_offset + (max_off - viewport_row)).min(history);
        let delta = desired - display_offset;
        if delta != 0 {
            term.scroll_display(GridScroll::Delta(delta));
        }
    } else if viewport_row > screen_lines - 1 - max_off {
        let overflow = viewport_row - (screen_lines - 1 - max_off);
        let desired = (display_offset - overflow).max(0);
        let delta = desired - display_offset;
        if delta != 0 {
            term.scroll_display(GridScroll::Delta(delta));
        }
    }
}
