#![doc = r#"Terminal transcript sessions and editor-facing command execution surfaces."#]

use std::{
    borrow::Cow,
    collections::HashMap,
    error::Error,
    fmt,
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
};

use alacritty_terminal::{
    event::{Event as AlacrittyEvent, EventListener, WindowSize},
    event_loop::{EventLoop, EventLoopSendError, EventLoopSender, Msg},
    grid::{Dimensions, Scroll as GridScroll},
    sync::FairMutex,
    term::{
        Config as AlacrittyConfig, Term,
        cell::{Cell as TerminalCell, Flags},
        color::Colors as TerminalColors,
        point_to_viewport,
    },
    tty::{self, Options as TtyOptions, Shell as TtyShell},
    vte::ansi::{Color as TerminalColor, CursorShape, NamedColor, Rgb},
};
use editor_jobs::{JobError, JobManager, JobResult, JobSpec};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Terminal transcript sessions and editor-facing command execution surfaces.";

/// RGB color used by terminal render snapshots.
pub type TerminalRgb = Rgb;

const DEFAULT_TERMINAL_SCROLLBACK: usize = 10_000;
const DEFAULT_CELL_SIZE: u16 = 1;
const DIM_FACTOR: f32 = 0.66;

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Distinguishes stdout and stderr transcript lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalStream {
    /// Normal stdout output.
    Stdout,
    /// Error stream output.
    Stderr,
}

/// One line in a terminal transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLine {
    stream: TerminalStream,
    text: String,
}

impl TerminalLine {
    fn new(stream: TerminalStream, text: impl Into<String>) -> Self {
        Self {
            stream,
            text: text.into(),
        }
    }

    /// Returns the originating stream.
    pub const fn stream(&self) -> TerminalStream {
        self.stream
    }

    /// Returns the line text without a trailing newline.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Collected transcript for a terminal session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalTranscript {
    lines: Vec<TerminalLine>,
    exit_code: Option<i32>,
}

impl TerminalTranscript {
    /// Returns all transcript lines.
    pub fn lines(&self) -> &[TerminalLine] {
        &self.lines
    }

    /// Returns the number of transcript lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns the exit code, if any.
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Reports whether the session exited successfully.
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Materialized terminal session suitable for editor buffers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSession {
    title: String,
    command_label: String,
    transcript: TerminalTranscript,
}

impl TerminalSession {
    /// Runs a terminal command to completion and captures its transcript.
    pub fn run(
        jobs: &mut JobManager,
        title: impl Into<String>,
        spec: JobSpec,
    ) -> Result<Self, JobError> {
        let handle = jobs.spawn(spec.with_kind(editor_jobs::JobKind::Terminal))?;
        let result = handle.wait()?;
        Ok(Self::from_job_result(title, result))
    }

    /// Returns the terminal title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the executed command label.
    pub fn command_label(&self) -> &str {
        &self.command_label
    }

    /// Returns the collected transcript.
    pub fn transcript(&self) -> &TerminalTranscript {
        &self.transcript
    }

    fn from_job_result(title: impl Into<String>, result: JobResult) -> Self {
        let mut lines = Vec::new();
        append_lines(&mut lines, TerminalStream::Stdout, result.stdout());
        append_lines(&mut lines, TerminalStream::Stderr, result.stderr());

        Self {
            title: title.into(),
            command_label: result.spec().label().to_owned(),
            transcript: TerminalTranscript {
                lines,
                exit_code: result.exit_code(),
            },
        }
    }
}

/// Launch configuration for an interactive PTY-backed terminal session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTerminalConfig {
    title: String,
    program: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    rows: u16,
    cols: u16,
    scrollback: usize,
}

impl LiveTerminalConfig {
    /// Creates a PTY-backed terminal configuration.
    pub fn new(
        title: impl Into<String>,
        program: impl Into<String>,
        args: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            title: title.into(),
            program: program.into(),
            args: args.into_iter().collect(),
            cwd: None,
            rows: 24,
            cols: 80,
            scrollback: DEFAULT_TERMINAL_SCROLLBACK,
        }
    }

    /// Sets the working directory used for the spawned shell.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Sets the initial terminal size.
    pub fn with_size(mut self, rows: u16, cols: u16) -> Self {
        self.rows = rows.max(1);
        self.cols = cols.max(1);
        self
    }

    /// Sets the scrollback length retained by the in-memory parser.
    pub fn with_scrollback(mut self, scrollback: usize) -> Self {
        self.scrollback = scrollback.max(1);
        self
    }

    /// Returns the configured title.
    pub fn title(&self) -> &str {
        &self.title
    }
}

/// Special keys that can be forwarded into a live terminal session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalKey {
    Enter,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    CtrlC,
}

/// Cursor shape exposed by a live terminal snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCursorShape {
    Hidden,
    Block,
    Underline,
    Beam,
    HollowBlock,
}

/// Cursor state for the visible terminal viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCursorSnapshot {
    row: u16,
    col: u16,
    width_cells: u16,
    shape: TerminalCursorShape,
    text: String,
}

impl TerminalCursorSnapshot {
    /// Creates a visible terminal cursor snapshot.
    pub fn new(
        row: u16,
        col: u16,
        width_cells: u16,
        shape: TerminalCursorShape,
        text: impl Into<String>,
    ) -> Self {
        Self {
            row,
            col,
            width_cells: width_cells.max(1),
            shape,
            text: text.into(),
        }
    }

    /// Returns the cursor row within the visible viewport.
    pub const fn row(&self) -> u16 {
        self.row
    }

    /// Returns the cursor column within the visible viewport.
    pub const fn col(&self) -> u16 {
        self.col
    }

    /// Returns the cursor width in terminal cells.
    pub const fn width_cells(&self) -> u16 {
        self.width_cells
    }

    /// Returns the renderable cursor shape.
    pub const fn shape(&self) -> TerminalCursorShape {
        self.shape
    }

    /// Returns the text currently under the cursor, if any.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Styled text run for a visible terminal row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalRenderRun {
    col: u16,
    width_cells: u16,
    text: String,
    foreground: Rgb,
    background: Option<Rgb>,
    underline: Option<Rgb>,
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
    runs: Vec<TerminalRenderRun>,
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
    rows: u16,
    cols: u16,
    lines: Vec<TerminalRenderLine>,
    cursor: Option<TerminalCursorSnapshot>,
    exit_code: Option<i32>,
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

/// Viewport scroll commands for live terminals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalViewportScroll {
    LineDelta(i32),
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,
    Top,
    Bottom,
}

/// Plain-text snapshot of the current terminal contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSnapshot {
    lines: Vec<String>,
    exit_code: Option<i32>,
}

impl TerminalSnapshot {
    /// Returns the rendered terminal lines.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Returns the exit code, if the terminal process has exited.
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }
}

/// Errors surfaced by the live PTY-backed terminal session.
#[derive(Debug)]
pub enum LiveTerminalError {
    Io(std::io::Error),
    Pty(String),
}

impl fmt::Display for LiveTerminalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Pty(error) => formatter.write_str(error),
        }
    }
}

impl Error for LiveTerminalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Pty(_) => None,
        }
    }
}

impl From<std::io::Error> for LiveTerminalError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<EventLoopSendError> for LiveTerminalError {
    fn from(error: EventLoopSendError) -> Self {
        Self::Pty(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalDimensions {
    rows: u16,
    cols: u16,
}

impl TerminalDimensions {
    fn new(rows: u16, cols: u16) -> Self {
        Self {
            rows: rows.max(1),
            cols: cols.max(1),
        }
    }
}

impl Dimensions for TerminalDimensions {
    fn total_lines(&self) -> usize {
        self.rows as usize
    }

    fn screen_lines(&self) -> usize {
        self.rows as usize
    }

    fn columns(&self) -> usize {
        self.cols as usize
    }
}

impl From<TerminalDimensions> for WindowSize {
    fn from(value: TerminalDimensions) -> Self {
        Self {
            num_lines: value.rows,
            num_cols: value.cols,
            cell_width: DEFAULT_CELL_SIZE,
            cell_height: DEFAULT_CELL_SIZE,
        }
    }
}

#[derive(Clone)]
struct QueuedEventListener {
    tx: Sender<AlacrittyEvent>,
}

impl EventListener for QueuedEventListener {
    fn send_event(&self, event: AlacrittyEvent) {
        let _ = self.tx.send(event);
    }
}

/// Long-lived Alacritty-backed terminal session suitable for live editor buffers.
pub struct LiveTerminalSession {
    configured_title: String,
    title: String,
    term: Arc<FairMutex<Term<QueuedEventListener>>>,
    sender: EventLoopSender,
    event_rx: Receiver<AlacrittyEvent>,
    rows: u16,
    cols: u16,
    exit_code: Option<i32>,
    process_id: Option<u32>,
}

impl LiveTerminalSession {
    /// Spawns a new PTY-backed terminal session.
    pub fn spawn(config: LiveTerminalConfig) -> Result<Self, LiveTerminalError> {
        let dimensions = TerminalDimensions::new(config.rows, config.cols);
        let tty_options = terminal_tty_options(&config);
        let pty = tty::new(&tty_options, dimensions.into(), 0)?;
        let process_id = tty_process_id(&pty);
        let (event_tx, event_rx) = mpsc::channel();
        let listener = QueuedEventListener { tx: event_tx };
        let term = Arc::new(FairMutex::new(Term::new(
            AlacrittyConfig {
                scrolling_history: config.scrollback,
                ..AlacrittyConfig::default()
            },
            &dimensions,
            listener.clone(),
        )));
        let event_loop = EventLoop::new(term.clone(), listener, pty, true, false)?;
        let sender = event_loop.channel();
        let _ = event_loop.spawn();

        Ok(Self {
            configured_title: config.title.clone(),
            title: config.title,
            term,
            sender,
            event_rx,
            rows: dimensions.rows,
            cols: dimensions.cols,
            exit_code: None,
            process_id,
        })
    }

    /// Returns the configured terminal title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the child process id, if available.
    pub fn process_id(&self) -> Option<u32> {
        self.process_id
    }

    /// Returns whether the process has exited.
    pub fn has_exited(&self) -> bool {
        self.exit_code.is_some()
    }

    /// Drains any pending terminal output and updates the cached exit status.
    pub fn poll(&mut self) -> Result<bool, LiveTerminalError> {
        let mut changed = false;
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => {
                    changed |= self.process_event(event)?;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if self.exit_code.is_none() {
                        self.exit_code = Some(-1);
                        changed = true;
                    }
                    break;
                }
            }
        }
        Ok(changed)
    }

    /// Resizes the underlying PTY and terminal grid.
    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<bool, LiveTerminalError> {
        let dimensions = TerminalDimensions::new(rows, cols);
        if self.rows == dimensions.rows && self.cols == dimensions.cols {
            return Ok(false);
        }

        self.term.lock().resize(dimensions);
        self.sender.send(Msg::Resize(dimensions.into()))?;
        self.rows = dimensions.rows;
        self.cols = dimensions.cols;
        Ok(true)
    }

    /// Writes raw text input into the terminal.
    pub fn write_text(&mut self, text: &str) -> Result<(), LiveTerminalError> {
        if text.is_empty() {
            return Ok(());
        }
        self.sender
            .send(Msg::Input(Cow::Owned(text.as_bytes().to_vec())))?;
        Ok(())
    }

    /// Writes a special terminal key into the PTY stream.
    pub fn write_key(&mut self, key: TerminalKey) -> Result<(), LiveTerminalError> {
        self.sender
            .send(Msg::Input(Cow::Borrowed(terminal_key_bytes(key))))?;
        Ok(())
    }

    /// Returns a plain-text snapshot of the current terminal contents.
    pub fn snapshot(&self) -> TerminalSnapshot {
        let term = self.term.lock();
        TerminalSnapshot {
            lines: terminal_snapshot_lines(&term),
            exit_code: self.exit_code,
        }
    }

    /// Returns a styled render snapshot for the visible terminal viewport.
    pub fn render_snapshot(&self) -> TerminalRenderSnapshot {
        let term = self.term.lock();
        terminal_render_snapshot(&term, self.rows, self.cols, self.exit_code)
    }

    /// Scrolls the visible terminal viewport.
    pub fn scroll_viewport(&mut self, scroll: TerminalViewportScroll) -> bool {
        let grid_scroll = match scroll {
            TerminalViewportScroll::LineDelta(lines) => GridScroll::Delta(lines),
            TerminalViewportScroll::PageUp => GridScroll::PageUp,
            TerminalViewportScroll::PageDown => GridScroll::PageDown,
            TerminalViewportScroll::HalfPageUp => {
                GridScroll::Delta((self.rows.max(1) / 2).max(1) as i32)
            }
            TerminalViewportScroll::HalfPageDown => {
                GridScroll::Delta(-((self.rows.max(1) / 2).max(1) as i32))
            }
            TerminalViewportScroll::Top => GridScroll::Top,
            TerminalViewportScroll::Bottom => GridScroll::Bottom,
        };
        let mut term = self.term.lock();
        let previous_offset = term.grid().display_offset();
        term.scroll_display(grid_scroll);
        previous_offset != term.grid().display_offset()
    }

    /// Terminates the child process if it is still running.
    pub fn kill(&mut self) -> Result<(), LiveTerminalError> {
        if self.exit_code.is_some() {
            return Ok(());
        }
        match self.sender.send(Msg::Shutdown) {
            Ok(()) | Err(EventLoopSendError::Send(_)) => {
                self.exit_code = Some(-1);
                Ok(())
            }
            Err(error) => Err(LiveTerminalError::from(error)),
        }
    }

    fn process_event(&mut self, event: AlacrittyEvent) -> Result<bool, LiveTerminalError> {
        match event {
            AlacrittyEvent::Wakeup => Ok(true),
            AlacrittyEvent::Title(title) => {
                self.title = title;
                Ok(true)
            }
            AlacrittyEvent::ResetTitle => {
                self.title.clone_from(&self.configured_title);
                Ok(true)
            }
            AlacrittyEvent::PtyWrite(text) => {
                self.sender
                    .send(Msg::Input(Cow::Owned(text.into_bytes())))?;
                Ok(false)
            }
            AlacrittyEvent::TextAreaSizeRequest(format) => {
                self.sender.send(Msg::Input(Cow::Owned(
                    format(self.window_size()).into_bytes(),
                )))?;
                Ok(false)
            }
            AlacrittyEvent::ColorRequest(index, format) => {
                let color = self.term.lock().colors()[index].unwrap_or_default();
                self.sender
                    .send(Msg::Input(Cow::Owned(format(color).into_bytes())))?;
                Ok(false)
            }
            AlacrittyEvent::ClipboardStore(_, _) => Ok(false),
            AlacrittyEvent::ClipboardLoad(_, format) => {
                self.sender
                    .send(Msg::Input(Cow::Owned(format("").into_bytes())))?;
                Ok(false)
            }
            AlacrittyEvent::CursorBlinkingChange => Ok(false),
            AlacrittyEvent::MouseCursorDirty => Ok(false),
            AlacrittyEvent::Bell => Ok(false),
            AlacrittyEvent::Exit => {
                if self.exit_code.is_none() {
                    self.exit_code = Some(-1);
                    return Ok(true);
                }
                Ok(false)
            }
            AlacrittyEvent::ChildExit(code) => {
                let changed = self.exit_code != Some(code);
                self.exit_code = Some(code);
                Ok(changed)
            }
        }
    }

    fn window_size(&self) -> WindowSize {
        TerminalDimensions::new(self.rows, self.cols).into()
    }
}

impl Drop for LiveTerminalSession {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

fn append_lines(lines: &mut Vec<TerminalLine>, stream: TerminalStream, text: &str) {
    for line in text.lines() {
        lines.push(TerminalLine::new(stream, line));
    }
}

fn terminal_key_bytes(key: TerminalKey) -> &'static [u8] {
    match key {
        TerminalKey::Enter => b"\r",
        TerminalKey::Tab => b"\t",
        TerminalKey::BackTab => b"\x1b[Z",
        TerminalKey::Backspace => b"\x7f",
        TerminalKey::Delete => b"\x1b[3~",
        TerminalKey::Left => b"\x1b[D",
        TerminalKey::Right => b"\x1b[C",
        TerminalKey::Up => b"\x1b[A",
        TerminalKey::Down => b"\x1b[B",
        TerminalKey::Home => b"\x1b[H",
        TerminalKey::End => b"\x1b[F",
        TerminalKey::PageUp => b"\x1b[5~",
        TerminalKey::PageDown => b"\x1b[6~",
        TerminalKey::CtrlC => b"\x03",
    }
}

fn terminal_snapshot_lines<T: EventListener>(term: &Term<T>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = None;
    let mut current_text = String::new();
    for indexed in term.renderable_content().display_iter {
        let line = indexed.point.line.0;
        if current_line != Some(line) {
            if current_line.is_some() {
                push_snapshot_line(&mut lines, &current_text);
                current_text.clear();
            }
            current_line = Some(line);
        }

        let cell = indexed.cell;
        if cell
            .flags
            .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
        {
            continue;
        }

        let character = if cell.flags.contains(Flags::HIDDEN) {
            ' '
        } else {
            cell.c
        };
        current_text.push(character);
        if let Some(zerowidth) = cell.zerowidth() {
            current_text.extend(zerowidth.iter().copied());
        }
    }

    if current_line.is_some() {
        push_snapshot_line(&mut lines, &current_text);
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn terminal_render_snapshot<T: EventListener>(
    term: &Term<T>,
    rows: u16,
    cols: u16,
    exit_code: Option<i32>,
) -> TerminalRenderSnapshot {
    let mut lines = vec![TerminalRenderLine::default(); rows as usize];
    let content = term.renderable_content();
    let cursor_point = content.cursor.point;
    let cursor_shape = map_terminal_cursor_shape(content.cursor.shape);
    let display_offset = content.display_offset;
    let colors = content.colors;
    let mut cursor = None;

    for indexed in content.display_iter {
        let cell = indexed.cell;
        if cell
            .flags
            .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
        {
            continue;
        }
        let Some(viewport_point) = point_to_viewport(display_offset, indexed.point) else {
            continue;
        };
        let row = viewport_point.line;
        if row >= lines.len() {
            continue;
        }

        let col = viewport_point.column.0 as u16;
        let width_cells = if cell.flags.contains(Flags::WIDE_CHAR) {
            2
        } else {
            1
        };
        let text = terminal_cell_text(cell);
        if indexed.point == cursor_point && cursor_shape != TerminalCursorShape::Hidden {
            cursor = Some(TerminalCursorSnapshot {
                row: row as u16,
                col,
                width_cells,
                shape: cursor_shape,
                text: text.clone(),
            });
        }

        let mut foreground = resolve_terminal_foreground(colors, cell.fg, cell.flags);
        let mut background = resolve_terminal_background(colors, cell.bg);
        let mut background_visible =
            !matches!(cell.bg, TerminalColor::Named(NamedColor::Background));
        if cell.flags.contains(Flags::INVERSE) {
            std::mem::swap(&mut foreground, &mut background);
            background_visible = true;
        }
        let underline = terminal_underline_color(colors, cell, foreground);
        let character_visible = text.chars().any(|character| character != ' ');
        if !(character_visible || background_visible || underline.is_some()) {
            continue;
        }
        push_terminal_render_run(
            &mut lines[row].runs,
            col,
            width_cells,
            text,
            foreground,
            background_visible.then_some(background),
            underline,
        );
    }

    TerminalRenderSnapshot {
        rows,
        cols,
        lines,
        cursor,
        exit_code,
    }
}

fn map_terminal_cursor_shape(shape: CursorShape) -> TerminalCursorShape {
    match shape {
        CursorShape::Hidden => TerminalCursorShape::Hidden,
        CursorShape::Block => TerminalCursorShape::Block,
        CursorShape::Underline => TerminalCursorShape::Underline,
        CursorShape::Beam => TerminalCursorShape::Beam,
        CursorShape::HollowBlock => TerminalCursorShape::HollowBlock,
    }
}

fn terminal_cell_text(cell: &TerminalCell) -> String {
    let mut text = String::new();
    text.push(if cell.flags.contains(Flags::HIDDEN) {
        ' '
    } else {
        cell.c
    });
    if let Some(zerowidth) = cell.zerowidth() {
        text.extend(zerowidth.iter().copied());
    }
    text
}

fn terminal_underline_color(
    colors: &TerminalColors,
    cell: &TerminalCell,
    foreground: Rgb,
) -> Option<Rgb> {
    if !cell.flags.intersects(Flags::ALL_UNDERLINES) && cell.underline_color().is_none() {
        return None;
    }
    Some(
        cell.underline_color()
            .map(|color| resolve_terminal_plain_color(colors, color))
            .unwrap_or(foreground),
    )
}

fn push_terminal_render_run(
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

fn resolve_terminal_foreground(colors: &TerminalColors, color: TerminalColor, flags: Flags) -> Rgb {
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

fn resolve_terminal_background(colors: &TerminalColors, color: TerminalColor) -> Rgb {
    match color {
        TerminalColor::Spec(rgb) => rgb,
        TerminalColor::Named(named) => resolve_terminal_named_color(colors, named),
        TerminalColor::Indexed(index) => resolve_terminal_index_color(colors, index as usize),
    }
}

fn resolve_terminal_plain_color(colors: &TerminalColors, color: TerminalColor) -> Rgb {
    match color {
        TerminalColor::Spec(rgb) => rgb,
        TerminalColor::Named(named) => resolve_terminal_named_color(colors, named),
        TerminalColor::Indexed(index) => resolve_terminal_index_color(colors, index as usize),
    }
}

fn resolve_terminal_named_color(colors: &TerminalColors, named: NamedColor) -> Rgb {
    colors[named].unwrap_or_else(|| default_terminal_named_color(named))
}

fn resolve_terminal_index_color(colors: &TerminalColors, index: usize) -> Rgb {
    colors[index].unwrap_or_else(|| default_terminal_index_color(index))
}

fn default_terminal_index_color(index: usize) -> Rgb {
    match index {
        0 => default_terminal_named_color(NamedColor::Black),
        1 => default_terminal_named_color(NamedColor::Red),
        2 => default_terminal_named_color(NamedColor::Green),
        3 => default_terminal_named_color(NamedColor::Yellow),
        4 => default_terminal_named_color(NamedColor::Blue),
        5 => default_terminal_named_color(NamedColor::Magenta),
        6 => default_terminal_named_color(NamedColor::Cyan),
        7 => default_terminal_named_color(NamedColor::White),
        8 => default_terminal_named_color(NamedColor::BrightBlack),
        9 => default_terminal_named_color(NamedColor::BrightRed),
        10 => default_terminal_named_color(NamedColor::BrightGreen),
        11 => default_terminal_named_color(NamedColor::BrightYellow),
        12 => default_terminal_named_color(NamedColor::BrightBlue),
        13 => default_terminal_named_color(NamedColor::BrightMagenta),
        14 => default_terminal_named_color(NamedColor::BrightCyan),
        15 => default_terminal_named_color(NamedColor::BrightWhite),
        16..=231 => {
            let index = index - 16;
            let blue = index % 6;
            let green = (index / 6) % 6;
            let red = index / 36;
            Rgb {
                r: cube_color_component(red),
                g: cube_color_component(green),
                b: cube_color_component(blue),
            }
        }
        232..=255 => {
            let value = ((index - 232) * 10 + 8) as u8;
            Rgb {
                r: value,
                g: value,
                b: value,
            }
        }
        256 => default_terminal_named_color(NamedColor::Foreground),
        257 => default_terminal_named_color(NamedColor::Background),
        258 => default_terminal_named_color(NamedColor::Cursor),
        259 => default_terminal_named_color(NamedColor::DimBlack),
        260 => default_terminal_named_color(NamedColor::DimRed),
        261 => default_terminal_named_color(NamedColor::DimGreen),
        262 => default_terminal_named_color(NamedColor::DimYellow),
        263 => default_terminal_named_color(NamedColor::DimBlue),
        264 => default_terminal_named_color(NamedColor::DimMagenta),
        265 => default_terminal_named_color(NamedColor::DimCyan),
        266 => default_terminal_named_color(NamedColor::DimWhite),
        267 => default_terminal_named_color(NamedColor::BrightForeground),
        268 => default_terminal_named_color(NamedColor::DimForeground),
        _ => default_terminal_named_color(NamedColor::Foreground),
    }
}

fn cube_color_component(index: usize) -> u8 {
    if index == 0 {
        0
    } else {
        (index as u8).saturating_mul(40).saturating_add(55)
    }
}

fn default_terminal_named_color(named: NamedColor) -> Rgb {
    match named {
        NamedColor::Black => Rgb {
            r: 12,
            g: 12,
            b: 12,
        },
        NamedColor::Red => Rgb {
            r: 205,
            g: 49,
            b: 49,
        },
        NamedColor::Green => Rgb {
            r: 13,
            g: 188,
            b: 121,
        },
        NamedColor::Yellow => Rgb {
            r: 229,
            g: 229,
            b: 16,
        },
        NamedColor::Blue => Rgb {
            r: 36,
            g: 114,
            b: 200,
        },
        NamedColor::Magenta => Rgb {
            r: 188,
            g: 63,
            b: 188,
        },
        NamedColor::Cyan => Rgb {
            r: 17,
            g: 168,
            b: 205,
        },
        NamedColor::White => Rgb {
            r: 229,
            g: 229,
            b: 229,
        },
        NamedColor::BrightBlack => Rgb {
            r: 102,
            g: 102,
            b: 102,
        },
        NamedColor::BrightRed => Rgb {
            r: 241,
            g: 76,
            b: 76,
        },
        NamedColor::BrightGreen => Rgb {
            r: 35,
            g: 209,
            b: 139,
        },
        NamedColor::BrightYellow => Rgb {
            r: 245,
            g: 245,
            b: 67,
        },
        NamedColor::BrightBlue => Rgb {
            r: 59,
            g: 142,
            b: 234,
        },
        NamedColor::BrightMagenta => Rgb {
            r: 214,
            g: 112,
            b: 214,
        },
        NamedColor::BrightCyan => Rgb {
            r: 41,
            g: 184,
            b: 219,
        },
        NamedColor::BrightWhite => Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        NamedColor::Foreground => Rgb {
            r: 215,
            g: 221,
            b: 232,
        },
        NamedColor::Background => Rgb {
            r: 15,
            g: 16,
            b: 20,
        },
        NamedColor::Cursor => Rgb {
            r: 110,
            g: 170,
            b: 255,
        },
        NamedColor::DimBlack => default_terminal_named_color(NamedColor::Black) * DIM_FACTOR,
        NamedColor::DimRed => default_terminal_named_color(NamedColor::Red) * DIM_FACTOR,
        NamedColor::DimGreen => default_terminal_named_color(NamedColor::Green) * DIM_FACTOR,
        NamedColor::DimYellow => default_terminal_named_color(NamedColor::Yellow) * DIM_FACTOR,
        NamedColor::DimBlue => default_terminal_named_color(NamedColor::Blue) * DIM_FACTOR,
        NamedColor::DimMagenta => default_terminal_named_color(NamedColor::Magenta) * DIM_FACTOR,
        NamedColor::DimCyan => default_terminal_named_color(NamedColor::Cyan) * DIM_FACTOR,
        NamedColor::DimWhite => default_terminal_named_color(NamedColor::White) * DIM_FACTOR,
        NamedColor::BrightForeground => default_terminal_named_color(NamedColor::Foreground),
        NamedColor::DimForeground => {
            default_terminal_named_color(NamedColor::Foreground) * DIM_FACTOR
        }
    }
}

fn push_snapshot_line(lines: &mut Vec<String>, line: &str) {
    lines.push(line.trim_end_matches(' ').to_owned());
}

fn terminal_tty_options(config: &LiveTerminalConfig) -> TtyOptions {
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
fn tty_process_id(_pty: &tty::Pty) -> Option<u32> {
    None
}

#[cfg(not(windows))]
fn tty_process_id(pty: &tty::Pty) -> Option<u32> {
    Some(pty.child().id())
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::{
        index::{Column, Line, Point},
        term::test::mock_term,
    };
    use editor_jobs::{JobManager, JobSpec};

    use super::{
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
}
