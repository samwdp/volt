#![doc = r#"SDL3 windowing, input, and demo shell rendering for the native editor."#]

use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fmt,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use editor_buffer::{TextBuffer, TextPoint, TextRange, WordKind};
use editor_core::{
    Buffer, BufferId, BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope,
    KeymapVimMode, WorkspaceId, builtins,
};
use editor_fs::discover_projects;
use editor_git::list_repository_files;
use editor_jobs::{JobManager, JobSpec};
use editor_picker::{PickerItem, PickerSession};
use editor_plugin_host::load_auto_loaded_packages;
use editor_render::{
    DrawCommand, PixelRect, RenderBackend, RenderColor, RenderError, centered_rect,
    find_font_by_name, find_system_monospace_font, horizontal_pane_rects,
};
use editor_syntax::{SyntaxError, SyntaxRegistry, SyntaxSnapshot};
use editor_theme::{Color as ThemeColor, ThemeRegistry};
use sdl3::{
    event::Event,
    keyboard::{Keycode, Mod},
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{Canvas, RenderTarget},
    ttf::Font,
    video::Window,
};
use sdl3_ttf_sys as _;

const HOOK_MOVE_LEFT: &str = "editor.cursor.move-left";
const HOOK_MOVE_DOWN: &str = "editor.cursor.move-down";
const HOOK_MOVE_UP: &str = "editor.cursor.move-up";
const HOOK_MOVE_RIGHT: &str = "editor.cursor.move-right";
const HOOK_MOVE_WORD_FORWARD: &str = "editor.cursor.move-word-forward";
const HOOK_MOVE_WORD_BACKWARD: &str = "editor.cursor.move-word-backward";
const HOOK_MOVE_WORD_END: &str = "editor.cursor.move-word-end";
const HOOK_MOVE_BIG_WORD_FORWARD: &str = "editor.cursor.move-big-word-forward";
const HOOK_MOVE_BIG_WORD_BACKWARD: &str = "editor.cursor.move-big-word-backward";
const HOOK_MOVE_BIG_WORD_END: &str = "editor.cursor.move-big-word-end";
const HOOK_MOVE_SENTENCE_FORWARD: &str = "editor.cursor.move-sentence-forward";
const HOOK_MOVE_SENTENCE_BACKWARD: &str = "editor.cursor.move-sentence-backward";
const HOOK_MOVE_PARAGRAPH_FORWARD: &str = "editor.cursor.move-paragraph-forward";
const HOOK_MOVE_PARAGRAPH_BACKWARD: &str = "editor.cursor.move-paragraph-backward";
const HOOK_MATCH_PAIR: &str = "editor.cursor.match-pair";
const HOOK_MOVE_LINE_START: &str = "editor.cursor.move-line-start";
const HOOK_MOVE_LINE_FIRST_NON_BLANK: &str = "editor.cursor.move-line-first-non-blank";
const HOOK_MOVE_LINE_END: &str = "editor.cursor.move-line-end";
const HOOK_MOVE_SCREEN_TOP: &str = "editor.cursor.move-screen-top";
const HOOK_MOVE_SCREEN_MIDDLE: &str = "editor.cursor.move-screen-middle";
const HOOK_MOVE_SCREEN_BOTTOM: &str = "editor.cursor.move-screen-bottom";
const HOOK_GOTO_FIRST_LINE: &str = "editor.cursor.goto-first-line";
const HOOK_GOTO_LAST_LINE: &str = "editor.cursor.goto-last-line";
const HOOK_SCROLL_HALF_PAGE_DOWN: &str = "editor.vim.scroll-half-page-down";
const HOOK_SCROLL_HALF_PAGE_UP: &str = "editor.vim.scroll-half-page-up";
const HOOK_SCROLL_PAGE_DOWN: &str = "editor.vim.scroll-page-down";
const HOOK_SCROLL_PAGE_UP: &str = "editor.vim.scroll-page-up";
const HOOK_SCROLL_LINE_DOWN: &str = "editor.vim.scroll-line-down";
const HOOK_SCROLL_LINE_UP: &str = "editor.vim.scroll-line-up";
const HOOK_MODE_INSERT: &str = "editor.mode.insert";
const HOOK_MODE_NORMAL: &str = "editor.mode.normal";
const HOOK_VIM_EDIT: &str = "editor.vim.edit";
const HOOK_BUFFER_SAVE: &str = "buffer.save";
const HOOK_WORKSPACE_SAVE: &str = "workspace.save";
const HOOK_WORKSPACE_FORMAT: &str = "workspace.format";
const HOOK_WORKSPACE_FORMATTER_REGISTER: &str = "workspace.formatter.register";
const HOOK_PICKER_OPEN: &str = "ui.picker.open";
const HOOK_PICKER_NEXT: &str = "ui.picker.next";
const HOOK_PICKER_PREVIOUS: &str = "ui.picker.previous";
const HOOK_PICKER_SUBMIT: &str = "ui.picker.submit";
const HOOK_PICKER_CANCEL: &str = "ui.picker.cancel";
const HOOK_POPUP_TOGGLE: &str = "ui.popup.toggle";
const OPTION_LINE_NUMBER_RELATIVE: &str = "ui.line-number.relative";
const OPTION_FONT: &str = "font";
const OPTION_FONT_SIZE: &str = "font_size";
const OPTION_OPACITY: &str = "opacity";
const OPTION_OPACITY_TYPE: &str = "opacity_type";
const OPTION_CURSOR_ROUNDNESS: &str = "cursor_roundness";
const OPTION_PICKER_ROUNDNESS: &str = "picker_roundness";
const SEARCH_PICKER_ITEM_LIMIT: usize = 512;
const WINDOW_ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../volt/assets/logo.png"
));

/// Configures the demo shell loop.
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// Window title.
    pub title: String,
    /// Window width in pixels.
    pub width: u32,
    /// Window height in pixels.
    pub height: u32,
    /// Monospace font size in pixels.
    pub font_size: u32,
    /// Whether the window should start hidden.
    pub hidden: bool,
    /// Renderer backend requested by the shell configuration.
    pub render_backend: RenderBackend,
    /// Optional frame limit used for smoke tests.
    pub frame_limit: Option<u32>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            title: "volt shell demo".to_owned(),
            width: 1200,
            height: 760,
            font_size: 18,
            hidden: false,
            render_backend: RenderBackend::SdlCanvas,
            frame_limit: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpacityType {
    Transparent,
    Win32Backdrop(Win32Backdrop),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Win32Backdrop {
    Auto,
    None,
    Mica,
    MicaAlt,
    Acrylic,
    Tabbed,
}

impl OpacityType {
    fn from_value(value: &str) -> Option<Self> {
        let value = value.trim().to_ascii_lowercase();
        match value.as_str() {
            "transparent" => Some(Self::Transparent),
            "blurred" | "blur" | "acrylic" => Some(Self::Win32Backdrop(Win32Backdrop::Acrylic)),
            "auto" => Some(Self::Win32Backdrop(Win32Backdrop::Auto)),
            "none" | "disable" | "disabled" => Some(Self::Win32Backdrop(Win32Backdrop::None)),
            "mica" => Some(Self::Win32Backdrop(Win32Backdrop::Mica)),
            "mica-alt" | "mica_alt" | "micaalt" => {
                Some(Self::Win32Backdrop(Win32Backdrop::MicaAlt))
            }
            "tabbed" => Some(Self::Win32Backdrop(Win32Backdrop::Tabbed)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ThemeRuntimeSettings {
    font_request: Option<String>,
    font_size: u32,
    opacity: f32,
    opacity_type: OpacityType,
}

/// Summary returned after the demo shell exits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSummary {
    /// Number of frames presented.
    pub frames_rendered: u32,
    /// Number of visible panes.
    pub pane_count: usize,
    /// Whether the picker popup was visible when the loop exited.
    pub popup_visible: bool,
    /// Renderer backend used for the shell session.
    pub render_backend: RenderBackend,
    /// Concrete SDL renderer chosen for the shell session.
    pub renderer_name: String,
    /// Font path selected by the text renderer.
    pub font_path: String,
}

/// Errors raised while creating or running the SDL demo shell.
#[derive(Debug)]
pub enum ShellError {
    /// SDL initialization or rendering failed.
    Sdl(String),
    /// Font lookup failed before SDL_ttf could load the font.
    Render(RenderError),
    /// Runtime or shell orchestration failed.
    Runtime(String),
}

enum DrawTarget<'a> {
    Scene(&'a mut Vec<DrawCommand>),
}

impl DrawTarget<'_> {
    fn clear(&mut self, color: Color) {
        match self {
            Self::Scene(scene) => scene.push(DrawCommand::Clear {
                color: to_render_color(color),
            }),
        }
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sdl(error) => write!(formatter, "SDL error: {error}"),
            Self::Render(error) => error.fmt(formatter),
            Self::Runtime(error) => write!(formatter, "runtime error: {error}"),
        }
    }
}

impl Error for ShellError {}

impl From<RenderError> for ShellError {
    fn from(error: RenderError) -> Self {
        Self::Render(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Normal,
    Insert,
    Replace,
    Visual,
}

impl InputMode {
    fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Replace => "REPLACE",
            Self::Visual => "VISUAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VimOperator {
    Delete,
    Change,
    Yank,
    ToggleCase,
    Lowercase,
    Uppercase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VimFindKind {
    ForwardTo,
    BackwardTo,
    ForwardBefore,
    BackwardAfter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VimSearchDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellMotion {
    Left,
    Down,
    Up,
    Right,
    WordForward,
    WordBackward,
    WordEnd,
    BigWordForward,
    BigWordBackward,
    BigWordEnd,
    SentenceForward,
    SentenceBackward,
    ParagraphForward,
    ParagraphBackward,
    WordEndBackward,
    BigWordEndBackward,
    MatchPair,
    LineStart,
    LineFirstNonBlank,
    LineEnd,
    ScreenTop,
    ScreenMiddle,
    ScreenBottom,
    FirstLine,
    LastLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollCommand {
    HalfPageDown,
    HalfPageUp,
    PageDown,
    PageUp,
    LineDown,
    LineUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VimTextObjectKind {
    Word,
    BigWord,
    Sentence,
    Paragraph,
    Delimited { open: char, close: char },
    Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VimPending {
    Operator {
        operator: VimOperator,
        count: usize,
    },
    Format {
        count: usize,
    },
    FindTarget {
        operator: Option<VimOperator>,
        kind: VimFindKind,
        count: usize,
    },
    GPrefix {
        operator: Option<VimOperator>,
        line_target: Option<usize>,
    },
    TextObject {
        operator: VimOperator,
        around: bool,
        count: usize,
    },
    VisualTextObject {
        around: bool,
        count: usize,
    },
    ReplaceChar {
        count: usize,
    },
    Register,
    MarkSet,
    MarkJump {
        linewise: bool,
    },
    MacroRecord,
    MacroPlayback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LastFind {
    kind: VimFindKind,
    target: char,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LastSearch {
    direction: VimSearchDirection,
    query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum YankRegister {
    Character(String),
    Line(String),
    Block(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FormatterSpec {
    language_id: String,
    program: String,
    args: Vec<String>,
}

impl FormatterSpec {
    fn from_hook_detail(detail: &str) -> Result<Self, String> {
        let mut parts = detail
            .split('|')
            .map(str::trim)
            .filter(|part| !part.is_empty());
        let language_id = parts
            .next()
            .ok_or_else(|| "formatter registration missing language id".to_owned())?;
        let program = parts
            .next()
            .ok_or_else(|| "formatter registration missing program".to_owned())?;
        let args = parts.map(|part| part.to_owned()).collect();
        Ok(Self {
            language_id: language_id.to_owned(),
            program: program.to_owned(),
            args,
        })
    }
}

#[derive(Debug, Default, Clone)]
struct FormatterRegistry {
    formatters: BTreeMap<String, FormatterSpec>,
}

impl FormatterRegistry {
    fn register(&mut self, spec: FormatterSpec) -> Result<(), String> {
        if let Some(existing) = self.formatters.get(&spec.language_id) {
            if existing == &spec {
                return Ok(());
            }
            return Err(format!(
                "formatter already registered for language `{}`",
                spec.language_id
            ));
        }
        self.formatters.insert(spec.language_id.clone(), spec);
        Ok(())
    }

    fn formatter_for_language(&self, language_id: &str) -> Option<&FormatterSpec> {
        self.formatters.get(language_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VimMark {
    buffer_id: BufferId,
    point: TextPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VimVisualSnapshot {
    buffer_id: BufferId,
    anchor: TextPoint,
    head: TextPoint,
    kind: VisualSelectionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VimRecordedInput {
    Text(String),
    Chord(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum VisualSelectionKind {
    #[default]
    Character,
    Line,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockSelection {
    start_line: usize,
    end_line: usize,
    start_col: usize,
    end_col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisualSelection {
    Range(TextRange),
    Block(BlockSelection),
}

#[derive(Debug, Clone)]
struct BlockInsertState {
    selection: BlockSelection,
    insert_col: usize,
    origin_line: usize,
    original_line: String,
}

#[derive(Debug, Clone, Copy)]
struct YankFlash {
    buffer_id: BufferId,
    selection: VisualSelection,
    until: Instant,
}

#[derive(Debug, Clone, Default)]
struct VimState {
    count: Option<usize>,
    pending: Option<VimPending>,
    visual_anchor: Option<TextPoint>,
    visual_kind: VisualSelectionKind,
    last_find: Option<LastFind>,
    last_search: Option<LastSearch>,
    yank: Option<YankRegister>,
    registers: BTreeMap<char, YankRegister>,
    active_register: Option<char>,
    marks: BTreeMap<char, VimMark>,
    last_visual: Option<VimVisualSnapshot>,
    pending_change_prefix: Option<VimRecordedInput>,
    recording_macro: Option<char>,
    macro_buffer: Vec<VimRecordedInput>,
    macros: BTreeMap<char, Vec<VimRecordedInput>>,
    last_macro: Option<char>,
    skip_next_macro_input: bool,
    recording_change: bool,
    finish_change_on_normal: bool,
    finish_change_after_input: bool,
    change_buffer: Vec<VimRecordedInput>,
    last_change: Vec<VimRecordedInput>,
    replaying: bool,
    block_insert: Option<BlockInsertState>,
}

impl VimState {
    fn push_count_digit(&mut self, digit: usize) {
        self.count = Some(
            self.count
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add(digit),
        );
    }

    fn take_count(&mut self) -> Option<usize> {
        self.count.take()
    }

    fn take_count_or_one(&mut self) -> usize {
        self.take_count().unwrap_or(1)
    }

    fn clear_transient(&mut self) {
        self.count = None;
        self.pending = None;
        self.pending_change_prefix = None;
    }
}

#[derive(Debug, Clone)]
struct LineSyntaxSpan {
    start: usize,
    end: usize,
    theme_token: String,
}

#[derive(Debug, Clone)]
struct ShellBuffer {
    id: BufferId,
    name: String,
    kind: BufferKind,
    text: TextBuffer,
    scroll_row: usize,
    viewport_lines: usize,
    syntax_error: Option<String>,
    syntax_lines: BTreeMap<usize, Vec<LineSyntaxSpan>>,
    syntax_dirty: bool,
    last_edit_at: Option<Instant>,
}

impl ShellBuffer {
    fn from_runtime_buffer(buffer: &Buffer, lines: Vec<String>) -> Self {
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };

        Self {
            id: buffer.id(),
            name: buffer.name().to_owned(),
            kind: buffer.kind().clone(),
            text,
            scroll_row: 0,
            viewport_lines: 1,
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            last_edit_at: None,
        }
    }

    fn from_text_buffer(buffer: &Buffer, text: TextBuffer) -> Self {
        Self {
            id: buffer.id(),
            name: buffer.name().to_owned(),
            kind: buffer.kind().clone(),
            text,
            scroll_row: 0,
            viewport_lines: 1,
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            last_edit_at: None,
        }
    }

    fn placeholder(buffer_id: BufferId, name: &str, kind: BufferKind) -> Self {
        let lines = placeholder_lines(name, &kind);
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };

        Self {
            id: buffer_id,
            name: name.to_owned(),
            kind,
            text,
            scroll_row: 0,
            viewport_lines: 1,
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            last_edit_at: None,
        }
    }

    fn id(&self) -> BufferId {
        self.id
    }

    fn display_name(&self) -> &str {
        &self.name
    }

    fn kind_label(&self) -> String {
        buffer_kind_label(&self.kind)
    }

    fn cursor_row(&self) -> usize {
        self.text.cursor().line
    }

    fn cursor_col(&self) -> usize {
        self.text.cursor().column
    }

    fn cursor_point(&self) -> TextPoint {
        self.text.cursor()
    }

    fn line_count(&self) -> usize {
        self.text.line_count()
    }

    fn visible_lines(&self, max_lines: usize) -> Vec<String> {
        self.text.lines(self.scroll_row, max_lines)
    }

    fn line_len_chars(&self, line_index: usize) -> usize {
        self.text.line_len_chars(line_index).unwrap_or(0)
    }

    fn path(&self) -> Option<&Path> {
        self.text.path()
    }

    fn is_dirty(&self) -> bool {
        self.text.is_dirty()
    }

    fn save_to_path(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.text.save_to_path(path)
    }

    fn set_syntax_snapshot(&mut self, syntax: Option<SyntaxSnapshot>) {
        self.syntax_lines = syntax.as_ref().map(index_syntax_lines).unwrap_or_default();
        self.syntax_dirty = false;
        self.last_edit_at = None;
    }

    fn set_syntax_error(&mut self, error: Option<String>) {
        self.syntax_error = error;
    }

    fn mark_syntax_dirty(&mut self) {
        if self.kind == BufferKind::File {
            self.syntax_dirty = true;
            self.last_edit_at = Some(Instant::now());
        }
    }

    fn syntax_refresh_due(&self, now: Instant) -> bool {
        const SYNTAX_REFRESH_DEBOUNCE: Duration = Duration::from_millis(75);
        self.syntax_dirty
            && self
                .last_edit_at
                .map(|last_edit_at| now.duration_since(last_edit_at) >= SYNTAX_REFRESH_DEBOUNCE)
                .unwrap_or(true)
    }

    fn line_syntax_spans(&self, line_index: usize) -> Option<&[LineSyntaxSpan]> {
        self.syntax_lines.get(&line_index).map(Vec::as_slice)
    }

    fn insert_text(&mut self, text: &str) {
        self.text.insert_text(text);
    }

    fn replace_mode_text(&mut self, text: &str) {
        for character in text.chars() {
            if character == '\n' {
                self.text.insert_newline();
                continue;
            }

            let point = self.cursor_point();
            let Some(next) = self.point_after(point) else {
                self.text.insert_text(&character.to_string());
                continue;
            };

            let current = self.slice(TextRange::new(point, next));
            if current == "\n" {
                self.text.insert_text(&character.to_string());
            } else {
                self.text
                    .replace(TextRange::new(point, next), &character.to_string());
            }
        }
    }

    fn insert_newline(&mut self) {
        self.text.insert_newline();
    }

    fn backspace(&mut self) {
        let _ = self.text.backspace();
    }

    fn move_left(&mut self) -> bool {
        self.text.move_left()
    }

    fn move_right(&mut self) -> bool {
        self.text.move_right()
    }

    fn move_up(&mut self) -> bool {
        self.text.move_up()
    }

    fn move_down(&mut self) -> bool {
        self.text.move_down()
    }

    fn move_word_forward(&mut self) -> bool {
        self.text.move_word_forward()
    }

    fn move_big_word_forward(&mut self) -> bool {
        self.text.move_big_word_forward()
    }

    fn move_word_backward(&mut self) -> bool {
        self.text.move_word_backward()
    }

    fn move_big_word_backward(&mut self) -> bool {
        self.text.move_big_word_backward()
    }

    fn move_word_end(&mut self) -> bool {
        self.text.move_word_end_forward()
    }

    fn move_big_word_end(&mut self) -> bool {
        self.text.move_big_word_end_forward()
    }

    fn move_word_end_backward(&mut self) -> bool {
        self.text.move_word_end_backward()
    }

    fn move_big_word_end_backward(&mut self) -> bool {
        self.text.move_big_word_end_backward()
    }

    fn move_matching_delimiter(&mut self) -> bool {
        self.text.move_matching_delimiter()
    }

    fn move_sentence_forward(&mut self) -> bool {
        self.text.move_sentence_forward()
    }

    fn move_sentence_backward(&mut self) -> bool {
        self.text.move_sentence_backward()
    }

    fn move_paragraph_forward(&mut self) -> bool {
        self.text.move_paragraph_forward()
    }

    fn move_paragraph_backward(&mut self) -> bool {
        self.text.move_paragraph_backward()
    }

    fn set_cursor(&mut self, point: TextPoint) {
        self.text.set_cursor(point);
    }

    fn point_after(&self, point: TextPoint) -> Option<TextPoint> {
        self.text.point_after(point)
    }

    fn move_line_start(&mut self) -> bool {
        let before = self.cursor_point();
        self.text
            .set_cursor(editor_buffer::TextPoint::new(self.cursor_row(), 0));
        self.cursor_point() != before
    }

    fn move_line_first_non_blank(&mut self) -> bool {
        let before = self.cursor_point();
        if let Some(point) = self.text.first_non_blank_in_line(self.cursor_row()) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn move_line_end(&mut self) -> bool {
        let before = self.cursor_point();
        let line = self.cursor_row();
        let column = self
            .text
            .line_len_chars(line)
            .map(|line_len| line_len.saturating_sub(1))
            .unwrap_or(0);
        self.text
            .set_cursor(editor_buffer::TextPoint::new(line, column));
        self.cursor_point() != before
    }

    fn goto_first_line(&mut self) -> bool {
        let before = self.cursor_point();
        if let Some(point) = self.text.first_non_blank_in_line(0) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn goto_last_line(&mut self) -> bool {
        let before = self.cursor_point();
        let line = self.line_count().saturating_sub(1);
        if let Some(point) = self.text.first_non_blank_in_line(line) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn goto_line(&mut self, line_index: usize) -> bool {
        let before = self.cursor_point();
        let line = line_index.min(self.line_count().saturating_sub(1));
        let point = self
            .text
            .first_non_blank_in_line(line)
            .unwrap_or(TextPoint::new(line, 0));
        self.text.set_cursor(point);
        self.cursor_point() != before
    }

    fn append_after_cursor(&mut self) {
        let line = self.cursor_row();
        let column = self
            .text
            .line_len_chars(line)
            .map(|line_len| {
                if self.cursor_col() < line_len {
                    self.cursor_col() + 1
                } else {
                    line_len
                }
            })
            .unwrap_or(self.cursor_col());
        self.text
            .set_cursor(editor_buffer::TextPoint::new(line, column));
    }

    fn append_line_end(&mut self) {
        let line = self.cursor_row();
        let column = self.text.line_len_chars(line).unwrap_or(0);
        self.text
            .set_cursor(editor_buffer::TextPoint::new(line, column));
    }

    fn insert_line_start(&mut self) {
        if let Some(point) = self.text.first_non_blank_in_line(self.cursor_row()) {
            self.text.set_cursor(point);
        }
    }

    fn open_line_below(&mut self) {
        let line = self.cursor_row();
        let column = self.text.line_len_chars(line).unwrap_or(0);
        self.text
            .set_cursor(editor_buffer::TextPoint::new(line, column));
        self.text.insert_newline();
    }

    fn open_line_above(&mut self) {
        let line = self.cursor_row();
        self.text.set_cursor(editor_buffer::TextPoint::new(line, 0));
        self.text.insert_newline();
        let _ = self.text.move_up();
    }

    fn undo(&mut self) {
        let _ = self.text.undo();
    }

    fn redo(&mut self) {
        let _ = self.text.redo();
    }

    fn delete_range(&mut self, range: TextRange) {
        self.text.delete(range);
    }

    fn replace_range(&mut self, range: TextRange, text: &str) {
        self.text.replace(range, text);
    }

    fn replace_chars_at_cursor(&mut self, character: char, count: usize) -> bool {
        let original = self.cursor_point();
        let mut replaced = false;
        let mut point = original;
        for _ in 0..count.max(1) {
            let Some(next) = self.point_after(point) else {
                break;
            };
            if self.slice(TextRange::new(point, next)) == "\n" {
                break;
            }
            self.replace_range(TextRange::new(point, next), &character.to_string());
            replaced = true;
            point = next;
        }
        self.set_cursor(original);
        replaced
    }

    fn slice(&self, range: TextRange) -> String {
        self.text.slice(range)
    }

    fn line_range(&self, line_index: usize) -> Option<TextRange> {
        self.text.line_range(line_index)
    }

    fn line_span_range(&self, start_line: usize, count: usize) -> Option<TextRange> {
        if self.line_count() == 0 || count == 0 {
            return None;
        }

        let start_line = start_line.min(self.line_count().saturating_sub(1));
        let end_line =
            (start_line + count.saturating_sub(1)).min(self.line_count().saturating_sub(1));
        Some(TextRange::new(
            self.line_range(start_line)?.start(),
            self.line_range(end_line)?.end(),
        ))
    }

    fn text_object_range(
        &self,
        kind: VimTextObjectKind,
        around: bool,
        count: usize,
    ) -> Option<TextRange> {
        match kind {
            VimTextObjectKind::Word => self.text.word_range_at(self.cursor_point(), around, count),
            VimTextObjectKind::BigWord => {
                self.text
                    .word_range_at_kind(self.cursor_point(), WordKind::BigWord, around, count)
            }
            VimTextObjectKind::Sentence => {
                self.text
                    .sentence_range_at(self.cursor_point(), around, count)
            }
            VimTextObjectKind::Paragraph => {
                self.text
                    .paragraph_range_at(self.cursor_point(), around, count)
            }
            VimTextObjectKind::Delimited { open, close } => {
                self.text
                    .delimited_range_at(self.cursor_point(), open, close, around)
            }
            VimTextObjectKind::Tag => self.text.tag_range_at(self.cursor_point(), around),
        }
    }

    fn move_find(&mut self, kind: VimFindKind, target: char, count: usize) -> bool {
        let repeat = count.max(1);
        let mut moved = false;
        for _ in 0..repeat {
            let next = match kind {
                VimFindKind::ForwardTo => {
                    self.text.find_forward_in_line(self.cursor_point(), target)
                }
                VimFindKind::BackwardTo => {
                    self.text.find_backward_in_line(self.cursor_point(), target)
                }
                VimFindKind::ForwardBefore => self
                    .text
                    .find_forward_in_line(self.cursor_point(), target)
                    .and_then(|point| self.text.point_before(point)),
                VimFindKind::BackwardAfter => self
                    .text
                    .find_backward_in_line(self.cursor_point(), target)
                    .and_then(|point| self.text.point_after(point)),
            };
            let Some(next) = next else {
                return moved;
            };
            self.text.set_cursor(next);
            moved = true;
        }
        moved
    }

    fn insert_at(&mut self, point: TextPoint, text: &str) {
        self.text.set_cursor(point);
        self.text.insert_text(text);
    }

    fn scroll_by(&mut self, delta: i32) {
        let max_scroll = self.line_count().saturating_sub(1) as i32;
        let next = (self.scroll_row as i32 + delta).clamp(0, max_scroll);
        self.scroll_row = next as usize;
    }

    fn set_viewport_lines(&mut self, visible_lines: usize) {
        self.viewport_lines = visible_lines.max(1);
    }

    fn viewport_lines(&self) -> usize {
        self.viewport_lines.max(1)
    }

    fn line_at_viewport_offset(&self, offset: usize) -> usize {
        let max_line = self.line_count().saturating_sub(1);
        self.scroll_row.saturating_add(offset).min(max_line)
    }

    fn move_to_viewport_offset(&mut self, offset: usize) -> bool {
        if self.line_count() == 0 {
            return false;
        }
        let target_line = self.line_at_viewport_offset(offset);
        self.goto_line(target_line)
    }

    fn move_to_viewport_middle(&mut self) -> bool {
        let middle = self.viewport_lines().saturating_sub(1) / 2;
        self.move_to_viewport_offset(middle)
    }

    fn ensure_visible(&mut self, visible_lines: usize) {
        let cursor_row = self.cursor_row();
        if cursor_row < self.scroll_row {
            self.scroll_row = cursor_row;
            return;
        }

        let bottom = self.scroll_row + visible_lines.saturating_sub(1);
        if cursor_row > bottom {
            self.scroll_row = cursor_row.saturating_sub(visible_lines.saturating_sub(1));
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ShellPane {
    buffer_id: BufferId,
}

#[derive(Debug, Clone)]
enum PickerAction {
    NoOp,
    ExecuteCommand(String),
    FocusBuffer(BufferId),
    OpenFile(PathBuf),
    ActivateTheme(String),
    VimSearch(VimSearchDirection),
    VimSearchResult {
        direction: VimSearchDirection,
        target: TextPoint,
    },
    InstallTreeSitterLanguage(String),
    CreateWorkspace { name: String, root: PathBuf },
    SwitchWorkspace(WorkspaceId),
    DeleteWorkspace(WorkspaceId),
}

#[derive(Debug, Clone)]
struct PickerEntry {
    item: PickerItem,
    action: PickerAction,
}

#[derive(Debug, Clone)]
struct PickerOverlay {
    session: PickerSession,
    actions: BTreeMap<String, PickerAction>,
    submit_action: Option<PickerAction>,
}

impl PickerOverlay {
    fn from_entries(title: impl Into<String>, entries: Vec<PickerEntry>) -> Self {
        let title = title.into();
        let mut actions = BTreeMap::new();
        let items = entries
            .into_iter()
            .map(|entry| {
                actions.insert(entry.item.id().to_owned(), entry.action);
                entry.item
            })
            .collect();

        Self {
            session: PickerSession::new(title, items).with_result_limit(48),
            actions,
            submit_action: None,
        }
    }

    fn search(title: impl Into<String>, direction: VimSearchDirection, entries: Vec<PickerEntry>) -> Self {
        let title = title.into();
        let mut actions = BTreeMap::new();
        let items = entries
            .into_iter()
            .map(|entry| {
                actions.insert(entry.item.id().to_owned(), entry.action);
                entry.item
            })
            .collect();

        Self {
            session: PickerSession::new(title, items)
                .with_result_limit(48)
                .with_preserve_order(),
            actions,
            submit_action: Some(PickerAction::VimSearch(direction)),
        }
    }

    fn session(&self) -> &PickerSession {
        &self.session
    }

    fn selected_action(&self) -> Option<PickerAction> {
        if let Some(selected) = self.session.selected()
            && let Some(action) = self.actions.get(selected.item().id())
        {
            return Some(action.clone());
        }
        self.submit_action.clone()
    }

    fn vim_search_direction(&self) -> Option<VimSearchDirection> {
        match self.submit_action.as_ref() {
            Some(PickerAction::VimSearch(direction)) => Some(*direction),
            _ => None,
        }
    }

    fn set_entries(&mut self, entries: Vec<PickerEntry>, selected_index: usize) {
        let mut actions = BTreeMap::new();
        let items = entries
            .into_iter()
            .map(|entry| {
                actions.insert(entry.item.id().to_owned(), entry.action);
                entry.item
            })
            .collect();
        self.actions = actions;
        self.session.set_items(items);
        self.session.set_selected_index(selected_index);
    }

    fn append_query(&mut self, text: &str) {
        let mut query = self.session.query().to_owned();
        query.push_str(text);
        self.session.set_query(query);
    }

    fn backspace_query(&mut self) {
        let mut query = self.session.query().chars().collect::<Vec<_>>();
        if query.pop().is_some() {
            self.session
                .set_query(query.into_iter().collect::<String>());
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimePopupSnapshot {
    title: String,
    buffer_ids: Vec<BufferId>,
    active_buffer: BufferId,
}

#[derive(Debug, Clone)]
struct ShellWorkspaceView {
    buffer_ids: Vec<BufferId>,
    panes: Vec<ShellPane>,
    active_pane: usize,
    split_buffer_id: BufferId,
}

impl ShellWorkspaceView {
    fn new(
        primary_buffer_id: BufferId,
        split_buffer_id: BufferId,
        buffer_ids: Vec<BufferId>,
    ) -> Self {
        Self {
            buffer_ids,
            panes: vec![ShellPane {
                buffer_id: primary_buffer_id,
            }],
            active_pane: 0,
            split_buffer_id,
        }
    }
}

#[derive(Debug, Clone)]
struct ShellUiState {
    buffers: Vec<ShellBuffer>,
    workspace_views: BTreeMap<WorkspaceId, ShellWorkspaceView>,
    active_workspace: WorkspaceId,
    previous_workspace: Option<WorkspaceId>,
    default_workspace: WorkspaceId,
    input_mode: InputMode,
    vim: VimState,
    attached_lsp_servers: BTreeMap<WorkspaceId, String>,
    picker: Option<PickerOverlay>,
    yank_flash: Option<YankFlash>,
}

impl ShellUiState {
    fn new(
        default_workspace: WorkspaceId,
        primary: ShellBuffer,
        secondary: ShellBuffer,
        split_buffer_id: BufferId,
    ) -> Self {
        let primary_buffer_id = primary.id();
        let secondary_buffer_id = secondary.id();
        let mut workspace_views = BTreeMap::new();
        workspace_views.insert(
            default_workspace,
            ShellWorkspaceView::new(
                primary_buffer_id,
                split_buffer_id,
                vec![primary_buffer_id, secondary_buffer_id],
            ),
        );
        Self {
            buffers: vec![primary, secondary],
            workspace_views,
            active_workspace: default_workspace,
            previous_workspace: None,
            default_workspace,
            input_mode: InputMode::Normal,
            vim: VimState::default(),
            attached_lsp_servers: BTreeMap::new(),
            picker: None,
            yank_flash: None,
        }
    }

    fn pane_count(&self) -> usize {
        self.workspace_view()
            .map(|view| view.panes.len())
            .unwrap_or(0)
    }

    fn picker_visible(&self) -> bool {
        self.picker.is_some()
    }

    fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    fn enter_normal_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.vim.visual_anchor = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.clear_transient();
    }

    fn enter_insert_mode(&mut self) {
        self.input_mode = InputMode::Insert;
        self.vim.visual_anchor = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.clear_transient();
    }

    fn enter_replace_mode(&mut self) {
        self.input_mode = InputMode::Replace;
        self.vim.visual_anchor = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.clear_transient();
    }

    fn enter_visual_mode(&mut self, anchor: TextPoint, kind: VisualSelectionKind) {
        self.input_mode = InputMode::Visual;
        self.vim.visual_anchor = Some(anchor);
        self.vim.visual_kind = kind;
        self.vim.clear_transient();
    }

    fn vim(&self) -> &VimState {
        &self.vim
    }

    fn vim_mut(&mut self) -> &mut VimState {
        &mut self.vim
    }

    fn active_workspace(&self) -> WorkspaceId {
        self.active_workspace
    }

    fn previous_workspace(&self) -> Option<WorkspaceId> {
        self.previous_workspace
    }

    fn default_workspace(&self) -> WorkspaceId {
        self.default_workspace
    }

    fn has_workspace(&self, workspace_id: WorkspaceId) -> bool {
        self.workspace_views.contains_key(&workspace_id)
    }

    fn switch_workspace(&mut self, workspace_id: WorkspaceId) {
        if self.active_workspace != workspace_id {
            self.previous_workspace = Some(self.active_workspace);
            self.active_workspace = workspace_id;
        }
        self.close_picker();
    }

    fn add_workspace(
        &mut self,
        workspace_id: WorkspaceId,
        primary: ShellBuffer,
        secondary: ShellBuffer,
        split_buffer_id: BufferId,
    ) {
        let primary_buffer_id = primary.id();
        let secondary_buffer_id = secondary.id();
        self.insert_buffer(primary);
        self.insert_buffer(secondary);
        self.workspace_views.insert(
            workspace_id,
            ShellWorkspaceView::new(
                primary_buffer_id,
                split_buffer_id,
                vec![primary_buffer_id, secondary_buffer_id],
            ),
        );
    }

    fn remove_workspace(&mut self, workspace_id: WorkspaceId) {
        let removed = self.workspace_views.remove(&workspace_id);
        self.attached_lsp_servers.remove(&workspace_id);
        if let Some(removed) = removed {
            self.buffers
                .retain(|buffer| !removed.buffer_ids.contains(&buffer.id()));
        }
        if self.previous_workspace == Some(workspace_id) {
            self.previous_workspace = None;
        }
        if self.active_workspace == workspace_id {
            self.active_workspace = self.default_workspace;
        }
    }

    fn set_attached_lsp_server(
        &mut self,
        workspace_id: WorkspaceId,
        attached_lsp_server: Option<String>,
    ) {
        match attached_lsp_server {
            Some(server) => {
                self.attached_lsp_servers.insert(workspace_id, server);
            }
            None => {
                self.attached_lsp_servers.remove(&workspace_id);
            }
        }
    }

    fn panes(&self) -> Option<&[ShellPane]> {
        self.workspace_view().map(|view| view.panes.as_slice())
    }

    fn active_pane_index(&self) -> usize {
        self.workspace_view()
            .map(|view| view.active_pane)
            .unwrap_or(0)
    }

    fn active_workspace_buffer_ids(&self) -> Option<&[BufferId]> {
        self.workspace_view().map(|view| view.buffer_ids.as_slice())
    }

    fn attached_lsp_server(&self) -> Option<&str> {
        self.attached_lsp_servers
            .get(&self.active_workspace)
            .map(String::as_str)
    }

    fn workspace_view(&self) -> Option<&ShellWorkspaceView> {
        self.workspace_views.get(&self.active_workspace)
    }

    fn workspace_view_mut(&mut self) -> Option<&mut ShellWorkspaceView> {
        self.workspace_views.get_mut(&self.active_workspace)
    }

    fn insert_buffer(&mut self, buffer: ShellBuffer) {
        if let Some(existing) = self
            .buffers
            .iter_mut()
            .find(|existing| existing.id() == buffer.id())
        {
            *existing = buffer;
        } else {
            self.buffers.push(buffer);
        }
    }

    fn picker(&self) -> Option<&PickerOverlay> {
        self.picker.as_ref()
    }

    fn picker_mut(&mut self) -> Option<&mut PickerOverlay> {
        self.picker.as_mut()
    }

    fn set_picker(&mut self, picker: PickerOverlay) {
        self.picker = Some(picker);
    }

    fn close_picker(&mut self) {
        self.picker = None;
    }

    fn set_yank_flash(&mut self, buffer_id: BufferId, selection: VisualSelection) {
        const YANK_FLASH_DURATION: Duration = Duration::from_millis(140);
        self.yank_flash = Some(YankFlash {
            buffer_id,
            selection,
            until: Instant::now() + YANK_FLASH_DURATION,
        });
    }

    fn yank_flash(&self, buffer_id: BufferId, now: Instant) -> Option<VisualSelection> {
        self.yank_flash.and_then(|flash| {
            (flash.buffer_id == buffer_id && now <= flash.until).then_some(flash.selection)
        })
    }

    fn buffer(&self, buffer_id: BufferId) -> Option<&ShellBuffer> {
        self.buffers.iter().find(|buffer| buffer.id() == buffer_id)
    }

    fn buffer_mut(&mut self, buffer_id: BufferId) -> Option<&mut ShellBuffer> {
        self.buffers
            .iter_mut()
            .find(|buffer| buffer.id() == buffer_id)
    }

    fn ensure_buffer(
        &mut self,
        buffer_id: BufferId,
        name: &str,
        kind: BufferKind,
    ) -> &mut ShellBuffer {
        if let Some(view) = self.workspace_view_mut()
            && !view.buffer_ids.contains(&buffer_id)
        {
            view.buffer_ids.push(buffer_id);
        }

        if let Some(index) = self
            .buffers
            .iter()
            .position(|buffer| buffer.id() == buffer_id)
        {
            return &mut self.buffers[index];
        }

        self.buffers
            .push(ShellBuffer::placeholder(buffer_id, name, kind));
        let index = self.buffers.len() - 1;
        &mut self.buffers[index]
    }

    fn active_buffer_mut(&mut self) -> Option<&mut ShellBuffer> {
        let buffer_id = self
            .workspace_view()?
            .panes
            .get(self.active_pane_index())?
            .buffer_id;
        self.buffer_mut(buffer_id)
    }

    fn active_buffer_id(&self) -> Option<BufferId> {
        self.workspace_view()?
            .panes
            .get(self.active_pane_index())
            .map(|pane| pane.buffer_id)
    }

    fn focus_buffer_in_active_pane(&mut self, buffer_id: BufferId) {
        if self.buffers.iter().any(|buffer| buffer.id() == buffer_id)
            && let Some(view) = self.workspace_view_mut()
            && let Some(pane) = view.panes.get_mut(view.active_pane)
        {
            if !view.buffer_ids.contains(&buffer_id) {
                view.buffer_ids.push(buffer_id);
            }
            pane.buffer_id = buffer_id;
        }
    }

    fn focus_buffer(&mut self, buffer_id: BufferId) {
        self.focus_buffer_in_active_pane(buffer_id);
        self.close_picker();
    }

    fn split_horizontal(&mut self) {
        if let Some(view) = self.workspace_view_mut()
            && view.panes.len() == 1
        {
            view.panes.push(ShellPane {
                buffer_id: view.split_buffer_id,
            });
        }
    }

    fn cycle_active_pane(&mut self) {
        if !self.picker_visible()
            && let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
        {
            view.active_pane = (view.active_pane + 1) % view.panes.len();
        }
    }
}

struct ShellState {
    runtime: EditorRuntime,
}

impl ShellState {
    fn new() -> Result<Self, ShellError> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("volt");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "default", None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;

        register_shell_hooks(&mut runtime).map_err(ShellError::Runtime)?;

        let notes_id = runtime
            .model_mut()
            .create_buffer(workspace_id, "*notes*", BufferKind::Scratch, None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let scratch_id = runtime
            .model_mut()
            .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;

        let (scratch, notes) = {
            let workspace = runtime
                .model()
                .workspace(workspace_id)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            let scratch = workspace.buffer(scratch_id).ok_or_else(|| {
                ShellError::Runtime("scratch buffer missing after bootstrap".to_owned())
            })?;
            let notes = workspace.buffer(notes_id).ok_or_else(|| {
                ShellError::Runtime("notes buffer missing after bootstrap".to_owned())
            })?;
            (
                ShellBuffer::from_runtime_buffer(scratch, initial_scratch_lines()),
                ShellBuffer::from_runtime_buffer(notes, initial_notes_lines()),
            )
        };

        runtime
            .services_mut()
            .insert(ShellUiState::new(workspace_id, scratch, notes, notes_id));
        runtime.services_mut().insert(FormatterRegistry::default());
        runtime.services_mut().insert(Mutex::new(JobManager::new()));
        let mut syntax_registry = SyntaxRegistry::new();
        syntax_registry
            .register_all(user::syntax_languages())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        runtime.services_mut().insert(syntax_registry);
        let mut theme_registry = ThemeRegistry::new();
        theme_registry
            .register_all(user::themes())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        runtime.services_mut().insert(theme_registry);
        load_auto_loaded_packages(&mut runtime, &user::packages())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        ensure_picker_keybindings(&mut runtime).map_err(ShellError::Runtime)?;
        register_lsp_status_hooks(&mut runtime).map_err(ShellError::Runtime)?;

        Ok(Self { runtime })
    }

    fn handle_event(&mut self, event: Event, visible_lines: usize) -> Result<bool, ShellError> {
        self.active_buffer_mut()?.set_viewport_lines(visible_lines);
        match event {
            Event::Quit { .. } => return Ok(true),
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                repeat: false,
                ..
            } => {
                if self.try_runtime_keybinding(keycode, keymod)? {
                    self.sync_active_buffer().map_err(ShellError::Runtime)?;
                    self.ensure_visible(visible_lines)?;
                    return Ok(false);
                }

                if keymod.intersects(ctrl_mod()) && keycode == Keycode::Q {
                    return Ok(true);
                }

                if self.picker_visible()? {
                    if keycode == Keycode::Backspace
                        && let Some(picker) = self.ui_mut()?.picker_mut()
                    {
                        picker.backspace_query();
                        self.refresh_vim_search_picker()?;
                    }
                    self.ensure_visible(visible_lines)?;
                    return Ok(false);
                }

                match keycode {
                    Keycode::Left => {
                        let _ = self.active_buffer_mut()?.move_left();
                    }
                    Keycode::Right => {
                        let _ = self.active_buffer_mut()?.move_right();
                    }
                    Keycode::Up => {
                        let _ = self.active_buffer_mut()?.move_up();
                    }
                    Keycode::Down => {
                        let _ = self.active_buffer_mut()?.move_down();
                    }
                    Keycode::PageDown => self.active_buffer_mut()?.scroll_by(visible_lines as i32),
                    Keycode::PageUp => self.active_buffer_mut()?.scroll_by(-(visible_lines as i32)),
                    Keycode::Return | Keycode::KpEnter
                        if matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace) =>
                    {
                        self.active_buffer_mut()?.insert_newline();
                        self.mark_active_buffer_syntax_dirty()?;
                    }
                    Keycode::Backspace
                        if matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace) =>
                    {
                        self.active_buffer_mut()?.backspace();
                        self.mark_active_buffer_syntax_dirty()?;
                    }
                    Keycode::Tab => self.ui_mut()?.cycle_active_pane(),
                    Keycode::F2 => self.ui_mut()?.split_horizontal(),
                    _ => {}
                }
            }
            Event::TextInput { text, .. } => {
                self.handle_text_input(&text)?;
            }
            _ => {}
        }

        self.ensure_visible(visible_lines)?;
        Ok(false)
    }

    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        target: &mut DrawTarget<'_>,
        font: &Font<'_>,
        width: u32,
        height: u32,
        cell_width: i32,
        line_height: i32,
        ascent: i32,
    ) -> Result<(), ShellError> {
        let ui = self.ui()?;
        let runtime_popup = self.runtime_popup()?;
        let theme_registry = self.runtime.services().get::<ThemeRegistry>();
        let workspace_name = self
            .runtime
            .model()
            .active_workspace()
            .map_err(|error| ShellError::Runtime(error.to_string()))?
            .name()
            .to_owned();
        render_shell_state(
            target,
            font,
            ui,
            runtime_popup.as_ref(),
            &workspace_name,
            ui.attached_lsp_server(),
            theme_registry,
            width,
            height,
            cell_width,
            line_height,
            ascent,
            Instant::now(),
        )
    }

    fn pane_count(&self) -> Result<usize, ShellError> {
        Ok(self.ui()?.pane_count())
    }

    fn picker_visible(&self) -> Result<bool, ShellError> {
        Ok(self.ui()?.picker_visible())
    }

    fn popup_visible(&self) -> Result<bool, ShellError> {
        Ok(self.picker_visible()? || self.runtime_popup()?.is_some())
    }

    fn ui(&self) -> Result<&ShellUiState, ShellError> {
        shell_ui(&self.runtime).map_err(ShellError::Runtime)
    }

    fn ui_mut(&mut self) -> Result<&mut ShellUiState, ShellError> {
        shell_ui_mut(&mut self.runtime).map_err(ShellError::Runtime)
    }

    fn input_mode(&self) -> Result<InputMode, ShellError> {
        Ok(self.ui()?.input_mode())
    }

    fn active_buffer_mut(&mut self) -> Result<&mut ShellBuffer, ShellError> {
        let popup_buffer_id = self.runtime_popup()?.map(|popup| popup.active_buffer);
        let ui = self.ui_mut()?;
        if let Some(buffer_id) = popup_buffer_id {
            return ui
                .buffer_mut(buffer_id)
                .ok_or_else(|| ShellError::Runtime("active popup buffer is missing".to_owned()));
        }

        ui.active_buffer_mut()
            .ok_or_else(|| ShellError::Runtime("active shell buffer is missing".to_owned()))
    }

    fn handle_vim_pending_text(&mut self, chord: &str) -> Result<bool, ShellError> {
        let pending = self.ui()?.vim().pending;
        let Some(pending) = pending else {
            return Ok(false);
        };

        match pending {
            VimPending::Operator { operator, count } => {
                if let Some(digit) = vim_count_digit(chord, self.ui()?.vim().count.is_some()) {
                    self.ui_mut()?.vim_mut().push_count_digit(digit);
                    return Ok(true);
                }

                match (operator, chord) {
                    (VimOperator::Delete, "d")
                    | (VimOperator::Change, "c")
                    | (VimOperator::Yank, "y") => {
                        let lines =
                            count.saturating_mul(self.ui_mut()?.vim_mut().take_count_or_one());
                        apply_linewise_operator(&mut self.runtime, operator, lines)
                            .map_err(ShellError::Runtime)?;
                        return Ok(true);
                    }
                    (_, "i") | (_, "a") => {
                        let around = chord == "a";
                        let count =
                            count.saturating_mul(self.ui_mut()?.vim_mut().take_count_or_one());
                        self.ui_mut()?.vim_mut().pending = Some(VimPending::TextObject {
                            operator,
                            around,
                            count,
                        });
                        return Ok(true);
                    }
                    (_, "g") => {
                        let line_target = self.ui_mut()?.vim_mut().take_count();
                        self.ui_mut()?.vim_mut().pending = Some(VimPending::GPrefix {
                            operator: Some(operator),
                            line_target,
                        });
                        return Ok(true);
                    }
                    _ => {}
                }

                Ok(false)
            }
            VimPending::Format { .. } => {
                if chord == "=" {
                    self.ui_mut()?.vim_mut().clear_transient();
                    emit_workspace_format(&mut self.runtime).map_err(ShellError::Runtime)?;
                    return Ok(true);
                }
                self.ui_mut()?.vim_mut().clear_transient();
                Ok(false)
            }
            VimPending::FindTarget {
                operator,
                kind,
                count,
            } => {
                if let Some(target) = chord.chars().next() {
                    resolve_find_target(&mut self.runtime, operator, kind, count, target)
                        .map_err(ShellError::Runtime)?;
                    return Ok(true);
                }
                Ok(false)
            }
            VimPending::GPrefix {
                operator,
                line_target,
            } => {
                match chord {
                    "g" | "e" | "E" => {
                        if operator.is_none() {
                            self.ui_mut()?.vim_mut().pending_change_prefix = None;
                        }
                        resolve_g_prefix(&mut self.runtime, operator, line_target, chord)
                            .map_err(ShellError::Runtime)?;
                    }
                    "v" if operator.is_none() => {
                        self.ui_mut()?.vim_mut().pending_change_prefix = None;
                        restore_last_visual_selection(&mut self.runtime)
                            .map_err(ShellError::Runtime)?;
                    }
                    "~" | "u" | "U" if operator.is_none() => {
                        let operator = match chord {
                            "~" => VimOperator::ToggleCase,
                            "u" => VimOperator::Lowercase,
                            "U" => VimOperator::Uppercase,
                            _ => VimOperator::ToggleCase,
                        };
                        let prefix = self.ui_mut()?.vim_mut().pending_change_prefix.take();
                        start_change_recording_with_prefix(&mut self.runtime, prefix)
                            .map_err(ShellError::Runtime)?;
                        let count = line_target.unwrap_or(1);
                        self.ui_mut()?.vim_mut().pending =
                            Some(VimPending::Operator { operator, count });
                    }
                    _ => {
                        if operator.is_none() {
                            self.ui_mut()?.vim_mut().pending_change_prefix = None;
                        }
                        self.ui_mut()?.vim_mut().clear_transient();
                    }
                }
                Ok(true)
            }
            VimPending::TextObject {
                operator,
                around,
                count,
            } => {
                if let Some(kind) = vim_text_object_kind(chord) {
                    apply_text_object_operator(&mut self.runtime, operator, kind, around, count)
                        .map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::VisualTextObject { around, count } => {
                if let Some(kind) = vim_text_object_kind(chord) {
                    apply_visual_text_object(&mut self.runtime, kind, around, count)
                        .map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::ReplaceChar { count } => {
                let Some(character) = chord.chars().next() else {
                    self.ui_mut()?.vim_mut().clear_transient();
                    return Ok(true);
                };
                if character != '\n' {
                    let replaced = self
                        .active_buffer_mut()?
                        .replace_chars_at_cursor(character, count);
                    if replaced {
                        self.mark_active_buffer_syntax_dirty()?;
                    }
                }
                self.ui_mut()?.enter_normal_mode();
                schedule_finish_change(&mut self.runtime).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            VimPending::Register => {
                if let Some(register) = chord.chars().next() {
                    self.ui_mut()?.vim_mut().active_register = Some(register);
                }
                self.ui_mut()?.vim_mut().clear_transient();
                Ok(true)
            }
            VimPending::MarkSet => {
                if let Some(mark) = chord.chars().next() {
                    let buffer_id = active_shell_buffer_id(&self.runtime)
                        .map_err(ShellError::Runtime)?;
                    let point = self.active_buffer_mut()?.cursor_point();
                    self.ui_mut()?.vim_mut().marks.insert(
                        mark,
                        VimMark {
                            buffer_id,
                            point,
                        },
                    );
                }
                self.ui_mut()?.vim_mut().clear_transient();
                Ok(true)
            }
            VimPending::MarkJump { linewise } => {
                if let Some(mark) = chord.chars().next() {
                    jump_to_mark(&mut self.runtime, mark, linewise)
                        .map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::MacroRecord => {
                if let Some(register) = chord.chars().next() {
                    start_macro_record(&mut self.runtime, register)
                        .map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::MacroPlayback => {
                let repeat = self.ui_mut()?.vim_mut().take_count_or_one();
                let register = chord.chars().next();
                self.ui_mut()?.vim_mut().clear_transient();
                self.play_macro(register, repeat)?;
                Ok(true)
            }
        }
    }

    fn handle_vim_count_input(&mut self, chord: &str) -> Result<bool, ShellError> {
        if matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace) {
            return Ok(false);
        }

        let has_count = self.ui()?.vim().count.is_some();
        let Some(digit) = vim_count_digit(chord, has_count) else {
            return Ok(false);
        };
        self.ui_mut()?.vim_mut().push_count_digit(digit);
        Ok(true)
    }

    fn clear_stale_vim_count(&mut self) -> Result<(), ShellError> {
        let should_clear = {
            let ui = self.ui()?;
            !matches!(ui.input_mode(), InputMode::Insert | InputMode::Replace)
                && ui.vim().pending.is_none()
                && ui.vim().count.is_some()
        };
        if should_clear {
            self.ui_mut()?.vim_mut().count = None;
        }
        Ok(())
    }

    fn record_vim_input(&mut self, input: VimRecordedInput) -> Result<(), ShellError> {
        let input_mode = self.input_mode()?;
        let vim = self.ui_mut()?.vim_mut();
        if vim.replaying {
            return Ok(());
        }
        let skip_macro = matches!(
            (&input, input_mode),
            (VimRecordedInput::Text(text), InputMode::Normal | InputMode::Visual)
                if text == "q"
        );
        if vim.recording_macro.is_some() && !skip_macro {
            if vim.skip_next_macro_input {
                vim.skip_next_macro_input = false;
            } else {
                vim.macro_buffer.push(input.clone());
            }
        }
        if vim.recording_change {
            vim.change_buffer.push(input);
        }
        Ok(())
    }

    fn maybe_finish_change_after_input(&mut self) -> Result<(), ShellError> {
        let finish = self.ui_mut()?.vim_mut().finish_change_after_input;
        if finish {
            self.ui_mut()?.vim_mut().finish_change_after_input = false;
            finish_change_recording(&mut self.runtime).map_err(ShellError::Runtime)?;
        }
        Ok(())
    }

    fn execute_recorded_chord(&mut self, chord: &str) -> Result<(), ShellError> {
        let vim_mode = keymap_vim_mode(self.input_mode()?);

        if self.picker_visible()?
            && self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Popup, vim_mode, chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Popup, vim_mode, chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            return Ok(());
        }

        if self
            .runtime
            .keymaps()
            .contains_for_mode(&KeymapScope::Global, vim_mode, chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Global, vim_mode, chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            return Ok(());
        }

        if !self.picker_visible()?
            && !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
            && self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Workspace, vim_mode, chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.sync_active_buffer().map_err(ShellError::Runtime)?;
            self.clear_stale_vim_count()?;
        }

        Ok(())
    }

    fn replay_recorded_inputs(&mut self, inputs: &[VimRecordedInput]) -> Result<(), ShellError> {
        if inputs.is_empty() {
            return Ok(());
        }

        self.ui_mut()?.vim_mut().replaying = true;
        let result = inputs.iter().try_for_each(|input| match input {
            VimRecordedInput::Text(text) => self.handle_text_input(text),
            VimRecordedInput::Chord(chord) => self.execute_recorded_chord(chord),
        });
        self.ui_mut()?.vim_mut().replaying = false;
        result
    }

    fn repeat_last_change(&mut self) -> Result<(), ShellError> {
        if self.ui()?.vim().replaying {
            return Ok(());
        }
        let repeat = self.ui_mut()?.vim_mut().take_count_or_one();
        let inputs = self.ui()?.vim().last_change.clone();
        if inputs.is_empty() {
            return Ok(());
        }
        for _ in 0..repeat {
            self.replay_recorded_inputs(&inputs)?;
        }
        Ok(())
    }

    fn play_macro(&mut self, register: Option<char>, repeat: usize) -> Result<(), ShellError> {
        if self.ui()?.vim().replaying {
            return Ok(());
        }
        let inputs = {
            let vim = self.ui_mut()?.vim_mut();
            let target = match register {
                Some('@') => vim.last_macro,
                Some(register) => Some(register),
                None => None,
            };
            let Some(register) = target else {
                vim.clear_transient();
                return Ok(());
            };
            let inputs = vim.macros.get(&register).cloned().unwrap_or_default();
            vim.last_macro = Some(register);
            inputs
        };

        if inputs.is_empty() {
            self.ui_mut()?.vim_mut().clear_transient();
            return Ok(());
        }
        for _ in 0..repeat.max(1) {
            self.replay_recorded_inputs(&inputs)?;
        }
        self.ui_mut()?.vim_mut().clear_transient();
        Ok(())
    }

    fn handle_text_input(&mut self, text: &str) -> Result<(), ShellError> {
        if self.picker_visible()? {
            if let Some(picker) = self.ui_mut()?.picker_mut() {
                picker.append_query(text);
            }
            self.refresh_vim_search_picker()?;
            return Ok(());
        }

        match self.input_mode()? {
            InputMode::Insert => {
                self.active_buffer_mut()?.insert_text(text);
                self.mark_active_buffer_syntax_dirty()?;
                self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            InputMode::Replace => {
                self.active_buffer_mut()?.replace_mode_text(text);
                self.mark_active_buffer_syntax_dirty()?;
                self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            _ => {}
        }

        if let Some(chord) = text_chord(text) {
            if self.handle_vim_pending_text(&chord)? || self.handle_vim_count_input(&chord)? {
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }

            if chord == "." && !self.picker_visible()? {
                self.repeat_last_change()?;
                return Ok(());
            }

            if self.runtime.keymaps().contains_for_mode(
                &KeymapScope::Workspace,
                keymap_vim_mode(self.input_mode()?),
                &chord,
            ) {
                self.runtime
                    .execute_key_binding_for_mode(
                        &KeymapScope::Workspace,
                        keymap_vim_mode(self.input_mode()?),
                        &chord,
                    )
                    .map_err(|error| ShellError::Runtime(error.to_string()))?;
                self.sync_active_buffer().map_err(ShellError::Runtime)?;
                self.clear_stale_vim_count()?;
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
            }
        }

        Ok(())
    }

    fn refresh_vim_search_picker(&mut self) -> Result<(), ShellError> {
        let (direction, query) = {
            let ui = self.ui()?;
            let Some(picker) = ui.picker() else {
                return Ok(());
            };
            let Some(direction) = picker.vim_search_direction() else {
                return Ok(());
            };
            (direction, picker.session().query().to_owned())
        };

        let search_data = {
            let buffer = self.active_buffer_mut()?;
            vim_search_entries(buffer, direction, &query)
        };

        if let Some(picker) = self.ui_mut()?.picker_mut()
            && picker.vim_search_direction().is_some()
        {
            picker.set_entries(search_data.entries, search_data.selected_index);
        }

        Ok(())
    }

    fn try_runtime_keybinding(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        let Some(chord) = keydown_chord(keycode, keymod) else {
            return Ok(false);
        };

        let vim_mode = keymap_vim_mode(self.input_mode()?);

        if self.picker_visible()?
            && self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Popup, vim_mode, &chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Popup, vim_mode, &chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if self
            .runtime
            .keymaps()
            .contains_for_mode(&KeymapScope::Global, vim_mode, &chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Global, vim_mode, &chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if !self.picker_visible()?
            && !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
            && self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn sync_active_buffer(&mut self) -> Result<(), String> {
        sync_active_buffer(&mut self.runtime)
    }

    fn ensure_visible(&mut self, visible_lines: usize) -> Result<(), ShellError> {
        self.active_buffer_mut()?.ensure_visible(visible_lines);
        Ok(())
    }

    fn runtime_popup(&self) -> Result<Option<RuntimePopupSnapshot>, ShellError> {
        active_runtime_popup(&self.runtime).map_err(ShellError::Runtime)
    }

    fn mark_active_buffer_syntax_dirty(&mut self) -> Result<(), ShellError> {
        self.active_buffer_mut()?.mark_syntax_dirty();
        Ok(())
    }

    fn refresh_pending_syntax(&mut self) -> Result<(), ShellError> {
        refresh_pending_syntax(&mut self.runtime).map_err(ShellError::Runtime)
    }
}

/// Runs the SDL3 + SDL_ttf demo shell.
pub fn run_demo_shell(config: ShellConfig) -> Result<ShellSummary, ShellError> {
    let sdl_context = sdl3::init().map_err(|error| ShellError::Sdl(error.to_string()))?;
    let video = sdl_context
        .video()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let ttf = sdl3::ttf::init().map_err(|error| ShellError::Sdl(error.to_string()))?;

    let mut state = ShellState::new()?;
    let mut theme_settings =
        theme_runtime_settings(state.runtime.services().get::<ThemeRegistry>(), &config);
    let mut font_path = resolve_font_path(theme_settings.font_request.as_deref())?;
    let mut window_builder = video.window(&config.title, config.width, config.height);
    window_builder.position_centered().resizable();
    if config.hidden {
        window_builder.hidden();
    }
    let mut window = window_builder
        .build()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let icon = load_window_icon()?;
    if !window.set_icon(icon) {
        return Err(ShellError::Sdl(sdl3::get_error().to_string()));
    }
    video.text_input().start(&window);
    apply_window_opacity(&mut window, theme_settings.opacity, theme_settings.opacity_type)?;

    let mut font = ttf
        .load_font(&font_path, theme_settings.font_size as f32)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let mut line_height = font.height().max(1) as usize;
    let mut ascent = font.ascent();
    let mut cell_width = font
        .size_of_char('M')
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0
        .max(1) as i32;

    let mut canvas = window.into_canvas();
    let renderer_name = canvas.renderer_name.clone();
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let mut frames_rendered = 0;

    loop {
        update_theme_runtime(
            &mut canvas,
            &ttf,
            &state,
            &config,
            &mut theme_settings,
            &mut font,
            &mut font_path,
            &mut line_height,
            &mut ascent,
            &mut cell_width,
        )?;
        let (render_width, render_height) = canvas
            .output_size()
            .map_err(|error| ShellError::Sdl(error.to_string()))?;
        let visible_lines = (((render_height.saturating_sub(72)) as usize) / line_height).max(1);

        for event in event_pump.poll_iter() {
            if state.handle_event(event, visible_lines)? {
                return Ok(ShellSummary {
                    frames_rendered,
                    pane_count: state.pane_count()?,
                    popup_visible: state.popup_visible()?,
                    render_backend: RenderBackend::SdlCanvas,
                    renderer_name: renderer_name.clone(),
                    font_path: font_path.display().to_string(),
                });
            }
        }

        state.refresh_pending_syntax()?;

        let mut scene = Vec::new();
        state.render(
            &mut DrawTarget::Scene(&mut scene),
            &font,
            render_width,
            render_height,
            cell_width,
            line_height as i32,
            ascent,
        )?;
        present_scene_to_canvas(&mut canvas, &font, &scene)?;
        frames_rendered += 1;

        if let Some(frame_limit) = config.frame_limit
            && frames_rendered >= frame_limit
        {
            break;
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(ShellSummary {
        frames_rendered,
        pane_count: state.pane_count()?,
        popup_visible: state.popup_visible()?,
        render_backend: RenderBackend::SdlCanvas,
        renderer_name,
        font_path: font_path.display().to_string(),
    })
}

fn update_theme_runtime(
    canvas: &mut Canvas<Window>,
    ttf: &sdl3::ttf::Sdl3TtfContext,
    state: &ShellState,
    config: &ShellConfig,
    theme_settings: &mut ThemeRuntimeSettings,
    font: &mut Font<'_>,
    font_path: &mut PathBuf,
    line_height: &mut usize,
    ascent: &mut i32,
    cell_width: &mut i32,
) -> Result<(), ShellError> {
    let updated = theme_runtime_settings(state.runtime.services().get::<ThemeRegistry>(), config);
    if &updated == theme_settings {
        return Ok(());
    }

    if updated.font_size != theme_settings.font_size
        || updated.font_request != theme_settings.font_request
    {
        let next_font_path = resolve_font_path(updated.font_request.as_deref())?;
        let next_font = ttf
            .load_font(&next_font_path, updated.font_size as f32)
            .map_err(|error| ShellError::Sdl(error.to_string()))?;
        *font_path = next_font_path;
        *font = next_font;
        *line_height = font.height().max(1) as usize;
        *ascent = font.ascent();
        *cell_width = font
            .size_of_char('M')
            .map_err(|error| ShellError::Sdl(error.to_string()))?
            .0
            .max(1) as i32;
    }

    if updated.opacity != theme_settings.opacity
        || updated.opacity_type != theme_settings.opacity_type
    {
        apply_window_opacity(
            canvas.window_mut(),
            updated.opacity,
            updated.opacity_type,
        )?;
    }

    *theme_settings = updated;
    Ok(())
}

fn theme_runtime_settings(
    theme_registry: Option<&ThemeRegistry>,
    config: &ShellConfig,
) -> ThemeRuntimeSettings {
    let font_request = theme_registry
        .and_then(|registry| registry.resolve_string(OPTION_FONT))
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("default"))
        .map(str::to_owned);
    let font_size = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_FONT_SIZE))
        .map(|value| value.max(1.0).round() as u32)
        .unwrap_or(config.font_size);
    let opacity = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_OPACITY))
        .map(|value| value.clamp(0.0, 1.0) as f32)
        .unwrap_or(1.0);
    let opacity_type = theme_registry
        .and_then(|registry| registry.resolve_string(OPTION_OPACITY_TYPE))
        .and_then(OpacityType::from_value)
        .unwrap_or(OpacityType::Transparent);

    ThemeRuntimeSettings {
        font_request,
        font_size,
        opacity,
        opacity_type,
    }
}

fn resolve_font_path(request: Option<&str>) -> Result<PathBuf, ShellError> {
    if let Some(request) = request.and_then(|value| (!value.is_empty()).then_some(value)) {
        if let Some(path) = resolve_font_request(request) {
            return Ok(path);
        }
    }
    find_system_monospace_font().map_err(ShellError::from)
}

fn resolve_font_request(request: &str) -> Option<PathBuf> {
    let trimmed = request.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return path.exists().then(|| path.to_path_buf());
    }
    if path.extension().is_some() || trimmed.contains('/') || trimmed.contains('\\') {
        if let Ok(exe_path) = env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            let candidate = exe_dir.join(path);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        if path.exists() {
            return Some(path.to_path_buf());
        }
        return None;
    }
    find_font_by_name(trimmed)
}

fn apply_window_opacity(
    window: &mut Window,
    opacity: f32,
    opacity_type: OpacityType,
) -> Result<(), ShellError> {
    let opacity = match opacity_type {
        OpacityType::Transparent => opacity,
        // Keep window opacity at 1.0 for backdrops so text stays fully opaque.
        OpacityType::Win32Backdrop(_) => 1.0,
    };
    window
        .set_opacity(opacity)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    apply_win32_backdrop(window, opacity_type)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_win32_backdrop(window: &Window, opacity_type: OpacityType) -> Result<(), ShellError> {
    use windows_sys::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMSBT_AUTO, DWMSBT_MAINWINDOW, DWMSBT_NONE,
        DWMSBT_TABBEDWINDOW, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
    };

    let backdrop = match opacity_type {
        OpacityType::Transparent => Win32Backdrop::None,
        OpacityType::Win32Backdrop(backdrop) => backdrop,
    };
    let backdrop_value = match backdrop {
        Win32Backdrop::Auto => DWMSBT_AUTO,
        Win32Backdrop::None => DWMSBT_NONE,
        Win32Backdrop::Mica => DWMSBT_MAINWINDOW,
        // Mica Alt and tabbed both map to the tabbed system backdrop on Windows 11.
        Win32Backdrop::MicaAlt => DWMSBT_TABBEDWINDOW,
        Win32Backdrop::Acrylic => DWMSBT_TRANSIENTWINDOW,
        Win32Backdrop::Tabbed => DWMSBT_TABBEDWINDOW,
    };
    let Some(hwnd) = win32_hwnd(window) else {
        return Ok(());
    };
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop_value as *const _ as *const _,
            std::mem::size_of_val(&backdrop_value) as u32,
        )
    };
    if result != 0 {
        return Err(ShellError::Sdl(format!(
            "failed to set win32 backdrop: 0x{result:08X}",
            result = result as u32
        )));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn apply_win32_backdrop(_: &Window, _: OpacityType) -> Result<(), ShellError> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn win32_hwnd(window: &Window) -> Option<windows_sys::Win32::Foundation::HWND> {
    use sdl3::sys;
    use std::ptr;

    unsafe {
        let properties = sys::video::SDL_GetWindowProperties(window.raw());
        if properties == 0 {
            return None;
        }
        let hwnd = sys::properties::SDL_GetPointerProperty(
            properties,
            sys::video::SDL_PROP_WINDOW_WIN32_HWND_POINTER,
            ptr::null_mut(),
        );
        (!hwnd.is_null()).then_some(hwnd as windows_sys::Win32::Foundation::HWND)
    }
}

fn register_shell_hooks(runtime: &mut EditorRuntime) -> Result<(), String> {
    register_hook(runtime, HOOK_MOVE_LEFT, "Moves the active cursor left.")?;
    register_hook(runtime, HOOK_MOVE_DOWN, "Moves the active cursor down.")?;
    register_hook(runtime, HOOK_MOVE_UP, "Moves the active cursor up.")?;
    register_hook(runtime, HOOK_MOVE_RIGHT, "Moves the active cursor right.")?;
    register_hook(
        runtime,
        HOOK_MOVE_WORD_FORWARD,
        "Moves the active cursor to the next word.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_WORD_BACKWARD,
        "Moves the active cursor to the previous word.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_WORD_END,
        "Moves the active cursor to the end of the current or next word.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_BIG_WORD_FORWARD,
        "Moves the active cursor to the next Vim WORD.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_BIG_WORD_BACKWARD,
        "Moves the active cursor to the previous Vim WORD.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_BIG_WORD_END,
        "Moves the active cursor to the end of the current or next Vim WORD.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SENTENCE_FORWARD,
        "Moves the active cursor to the start of the next sentence.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SENTENCE_BACKWARD,
        "Moves the active cursor to the start of the current or previous sentence.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_PARAGRAPH_FORWARD,
        "Moves the active cursor to the start of the next paragraph.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_PARAGRAPH_BACKWARD,
        "Moves the active cursor to the start of the current or previous paragraph.",
    )?;
    register_hook(
        runtime,
        HOOK_MATCH_PAIR,
        "Moves the active cursor to the matching paired delimiter.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_LINE_START,
        "Moves to the start of the current line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_LINE_FIRST_NON_BLANK,
        "Moves to the first non-blank character on the current line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_LINE_END,
        "Moves to the end of the current line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SCREEN_TOP,
        "Moves to the first visible screen line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SCREEN_MIDDLE,
        "Moves to the middle visible screen line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SCREEN_BOTTOM,
        "Moves to the last visible screen line.",
    )?;
    register_hook(
        runtime,
        HOOK_GOTO_FIRST_LINE,
        "Moves to the first line in the buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_GOTO_LAST_LINE,
        "Moves to the last line in the buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_MODE_INSERT,
        "Switches the shell into insert mode.",
    )?;
    register_hook(
        runtime,
        HOOK_MODE_NORMAL,
        "Switches the shell into normal mode.",
    )?;
    register_hook(runtime, HOOK_VIM_EDIT, "Runs a Vim editing action.")?;
    register_hook(runtime, HOOK_BUFFER_SAVE, "Saves the active file buffer.")?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_SAVE,
        "Saves all modified file buffers in the active workspace.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_FORMAT,
        "Formats the active buffer or visual selection.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_FORMATTER_REGISTER,
        "Registers a language formatter for workspace.format.",
    )?;
    register_hook(runtime, HOOK_PICKER_OPEN, "Opens a named picker provider.")?;
    register_hook(
        runtime,
        HOOK_PICKER_NEXT,
        "Moves the picker selection down.",
    )?;
    register_hook(
        runtime,
        HOOK_PICKER_PREVIOUS,
        "Moves the picker selection up.",
    )?;
    register_hook(
        runtime,
        HOOK_PICKER_SUBMIT,
        "Executes the selected picker action.",
    )?;
    register_hook(runtime, HOOK_PICKER_CANCEL, "Closes the active picker.")?;
    register_hook(
        runtime,
        HOOK_SCROLL_HALF_PAGE_DOWN,
        "Scrolls down by half a page in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_HALF_PAGE_UP,
        "Scrolls up by half a page in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_PAGE_DOWN,
        "Scrolls down by a full page in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_PAGE_UP,
        "Scrolls up by a full page in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_LINE_DOWN,
        "Scrolls the viewport down by one line in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_LINE_UP,
        "Scrolls the viewport up by one line in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_POPUP_TOGGLE,
        "Shows or closes the docked popup window.",
    )?;

    runtime
        .subscribe_hook(HOOK_MOVE_LEFT, "shell.move-left", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::Left)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_DOWN, "shell.move-down", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::Down)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_UP, "shell.move-up", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::Up)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_RIGHT, "shell.move-right", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::Right)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_WORD_FORWARD,
            "shell.move-word-forward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::WordForward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_WORD_BACKWARD,
            "shell.move-word-backward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::WordBackward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_WORD_END, "shell.move-word-end", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::WordEnd)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_BIG_WORD_FORWARD,
            "shell.move-big-word-forward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::BigWordForward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_BIG_WORD_BACKWARD,
            "shell.move-big-word-backward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::BigWordBackward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_BIG_WORD_END,
            "shell.move-big-word-end",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::BigWordEnd)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SENTENCE_FORWARD,
            "shell.move-sentence-forward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::SentenceForward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SENTENCE_BACKWARD,
            "shell.move-sentence-backward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::SentenceBackward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_PARAGRAPH_FORWARD,
            "shell.move-paragraph-forward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ParagraphForward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_PARAGRAPH_BACKWARD,
            "shell.move-paragraph-backward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ParagraphBackward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MATCH_PAIR, "shell.match-pair", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::MatchPair)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_LINE_START,
            "shell.move-line-start",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::LineStart)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_LINE_FIRST_NON_BLANK,
            "shell.move-line-first-non-blank",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::LineFirstNonBlank)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_LINE_END, "shell.move-line-end", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::LineEnd)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SCREEN_TOP,
            "shell.move-screen-top",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ScreenTop)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SCREEN_MIDDLE,
            "shell.move-screen-middle",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ScreenMiddle)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SCREEN_BOTTOM,
            "shell.move-screen-bottom",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ScreenBottom)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_GOTO_FIRST_LINE,
            "shell.goto-first-line",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::FirstLine)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_GOTO_LAST_LINE, "shell.goto-last-line", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::LastLine)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SCROLL_HALF_PAGE_DOWN,
            "shell.scroll-half-page-down",
            |_, runtime| {
                apply_scroll_command(runtime, ScrollCommand::HalfPageDown)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SCROLL_HALF_PAGE_UP,
            "shell.scroll-half-page-up",
            |_, runtime| {
                apply_scroll_command(runtime, ScrollCommand::HalfPageUp)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SCROLL_PAGE_DOWN,
            "shell.scroll-page-down",
            |_, runtime| {
                apply_scroll_command(runtime, ScrollCommand::PageDown)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_SCROLL_PAGE_UP, "shell.scroll-page-up", |_, runtime| {
            apply_scroll_command(runtime, ScrollCommand::PageUp)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SCROLL_LINE_DOWN,
            "shell.scroll-line-down",
            |_, runtime| {
                apply_scroll_command(runtime, ScrollCommand::LineDown)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_SCROLL_LINE_UP, "shell.scroll-line-up", |_, runtime| {
            apply_scroll_command(runtime, ScrollCommand::LineUp)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MODE_INSERT, "shell.enter-insert-mode", |_, runtime| {
            start_change_recording(runtime)?;
            mark_change_finish_on_normal(runtime)?;
            shell_ui_mut(runtime)?.enter_insert_mode();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MODE_NORMAL, "shell.enter-normal-mode", |_, runtime| {
            let finish_change = {
                let vim = shell_ui(runtime)?.vim();
                vim.recording_change && vim.finish_change_on_normal
            };
            let visual_snapshot = {
                let (anchor, kind) = {
                    let ui = shell_ui(runtime)?;
                    if ui.input_mode() != InputMode::Visual {
                        (None, ui.vim().visual_kind)
                    } else {
                        (ui.vim().visual_anchor, ui.vim().visual_kind)
                    }
                };
                if let Some(anchor) = anchor {
                    let head = active_shell_buffer_mut(runtime)?.cursor_point();
                    Some((anchor, head, kind))
                } else {
                    None
                }
            };
            if matches!(
                shell_ui(runtime)?.input_mode(),
                InputMode::Insert | InputMode::Replace
            ) {
                let _ = active_shell_buffer_mut(runtime)?.move_left();
            }
            apply_pending_block_insert(runtime)?;
            shell_ui_mut(runtime)?.enter_normal_mode();
            if let Some((anchor, head, kind)) = visual_snapshot {
                store_last_visual_selection(runtime, anchor, head, kind)?;
            }
            if finish_change {
                finish_change_recording(runtime)?;
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_VIM_EDIT, "shell.vim-edit", |event, runtime| {
            let detail = event.detail.as_deref().unwrap_or_default();
            match detail {
                "delete-char" => {
                    delete_chars(runtime, false)?;
                }
                "delete-char-before" => {
                    delete_chars(runtime, true)?;
                }
                "delete-line-end" => {
                    start_change_recording(runtime)?;
                    apply_motion_alias(runtime, VimOperator::Delete, ShellMotion::LineEnd)?;
                }
                "change-line-end" => {
                    start_change_recording(runtime)?;
                    apply_motion_alias(runtime, VimOperator::Change, ShellMotion::LineEnd)?;
                }
                "yank-line" => {
                    let lines = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
                    apply_linewise_operator(runtime, VimOperator::Yank, lines)?;
                }
                "substitute-char" => {
                    substitute_chars(runtime)?;
                }
                "substitute-line" => {
                    let lines = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
                    apply_linewise_operator(runtime, VimOperator::Change, lines)?;
                }
                "replace-char" => {
                    start_replace_char(runtime)?;
                }
                "enter-replace-mode" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    shell_ui_mut(runtime)?.enter_replace_mode();
                }
                "toggle-case" => {
                    toggle_case_chars(runtime)?;
                }
                "append" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.append_after_cursor();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "append-line-end" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.append_line_end();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "insert-line-start" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.insert_line_start();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "open-line-below" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.open_line_below();
                    buffer.mark_syntax_dirty();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "open-line-above" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.open_line_above();
                    buffer.mark_syntax_dirty();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "undo" => {
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.undo();
                    buffer.mark_syntax_dirty();
                }
                "redo" => {
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.redo();
                    buffer.mark_syntax_dirty();
                }
                "enter-visual" => {
                    toggle_visual_mode(runtime)?;
                }
                "enter-visual-line" => {
                    toggle_visual_line_mode(runtime)?;
                }
                "enter-visual-block" => {
                    toggle_visual_block_mode(runtime)?;
                }
                "start-delete-operator" => {
                    start_vim_operator(runtime, VimOperator::Delete)?;
                }
                "start-change-operator" => {
                    start_vim_operator(runtime, VimOperator::Change)?;
                }
                "start-yank-operator" => {
                    start_vim_operator(runtime, VimOperator::Yank)?;
                }
                "start-format-operator" => {
                    start_vim_format(runtime)?;
                }
                "start-g-prefix" => {
                    start_vim_g_prefix(runtime)?;
                }
                "start-find-forward" => {
                    start_vim_find(runtime, VimFindKind::ForwardTo)?;
                }
                "start-find-backward" => {
                    start_vim_find(runtime, VimFindKind::BackwardTo)?;
                }
                "start-till-forward" => {
                    start_vim_find(runtime, VimFindKind::ForwardBefore)?;
                }
                "start-till-backward" => {
                    start_vim_find(runtime, VimFindKind::BackwardAfter)?;
                }
                "repeat-find-next" => {
                    repeat_last_find(runtime, false)?;
                }
                "repeat-find-previous" => {
                    repeat_last_find(runtime, true)?;
                }
                "start-search-forward" => {
                    open_vim_search_prompt(runtime, VimSearchDirection::Forward)?;
                }
                "start-search-backward" => {
                    open_vim_search_prompt(runtime, VimSearchDirection::Backward)?;
                }
                "search-word-forward" => {
                    search_word_under_cursor(runtime, VimSearchDirection::Forward)?;
                }
                "search-word-backward" => {
                    search_word_under_cursor(runtime, VimSearchDirection::Backward)?;
                }
                "repeat-search-next" => {
                    repeat_vim_search(runtime, false)?;
                }
                "repeat-search-previous" => {
                    repeat_vim_search(runtime, true)?;
                }
                "select-register" => {
                    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::Register);
                }
                "set-mark" => {
                    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::MarkSet);
                }
                "goto-mark-line" => {
                    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::MarkJump {
                        linewise: true,
                    });
                }
                "goto-mark" => {
                    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::MarkJump {
                        linewise: false,
                    });
                }
                "toggle-macro-record" => {
                    if shell_ui(runtime)?.vim().recording_macro.is_some() {
                        stop_macro_record(runtime)?;
                    } else {
                        shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::MacroRecord);
                    }
                }
                "start-macro-playback" => {
                    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::MacroPlayback);
                }
                "put-after" => {
                    put_yank(runtime, true)?;
                }
                "put-before" => {
                    put_yank(runtime, false)?;
                }
                "visual-swap-anchor" => {
                    swap_visual_anchor(runtime)?;
                }
                "start-visual-inner-text-object" => {
                    start_visual_text_object(runtime, false)?;
                }
                "start-visual-around-text-object" => {
                    start_visual_text_object(runtime, true)?;
                }
                "visual-delete" => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::Delete)?;
                }
                "visual-change" => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::Change)?;
                }
                "visual-block-insert" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    start_visual_block_insert(runtime, false)?;
                }
                "visual-block-append" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    start_visual_block_insert(runtime, true)?;
                }
                "visual-format" => {
                    start_change_recording(runtime)?;
                    emit_workspace_format(runtime)?;
                }
                "visual-yank" => {
                    apply_visual_operator(runtime, VimOperator::Yank)?;
                }
                "visual-toggle-case" => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::ToggleCase)?;
                }
                "visual-lowercase" => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::Lowercase)?;
                }
                "visual-uppercase" => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::Uppercase)?;
                }
                _ => {}
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_FORMATTER_REGISTER,
            "shell.formatter-register",
            |event, runtime| {
                let detail = event
                    .detail
                    .as_deref()
                    .ok_or_else(|| "formatter registration hook missing detail".to_owned())?;
                let spec = FormatterSpec::from_hook_detail(detail)?;
                formatter_registry_mut(runtime)?.register(spec)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_WORKSPACE_FORMAT, "shell.workspace-format", |_, runtime| {
            format_workspace(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BUFFER_SAVE, "shell.buffer-save", |event, runtime| {
            let workspace_id = event
                .workspace_id
                .or_else(|| runtime.model().active_workspace_id().ok())
                .ok_or_else(|| "buffer.save hook missing workspace".to_owned())?;
            let buffer_id = event
                .buffer_id
                .unwrap_or(active_shell_buffer_id(runtime)?);
            save_buffer(runtime, workspace_id, buffer_id)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_WORKSPACE_SAVE, "shell.workspace-save", |event, runtime| {
            let workspace_id = event
                .workspace_id
                .or_else(|| runtime.model().active_workspace_id().ok())
                .ok_or_else(|| "workspace.save hook missing workspace".to_owned())?;
            save_workspace(runtime, workspace_id)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_OPEN, "shell.open-picker", |event, runtime| {
            let picker = picker_overlay(runtime, event.detail.as_deref().unwrap_or("commands"))?;
            shell_ui_mut(runtime)?.set_picker(picker);
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_NEXT, "shell.picker-next", |_, runtime| {
            if let Some(picker) = shell_ui_mut(runtime)?.picker_mut() {
                picker.session.select_next();
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PICKER_PREVIOUS,
            "shell.picker-previous",
            |_, runtime| {
                if let Some(picker) = shell_ui_mut(runtime)?.picker_mut() {
                    picker.session.select_previous();
                }
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_CANCEL, "shell.picker-cancel", |_, runtime| {
            shell_ui_mut(runtime)?.close_picker();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_POPUP_TOGGLE, "shell.popup-toggle", |_, runtime| {
            toggle_runtime_popup(runtime)?;
            sync_active_buffer(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_SUBMIT, "shell.picker-submit", |_, runtime| {
            let (action, query) = {
                let ui = shell_ui_mut(runtime)?;
                let action = ui
                    .picker()
                    .and_then(PickerOverlay::selected_action)
                    .ok_or_else(|| "picker has no selected item".to_owned())?;
                let query = ui
                    .picker()
                    .map(|picker| picker.session().query().to_owned())
                    .unwrap_or_default();
                ui.close_picker();
                (action, query)
            };

            match action {
                PickerAction::NoOp => {}
                PickerAction::ExecuteCommand(command_name) => {
                    runtime
                        .execute_command(&command_name)
                        .map_err(|error| error.to_string())?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::FocusBuffer(buffer_id) => {
                    let workspace_id = runtime
                        .model()
                        .active_workspace_id()
                        .map_err(|error| error.to_string())?;
                    runtime
                        .model_mut()
                        .focus_buffer(workspace_id, buffer_id)
                        .map_err(|error| error.to_string())?;
                    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
                    sync_active_buffer(runtime)?;
                }
                PickerAction::OpenFile(path) => {
                    open_workspace_file(runtime, &path)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::ActivateTheme(theme_id) => {
                    let registry = runtime
                        .services_mut()
                        .get_mut::<ThemeRegistry>()
                        .ok_or_else(|| "theme registry service missing".to_owned())?;
                    registry.activate(&theme_id).map_err(|error| error.to_string())?;
                }
                PickerAction::VimSearch(direction) => {
                    submit_vim_search(runtime, direction, &query)?;
                }
                PickerAction::VimSearchResult { direction, target } => {
                    apply_vim_search_result(runtime, direction, target, &query)?;
                }
                PickerAction::InstallTreeSitterLanguage(language_id) => {
                    install_tree_sitter_language(runtime, &language_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::CreateWorkspace { name, root } => {
                    open_workspace_from_project(runtime, &name, &root)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::SwitchWorkspace(workspace_id) => {
                    switch_runtime_workspace(runtime, workspace_id)?;
                }
                PickerAction::DeleteWorkspace(workspace_id) => {
                    delete_runtime_workspace(runtime, workspace_id)?;
                    sync_active_buffer(runtime)?;
                }
            }

            Ok(())
        })
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn load_window_icon() -> Result<sdl3::surface::Surface<'static>, ShellError> {
    let image = image::load_from_memory_with_format(WINDOW_ICON_BYTES, image::ImageFormat::Png)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let row_bytes = width as usize * 4;
    // ABGR8888 maps to RGBA byte order on little-endian, matching image::Rgba8 output.
    let mut surface = sdl3::surface::Surface::new(width, height, PixelFormat::ABGR8888)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let pitch = surface.pitch() as usize;
    if pitch < row_bytes {
        return Err(ShellError::Sdl(format!(
            "icon pitch {pitch} is smaller than row width {row_bytes}"
        )));
    }
    let raw = rgba.into_raw();
    surface.with_lock_mut(|buffer| {
        for row in 0..height as usize {
            let src_start = row * row_bytes;
            let dst_start = row * pitch;
            buffer[dst_start..dst_start + row_bytes]
                .copy_from_slice(&raw[src_start..src_start + row_bytes]);
        }
    });
    Ok(surface)
}

fn register_lsp_status_hooks(runtime: &mut EditorRuntime) -> Result<(), String> {
    if !runtime.hooks().contains("lsp.server-start") {
        return Ok(());
    }

    runtime
        .subscribe_hook(
            "lsp.server-start",
            "shell.track-lsp-server",
            |event, runtime| {
                let workspace_id = event
                    .workspace_id
                    .or_else(|| runtime.model().active_workspace_id().ok())
                    .ok_or_else(|| "lsp status hook is missing a workspace".to_owned())?;
                shell_ui_mut(runtime)?.set_attached_lsp_server(workspace_id, event.detail.clone());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())
}

fn register_hook(runtime: &mut EditorRuntime, name: &str, description: &str) -> Result<(), String> {
    runtime
        .register_hook(name, description)
        .map_err(|error| error.to_string())
}

fn ensure_picker_keybindings(runtime: &mut EditorRuntime) -> Result<(), String> {
    let bindings = [
        ("F3", "picker.open-commands"),
        ("F4", "picker.open-buffers"),
        ("F5", "picker.toggle-popup-window"),
        ("F6", "picker.open-keybindings"),
    ];

    for (chord, command) in bindings {
        if !runtime.commands().contains(command) {
            continue;
        }
        if runtime.keymaps().contains(&KeymapScope::Global, chord) {
            continue;
        }
        runtime
            .register_key_binding(
                chord,
                command,
                KeymapScope::Global,
                CommandSource::UserPackage("picker".to_owned()),
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn shell_ui(runtime: &EditorRuntime) -> Result<&ShellUiState, String> {
    runtime
        .services()
        .get::<ShellUiState>()
        .ok_or_else(|| "shell UI state service missing".to_owned())
}

fn shell_ui_mut(runtime: &mut EditorRuntime) -> Result<&mut ShellUiState, String> {
    runtime
        .services_mut()
        .get_mut::<ShellUiState>()
        .ok_or_else(|| "shell UI state service missing".to_owned())
}

fn vim_count_digit(chord: &str, has_existing_count: bool) -> Option<usize> {
    let mut characters = chord.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }
    (character.is_ascii_digit() && (character != '0' || has_existing_count))
        .then(|| character.to_digit(10))
        .flatten()
        .map(|digit| digit as usize)
}

fn active_shell_buffer_id(runtime: &EditorRuntime) -> Result<BufferId, String> {
    if let Some(popup) = active_runtime_popup(runtime)? {
        return Ok(popup.active_buffer);
    }

    shell_ui(runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active shell buffer is missing".to_owned())
}

fn active_shell_buffer_mut(runtime: &mut EditorRuntime) -> Result<&mut ShellBuffer, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    shell_ui_mut(runtime)?
        .buffer_mut(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())
}

fn reverse_find_kind(kind: VimFindKind) -> VimFindKind {
    match kind {
        VimFindKind::ForwardTo => VimFindKind::BackwardTo,
        VimFindKind::BackwardTo => VimFindKind::ForwardTo,
        VimFindKind::ForwardBefore => VimFindKind::BackwardAfter,
        VimFindKind::BackwardAfter => VimFindKind::ForwardBefore,
    }
}

fn reverse_search_direction(direction: VimSearchDirection) -> VimSearchDirection {
    match direction {
        VimSearchDirection::Forward => VimSearchDirection::Backward,
        VimSearchDirection::Backward => VimSearchDirection::Forward,
    }
}

fn vim_delimited_text_object(chord: &str) -> Option<(char, char)> {
    match chord {
        "(" | ")" | "b" => Some(('(', ')')),
        "[" | "]" => Some(('[', ']')),
        "{" | "}" | "B" => Some(('{', '}')),
        ">" => Some(('<', '>')),
        "\"" => Some(('"', '"')),
        "'" => Some(('\'', '\'')),
        "`" => Some(('`', '`')),
        _ => None,
    }
}

fn vim_text_object_kind(chord: &str) -> Option<VimTextObjectKind> {
    match chord {
        "w" => Some(VimTextObjectKind::Word),
        "W" => Some(VimTextObjectKind::BigWord),
        "s" => Some(VimTextObjectKind::Sentence),
        "p" => Some(VimTextObjectKind::Paragraph),
        "t" => Some(VimTextObjectKind::Tag),
        _ => vim_delimited_text_object(chord)
            .map(|(open, close)| VimTextObjectKind::Delimited { open, close }),
    }
}

fn search_is_case_sensitive(_query: &str) -> bool {
    false
}

fn normalize_search_char(ch: char, case_sensitive: bool) -> char {
    if case_sensitive {
        ch
    } else {
        ch.to_ascii_lowercase()
    }
}

fn normalize_search_pattern(query: &str, case_sensitive: bool) -> Vec<char> {
    query
        .chars()
        .map(|ch| normalize_search_char(ch, case_sensitive))
        .collect()
}

fn matches_pattern_at(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
) -> bool {
    pattern.iter().enumerate().all(|(offset, expected)| {
        buffer
            .text
            .char_at_point(buffer.text.point_from_char_index(start_char + offset))
            .map(|ch| normalize_search_char(ch, case_sensitive))
            == Some(*expected)
    })
}

fn search_forward(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
    wrap: bool,
) -> Option<TextPoint> {
    if pattern.is_empty() || pattern.len() > buffer.text.char_count() {
        return None;
    }

    let max_start = buffer.text.char_count().saturating_sub(pattern.len());
    let first_pass_start = start_char.min(max_start.saturating_add(1));
    for char_index in first_pass_start..=max_start {
        if matches_pattern_at(buffer, char_index, pattern, case_sensitive) {
            return Some(buffer.text.point_from_char_index(char_index));
        }
    }

    if wrap {
        for char_index in 0..first_pass_start.min(max_start.saturating_add(1)) {
            if matches_pattern_at(buffer, char_index, pattern, case_sensitive) {
                return Some(buffer.text.point_from_char_index(char_index));
            }
        }
    }

    None
}

fn search_backward(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
    wrap: bool,
) -> Option<TextPoint> {
    if pattern.is_empty() || pattern.len() > buffer.text.char_count() {
        return None;
    }

    let max_start = buffer.text.char_count().saturating_sub(pattern.len());
    let first_pass_start = start_char.min(max_start);
    for char_index in (0..=first_pass_start).rev() {
        if matches_pattern_at(buffer, char_index, pattern, case_sensitive) {
            return Some(buffer.text.point_from_char_index(char_index));
        }
    }

    if wrap && first_pass_start < max_start {
        for char_index in ((first_pass_start + 1)..=max_start).rev() {
            if matches_pattern_at(buffer, char_index, pattern, case_sensitive) {
                return Some(buffer.text.point_from_char_index(char_index));
            }
        }
    }

    None
}

fn char_at_index(buffer: &ShellBuffer, char_index: usize) -> Option<char> {
    buffer
        .text
        .char_at_point(buffer.text.point_from_char_index(char_index))
}

fn find_char_forward(
    buffer: &ShellBuffer,
    start_char: usize,
    target: char,
    case_sensitive: bool,
) -> Option<usize> {
    let char_count = buffer.text.char_count();
    for char_index in start_char..char_count {
        if char_at_index(buffer, char_index)
            .map(|ch| normalize_search_char(ch, case_sensitive))
            == Some(target)
        {
            return Some(char_index);
        }
    }
    None
}

fn fuzzy_match_end(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
) -> Option<usize> {
    if pattern.is_empty()
        || char_at_index(buffer, start_char)
            .map(|ch| normalize_search_char(ch, case_sensitive))
            != Some(pattern[0])
    {
        return None;
    }

    let mut last_index = start_char;
    let mut next_index = start_char.saturating_add(1);
    for target in pattern.iter().skip(1) {
        let Some(found) = find_char_forward(buffer, next_index, *target, case_sensitive) else {
            return None;
        };
        last_index = found;
        next_index = found.saturating_add(1);
    }

    Some(last_index)
}

fn search_fuzzy_forward(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
    wrap: bool,
) -> Option<TextPoint> {
    let char_count = buffer.text.char_count();
    if pattern.is_empty() || pattern.len() > char_count {
        return None;
    }

    let max_start = char_count.saturating_sub(1);
    let first_pass_start = start_char.min(max_start.saturating_add(1));
    let mut best: Option<(usize, usize)> = None;

    if first_pass_start <= max_start {
        for char_index in first_pass_start..=max_start {
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive) else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.map_or(true, |(_, best_span)| span < best_span) {
                best = Some((char_index, span));
            }
        }
    }
    if best.is_some() {
        return best.map(|(start, _)| buffer.text.point_from_char_index(start));
    }

    if wrap {
        for char_index in 0..first_pass_start.min(max_start.saturating_add(1)) {
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive) else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.map_or(true, |(_, best_span)| span < best_span) {
                best = Some((char_index, span));
            }
        }
    }

    best.map(|(start, _)| buffer.text.point_from_char_index(start))
}

fn search_fuzzy_backward(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
    wrap: bool,
) -> Option<TextPoint> {
    let char_count = buffer.text.char_count();
    if pattern.is_empty() || pattern.len() > char_count {
        return None;
    }

    let max_start = char_count.saturating_sub(1);
    let first_pass_start = start_char.min(max_start);
    let mut best: Option<(usize, usize)> = None;

    for char_index in (0..=first_pass_start).rev() {
        let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive) else {
            continue;
        };
        let span = end_index.saturating_sub(char_index);
        if best.map_or(true, |(_, best_span)| span < best_span) {
            best = Some((char_index, span));
        }
    }
    if best.is_some() {
        return best.map(|(start, _)| buffer.text.point_from_char_index(start));
    }

    if wrap && first_pass_start < max_start {
        for char_index in ((first_pass_start + 1)..=max_start).rev() {
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive) else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.map_or(true, |(_, best_span)| span < best_span) {
                best = Some((char_index, span));
            }
        }
    }

    best.map(|(start, _)| buffer.text.point_from_char_index(start))
}

fn search_buffer(
    buffer: &ShellBuffer,
    direction: VimSearchDirection,
    query: &str,
) -> Option<TextPoint> {
    let case_sensitive = search_is_case_sensitive(query);
    let pattern = normalize_search_pattern(query, case_sensitive);
    if pattern.is_empty() {
        return None;
    }

    let cursor = buffer.cursor_point();
    let exact_match = match direction {
        VimSearchDirection::Forward => {
            let start_char = buffer
                .point_after(cursor)
                .map(|point| buffer.text.point_to_char_index(point))
                .unwrap_or(buffer.text.char_count());
            search_forward(buffer, start_char, &pattern, case_sensitive, true)
        }
        VimSearchDirection::Backward => {
            let start_char = buffer
                .text
                .point_before(cursor)
                .map(|point| buffer.text.point_to_char_index(point))
                .unwrap_or_else(|| buffer.text.char_count().saturating_sub(pattern.len()));
            search_backward(buffer, start_char, &pattern, case_sensitive, true)
        }
    };

    if exact_match.is_some() {
        return exact_match;
    }

    match direction {
        VimSearchDirection::Forward => {
            let start_char = buffer
                .point_after(cursor)
                .map(|point| buffer.text.point_to_char_index(point))
                .unwrap_or(buffer.text.char_count());
            search_fuzzy_forward(buffer, start_char, &pattern, case_sensitive, true)
        }
        VimSearchDirection::Backward => {
            let start_char = buffer
                .text
                .point_before(cursor)
                .map(|point| buffer.text.point_to_char_index(point))
                .unwrap_or_else(|| buffer.text.char_count().saturating_sub(pattern.len()));
            search_fuzzy_backward(buffer, start_char, &pattern, case_sensitive, true)
        }
    }
}

#[derive(Debug, Clone)]
struct VimSearchMatch {
    point: TextPoint,
    char_index: usize,
    span: usize,
    line_text: String,
}

#[derive(Debug, Clone)]
struct SearchPickerData {
    entries: Vec<PickerEntry>,
    selected_index: usize,
}

fn exact_match_positions_in_chars(
    chars: &[char],
    pattern: &[char],
    case_sensitive: bool,
) -> Vec<usize> {
    if pattern.is_empty() || pattern.len() > chars.len() {
        return Vec::new();
    }

    let max_start = chars.len().saturating_sub(pattern.len());
    let mut matches = Vec::new();
    for start in 0..=max_start {
        if pattern
            .iter()
            .enumerate()
            .all(|(offset, expected)| {
                normalize_search_char(chars[start + offset], case_sensitive) == *expected
            })
        {
            matches.push(start);
        }
    }
    matches
}

fn fuzzy_match_end_in_chars(
    chars: &[char],
    start: usize,
    pattern: &[char],
    case_sensitive: bool,
) -> Option<usize> {
    if pattern.is_empty()
        || chars
            .get(start)
            .copied()
            .map(|ch| normalize_search_char(ch, case_sensitive))
            != Some(pattern[0])
    {
        return None;
    }

    let mut last_index = start;
    let mut next_index = start.saturating_add(1);
    for target in pattern.iter().skip(1) {
        let Some(found) = chars
            .get(next_index..)
            .and_then(|slice| {
                slice
                    .iter()
                    .position(|ch| normalize_search_char(*ch, case_sensitive) == *target)
            })
            .map(|offset| next_index + offset)
        else {
            return None;
        };
        last_index = found;
        next_index = found.saturating_add(1);
    }
    Some(last_index)
}

fn fuzzy_match_positions_in_chars(
    chars: &[char],
    pattern: &[char],
    case_sensitive: bool,
) -> Vec<(usize, usize)> {
    if pattern.is_empty() || pattern.len() > chars.len() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    for start in 0..chars.len() {
        if let Some(end) = fuzzy_match_end_in_chars(chars, start, pattern, case_sensitive) {
            matches.push((start, end.saturating_sub(start)));
        }
    }
    matches
}

fn search_start_char(buffer: &ShellBuffer, direction: VimSearchDirection, pattern_len: usize) -> usize {
    let cursor = buffer.cursor_point();
    match direction {
        VimSearchDirection::Forward => buffer
            .point_after(cursor)
            .map(|point| buffer.text.point_to_char_index(point))
            .unwrap_or(buffer.text.char_count()),
        VimSearchDirection::Backward => buffer
            .text
            .point_before(cursor)
            .map(|point| buffer.text.point_to_char_index(point))
            .unwrap_or_else(|| buffer.text.char_count().saturating_sub(pattern_len)),
    }
}

fn pick_search_selection_index(
    matches: &[VimSearchMatch],
    direction: VimSearchDirection,
    start_char: usize,
) -> usize {
    if matches.is_empty() {
        return 0;
    }

    let mut candidates: Vec<(usize, &VimSearchMatch)> = matches
        .iter()
        .enumerate()
        .filter(|(_, matched)| match direction {
            VimSearchDirection::Forward => matched.char_index >= start_char,
            VimSearchDirection::Backward => matched.char_index <= start_char,
        })
        .collect();

    if candidates.is_empty() {
        candidates = matches.iter().enumerate().collect();
    }

    candidates
        .into_iter()
        .min_by(|(_, left), (_, right)| {
            let span_order = left.span.cmp(&right.span);
            if span_order != std::cmp::Ordering::Equal {
                return span_order;
            }
            match direction {
                VimSearchDirection::Forward => left.char_index.cmp(&right.char_index),
                VimSearchDirection::Backward => right.char_index.cmp(&left.char_index),
            }
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn vim_search_entries(
    buffer: &ShellBuffer,
    direction: VimSearchDirection,
    query: &str,
) -> SearchPickerData {
    let query = query.trim();
    if query.is_empty() {
        return SearchPickerData {
            entries: Vec::new(),
            selected_index: 0,
        };
    }

    let case_sensitive = search_is_case_sensitive(query);
    let pattern = normalize_search_pattern(query, case_sensitive);
    let line_count = buffer.text.line_count();
    let mut matches = Vec::new();

    for line_index in 0..line_count {
        let Some(line) = buffer.text.line(line_index) else {
            continue;
        };
        let chars: Vec<char> = line.chars().collect();
        let positions = exact_match_positions_in_chars(&chars, &pattern, case_sensitive);
        for start in positions {
            let point = TextPoint::new(line_index, start);
            let char_index = buffer.text.point_to_char_index(point);
            matches.push(VimSearchMatch {
                point,
                char_index,
                span: pattern.len().saturating_sub(1),
                line_text: line.clone(),
            });
        }
    }

    if matches.is_empty() {
        for line_index in 0..line_count {
        let Some(line) = buffer.text.line(line_index) else {
            continue;
        };
        let chars: Vec<char> = line.chars().collect();
        let positions = fuzzy_match_positions_in_chars(&chars, &pattern, case_sensitive);
        for (start, span) in positions {
            let point = TextPoint::new(line_index, start);
            let char_index = buffer.text.point_to_char_index(point);
            matches.push(VimSearchMatch {
                    point,
                    char_index,
                    span,
                    line_text: line.clone(),
                });
            }
        }
    }

    matches.sort_by_key(|matched| (matched.point.line, matched.point.column));

    if matches.len() > SEARCH_PICKER_ITEM_LIMIT {
        matches.truncate(SEARCH_PICKER_ITEM_LIMIT);
    }

    let start_char = search_start_char(buffer, direction, pattern.len());
    let selected_index = pick_search_selection_index(&matches, direction, start_char);

    let entries = matches
        .into_iter()
        .map(|matched| {
            let detail = format!(
                "Ln {}, Col {}",
                matched.point.line + 1,
                matched.point.column + 1
            );
            PickerEntry {
                item: PickerItem::new(
                    format!("{}:{}", matched.point.line, matched.point.column),
                    matched.line_text,
                    detail,
                    None::<String>,
                ),
                action: PickerAction::VimSearchResult {
                    direction,
                    target: matched.point,
                },
            }
        })
        .collect();

    SearchPickerData {
        entries,
        selected_index,
    }
}

fn move_buffer_with_motion(
    buffer: &mut ShellBuffer,
    motion: ShellMotion,
    count: Option<usize>,
) -> bool {
    let repeat = count.unwrap_or(1).max(1);
    match motion {
        ShellMotion::Left => (0..repeat).fold(false, |moved, _| buffer.move_left() || moved),
        ShellMotion::Down => (0..repeat).fold(false, |moved, _| buffer.move_down() || moved),
        ShellMotion::Up => (0..repeat).fold(false, |moved, _| buffer.move_up() || moved),
        ShellMotion::Right => (0..repeat).fold(false, |moved, _| buffer.move_right() || moved),
        ShellMotion::WordForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_forward() || moved)
        }
        ShellMotion::BigWordForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_forward() || moved)
        }
        ShellMotion::WordBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_backward() || moved)
        }
        ShellMotion::BigWordBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_backward() || moved)
        }
        ShellMotion::WordEnd => (0..repeat).fold(false, |moved, _| buffer.move_word_end() || moved),
        ShellMotion::BigWordEnd => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_end() || moved)
        }
        ShellMotion::SentenceForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_sentence_forward() || moved)
        }
        ShellMotion::SentenceBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_sentence_backward() || moved)
        }
        ShellMotion::ParagraphForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_paragraph_forward() || moved)
        }
        ShellMotion::ParagraphBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_paragraph_backward() || moved)
        }
        ShellMotion::WordEndBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_backward() || moved)
        }
        ShellMotion::BigWordEndBackward => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_backward() || moved
        }),
        ShellMotion::MatchPair => buffer.move_matching_delimiter(),
        ShellMotion::LineStart => buffer.move_line_start(),
        ShellMotion::LineFirstNonBlank => buffer.move_line_first_non_blank(),
        ShellMotion::LineEnd => {
            let line_repeat = repeat.saturating_sub(1);
            let moved_line = if line_repeat == 0 {
                false
            } else {
                (0..line_repeat).fold(false, |moved, _| buffer.move_down() || moved)
            };
            buffer.move_line_end() || moved_line
        }
        ShellMotion::ScreenTop => buffer.move_to_viewport_offset(repeat.saturating_sub(1)),
        ShellMotion::ScreenMiddle => buffer.move_to_viewport_middle(),
        ShellMotion::ScreenBottom => {
            let viewport = buffer.viewport_lines();
            let offset = viewport.saturating_sub(repeat.min(viewport));
            buffer.move_to_viewport_offset(offset)
        }
        ShellMotion::FirstLine => {
            if let Some(line) = count {
                buffer.goto_line(line.saturating_sub(1))
            } else {
                buffer.goto_first_line()
            }
        }
        ShellMotion::LastLine => {
            if let Some(line) = count {
                buffer.goto_line(line.saturating_sub(1))
            } else {
                buffer.goto_last_line()
            }
        }
    }
}

fn motion_is_inclusive(motion: ShellMotion) -> bool {
    matches!(
        motion,
        ShellMotion::WordEnd
            | ShellMotion::BigWordEnd
            | ShellMotion::WordEndBackward
            | ShellMotion::BigWordEndBackward
            | ShellMotion::MatchPair
            | ShellMotion::LineEnd
    )
}

fn charwise_motion_range(
    buffer: &ShellBuffer,
    start: TextPoint,
    target: TextPoint,
    inclusive: bool,
) -> Option<TextRange> {
    let range = if target >= start {
        let end = if inclusive {
            buffer.point_after(target).unwrap_or(target)
        } else {
            target
        };
        TextRange::new(start, end)
    } else {
        let end = if inclusive {
            buffer.point_after(start).unwrap_or(start)
        } else {
            start
        };
        TextRange::new(target, end)
    };
    (range.start() != range.end()).then_some(range.normalized())
}

fn visual_selection(
    buffer: &ShellBuffer,
    anchor: TextPoint,
    kind: VisualSelectionKind,
) -> Option<VisualSelection> {
    let head = buffer.cursor_point();
    match kind {
        VisualSelectionKind::Character => {
            let range = if head >= anchor {
                TextRange::new(anchor, buffer.point_after(head).unwrap_or(head))
            } else {
                TextRange::new(head, buffer.point_after(anchor).unwrap_or(anchor))
            };
            (range.start() != range.end()).then_some(VisualSelection::Range(range.normalized()))
        }
        VisualSelectionKind::Line => {
            let start_line = anchor.line.min(head.line);
            let line_count = anchor.line.max(head.line).saturating_sub(start_line) + 1;
            buffer
                .line_span_range(start_line, line_count)
                .map(VisualSelection::Range)
        }
        VisualSelectionKind::Block => {
            let end_col = anchor.column.max(head.column).saturating_add(1);
            Some(VisualSelection::Block(BlockSelection {
                start_line: anchor.line.min(head.line),
                end_line: anchor.line.max(head.line),
                start_col: anchor.column.min(head.column),
                end_col,
            }))
        }
    }
}

fn block_selection_ranges(buffer: &ShellBuffer, selection: BlockSelection) -> Vec<TextRange> {
    (selection.start_line..=selection.end_line)
        .filter_map(|line_index| {
            let line_len = buffer.line_len_chars(line_index);
            let start = selection.start_col.min(line_len);
            let end = selection.end_col.min(line_len);
            (start < end).then(|| {
                TextRange::new(
                    TextPoint::new(line_index, start),
                    TextPoint::new(line_index, end),
                )
            })
        })
        .collect()
}

fn line_text_without_newline(buffer: &ShellBuffer, line_index: usize) -> Option<String> {
    if line_index >= buffer.line_count() {
        return None;
    }
    let line_len = buffer.line_len_chars(line_index);
    Some(buffer.slice(TextRange::new(
        TextPoint::new(line_index, 0),
        TextPoint::new(line_index, line_len),
    )))
}

fn resolve_block_insert_text(original: &str, current: &str, insert_col: usize) -> String {
    let original_chars: Vec<char> = original.chars().collect();
    let current_chars: Vec<char> = current.chars().collect();
    let prefix_len = insert_col
        .min(original_chars.len())
        .min(current_chars.len());
    if original_chars[..prefix_len] != current_chars[..prefix_len] {
        return current_chars[prefix_len..].iter().collect();
    }
    let suffix = &original_chars[prefix_len..];
    if current_chars.len() >= prefix_len + suffix.len() {
        let suffix_start = current_chars.len() - suffix.len();
        if current_chars[suffix_start..] == *suffix {
            return current_chars[prefix_len..suffix_start].iter().collect();
        }
    }
    current_chars[prefix_len..].iter().collect()
}

fn prepare_block_insert_state(
    runtime: &mut EditorRuntime,
    selection: BlockSelection,
    insert_col: usize,
    origin_line: usize,
) -> Result<(), String> {
    let original_line = {
        let buffer = active_shell_buffer_mut(runtime)?;
        line_text_without_newline(buffer, origin_line)
            .ok_or_else(|| "block insert origin line is missing".to_owned())?
    };
    shell_ui_mut(runtime)?.vim_mut().block_insert = Some(BlockInsertState {
        selection,
        insert_col,
        origin_line,
        original_line,
    });
    Ok(())
}

fn apply_pending_block_insert(runtime: &mut EditorRuntime) -> Result<(), String> {
    let pending = shell_ui_mut(runtime)?.vim_mut().block_insert.take();
    let Some(pending) = pending else {
        return Ok(());
    };
    let origin_line = pending.origin_line;
    let original_line = pending.original_line;
    let insert_col = pending.insert_col;
    let selection = pending.selection;
    let buffer = active_shell_buffer_mut(runtime)?;
    let Some(current_line) = line_text_without_newline(buffer, origin_line) else {
        return Err("block insert origin line is missing".to_owned());
    };
    let origin_col = insert_col.min(original_line.chars().count());
    let inserted = resolve_block_insert_text(&original_line, &current_line, origin_col);
    if inserted.is_empty() {
        return Ok(());
    }
    let cursor = buffer.cursor_point();
    for line in (selection.start_line..=selection.end_line).rev() {
        if line == origin_line || line >= buffer.line_count() {
            continue;
        }
        let target_col = insert_col.min(buffer.line_len_chars(line));
        buffer.insert_at(TextPoint::new(line, target_col), &inserted);
    }
    buffer.set_cursor(cursor);
    buffer.mark_syntax_dirty();
    Ok(())
}

fn start_visual_block_insert(runtime: &mut EditorRuntime, append: bool) -> Result<(), String> {
    let (selection, insert_col, origin_line) = {
        let ui = shell_ui(runtime)?;
        let anchor = ui
            .vim()
            .visual_anchor
            .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
        let buffer = ui
            .buffer(active_shell_buffer_id(runtime)?)
            .ok_or_else(|| "active visual buffer is missing".to_owned())?;
        let selection = match visual_selection(buffer, anchor, ui.vim().visual_kind) {
            Some(VisualSelection::Block(block)) => block,
            _ => return Err("visual block insert requires block selection".to_owned()),
        };
        let insert_col = if append {
            selection.end_col
        } else {
            selection.start_col
        };
        (selection, insert_col, selection.start_line)
    };
    {
        let buffer = active_shell_buffer_mut(runtime)?;
        let line_len = buffer.line_len_chars(origin_line);
        let target_col = insert_col.min(line_len);
        buffer.set_cursor(TextPoint::new(origin_line, target_col));
    }
    prepare_block_insert_state(runtime, selection, insert_col, origin_line)?;
    shell_ui_mut(runtime)?.enter_insert_mode();
    Ok(())
}

fn line_flash_selection_for_range(
    buffer: &ShellBuffer,
    range: TextRange,
) -> Option<VisualSelection> {
    let range = range.normalized();
    let line_count = range.end().line.saturating_sub(range.start().line) + 1;
    buffer
        .line_span_range(range.start().line, line_count)
        .map(VisualSelection::Range)
}

fn transform_case_text(text: &str, operator: VimOperator) -> String {
    text.chars()
        .map(|character| match operator {
            VimOperator::ToggleCase => {
                if character.is_lowercase() {
                    character.to_uppercase().collect::<String>()
                } else if character.is_uppercase() {
                    character.to_lowercase().collect::<String>()
                } else {
                    character.to_string()
                }
            }
            VimOperator::Lowercase => character.to_lowercase().collect::<String>(),
            VimOperator::Uppercase => character.to_uppercase().collect::<String>(),
            _ => character.to_string(),
        })
        .collect()
}

fn store_yank_register(runtime: &mut EditorRuntime, yank: YankRegister) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    vim.yank = Some(yank.clone());
    if let Some(register) = vim.active_register.take() {
        vim.registers.insert(register, yank);
    }
    Ok(())
}

fn start_change_recording(runtime: &mut EditorRuntime) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    if vim.replaying {
        return Ok(());
    }
    if !vim.recording_change {
        vim.recording_change = true;
        vim.change_buffer.clear();
    }
    Ok(())
}

fn start_change_recording_with_prefix(
    runtime: &mut EditorRuntime,
    prefix: Option<VimRecordedInput>,
) -> Result<(), String> {
    start_change_recording(runtime)?;
    if let Some(input) = prefix {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        if vim.change_buffer.is_empty() {
            vim.change_buffer.push(input);
        }
    }
    Ok(())
}

fn mark_change_finish_on_normal(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.vim_mut().finish_change_on_normal = true;
    Ok(())
}

fn schedule_finish_change(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.vim_mut().finish_change_after_input = true;
    Ok(())
}

fn finish_change_recording(runtime: &mut EditorRuntime) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    if vim.recording_change {
        if !vim.change_buffer.is_empty() {
            vim.last_change = vim.change_buffer.clone();
        }
        vim.change_buffer.clear();
        vim.recording_change = false;
        vim.finish_change_on_normal = false;
        vim.finish_change_after_input = false;
    }
    Ok(())
}

fn start_macro_record(runtime: &mut EditorRuntime, register: char) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    vim.recording_macro = Some(register);
    vim.macro_buffer.clear();
    vim.skip_next_macro_input = true;
    vim.clear_transient();
    Ok(())
}

fn stop_macro_record(runtime: &mut EditorRuntime) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    if let Some(register) = vim.recording_macro.take() {
        let recorded = std::mem::take(&mut vim.macro_buffer);
        vim.macros.insert(register, recorded);
    }
    vim.clear_transient();
    Ok(())
}

fn store_last_visual_selection(
    runtime: &mut EditorRuntime,
    anchor: TextPoint,
    head: TextPoint,
    kind: VisualSelectionKind,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    shell_ui_mut(runtime)?.vim_mut().last_visual = Some(VimVisualSnapshot {
        buffer_id,
        anchor,
        head,
        kind,
    });
    Ok(())
}

fn restore_last_visual_selection(runtime: &mut EditorRuntime) -> Result<(), String> {
    let snapshot = shell_ui(runtime)?.vim().last_visual;
    let Some(snapshot) = snapshot else {
        return Ok(());
    };
    let ui = shell_ui_mut(runtime)?;
    ui.focus_buffer(snapshot.buffer_id);
    let buffer = ui
        .buffer_mut(snapshot.buffer_id)
        .ok_or_else(|| "visual buffer is missing".to_owned())?;
    buffer.set_cursor(snapshot.head);
    ui.enter_visual_mode(snapshot.anchor, snapshot.kind);
    Ok(())
}

fn jump_to_mark(
    runtime: &mut EditorRuntime,
    mark: char,
    linewise: bool,
) -> Result<(), String> {
    let snapshot = shell_ui(runtime)?.vim().marks.get(&mark).copied();
    let Some(snapshot) = snapshot else {
        return Ok(());
    };
    let ui = shell_ui_mut(runtime)?;
    ui.focus_buffer(snapshot.buffer_id);
    let buffer = ui
        .buffer_mut(snapshot.buffer_id)
        .ok_or_else(|| "mark buffer is missing".to_owned())?;
    if linewise {
        buffer.goto_line(snapshot.point.line);
    } else {
        buffer.set_cursor(snapshot.point);
    }
    ui.vim_mut().clear_transient();
    Ok(())
}

fn apply_operator_to_range(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    range: TextRange,
    linewise: bool,
    original_cursor: TextPoint,
    flash_selection: Option<VisualSelection>,
) -> Result<(), String> {
    let removed = active_shell_buffer_mut(runtime)?.slice(range);
    if removed.is_empty() {
        shell_ui_mut(runtime)?.enter_normal_mode();
        return Ok(());
    }

    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        let yank = if linewise {
            YankRegister::Line(removed.clone())
        } else {
            YankRegister::Character(removed.clone())
        };
        store_yank_register(runtime, yank)?;
    }

    match operator {
        VimOperator::Delete => {
            let buffer = active_shell_buffer_mut(runtime)?;
            buffer.delete_range(range);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            schedule_finish_change(runtime)?;
        }
        VimOperator::Change => {
            let buffer = active_shell_buffer_mut(runtime)?;
            if linewise && removed.ends_with('\n') {
                buffer.replace_range(range, "\n");
                buffer.set_cursor(range.start());
            } else {
                buffer.delete_range(range);
            }
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_insert_mode();
            mark_change_finish_on_normal(runtime)?;
        }
        VimOperator::Yank => {
            if let Some(selection) = flash_selection {
                let buffer_id = active_shell_buffer_id(runtime)?;
                shell_ui_mut(runtime)?.set_yank_flash(buffer_id, selection);
            }
            active_shell_buffer_mut(runtime)?.set_cursor(original_cursor);
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
            let buffer = active_shell_buffer_mut(runtime)?;
            let replaced = transform_case_text(&removed, operator);
            buffer.replace_range(range, &replaced);
            buffer.set_cursor(original_cursor);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            schedule_finish_change(runtime)?;
        }
    }

    Ok(())
}

fn apply_block_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    selection: BlockSelection,
    original_cursor: TextPoint,
    flash_selection: Option<VisualSelection>,
) -> Result<(), String> {
    let (ranges, yanked) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let ranges = block_selection_ranges(buffer, selection);
        let yanked = ranges
            .iter()
            .map(|range| buffer.slice(*range))
            .collect::<Vec<_>>();
        (ranges, yanked)
    };
    if ranges.is_empty() {
        shell_ui_mut(runtime)?.enter_normal_mode();
        return Ok(());
    }

    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        store_yank_register(runtime, YankRegister::Block(yanked))?;
    }
    let target_cursor = ranges[0].start();

    match operator {
        VimOperator::Delete => {
            let buffer = active_shell_buffer_mut(runtime)?;
            for range in ranges.iter().rev().copied() {
                buffer.delete_range(range);
            }
            buffer.set_cursor(target_cursor);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            schedule_finish_change(runtime)?;
        }
        VimOperator::Change => {
            {
                let buffer = active_shell_buffer_mut(runtime)?;
                for range in ranges.iter().rev().copied() {
                    buffer.delete_range(range);
                }
                buffer.set_cursor(target_cursor);
                buffer.mark_syntax_dirty();
            }
            prepare_block_insert_state(runtime, selection, selection.start_col, target_cursor.line)?;
            shell_ui_mut(runtime)?.enter_insert_mode();
            mark_change_finish_on_normal(runtime)?;
        }
        VimOperator::Yank => {
            if let Some(selection) = flash_selection {
                let buffer_id = active_shell_buffer_id(runtime)?;
                shell_ui_mut(runtime)?.set_yank_flash(buffer_id, selection);
            }
            active_shell_buffer_mut(runtime)?.set_cursor(original_cursor);
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
            let buffer = active_shell_buffer_mut(runtime)?;
            for range in ranges.iter().copied() {
                let removed = buffer.slice(range);
                let replaced = transform_case_text(&removed, operator);
                buffer.replace_range(range, &replaced);
            }
            buffer.set_cursor(original_cursor);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            schedule_finish_change(runtime)?;
        }
    }

    Ok(())
}

fn apply_visual_operator(runtime: &mut EditorRuntime, operator: VimOperator) -> Result<(), String> {
    let (selection, cursor, kind, anchor) = {
        let ui = shell_ui(runtime)?;
        let anchor = ui
            .vim()
            .visual_anchor
            .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
        let kind = ui.vim().visual_kind;
        let buffer = shell_ui(runtime)?
            .buffer(active_shell_buffer_id(runtime)?)
            .ok_or_else(|| "active visual buffer is missing".to_owned())?;
        (
            visual_selection(buffer, anchor, kind)
                .ok_or_else(|| "visual selection is empty".to_owned())?,
            buffer.cursor_point(),
            kind,
            anchor,
        )
    };
    store_last_visual_selection(runtime, anchor, cursor, kind)?;

    match selection {
        VisualSelection::Range(range) => apply_operator_to_range(
            runtime,
            operator,
            range,
            matches!(kind, VisualSelectionKind::Line),
            cursor,
            (operator == VimOperator::Yank).then_some(selection),
        ),
        VisualSelection::Block(block) => apply_block_operator(
            runtime,
            operator,
            block,
            cursor,
            (operator == VimOperator::Yank).then_some(selection),
        ),
    }
}

fn emit_workspace_format(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    runtime
        .emit_hook(
            HOOK_WORKSPACE_FORMAT,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_buffer(buffer_id),
        )
        .map_err(|error| error.to_string())
}

fn save_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
) -> Result<(), String> {
    let (path, buffer_kind) = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let buffer = workspace
            .buffer(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        (buffer.path().map(Path::to_path_buf), buffer.kind().clone())
    };

    if buffer_kind != BufferKind::File {
        return Ok(());
    }

    let path = path.ok_or_else(|| "buffer.save requires a file path".to_owned())?;
    runtime
        .emit_hook(
            builtins::BEFORE_SAVE,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_buffer(buffer_id)
                .with_detail(path.display().to_string()),
        )
        .map_err(|error| error.to_string())?;

    {
        let buffer = shell_ui_mut(runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))?;
        buffer
            .save_to_path(&path)
            .map_err(|error| format!("failed to save `{}`: {error}", path.display()))?;
    }

    runtime
        .emit_hook(
            builtins::AFTER_SAVE,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_buffer(buffer_id)
                .with_detail(path.display().to_string()),
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn save_workspace(runtime: &mut EditorRuntime, workspace_id: WorkspaceId) -> Result<(), String> {
    let buffer_ids = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        workspace.buffers().map(Buffer::id).collect::<Vec<_>>()
    };

    for buffer_id in buffer_ids {
        let (buffer_kind, path) = {
            let workspace = runtime
                .model()
                .workspace(workspace_id)
                .map_err(|error| error.to_string())?;
            let buffer = workspace
                .buffer(buffer_id)
                .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
            (buffer.kind().clone(), buffer.path().map(Path::to_path_buf))
        };

        if buffer_kind != BufferKind::File {
            continue;
        }

        let is_dirty = {
            let ui = shell_ui(runtime)?;
            let buffer = ui
                .buffer(buffer_id)
                .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))?;
            buffer.is_dirty()
        };

        if !is_dirty {
            continue;
        }

        let path = path.ok_or_else(|| format!("file buffer `{buffer_id}` is missing a path"))?;
        save_buffer(runtime, workspace_id, buffer_id).map_err(|error| {
            format!("failed to save `{}`: {error}", path.display())
        })?;
    }

    Ok(())
}

fn format_workspace(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (path, extension, original_cursor, selection, buffer_kind) = {
        let ui = shell_ui(runtime)?;
        let buffer = ui
            .buffer(buffer_id)
            .ok_or_else(|| "active buffer is missing".to_owned())?;
        let path = buffer
            .path()
            .ok_or_else(|| "active buffer does not have a file path".to_owned())?;
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_owned);
        let original_cursor = buffer.cursor_point();
        let selection = if ui.input_mode() == InputMode::Visual {
            let anchor = ui
                .vim()
                .visual_anchor
                .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
            let kind = ui.vim().visual_kind;
            let selection = visual_selection(buffer, anchor, kind)
                .ok_or_else(|| "visual selection is empty".to_owned())?;
            Some((selection, anchor, original_cursor, kind))
        } else {
            None
        };
        (path.to_path_buf(), extension, original_cursor, selection, buffer.kind.clone())
    };

    if buffer_kind != BufferKind::File {
        return Err("workspace.format only supports file buffers".to_owned());
    }

    let formatter = formatter_for_path(runtime, &path)?;
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .or_else(|| active_workspace_root(runtime).ok().flatten());

    start_change_recording(runtime)?;

    if let Some((selection, anchor, head, kind)) = selection {
        store_last_visual_selection(runtime, anchor, head, kind)?;
        format_visual_selection(
            runtime,
            &formatter,
            selection,
            extension.as_deref(),
            cwd.as_deref(),
            original_cursor,
        )?;
    } else {
        format_entire_buffer(
            runtime,
            &formatter,
            extension.as_deref(),
            cwd.as_deref(),
            original_cursor,
        )?;
    }

    Ok(())
}

fn formatter_for_path(runtime: &EditorRuntime, path: &Path) -> Result<FormatterSpec, String> {
    let syntax = runtime
        .services()
        .get::<SyntaxRegistry>()
        .ok_or_else(|| "syntax registry service missing".to_owned())?;
    let language = syntax
        .language_for_path(path)
        .ok_or_else(|| format!("no syntax language registered for `{}`", path.display()))?;
    let formatter = formatter_registry(runtime)?
        .formatter_for_language(language.id())
        .ok_or_else(|| format!("no formatter registered for language `{}`", language.id()))?;
    Ok(formatter.clone())
}

fn format_entire_buffer(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    extension: Option<&str>,
    cwd: Option<&Path>,
    original_cursor: TextPoint,
) -> Result<(), String> {
    let input = {
        let buffer = active_shell_buffer_mut(runtime)?;
        buffer.text.text()
    };
    let formatted = format_text_with_formatter(runtime, formatter, &input, extension, cwd)?;
    {
        let buffer = active_shell_buffer_mut(runtime)?;
        if formatted != input {
            let end = buffer.text.point_from_char_index(buffer.text.char_count());
            buffer.replace_range(TextRange::new(TextPoint::default(), end), &formatted);
            buffer.mark_syntax_dirty();
        }
        buffer.set_cursor(original_cursor);
    }
    shell_ui_mut(runtime)?.enter_normal_mode();
    schedule_finish_change(runtime)?;
    Ok(())
}

fn format_visual_selection(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    selection: VisualSelection,
    extension: Option<&str>,
    cwd: Option<&Path>,
    original_cursor: TextPoint,
) -> Result<(), String> {
    match selection {
        VisualSelection::Range(range) => {
            format_range_with_formatter(runtime, formatter, range, extension, cwd)?;
        }
        VisualSelection::Block(block) => {
            format_block_with_formatter(runtime, formatter, block, extension, cwd)?;
        }
    }
    let buffer = active_shell_buffer_mut(runtime)?;
    buffer.set_cursor(original_cursor);
    buffer.mark_syntax_dirty();
    shell_ui_mut(runtime)?.enter_normal_mode();
    schedule_finish_change(runtime)?;
    Ok(())
}

fn format_range_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    range: TextRange,
    extension: Option<&str>,
    cwd: Option<&Path>,
) -> Result<(), String> {
    let input = {
        let buffer = active_shell_buffer_mut(runtime)?;
        buffer.slice(range)
    };
    let formatted = format_text_with_formatter(runtime, formatter, &input, extension, cwd)?;
    active_shell_buffer_mut(runtime)?.replace_range(range, &formatted);
    Ok(())
}

fn format_block_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    selection: BlockSelection,
    extension: Option<&str>,
    cwd: Option<&Path>,
) -> Result<(), String> {
    let (ranges, snippets) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let ranges = block_selection_ranges(buffer, selection);
        let snippets = ranges
            .iter()
            .map(|range| buffer.slice(*range))
            .collect::<Vec<_>>();
        (ranges, snippets)
    };

    if ranges.is_empty() {
        return Ok(());
    }

    let mut replacements = Vec::with_capacity(snippets.len());
    for snippet in snippets {
        let formatted = format_text_with_formatter(runtime, formatter, &snippet, extension, cwd)?;
        let formatted = normalize_block_output(&formatted)?;
        replacements.push(formatted);
    }

    let buffer = active_shell_buffer_mut(runtime)?;
    for index in (0..ranges.len()).rev() {
        buffer.replace_range(ranges[index], &replacements[index]);
    }

    Ok(())
}

fn normalize_block_output(formatted: &str) -> Result<String, String> {
    let trimmed = formatted.trim_end_matches(['\n', '\r']);
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err("formatter output spans multiple lines for a block selection".to_owned());
    }
    Ok(trimmed.to_owned())
}

fn format_text_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    input: &str,
    extension: Option<&str>,
    cwd: Option<&Path>,
) -> Result<String, String> {
    let temp_path = formatter_temp_path(extension);
    fs::write(&temp_path, input).map_err(|error| {
        format!(
            "failed to write formatter input `{}`: {error}",
            temp_path.display()
        )
    })?;

    let mut args = formatter.args.clone();
    args.push(temp_path.to_string_lossy().into_owned());
    let mut spec = JobSpec::command(
        format!("format-{}", formatter.language_id),
        formatter.program.clone(),
        args,
    );
    if let Some(cwd) = cwd {
        spec = spec.with_cwd(cwd.to_path_buf());
    }

    let manager = runtime
        .services()
        .get::<Mutex<JobManager>>()
        .ok_or_else(|| "job manager service missing".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "job manager lock poisoned".to_owned())?;
    let handle = manager.spawn(spec).map_err(|error| error.to_string())?;
    drop(manager);
    let result = handle.wait().map_err(|error| error.to_string())?;

    if !result.succeeded() {
        cleanup_formatter_temp(&temp_path);
        return Err(format!(
            "formatter `{}` failed: {}",
            formatter.program,
            result.transcript()
        ));
    }

    let formatted = fs::read_to_string(&temp_path).map_err(|error| {
        format!(
            "failed to read formatter output `{}`: {error}",
            temp_path.display()
        )
    })?;
    cleanup_formatter_temp(&temp_path);
    Ok(formatted)
}

fn formatter_temp_path(extension: Option<&str>) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    let mut filename = format!("volt-format-{}-{unique}", std::process::id());
    if let Some(extension) = extension.filter(|extension| !extension.is_empty()) {
        filename.push('.');
        filename.push_str(extension);
    }
    std::env::temp_dir().join(filename)
}

fn cleanup_formatter_temp(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        eprintln!(
            "failed to remove formatter temp file `{}`: {error}",
            path.display()
        );
    }
}

fn apply_linewise_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    line_count: usize,
) -> Result<(), String> {
    let (range, original_cursor, flash_selection) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let original_cursor = buffer.cursor_point();
        let range = buffer
            .line_span_range(buffer.cursor_row(), line_count.max(1))
            .ok_or_else(|| "linewise Vim range could not be resolved".to_owned())?;
        (range, original_cursor, Some(VisualSelection::Range(range)))
    };
    apply_operator_to_range(
        runtime,
        operator,
        range,
        true,
        original_cursor,
        flash_selection,
    )
}

fn apply_text_object_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    kind: VimTextObjectKind,
    around: bool,
    count: usize,
) -> Result<(), String> {
    let (range, original_cursor, flash_selection) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let original_cursor = buffer.cursor_point();
        let range = buffer
            .text_object_range(kind, around, count.max(1))
            .ok_or_else(|| "text object is unavailable at the current cursor".to_owned())?;
        (
            range,
            original_cursor,
            line_flash_selection_for_range(buffer, range),
        )
    };
    apply_operator_to_range(
        runtime,
        operator,
        range,
        false,
        original_cursor,
        flash_selection,
    )
}

fn apply_motion_alias(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    motion: ShellMotion,
) -> Result<(), String> {
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    apply_operator_motion(runtime, operator, count, motion, None)
}

fn apply_visual_text_object(
    runtime: &mut EditorRuntime,
    kind: VimTextObjectKind,
    around: bool,
    count: usize,
) -> Result<(), String> {
    let (anchor, head) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let range = buffer
            .text_object_range(kind, around, count.max(1))
            .ok_or_else(|| "text object is unavailable at the current cursor".to_owned())?;
        let head = buffer
            .text
            .point_before(range.end())
            .unwrap_or(range.start());
        (range.start(), head)
    };
    active_shell_buffer_mut(runtime)?.set_cursor(head);
    shell_ui_mut(runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);
    Ok(())
}

fn delete_chars(runtime: &mut EditorRuntime, backward: bool) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    let motion = if backward {
        ShellMotion::Left
    } else {
        ShellMotion::Right
    };
    apply_operator_motion(runtime, VimOperator::Delete, count, motion, Some(1))
}

fn substitute_chars(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    apply_operator_motion(
        runtime,
        VimOperator::Change,
        count,
        ShellMotion::Right,
        Some(1),
    )
}

fn start_replace_char(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::ReplaceChar { count });
    Ok(())
}

fn toggle_case_chars(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    let (range, end_point) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let Some(range) = range_for_char_count(buffer, count) else {
            return Ok(());
        };
        range
    };
    let buffer = active_shell_buffer_mut(runtime)?;
    let removed = buffer.slice(range);
    let replaced = transform_case_text(&removed, VimOperator::ToggleCase);
    buffer.replace_range(range, &replaced);
    buffer.set_cursor(end_point);
    buffer.mark_syntax_dirty();
    shell_ui_mut(runtime)?.enter_normal_mode();
    schedule_finish_change(runtime)?;
    Ok(())
}

fn range_for_char_count(buffer: &ShellBuffer, count: usize) -> Option<(TextRange, TextPoint)> {
    let start = buffer.cursor_point();
    let mut end = start;
    for _ in 0..count.max(1) {
        let next = buffer.point_after(end)?;
        if buffer.slice(TextRange::new(end, next)) == "\n" {
            break;
        }
        end = next;
    }
    (end != start).then_some((TextRange::new(start, end), end))
}

fn apply_operator_motion(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    operator_count: usize,
    motion: ShellMotion,
    motion_count: Option<usize>,
) -> Result<(), String> {
    let (range, linewise, original_cursor, flash_selection) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let original_cursor = buffer.cursor_point();
        let range = match motion {
            ShellMotion::Down => {
                let line_count = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .saturating_add(1);
                buffer.line_span_range(buffer.cursor_row(), line_count)
            }
            ShellMotion::Up => {
                let line_count = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .saturating_add(1);
                let start_line = buffer
                    .cursor_row()
                    .saturating_sub(line_count.saturating_sub(1));
                Some(TextRange::new(
                    buffer
                        .line_range(start_line)
                        .ok_or_else(|| "up motion start line is unavailable".to_owned())?
                        .start(),
                    buffer
                        .line_range(buffer.cursor_row())
                        .ok_or_else(|| "up motion end line is unavailable".to_owned())?
                        .end(),
                ))
            }
            ShellMotion::FirstLine => {
                let target_line = motion_count.unwrap_or(1).saturating_sub(1);
                let start_line = target_line.min(buffer.cursor_row());
                let end_line = target_line.max(buffer.cursor_row());
                Some(TextRange::new(
                    buffer
                        .line_range(start_line)
                        .ok_or_else(|| "first-line range start is unavailable".to_owned())?
                        .start(),
                    buffer
                        .line_range(end_line)
                        .ok_or_else(|| "first-line range end is unavailable".to_owned())?
                        .end(),
                ))
            }
            ShellMotion::LastLine => {
                let target_line = motion_count
                    .map(|line| line.saturating_sub(1))
                    .unwrap_or(buffer.line_count().saturating_sub(1));
                let start_line = target_line.min(buffer.cursor_row());
                let end_line = target_line.max(buffer.cursor_row());
                Some(TextRange::new(
                    buffer
                        .line_range(start_line)
                        .ok_or_else(|| "last-line range start is unavailable".to_owned())?
                        .start(),
                    buffer
                        .line_range(end_line)
                        .ok_or_else(|| "last-line range end is unavailable".to_owned())?
                        .end(),
                ))
            }
            _ => {
                let repeat = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .max(1);
                if !move_buffer_with_motion(buffer, motion, Some(repeat)) {
                    None
                } else {
                    let target = buffer.cursor_point();
                    let range = charwise_motion_range(
                        buffer,
                        original_cursor,
                        target,
                        motion_is_inclusive(motion),
                    );
                    buffer.set_cursor(original_cursor);
                    range
                }
            }
        };
        let range =
            range.ok_or_else(|| "Vim operator motion did not resolve a range".to_owned())?;
        (
            range,
            matches!(
                motion,
                ShellMotion::Down
                    | ShellMotion::Up
                    | ShellMotion::FirstLine
                    | ShellMotion::LastLine
            ),
            original_cursor,
            line_flash_selection_for_range(buffer, range),
        )
    };

    apply_operator_to_range(
        runtime,
        operator,
        range,
        linewise,
        original_cursor,
        flash_selection,
    )
}

fn apply_motion_command(runtime: &mut EditorRuntime, motion: ShellMotion) -> Result<(), String> {
    let pending_operator = match shell_ui(runtime)?.vim().pending {
        Some(VimPending::Operator { operator, count }) => Some((operator, count)),
        _ => None,
    };

    if let Some((operator, count)) = pending_operator {
        let motion_count = shell_ui_mut(runtime)?.vim_mut().take_count();
        return apply_operator_motion(runtime, operator, count, motion, motion_count);
    }

    let count = shell_ui_mut(runtime)?.vim_mut().take_count();
    move_buffer_with_motion(active_shell_buffer_mut(runtime)?, motion, count);
    Ok(())
}

fn apply_scroll_command(runtime: &mut EditorRuntime, command: ScrollCommand) -> Result<(), String> {
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    let buffer = active_shell_buffer_mut(runtime)?;
    let viewport = buffer.viewport_lines().max(1);
    match command {
        ScrollCommand::HalfPageDown => {
            scroll_buffer_with_cursor(buffer, ((viewport / 2).max(1) * count) as i32);
            Ok(())
        }
        ScrollCommand::HalfPageUp => {
            scroll_buffer_with_cursor(buffer, -(((viewport / 2).max(1) * count) as i32));
            Ok(())
        }
        ScrollCommand::PageDown => {
            scroll_buffer_with_cursor(buffer, (viewport * count) as i32);
            Ok(())
        }
        ScrollCommand::PageUp => {
            scroll_buffer_with_cursor(buffer, -((viewport * count) as i32));
            Ok(())
        }
        ScrollCommand::LineDown => {
            scroll_buffer_viewport_only(buffer, count as i32);
            Ok(())
        }
        ScrollCommand::LineUp => {
            scroll_buffer_viewport_only(buffer, -(count as i32));
            Ok(())
        }
    }
}

fn scroll_buffer_with_cursor(buffer: &mut ShellBuffer, delta: i32) {
    let screen_offset = buffer.cursor_row().saturating_sub(buffer.scroll_row);
    buffer.scroll_by(delta);
    let target_line = buffer.line_at_viewport_offset(screen_offset);
    let _ = buffer.goto_line(target_line);
}

fn scroll_buffer_viewport_only(buffer: &mut ShellBuffer, delta: i32) {
    buffer.scroll_by(delta);
    let top = buffer.scroll_row;
    let bottom = buffer.line_at_viewport_offset(buffer.viewport_lines().saturating_sub(1));
    if buffer.cursor_row() < top {
        let _ = buffer.goto_line(top);
    } else if buffer.cursor_row() > bottom {
        let _ = buffer.goto_line(bottom);
    }
}

fn resolve_find_target(
    runtime: &mut EditorRuntime,
    operator: Option<VimOperator>,
    kind: VimFindKind,
    count: usize,
    target: char,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.vim_mut().last_find = Some(LastFind { kind, target });

    if let Some(operator) = operator {
        let (range, original_cursor, flash_selection) = {
            let buffer = active_shell_buffer_mut(runtime)?;
            let original_cursor = buffer.cursor_point();
            if !buffer.move_find(kind, target, count.max(1)) {
                shell_ui_mut(runtime)?.enter_normal_mode();
                return Ok(());
            }
            let moved_to = buffer.cursor_point();
            let range = charwise_motion_range(
                buffer,
                original_cursor,
                moved_to,
                matches!(kind, VimFindKind::ForwardTo | VimFindKind::BackwardTo),
            )
            .ok_or_else(|| "find motion did not resolve a Vim range".to_owned())?;
            buffer.set_cursor(original_cursor);
            (
                range,
                original_cursor,
                line_flash_selection_for_range(buffer, range),
            )
        };
        apply_operator_to_range(
            runtime,
            operator,
            range,
            false,
            original_cursor,
            flash_selection,
        )?;
    } else {
        active_shell_buffer_mut(runtime)?.move_find(kind, target, count.max(1));
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
    }

    Ok(())
}

fn repeat_last_find(runtime: &mut EditorRuntime, reverse: bool) -> Result<(), String> {
    let last_find = shell_ui(runtime)?
        .vim()
        .last_find
        .ok_or_else(|| "no previous Vim find motion is available".to_owned())?;
    let kind = if reverse {
        reverse_find_kind(last_find.kind)
    } else {
        last_find.kind
    };

    let pending_operator = match shell_ui(runtime)?.vim().pending {
        Some(VimPending::Operator { operator, count }) => Some((operator, count)),
        _ => None,
    };
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    if let Some((operator, operator_count)) = pending_operator {
        resolve_find_target(
            runtime,
            Some(operator),
            kind,
            operator_count.saturating_mul(count).max(1),
            last_find.target,
        )
    } else {
        resolve_find_target(runtime, None, kind, count, last_find.target)
    }
}

fn open_vim_search_prompt(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
) -> Result<(), String> {
    let title = match direction {
        VimSearchDirection::Forward => "Search /",
        VimSearchDirection::Backward => "Search ?",
    };
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::search(
        title,
        direction,
        Vec::new(),
    ));
    Ok(())
}

fn run_vim_search(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
    query: &str,
) -> Result<(), String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(());
    }

    let target = {
        let buffer = active_shell_buffer_mut(runtime)?;
        search_buffer(buffer, direction, query)
            .ok_or_else(|| format!("no matches found for `{query}`"))?
    };
    active_shell_buffer_mut(runtime)?.set_cursor(target);
    shell_ui_mut(runtime)?.vim_mut().last_search = Some(LastSearch {
        direction,
        query: query.to_owned(),
    });
    shell_ui_mut(runtime)?.vim_mut().clear_transient();
    Ok(())
}

fn apply_vim_search_result(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
    target: TextPoint,
    query: &str,
) -> Result<(), String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(());
    }

    active_shell_buffer_mut(runtime)?.set_cursor(target);
    shell_ui_mut(runtime)?.vim_mut().last_search = Some(LastSearch {
        direction,
        query: query.to_owned(),
    });
    shell_ui_mut(runtime)?.vim_mut().clear_transient();
    Ok(())
}

fn search_word_under_cursor(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
) -> Result<(), String> {
    let query = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let range = buffer
            .text_object_range(VimTextObjectKind::Word, false, 1)
            .ok_or_else(|| "no Vim word is available at the current cursor".to_owned())?;
        buffer.slice(range)
    };
    run_vim_search(runtime, direction, &query)
}

fn submit_vim_search(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
    query: &str,
) -> Result<(), String> {
    if !query.trim().is_empty() {
        return run_vim_search(runtime, direction, query);
    }

    let last_search = shell_ui(runtime)?
        .vim()
        .last_search
        .clone()
        .ok_or_else(|| "no previous Vim search is available".to_owned())?;
    run_vim_search(runtime, direction, &last_search.query)
}

fn repeat_vim_search(runtime: &mut EditorRuntime, reverse: bool) -> Result<(), String> {
    let last_search = shell_ui(runtime)?
        .vim()
        .last_search
        .clone()
        .ok_or_else(|| "no previous Vim search is available".to_owned())?;
    let direction = if reverse {
        reverse_search_direction(last_search.direction)
    } else {
        last_search.direction
    };
    run_vim_search(runtime, direction, &last_search.query)
}

fn resolve_g_prefix(
    runtime: &mut EditorRuntime,
    operator: Option<VimOperator>,
    line_target: Option<usize>,
    chord: &str,
) -> Result<(), String> {
    match chord {
        "g" => {
            if let Some(operator) = operator {
                let (range, original_cursor, flash_selection) = {
                    let buffer = active_shell_buffer_mut(runtime)?;
                    let original_cursor = buffer.cursor_point();
                    let target_line = line_target.unwrap_or(1).saturating_sub(1);
                    let start_line = target_line.min(buffer.cursor_row());
                    let end_line = target_line.max(buffer.cursor_row());
                    let range = TextRange::new(
                        buffer
                            .line_range(start_line)
                            .ok_or_else(|| "gg range start is unavailable".to_owned())?
                            .start(),
                        buffer
                            .line_range(end_line)
                            .ok_or_else(|| "gg range end is unavailable".to_owned())?
                            .end(),
                    );
                    (range, original_cursor, Some(VisualSelection::Range(range)))
                };
                apply_operator_to_range(
                    runtime,
                    operator,
                    range,
                    true,
                    original_cursor,
                    flash_selection,
                )
            } else {
                let target_line = line_target.unwrap_or(1).saturating_sub(1);
                active_shell_buffer_mut(runtime)?.goto_line(target_line);
                shell_ui_mut(runtime)?.vim_mut().clear_transient();
                Ok(())
            }
        }
        "e" | "E" => {
            let motion = if chord == "e" {
                ShellMotion::WordEndBackward
            } else {
                ShellMotion::BigWordEndBackward
            };
            if let Some(operator) = operator {
                let motion_count = line_target;
                let operator_count = 1;
                apply_operator_motion(runtime, operator, operator_count, motion, motion_count)
            } else {
                move_buffer_with_motion(active_shell_buffer_mut(runtime)?, motion, line_target);
                shell_ui_mut(runtime)?.vim_mut().clear_transient();
                Ok(())
            }
        }
        _ => {
            shell_ui_mut(runtime)?.vim_mut().clear_transient();
            Ok(())
        }
    }
}

fn start_vim_operator(runtime: &mut EditorRuntime, operator: VimOperator) -> Result<(), String> {
    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::ToggleCase | VimOperator::Lowercase
            | VimOperator::Uppercase
    ) {
        start_change_recording(runtime)?;
    }
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::Operator { operator, count });
    Ok(())
}

fn start_vim_format(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::Format { count });
    Ok(())
}

fn start_vim_find(runtime: &mut EditorRuntime, kind: VimFindKind) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    let pending_operator = match ui.vim().pending {
        Some(VimPending::Operator { operator, count }) => Some((operator, count)),
        _ => None,
    };
    let count = ui.vim_mut().take_count_or_one();
    ui.vim_mut().pending = Some(VimPending::FindTarget {
        operator: pending_operator.map(|(operator, _)| operator),
        kind,
        count: pending_operator
            .map(|(_, operator_count)| operator_count.saturating_mul(count))
            .unwrap_or(count),
    });
    Ok(())
}

fn start_vim_g_prefix(runtime: &mut EditorRuntime) -> Result<(), String> {
    let line_target = shell_ui_mut(runtime)?.vim_mut().take_count();
    let vim = shell_ui_mut(runtime)?.vim_mut();
    vim.pending_change_prefix = Some(VimRecordedInput::Text("g".to_owned()));
    vim.pending = Some(VimPending::GPrefix {
        operator: None,
        line_target,
    });
    Ok(())
}

fn start_visual_mode_with_kind(
    runtime: &mut EditorRuntime,
    kind: VisualSelectionKind,
) -> Result<(), String> {
    let cursor = active_shell_buffer_mut(runtime)?.cursor_point();
    shell_ui_mut(runtime)?.enter_visual_mode(cursor, kind);
    Ok(())
}

fn start_visual_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_visual_mode_with_kind(runtime, VisualSelectionKind::Character)
}

fn start_visual_line_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_visual_mode_with_kind(runtime, VisualSelectionKind::Line)
}

fn start_visual_block_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_visual_mode_with_kind(runtime, VisualSelectionKind::Block)
}

fn start_visual_text_object(runtime: &mut EditorRuntime, around: bool) -> Result<(), String> {
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::VisualTextObject { around, count });
    Ok(())
}

fn toggle_visual_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let mode = shell_ui(runtime)?.input_mode();
    if mode != InputMode::Visual {
        return start_visual_mode(runtime);
    }

    if shell_ui(runtime)?.vim().visual_kind == VisualSelectionKind::Character {
        shell_ui_mut(runtime)?.enter_normal_mode();
    } else {
        let ui = shell_ui_mut(runtime)?;
        ui.vim_mut().visual_kind = VisualSelectionKind::Character;
        ui.vim_mut().clear_transient();
    }

    Ok(())
}

fn toggle_visual_line_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let mode = shell_ui(runtime)?.input_mode();
    if mode != InputMode::Visual {
        return start_visual_line_mode(runtime);
    }

    if shell_ui(runtime)?.vim().visual_kind == VisualSelectionKind::Line {
        shell_ui_mut(runtime)?.enter_normal_mode();
    } else {
        let ui = shell_ui_mut(runtime)?;
        ui.vim_mut().visual_kind = VisualSelectionKind::Line;
        ui.vim_mut().clear_transient();
    }

    Ok(())
}

fn toggle_visual_block_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let mode = shell_ui(runtime)?.input_mode();
    if mode != InputMode::Visual {
        return start_visual_block_mode(runtime);
    }

    if shell_ui(runtime)?.vim().visual_kind == VisualSelectionKind::Block {
        shell_ui_mut(runtime)?.enter_normal_mode();
    } else {
        let ui = shell_ui_mut(runtime)?;
        ui.vim_mut().visual_kind = VisualSelectionKind::Block;
        ui.vim_mut().clear_transient();
    }

    Ok(())
}

fn swap_visual_anchor(runtime: &mut EditorRuntime) -> Result<(), String> {
    let current = active_shell_buffer_mut(runtime)?.cursor_point();
    let anchor = shell_ui(runtime)?
        .vim()
        .visual_anchor
        .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
    active_shell_buffer_mut(runtime)?.set_cursor(anchor);
    let ui = shell_ui_mut(runtime)?;
    ui.vim_mut().visual_anchor = Some(current);
    ui.vim_mut().clear_transient();
    Ok(())
}

fn put_yank(runtime: &mut EditorRuntime, after: bool) -> Result<(), String> {
    start_change_recording(runtime)?;
    let yank = {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        if let Some(register) = vim.active_register.take() {
            vim.registers.get(&register).cloned().or_else(|| vim.yank.clone())
        } else {
            vim.yank.clone()
        }
    };
    let Some(yank) = yank else {
        return Ok(());
    };

    {
        let buffer = active_shell_buffer_mut(runtime)?;
        match yank {
            YankRegister::Character(text) => {
                let insertion_point = if after {
                    buffer
                        .point_after(buffer.cursor_point())
                        .unwrap_or_else(|| buffer.cursor_point())
                } else {
                    buffer.cursor_point()
                };
                buffer.insert_at(insertion_point, &text);
            }
            YankRegister::Line(mut text) => {
                if !text.ends_with('\n') {
                    text.push('\n');
                }
                let line = buffer.cursor_row();
                let insertion_point = if after {
                    buffer
                        .line_range(line)
                        .map(TextRange::end)
                        .unwrap_or_else(|| buffer.cursor_point())
                } else {
                    buffer
                        .line_range(line)
                        .map(TextRange::start)
                        .unwrap_or_else(|| buffer.cursor_point())
                };
                let text = if after && line + 1 >= buffer.line_count() {
                    format!("\n{text}")
                } else {
                    text
                };
                buffer.insert_at(insertion_point, &text);
                if after {
                    buffer.goto_line(line.saturating_add(1));
                } else {
                    buffer.goto_line(line);
                }
            }
            YankRegister::Block(lines) => {
                let origin = buffer.cursor_point();
                let insertion_col = if after {
                    origin.column.saturating_add(1)
                } else {
                    origin.column
                };
                ensure_buffer_has_line(
                    buffer,
                    origin.line.saturating_add(lines.len().saturating_sub(1)),
                );
                for (offset, segment) in lines.iter().enumerate().rev() {
                    let target_line = origin.line + offset;
                    let target_col = insertion_col.min(buffer.line_len_chars(target_line));
                    buffer.insert_at(TextPoint::new(target_line, target_col), segment);
                }
                let target_col = insertion_col.min(buffer.line_len_chars(origin.line));
                buffer.set_cursor(TextPoint::new(origin.line, target_col));
            }
        }
        buffer.mark_syntax_dirty();
    }

    shell_ui_mut(runtime)?.vim_mut().clear_transient();
    schedule_finish_change(runtime)?;
    Ok(())
}

fn ensure_buffer_has_line(buffer: &mut ShellBuffer, target_line: usize) {
    while buffer.line_count() <= target_line {
        let last_line = buffer.line_count().saturating_sub(1);
        let point = TextPoint::new(last_line, buffer.line_len_chars(last_line));
        buffer.insert_at(point, "\n");
    }
}

fn syntax_registry_mut(runtime: &mut EditorRuntime) -> Result<&mut SyntaxRegistry, String> {
    runtime
        .services_mut()
        .get_mut::<SyntaxRegistry>()
        .ok_or_else(|| "syntax registry service missing".to_owned())
}

fn formatter_registry(runtime: &EditorRuntime) -> Result<&FormatterRegistry, String> {
    runtime
        .services()
        .get::<FormatterRegistry>()
        .ok_or_else(|| "formatter registry service missing".to_owned())
}

fn formatter_registry_mut(runtime: &mut EditorRuntime) -> Result<&mut FormatterRegistry, String> {
    runtime
        .services_mut()
        .get_mut::<FormatterRegistry>()
        .ok_or_else(|| "formatter registry service missing".to_owned())
}

fn sync_active_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some((buffer_id, buffer_name, buffer_kind)) = active_runtime_buffer(runtime)? else {
        return Ok(());
    };

    {
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(buffer_id, &buffer_name, buffer_kind);
        ui.focus_buffer_in_active_pane(buffer_id);
    }
    Ok(())
}

fn refresh_pending_syntax(runtime: &mut EditorRuntime) -> Result<(), String> {
    let now = Instant::now();
    let buffer_ids = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .filter(|buffer| buffer.syntax_refresh_due(now))
            .map(ShellBuffer::id)
            .collect::<Vec<_>>()
    };

    for buffer_id in buffer_ids {
        refresh_buffer_syntax(runtime, buffer_id)?;
    }

    Ok(())
}

fn active_window_id(runtime: &EditorRuntime) -> Result<editor_core::WindowId, String> {
    runtime
        .model()
        .active_window_id()
        .ok_or_else(|| "active window is missing".to_owned())
}

fn active_workspace_root(runtime: &EditorRuntime) -> Result<Option<PathBuf>, String> {
    Ok(runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?
        .root()
        .map(Path::to_path_buf))
}

fn find_workspace_by_root(
    runtime: &EditorRuntime,
    root: &std::path::Path,
) -> Result<Option<WorkspaceId>, String> {
    let window = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?;
    Ok(window.workspaces().find_map(|workspace| {
        workspace
            .root()
            .filter(|workspace_root| *workspace_root == root)
            .map(|_| workspace.id())
    }))
}

fn find_workspace_file_buffer(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
    path: &Path,
) -> Result<Option<BufferId>, String> {
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    Ok(workspace
        .buffers()
        .find(|buffer| buffer.path() == Some(path))
        .map(Buffer::id))
}

fn switch_runtime_workspace(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    runtime
        .model_mut()
        .switch_workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.switch_workspace(workspace_id);
    sync_active_buffer(runtime)
}

fn open_workspace_from_project(
    runtime: &mut EditorRuntime,
    name: &str,
    root: &std::path::Path,
) -> Result<WorkspaceId, String> {
    if let Some(workspace_id) = find_workspace_by_root(runtime, root)? {
        switch_runtime_workspace(runtime, workspace_id)?;
        return Ok(workspace_id);
    }

    let window_id = active_window_id(runtime)?;
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, name, Some(root.to_path_buf()))
        .map_err(|error| error.to_string())?;
    let notes_id = runtime
        .model_mut()
        .create_buffer(workspace_id, "*notes*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    let scratch_id = runtime
        .model_mut()
        .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;

    let (scratch, notes) = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let scratch = workspace
            .buffer(scratch_id)
            .ok_or_else(|| "new workspace scratch buffer is missing".to_owned())?;
        let notes = workspace
            .buffer(notes_id)
            .ok_or_else(|| "new workspace notes buffer is missing".to_owned())?;
        (
            ShellBuffer::from_runtime_buffer(
                scratch,
                workspace_scratch_lines(workspace.name(), workspace.root()),
            ),
            ShellBuffer::from_runtime_buffer(
                notes,
                workspace_notes_lines(workspace.name(), workspace.root()),
            ),
        )
    };

    let ui = shell_ui_mut(runtime)?;
    ui.add_workspace(workspace_id, scratch, notes, notes_id);
    ui.switch_workspace(workspace_id);

    runtime
        .emit_hook(
            builtins::WORKSPACE_OPEN,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_detail(name),
        )
        .map_err(|error| error.to_string())?;

    Ok(workspace_id)
}

fn delete_runtime_workspace(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let next_workspace = {
        let ui = shell_ui(runtime)?;
        if workspace_id == ui.default_workspace() {
            return Err("the default workspace cannot be deleted".to_owned());
        }

        if ui.active_workspace() != workspace_id {
            ui.active_workspace()
        } else {
            ui.previous_workspace()
                .filter(|candidate| ui.has_workspace(*candidate) && *candidate != workspace_id)
                .unwrap_or(ui.default_workspace())
        }
    };

    let window_id = active_window_id(runtime)?;
    let removed = runtime
        .model_mut()
        .close_workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.remove_workspace(workspace_id);
    runtime
        .emit_hook(
            builtins::WORKSPACE_CLOSE,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_detail(removed.name()),
        )
        .map_err(|error| error.to_string())?;

    switch_runtime_workspace(runtime, next_workspace)
}

fn active_runtime_popup(runtime: &EditorRuntime) -> Result<Option<RuntimePopupSnapshot>, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let Some(popup) = workspace.popups().next() else {
        return Ok(None);
    };

    Ok(Some(RuntimePopupSnapshot {
        title: popup.title().to_owned(),
        buffer_ids: popup.buffer_ids().to_vec(),
        active_buffer: popup.active_buffer(),
    }))
}

fn toggle_runtime_popup(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let popup_id = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .popups()
        .next()
        .map(|popup| popup.id());

    if let Some(popup_id) = popup_id {
        runtime
            .model_mut()
            .close_popup(workspace_id, popup_id)
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    let buffer_id = runtime
        .model_mut()
        .create_buffer(workspace_id, "*popup*", BufferKind::Diagnostics, None)
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![buffer_id], buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.ensure_buffer(buffer_id, "*popup*", BufferKind::Diagnostics);
    Ok(())
}

fn active_runtime_buffer(
    runtime: &EditorRuntime,
) -> Result<Option<(BufferId, String, BufferKind)>, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let Some(buffer_id) = workspace
        .active_pane()
        .and_then(|pane| pane.active_buffer())
    else {
        return Ok(None);
    };
    let buffer = workspace
        .buffer(buffer_id)
        .ok_or_else(|| format!("runtime buffer `{buffer_id}` is missing"))?;
    Ok(Some((
        buffer_id,
        buffer.name().to_owned(),
        buffer.kind().clone(),
    )))
}

fn refresh_workspace_syntax(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_ids = shell_ui(runtime)?
        .active_workspace_buffer_ids()
        .map(|buffer_ids| buffer_ids.to_vec())
        .unwrap_or_default();
    for buffer_id in buffer_ids {
        refresh_buffer_syntax(runtime, buffer_id)?;
    }
    Ok(())
}

fn refresh_buffer_syntax(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let Some((path, text)) = shell_ui(runtime)?.buffer(buffer_id).and_then(|buffer| {
        buffer
            .path()
            .map(|path| (path.to_path_buf(), buffer.text.clone()))
    }) else {
        if let Some(buffer) = shell_ui_mut(runtime)?.buffer_mut(buffer_id) {
            buffer.set_syntax_snapshot(None);
            buffer.set_syntax_error(None);
        }
        return Ok(());
    };

    let syntax_result = {
        let registry = syntax_registry_mut(runtime)?;
        match registry.highlight_buffer_for_path(&path, &text) {
            Ok(snapshot) => Ok(snapshot),
            Err(SyntaxError::GrammarNotInstalled { language_id, .. }) => {
                if let Err(error) = registry.install_language(&language_id) {
                    Err(error)
                } else {
                    registry.highlight_buffer_for_path(&path, &text)
                }
            }
            Err(error) => Err(error),
        }
    };

    let ui = shell_ui_mut(runtime)?;
    if let Some(buffer) = ui.buffer_mut(buffer_id) {
        match syntax_result {
            Ok(snapshot) => {
                buffer.set_syntax_snapshot(Some(snapshot));
                buffer.set_syntax_error(None);
            }
            Err(error) => {
                eprintln!(
                    "tree-sitter syntax refresh failed for `{}`: {error}",
                    path.display()
                );
                buffer.set_syntax_snapshot(None);
                buffer.set_syntax_error(Some(error.to_string()));
            }
        }
    }

    Ok(())
}

fn install_tree_sitter_language(
    runtime: &mut EditorRuntime,
    language_id: &str,
) -> Result<(), String> {
    syntax_registry_mut(runtime)?
        .install_language(language_id)
        .map_err(|error| error.to_string())?;
    refresh_workspace_syntax(runtime)
}

fn file_open_detail(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
}

fn open_workspace_file(runtime: &mut EditorRuntime, path: &Path) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if let Some(existing) = find_workspace_file_buffer(runtime, workspace_id, path)? {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        shell_ui_mut(runtime)?.focus_buffer_in_active_pane(existing);
        refresh_buffer_syntax(runtime, existing)?;
        return Ok(existing);
    }

    let workspace_root = active_workspace_root(runtime)?;
    let display_name = workspace_relative_path(workspace_root.as_deref(), path);
    let text = TextBuffer::load_from_path(path)
        .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            display_name.as_str(),
            BufferKind::File,
            Some(path.to_path_buf()),
        )
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("new file buffer `{buffer_id}` is missing"))?;
    let shell_buffer = ShellBuffer::from_text_buffer(buffer, text);

    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
    }

    if let Some(detail) = file_open_detail(path) {
        runtime
            .emit_hook(
                builtins::FILE_OPEN,
                HookEvent::new()
                    .with_workspace(workspace_id)
                    .with_buffer(buffer_id)
                    .with_detail(detail),
            )
            .map_err(|error| error.to_string())?;
    }
    refresh_buffer_syntax(runtime, buffer_id)?;

    Ok(buffer_id)
}

fn picker_overlay(runtime: &EditorRuntime, provider: &str) -> Result<PickerOverlay, String> {
    match provider {
        "commands" => Ok(command_picker_overlay(runtime)),
        "buffers" => buffer_picker_overlay(runtime),
        "keybindings" => Ok(keybinding_picker_overlay(runtime)),
        "treesitter.languages" => treesitter_install_picker_overlay(runtime),
        "workspace.projects" => workspace_project_picker_overlay(runtime),
        "workspace.switch" => workspace_switch_picker_overlay(runtime),
        "workspace.delete" => workspace_delete_picker_overlay(runtime),
        "workspace.files" => workspace_file_picker_overlay(runtime),
        "themes" => theme_picker_overlay(runtime),
        other => Err(format!("unknown picker provider `{other}`")),
    }
}

fn command_picker_overlay(runtime: &EditorRuntime) -> PickerOverlay {
    let entries = runtime
        .commands()
        .definitions()
        .into_iter()
        .map(|definition| PickerEntry {
            item: PickerItem::new(
                definition.name(),
                definition.name(),
                definition.description(),
                Some(definition.description()),
            ),
            action: PickerAction::ExecuteCommand(definition.name().to_owned()),
        })
        .collect();

    PickerOverlay::from_entries("Command Palette", entries)
}

fn buffer_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let ui = shell_ui(runtime)?;
    let entries = ui
        .active_workspace_buffer_ids()
        .into_iter()
        .flatten()
        .filter_map(|buffer_id| ui.buffer(*buffer_id))
        .map(|buffer| PickerEntry {
            item: PickerItem::new(
                buffer.id().to_string(),
                buffer.display_name(),
                buffer.kind_label(),
                Some(format!(
                    "{} | row {}, col {}",
                    buffer.kind_label(),
                    buffer.cursor_row() + 1,
                    buffer.cursor_col() + 1,
                )),
            ),
            action: PickerAction::FocusBuffer(buffer.id()),
        })
        .collect();

    Ok(PickerOverlay::from_entries("Buffers", entries))
}

fn treesitter_install_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let registry = runtime
        .services()
        .get::<SyntaxRegistry>()
        .ok_or_else(|| "syntax registry service missing".to_owned())?;
    let entries = registry
        .languages()
        .map(|language| {
            let detail = match language.grammar() {
                Some(grammar) => {
                    let installed = registry.is_installed(language.id()).unwrap_or(false);
                    let status = if installed { "installed" } else { "missing" };
                    format!("{status} | {}", grammar.repository_url())
                }
                None => "built-in grammar".to_owned(),
            };
            let preview = language.grammar().map(|grammar| {
                grammar
                    .install_directory(registry.install_root())
                    .display()
                    .to_string()
            });
            PickerEntry {
                item: PickerItem::new(language.id(), language.id(), detail, preview),
                action: PickerAction::InstallTreeSitterLanguage(language.id().to_owned()),
            }
        })
        .collect();

    Ok(PickerOverlay::from_entries("Tree-sitter Install", entries))
}

fn workspace_project_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let entries = discover_projects(&user::workspace::project_search_roots())
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|project| {
            let existing_workspace = find_workspace_by_root(runtime, project.root())?;
            let detail = if existing_workspace.is_some() {
                format!("{} | open workspace", project.kind().label())
            } else {
                project.kind().label().to_owned()
            };
            let action = existing_workspace.map_or(
                PickerAction::CreateWorkspace {
                    name: project.name().to_owned(),
                    root: project.root().to_path_buf(),
                },
                PickerAction::SwitchWorkspace,
            );
            Ok(PickerEntry {
                item: PickerItem::new(
                    project.root().display().to_string(),
                    project.name(),
                    detail,
                    Some(project.root().display().to_string()),
                ),
                action,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(PickerOverlay::from_entries("Projects", entries))
}

fn workspace_switch_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let entries = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?
        .workspaces()
        .map(|workspace| PickerEntry {
            item: PickerItem::new(
                workspace.id().to_string(),
                workspace.name(),
                workspace
                    .root()
                    .map(|root| root.display().to_string())
                    .unwrap_or_else(|| "default workspace".to_owned()),
                workspace.root().map(|root| root.display().to_string()),
            ),
            action: PickerAction::SwitchWorkspace(workspace.id()),
        })
        .collect();

    Ok(PickerOverlay::from_entries("Workspaces", entries))
}

fn workspace_delete_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let default_workspace = shell_ui(runtime)?.default_workspace();
    let entries = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?
        .workspaces()
        .filter(|workspace| workspace.id() != default_workspace)
        .map(|workspace| PickerEntry {
            item: PickerItem::new(
                workspace.id().to_string(),
                workspace.name(),
                workspace
                    .root()
                    .map(|root| root.display().to_string())
                    .unwrap_or_else(|| "workspace".to_owned()),
                Some("Deletes the selected workspace.".to_owned()),
            ),
            action: PickerAction::DeleteWorkspace(workspace.id()),
        })
        .collect();

    Ok(PickerOverlay::from_entries("Delete Workspace", entries))
}

fn workspace_file_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let workspace = runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?;
    let Some(root) = workspace.root() else {
        return Ok(message_picker_overlay(
            "Workspace Files",
            "Workspace has no project root",
            "Open a project-backed workspace before listing files.",
            Some(
                "workspace.list-files works from a project workspace created by workspace.new."
                    .to_owned(),
            ),
        ));
    };

    let files = match list_repository_files(root) {
        Ok(files) => files,
        Err(error) => {
            return Ok(message_picker_overlay(
                "Workspace Files",
                "Unable to read workspace files",
                &error.to_string(),
                Some(root.display().to_string()),
            ));
        }
    };

    if files.is_empty() {
        return Ok(message_picker_overlay(
            "Workspace Files",
            "No visible files found",
            "Git did not report any tracked or unignored files for this workspace.",
            Some(root.display().to_string()),
        ));
    }

    let entries = files
        .into_iter()
        .map(|relative_path| {
            let path = root.join(&relative_path);
            let label = workspace_relative_path(Some(root), &path);
            let detail = relative_path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(|parent| parent.display().to_string())
                .unwrap_or_else(|| "workspace root".to_owned());
            PickerEntry {
                item: PickerItem::new(
                    path.display().to_string(),
                    label,
                    detail,
                    Some(path.display().to_string()),
                ),
                action: PickerAction::OpenFile(path),
            }
        })
        .collect();

    Ok(PickerOverlay::from_entries("Workspace Files", entries))
}

fn keybinding_picker_overlay(runtime: &EditorRuntime) -> PickerOverlay {
    let entries = runtime
        .keymaps()
        .bindings()
        .into_iter()
        .map(|binding| {
            let description = runtime
                .commands()
                .get(binding.command_name())
                .map(|definition| definition.description().to_owned())
                .unwrap_or_else(|| "Command description unavailable.".to_owned());
            let scope = binding.scope().to_string();
            let mode = binding.vim_mode().to_string();
            PickerEntry {
                item: PickerItem::new(
                    format!("{scope}:{mode}:{}", binding.chord()),
                    binding.chord(),
                    format!(
                        "{} [{}] -> {}",
                        binding.scope(),
                        mode,
                        binding.command_name()
                    ),
                    Some(description),
                ),
                action: PickerAction::ExecuteCommand(binding.command_name().to_owned()),
            }
        })
        .collect();

    let mut overlay = PickerOverlay::from_entries("Keybindings", entries);
    overlay.session.set_result_limit(256);
    overlay
}

fn theme_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let registry = runtime
        .services()
        .get::<ThemeRegistry>()
        .ok_or_else(|| "theme registry service missing".to_owned())?;
    let entries = registry
        .themes()
        .map(|theme| {
            let theme_id = theme.id().to_owned();
            PickerEntry {
                item: PickerItem::new(&theme_id, theme.name(), "Theme", Some(theme_id.clone())),
                action: PickerAction::ActivateTheme(theme_id),
            }
        })
        .collect();
    Ok(PickerOverlay::from_entries("Themes", entries))
}

fn message_picker_overlay(
    title: &str,
    label: &str,
    detail: &str,
    preview: Option<String>,
) -> PickerOverlay {
    PickerOverlay::from_entries(
        title,
        vec![PickerEntry {
            item: PickerItem::new(label, label, detail, preview),
            action: PickerAction::NoOp,
        }],
    )
}

fn workspace_relative_path(root: Option<&Path>, path: &Path) -> String {
    root.and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn keydown_chord(keycode: Keycode, keymod: Mod) -> Option<String> {
    if keymod.intersects(ctrl_mod()) {
        return match keycode {
            Keycode::B => Some("Ctrl+b".to_owned()),
            Keycode::D => Some("Ctrl+d".to_owned()),
            Keycode::E => Some("Ctrl+e".to_owned()),
            Keycode::F => Some("Ctrl+f".to_owned()),
            Keycode::N => Some("Ctrl+n".to_owned()),
            Keycode::P => Some("Ctrl+p".to_owned()),
            Keycode::R => Some("Ctrl+r".to_owned()),
            Keycode::U => Some("Ctrl+u".to_owned()),
            Keycode::V => Some("Ctrl+v".to_owned()),
            Keycode::Y => Some("Ctrl+y".to_owned()),
            Keycode::Grave => Some("Ctrl+`".to_owned()),
            _ => None,
        };
    }

    match keycode {
        Keycode::F3 => Some("F3".to_owned()),
        Keycode::F4 => Some("F4".to_owned()),
        Keycode::F5 => Some("F5".to_owned()),
        Keycode::F6 => Some("F6".to_owned()),
        Keycode::Escape => Some("Escape".to_owned()),
        Keycode::Return | Keycode::KpEnter => Some("Enter".to_owned()),
        _ => None,
    }
}

fn text_chord(text: &str) -> Option<String> {
    let mut characters = text.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }
    Some(character.to_string())
}

fn ctrl_mod() -> Mod {
    Mod::LCTRLMOD | Mod::RCTRLMOD
}

fn keymap_vim_mode(input_mode: InputMode) -> KeymapVimMode {
    match input_mode {
        InputMode::Normal => KeymapVimMode::Normal,
        InputMode::Insert | InputMode::Replace => KeymapVimMode::Insert,
        InputMode::Visual => KeymapVimMode::Visual,
    }
}

fn initial_scratch_lines() -> Vec<String> {
    vec![
        "Volt SDL shell is now driven by the compiled user packages.".to_owned(),
        "NORMAL mode is loaded from user/vim.rs out of the box.".to_owned(),
        "The shell starts in the default workspace and renders its statusline from user/statusline.rs.".to_owned(),
        "Use h/j/k/l to move, w to jump forward by word, and : to open the command picker.".to_owned(),
        "Press i to enter INSERT mode, then type directly into the active buffer.".to_owned(),
        "F3 opens the command picker and F4 opens the buffer picker through user/picker.rs.".to_owned(),
        "F5 toggles the docked popup window and F6 opens a searchable keybinding picker.".to_owned(),
        "Inside a picker use Ctrl-n and Ctrl-p to move, Enter to run, and Escape to close.".to_owned(),
        "F2 splits the layout, Tab changes panes, Ctrl+` opens the terminal buffer, and Ctrl+q quits.".to_owned(),
        "Volt SDL shell is now driven by the compiled user packages.".to_owned(),
        "NORMAL mode is loaded from user/vim.rs out of the box.".to_owned(),
        "The shell starts in the default workspace and renders its statusline from user/statusline.rs.".to_owned(),
        "Use h/j/k/l to move, w to jump forward by word, and : to open the command picker.".to_owned(),
        "Press i to enter INSERT mode, then type directly into the active buffer.".to_owned(),
        "F3 opens the command picker and F4 opens the buffer picker through user/picker.rs.".to_owned(),
        "F5 toggles the docked popup window and F6 opens a searchable keybinding picker.".to_owned(),
        "Inside a picker use Ctrl-n and Ctrl-p to move, Enter to run, and Escape to close.".to_owned(),
        "F2 splits the layout, Tab changes panes, Ctrl+` opens the terminal buffer, and Ctrl+q quits.".to_owned(),
    ]
}

fn workspace_scratch_lines(name: &str, root: Option<&std::path::Path>) -> Vec<String> {
    if name == "default" && root.is_none() {
        return initial_scratch_lines();
    }

    let mut lines = vec![format!("Workspace `{name}` is now active.")];
    if let Some(root) = root {
        lines.push(format!("Root: {}", root.display()));
    }
    lines.push("This workspace was opened from the project picker.".to_owned());
    lines.push(
        "Run `workspace.switch` to change workspaces or `workspace.delete` to close one."
            .to_owned(),
    );
    lines
}

fn initial_notes_lines() -> Vec<String> {
    vec![
        "Second pane notes.".to_owned(),
        "Use F2 to split horizontally and Tab to move between panes.".to_owned(),
        "The buffer picker opened by F4 reuses the same searchable popup surface as F3.".to_owned(),
    ]
}

fn workspace_notes_lines(name: &str, root: Option<&std::path::Path>) -> Vec<String> {
    if name == "default" && root.is_none() {
        return initial_notes_lines();
    }

    let mut lines = vec![format!("Notes for workspace `{name}`.")];
    if let Some(root) = root {
        lines.push(format!("Project root: {}", root.display()));
    }
    lines.push("Use this buffer for project-specific notes or scratch edits.".to_owned());
    lines
}

fn placeholder_lines(name: &str, kind: &BufferKind) -> Vec<String> {
    match name {
        "*scratch*" => initial_scratch_lines(),
        "*notes*" => initial_notes_lines(),
        _ => match kind {
            BufferKind::Scratch => vec![
                format!("{name} is a scratch buffer created by the runtime."),
                "This buffer can be focused from the generic buffer picker.".to_owned(),
            ],
            BufferKind::Picker => vec![
                format!("{name} is a picker-backed buffer."),
                "The SDL shell renders picker state through the popup search UI.".to_owned(),
            ],
            BufferKind::Terminal => vec![
                format!("{name} was opened by the compiled terminal package."),
                "Terminal rendering is still placeholder content in this vertical slice.".to_owned(),
            ],
            BufferKind::Git => vec![
                format!("{name} is reserved for git workflows."),
                "The next iteration can wire real magit-style status content here.".to_owned(),
            ],
            BufferKind::Directory => vec![
                format!("{name} is a directory buffer."),
                "Oil-style editing surfaces can be rendered through the same shell.".to_owned(),
            ],
            BufferKind::Compilation => vec![
                format!("{name} collects compilation output."),
                "Compilation runner integration is available through the core job model.".to_owned(),
            ],
            BufferKind::Diagnostics => vec![
                format!("{name} is a diagnostics-oriented buffer."),
                "LSP and DAP packages can surface structured status here.".to_owned(),
            ],
            BufferKind::File => vec![
                format!("{name} is a file-backed buffer placeholder."),
                "File loading is not yet wired into the SDL shell event loop.".to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) => vec![
                format!("{name} was opened for plugin kind `{plugin_kind}`."),
                "Users can change this behavior by editing the matching user package and recompiling.".to_owned(),
            ],
        },
    }
}

fn buffer_kind_label(kind: &BufferKind) -> String {
    match kind {
        BufferKind::File => "file".to_owned(),
        BufferKind::Scratch => "scratch".to_owned(),
        BufferKind::Picker => "picker".to_owned(),
        BufferKind::Terminal => "terminal".to_owned(),
        BufferKind::Git => "git".to_owned(),
        BufferKind::Directory => "directory".to_owned(),
        BufferKind::Compilation => "compilation".to_owned(),
        BufferKind::Diagnostics => "diagnostics".to_owned(),
        BufferKind::Plugin(plugin_kind) => plugin_kind.clone(),
    }
}

fn picker_scroll_top(match_count: usize, selected_index: usize, visible_rows: usize) -> usize {
    let visible_rows = visible_rows.max(1);
    if match_count <= visible_rows {
        return 0;
    }

    selected_index
        .saturating_sub(visible_rows.saturating_sub(1))
        .min(match_count - visible_rows)
}

fn popup_window_height(content_height: u32, line_height: i32) -> u32 {
    let row_height = line_height.max(1) as u32;
    if content_height <= row_height {
        return content_height;
    }

    let desired = (content_height / 5).max(row_height * 4);
    let max_height = content_height.saturating_sub(row_height).max(row_height);
    let clamped = desired.min(max_height);
    (clamped / row_height).max(1) * row_height
}

#[allow(clippy::too_many_arguments)]
fn render_shell_state(
    target: &mut DrawTarget<'_>,
    font: &Font<'_>,
    state: &ShellUiState,
    runtime_popup: Option<&RuntimePopupSnapshot>,
    workspace_name: &str,
    lsp_server: Option<&str>,
    theme_registry: Option<&ThemeRegistry>,
    width: u32,
    height: u32,
    cell_width: i32,
    line_height: i32,
    ascent: i32,
    now: Instant,
) -> Result<(), ShellError> {
    let content_height = height;
    let popup_height = runtime_popup
        .map(|_| popup_window_height(height, line_height))
        .unwrap_or(0);
    let pane_height = content_height.saturating_sub(popup_height);
    let panes = state
        .panes()
        .ok_or_else(|| ShellError::Runtime("active workspace view is missing".to_owned()))?;
    let pane_rects = horizontal_pane_rects(width, pane_height, panes.len());

    target.clear(Color::RGB(15, 16, 20));

    for (pane_index, pane) in panes.iter().enumerate() {
        let rect = pane_rects[pane_index];
        let active = pane_index == state.active_pane_index()
            && !state.picker_visible()
            && runtime_popup.is_none();
        let background = if active {
            Color::RGB(27, 31, 39)
        } else {
            Color::RGB(22, 24, 30)
        };
        fill_rect(
            target,
            PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
            background,
        )?;
        fill_rect(
            target,
            PixelRectToRect::rect(rect.x, rect.y, rect.width, 1),
            Color::RGB(50, 55, 66),
        )?;

        if let Some(buffer) = state.buffer(pane.buffer_id) {
            let visual_range = (state.input_mode() == InputMode::Visual && active)
                .then(|| {
                    state.vim().visual_anchor.and_then(|anchor| {
                        visual_selection(buffer, anchor, state.vim().visual_kind)
                    })
                })
                .flatten();
            let yank_flash = state.yank_flash(buffer.id(), now);
            render_buffer(
                target,
                font,
                buffer,
                PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
                active,
                visual_range,
                yank_flash,
                state.input_mode(),
                state.vim().recording_macro,
                workspace_name,
                lsp_server,
                theme_registry,
                cell_width,
                line_height,
                ascent,
            )?;
        }
    }

    if let Some(popup) = runtime_popup {
        render_runtime_popup_overlay(
            target,
            font,
            state,
            popup,
            PixelRectToRect::rect(0, pane_height as i32, width, popup_height),
            workspace_name,
            lsp_server,
            theme_registry,
            cell_width,
            line_height,
            ascent,
            now,
        )?;
    }

    if let Some(picker) = state.picker() {
        render_picker_overlay(
            target,
            font,
            picker,
            width,
            height,
            line_height,
            theme_registry,
        )?;
    }

    Ok(())
}

fn render_picker_overlay(
    target: &mut DrawTarget<'_>,
    font: &Font<'_>,
    picker: &PickerOverlay,
    width: u32,
    height: u32,
    line_height: i32,
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(), ShellError> {
    let popup_rect = centered_rect(width, height, width * 2 / 3, height * 3 / 5);
    let picker_roundness = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_PICKER_ROUNDNESS))
        .map(|value| value.clamp(0.0, 64.0).round() as u32)
        .unwrap_or(16);
    fill_rounded_rect(
        target,
        PixelRectToRect::rect(
            popup_rect.x,
            popup_rect.y,
            popup_rect.width,
            popup_rect.height,
        ),
        picker_roundness,
        Color::RGB(29, 32, 40),
    )?;
    fill_rect(
        target,
        PixelRectToRect::rect(
            popup_rect.x + 14,
            popup_rect.y,
            popup_rect.width.saturating_sub(28),
            2,
        ),
        Color::RGB(110, 170, 255),
    )?;

    draw_text(
        target,
        popup_rect.x + 16,
        popup_rect.y + 16,
        picker.session().title(),
        Color::RGBA(215, 221, 232, 255),
    )?;

    let query = format!("Query > {}", picker.session().query());
    draw_text(
        target,
        popup_rect.x + 16,
        popup_rect.y + line_height + 24,
        &query,
        Color::RGBA(180, 191, 208, 255),
    )?;

    let summary = format!(
        "{} / {} results",
        picker.session().match_count(),
        picker.session().item_count(),
    );
    draw_text(
        target,
        popup_rect.x + 16,
        popup_rect.y + (line_height * 2) + 28,
        &summary,
        Color::RGBA(120, 132, 150, 255),
    )?;

    let row_height = (line_height + 8).max(24);
    let list_top = popup_rect.y + (line_height * 3) + 42;
    let list_height = popup_rect.height as i32 - ((line_height * 4) + 62).max(0);
    let visible_rows = (list_height.max(row_height) / row_height).max(1) as usize;
    let selected_id = picker
        .session()
        .selected()
        .map(|selected| selected.item().id().to_owned());
    let selected_index = selected_id
        .as_deref()
        .and_then(|selected_id| {
            picker
                .session()
                .matches()
                .iter()
                .position(|matched| matched.item().id() == selected_id)
        })
        .unwrap_or(0);
    let scroll_top =
        picker_scroll_top(picker.session().match_count(), selected_index, visible_rows);

    if picker.session().matches().is_empty() {
        draw_text(
            target,
            popup_rect.x + 16,
            list_top,
            "No matches.",
            Color::RGBA(120, 132, 150, 255),
        )?;
        return Ok(());
    }

    for (index, matched) in picker
        .session()
        .matches()
        .iter()
        .skip(scroll_top)
        .take(visible_rows)
        .enumerate()
    {
        let row_y = list_top + index as i32 * row_height;
        let selected = selected_id.as_deref() == Some(matched.item().id());
        let content_left = popup_rect.x + 18;
        let content_width = popup_rect.width.saturating_sub(36);
        let label_width = (content_width * 2 / 5).max(160);
        let detail_x = content_left + label_width as i32 + 16;
        let detail_width = content_width.saturating_sub(label_width + 16);
        if selected {
            fill_rect(
                target,
                PixelRectToRect::rect(
                    popup_rect.x + 12,
                    row_y - 2,
                    popup_rect.width.saturating_sub(24),
                    row_height as u32,
                ),
                Color::RGB(45, 61, 85),
            )?;
        }

        let label = truncate_text_to_width(font, matched.item().label(), label_width)?;
        let detail = truncate_text_to_width(font, matched.item().detail(), detail_width)?;
        draw_text(
            target,
            content_left,
            row_y,
            &label,
            if selected {
                Color::RGBA(255, 255, 255, 255)
            } else {
                Color::RGBA(215, 221, 232, 255)
            },
        )?;
        draw_text(
            target,
            detail_x,
            row_y,
            &detail,
            Color::RGBA(150, 163, 182, 255),
        )?;
    }

    if let Some(preview) = picker
        .session()
        .selected()
        .and_then(|selected| selected.item().preview())
    {
        draw_text(
            target,
            popup_rect.x + 16,
            popup_rect.y + popup_rect.height as i32 - line_height - 18,
            &truncate_text_to_width(font, preview, popup_rect.width.saturating_sub(32))?,
            Color::RGBA(120, 132, 150, 255),
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_runtime_popup_overlay(
    target: &mut DrawTarget<'_>,
    font: &Font<'_>,
    state: &ShellUiState,
    popup: &RuntimePopupSnapshot,
    popup_rect: Rect,
    workspace_name: &str,
    lsp_server: Option<&str>,
    theme_registry: Option<&ThemeRegistry>,
    cell_width: i32,
    line_height: i32,
    ascent: i32,
    now: Instant,
) -> Result<(), ShellError> {
    fill_rect(target, popup_rect, Color::RGB(29, 32, 40))?;
    fill_rect(
        target,
        PixelRectToRect::rect(
            popup_rect.x() + 12,
            popup_rect.y(),
            popup_rect.width().saturating_sub(24),
            2,
        ),
        Color::RGB(110, 170, 255),
    )?;
    draw_text(
        target,
        popup_rect.x() + 14,
        popup_rect.y() + 16,
        &popup.title,
        Color::RGBA(215, 221, 232, 255),
    )?;

    let mut tab_x = popup_rect.x() + 14;
    for buffer_id in &popup.buffer_ids {
        if let Some(buffer) = state.buffer(*buffer_id) {
            let tab_color = if *buffer_id == popup.active_buffer {
                Color::RGBA(110, 170, 255, 255)
            } else {
                Color::RGBA(120, 132, 150, 255)
            };
            draw_text(
                target,
                tab_x,
                popup_rect.y() + line_height + 22,
                buffer.display_name(),
                tab_color,
            )?;
            tab_x += 180;
        }
    }

    let popup_buffer_rect = PixelRectToRect::rect(
        popup_rect.x() + 14,
        popup_rect.y() + (line_height * 2) + 28,
        popup_rect.width().saturating_sub(28),
        popup_rect
            .height()
            .saturating_sub((line_height as u32 * 2) + 36),
    );
    if let Some(buffer) = state.buffer(popup.active_buffer) {
        let visual_range = (state.input_mode() == InputMode::Visual)
            .then(|| {
                state
                    .vim()
                    .visual_anchor
                    .and_then(|anchor| visual_selection(buffer, anchor, state.vim().visual_kind))
            })
            .flatten();
        let yank_flash = state.yank_flash(buffer.id(), now);
        render_buffer(
            target,
            font,
            buffer,
            popup_buffer_rect,
            true,
            visual_range,
            yank_flash,
            state.input_mode(),
            state.vim().recording_macro,
            workspace_name,
            lsp_server,
            theme_registry,
            cell_width,
            line_height,
            ascent,
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_buffer(
    target: &mut DrawTarget<'_>,
    font: &Font<'_>,
    buffer: &ShellBuffer,
    rect: Rect,
    active: bool,
    visual_selection: Option<VisualSelection>,
    yank_flash: Option<VisualSelection>,
    input_mode: InputMode,
    recording_macro: Option<char>,
    workspace_name: &str,
    lsp_server: Option<&str>,
    theme_registry: Option<&ThemeRegistry>,
    cell_width: i32,
    line_height: i32,
    ascent: i32,
) -> Result<(), ShellError> {
    let title_color = if active {
        Color::RGBA(110, 170, 255, 255)
    } else {
        Color::RGBA(120, 132, 150, 255)
    };
    let text_color = Color::RGBA(215, 221, 232, 255);
    let muted = Color::RGBA(120, 132, 150, 255);
    let cursor = Color::RGB(110, 170, 255);
    let selection = Color::RGBA(55, 71, 99, 255);
    let relative_line_numbers = theme_registry
        .and_then(|registry| registry.resolve_bool(OPTION_LINE_NUMBER_RELATIVE))
        .unwrap_or(false);
    let cursor_roundness = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_CURSOR_ROUNDNESS))
        .map(|value| value.clamp(0.0, 16.0).round() as u32)
        .unwrap_or(2);
    let yank_flash_color = theme_registry
        .and_then(|registry| registry.resolve("ui.yank-flash"))
        .map(to_sdl_color)
        .unwrap_or(Color::RGBA(112, 196, 255, 120));
    let statusline = truncate_text_to_width(
        font,
        &user::statusline::compose(&user::statusline::StatuslineContext {
            vim_mode: input_mode.label(),
            recording_macro,
            workspace_name,
            buffer_name: buffer.display_name(),
            line: buffer.cursor_row() + 1,
            column: buffer.cursor_col() + 1,
            lsp_server,
        }),
        rect.width().saturating_sub(24),
    )?;

    let body_y = rect.y() + 10;
    let statusline_y = rect.y() + rect.height() as i32 - line_height - 8;
    let visible_body_height = (statusline_y - body_y - 10).max(line_height);
    let visible_lines = (visible_body_height / line_height.max(1)).max(1) as usize;
    let line_number_width = cell_width * 5;
    let cursor_row = buffer.cursor_row();
    let cursor_col = buffer.cursor_col();
    let cursor_row_on_screen = cursor_row.saturating_sub(buffer.scroll_row);
    if cursor_row_on_screen < visible_lines {
        let cursor_width = match input_mode {
            InputMode::Normal | InputMode::Visual => cell_width.max(2) as u32,
            InputMode::Insert | InputMode::Replace => (cell_width / 4).max(2) as u32,
        };
        fill_rounded_rect(
            target,
            PixelRectToRect::rect(
                rect.x() + 12 + line_number_width + (cursor_col as i32 * cell_width),
                body_y + cursor_row_on_screen as i32 * line_height,
                cursor_width,
                line_height.max(2) as u32,
            ),
            cursor_roundness,
            cursor,
        )?;
    }

    for (row_offset, line) in buffer.visible_lines(visible_lines).into_iter().enumerate() {
        let y = body_y + row_offset as i32 * line_height;
        let line_index = buffer.scroll_row + row_offset;
        let line_len = buffer.line_len_chars(line_index);
        if let Some((selection_start, selection_end)) =
            visual_selection.and_then(|selection_state| {
                selection_columns_for_visual(selection_state, line_index, line_len)
            })
        {
            fill_rect(
                target,
                PixelRectToRect::rect(
                    rect.x() + 12 + line_number_width + selection_start as i32 * cell_width,
                    y,
                    ((selection_end.saturating_sub(selection_start)) as i32 * cell_width.max(1))
                        as u32,
                    line_height.max(1) as u32,
                ),
                selection,
            )?;
        }
        if let Some((selection_start, selection_end)) = yank_flash.and_then(|selection_state| {
            selection_columns_for_visual(selection_state, line_index, line_len)
        }) {
            fill_rect(
                target,
                PixelRectToRect::rect(
                    rect.x() + 12 + line_number_width + selection_start as i32 * cell_width,
                    y,
                    ((selection_end.saturating_sub(selection_start)) as i32 * cell_width.max(1))
                        as u32,
                    line_height.max(1) as u32,
                ),
                yank_flash_color,
            )?;
        }
        let line_number = if relative_line_numbers {
            if line_index == cursor_row {
                0
            } else {
                cursor_row.abs_diff(line_index)
            }
        } else {
            line_index + 1
        };
        draw_text(
            target,
            rect.x() + 12,
            y,
            &format!("{:>4}", line_number),
            muted,
        )?;
        draw_buffer_text(
            target,
            font,
            rect.x() + 12 + line_number_width,
            y,
            &line,
            buffer.line_syntax_spans(buffer.scroll_row + row_offset),
            theme_registry,
            text_color,
        )?;
    }

    fill_rect(
        target,
        PixelRectToRect::rect(
            rect.x() + 8,
            statusline_y - 6,
            rect.width().saturating_sub(16),
            1,
        ),
        Color::RGB(50, 55, 66),
    )?;
    draw_text(
        target,
        rect.x() + 12,
        statusline_y,
        &statusline,
        title_color,
    )?;

    let _ = ascent;
    fill_rect(
        target,
        PixelRectToRect::rect(
            rect.x(),
            rect.y() + rect.height() as i32 - 2,
            rect.width(),
            1,
        ),
        Color::RGB(50, 55, 66),
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_buffer_text(
    target: &mut DrawTarget<'_>,
    font: &Font<'_>,
    x: i32,
    y: i32,
    line: &str,
    line_syntax_spans: Option<&[LineSyntaxSpan]>,
    theme_registry: Option<&ThemeRegistry>,
    default_color: Color,
) -> Result<(), ShellError> {
    let mut draw_x = x;
    for (segment, color) in
        line_color_segments(line, line_syntax_spans, theme_registry, default_color)
    {
        if segment.is_empty() {
            continue;
        }
        draw_text(target, draw_x, y, &segment, color)?;
        draw_x += text_width(font, &segment)? as i32;
    }
    Ok(())
}

fn line_color_segments(
    line: &str,
    line_syntax_spans: Option<&[LineSyntaxSpan]>,
    theme_registry: Option<&ThemeRegistry>,
    default_color: Color,
) -> Vec<(String, Color)> {
    let Some(line_syntax_spans) = line_syntax_spans else {
        return vec![(line.to_owned(), default_color)];
    };

    let relevant_spans = line_syntax_spans
        .iter()
        .filter_map(|span| {
            let start = clamp_to_char_boundary(line, span.start);
            let end = clamp_to_char_boundary(line, span.end.min(line.len()));
            if start >= end {
                return None;
            }

            Some((start, end, span.theme_token.as_str()))
        })
        .collect::<Vec<_>>();
    if relevant_spans.is_empty() {
        return vec![(line.to_owned(), default_color)];
    }

    let mut breakpoints = vec![0, line.len()];
    for (start, end, _) in &relevant_spans {
        breakpoints.push(*start);
        breakpoints.push(*end);
    }
    breakpoints.sort_unstable();
    breakpoints.dedup();

    let mut segments = Vec::new();
    for window in breakpoints.windows(2) {
        let start = window[0];
        let end = window[1];
        if start >= end {
            continue;
        }
        let Some(text) = line.get(start..end) else {
            continue;
        };
        let color = relevant_spans
            .iter()
            .filter(|(span_start, span_end, _)| start >= *span_start && end <= *span_end)
            .min_by_key(|(span_start, span_end, _)| span_end.saturating_sub(*span_start))
            .and_then(|(_, _, token)| theme_registry.and_then(|registry| registry.resolve(token)))
            .map(to_sdl_color)
            .unwrap_or(default_color);
        segments.push((text.to_owned(), color));
    }

    if segments.is_empty() {
        vec![(line.to_owned(), default_color)]
    } else {
        segments
    }
}

fn selection_columns_for_line(
    range: TextRange,
    line_index: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    let range = range.normalized();
    if line_index < range.start().line || line_index > range.end().line {
        return None;
    }

    let start = if line_index == range.start().line {
        range.start().column
    } else {
        0
    };
    let end = if line_index == range.end().line {
        range.end().column
    } else {
        line_len
    };
    let start = start.min(line_len);
    let end = end.min(line_len);
    (start < end).then_some((start, end))
}

fn selection_columns_for_visual(
    selection: VisualSelection,
    line_index: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    match selection {
        VisualSelection::Range(range) => selection_columns_for_line(range, line_index, line_len),
        VisualSelection::Block(block) => {
            if line_index < block.start_line || line_index > block.end_line {
                return None;
            }
            let start = block.start_col.min(line_len);
            let end = block.end_col.min(line_len);
            (start < end).then_some((start, end))
        }
    }
}

fn index_syntax_lines(snapshot: &SyntaxSnapshot) -> BTreeMap<usize, Vec<LineSyntaxSpan>> {
    let mut syntax_lines = BTreeMap::new();
    for span in &snapshot.highlight_spans {
        for line_index in span.start_position.line..=span.end_position.line {
            let start = if line_index == span.start_position.line {
                span.start_position.column
            } else {
                0
            };
            let end = if line_index == span.end_position.line {
                span.end_position.column
            } else {
                usize::MAX
            };
            if start >= end {
                continue;
            }
            syntax_lines
                .entry(line_index)
                .or_insert_with(Vec::new)
                .push(LineSyntaxSpan {
                    start,
                    end,
                    theme_token: span.theme_token.clone(),
                });
        }
    }

    syntax_lines
}

fn clamp_to_char_boundary(text: &str, index: usize) -> usize {
    let mut clamped = index.min(text.len());
    while clamped > 0 && !text.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}

fn text_width(font: &Font<'_>, text: &str) -> Result<u32, ShellError> {
    Ok(font
        .size_of(text)
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0)
}

fn to_sdl_color(color: ThemeColor) -> Color {
    Color::RGBA(color.r, color.g, color.b, color.a)
}

fn to_render_color(color: Color) -> RenderColor {
    RenderColor::rgba(color.r, color.g, color.b, color.a)
}

fn from_render_color(color: RenderColor) -> Color {
    Color::RGBA(color.r, color.g, color.b, color.a)
}

fn to_pixel_rect(rect: Rect) -> PixelRect {
    PixelRect::new(rect.x(), rect.y(), rect.width(), rect.height())
}

fn present_scene_to_canvas(
    canvas: &mut Canvas<Window>,
    font: &Font<'_>,
    scene: &[DrawCommand],
) -> Result<(), ShellError> {
    for command in scene {
        match command {
            DrawCommand::Clear { color } => {
                canvas.set_draw_color(from_render_color(*color));
                canvas.clear();
            }
            DrawCommand::FillRect { rect, color } => {
                canvas.set_draw_color(from_render_color(*color));
                canvas
                    .fill_rect(PixelRectToRect::from_pixel_rect(*rect))
                    .map_err(|error| ShellError::Sdl(error.to_string()))?;
            }
            DrawCommand::FillRoundedRect {
                rect,
                radius,
                color,
            } => fill_rounded_rect_canvas(
                canvas,
                PixelRectToRect::from_pixel_rect(*rect),
                *radius,
                from_render_color(*color),
            )?,
            DrawCommand::Text { x, y, text, color } => {
                if text.is_empty() {
                    continue;
                }

                let surface = font
                    .render(text)
                    .blended(from_render_color(*color))
                    .map_err(|error| ShellError::Sdl(error.to_string()))?;
                let texture_creator = canvas.texture_creator();
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .map_err(|error| ShellError::Sdl(error.to_string()))?;
                canvas
                    .copy(
                        &texture,
                        None,
                        Rect::new(*x, *y, surface.width(), surface.height()),
                    )
                    .map_err(|error| ShellError::Sdl(error.to_string()))?;
            }
        }
    }

    canvas.present();
    Ok(())
}

fn draw_text(
    target: &mut DrawTarget<'_>,
    x: i32,
    y: i32,
    text: &str,
    color: Color,
) -> Result<(), ShellError> {
    if text.is_empty() {
        return Ok(());
    }

    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::Text {
            x,
            y,
            text: text.to_owned(),
            color: to_render_color(color),
        }),
    }

    Ok(())
}

fn fill_rect(target: &mut DrawTarget<'_>, rect: Rect, color: Color) -> Result<(), ShellError> {
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::FillRect {
            rect: to_pixel_rect(rect),
            color: to_render_color(color),
        }),
    }
    Ok(())
}

fn fill_rounded_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::FillRoundedRect {
            rect: to_pixel_rect(rect),
            radius,
            color: to_render_color(color),
        }),
    }
    Ok(())
}

fn fill_rounded_rect_canvas<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    let radius = radius.min(rect.width() / 2).min(rect.height() / 2) as i32;
    if radius <= 1 {
        canvas.set_draw_color(color);
        return canvas
            .fill_rect(rect)
            .map_err(|error| ShellError::Sdl(error.to_string()));
    }

    canvas.set_draw_color(color);
    let rect_height = rect.height() as i32;
    let rect_width = rect.width() as i32;

    for row in 0..rect_height {
        let inset = if row < radius {
            let dy = radius - row - 1;
            radius - ((radius * radius - dy * dy) as f64).sqrt().floor() as i32
        } else if row >= rect_height - radius {
            let dy = row - (rect_height - radius);
            radius - ((radius * radius - dy * dy) as f64).sqrt().floor() as i32
        } else {
            0
        };

        let width = rect_width - (inset * 2);
        if width <= 0 {
            continue;
        }

        canvas
            .fill_rect(Rect::new(rect.x() + inset, rect.y() + row, width as u32, 1))
            .map_err(|error| ShellError::Sdl(error.to_string()))?;
    }

    Ok(())
}

fn truncate_text_to_width(
    font: &Font<'_>,
    text: &str,
    max_width: u32,
) -> Result<String, ShellError> {
    if text.is_empty() || max_width == 0 {
        return Ok(String::new());
    }

    if font
        .size_of(text)
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0
        <= max_width
    {
        return Ok(text.to_owned());
    }

    let ellipsis = "...";
    let ellipsis_width = font
        .size_of(ellipsis)
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0;
    if ellipsis_width >= max_width {
        return Ok("...".to_owned());
    }

    let mut truncated = String::new();
    for character in text.chars() {
        let mut candidate = truncated.clone();
        candidate.push(character);
        candidate.push_str(ellipsis);
        if font
            .size_of(&candidate)
            .map_err(|error| ShellError::Sdl(error.to_string()))?
            .0
            > max_width
        {
            break;
        }
        truncated.push(character);
    }

    truncated.push_str(ellipsis);
    Ok(truncated)
}

struct PixelRectToRect;

impl PixelRectToRect {
    fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
        Rect::new(x, y, width, height)
    }

    fn from_pixel_rect(rect: PixelRect) -> Rect {
        Self::rect(rect.x, rect.y, rect.width, rect.height)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{Instant, SystemTime, UNIX_EPOCH},
    };

    use super::{
        BlockSelection, InputMode, ShellConfig, ShellState, VisualSelection, VisualSelectionKind,
        delete_runtime_workspace, open_workspace_from_project, switch_runtime_workspace,
        workspace_delete_picker_overlay, workspace_switch_picker_overlay,
    };
    use editor_buffer::{TextBuffer, TextPoint};
    use editor_core::{BufferKind, HookEvent};
    use editor_render::RenderBackend;
    use sdl3::keyboard::{Keycode, Mod};

    fn temp_workspace_root(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-workspace-{name}-{unique}"))
    }

    fn git_available() -> bool {
        Command::new("git").arg("--version").output().is_ok()
    }

    fn run_git(root: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new("git").args(args).current_dir(root).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("git {:?} failed with status {status}", args).into())
        }
    }

    #[test]
    fn vim_bindings_switch_modes_and_move_words() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");

        state.handle_text_input("w")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 6);

        state.handle_text_input("i")?;
        assert_eq!(state.input_mode()?, InputMode::Insert);

        state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?;
        assert_eq!(state.input_mode()?, InputMode::Normal);
        Ok(())
    }

    #[test]
    fn vim_extended_motions_and_edit_commands_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");

        state.handle_text_input("w")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 6);

        state.handle_text_input("b")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        state.handle_text_input("e")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 4);

        state.handle_text_input("$")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 9);

        state.handle_text_input("0")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        state.handle_text_input("A")?;
        assert_eq!(state.input_mode()?, InputMode::Insert);
        state.handle_text_input("!")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        assert_eq!(state.input_mode()?, InputMode::Normal);
        assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta!");

        state.handle_text_input("u")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta");

        assert!(state.try_runtime_keybinding(Keycode::R, Mod::LCTRLMOD)?);
        assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta!");

        Ok(())
    }

    #[test]
    fn vim_counts_operators_and_text_objects_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta gamma");

        state.handle_text_input("d")?;
        state.handle_text_input("w")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "beta gamma");

        state.handle_text_input("u")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta gamma");

        state.handle_text_input("c")?;
        state.handle_text_input("i")?;
        state.handle_text_input("w")?;
        assert_eq!(state.input_mode()?, InputMode::Insert);
        state.handle_text_input("delta")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        assert_eq!(state.active_buffer_mut()?.text.text(), "delta beta gamma");

        state.handle_text_input("y")?;
        state.handle_text_input("y")?;
        state.handle_text_input("p")?;
        assert_eq!(
            state.active_buffer_mut()?.text.text(),
            "delta beta gamma\ndelta beta gamma\n"
        );

        Ok(())
    }

    #[test]
    fn vim_visual_mode_and_find_repeats_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("bananas");

        state.handle_text_input("f")?;
        state.handle_text_input("a")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 1);

        state.handle_text_input(";")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 3);

        state.handle_text_input(",")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 1);

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
        state.handle_text_input("v")?;
        assert_eq!(state.input_mode()?, InputMode::Visual);
        state.handle_text_input("e")?;
        state.handle_text_input("d")?;
        assert_eq!(state.input_mode()?, InputMode::Normal);
        assert_eq!(state.active_buffer_mut()?.text.text(), " beta");

        Ok(())
    }

    #[test]
    fn vim_linewise_visual_mode_and_anchor_swap_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree\n");

        state.handle_text_input("V")?;
        assert_eq!(state.input_mode()?, InputMode::Visual);
        assert_eq!(state.ui()?.vim().visual_kind, VisualSelectionKind::Line);

        state.handle_text_input("j")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 1);

        state.handle_text_input("o")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);
        assert_eq!(state.ui()?.vim().visual_anchor, Some(TextPoint::new(1, 0)));

        state.handle_text_input("d")?;
        assert_eq!(state.input_mode()?, InputMode::Normal);
        assert_eq!(state.active_buffer_mut()?.text.text(), "three\n");

        Ok(())
    }

    #[test]
    fn vim_visual_block_insert_applies_to_all_lines() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree");
        state
            .active_buffer_mut()?
            .set_cursor(TextPoint::new(0, 0));

        state.handle_text_input("v")?;
        assert_eq!(state.input_mode()?, InputMode::Visual);
        assert!(state.try_runtime_keybinding(Keycode::V, Mod::LCTRLMOD)?);
        assert_eq!(state.ui()?.vim().visual_kind, VisualSelectionKind::Block);

        state.handle_text_input("j")?;
        state.handle_text_input("j")?;
        state.handle_text_input("I")?;
        assert_eq!(state.input_mode()?, InputMode::Insert);
        state.handle_text_input("x")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        assert_eq!(state.input_mode()?, InputMode::Normal);
        assert_eq!(state.active_buffer_mut()?.text.text(), "xone\nxtwo\nxthree");
        Ok(())
    }

    #[test]
    fn vim_visual_text_objects_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");

        state.handle_text_input("v")?;
        state.handle_text_input("i")?;
        state.handle_text_input("w")?;
        state.handle_text_input("d")?;
        assert_eq!(state.input_mode()?, InputMode::Normal);
        assert_eq!(state.active_buffer_mut()?.text.text(), " beta");

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));

        state.handle_text_input("v")?;
        state.handle_text_input("a")?;
        state.handle_text_input("w")?;
        state.handle_text_input("d")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "beta");

        Ok(())
    }

    #[test]
    fn vim_counted_line_end_and_aliases_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree");

        state.handle_text_input("2")?;
        state.handle_text_input("$")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 1);
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 2);

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta gamma");
        state.handle_text_input("w")?;
        state.handle_text_input("D")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "alpha ");

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta gamma");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("w")?;
        state.handle_text_input("C")?;
        assert_eq!(state.input_mode()?, InputMode::Insert);
        state.handle_text_input("delta")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        assert_eq!(state.active_buffer_mut()?.text.text(), "alpha delta");

        state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree\n");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("2")?;
        state.handle_text_input("Y")?;
        state.handle_text_input("p")?;
        assert_eq!(
            state.active_buffer_mut()?.text.text(),
            "one\none\ntwo\ntwo\nthree\n"
        );

        Ok(())
    }

    #[test]
    fn vim_substitute_delete_counts_and_visual_toggles_work()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");

        state.handle_text_input("2")?;
        state.handle_text_input("x")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "pha beta");

        state.handle_text_input("2")?;
        state.handle_text_input("s")?;
        assert_eq!(state.input_mode()?, InputMode::Insert);
        state.handle_text_input("Z")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        assert_eq!(state.active_buffer_mut()?.text.text(), "Za beta");

        state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree\n");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 1));
        state.handle_text_input("V")?;
        assert_eq!(state.ui()?.vim().visual_kind, VisualSelectionKind::Line);
        state.handle_text_input("v")?;
        assert_eq!(state.input_mode()?, InputMode::Visual);
        assert_eq!(
            state.ui()?.vim().visual_kind,
            VisualSelectionKind::Character
        );
        state.handle_text_input("v")?;
        assert_eq!(state.input_mode()?, InputMode::Normal);

        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("2")?;
        state.handle_text_input("S")?;
        assert_eq!(state.input_mode()?, InputMode::Insert);
        state.handle_text_input("hello")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        assert_eq!(state.active_buffer_mut()?.text.text(), "hello\nthree\n");

        Ok(())
    }

    #[test]
    fn vim_search_prompt_and_repeats_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("one two one two");

        state.handle_text_input("/")?;
        assert!(state.picker_visible()?);
        state.handle_text_input("one")?;
        assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
        assert!(!state.picker_visible()?);
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 8);

        state.handle_text_input("n")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        state.handle_text_input("N")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 8);

        state.handle_text_input("?")?;
        state.handle_text_input("two")?;
        assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 4);

        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("*")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 8);

        state.handle_text_input("#")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        Ok(())
    }

    #[test]
    fn vim_search_prompt_supports_fuzzy_matches() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("one two one two");

        state.handle_text_input("/")?;
        state.handle_text_input("otw")?;
        assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 8);

        Ok(())
    }

    #[test]
    fn vim_search_prompt_matches_case_insensitive() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("Users user");

        state.handle_text_input("/")?;
        state.handle_text_input("user")?;
        let picker = state.ui()?.picker().ok_or("missing search picker")?;
        assert_eq!(picker.session().match_count(), 2);

        Ok(())
    }

    #[test]
    fn vim_search_picker_selects_match_entries() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text =
            TextBuffer::from_text("alpha\nsplit here\nbeta\nsplit again\n");

        state.handle_text_input("/")?;
        state.handle_text_input("split")?;
        let picker = state.ui()?.picker().ok_or("missing search picker")?;
        assert!(picker.session().match_count() > 0);
        let selected = picker
            .session()
            .selected()
            .ok_or("missing search selection")?;
        let selected_id = selected.item().id();
        let (line, column) = selected_id
            .split_once(':')
            .ok_or("missing search id delimiter")?;
        let line: usize = line.parse()?;
        let column: usize = column.parse()?;

        assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
        assert_eq!(state.active_buffer_mut()?.cursor_row(), line);
        assert_eq!(state.active_buffer_mut()?.cursor_col(), column);
        Ok(())
    }

    #[test]
    fn vim_quickref_word_motions_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha-beta gamma");
        state.handle_text_input("W")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 11);

        state.handle_text_input("B")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        state.handle_text_input("E")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 9);

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta gamma");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 11));
        state.handle_text_input("g")?;
        state.handle_text_input("e")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 9);

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha-beta gamma");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 11));
        state.handle_text_input("g")?;
        state.handle_text_input("E")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 9);

        state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar])");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 4));
        state.handle_text_input("%")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 13);

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha-beta gamma");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("d")?;
        state.handle_text_input("W")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "gamma");

        state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar])");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 4));
        state.handle_text_input("d")?;
        state.handle_text_input("%")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "call");

        Ok(())
    }

    #[test]
    fn vim_quickref_structure_motions_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;

        state.active_buffer_mut()?.text = TextBuffer::from_text("Alpha. Bravo! Charlie?");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input(")")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 7);

        state.handle_text_input("(")?;
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\n\nthree\nfour\n\nfive");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(3, 1));
        state.handle_text_input("{")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 2);

        state.handle_text_input("{")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        state.active_buffer_mut()?.set_cursor(TextPoint::new(3, 1));
        state.handle_text_input("{")?;
        state.handle_text_input("}")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 5);

        state.handle_text_input("}")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 6);

        state.active_buffer_mut()?.set_viewport_lines(4);
        state.active_buffer_mut()?.scroll_row = 3;
        state.handle_text_input("H")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 3);

        state.handle_text_input("M")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 4);

        state.handle_text_input("L")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 6);

        state.active_buffer_mut()?.scroll_row = 0;
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        assert!(state.try_runtime_keybinding(Keycode::D, Mod::LCTRLMOD)?);
        assert_eq!(state.active_buffer_mut()?.scroll_row, 2);
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 2);

        assert!(state.try_runtime_keybinding(Keycode::U, Mod::LCTRLMOD)?);
        assert_eq!(state.active_buffer_mut()?.scroll_row, 0);
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);

        assert!(state.try_runtime_keybinding(Keycode::F, Mod::LCTRLMOD)?);
        assert_eq!(state.active_buffer_mut()?.scroll_row, 4);
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 4);

        assert!(state.try_runtime_keybinding(Keycode::Y, Mod::LCTRLMOD)?);
        assert_eq!(state.active_buffer_mut()?.scroll_row, 3);
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 4);

        assert!(state.try_runtime_keybinding(Keycode::E, Mod::LCTRLMOD)?);
        assert_eq!(state.active_buffer_mut()?.scroll_row, 4);
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 4);

        Ok(())
    }

    #[test]
    fn vim_quickref_change_ops_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha");
        state.handle_text_input("r")?;
        state.handle_text_input("Z")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "Zlpha");
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha");
        state.handle_text_input("2")?;
        state.handle_text_input("r")?;
        state.handle_text_input("x")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "xxpha");

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha");
        state.handle_text_input("R")?;
        assert_eq!(state.input_mode()?, InputMode::Replace);
        state.handle_text_input("XYZ")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        assert_eq!(state.input_mode()?, InputMode::Normal);
        assert_eq!(state.active_buffer_mut()?.text.text(), "XYZha");

        state.active_buffer_mut()?.text = TextBuffer::from_text("aBcD");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("~")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "ABcD");
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 1);

        state.active_buffer_mut()?.text = TextBuffer::from_text("aBcD");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("3")?;
        state.handle_text_input("~")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "AbCD");
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 3);

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("g")?;
        state.handle_text_input("U")?;
        state.handle_text_input("w")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "ALPHA beta");

        state.active_buffer_mut()?.text = TextBuffer::from_text("ALPHA beta");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("g")?;
        state.handle_text_input("u")?;
        state.handle_text_input("w")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta");

        state.active_buffer_mut()?.text = TextBuffer::from_text("ABcd");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("v")?;
        state.handle_text_input("l")?;
        state.handle_text_input("u")?;
        assert_eq!(state.input_mode()?, InputMode::Normal);
        assert_eq!(state.active_buffer_mut()?.text.text(), "abcd");

        Ok(())
    }

    #[test]
    fn vim_quickref_repeat_registers_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("d")?;
        state.handle_text_input("w")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "beta");

        state.handle_text_input(".")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "");

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha");
        state.handle_text_input("q")?;
        state.handle_text_input("a")?;
        state.handle_text_input("A")?;
        state.handle_text_input("!")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        state.handle_text_input("q")?;
        assert!(state
            .ui()?
            .vim()
            .macros
            .contains_key(&'a'));

        state.active_buffer_mut()?.text = TextBuffer::from_text("beta");
        state.handle_text_input("@")?;
        state.handle_text_input("a")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "beta!");

        state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\n");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 1));
        state.handle_text_input("m")?;
        state.handle_text_input("a")?;
        state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 0));
        state.handle_text_input("'")?;
        state.handle_text_input("a")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

        state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 0));
        state.handle_text_input("`")?;
        state.handle_text_input("a")?;
        assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);
        assert_eq!(state.active_buffer_mut()?.cursor_col(), 1);

        Ok(())
    }

    #[test]
    fn vim_yanks_set_flash_selections() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta\ngamma delta");
        let buffer_id = state.active_buffer_mut()?.id();

        state.handle_text_input("y")?;
        state.handle_text_input("w")?;
        let line_flash = state.ui()?.yank_flash(buffer_id, Instant::now());
        assert_eq!(
            line_flash,
            Some(VisualSelection::Range(
                state
                    .active_buffer_mut()?
                    .line_range(0)
                    .ok_or("missing line range")?
            ))
        );

        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        state.handle_text_input("V")?;
        state.handle_text_input("j")?;
        state.handle_text_input("y")?;
        let line_flash = state.ui()?.yank_flash(buffer_id, Instant::now());
        assert_eq!(
            line_flash,
            Some(VisualSelection::Range(
                state
                    .active_buffer_mut()?
                    .line_span_range(0, 2)
                    .ok_or("missing span")?
            ))
        );

        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
        assert!(state.try_runtime_keybinding(Keycode::V, Mod::LCTRLMOD)?);
        assert_eq!(state.ui()?.vim().visual_kind, VisualSelectionKind::Block);
        state.handle_text_input("l")?;
        state.handle_text_input("j")?;
        state.handle_text_input("y")?;
        assert_eq!(
            state.ui()?.yank_flash(buffer_id, Instant::now()),
            Some(VisualSelection::Block(BlockSelection {
                start_line: 0,
                end_line: 1,
                start_col: 0,
                end_col: 2,
            }))
        );

        Ok(())
    }

    #[test]
    fn vim_delimited_text_objects_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar], \"baz\")");

        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 17));
        state.handle_text_input("c")?;
        state.handle_text_input("i")?;
        state.handle_text_input("\"")?;
        assert_eq!(state.input_mode()?, InputMode::Insert);
        state.handle_text_input("qux")?;
        assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
        assert_eq!(
            state.active_buffer_mut()?.text.text(),
            "call(foo[bar], \"qux\")"
        );

        state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar], \"baz\")");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 9));
        state.handle_text_input("v")?;
        state.handle_text_input("i")?;
        state.handle_text_input("[")?;
        state.handle_text_input("d")?;
        assert_eq!(
            state.active_buffer_mut()?.text.text(),
            "call(foo[], \"baz\")"
        );

        state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar], \"baz\")");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 6));
        state.handle_text_input("d")?;
        state.handle_text_input("a")?;
        state.handle_text_input("(")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "call");

        state.active_buffer_mut()?.text = TextBuffer::from_text("outer{alpha(beta)}");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 12));
        state.handle_text_input("d")?;
        state.handle_text_input("i")?;
        state.handle_text_input("b")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "outer{alpha()}");

        state.active_buffer_mut()?.text = TextBuffer::from_text("outer{alpha(beta)}");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 12));
        state.handle_text_input("d")?;
        state.handle_text_input("a")?;
        state.handle_text_input("B")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "outer");

        Ok(())
    }

    #[test]
    fn vim_quickref_text_objects_work() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;

        state.active_buffer_mut()?.text = TextBuffer::from_text("alpha-beta gamma");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 7));
        state.handle_text_input("d")?;
        state.handle_text_input("i")?;
        state.handle_text_input("W")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), " gamma");

        state.active_buffer_mut()?.text = TextBuffer::from_text("Alpha. Bravo! Charlie?");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 9));
        state.handle_text_input("d")?;
        state.handle_text_input("i")?;
        state.handle_text_input("s")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "Alpha.  Charlie?");

        state.active_buffer_mut()?.text = TextBuffer::from_text("one\n\nalpha\nbeta\n\ntwo\n");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(2, 1));
        state.handle_text_input("d")?;
        state.handle_text_input("i")?;
        state.handle_text_input("p")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "one\n\n\ntwo\n");

        state.active_buffer_mut()?.text = TextBuffer::from_text("foo <bar> baz");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 5));
        state.handle_text_input("d")?;
        state.handle_text_input("i")?;
        state.handle_text_input(">")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "foo <> baz");

        state.active_buffer_mut()?.text = TextBuffer::from_text("<div>hello</div>");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 6));
        state.handle_text_input("d")?;
        state.handle_text_input("i")?;
        state.handle_text_input("t")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "<div></div>");

        state.active_buffer_mut()?.text = TextBuffer::from_text("<div>hello</div>");
        state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 6));
        state.handle_text_input("v")?;
        state.handle_text_input("i")?;
        state.handle_text_input("t")?;
        state.handle_text_input("d")?;
        assert_eq!(state.active_buffer_mut()?.text.text(), "<div></div>");

        Ok(())
    }

    #[test]
    fn picker_bindings_open_and_navigate_results() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;

        assert!(state.try_runtime_keybinding(Keycode::F4, Mod::NOMOD)?);
        let initial = state
            .ui()?
            .picker()
            .and_then(|picker| picker.session().selected())
            .map(|selected| selected.item().id().to_owned())
            .ok_or("missing initial picker selection")?;

        assert!(state.try_runtime_keybinding(Keycode::N, Mod::LCTRLMOD)?);
        let next = state
            .ui()?
            .picker()
            .and_then(|picker| picker.session().selected())
            .map(|selected| selected.item().id().to_owned())
            .ok_or("missing next picker selection")?;

        assert_ne!(initial, next);

        assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
        assert!(!state.picker_visible()?);
        assert_eq!(state.active_buffer_mut()?.id().to_string(), next);
        Ok(())
    }

    #[test]
    fn keybinding_picker_lists_runtime_bindings() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;

        assert!(state.try_runtime_keybinding(Keycode::F6, Mod::NOMOD)?);
        let picker = state.ui()?.picker().ok_or("missing keybinding picker")?;
        assert_eq!(picker.session().title(), "Keybindings");
        assert!(picker.session().matches().iter().any(|matched| {
            matched.item().label() == "F3"
                && matched.item().detail().contains("picker.open-commands")
        }));

        state.handle_text_input("h")?;
        let picker = state.ui()?.picker().ok_or("missing keybinding picker")?;
        assert!(picker.session().matches().iter().any(|matched| {
            matched.item().label() == "h" && matched.item().detail().contains("[normal]")
        }));

        Ok(())
    }

    #[test]
    fn shell_defaults_to_sdl_canvas_backend() {
        assert_eq!(
            ShellConfig::default().render_backend,
            RenderBackend::SdlCanvas
        );
    }

    #[test]
    fn lsp_hook_updates_statusline_state() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        let workspace_id = state.runtime.model().active_workspace_id()?;

        state
            .runtime
            .emit_hook(
                "lsp.server-start",
                HookEvent::new()
                    .with_workspace(workspace_id)
                    .with_detail("rust-analyzer"),
            )
            .map_err(|error| error.to_string())?;

        assert_eq!(state.ui()?.attached_lsp_server(), Some("rust-analyzer"));
        assert_eq!(state.runtime.model().active_workspace()?.name(), "default");
        Ok(())
    }

    #[test]
    fn workspace_helpers_open_switch_and_delete_workspaces()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        let default_workspace = state.runtime.model().active_workspace_id()?;
        let root = temp_workspace_root("demo");
        fs::create_dir_all(&root)?;

        let demo_workspace = open_workspace_from_project(&mut state.runtime, "demo", &root)?;
        assert_eq!(state.runtime.model().active_workspace()?.name(), "demo");
        assert_eq!(state.ui()?.active_workspace(), demo_workspace);

        switch_runtime_workspace(&mut state.runtime, default_workspace)?;
        assert_eq!(
            state.runtime.model().active_workspace_id()?,
            default_workspace
        );
        assert_eq!(state.ui()?.active_workspace(), default_workspace);

        delete_runtime_workspace(&mut state.runtime, demo_workspace)?;
        assert_eq!(
            state.runtime.model().active_workspace_id()?,
            default_workspace
        );
        assert_eq!(state.ui()?.active_workspace(), default_workspace);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn workspace_delete_picker_hides_default_workspace() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ShellState::new()?;
        let root = temp_workspace_root("picker");
        fs::create_dir_all(&root)?;
        let _workspace_id = open_workspace_from_project(&mut state.runtime, "picker", &root)?;

        let switch_picker = workspace_switch_picker_overlay(&state.runtime)?;
        assert!(
            switch_picker
                .session()
                .matches()
                .iter()
                .any(|matched| matched.item().label() == "default")
        );
        assert!(
            switch_picker
                .session()
                .matches()
                .iter()
                .any(|matched| matched.item().label() == "picker")
        );

        let delete_picker = workspace_delete_picker_overlay(&state.runtime)?;
        assert!(
            delete_picker
                .session()
                .matches()
                .iter()
                .all(|matched| matched.item().label() != "default")
        );
        assert!(
            delete_picker
                .session()
                .matches()
                .iter()
                .any(|matched| matched.item().label() == "picker")
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn workspace_file_picker_lists_visible_files_and_opens_selection()
    -> Result<(), Box<dyn std::error::Error>> {
        if !git_available() {
            return Ok(());
        }

        let mut state = ShellState::new()?;
        let root = temp_workspace_root("files");
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join(".gitignore"), "ignored.txt\n")?;
        fs::write(
            root.join("src").join("main.rs"),
            "fn main() {\n    println!(\"hi\");\n}\n",
        )?;
        fs::write(root.join("ignored.txt"), "ignored\n")?;

        run_git(&root, &["init", "-q"])?;
        run_git(&root, &["add", ".gitignore", "src/main.rs"])?;
        open_workspace_from_project(&mut state.runtime, "files", &root)?;

        state
            .runtime
            .execute_command("workspace.list-files")
            .map_err(|error| error.to_string())?;
        let picker = state
            .ui()?
            .picker()
            .ok_or("missing workspace file picker")?;
        assert_eq!(picker.session().title(), "Workspace Files");
        assert!(
            picker
                .session()
                .matches()
                .iter()
                .any(|matched| matched.item().label().contains("main.rs"))
        );
        assert!(
            picker
                .session()
                .matches()
                .iter()
                .all(|matched| !matched.item().label().contains("ignored.txt"))
        );

        state.handle_text_input("main.rs")?;
        assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
        assert!(!state.picker_visible()?);
        let active = state.active_buffer_mut()?;
        assert_eq!(active.kind, BufferKind::File);
        assert!(active.display_name().contains("main.rs"));
        assert_eq!(active.text.line(0).as_deref(), Some("fn main() {"));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
