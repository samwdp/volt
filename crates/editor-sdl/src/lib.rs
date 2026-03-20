#![doc = r#"SDL3 windowing, input, and demo shell rendering for the native editor."#]

use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use editor_buffer::TextBuffer;
use editor_core::{
    Buffer, BufferId, BufferKind, EditorRuntime, HookEvent, KeymapScope, KeymapVimMode,
    WorkspaceId, builtins,
};
use editor_fs::discover_projects;
use editor_git::list_repository_files;
use editor_picker::{PickerItem, PickerSession};
use editor_plugin_host::load_auto_loaded_packages;
use editor_render::{
    RenderError, centered_rect, find_system_monospace_font, horizontal_pane_rects,
};
use editor_syntax::{SyntaxError, SyntaxRegistry, SyntaxSnapshot};
use editor_theme::{Color as ThemeColor, ThemeRegistry};
use sdl3::{
    event::Event,
    keyboard::{Keycode, Mod},
    pixels::Color,
    rect::Rect,
    render::Canvas,
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
const HOOK_MOVE_LINE_START: &str = "editor.cursor.move-line-start";
const HOOK_MOVE_LINE_FIRST_NON_BLANK: &str = "editor.cursor.move-line-first-non-blank";
const HOOK_MOVE_LINE_END: &str = "editor.cursor.move-line-end";
const HOOK_GOTO_FIRST_LINE: &str = "editor.cursor.goto-first-line";
const HOOK_GOTO_LAST_LINE: &str = "editor.cursor.goto-last-line";
const HOOK_MODE_INSERT: &str = "editor.mode.insert";
const HOOK_MODE_NORMAL: &str = "editor.mode.normal";
const HOOK_VIM_EDIT: &str = "editor.vim.edit";
const HOOK_PICKER_OPEN: &str = "ui.picker.open";
const HOOK_PICKER_NEXT: &str = "ui.picker.next";
const HOOK_PICKER_PREVIOUS: &str = "ui.picker.previous";
const HOOK_PICKER_SUBMIT: &str = "ui.picker.submit";
const HOOK_PICKER_CANCEL: &str = "ui.picker.cancel";
const HOOK_POPUP_TOGGLE: &str = "ui.popup.toggle";

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
            frame_limit: None,
        }
    }
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
}

impl InputMode {
    fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
        }
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

    fn line_count(&self) -> usize {
        self.text.line_count()
    }

    fn visible_lines(&self, max_lines: usize) -> Vec<String> {
        self.text.lines(self.scroll_row, max_lines)
    }

    fn path(&self) -> Option<&Path> {
        self.text.path()
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

    fn insert_newline(&mut self) {
        self.text.insert_newline();
    }

    fn backspace(&mut self) {
        let _ = self.text.backspace();
    }

    fn move_left(&mut self) {
        let _ = self.text.move_left();
    }

    fn move_right(&mut self) {
        let _ = self.text.move_right();
    }

    fn move_up(&mut self) {
        let _ = self.text.move_up();
    }

    fn move_down(&mut self) {
        let _ = self.text.move_down();
    }

    fn move_word_forward(&mut self) {
        let _ = self.text.move_word_forward();
    }

    fn move_word_backward(&mut self) {
        let _ = self.text.move_word_backward();
    }

    fn move_word_end(&mut self) {
        let _ = self.text.move_word_end_forward();
    }

    fn move_line_start(&mut self) {
        self.text
            .set_cursor(editor_buffer::TextPoint::new(self.cursor_row(), 0));
    }

    fn move_line_first_non_blank(&mut self) {
        if let Some(point) = self.text.first_non_blank_in_line(self.cursor_row()) {
            self.text.set_cursor(point);
        }
    }

    fn move_line_end(&mut self) {
        let line = self.cursor_row();
        let column = self
            .text
            .line_len_chars(line)
            .map(|line_len| line_len.saturating_sub(1))
            .unwrap_or(0);
        self.text
            .set_cursor(editor_buffer::TextPoint::new(line, column));
    }

    fn goto_first_line(&mut self) {
        if let Some(point) = self.text.first_non_blank_in_line(0) {
            self.text.set_cursor(point);
        }
    }

    fn goto_last_line(&mut self) {
        let line = self.line_count().saturating_sub(1);
        if let Some(point) = self.text.first_non_blank_in_line(line) {
            self.text.set_cursor(point);
        }
    }

    fn delete_char(&mut self) {
        let _ = self.text.delete_forward();
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

    fn scroll_by(&mut self, delta: i32) {
        let max_scroll = self.line_count().saturating_sub(1) as i32;
        let next = (self.scroll_row as i32 + delta).clamp(0, max_scroll);
        self.scroll_row = next as usize;
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
        }
    }

    fn session(&self) -> &PickerSession {
        &self.session
    }

    fn selected_action(&self) -> Option<PickerAction> {
        let selected = self.session.selected()?;
        self.actions.get(selected.item().id()).cloned()
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
    attached_lsp_servers: BTreeMap<WorkspaceId, String>,
    picker: Option<PickerOverlay>,
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
            attached_lsp_servers: BTreeMap::new(),
            picker: None,
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

    fn set_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
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
        register_lsp_status_hooks(&mut runtime).map_err(ShellError::Runtime)?;

        Ok(Self { runtime })
    }

    fn handle_event(&mut self, event: Event, visible_lines: usize) -> Result<bool, ShellError> {
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
                    }
                    self.ensure_visible(visible_lines)?;
                    return Ok(false);
                }

                match keycode {
                    Keycode::Left => self.active_buffer_mut()?.move_left(),
                    Keycode::Right => self.active_buffer_mut()?.move_right(),
                    Keycode::Up => self.active_buffer_mut()?.move_up(),
                    Keycode::Down => self.active_buffer_mut()?.move_down(),
                    Keycode::PageDown => self.active_buffer_mut()?.scroll_by(visible_lines as i32),
                    Keycode::PageUp => self.active_buffer_mut()?.scroll_by(-(visible_lines as i32)),
                    Keycode::Return | Keycode::KpEnter
                        if self.input_mode()? == InputMode::Insert =>
                    {
                        self.active_buffer_mut()?.insert_newline();
                        self.mark_active_buffer_syntax_dirty()?;
                    }
                    Keycode::Backspace if self.input_mode()? == InputMode::Insert => {
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
        canvas: &mut Canvas<Window>,
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
            canvas,
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

    fn handle_text_input(&mut self, text: &str) -> Result<(), ShellError> {
        if self.picker_visible()? {
            if let Some(picker) = self.ui_mut()?.picker_mut() {
                picker.append_query(text);
            }
            return Ok(());
        }

        if self.input_mode()? == InputMode::Insert {
            self.active_buffer_mut()?.insert_text(text);
            self.mark_active_buffer_syntax_dirty()?;
            return Ok(());
        }

        if let Some(chord) = text_chord(text)
            && self.runtime.keymaps().contains_for_mode(
                &KeymapScope::Workspace,
                keymap_vim_mode(self.input_mode()?),
                &chord,
            )
        {
            self.runtime
                .execute_key_binding_for_mode(
                    &KeymapScope::Workspace,
                    keymap_vim_mode(self.input_mode()?),
                    &chord,
                )
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.sync_active_buffer().map_err(ShellError::Runtime)?;
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
            return Ok(true);
        }

        if !self.picker_visible()?
            && self.input_mode()? != InputMode::Insert
            && self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
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

    let font_path = find_system_monospace_font()?;
    let mut window_builder = video.window(&config.title, config.width, config.height);
    window_builder.position_centered().resizable();
    if config.hidden {
        window_builder.hidden();
    }
    let window = window_builder
        .build()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    video.text_input().start(&window);

    let font = ttf
        .load_font(&font_path, config.font_size as f32)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let line_height = font.height().max(1) as usize;
    let ascent = font.ascent();
    let cell_width = font
        .size_of_char('M')
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0
        .max(1) as i32;

    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let mut state = ShellState::new()?;
    let mut frames_rendered = 0;

    loop {
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
                    font_path: font_path.display().to_string(),
                });
            }
        }

        state.refresh_pending_syntax()?;

        state.render(
            &mut canvas,
            &font,
            render_width,
            render_height,
            cell_width,
            line_height as i32,
            ascent,
        )?;
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
        font_path: font_path.display().to_string(),
    })
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
        HOOK_POPUP_TOGGLE,
        "Shows or closes the docked popup window.",
    )?;

    runtime
        .subscribe_hook(HOOK_MOVE_LEFT, "shell.move-left", |_, runtime| {
            shell_ui_mut(runtime)?
                .active_buffer_mut()
                .ok_or_else(|| "active shell buffer missing".to_owned())?
                .move_left();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_DOWN, "shell.move-down", |_, runtime| {
            shell_ui_mut(runtime)?
                .active_buffer_mut()
                .ok_or_else(|| "active shell buffer missing".to_owned())?
                .move_down();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_UP, "shell.move-up", |_, runtime| {
            shell_ui_mut(runtime)?
                .active_buffer_mut()
                .ok_or_else(|| "active shell buffer missing".to_owned())?
                .move_up();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_RIGHT, "shell.move-right", |_, runtime| {
            shell_ui_mut(runtime)?
                .active_buffer_mut()
                .ok_or_else(|| "active shell buffer missing".to_owned())?
                .move_right();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_WORD_FORWARD,
            "shell.move-word-forward",
            |_, runtime| {
                shell_ui_mut(runtime)?
                    .active_buffer_mut()
                    .ok_or_else(|| "active shell buffer missing".to_owned())?
                    .move_word_forward();
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_WORD_BACKWARD,
            "shell.move-word-backward",
            |_, runtime| {
                shell_ui_mut(runtime)?
                    .active_buffer_mut()
                    .ok_or_else(|| "active shell buffer missing".to_owned())?
                    .move_word_backward();
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_WORD_END, "shell.move-word-end", |_, runtime| {
            shell_ui_mut(runtime)?
                .active_buffer_mut()
                .ok_or_else(|| "active shell buffer missing".to_owned())?
                .move_word_end();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_LINE_START,
            "shell.move-line-start",
            |_, runtime| {
                shell_ui_mut(runtime)?
                    .active_buffer_mut()
                    .ok_or_else(|| "active shell buffer missing".to_owned())?
                    .move_line_start();
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_LINE_FIRST_NON_BLANK,
            "shell.move-line-first-non-blank",
            |_, runtime| {
                shell_ui_mut(runtime)?
                    .active_buffer_mut()
                    .ok_or_else(|| "active shell buffer missing".to_owned())?
                    .move_line_first_non_blank();
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_LINE_END, "shell.move-line-end", |_, runtime| {
            shell_ui_mut(runtime)?
                .active_buffer_mut()
                .ok_or_else(|| "active shell buffer missing".to_owned())?
                .move_line_end();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_GOTO_FIRST_LINE,
            "shell.goto-first-line",
            |_, runtime| {
                shell_ui_mut(runtime)?
                    .active_buffer_mut()
                    .ok_or_else(|| "active shell buffer missing".to_owned())?
                    .goto_first_line();
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_GOTO_LAST_LINE, "shell.goto-last-line", |_, runtime| {
            shell_ui_mut(runtime)?
                .active_buffer_mut()
                .ok_or_else(|| "active shell buffer missing".to_owned())?
                .goto_last_line();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MODE_INSERT, "shell.enter-insert-mode", |_, runtime| {
            shell_ui_mut(runtime)?.set_input_mode(InputMode::Insert);
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MODE_NORMAL, "shell.enter-normal-mode", |_, runtime| {
            let ui = shell_ui_mut(runtime)?;
            if ui.input_mode() == InputMode::Insert
                && let Some(buffer) = ui.active_buffer_mut()
            {
                buffer.move_left();
            }
            ui.set_input_mode(InputMode::Normal);
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_VIM_EDIT, "shell.vim-edit", |event, runtime| {
            let detail = event.detail.as_deref().unwrap_or_default();
            let ui = shell_ui_mut(runtime)?;
            let buffer = ui
                .active_buffer_mut()
                .ok_or_else(|| "active shell buffer missing".to_owned())?;
            match detail {
                "delete-char" => {
                    buffer.delete_char();
                    buffer.mark_syntax_dirty();
                }
                "append" => {
                    buffer.append_after_cursor();
                    ui.set_input_mode(InputMode::Insert);
                }
                "append-line-end" => {
                    buffer.append_line_end();
                    ui.set_input_mode(InputMode::Insert);
                }
                "insert-line-start" => {
                    buffer.insert_line_start();
                    ui.set_input_mode(InputMode::Insert);
                }
                "open-line-below" => {
                    buffer.open_line_below();
                    buffer.mark_syntax_dirty();
                    ui.set_input_mode(InputMode::Insert);
                }
                "open-line-above" => {
                    buffer.open_line_above();
                    buffer.mark_syntax_dirty();
                    ui.set_input_mode(InputMode::Insert);
                }
                "undo" => {
                    buffer.undo();
                    buffer.mark_syntax_dirty();
                }
                "redo" => {
                    buffer.redo();
                    buffer.mark_syntax_dirty();
                }
                _ => {}
            }
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
            let action = {
                let ui = shell_ui_mut(runtime)?;
                let action = ui
                    .picker()
                    .and_then(PickerOverlay::selected_action)
                    .ok_or_else(|| "picker has no selected item".to_owned())?;
                ui.close_picker();
                action
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

fn syntax_registry_mut(runtime: &mut EditorRuntime) -> Result<&mut SyntaxRegistry, String> {
    runtime
        .services_mut()
        .get_mut::<SyntaxRegistry>()
        .ok_or_else(|| "syntax registry service missing".to_owned())
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

    PickerOverlay::from_entries("Keybindings", entries)
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
            Keycode::N => Some("Ctrl+n".to_owned()),
            Keycode::P => Some("Ctrl+p".to_owned()),
            Keycode::R => Some("Ctrl+r".to_owned()),
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
        InputMode::Insert => KeymapVimMode::Insert,
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
    canvas: &mut Canvas<Window>,
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

    canvas.set_draw_color(Color::RGB(15, 16, 20));
    canvas.clear();

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
            canvas,
            PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
            background,
        )?;
        fill_rect(
            canvas,
            PixelRectToRect::rect(rect.x, rect.y, rect.width, 1),
            Color::RGB(50, 55, 66),
        )?;

        if let Some(buffer) = state.buffer(pane.buffer_id) {
            render_buffer(
                canvas,
                font,
                buffer,
                PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
                active,
                state.input_mode(),
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
            canvas,
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
        )?;
    }

    if let Some(picker) = state.picker() {
        render_picker_overlay(canvas, font, picker, width, height, line_height)?;
    }
    canvas.present();

    Ok(())
}

fn render_picker_overlay(
    canvas: &mut Canvas<Window>,
    font: &Font<'_>,
    picker: &PickerOverlay,
    width: u32,
    height: u32,
    line_height: i32,
) -> Result<(), ShellError> {
    let popup_rect = centered_rect(width, height, width * 2 / 3, height * 3 / 5);
    fill_rounded_rect(
        canvas,
        PixelRectToRect::rect(
            popup_rect.x,
            popup_rect.y,
            popup_rect.width,
            popup_rect.height,
        ),
        16,
        Color::RGB(29, 32, 40),
    )?;
    fill_rect(
        canvas,
        PixelRectToRect::rect(
            popup_rect.x + 14,
            popup_rect.y,
            popup_rect.width.saturating_sub(28),
            2,
        ),
        Color::RGB(110, 170, 255),
    )?;

    draw_text(
        canvas,
        font,
        popup_rect.x + 16,
        popup_rect.y + 16,
        picker.session().title(),
        Color::RGBA(215, 221, 232, 255),
    )?;

    let query = format!("Query > {}", picker.session().query());
    draw_text(
        canvas,
        font,
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
        canvas,
        font,
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
            canvas,
            font,
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
                canvas,
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
            canvas,
            font,
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
            canvas,
            font,
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
            canvas,
            font,
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
    canvas: &mut Canvas<Window>,
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
) -> Result<(), ShellError> {
    fill_rect(canvas, popup_rect, Color::RGB(29, 32, 40))?;
    fill_rect(
        canvas,
        PixelRectToRect::rect(
            popup_rect.x() + 12,
            popup_rect.y(),
            popup_rect.width().saturating_sub(24),
            2,
        ),
        Color::RGB(110, 170, 255),
    )?;
    draw_text(
        canvas,
        font,
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
                canvas,
                font,
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
        render_buffer(
            canvas,
            font,
            buffer,
            popup_buffer_rect,
            true,
            state.input_mode(),
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
    canvas: &mut Canvas<Window>,
    font: &Font<'_>,
    buffer: &ShellBuffer,
    rect: Rect,
    active: bool,
    input_mode: InputMode,
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
    let statusline = truncate_text_to_width(
        font,
        &user::statusline::compose(&user::statusline::StatuslineContext {
            vim_mode: input_mode.label(),
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
            InputMode::Normal => cell_width.max(2) as u32,
            InputMode::Insert => (cell_width / 4).max(2) as u32,
        };
        fill_rounded_rect(
            canvas,
            PixelRectToRect::rect(
                rect.x() + 12 + line_number_width + (cursor_col as i32 * cell_width),
                body_y + cursor_row_on_screen as i32 * line_height,
                cursor_width,
                line_height.max(2) as u32,
            ),
            2,
            cursor,
        )?;
    }

    for (row_offset, line) in buffer.visible_lines(visible_lines).into_iter().enumerate() {
        let y = body_y + row_offset as i32 * line_height;
        draw_text(
            canvas,
            font,
            rect.x() + 12,
            y,
            &format!("{:>4}", buffer.scroll_row + row_offset + 1),
            muted,
        )?;
        draw_buffer_text(
            canvas,
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
        canvas,
        PixelRectToRect::rect(
            rect.x() + 8,
            statusline_y - 6,
            rect.width().saturating_sub(16),
            1,
        ),
        Color::RGB(50, 55, 66),
    )?;
    draw_text(
        canvas,
        font,
        rect.x() + 12,
        statusline_y,
        &statusline,
        title_color,
    )?;

    let _ = ascent;
    fill_rect(
        canvas,
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
    canvas: &mut Canvas<Window>,
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
        draw_text(canvas, font, draw_x, y, &segment, color)?;
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

fn draw_text(
    canvas: &mut Canvas<Window>,
    font: &Font<'_>,
    x: i32,
    y: i32,
    text: &str,
    color: Color,
) -> Result<(), ShellError> {
    if text.is_empty() {
        return Ok(());
    }

    let surface = font
        .render(text)
        .blended(color)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    canvas
        .copy(
            &texture,
            None,
            Rect::new(x, y, surface.width(), surface.height()),
        )
        .map_err(|error| ShellError::Sdl(error.to_string()))?;

    Ok(())
}

fn fill_rect(canvas: &mut Canvas<Window>, rect: Rect, color: Color) -> Result<(), ShellError> {
    canvas.set_draw_color(color);
    canvas
        .fill_rect(rect)
        .map_err(|error| ShellError::Sdl(error.to_string()))
}

fn fill_rounded_rect(
    canvas: &mut Canvas<Window>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    let radius = radius.min(rect.width() / 2).min(rect.height() / 2) as i32;
    if radius <= 1 {
        return fill_rect(canvas, rect, color);
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
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        InputMode, ShellState, delete_runtime_workspace, open_workspace_from_project,
        switch_runtime_workspace, workspace_delete_picker_overlay, workspace_switch_picker_overlay,
    };
    use editor_buffer::TextBuffer;
    use editor_core::{BufferKind, HookEvent};
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
        assert!(picker.session().matches().iter().any(|matched| {
            matched.item().label() == "h" && matched.item().detail().contains("[normal]")
        }));

        Ok(())
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
