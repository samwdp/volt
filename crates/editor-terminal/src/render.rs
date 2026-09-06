use std::collections::HashMap;

use alacritty_terminal::{
    term::{cell::Flags, color::Colors as TerminalColors},
    tty::{self, Options as TtyOptions, Shell as TtyShell},
    vte::ansi::{Color as TerminalColor, NamedColor, Rgb},
};

use crate::session::*;

/// Styled text run for a visible terminal row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalRenderRun {
    pub(crate) col: u16,
    pub(crate) width_cells: u16,
    pub(crate) text: String,
    pub(crate) foreground: Rgb,
    pub(crate) background: Option<Rgb>,
    pub(crate) underline: Option<Rgb>,
}

impl TerminalRenderRun {
    /// Creates a styled terminal render run.
    pub fn new(
        col: u16,
        width_cells: u16,
        text: impl Into<String>,
        foreground: Rgb,
        background: Option<Rgb>,
        underline: Option<Rgb>,
    ) -> Self {
        Self {
            col,
            width_cells: width_cells.max(1),
            text: text.into(),
            foreground,
            background,
            underline,
        }
    }

    /// Returns the starting column for this run.
    pub const fn col(&self) -> u16 {
        self.col
    }

    /// Returns the width of this run in terminal cells.
    pub const fn width_cells(&self) -> u16 {
        self.width_cells
    }

    /// Returns the run text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the run foreground color.
    pub const fn foreground(&self) -> Rgb {
        self.foreground
    }

    /// Returns the run background color, when the cell background is visible.
    pub const fn background(&self) -> Option<Rgb> {
        self.background
    }

    /// Returns the underline color, when the run is underlined.
    pub const fn underline(&self) -> Option<Rgb> {
        self.underline
    }
}

/// One visible terminal viewport row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerminalRenderLine {
    pub(crate) runs: Vec<TerminalRenderRun>,
}

impl TerminalRenderLine {
    /// Creates a terminal render line from styled runs.
    pub fn new(runs: Vec<TerminalRenderRun>) -> Self {
        Self { runs }
    }

    /// Returns the styled runs on this row.
    pub fn runs(&self) -> &[TerminalRenderRun] {
        &self.runs
    }
}

/// Renderable snapshot of the visible terminal viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalRenderSnapshot {
    pub(crate) rows: u16,
    pub(crate) cols: u16,
    pub(crate) lines: Vec<TerminalRenderLine>,
    pub(crate) cursor: Option<TerminalCursorSnapshot>,
    pub(crate) exit_code: Option<i32>,
}

impl TerminalRenderSnapshot {
    /// Creates a terminal render snapshot for the visible viewport.
    pub fn new(
        rows: u16,
        cols: u16,
        lines: Vec<TerminalRenderLine>,
        cursor: Option<TerminalCursorSnapshot>,
        exit_code: Option<i32>,
    ) -> Self {
        Self {
            rows: rows.max(1),
            cols: cols.max(1),
            lines,
            cursor,
            exit_code,
        }
    }

    /// Returns the viewport height in rows.
    pub const fn rows(&self) -> u16 {
        self.rows
    }

    /// Returns the viewport width in columns.
    pub const fn cols(&self) -> u16 {
        self.cols
    }

    /// Returns the visible terminal rows.
    pub fn lines(&self) -> &[TerminalRenderLine] {
        &self.lines
    }

    /// Returns the visible cursor, if it is in the viewport.
    pub fn cursor(&self) -> Option<&TerminalCursorSnapshot> {
        self.cursor.as_ref()
    }

    /// Returns the terminal exit code, if the child process has exited.
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }
}

pub(crate) fn push_terminal_render_run(
    runs: &mut Vec<TerminalRenderRun>,
    col: u16,
    width_cells: u16,
    text: String,
    foreground: Rgb,
    background: Option<Rgb>,
    underline: Option<Rgb>,
) {
    if let Some(last) = runs.last_mut()
        && last.col.saturating_add(last.width_cells) == col
        && last.foreground == foreground
        && last.background == background
        && last.underline == underline
    {
        last.width_cells = last.width_cells.saturating_add(width_cells);
        last.text.push_str(&text);
        return;
    }
    runs.push(TerminalRenderRun {
        col,
        width_cells,
        text,
        foreground,
        background,
        underline,
    });
}

pub(crate) fn resolve_terminal_foreground(
    colors: &TerminalColors,
    color: TerminalColor,
    flags: Flags,
) -> Rgb {
    match color {
        TerminalColor::Spec(rgb) => {
            if flags.intersects(Flags::DIM) && !flags.intersects(Flags::BOLD) {
                rgb * DIM_FACTOR
            } else {
                rgb
            }
        }
        TerminalColor::Named(named) => {
            let resolved = if flags.intersects(Flags::DIM) && !flags.intersects(Flags::BOLD) {
                named.to_dim()
            } else if flags.intersects(Flags::BOLD) {
                named.to_bright()
            } else {
                named
            };
            resolve_terminal_named_color(colors, resolved)
        }
        TerminalColor::Indexed(index) => {
            let resolved_index = if flags.intersects(Flags::DIM) && !flags.intersects(Flags::BOLD) {
                match index {
                    0..=7 => NamedColor::DimBlack as usize + index as usize,
                    8..=15 => index as usize - 8,
                    _ => index as usize,
                }
            } else if flags.intersects(Flags::BOLD) && index <= 7 {
                index as usize + 8
            } else {
                index as usize
            };
            resolve_terminal_index_color(colors, resolved_index)
        }
    }
}

pub(crate) fn resolve_terminal_background(colors: &TerminalColors, color: TerminalColor) -> Rgb {
    match color {
        TerminalColor::Spec(rgb) => rgb,
        TerminalColor::Named(named) => resolve_terminal_named_color(colors, named),
        TerminalColor::Indexed(index) => resolve_terminal_index_color(colors, index as usize),
    }
}

pub(crate) fn resolve_terminal_plain_color(colors: &TerminalColors, color: TerminalColor) -> Rgb {
    match color {
        TerminalColor::Spec(rgb) => rgb,
        TerminalColor::Named(named) => resolve_terminal_named_color(colors, named),
        TerminalColor::Indexed(index) => resolve_terminal_index_color(colors, index as usize),
    }
}

pub(crate) fn resolve_terminal_named_color(colors: &TerminalColors, named: NamedColor) -> Rgb {
    colors[named].unwrap_or_else(|| default_terminal_named_color(named))
}

pub(crate) fn resolve_terminal_index_color(colors: &TerminalColors, index: usize) -> Rgb {
    colors[index].unwrap_or_else(|| default_terminal_index_color(index))
}

pub(crate) fn push_snapshot_line(lines: &mut Vec<String>, line: &str) {
    lines.push(line.trim_end_matches(' ').to_owned());
}

pub(crate) fn terminal_tty_options(config: &LiveTerminalConfig) -> TtyOptions {
    let mut env = std::env::vars().collect::<HashMap<_, _>>();
    env.remove("SHLVL");
    env.insert("COLORTERM".to_owned(), "truecolor".to_owned());
    env.insert("TERM".to_owned(), "xterm-256color".to_owned());
    env.insert("TERM_PROGRAM".to_owned(), "volt".to_owned());
    env.insert(
        "TERM_PROGRAM_VERSION".to_owned(),
        env!("CARGO_PKG_VERSION").to_owned(),
    );

    TtyOptions {
        // PTY-backed terminals must spawn the real shell directly; wrapping them in the
        // supervisor binary on Windows breaks ConPTY embedding and opens a separate window.
        shell: Some(TtyShell::new(config.program.clone(), config.args.clone())),
        working_directory: config.cwd.clone(),
        drain_on_exit: true,
        env,
        #[cfg(windows)]
        escape_args: true,
    }
}

#[cfg(windows)]
pub(crate) fn tty_process_id(_pty: &tty::Pty) -> Option<u32> {
    None
}

#[cfg(not(windows))]
pub(crate) fn tty_process_id(pty: &tty::Pty) -> Option<u32> {
    Some(pty.child().id())
}
