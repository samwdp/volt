#![doc = r#"Shared extension-facing types used by the core editor and the compiled user library."#]

pub mod abi;
pub mod treesitter;

use abi_stable::{
    StableAbi,
    std_types::{ROption, RString, RVec},
};

pub use editor_core::{Section, SectionAction, SectionItem, SectionTree};
pub use editor_dap::DebugAdapterSpec;
pub use editor_fs::{DirectoryEntry, DirectoryEntryKind, ProjectSearchRoot};
pub use editor_git::{GitStatusSnapshot, StatusEntry};
pub use editor_icons::{IconFontCategory, IconFontSymbol};
pub use editor_lsp::{LanguageServerRootStrategy, LanguageServerSpec, LspCompletionKind};
pub use editor_syntax::{
    CaptureThemeMapping, GrammarSource, LanguageConfiguration, SyntaxNodeContext, SyntaxPoint,
};
pub use editor_theme::{Color, Theme, ThemeOption};

pub use abi::{
    AbiAcpClient, AbiAutocompleteProvider, AbiBrowserFeatureSpec, AbiCaptureThemeMapping, AbiColor,
    AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, AbiDebugAdapterSpec,
    AbiDirectoryEntry, AbiDirectoryEntryKind, AbiGhostTextContext, AbiGhostTextLine,
    AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitLogEntry, AbiGitPrefixBinding, AbiGitStashEntry,
    AbiGitStatusPrefix, AbiGitStatusSnapshot, AbiHoverProvider, AbiIconFontCategory,
    AbiIconFontSymbol, AbiKeymapConfig, AbiLanguageConfiguration, AbiLanguageServerRootStrategy,
    AbiLanguageServerSpec, AbiLigatureConfig, AbiLspDiagnosticsInfo, AbiMarkdownPrettyConfig,
    AbiOilDefaults, AbiOilFeatureSpec, AbiOilKeyAction, AbiOilKeybindings, AbiOilSortMode,
    AbiPaneConfig, AbiPdfOpenMode, AbiPickerLayout, AbiPickerTruncateStrategy, AbiSection,
    AbiSectionAction, AbiSectionItem, AbiSectionTree, AbiStatusEntry, AbiStatuslineContext,
    AbiStringPair, AbiTerminalConfig, AbiTerminalFeatureSpec, AbiTheme, AbiThemeOption,
    AbiThemeOptionEntry, AbiThemeToken, AbiWorkspaceDockSide, AbiWorkspaceRoot, UserLibraryModule,
    UserLibraryModuleRef,
};
pub use editor_icons::symbols;

// ─── Protocol hook name constants ───────────────────────────────────────────
//
// These string identifiers form the stable "protocol" between the host editor
// and the compiled user library.  Keeping them here means both sides always
// agree on the exact identifier without the host depending on the user crate.

/// Hook name constants for the autocomplete subsystem.
pub mod autocomplete_hooks {
    pub const TRIGGER: &str = "ui.autocomplete.trigger";
    pub const NEXT: &str = "ui.autocomplete.next";
    pub const PREVIOUS: &str = "ui.autocomplete.previous";
    pub const ACCEPT: &str = "ui.autocomplete.accept";
    pub const CANCEL: &str = "ui.autocomplete.cancel";
}

/// Hook name constants for the hover subsystem.
pub mod hover_hooks {
    pub const TOGGLE: &str = "ui.hover.toggle";
    pub const FOCUS: &str = "ui.hover.focus";
    pub const NEXT: &str = "ui.hover.next";
    pub const PREVIOUS: &str = "ui.hover.previous";
}

/// Hook name constants for picker UI.
pub mod picker_hooks {
    pub const OPEN: &str = "ui.picker.open";
    pub const NEXT: &str = "ui.picker.next";
    pub const PREVIOUS: &str = "ui.picker.previous";
    pub const SUBMIT: &str = "ui.picker.submit";
    pub const CANCEL: &str = "ui.picker.cancel";
}

/// Hook name constants for the LSP subsystem.
pub mod lsp_hooks {
    pub const START: &str = "lsp.server-start";
    pub const STOP: &str = "lsp.server-stop";
    pub const RESTART: &str = "lsp.server-restart";
    pub const LOG: &str = "lsp.open-log";
    pub const DEFINITION: &str = "lsp.goto-definition";
    pub const REFERENCES: &str = "lsp.goto-references";
    pub const IMPLEMENTATION: &str = "lsp.goto-implementation";
    pub const DIAGNOSTICS: &str = "lsp.diagnostics";
    pub const CODE_ACTIONS: &str = "lsp.code-actions";
    pub const COPILOT_SIGN_IN: &str = "lsp.copilot-sign-in";
    pub const COPILOT_SIGN_OUT: &str = "lsp.copilot-sign-out";
}

/// Hook name constants for the git subsystem.
pub mod git_hooks {
    pub const STATUS_OPEN_POPUP: &str = "ui.git.status-open-popup";
    pub const DIFF_OPEN: &str = "ui.git.diff-open";
    pub const LOG_OPEN: &str = "ui.git.log-open";
    pub const STASH_LIST_OPEN: &str = "ui.git.stash-list-open";
}

/// Hook name constants for the oil directory browser.
pub mod oil_hooks {
    pub const OPEN: &str = "ui.oil.open";
    pub const OPEN_PARENT: &str = "ui.oil.open-parent";
    pub const ACTION: &str = "ui.oil.action";
    pub const GIT_WORKTREE: &str = "ui.oil.git-worktree";
}

/// Hook name constants for the browser buffer.
pub mod browser_hooks {
    pub const OPEN: &str = "ui.browser.open";
    pub const OPEN_BUFFER: &str = "ui.browser.open-buffer";
    pub const OPEN_POPUP: &str = "ui.browser.open-popup";
    pub const URL: &str = "ui.browser.url";
    pub const FOCUS_INPUT: &str = "ui.browser.focus-input";
    pub const SUBMIT: &str = "ui.browser.submit";
}

/// Hook name constants for generic input surfaces.
pub mod input_hooks {
    pub const SUBMIT: &str = "ui.input.submit";
    pub const CLEAR: &str = "ui.input.clear";
}

/// Hook name constants for database explorer buffers.
pub mod db_hooks {
    pub const CONNECT: &str = "db.connect";
    pub const DISCONNECT: &str = "db.disconnect";
    pub const SHOW_TABLES: &str = "db.show-tables";
    pub const NEW_QUERY_BUFFER: &str = "db.new-query-buffer";
    pub const EXECUTE_SQL: &str = "db.execute-sql";
    pub const SHOW_CONNECTIONS: &str = "db.show-connections";
    pub const SHOW_HISTORY: &str = "db.show-history";
    pub const SHOW_SNIPPETS: &str = "db.show-snippets";
    pub const SAVE_SNIPPET: &str = "db.save-snippet";
    pub const REFRESH_SCHEMA: &str = "db.refresh-schema";
    pub const ACTIVATE_LINE: &str = "db.activate-line";
}

/// Hook name constants for terminal buffers.
pub mod terminal_hooks {
    pub const OPEN_POPUP: &str = "ui.terminal.open-popup";
}

/// Hook name constants for the native image viewer.
pub mod image_hooks {
    pub const ZOOM_IN: &str = "ui.image.zoom-in";
    pub const ZOOM_OUT: &str = "ui.image.zoom-out";
    pub const ZOOM_RESET: &str = "ui.image.zoom-reset";
    pub const TOGGLE_MODE: &str = "ui.image.toggle-mode";
}

/// Hook name constants for the workspace Issues plugin.
pub mod issues_hooks {
    pub const BOARD_OPEN: &str = "ui.issues.board-open";
    pub const CREATE: &str = "ui.issues.create";
    pub const SCAN: &str = "ui.issues.scan";
    pub const CAPTURE_FOCUSED: &str = "ui.issues.capture-focused";
    pub const ACTIVATE_LINE: &str = "ui.issues.activate-line";
    pub const SET_STATUS: &str = "ui.issues.set-status";
    pub const PLACE: &str = "ui.issues.place";
    pub const OPEN_FROM_REF: &str = "ui.issues.open-from-ref";
    pub const JUMP_REFS: &str = "ui.issues.jump-refs";
    pub const TOGGLE_CLOSED: &str = "ui.issues.board-toggle-closed";
}

/// Hook name constants for the native PDF buffer.
pub mod pdf_hooks {
    pub const NEXT_PAGE: &str = "ui.pdf.next-page";
    pub const PREVIOUS_PAGE: &str = "ui.pdf.previous-page";
    pub const ROTATE_CLOCKWISE: &str = "ui.pdf.rotate-clockwise";
    pub const DELETE_PAGE: &str = "ui.pdf.delete-page";
}

// ─── Buffer kind string constants ────────────────────────────────────────────

/// Buffer kind strings used when creating or matching plugin buffers.
pub mod buffer_kinds {
    pub const GIT_STATUS: &str = "git-status";
    pub const GIT_COMMIT: &str = "git-commit";
    pub const GIT_DIFF: &str = "git-diff";
    pub const GIT_LOG: &str = "git-log";
    pub const GIT_STASH: &str = "git-stash";
    pub const ACP: &str = "acp";
    pub const BROWSER: &str = "browser";
    pub const CALCULATOR: &str = "calculator";
    pub const DB_CONNECT: &str = "db-connect";
    pub const DB_QUERY: &str = "db-query";
    pub const DB_CONNECTIONS: &str = "db-connections";
    pub const DB_SCHEMA: &str = "db-schema";
    pub const DB_HISTORY: &str = "db-history";
    pub const DB_SNIPPETS: &str = "db-snippets";
    pub const DB_RESULTS: &str = "db-results";
    pub const PDF: &str = "pdf";
    pub const ISSUES_BOARD: &str = "issues-board";
}

/// Controls how a plugin section is updated when plugin evaluation writes to it.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PluginBufferSectionUpdate {
    /// Replace the existing section contents with the newly produced lines.
    Replace,
    /// Append the newly produced lines after the existing section contents.
    Append,
}

/// Metadata for one rendered section within a plugin-owned buffer.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginBufferSection {
    name: RString,
    writable: bool,
    min_lines: ROption<usize>,
    initial_lines: RVec<RString>,
    update: PluginBufferSectionUpdate,
}

impl PluginBufferSection {
    /// Creates a new section with the given display name.
    pub fn new(name: impl Into<RString>) -> Self {
        Self {
            name: name.into(),
            writable: false,
            min_lines: ROption::RNone,
            initial_lines: RVec::new(),
            update: PluginBufferSectionUpdate::Replace,
        }
    }

    /// Marks the section as writable or read-only.
    pub fn with_writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    /// Declares the minimum number of wrapped rows reserved for the section.
    /// Values below 1 are clamped to 1.
    pub fn with_min_lines(mut self, min_lines: usize) -> Self {
        self.min_lines = ROption::RSome(min_lines.max(1));
        self
    }

    /// Seeds the section with the provided initial lines.
    pub fn with_initial_lines(mut self, lines: Vec<impl Into<RString>>) -> Self {
        self.initial_lines = lines.into_iter().map(Into::into).collect::<Vec<_>>().into();
        self
    }

    /// Controls whether evaluation replaces or appends to this section.
    pub fn with_update(mut self, update: PluginBufferSectionUpdate) -> Self {
        self.update = update;
        self
    }

    /// Returns the display name rendered for the section.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns whether the section is writable.
    pub const fn writable(&self) -> bool {
        self.writable
    }

    /// Returns the minimum number of wrapped rows reserved for the section.
    pub const fn min_lines(&self) -> Option<usize> {
        match self.min_lines {
            ROption::RSome(value) => Some(value),
            ROption::RNone => None,
        }
    }

    /// Returns the initial lines shown in the section.
    pub fn initial_lines(&self) -> &[RString] {
        self.initial_lines.as_slice()
    }

    /// Returns how evaluation updates this section.
    pub const fn update(&self) -> PluginBufferSectionUpdate {
        self.update
    }
}

/// Generic section metadata for plugin buffers that want host-rendered panes.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginBufferSections {
    sections: RVec<PluginBufferSection>,
}

impl PluginBufferSections {
    /// Creates a sectioned plugin buffer configuration.
    pub fn new(sections: Vec<PluginBufferSection>) -> Self {
        Self {
            sections: sections.into(),
        }
    }

    /// Returns the configured sections in display order.
    pub fn items(&self) -> &[PluginBufferSection] {
        self.sections.as_slice()
    }

    /// Returns the configured sections in display order.
    pub fn sections(&self) -> &[PluginBufferSection] {
        self.items()
    }
}

/// Declares a plugin-owned buffer kind and the host behavior it needs.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginBuffer {
    kind: RString,
    initial_lines: RVec<RString>,
    sections: ROption<PluginBufferSections>,
    evaluate_handler: ROption<RString>,
    evaluate_target_section: ROption<RString>,
    line_wrap: bool,
    key_bindings: RVec<PluginKeyBinding>,
}

/// Context passed to the user library when rendering the statusline.
#[derive(Debug, Clone, Copy)]
pub struct StatuslineContext<'a> {
    pub vim_mode: &'a str,
    pub recording_macro: Option<char>,
    pub workspace_name: &'a str,
    pub buffer_name: &'a str,
    pub buffer_modified: bool,
    pub language_id: Option<&'a str>,
    pub line: usize,
    pub column: usize,
    pub lsp_server: Option<&'a str>,
    pub lsp_diagnostics: Option<LspDiagnosticsInfo>,
    pub acp_connected: bool,
    pub git_branch: Option<&'a str>,
    pub git_added: usize,
    pub git_removed: usize,
}

/// One painted run of statusline text, optionally themed by token name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatuslineSpan {
    pub text: String,
    pub token: Option<String>,
}

impl StatuslineSpan {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            token: None,
        }
    }

    pub fn themed(text: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            token: Some(token.into()),
        }
    }
}

/// Left or right placement for a modeline segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelineAlignment {
    Left,
    Right,
}

/// One painted subsection inside a modeline segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelinePart {
    pub text: String,
    pub foreground: String,
    pub background: Option<String>,
}

impl ModelinePart {
    pub fn new(
        text: impl Into<String>,
        foreground: impl Into<String>,
        background: Option<String>,
    ) -> Self {
        Self {
            text: text.into(),
            foreground: foreground.into(),
            background,
        }
    }

    pub fn fg(text: impl Into<String>, foreground: impl Into<String>) -> Self {
        Self::new(text, foreground, None)
    }
}

/// Ordered group of modeline parts with left/right alignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelineSegment {
    pub alignment: ModelineAlignment,
    pub parts: Vec<ModelinePart>,
}

impl ModelineSegment {
    pub fn left(parts: Vec<ModelinePart>) -> Self {
        Self {
            alignment: ModelineAlignment::Left,
            parts,
        }
    }

    pub fn right(parts: Vec<ModelinePart>) -> Self {
        Self {
            alignment: ModelineAlignment::Right,
            parts,
        }
    }
}

const STATUSLINE_SPAN_MARK: &str = "\u{0002}SL1\u{0003}";
const MODELINE_MARK: &str = "\u{0002}SL2\u{0003}";
const STATUSLINE_SPAN_SEP: char = '\u{001E}';
const STATUSLINE_TOKEN_SEP: char = '\u{001F}';
const MODELINE_SEG_SEP: char = '\u{001D}';
const MODELINE_PART_SEP: char = '\u{001E}';
const MODELINE_FIELD_SEP: char = '\u{001F}';

/// Packs themed statusline spans into the existing `statusline_render` ABI string.
pub fn encode_statusline_spans(spans: &[StatuslineSpan]) -> String {
    let mut encoded = String::from(STATUSLINE_SPAN_MARK);
    for (index, span) in spans.iter().enumerate() {
        if index > 0 {
            encoded.push(STATUSLINE_SPAN_SEP);
        }
        if let Some(token) = &span.token {
            encoded.push_str(&sanitize_modeline_field(token));
        }
        encoded.push(STATUSLINE_TOKEN_SEP);
        encoded.push_str(&sanitize_modeline_field(&span.text));
    }
    encoded
}

/// Unpacks a `statusline_render` ABI string into themed spans.
pub fn decode_statusline_spans(raw: &str) -> Vec<StatuslineSpan> {
    if raw.starts_with(MODELINE_MARK) {
        return flatten_modeline_to_spans(&decode_modeline(raw));
    }
    let Some(payload) = raw.strip_prefix(STATUSLINE_SPAN_MARK) else {
        return vec![StatuslineSpan::plain(raw)];
    };
    if payload.is_empty() {
        return Vec::new();
    }
    payload
        .split(STATUSLINE_SPAN_SEP)
        .map(|part| match part.split_once(STATUSLINE_TOKEN_SEP) {
            Some(("", text)) => StatuslineSpan::plain(text),
            Some((token, text)) => StatuslineSpan::themed(text, token),
            None => StatuslineSpan::plain(part),
        })
        .collect()
}

/// Packs modeline segments into the existing `statusline_render` ABI string.
pub fn encode_modeline(segments: &[ModelineSegment]) -> String {
    let mut encoded = String::from(MODELINE_MARK);
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            encoded.push(MODELINE_SEG_SEP);
        }
        encoded.push(match segment.alignment {
            ModelineAlignment::Left => 'L',
            ModelineAlignment::Right => 'R',
        });
        for (part_index, part) in segment.parts.iter().enumerate() {
            if part_index > 0 {
                encoded.push(MODELINE_PART_SEP);
            }
            encoded.push_str(&sanitize_modeline_field(&part.foreground));
            encoded.push(MODELINE_FIELD_SEP);
            if let Some(background) = &part.background {
                encoded.push_str(&sanitize_modeline_field(background));
            }
            encoded.push(MODELINE_FIELD_SEP);
            encoded.push_str(&sanitize_modeline_field(&part.text));
        }
    }
    encoded
}

/// Unpacks a `statusline_render` ABI string into modeline segments.
pub fn decode_modeline(raw: &str) -> Vec<ModelineSegment> {
    if let Some(payload) = raw.strip_prefix(MODELINE_MARK) {
        if payload.is_empty() {
            return Vec::new();
        }
        return payload
            .split(MODELINE_SEG_SEP)
            .filter_map(decode_modeline_segment)
            .collect();
    }
    if raw.starts_with(STATUSLINE_SPAN_MARK) {
        let spans = decode_statusline_spans_sl1(raw);
        if spans.is_empty() {
            return Vec::new();
        }
        return vec![ModelineSegment::left(
            spans
                .into_iter()
                .map(|span| ModelinePart {
                    text: span.text,
                    foreground: span.token.unwrap_or_default(),
                    background: None,
                })
                .collect(),
        )];
    }
    if raw.is_empty() {
        return Vec::new();
    }
    vec![ModelineSegment::left(vec![ModelinePart::fg(
        raw,
        String::new(),
    )])]
}

fn decode_statusline_spans_sl1(raw: &str) -> Vec<StatuslineSpan> {
    let Some(payload) = raw.strip_prefix(STATUSLINE_SPAN_MARK) else {
        return vec![StatuslineSpan::plain(raw)];
    };
    if payload.is_empty() {
        return Vec::new();
    }
    payload
        .split(STATUSLINE_SPAN_SEP)
        .map(|part| match part.split_once(STATUSLINE_TOKEN_SEP) {
            Some(("", text)) => StatuslineSpan::plain(text),
            Some((token, text)) => StatuslineSpan::themed(text, token),
            None => StatuslineSpan::plain(part),
        })
        .collect()
}

fn decode_modeline_segment(raw: &str) -> Option<ModelineSegment> {
    let (alignment_char, parts_raw) = raw.split_at(raw.len().min(1));
    let alignment = match alignment_char {
        "L" => ModelineAlignment::Left,
        "R" => ModelineAlignment::Right,
        _ => return None,
    };
    if parts_raw.is_empty() {
        return Some(ModelineSegment {
            alignment,
            parts: Vec::new(),
        });
    }
    let parts = parts_raw
        .split(MODELINE_PART_SEP)
        .filter_map(|part| {
            let mut fields = part.splitn(3, MODELINE_FIELD_SEP);
            let foreground = fields.next()?.to_string();
            let background = fields.next()?;
            let text = fields.next()?.to_string();
            Some(ModelinePart {
                text,
                foreground,
                background: (!background.is_empty()).then(|| background.to_string()),
            })
        })
        .collect();
    Some(ModelineSegment { alignment, parts })
}

/// Flattens modeline segments into legacy statusline spans (foreground token only).
pub fn flatten_modeline_to_spans(segments: &[ModelineSegment]) -> Vec<StatuslineSpan> {
    let mut spans = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            spans.push(StatuslineSpan::plain(" "));
        }
        for (part_index, part) in segment.parts.iter().enumerate() {
            if part_index > 0 {
                spans.push(StatuslineSpan::plain(" "));
            }
            if part.foreground.is_empty() {
                spans.push(StatuslineSpan::plain(&part.text));
            } else {
                spans.push(StatuslineSpan::themed(&part.text, &part.foreground));
            }
        }
    }
    spans
}

/// Joins modeline part texts into a single display string.
pub fn flatten_modeline_text(segments: &[ModelineSegment]) -> String {
    let mut text = String::new();
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 && !text.is_empty() {
            text.push(' ');
        }
        for (part_index, part) in segment.parts.iter().enumerate() {
            if part_index > 0 {
                text.push(' ');
            }
            text.push_str(&part.text);
        }
    }
    text
}

fn sanitize_modeline_field(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            *character != MODELINE_SEG_SEP
                && *character != MODELINE_PART_SEP
                && *character != MODELINE_FIELD_SEP
        })
        .collect()
}

/// Context passed to the user library when producing inline ghost-text annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostTextContext<'a> {
    /// Stable numeric identifier for the active buffer.
    pub buffer_id: u64,
    /// Monotonic buffer revision for the current text snapshot.
    pub buffer_revision: u64,
    /// Active buffer display name.
    pub buffer_name: &'a str,
    /// Active buffer language identifier, if any.
    pub language_id: Option<&'a str>,
    /// Complete buffer text.
    pub buffer_text: &'a str,
    /// Zero-based topmost visible logical line in the current viewport.
    pub viewport_top_line: usize,
    /// Zero-based cursor line.
    pub cursor_line: usize,
    /// Zero-based cursor column.
    pub cursor_column: usize,
}

/// One ghost-text annotation rendered on a specific buffer line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhostTextLine {
    /// Zero-based buffer line index that should receive the annotation.
    pub line: usize,
    /// Rendered ghost-text content, including any icon prefix.
    pub text: String,
}

/// Built-in or user-defined source used to populate a picker.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PickerSource {
    /// User library supplies entries without any shell-side provider code.
    User,
    Static,
    Commands,
    Buffers,
    BufferClose,
    Keybindings,
    Themes,
    IconFonts,
    AcpClients,
    TreesitterLanguages,
    WorkspaceProjects,
    WorkspaceDashboard,
    WorkspaceSwitch,
    WorkspaceDelete,
    WorkspaceFiles,
    WorkspaceSearch,
    UndoTree,
}

/// Action executed when a user-defined static picker entry is submitted.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub enum PickerActionSpec {
    NoOp,
    ExecuteCommand {
        command: RString,
    },
    ExecuteCommands {
        commands: RVec<RString>,
    },
    EmitHook {
        hook: RString,
        detail: ROption<RString>,
    },
    FocusBuffer {
        buffer_id: u64,
    },
    CloseBuffer {
        buffer_id: u64,
    },
    OpenAcpClient {
        client_id: RString,
    },
    OpenFile {
        path: RString,
    },
    CreateWorkspaceFile {
        root: RString,
    },
    InstallTreeSitterLanguage {
        language_id: RString,
    },
    CreateWorkspace {
        name: RString,
        root: RString,
    },
    SwitchWorkspace {
        workspace_id: u64,
    },
    DeleteWorkspace {
        workspace_id: u64,
    },
    UndoTreeNode {
        buffer_id: u64,
        node_id: usize,
    },
    CopyToClipboard {
        text: RString,
    },
    ActivateTheme {
        theme_id: RString,
    },
}

impl PickerActionSpec {
    pub fn no_op() -> Self {
        Self::NoOp
    }

    pub fn execute_command(command: impl Into<RString>) -> Self {
        Self::ExecuteCommand {
            command: command.into(),
        }
    }

    pub fn execute_commands<I, S>(commands: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<RString>,
    {
        Self::ExecuteCommands {
            commands: commands.into_iter().map(Into::into).collect(),
        }
    }

    pub fn emit_hook(hook: impl Into<RString>, detail: Option<impl Into<RString>>) -> Self {
        Self::EmitHook {
            hook: hook.into(),
            detail: detail.map(Into::into).into(),
        }
    }

    pub fn open_file(path: impl Into<RString>) -> Self {
        Self::OpenFile { path: path.into() }
    }

    pub fn focus_buffer(buffer_id: u64) -> Self {
        Self::FocusBuffer { buffer_id }
    }

    pub fn close_buffer(buffer_id: u64) -> Self {
        Self::CloseBuffer { buffer_id }
    }

    pub fn open_acp_client(client_id: impl Into<RString>) -> Self {
        Self::OpenAcpClient {
            client_id: client_id.into(),
        }
    }

    pub fn create_workspace_file(root: impl Into<RString>) -> Self {
        Self::CreateWorkspaceFile { root: root.into() }
    }

    pub fn install_tree_sitter_language(language_id: impl Into<RString>) -> Self {
        Self::InstallTreeSitterLanguage {
            language_id: language_id.into(),
        }
    }

    pub fn create_workspace(name: impl Into<RString>, root: impl Into<RString>) -> Self {
        Self::CreateWorkspace {
            name: name.into(),
            root: root.into(),
        }
    }

    pub fn switch_workspace(workspace_id: u64) -> Self {
        Self::SwitchWorkspace { workspace_id }
    }

    pub fn delete_workspace(workspace_id: u64) -> Self {
        Self::DeleteWorkspace { workspace_id }
    }

    pub fn undo_tree_node(buffer_id: u64, node_id: usize) -> Self {
        Self::UndoTreeNode { buffer_id, node_id }
    }

    pub fn copy_to_clipboard(text: impl Into<RString>) -> Self {
        Self::CopyToClipboard { text: text.into() }
    }

    pub fn activate_theme(theme_id: impl Into<RString>) -> Self {
        Self::ActivateTheme {
            theme_id: theme_id.into(),
        }
    }
}

/// One entry in a user-defined static picker.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PickerItemSpec {
    id: RString,
    label: RString,
    detail: RString,
    preview: ROption<RString>,
    search_text: ROption<RString>,
    fringe: ROption<RString>,
    action: PickerActionSpec,
    divider: bool,
}

impl PickerItemSpec {
    pub fn new(
        id: impl Into<RString>,
        label: impl Into<RString>,
        detail: impl Into<RString>,
        action: PickerActionSpec,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: detail.into(),
            preview: ROption::RNone,
            search_text: ROption::RNone,
            fringe: ROption::RNone,
            action,
            divider: false,
        }
    }

    /// Creates a non-selectable horizontal divider row.
    pub fn divider() -> Self {
        Self {
            id: "__picker-divider__".into(),
            label: RString::from(""),
            detail: RString::from(""),
            preview: ROption::RNone,
            search_text: ROption::RNone,
            fringe: ROption::RNone,
            action: PickerActionSpec::NoOp,
            divider: true,
        }
    }

    pub const fn is_divider(&self) -> bool {
        self.divider
    }

    pub fn with_preview(mut self, preview: impl Into<RString>) -> Self {
        self.preview = ROption::RSome(preview.into());
        self
    }

    pub fn with_search_text(mut self, search_text: impl Into<RString>) -> Self {
        self.search_text = ROption::RSome(search_text.into());
        self
    }

    pub fn with_fringe(mut self, fringe: impl Into<RString>) -> Self {
        self.fringe = ROption::RSome(fringe.into());
        self
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }

    pub fn preview(&self) -> Option<&str> {
        self.preview.as_ref().map(RString::as_str).into()
    }

    pub fn search_text(&self) -> Option<&str> {
        self.search_text.as_ref().map(RString::as_str).into()
    }

    pub fn fringe(&self) -> Option<&str> {
        self.fringe.as_ref().map(RString::as_str).into()
    }

    pub fn action(&self) -> &PickerActionSpec {
        &self.action
    }
}
/// Picker label truncation when a row is narrower than the label.
///
/// Examples use `src/dir1/dir2/test.rs` unless noted. Ellipsis variants also
/// clip to the viewport width after any path transform.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PickerTruncateStrategy {
    /// Tail clip: `src/dir1/dir2/te...`
    EndEllipsis = 0,
    /// Head clip: `...ir2/test.rs`
    StartEllipsis,
    /// Both ends: `src/.../test.rs`
    MiddleEllipsis,
    /// First char per parent dir: `s/d/d/test.rs` (doom `truncate-with-project` relative)
    ShrinkDirectories,
    /// Shrink parent dirs and filename stem: `s/d/d/t.rs` (doom `truncate-all`)
    ShrinkAll,
    /// Filename only: `test.rs` (doom `file-name`)
    FileName,
    /// Parent + file: `dir2/test.rs` (doom `relative-to-project`)
    FileNameWithParent,
    /// Parent initial + file: `d/test.rs`
    ParentInitialFileName,
    /// Shrink leading dirs, keep last three segments full on long paths
    ShrinkLeadingKeepTail,
    /// Full path when it fits; otherwise head clip (doom `truncate-nil` display)
    Full,
    /// Full when it fits, else `ShrinkDirectories`, else head clip (doom `auto`)
    Auto,
}

/// Picker card size as fractions of the editor window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickerLayout {
    /// Horizontal fraction of the window (clamped by the shell to `0.15..=1.0`).
    pub width_fraction: f32,
    /// Vertical fraction of the window (clamped by the shell to `0.15..=1.0`).
    pub height_fraction: f32,
}

impl Default for PickerLayout {
    fn default() -> Self {
        Self {
            width_fraction: 2.0 / 3.0,
            height_fraction: 3.0 / 5.0,
        }
    }
}

/// One picker-instance extra keybind (chord → command) declared on a provider.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PickerExtraKeybindSpec {
    chord: RString,
    command_name: RString,
}

impl PickerExtraKeybindSpec {
    /// Creates a provider extra keybind.
    pub fn new(chord: impl Into<RString>, command_name: impl Into<RString>) -> Self {
        Self {
            chord: chord.into(),
            command_name: command_name.into(),
        }
    }

    /// Returns the chord string (for example `Ctrl+d`).
    pub fn chord(&self) -> &str {
        self.chord.as_str()
    }

    /// Returns the bound command name.
    pub fn command_name(&self) -> &str {
        self.command_name.as_str()
    }
}

/// Declares a picker provider that can be opened with `ui.picker.open`.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PickerProviderSpec {
    id: RString,
    title: RString,
    source: PickerSource,
    items: RVec<PickerItemSpec>,
    extra_keybinds: RVec<PickerExtraKeybindSpec>,
}

impl PickerProviderSpec {
    pub fn new(id: impl Into<RString>, title: impl Into<RString>, source: PickerSource) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            source,
            items: RVec::new(),
            extra_keybinds: RVec::new(),
        }
    }

    pub fn static_items(
        id: impl Into<RString>,
        title: impl Into<RString>,
        items: Vec<PickerItemSpec>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            source: PickerSource::Static,
            items: items.into(),
            extra_keybinds: RVec::new(),
        }
    }

    pub fn user(id: impl Into<RString>, title: impl Into<RString>) -> Self {
        Self::new(id, title, PickerSource::User)
    }

    /// Attaches extra chord→command bindings copied onto the open picker instance.
    pub fn with_extra_keybinds(mut self, keybinds: Vec<PickerExtraKeybindSpec>) -> Self {
        self.extra_keybinds = keybinds.into();
        self
    }

    /// Appends one extra chord→command binding.
    pub fn with_extra_keybind(
        mut self,
        chord: impl Into<RString>,
        command_name: impl Into<RString>,
    ) -> Self {
        self.extra_keybinds
            .push(PickerExtraKeybindSpec::new(chord, command_name));
        self
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub const fn source(&self) -> PickerSource {
        self.source
    }

    pub fn items(&self) -> &[PickerItemSpec] {
        self.items.as_slice()
    }

    /// Returns provider-declared extras copied onto each open instance.
    pub fn extra_keybinds(&self) -> &[PickerExtraKeybindSpec] {
        self.extra_keybinds.as_slice()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerCommandContext {
    pub name: RString,
    pub description: RString,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerBufferContext {
    pub id: u64,
    pub display_name: RString,
    pub path: ROption<RString>,
    pub kind_label: RString,
    pub preview: ROption<RString>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub dirty: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerKeybindingContext {
    pub id: RString,
    pub chord: RString,
    pub scope: RString,
    pub vim_mode: RString,
    pub command_names: RVec<RString>,
    pub description: RString,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerSyntaxLanguageContext {
    pub id: RString,
    pub detail: RString,
    pub preview: ROption<RString>,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerWorkspaceContext {
    pub id: u64,
    pub name: RString,
    pub root: ROption<RString>,
    pub is_default: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerWorkspaceFileContext {
    pub path: RString,
    pub label: RString,
    pub detail: RString,
    pub search_text: RString,
    pub fringe: RString,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerThemeContext {
    pub id: RString,
    pub name: RString,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerIconContext {
    pub id: RString,
    pub label: RString,
    pub detail: RString,
    pub glyph: RString,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerAcpClientContext {
    pub id: RString,
    pub label: RString,
    pub detail: RString,
}

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct PickerUndoTreeContext {
    pub buffer_id: u64,
    pub node_id: usize,
    pub fringe: RString,
    pub label: RString,
    pub detail: RString,
    pub preview: ROption<RString>,
    pub selected: bool,
}

/// Host-collected facts used by user modules to build picker entries.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PickerProviderContext {
    pub provider_id: RString,
    pub title: RString,
    pub source: PickerSource,
    pub commands: RVec<PickerCommandContext>,
    pub buffers: RVec<PickerBufferContext>,
    pub keybindings: RVec<PickerKeybindingContext>,
    pub syntax_languages: RVec<PickerSyntaxLanguageContext>,
    pub workspaces: RVec<PickerWorkspaceContext>,
    pub workspace_files: RVec<PickerWorkspaceFileContext>,
    pub workspace_root: ROption<RString>,
    pub themes: RVec<PickerThemeContext>,
    pub icons: RVec<PickerIconContext>,
    pub acp_clients: RVec<PickerAcpClientContext>,
    pub undo_tree: RVec<PickerUndoTreeContext>,
}

impl PickerProviderContext {
    pub fn new(
        provider_id: impl Into<RString>,
        title: impl Into<RString>,
        source: PickerSource,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            title: title.into(),
            source,
            commands: RVec::new(),
            buffers: RVec::new(),
            keybindings: RVec::new(),
            syntax_languages: RVec::new(),
            workspaces: RVec::new(),
            workspace_files: RVec::new(),
            workspace_root: ROption::RNone,
            themes: RVec::new(),
            icons: RVec::new(),
            acp_clients: RVec::new(),
            undo_tree: RVec::new(),
        }
    }
}

/// ACP picker family whose rows can be shaped by user configuration.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum AcpPickerKind {
    Modes,
    Models,
    Sessions,
    SlashCommands,
    FileMentions,
}

/// Host-provided ACP picker option data.
#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, StableAbi)]
pub struct AcpPickerOption {
    pub id: RString,
    pub label: RString,
    pub detail: RString,
    pub current: bool,
}

impl AcpPickerOption {
    pub fn new(id: impl Into<RString>, label: impl Into<RString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: RString::new(),
            current: false,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<RString>) -> Self {
        self.detail = detail.into();
        self
    }

    pub fn with_current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }
}

/// Host-collected facts used by user modules to shape ACP picker entries.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AcpPickerContext {
    pub kind: AcpPickerKind,
    pub title: RString,
    pub options: RVec<AcpPickerOption>,
}

impl AcpPickerContext {
    pub fn new(kind: AcpPickerKind, title: impl Into<RString>) -> Self {
        Self {
            kind,
            title: title.into(),
            options: RVec::new(),
        }
    }

    pub fn with_options(mut self, options: Vec<AcpPickerOption>) -> Self {
        self.options = options.into();
        self
    }
}

/// Action executed when an ACP picker entry is submitted.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub enum AcpActionSpec {
    SetMode { mode_id: RString },
    SetModel { model_id: RString },
    LoadSession { session_id: RString },
    InsertSlashCommand { command: RString },
    InsertFileMention { path: RString },
}

impl AcpActionSpec {
    pub fn set_mode(mode_id: impl Into<RString>) -> Self {
        Self::SetMode {
            mode_id: mode_id.into(),
        }
    }

    pub fn set_model(model_id: impl Into<RString>) -> Self {
        Self::SetModel {
            model_id: model_id.into(),
        }
    }

    pub fn load_session(session_id: impl Into<RString>) -> Self {
        Self::LoadSession {
            session_id: session_id.into(),
        }
    }

    pub fn insert_slash_command(command: impl Into<RString>) -> Self {
        Self::InsertSlashCommand {
            command: command.into(),
        }
    }

    pub fn insert_file_mention(path: impl Into<RString>) -> Self {
        Self::InsertFileMention {
            path: path.into(),
        }
    }
}

/// One user-shaped ACP picker row.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AcpPickerItemSpec {
    id: RString,
    label: RString,
    detail: RString,
    action: AcpActionSpec,
}

impl AcpPickerItemSpec {
    pub fn new(
        id: impl Into<RString>,
        label: impl Into<RString>,
        detail: impl Into<RString>,
        action: AcpActionSpec,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: detail.into(),
            action,
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }

    pub fn action(&self) -> &AcpActionSpec {
        &self.action
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum DbBrowserKind {
    Connections,
    Schema,
    History,
    Snippets,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum DbBrowserItemKind {
    Header,
    Empty,
    ActiveConnection,
    RememberedConnection,
    Table,
    View,
    Index,
    HistoryEntry,
    Snippet,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub enum DbActionSpec {
    ActivateRemembered {
        alias: RString,
    },
    DisconnectSession {
        session_id: u64,
    },
    OpenTablePreview {
        session_id: u64,
        schema: ROption<RString>,
        table: RString,
    },
    ExploreRows {
        session_id: u64,
        schema: ROption<RString>,
        table: RString,
    },
    RefreshSchema {
        session_id: u64,
    },
    OpenHistoryEntry {
        session_id: u64,
        sql: RString,
    },
    OpenSnippet {
        id: RString,
    },
    DeleteSnippet {
        id: RString,
    },
    DeleteRemembered {
        alias: RString,
    },
}

impl DbActionSpec {
    pub fn activate_remembered(alias: impl Into<RString>) -> Self {
        Self::ActivateRemembered {
            alias: alias.into(),
        }
    }

    pub fn disconnect_session(session_id: u64) -> Self {
        Self::DisconnectSession { session_id }
    }

    pub fn open_table_preview(
        session_id: u64,
        schema: Option<impl Into<RString>>,
        table: impl Into<RString>,
    ) -> Self {
        Self::OpenTablePreview {
            session_id,
            schema: schema.map(Into::into).into(),
            table: table.into(),
        }
    }

    pub fn explore_rows(
        session_id: u64,
        schema: Option<impl Into<RString>>,
        table: impl Into<RString>,
    ) -> Self {
        Self::ExploreRows {
            session_id,
            schema: schema.map(Into::into).into(),
            table: table.into(),
        }
    }

    pub fn refresh_schema(session_id: u64) -> Self {
        Self::RefreshSchema { session_id }
    }

    pub fn open_history_entry(session_id: u64, sql: impl Into<RString>) -> Self {
        Self::OpenHistoryEntry {
            session_id,
            sql: sql.into(),
        }
    }

    pub fn open_snippet(id: impl Into<RString>) -> Self {
        Self::OpenSnippet { id: id.into() }
    }

    pub fn delete_snippet(id: impl Into<RString>) -> Self {
        Self::DeleteSnippet { id: id.into() }
    }

    pub fn delete_remembered(alias: impl Into<RString>) -> Self {
        Self::DeleteRemembered {
            alias: alias.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct DbBrowserItemContext {
    pub kind: DbBrowserItemKind,
    pub label: RString,
    pub detail: RString,
    pub engine: RString,
    pub session_id: ROption<u64>,
    pub active: bool,
    pub remembered: bool,
    pub schema: ROption<RString>,
    pub table: ROption<RString>,
    pub sql: ROption<RString>,
    pub id: ROption<RString>,
    pub default_action: ROption<DbActionSpec>,
}

impl DbBrowserItemContext {
    pub fn new(kind: DbBrowserItemKind, label: impl Into<RString>) -> Self {
        Self {
            kind,
            label: label.into(),
            detail: RString::new(),
            engine: RString::new(),
            session_id: ROption::RNone,
            active: false,
            remembered: false,
            schema: ROption::RNone,
            table: ROption::RNone,
            sql: ROption::RNone,
            id: ROption::RNone,
            default_action: ROption::RNone,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<RString>) -> Self {
        self.detail = detail.into();
        self
    }

    pub fn with_engine(mut self, engine: impl Into<RString>) -> Self {
        self.engine = engine.into();
        self
    }

    pub fn with_session_id(mut self, session_id: u64) -> Self {
        self.session_id = ROption::RSome(session_id);
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn with_remembered(mut self, remembered: bool) -> Self {
        self.remembered = remembered;
        self
    }

    pub fn with_table(
        mut self,
        schema: Option<impl Into<RString>>,
        table: impl Into<RString>,
    ) -> Self {
        self.schema = schema.map(Into::into).into();
        self.table = ROption::RSome(table.into());
        self
    }

    pub fn with_sql(mut self, sql: impl Into<RString>) -> Self {
        self.sql = ROption::RSome(sql.into());
        self
    }

    pub fn with_id(mut self, id: impl Into<RString>) -> Self {
        self.id = ROption::RSome(id.into());
        self
    }

    pub fn with_default_action(mut self, action: DbActionSpec) -> Self {
        self.default_action = ROption::RSome(action);
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct DbBrowserContext {
    pub kind: DbBrowserKind,
    pub title: RString,
    pub items: RVec<DbBrowserItemContext>,
}

impl DbBrowserContext {
    pub fn new(kind: DbBrowserKind, title: impl Into<RString>) -> Self {
        Self {
            kind,
            title: title.into(),
            items: RVec::new(),
        }
    }

    pub fn with_items(mut self, items: impl Into<RVec<DbBrowserItemContext>>) -> Self {
        self.items = items.into();
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct DbBrowserItemSpec {
    line: RString,
    action: ROption<DbActionSpec>,
}

impl DbBrowserItemSpec {
    pub fn new(line: impl Into<RString>, action: Option<DbActionSpec>) -> Self {
        Self {
            line: line.into(),
            action: action.into(),
        }
    }

    pub fn line(&self) -> &str {
        self.line.as_str()
    }

    pub fn action(&self) -> Option<&DbActionSpec> {
        self.action.as_ref().into()
    }
}

fn default_db_browser_line(item: &DbBrowserItemContext) -> String {
    match item.kind {
        DbBrowserItemKind::Header | DbBrowserItemKind::Empty => item.label.to_string(),
        DbBrowserItemKind::ActiveConnection | DbBrowserItemKind::RememberedConnection => {
            format!(
                "{} {}{}",
                item.engine,
                item.label,
                if item.active { " [active]" } else { "" }
            )
        }
        DbBrowserItemKind::Table => format!("▦ {}", item.label),
        DbBrowserItemKind::View => format!("◫ {}", item.label),
        DbBrowserItemKind::Index => format!("◎ {}", item.label),
        DbBrowserItemKind::HistoryEntry | DbBrowserItemKind::Snippet => {
            format!("{} {} :: {}", item.engine, item.label, item.detail)
        }
    }
}

/// Stable contract implemented by the compiled user extension library.
pub trait UserLibrary: Send + Sync {
    fn packages(&self) -> Vec<PluginPackage> {
        Vec::new()
    }
    fn themes(&self) -> Vec<Theme> {
        Vec::new()
    }
    fn syntax_languages(&self) -> Vec<LanguageConfiguration> {
        Vec::new()
    }
    fn language_servers(&self) -> Vec<LanguageServerSpec> {
        Vec::new()
    }
    fn debug_adapters(&self) -> Vec<DebugAdapterSpec> {
        Vec::new()
    }
    fn autocomplete_providers(&self) -> Vec<AutocompleteProvider> {
        Vec::new()
    }
    fn autocomplete_result_limit(&self) -> usize {
        8
    }
    fn autocomplete_token_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_FORM_TEXTBOX
    }
    fn hover_providers(&self) -> Vec<HoverProvider> {
        Vec::new()
    }
    fn hover_line_limit(&self) -> usize {
        10
    }
    fn hover_token_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_HELP_CIRCLE_OUTLINE
    }
    fn hover_signature_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_SIGNATURE
    }
    fn picker_providers(&self) -> Vec<PickerProviderSpec> {
        Vec::new()
    }
    fn picker_provider_items(
        &self,
        context: &PickerProviderContext,
    ) -> Option<Vec<PickerItemSpec>> {
        self.picker_providers()
            .into_iter()
            .find(|provider| {
                provider.id() == context.provider_id.as_str()
                    && provider.source() == PickerSource::Static
            })
            .map(|provider| provider.items().to_vec())
    }
    fn picker_truncate_strategy(&self) -> PickerTruncateStrategy {
        PickerTruncateStrategy::Auto
    }
    fn picker_layout(&self) -> PickerLayout {
        PickerLayout::default()
    }
    fn acp_clients(&self) -> Vec<AcpClient> {
        Vec::new()
    }
    fn acp_client_by_id(&self, _id: &str) -> Option<AcpClient> {
        None
    }
    fn acp_picker_items(&self, context: &AcpPickerContext) -> Vec<AcpPickerItemSpec> {
        context
            .options
            .iter()
            .map(|option| {
                let action = match context.kind {
                    AcpPickerKind::Modes => AcpActionSpec::set_mode(option.id.clone()),
                    AcpPickerKind::Models => AcpActionSpec::set_model(option.id.clone()),
                    AcpPickerKind::Sessions => AcpActionSpec::load_session(option.id.clone()),
                    AcpPickerKind::SlashCommands => {
                        AcpActionSpec::insert_slash_command(option.id.clone())
                    }
                    AcpPickerKind::FileMentions => {
                        AcpActionSpec::insert_file_mention(option.id.clone())
                    }
                };
                AcpPickerItemSpec::new(
                    option.id.clone(),
                    option.label.clone(),
                    option.detail.clone(),
                    action,
                )
            })
            .collect()
    }
    fn db_browser_items(&self, context: &DbBrowserContext) -> Vec<DbBrowserItemSpec> {
        context
            .items
            .iter()
            .map(|item| {
                DbBrowserItemSpec::new(
                    default_db_browser_line(item),
                    item.default_action.clone().into(),
                )
            })
            .collect()
    }
    fn workspace_roots(&self) -> Vec<WorkspaceRoot> {
        Vec::new()
    }
    fn terminal_config(&self) -> TerminalConfig {
        #[cfg(target_os = "windows")]
        return TerminalConfig {
            program: "powershell.exe".to_owned(),
            args: vec!["-NoLogo".to_owned()],
        };
        #[cfg(not(target_os = "windows"))]
        return TerminalConfig {
            program: "bash".to_owned(),
            args: Vec::new(),
        };
    }
    fn commandline_enabled(&self) -> bool {
        true
    }
    fn pane_config(&self) -> PaneConfig {
        PaneConfig {
            golden_ratio: false,
        }
    }
    fn workspace_dock_config(&self) -> WorkspaceDockConfig {
        WorkspaceDockConfig::default()
    }
    fn markdown_pretty_config(&self) -> MarkdownPrettyConfig {
        MarkdownPrettyConfig::default()
    }
    fn keymap_config(&self) -> KeymapConfig {
        KeymapConfig::default()
    }
    fn ligature_config(&self) -> LigatureConfig {
        LigatureConfig { enabled: false }
    }
    fn rainbow_parens_config(&self) -> RainbowParensConfig {
        RainbowParensConfig { enabled: true }
    }
    fn oil_defaults(&self) -> OilDefaults {
        OilDefaults::default()
    }
    fn oil_keybindings(&self) -> OilKeybindings {
        OilKeybindings::default()
    }
    fn oil_keydown_action(&self, _chord: &str) -> Option<OilKeyAction> {
        None
    }
    fn oil_chord_action(&self, _had_prefix: bool, _chord: &str) -> Option<OilKeyAction> {
        None
    }
    fn oil_help_lines(&self) -> Vec<String> {
        Vec::new()
    }
    fn oil_directory_sections(
        &self,
        _root: &std::path::Path,
        _entries: &[DirectoryEntry],
        _show_hidden: bool,
        _sort_mode: OilSortMode,
        _trash_enabled: bool,
    ) -> SectionTree {
        SectionTree::default()
    }
    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str {
        label
    }
    fn git_status_sections(&self, _snapshot: &GitStatusSnapshot) -> SectionTree {
        SectionTree::default()
    }
    fn git_commit_template(&self, _snapshot: &GitStatusSnapshot) -> Vec<String> {
        Vec::new()
    }
    fn git_prefix_for_chord(&self, _chord: &str) -> Option<GitStatusPrefix> {
        None
    }
    fn git_command_for_chord(
        &self,
        _prefix: Option<GitStatusPrefix>,
        _chord: &str,
    ) -> Option<&'static str> {
        None
    }
    fn browser_buffer_lines(&self, _url: Option<&str>) -> Vec<String> {
        Vec::new()
    }
    fn browser_input_hint(&self, _url: Option<&str>) -> String {
        String::new()
    }
    fn browser_url_prompt(&self) -> String {
        String::new()
    }
    fn browser_url_placeholder(&self) -> String {
        String::new()
    }
    fn git_feature_spec(&self) -> GitFeatureSpec {
        GitFeatureSpec::default()
    }
    fn oil_feature_spec(&self) -> OilFeatureSpec {
        OilFeatureSpec::default()
    }
    fn browser_feature_spec(&self) -> BrowserFeatureSpec {
        BrowserFeatureSpec::default()
    }
    fn db_feature_spec(&self) -> DbFeatureSpec {
        DbFeatureSpec::default()
    }
    fn terminal_feature_spec(&self) -> TerminalFeatureSpec {
        TerminalFeatureSpec::default()
    }
    fn context_help_specs(&self) -> Vec<ContextHelpSpec> {
        let mut specs = Vec::new();
        specs.extend(self.git_feature_spec().context_help_specs());

        let oil = self.oil_feature_spec();
        if !oil.help.entries.is_empty() {
            specs.push(oil.help);
        }

        let browser = self.browser_feature_spec();
        if !browser.help.entries.is_empty() {
            specs.push(browser.help);
        }

        specs.extend(self.db_feature_spec().context_help_specs());

        let terminal = self.terminal_feature_spec();
        if !terminal.help.entries.is_empty() {
            specs.push(terminal.help);
        }

        specs
    }
    fn pdf_open_mode(&self) -> PdfOpenMode {
        PdfOpenMode::Rendered
    }
    fn headerline_lines(&self, _context: &GhostTextContext<'_>) -> Vec<String> {
        Vec::new()
    }
    fn ghost_text_lines(&self, _context: &GhostTextContext<'_>) -> Vec<GhostTextLine> {
        Vec::new()
    }
    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String {
        format!(
            " {} | {}:{} ",
            context.buffer_name, context.line, context.column
        )
    }
    fn statusline_spans(&self, context: &StatuslineContext<'_>) -> Vec<StatuslineSpan> {
        flatten_modeline_to_spans(&self.modeline_segments(context))
    }
    fn modeline_segments(&self, context: &StatuslineContext<'_>) -> Vec<ModelineSegment> {
        decode_modeline(&self.statusline_render(context))
    }
    fn statusline_lsp_connected_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_LAN_CONNECT
    }
    fn statusline_lsp_error_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_ERROR
    }
    fn statusline_lsp_warning_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_WARNING
    }
    fn lsp_diagnostic_icon(&self) -> &'static str {
        "●"
    }
    fn lsp_diagnostic_line_limit(&self) -> usize {
        8
    }
    fn lsp_show_buffer_diagnostics(&self) -> bool {
        true
    }
    fn gitfringe_token_added(&self) -> &'static str {
        "git.fringe.added"
    }
    fn gitfringe_token_modified(&self) -> &'static str {
        "git.fringe.modified"
    }
    fn gitfringe_token_removed(&self) -> &'static str {
        "git.fringe.removed"
    }
    fn gitfringe_symbol(&self) -> &'static str {
        "⏽"
    }
    fn icon_symbols(&self) -> &'static [IconFontSymbol] {
        editor_icons::all_symbols()
    }
    fn supports_plugin_evaluate(&self, kind: &str) -> bool {
        self.plugin_buffer(kind)
            .and_then(|buffer| buffer.evaluate_handler().map(str::to_owned))
            .is_some()
    }
    fn handle_plugin_evaluate(&self, kind: &str, input: &str) -> Vec<String> {
        match self
            .plugin_buffer(kind)
            .and_then(|buffer| buffer.evaluate_handler().map(str::to_owned))
        {
            Some(handler) => self.run_plugin_buffer_evaluator(&handler, input),
            None => vec![format!("no evaluator registered for plugin kind `{kind}`")],
        }
    }
    fn plugin_buffer_initial_lines(&self, kind: &str) -> Vec<String> {
        self.plugin_buffer(kind)
            .map(|buffer| {
                buffer
                    .sections()
                    .and_then(|sections| sections.items().first())
                    .map(|section| {
                        section
                            .initial_lines()
                            .iter()
                            .map(|line| line.to_string())
                            .collect()
                    })
                    .unwrap_or_else(|| {
                        buffer
                            .initial_lines()
                            .iter()
                            .map(|line| line.to_string())
                            .collect()
                    })
            })
            .unwrap_or_default()
    }
    fn plugin_buffer_sections(&self, kind: &str) -> Option<PluginBufferSections> {
        self.plugin_buffer(kind)
            .and_then(|buffer| buffer.sections().cloned())
    }
    fn plugin_buffer_key_bindings(&self, kind: &str) -> Vec<PluginKeyBinding> {
        self.plugin_buffer(kind)
            .map(|buffer| buffer.key_bindings().to_vec())
            .unwrap_or_default()
    }
    fn plugin_buffer_line_wrap(&self, kind: &str) -> bool {
        self.plugin_buffer(kind)
            .map(|buffer| buffer.line_wrap())
            .unwrap_or(true)
    }
    fn run_plugin_buffer_evaluator(&self, _handler: &str, _input: &str) -> Vec<String> {
        Vec::new()
    }
    fn plugin_buffer(&self, kind: &str) -> Option<PluginBuffer> {
        self.packages()
            .into_iter()
            .find_map(|package| package.buffer(kind).cloned())
    }
    fn default_build_command(&self, _language: &str) -> Option<String> {
        None
    }
}

impl PluginBuffer {
    /// Creates a new plugin buffer declaration for the given kind.
    pub fn new(kind: impl Into<RString>, initial_lines: Vec<impl Into<RString>>) -> Self {
        Self {
            kind: kind.into(),
            initial_lines: initial_lines
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            sections: ROption::RNone,
            evaluate_handler: ROption::RNone,
            evaluate_target_section: ROption::RNone,
            line_wrap: true,
            key_bindings: RVec::new(),
        }
    }

    /// Attaches split-pane metadata for the buffer.
    pub fn with_sections(mut self, sections: PluginBufferSections) -> Self {
        self.sections = ROption::RSome(sections);
        self
    }

    /// Declares the evaluator handler id used when `plugin.evaluate` fires.
    pub fn with_evaluate_handler(mut self, handler: impl Into<RString>) -> Self {
        self.evaluate_handler = ROption::RSome(handler.into());
        self
    }

    /// Declares which section receives evaluation output.
    pub fn with_evaluate_target_section(mut self, section_name: impl Into<RString>) -> Self {
        self.evaluate_target_section = ROption::RSome(section_name.into());
        self
    }

    /// Attaches keybindings that are only active while this buffer kind is focused.
    pub fn with_key_bindings(mut self, key_bindings: Vec<PluginKeyBinding>) -> Self {
        self.key_bindings = key_bindings.into();
        self
    }

    /// Controls whether this buffer wraps long lines by default.
    pub fn with_line_wrap(mut self, line_wrap: bool) -> Self {
        self.line_wrap = line_wrap;
        self
    }

    /// Returns the plugin buffer kind.
    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }

    /// Returns the initial text content for the buffer.
    pub fn initial_lines(&self) -> &[RString] {
        self.initial_lines.as_slice()
    }

    /// Returns the optional split-pane metadata for this buffer.
    pub fn sections(&self) -> Option<&PluginBufferSections> {
        match &self.sections {
            ROption::RSome(sections) => Some(sections),
            ROption::RNone => None,
        }
    }

    /// Returns the optional evaluator handler id for this buffer.
    pub fn evaluate_handler(&self) -> Option<&str> {
        match &self.evaluate_handler {
            ROption::RSome(handler) => Some(handler.as_str()),
            ROption::RNone => None,
        }
    }

    /// Returns the optional section that should receive evaluation output.
    pub fn evaluate_target_section(&self) -> Option<&str> {
        match &self.evaluate_target_section {
            ROption::RSome(section_name) => Some(section_name.as_str()),
            ROption::RNone => None,
        }
    }

    /// Returns the keybindings attached to this buffer kind.
    pub fn key_bindings(&self) -> &[PluginKeyBinding] {
        self.key_bindings.as_slice()
    }

    /// Returns whether this buffer wraps long lines by default.
    pub const fn line_wrap(&self) -> bool {
        self.line_wrap
    }
}

// ─── Generic plugin hooks ─────────────────────────────────────────────────────

/// Hook names owned by the host application's generic plugin infrastructure.
/// User plugins emit these hooks; the host handles them without needing to know
/// which specific plugin fired them.
pub mod plugin_hooks {
    /// Emitted by any user plugin that wants the host to evaluate the active
    /// buffer's input section and write the result to the output section.
    /// The host calls `UserLibrary::handle_plugin_evaluate` (defined in
    /// `editor-plugin-host`) with the active buffer's kind string.
    /// The separator line that divides input from output is
    /// [`EVALUATE_SEPARATOR_PREFIX`].
    pub const EVALUATE: &str = "plugin.evaluate";

    /// A line whose text starts with this prefix is treated as the output
    /// separator in an evaluatable plugin buffer.
    pub const EVALUATE_SEPARATOR_PREFIX: &str = "─── Output";

    /// Emitted when a plugin wants the host to run a build/compile command.
    /// Detail format: `{language}` (e.g. `"rust"`).  The host looks up the
    /// default command via `UserLibrary::default_build_command`, opens a
    /// prompt pre-filled with it, then streams output into the
    /// `*compile <workspace>*` popup.
    pub const RUN_COMMAND: &str = "plugin.run-command";

    /// Emitted when a plugin wants the host to re-run the last build command
    /// for the active workspace.  If no command has been run yet the host
    /// falls back to [`RUN_COMMAND`].
    pub const RERUN_COMMAND: &str = "plugin.rerun-command";

    /// Emitted when a plugin wants the host to switch focus between its
    /// currently active split panes (for example between input and output).
    pub const SWITCH_PANE: &str = "plugin.switch-pane";
}

// ─── Git action / section ID constants ───────────────────────────────────────

/// Section action IDs for the git status buffer.
pub mod git_actions {
    pub const STAGE_FILE: &str = "git.stage-file";
    pub const STAGE_ALL: &str = "git.stage-all";
    pub const UNSTAGE_FILE: &str = "git.unstage-file";
    pub const COMMIT_OPEN: &str = "git.commit-open";
    pub const PUSH: &str = "git.push";
    pub const SHOW_COMMIT: &str = "git.show-commit";
    pub const SHOW_STASH: &str = "git.show-stash";
}

/// Section IDs used in the git status buffer tree.
pub mod git_sections {
    pub const HEADERS: &str = "git.status.headers";
    pub const IN_PROGRESS: &str = "git.status.in-progress";
    pub const STAGED: &str = "git.status.staged";
    pub const UNSTAGED: &str = "git.status.unstaged";
    pub const UNTRACKED: &str = "git.status.untracked";
    pub const STASHES: &str = "git.status.stashes";
    pub const UNPULLED: &str = "git.status.unpulled";
    pub const UNPUSHED: &str = "git.status.unpushed";
    pub const REMOTE: &str = "git.status.remote";
    pub const COMMIT: &str = "git.status.commit";
}

// ─── Oil directory browser constants ─────────────────────────────────────────

/// Section / action ID constants for the oil directory browser.
pub mod oil_protocol {
    pub const ACTION_OIL_ENTRY: &str = "oil.entry";
    pub const SECTION_OIL_DIRECTORY: &str = "oil.directory";
}

// ─── Shared configuration types ──────────────────────────────────────────────

/// One contextual help row surfaced by a feature.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextHelpEntry {
    pub chord: String,
    pub action: String,
    pub description: String,
}

impl ContextHelpEntry {
    /// Creates one contextual help row.
    pub fn new(
        chord: impl Into<String>,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            chord: chord.into(),
            action: action.into(),
            description: description.into(),
        }
    }
}

/// Contextual help exported for a specific feature scope.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextHelpSpec {
    pub scope: String,
    pub title: String,
    pub entries: Vec<ContextHelpEntry>,
}

impl ContextHelpSpec {
    /// Creates one contextual help group.
    pub fn new(
        scope: impl Into<String>,
        title: impl Into<String>,
        entries: Vec<ContextHelpEntry>,
    ) -> Self {
        Self {
            scope: scope.into(),
            title: title.into(),
            entries,
        }
    }
}

/// User-declared Vim edit action carried through the legacy hook detail boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimEditAction {
    EnterVisual,
    EnterVisualLine,
    EnterVisualBlock,
    DeleteChar,
    DeleteCharBefore,
    DeleteLineEnd,
    ChangeLineEnd,
    YankLine,
    SubstituteChar,
    SubstituteLine,
    ReplaceChar,
    EnterReplaceMode,
    ToggleCase,
    StartDeleteOperator,
    StartChangeOperator,
    StartYankOperator,
    StartFormatOperator,
    VisualFormat,
    ToggleLineComment,
    VisualToggleComment,
    Append,
    AppendLineEnd,
    InsertLineStart,
    OpenLineBelow,
    OpenLineAbove,
    Undo,
    Redo,
    MulticursorAddNextMatch,
    MulticursorSelectAllMatches,
    StartGPrefix,
    StartFindForward,
    StartFindBackward,
    StartTillForward,
    StartTillBackward,
    RepeatFindNext,
    RepeatFindPrevious,
    StartSearchForward,
    StartSearchBackward,
    SearchWordForward,
    SearchWordBackward,
    RepeatSearchNext,
    RepeatSearchPrevious,
    SelectRegister,
    SetMark,
    GotoMarkLine,
    GotoMark,
    ToggleMacroRecord,
    StartMacroPlayback,
    PutAfter,
    PutBefore,
    VisualPutAfter,
    VisualPutBefore,
    VisualDelete,
    VisualChange,
    VisualReplaceChar,
    VisualBlockInsert,
    VisualBlockAppend,
    VisualYank,
    VisualToggleCase,
    VisualLowercase,
    VisualUppercase,
    VisualIndent,
    VisualOutdent,
    VisualJoin,
    VisualMoveDown,
    VisualMoveUp,
    VisualSwapAnchor,
    StartVisualInnerTextObject,
    StartVisualAroundTextObject,
}

/// Public Vim action spec used by user Vim bindings.
pub type VimActionSpec = VimEditAction;

/// Host-visible context for one user-declared Vim action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimActionContext {
    pub action: VimActionSpec,
}

impl VimActionContext {
    pub const fn new(action: VimActionSpec) -> Self {
        Self { action }
    }

    pub const fn action(self) -> VimActionSpec {
        self.action
    }
}

impl VimEditAction {
    pub fn hook_detail(self) -> &'static str {
        match self {
            Self::EnterVisual => "enter-visual",
            Self::EnterVisualLine => "enter-visual-line",
            Self::EnterVisualBlock => "enter-visual-block",
            Self::DeleteChar => "delete-char",
            Self::DeleteCharBefore => "delete-char-before",
            Self::DeleteLineEnd => "delete-line-end",
            Self::ChangeLineEnd => "change-line-end",
            Self::YankLine => "yank-line",
            Self::SubstituteChar => "substitute-char",
            Self::SubstituteLine => "substitute-line",
            Self::ReplaceChar => "replace-char",
            Self::EnterReplaceMode => "enter-replace-mode",
            Self::ToggleCase => "toggle-case",
            Self::StartDeleteOperator => "start-delete-operator",
            Self::StartChangeOperator => "start-change-operator",
            Self::StartYankOperator => "start-yank-operator",
            Self::StartFormatOperator => "start-format-operator",
            Self::VisualFormat => "visual-format",
            Self::ToggleLineComment => "toggle-line-comment",
            Self::VisualToggleComment => "visual-toggle-comment",
            Self::Append => "append",
            Self::AppendLineEnd => "append-line-end",
            Self::InsertLineStart => "insert-line-start",
            Self::OpenLineBelow => "open-line-below",
            Self::OpenLineAbove => "open-line-above",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::MulticursorAddNextMatch => "multicursor-add-next-match",
            Self::MulticursorSelectAllMatches => "multicursor-select-all-matches",
            Self::StartGPrefix => "start-g-prefix",
            Self::StartFindForward => "start-find-forward",
            Self::StartFindBackward => "start-find-backward",
            Self::StartTillForward => "start-till-forward",
            Self::StartTillBackward => "start-till-backward",
            Self::RepeatFindNext => "repeat-find-next",
            Self::RepeatFindPrevious => "repeat-find-previous",
            Self::StartSearchForward => "start-search-forward",
            Self::StartSearchBackward => "start-search-backward",
            Self::SearchWordForward => "search-word-forward",
            Self::SearchWordBackward => "search-word-backward",
            Self::RepeatSearchNext => "repeat-search-next",
            Self::RepeatSearchPrevious => "repeat-search-previous",
            Self::SelectRegister => "select-register",
            Self::SetMark => "set-mark",
            Self::GotoMarkLine => "goto-mark-line",
            Self::GotoMark => "goto-mark",
            Self::ToggleMacroRecord => "toggle-macro-record",
            Self::StartMacroPlayback => "start-macro-playback",
            Self::PutAfter => "put-after",
            Self::PutBefore => "put-before",
            Self::VisualPutAfter => "visual-put-after",
            Self::VisualPutBefore => "visual-put-before",
            Self::VisualDelete => "visual-delete",
            Self::VisualChange => "visual-change",
            Self::VisualReplaceChar => "visual-replace-char",
            Self::VisualBlockInsert => "visual-block-insert",
            Self::VisualBlockAppend => "visual-block-append",
            Self::VisualYank => "visual-yank",
            Self::VisualToggleCase => "visual-toggle-case",
            Self::VisualLowercase => "visual-lowercase",
            Self::VisualUppercase => "visual-uppercase",
            Self::VisualIndent => "visual-indent",
            Self::VisualOutdent => "visual-outdent",
            Self::VisualJoin => "visual-join",
            Self::VisualMoveDown => "visual-move-down",
            Self::VisualMoveUp => "visual-move-up",
            Self::VisualSwapAnchor => "visual-swap-anchor",
            Self::StartVisualInnerTextObject => "start-visual-inner-text-object",
            Self::StartVisualAroundTextObject => "start-visual-around-text-object",
        }
    }

    pub fn from_hook_detail(detail: &str) -> Option<Self> {
        match detail {
            "enter-visual" => Some(Self::EnterVisual),
            "enter-visual-line" => Some(Self::EnterVisualLine),
            "enter-visual-block" => Some(Self::EnterVisualBlock),
            "delete-char" => Some(Self::DeleteChar),
            "delete-char-before" => Some(Self::DeleteCharBefore),
            "delete-line-end" => Some(Self::DeleteLineEnd),
            "change-line-end" => Some(Self::ChangeLineEnd),
            "yank-line" => Some(Self::YankLine),
            "substitute-char" => Some(Self::SubstituteChar),
            "substitute-line" => Some(Self::SubstituteLine),
            "replace-char" => Some(Self::ReplaceChar),
            "enter-replace-mode" => Some(Self::EnterReplaceMode),
            "toggle-case" => Some(Self::ToggleCase),
            "start-delete-operator" => Some(Self::StartDeleteOperator),
            "start-change-operator" => Some(Self::StartChangeOperator),
            "start-yank-operator" => Some(Self::StartYankOperator),
            "start-format-operator" => Some(Self::StartFormatOperator),
            "visual-format" => Some(Self::VisualFormat),
            "toggle-line-comment" => Some(Self::ToggleLineComment),
            "visual-toggle-comment" => Some(Self::VisualToggleComment),
            "append" => Some(Self::Append),
            "append-line-end" => Some(Self::AppendLineEnd),
            "insert-line-start" => Some(Self::InsertLineStart),
            "open-line-below" => Some(Self::OpenLineBelow),
            "open-line-above" => Some(Self::OpenLineAbove),
            "undo" => Some(Self::Undo),
            "redo" => Some(Self::Redo),
            "multicursor-add-next-match" => Some(Self::MulticursorAddNextMatch),
            "multicursor-select-all-matches" => Some(Self::MulticursorSelectAllMatches),
            "start-g-prefix" => Some(Self::StartGPrefix),
            "start-find-forward" => Some(Self::StartFindForward),
            "start-find-backward" => Some(Self::StartFindBackward),
            "start-till-forward" => Some(Self::StartTillForward),
            "start-till-backward" => Some(Self::StartTillBackward),
            "repeat-find-next" => Some(Self::RepeatFindNext),
            "repeat-find-previous" => Some(Self::RepeatFindPrevious),
            "start-search-forward" => Some(Self::StartSearchForward),
            "start-search-backward" => Some(Self::StartSearchBackward),
            "search-word-forward" => Some(Self::SearchWordForward),
            "search-word-backward" => Some(Self::SearchWordBackward),
            "repeat-search-next" => Some(Self::RepeatSearchNext),
            "repeat-search-previous" => Some(Self::RepeatSearchPrevious),
            "select-register" => Some(Self::SelectRegister),
            "set-mark" => Some(Self::SetMark),
            "goto-mark-line" => Some(Self::GotoMarkLine),
            "goto-mark" => Some(Self::GotoMark),
            "toggle-macro-record" => Some(Self::ToggleMacroRecord),
            "start-macro-playback" => Some(Self::StartMacroPlayback),
            "put-after" => Some(Self::PutAfter),
            "put-before" => Some(Self::PutBefore),
            "visual-put-after" => Some(Self::VisualPutAfter),
            "visual-put-before" => Some(Self::VisualPutBefore),
            "visual-delete" => Some(Self::VisualDelete),
            "visual-change" => Some(Self::VisualChange),
            "visual-replace-char" => Some(Self::VisualReplaceChar),
            "visual-block-insert" => Some(Self::VisualBlockInsert),
            "visual-block-append" => Some(Self::VisualBlockAppend),
            "visual-yank" => Some(Self::VisualYank),
            "visual-toggle-case" => Some(Self::VisualToggleCase),
            "visual-lowercase" => Some(Self::VisualLowercase),
            "visual-uppercase" => Some(Self::VisualUppercase),
            "visual-indent" => Some(Self::VisualIndent),
            "visual-outdent" => Some(Self::VisualOutdent),
            "visual-join" => Some(Self::VisualJoin),
            "visual-move-down" => Some(Self::VisualMoveDown),
            "visual-move-up" => Some(Self::VisualMoveUp),
            "visual-swap-anchor" => Some(Self::VisualSwapAnchor),
            "start-visual-inner-text-object" => Some(Self::StartVisualInnerTextObject),
            "start-visual-around-text-object" => Some(Self::StartVisualAroundTextObject),
            _ => None,
        }
    }
}

/// User-configurable sort mode for oil directory buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OilSortMode {
    TypeThenName,
    TypeThenNameDesc,
}

impl OilSortMode {
    /// Returns the human-readable label shown in the oil buffer header.
    pub fn label(self) -> &'static str {
        match self {
            Self::TypeThenName => "type+name",
            Self::TypeThenNameDesc => "type+name desc",
        }
    }

    /// Returns the next mode in the cycle used by the oil UI.
    pub fn cycle(self) -> Self {
        match self {
            Self::TypeThenName => Self::TypeThenNameDesc,
            Self::TypeThenNameDesc => Self::TypeThenName,
        }
    }
}

/// An action resolved from an oil key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OilKeyAction {
    OpenEntry,
    OpenVerticalSplit,
    OpenHorizontalSplit,
    OpenNewPane,
    PreviewEntry,
    Refresh,
    Close,
    StartPrefix,
    OpenParent,
    OpenWorkspaceRoot,
    SetRoot,
    ShowHelp,
    CycleSort,
    ToggleHidden,
    ToggleTrash,
    OpenExternal,
    SetTabLocalRoot,
    CreateGitWorktree,
}

impl OilKeyAction {
    /// Returns the legacy hook detail used at the plugin action boundary.
    pub fn hook_detail(self) -> Option<&'static str> {
        match self {
            Self::OpenEntry => Some("open-entry"),
            Self::OpenVerticalSplit => Some("open-vertical-split"),
            Self::OpenHorizontalSplit => Some("open-horizontal-split"),
            Self::OpenNewPane => Some("open-new-pane"),
            Self::PreviewEntry => Some("preview-entry"),
            Self::Refresh => Some("refresh"),
            Self::Close => Some("close"),
            Self::StartPrefix => None,
            Self::OpenParent => Some("open-parent"),
            Self::OpenWorkspaceRoot => Some("open-workspace-root"),
            Self::SetRoot => Some("set-root"),
            Self::ShowHelp => Some("show-help"),
            Self::CycleSort => Some("cycle-sort"),
            Self::ToggleHidden => Some("toggle-hidden"),
            Self::ToggleTrash => Some("toggle-trash"),
            Self::OpenExternal => Some("open-external"),
            Self::SetTabLocalRoot => Some("set-tab-local-root"),
            Self::CreateGitWorktree => Some("git-worktree"),
        }
    }

    /// Converts legacy hook detail into the typed oil action contract.
    pub fn from_hook_detail(detail: &str) -> Option<Self> {
        match detail {
            "open-entry" => Some(Self::OpenEntry),
            "open-vertical-split" => Some(Self::OpenVerticalSplit),
            "open-horizontal-split" => Some(Self::OpenHorizontalSplit),
            "open-new-pane" => Some(Self::OpenNewPane),
            "preview-entry" => Some(Self::PreviewEntry),
            "refresh" => Some(Self::Refresh),
            "close" => Some(Self::Close),
            "open-parent" => Some(Self::OpenParent),
            "open-workspace-root" => Some(Self::OpenWorkspaceRoot),
            "set-root" => Some(Self::SetRoot),
            "show-help" => Some(Self::ShowHelp),
            "cycle-sort" => Some(Self::CycleSort),
            "toggle-hidden" => Some(Self::ToggleHidden),
            "toggle-trash" => Some(Self::ToggleTrash),
            "open-external" => Some(Self::OpenExternal),
            "set-tab-local-root" => Some(Self::SetTabLocalRoot),
            "git-worktree" => Some(Self::CreateGitWorktree),
            _ => None,
        }
    }
}

/// User-configurable default options for new oil directory buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OilDefaults {
    pub show_hidden: bool,
    pub sort_mode: OilSortMode,
    pub trash_enabled: bool,
}

impl Default for OilDefaults {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort_mode: OilSortMode::TypeThenName,
            trash_enabled: false,
        }
    }
}

/// User-configurable keybindings for the oil directory browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OilKeybindings {
    pub open_entry: &'static str,
    pub open_vertical_split: &'static str,
    pub open_horizontal_split: &'static str,
    pub open_new_pane: &'static str,
    pub preview_entry: &'static str,
    pub refresh: &'static str,
    pub close: &'static str,
    pub prefix: &'static str,
    pub open_parent: &'static str,
    pub open_workspace_root: &'static str,
    pub set_root: &'static str,
    pub show_help: &'static str,
    pub cycle_sort: &'static str,
    pub toggle_hidden: &'static str,
    pub toggle_trash: &'static str,
    pub open_external: &'static str,
    pub set_tab_local_root: &'static str,
    pub create_git_worktree: &'static str,
}

impl Default for OilKeybindings {
    fn default() -> Self {
        Self {
            open_entry: "Enter",
            open_vertical_split: "Ctrl+\\",
            open_horizontal_split: "Ctrl+|",
            open_new_pane: "Ctrl+t",
            preview_entry: "Ctrl+p",
            refresh: "Ctrl+l",
            close: "Ctrl+c",
            prefix: "g",
            open_parent: "-",
            open_workspace_root: "_",
            set_root: "`",
            show_help: "?",
            cycle_sort: "s",
            toggle_hidden: ".",
            toggle_trash: "\\",
            open_external: "x",
            set_tab_local_root: "~",
            create_git_worktree: "wn",
        }
    }
}

/// Git key-chord action prefix kind used for file-scoped git commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatusPrefix {
    Commit,
    Push,
    Fetch,
    Pull,
    Branch,
    Diff,
    Log,
    Stash,
    Merge,
    Rebase,
    CherryPick,
    Revert,
    Reset,
}

/// One prefix starter exported by git status feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitPrefixBinding {
    pub chord: String,
    pub prefix: GitStatusPrefix,
    pub action: String,
    pub description: String,
}

impl GitPrefixBinding {
    /// Creates one prefix starter binding.
    pub fn new(
        chord: impl Into<String>,
        prefix: GitStatusPrefix,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            chord: chord.into(),
            prefix,
            action: action.into(),
            description: description.into(),
        }
    }
}

/// One git command binding resolved from optional prefix + chord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommandBinding {
    pub prefix: Option<GitStatusPrefix>,
    pub chord: String,
    pub command_name: String,
    pub action: String,
    pub description: String,
}

impl GitCommandBinding {
    /// Creates one git command binding.
    pub fn new(
        prefix: Option<GitStatusPrefix>,
        chord: impl Into<String>,
        command_name: impl Into<String>,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            prefix,
            chord: chord.into(),
            command_name: command_name.into(),
            action: action.into(),
            description: description.into(),
        }
    }
}

/// Public contract for first-party git workflows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitFeatureSpec {
    pub status_buffer_name: String,
    pub commit_buffer_name: String,
    pub branch_popup_title: String,
    pub prefix_bindings: Vec<GitPrefixBinding>,
    pub command_bindings: Vec<GitCommandBinding>,
    pub status_help: ContextHelpSpec,
    pub view_help: ContextHelpSpec,
}

impl GitFeatureSpec {
    /// Resolves status-prefix starter chord.
    pub fn prefix_for_chord(&self, chord: &str) -> Option<GitStatusPrefix> {
        self.prefix_bindings
            .iter()
            .find(|binding| binding.chord == chord)
            .map(|binding| binding.prefix)
    }

    /// Resolves git command name from optional prefix + chord.
    pub fn command_for_chord(&self, prefix: Option<GitStatusPrefix>, chord: &str) -> Option<&str> {
        self.command_bindings
            .iter()
            .find(|binding| binding.prefix == prefix && binding.chord == chord)
            .map(|binding| binding.command_name.as_str())
    }

    /// Returns contextual help groups contributed by git feature.
    pub fn context_help_specs(&self) -> Vec<ContextHelpSpec> {
        vec![self.status_help.clone(), self.view_help.clone()]
    }
}

/// Public contract for oil directory workflows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OilFeatureSpec {
    pub defaults: OilDefaults,
    pub keybindings: OilKeybindings,
    pub help: ContextHelpSpec,
}

/// Public contract for browser workflows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BrowserFeatureSpec {
    pub buffer_name: String,
    pub url_prompt: String,
    pub url_placeholder: String,
    pub input_hint: String,
    pub help: ContextHelpSpec,
}

/// Public contract for database workflows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DbFeatureSpec {
    pub connect_buffer_name: String,
    pub connections_buffer_name: String,
    pub schema_buffer_name: String,
    pub history_buffer_name: String,
    pub snippets_buffer_name: String,
    pub results_buffer_name: String,
    pub execute_chord: String,
    pub connect_help: ContextHelpSpec,
    pub query_help: ContextHelpSpec,
    pub browser_help: ContextHelpSpec,
}

impl DbFeatureSpec {
    /// Returns contextual help groups contributed by database feature.
    pub fn context_help_specs(&self) -> Vec<ContextHelpSpec> {
        vec![
            self.connect_help.clone(),
            self.query_help.clone(),
            self.browser_help.clone(),
        ]
    }
}

/// Public contract for terminal workflows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerminalFeatureSpec {
    pub buffer_name: String,
    pub popup_buffer_name: String,
    pub help: ContextHelpSpec,
}

/// Autocomplete provider configuration exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutocompleteProvider {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub item_icon: String,
    pub or_group: Option<String>,
    pub buffer_kind: Option<String>,
    pub items: Vec<AutocompleteProviderItem>,
}

/// Static autocomplete item exported by a user library provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutocompleteProviderItem {
    pub label: String,
    pub replacement: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

/// Hover provider configuration exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverProvider {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub line_limit: usize,
    pub buffer_kind: Option<String>,
    pub topics: Vec<HoverProviderTopic>,
}

/// Static hover topic exported by a user library provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverProviderTopic {
    pub token: String,
    pub lines: Vec<String>,
}

/// ACP client configuration exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpClient {
    pub id: String,
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<String>,
}

/// Project search root exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRoot {
    pub path: String,
    pub max_depth: usize,
}

/// Terminal configuration exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalConfig {
    pub program: String,
    pub args: Vec<String>,
}

/// Default presentation mode used when opening PDF buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfOpenMode {
    Rendered,
    Markdown,
    Latex,
}

/// Text ligature configuration exported by the user library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LigatureConfig {
    pub enabled: bool,
}

/// Rainbow delimiter configuration exported by the user library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RainbowParensConfig {
    pub enabled: bool,
}

/// Pane layout configuration exported by the user library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneConfig {
    pub golden_ratio: bool,
}

/// Side of the window where the workspace dock appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceDockSide {
    #[default]
    Left,
    Right,
}

/// Workspace dock configuration exported by the user library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceDockConfig {
    pub side: WorkspaceDockSide,
    pub docked: bool,
}

impl Default for WorkspaceDockConfig {
    fn default() -> Self {
        Self {
            side: WorkspaceDockSide::Left,
            docked: false,
        }
    }
}

/// One treesitter-node → icon entry for Markdown Pretty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownPrettyIcon {
    pub node_kind: String,
    pub icon: String,
}

/// Markdown Pretty settings exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownPrettyConfig {
    pub enabled: bool,
    pub kill_switch_enabled: bool,
    pub kill_switch_max_lines: usize,
    pub kill_switch_max_bytes: usize,
    pub image_max_bytes: usize,
    pub image_max_rows: usize,
    pub icons: Vec<MarkdownPrettyIcon>,
}

impl Default for MarkdownPrettyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            kill_switch_enabled: false,
            kill_switch_max_lines: 20_000,
            kill_switch_max_bytes: 2_000_000,
            image_max_bytes: 10_000_000,
            image_max_rows: 24,
            icons: Vec::new(),
        }
    }
}

/// Keymap tunables exported by the user library (`ui.keymap.*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeymapConfig {
    /// Ambiguous-prefix timeout in milliseconds.
    pub ambiguous_prefix_timeout_ms: u64,
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self {
            ambiguous_prefix_timeout_ms: editor_core::DEFAULT_AMBIGUOUS_PREFIX_TIMEOUT_MS,
        }
    }
}

/// LSP diagnostic counts surfaced to the statusline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspDiagnosticsInfo {
    pub errors: usize,
    pub warnings: usize,
}

/// Stable keymap scopes shared across the host and the compiled user library.
///
/// Spoken product term for non-Global scopes is Minor Mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PluginKeymapScope {
    /// Binding is active globally (fallback when no Minor Mode claims the chord).
    Global,
    /// Workspace editing Minor Mode.
    Workspace,
    /// Popup Minor Mode (picker / popup focus).
    Popup,
    /// Autocomplete overlay Minor Mode.
    Autocomplete,
    /// Hover overlay Minor Mode.
    Hover,
}

/// Modal Vim state that can activate a keybinding.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PluginVimMode {
    /// Binding is always active regardless of the current Vim mode.
    Any,
    /// Binding is active while Vim normal mode is focused.
    Normal,
    /// Binding is active while Vim insert mode is focused.
    Insert,
    /// Binding is active while Vim visual mode is focused.
    Visual,
}

/// A command exported by a user package.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginCommand {
    name: RString,
    description: RString,
    actions: RVec<PluginAction>,
}

impl PluginCommand {
    /// Creates a new exported command.
    pub fn new(
        name: impl Into<RString>,
        description: impl Into<RString>,
        actions: Vec<PluginAction>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            actions: actions.into(),
        }
    }

    /// Returns the command identifier.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the command summary.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the actions performed when the command executes.
    pub fn actions(&self) -> &[PluginAction] {
        self.actions.as_slice()
    }
}

/// Action tags supported by the stable plugin ABI.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PluginActionKind {
    /// Write a diagnostic message through the host.
    LogMessage,
    /// Create or surface a buffer.
    OpenBuffer,
    /// Emit a hook event.
    EmitHook,
}

/// Describes how a buffer should be opened by the host runtime.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginBufferAction {
    buffer_name: RString,
    buffer_kind: RString,
    popup_title: ROption<RString>,
}

impl PluginBufferAction {
    /// Creates a new buffer-open action.
    pub fn new(
        buffer_name: impl Into<RString>,
        buffer_kind: impl Into<RString>,
        popup_title: Option<impl Into<RString>>,
    ) -> Self {
        Self {
            buffer_name: buffer_name.into(),
            buffer_kind: buffer_kind.into(),
            popup_title: match popup_title {
                Some(title) => ROption::RSome(title.into()),
                None => ROption::RNone,
            },
        }
    }

    /// Returns the target buffer name.
    pub fn buffer_name(&self) -> &str {
        self.buffer_name.as_str()
    }

    /// Returns the buffer kind tag consumed by the host.
    pub fn buffer_kind(&self) -> &str {
        self.buffer_kind.as_str()
    }

    /// Returns the popup title, if the buffer should open in a popup.
    pub fn popup_title(&self) -> Option<&str> {
        match &self.popup_title {
            ROption::RSome(title) => Some(title.as_str()),
            ROption::RNone => None,
        }
    }
}

/// Describes how a hook event should be emitted by the host runtime.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginEmitHookAction {
    hook_name: RString,
    detail: ROption<RString>,
}

impl PluginEmitHookAction {
    /// Creates a new hook emission action.
    pub fn new(hook_name: impl Into<RString>, detail: Option<impl Into<RString>>) -> Self {
        Self {
            hook_name: hook_name.into(),
            detail: match detail {
                Some(detail) => ROption::RSome(detail.into()),
                None => ROption::RNone,
            },
        }
    }

    /// Returns the hook identifier.
    pub fn hook_name(&self) -> &str {
        self.hook_name.as_str()
    }

    /// Returns the optional event detail.
    pub fn detail(&self) -> Option<&str> {
        match &self.detail {
            ROption::RSome(detail) => Some(detail.as_str()),
            ROption::RNone => None,
        }
    }
}

/// Stable action payload used by exported commands.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginAction {
    kind: PluginActionKind,
    message: ROption<RString>,
    buffer: ROption<PluginBufferAction>,
    hook: ROption<PluginEmitHookAction>,
}

impl PluginAction {
    /// Creates a logging action.
    pub fn log_message(message: impl Into<RString>) -> Self {
        Self {
            kind: PluginActionKind::LogMessage,
            message: ROption::RSome(message.into()),
            buffer: ROption::RNone,
            hook: ROption::RNone,
        }
    }

    /// Creates a buffer-opening action.
    pub fn open_buffer(
        buffer_name: impl Into<RString>,
        buffer_kind: impl Into<RString>,
        popup_title: Option<impl Into<RString>>,
    ) -> Self {
        Self {
            kind: PluginActionKind::OpenBuffer,
            message: ROption::RNone,
            buffer: ROption::RSome(PluginBufferAction::new(
                buffer_name,
                buffer_kind,
                popup_title,
            )),
            hook: ROption::RNone,
        }
    }

    /// Creates a hook-emission action.
    pub fn emit_hook(hook_name: impl Into<RString>, detail: Option<impl Into<RString>>) -> Self {
        Self {
            kind: PluginActionKind::EmitHook,
            message: ROption::RNone,
            buffer: ROption::RNone,
            hook: ROption::RSome(PluginEmitHookAction::new(hook_name, detail)),
        }
    }

    /// Returns the action kind.
    pub const fn kind(&self) -> PluginActionKind {
        self.kind
    }

    /// Returns the log message payload when present.
    pub fn message(&self) -> Option<&str> {
        match &self.message {
            ROption::RSome(message) => Some(message.as_str()),
            ROption::RNone => None,
        }
    }

    /// Returns the buffer payload when present.
    pub fn buffer(&self) -> Option<&PluginBufferAction> {
        match &self.buffer {
            ROption::RSome(buffer) => Some(buffer),
            ROption::RNone => None,
        }
    }

    /// Returns the hook payload when present.
    pub fn hook(&self) -> Option<&PluginEmitHookAction> {
        match &self.hook {
            ROption::RSome(hook) => Some(hook),
            ROption::RNone => None,
        }
    }
}

/// Metadata for a keybinding exported by a user package.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginKeyBinding {
    chord: RString,
    command_names: RVec<RString>,
    scope: PluginKeymapScope,
    vim_mode: PluginVimMode,
}

impl PluginKeyBinding {
    /// Creates a new keybinding.
    pub fn new(
        chord: impl Into<RString>,
        command_name: impl Into<RString>,
        scope: PluginKeymapScope,
    ) -> Self {
        Self::new_many(chord, [command_name], scope)
    }

    /// Creates a new keybinding that executes multiple commands in order.
    pub fn new_many<I, S>(
        chord: impl Into<RString>,
        command_names: I,
        scope: PluginKeymapScope,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<RString>,
    {
        let command_names = command_names
            .into_iter()
            .map(Into::into)
            .collect::<RVec<_>>();
        assert!(
            !command_names.is_empty(),
            "PluginKeyBinding requires at least one command"
        );
        Self {
            chord: chord.into(),
            command_names,
            scope,
            vim_mode: PluginVimMode::Any,
        }
    }

    /// Sets the Vim mode that activates the binding.
    pub fn with_vim_mode(mut self, vim_mode: PluginVimMode) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Returns the key chord.
    pub fn chord(&self) -> &str {
        self.chord.as_str()
    }

    /// Returns the first command targeted by the keybinding.
    pub fn command_name(&self) -> &str {
        self.command_names[0].as_str()
    }

    /// Returns all commands targeted by the keybinding.
    pub fn command_names(&self) -> &[RString] {
        self.command_names.as_slice()
    }

    /// Returns the scope that activates the keybinding.
    pub const fn scope(&self) -> PluginKeymapScope {
        self.scope
    }

    /// Returns the Vim mode that activates the binding.
    pub const fn vim_mode(&self) -> PluginVimMode {
        self.vim_mode
    }
}

/// Declares a custom hook exported by a user package.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginHookDeclaration {
    name: RString,
    description: RString,
}

impl PluginHookDeclaration {
    /// Creates a new custom hook declaration.
    pub fn new(name: impl Into<RString>, description: impl Into<RString>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }

    /// Returns the hook name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the hook description.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

/// Subscribes a package command to a hook.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginHookBinding {
    hook_name: RString,
    subscriber: RString,
    command_name: RString,
    detail_filter: ROption<RString>,
}

impl PluginHookBinding {
    /// Creates a new hook subscription.
    pub fn new(
        hook_name: impl Into<RString>,
        subscriber: impl Into<RString>,
        command_name: impl Into<RString>,
        detail_filter: Option<impl Into<RString>>,
    ) -> Self {
        Self {
            hook_name: hook_name.into(),
            subscriber: subscriber.into(),
            command_name: command_name.into(),
            detail_filter: match detail_filter {
                Some(filter) => ROption::RSome(filter.into()),
                None => ROption::RNone,
            },
        }
    }

    /// Returns the subscribed hook name.
    pub fn hook_name(&self) -> &str {
        self.hook_name.as_str()
    }

    /// Returns the subscriber identifier.
    pub fn subscriber(&self) -> &str {
        self.subscriber.as_str()
    }

    /// Returns the command that should run when the hook fires.
    pub fn command_name(&self) -> &str {
        self.command_name.as_str()
    }

    /// Returns the optional detail filter.
    pub fn detail_filter(&self) -> Option<&str> {
        match &self.detail_filter {
            ROption::RSome(filter) => Some(filter.as_str()),
            ROption::RNone => None,
        }
    }
}

/// Metadata advertised by a user package to the core host.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginPackage {
    name: RString,
    auto_load: bool,
    description: RString,
    commands: RVec<PluginCommand>,
    key_bindings: RVec<PluginKeyBinding>,
    hook_declarations: RVec<PluginHookDeclaration>,
    hook_bindings: RVec<PluginHookBinding>,
    buffers: RVec<PluginBuffer>,
}

impl PluginPackage {
    /// Creates a new package metadata record.
    pub fn new(name: impl Into<RString>, auto_load: bool, description: impl Into<RString>) -> Self {
        Self {
            name: name.into(),
            auto_load,
            description: description.into(),
            commands: RVec::new(),
            key_bindings: RVec::new(),
            hook_declarations: RVec::new(),
            hook_bindings: RVec::new(),
            buffers: RVec::new(),
        }
    }

    /// Adds exported commands to the package.
    pub fn with_commands(mut self, commands: Vec<PluginCommand>) -> Self {
        self.commands = commands.into();
        self
    }

    /// Adds exported keybindings to the package.
    pub fn with_key_bindings(mut self, key_bindings: Vec<PluginKeyBinding>) -> Self {
        self.key_bindings = key_bindings.into();
        self
    }

    /// Adds custom hook declarations to the package.
    pub fn with_hook_declarations(mut self, hook_declarations: Vec<PluginHookDeclaration>) -> Self {
        self.hook_declarations = hook_declarations.into();
        self
    }

    /// Adds hook subscriptions to the package.
    pub fn with_hook_bindings(mut self, hook_bindings: Vec<PluginHookBinding>) -> Self {
        self.hook_bindings = hook_bindings.into();
        self
    }

    /// Adds plugin-owned buffer declarations to the package.
    pub fn with_buffers(mut self, buffers: Vec<PluginBuffer>) -> Self {
        self.buffers = buffers.into();
        self
    }

    /// Returns the package identifier.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns whether the package should be loaded automatically at startup.
    pub const fn auto_load(&self) -> bool {
        self.auto_load
    }

    /// Returns the package summary.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the exported commands.
    pub fn commands(&self) -> &[PluginCommand] {
        self.commands.as_slice()
    }

    /// Returns the exported keybindings.
    pub fn key_bindings(&self) -> &[PluginKeyBinding] {
        self.key_bindings.as_slice()
    }

    /// Returns the custom hook declarations.
    pub fn hook_declarations(&self) -> &[PluginHookDeclaration] {
        self.hook_declarations.as_slice()
    }

    /// Returns the hook subscriptions.
    pub fn hook_bindings(&self) -> &[PluginHookBinding] {
        self.hook_bindings.as_slice()
    }

    /// Returns the plugin-owned buffer declarations.
    pub fn buffers(&self) -> &[PluginBuffer] {
        self.buffers.as_slice()
    }

    /// Returns the declared plugin buffer for the given kind, if any.
    pub fn buffer(&self, kind: &str) -> Option<&PluginBuffer> {
        self.buffers.iter().find(|buffer| buffer.kind() == kind)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ModelineAlignment, ModelinePart, ModelineSegment, OilKeyAction, PickerExtraKeybindSpec,
        PickerProviderSpec, PickerSource, PluginAction, PluginBuffer, PluginBufferSection,
        PluginBufferSectionUpdate, PluginBufferSections, PluginCommand, PluginHookBinding,
        PluginHookDeclaration, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
        StatuslineSpan, VimActionContext, VimActionSpec, VimEditAction, decode_modeline,
        decode_statusline_spans, encode_modeline, encode_statusline_spans,
    };

    #[test]
    fn picker_provider_spec_accepts_extra_keybinds() {
        let provider = PickerProviderSpec::new(
            "workspace.dashboard",
            "Worktrees",
            PickerSource::WorkspaceDashboard,
        )
        .with_extra_keybind("Ctrl+d", "workspace.worktree-remove")
        .with_extra_keybind("Ctrl+q", "quickfix.open");

        assert_eq!(provider.extra_keybinds().len(), 2);
        assert_eq!(provider.extra_keybinds()[0].chord(), "Ctrl+d");
        assert_eq!(
            provider.extra_keybinds()[0].command_name(),
            "workspace.worktree-remove"
        );
        assert_eq!(provider.extra_keybinds()[1].chord(), "Ctrl+q");
        assert_eq!(provider.extra_keybinds()[1].command_name(), "quickfix.open");

        let replaced = provider.with_extra_keybinds(vec![PickerExtraKeybindSpec::new(
            "Ctrl+x",
            "scratch.command",
        )]);
        assert_eq!(replaced.extra_keybinds().len(), 1);
        assert_eq!(replaced.extra_keybinds()[0].chord(), "Ctrl+x");
    }

    #[test]
    fn statusline_span_encoding_round_trips_themed_and_plain_runs() {
        let spans = vec![
            StatuslineSpan::themed("NORMAL", "ui.statusline.mode"),
            StatuslineSpan::plain(" │ "),
            StatuslineSpan::themed("main.rs", "ui.statusline.foreground"),
        ];
        let decoded = decode_statusline_spans(&encode_statusline_spans(&spans));
        assert_eq!(decoded, spans);
        assert_eq!(
            decode_statusline_spans("plain statusline"),
            vec![StatuslineSpan::plain("plain statusline")]
        );
    }

    #[test]
    fn modeline_encoding_round_trips_alignment_parts_and_backgrounds() {
        let segments = vec![
            ModelineSegment::left(vec![ModelinePart::new(
                " NORMAL ",
                "ui.modeline.mode.normal.foreground",
                Some("ui.modeline.mode.normal.background".into()),
            )]),
            ModelineSegment::left(vec![
                ModelinePart::fg("main", "ui.modeline.git.branch"),
                ModelinePart::fg("+12", "ui.modeline.git.added"),
                ModelinePart::fg("-3", "ui.modeline.git.removed"),
            ]),
            ModelineSegment::right(vec![ModelinePart::fg("Ln 1, Col 1", "ui.modeline.muted")]),
        ];
        let encoded = encode_modeline(&segments);
        assert!(encoded.starts_with('\u{0002}'));
        assert_eq!(decode_modeline(&encoded), segments);
        assert_eq!(
            decode_modeline(&encoded)
                .into_iter()
                .filter(|segment| segment.alignment == ModelineAlignment::Right)
                .count(),
            1
        );
        let spans = decode_statusline_spans(&encoded);
        assert!(spans.iter().any(|span| span.text == " NORMAL "));
        assert!(spans.iter().any(|span| {
            span.token.as_deref() == Some("ui.modeline.git.added") && span.text == "+12"
        }));
    }

    #[test]
    fn plugin_package_constructor_preserves_metadata_and_registrations() {
        let package = PluginPackage::new("lsp", true, "Language server integration.")
            .with_commands(vec![PluginCommand::new(
                "lsp.start",
                "Starts the language server.",
                vec![PluginAction::emit_hook("lsp.startup", Some("rust"))],
            )])
            .with_key_bindings(vec![
                PluginKeyBinding::new("Alt+x lsp.start", "lsp.start", PluginKeymapScope::Global)
                    .with_vim_mode(PluginVimMode::Normal),
            ])
            .with_hook_declarations(vec![PluginHookDeclaration::new(
                "lsp.startup",
                "Runs after an LSP startup command executes.",
            )])
            .with_buffers(vec![
                PluginBuffer::new("calculator", vec!["1 + 1"])
                    .with_sections(PluginBufferSections::new(vec![
                        PluginBufferSection::new("Input")
                            .with_writable(true)
                            .with_initial_lines(vec!["1 + 1"]),
                        PluginBufferSection::new("Output")
                            .with_min_lines(1)
                            .with_initial_lines(vec!["(press enter)".to_owned()])
                            .with_update(PluginBufferSectionUpdate::Replace),
                    ]))
                    .with_evaluate_handler("calculator.evaluate")
                    .with_evaluate_target_section("Output")
                    .with_key_bindings(vec![PluginKeyBinding::new(
                        "Ctrl+Enter",
                        "lsp.start",
                        PluginKeymapScope::Workspace,
                    )]),
            ])
            .with_hook_bindings(vec![PluginHookBinding::new(
                "buffer.file-open",
                "lsp.auto-start",
                "lsp.start",
                Some(".rs"),
            )]);

        assert_eq!(package.name(), "lsp");
        assert!(package.auto_load());
        assert_eq!(package.description(), "Language server integration.");
        assert_eq!(package.commands()[0].name(), "lsp.start");
        assert_eq!(package.key_bindings()[0].chord(), "Alt+x lsp.start");
        assert_eq!(
            package.key_bindings()[0]
                .command_names()
                .iter()
                .map(|name| name.as_str())
                .collect::<Vec<_>>(),
            vec!["lsp.start"]
        );
        assert_eq!(package.key_bindings()[0].vim_mode(), PluginVimMode::Normal);
        assert_eq!(package.hook_declarations()[0].name(), "lsp.startup");
        assert_eq!(package.hook_bindings()[0].detail_filter(), Some(".rs"));
        assert_eq!(package.buffers()[0].kind(), "calculator");
        assert_eq!(
            package.buffers()[0]
                .sections()
                .expect("sections should be present")
                .items()[0]
                .initial_lines()
                .iter()
                .map(|line| line.as_str())
                .collect::<Vec<_>>(),
            vec!["1 + 1"]
        );
        assert_eq!(
            package.buffers()[0]
                .sections()
                .expect("sections should be present")
                .items()
                .iter()
                .map(|section| section.name())
                .collect::<Vec<_>>(),
            vec!["Input", "Output"]
        );
        assert_eq!(
            package.buffers()[0].evaluate_handler(),
            Some("calculator.evaluate")
        );
        assert_eq!(
            package.buffers()[0].evaluate_target_section(),
            Some("Output")
        );
        assert_eq!(package.buffers()[0].key_bindings()[0].chord(), "Ctrl+Enter");
        assert!(package.buffers()[0].line_wrap());
        assert!(
            !PluginBuffer::new("table", vec!["wide"])
                .with_line_wrap(false)
                .line_wrap()
        );
    }

    #[test]
    fn plugin_key_binding_can_target_multiple_commands() {
        let binding = PluginKeyBinding::new_many(
            "Ctrl+d",
            ["vim.scroll-half-page-down", "vim.center-current-line"],
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal);

        assert_eq!(binding.chord(), "Ctrl+d");
        assert_eq!(binding.command_name(), "vim.scroll-half-page-down");
        assert_eq!(
            binding
                .command_names()
                .iter()
                .map(|name| name.as_str())
                .collect::<Vec<_>>(),
            vec!["vim.scroll-half-page-down", "vim.center-current-line"]
        );
        assert_eq!(binding.vim_mode(), PluginVimMode::Normal);
    }

    #[test]
    fn oil_key_action_hook_details_round_trip() {
        for action in [
            OilKeyAction::OpenEntry,
            OilKeyAction::OpenVerticalSplit,
            OilKeyAction::OpenHorizontalSplit,
            OilKeyAction::OpenNewPane,
            OilKeyAction::PreviewEntry,
            OilKeyAction::Refresh,
            OilKeyAction::Close,
            OilKeyAction::OpenParent,
            OilKeyAction::OpenWorkspaceRoot,
            OilKeyAction::SetRoot,
            OilKeyAction::ShowHelp,
            OilKeyAction::CycleSort,
            OilKeyAction::ToggleHidden,
            OilKeyAction::ToggleTrash,
            OilKeyAction::OpenExternal,
            OilKeyAction::SetTabLocalRoot,
            OilKeyAction::CreateGitWorktree,
        ] {
            let detail = action
                .hook_detail()
                .expect("action should expose legacy hook detail");
            assert_eq!(OilKeyAction::from_hook_detail(detail), Some(action));
        }
        assert_eq!(OilKeyAction::StartPrefix.hook_detail(), None);
        assert_eq!(OilKeyAction::from_hook_detail("__unknown__"), None);
    }

    #[test]
    fn vim_edit_action_hook_details_round_trip() {
        for action in [
            VimEditAction::EnterVisual,
            VimEditAction::EnterVisualLine,
            VimEditAction::EnterVisualBlock,
            VimEditAction::DeleteChar,
            VimEditAction::DeleteCharBefore,
            VimEditAction::DeleteLineEnd,
            VimEditAction::ChangeLineEnd,
            VimEditAction::YankLine,
            VimEditAction::SubstituteChar,
            VimEditAction::SubstituteLine,
            VimEditAction::ReplaceChar,
            VimEditAction::EnterReplaceMode,
            VimEditAction::ToggleCase,
            VimEditAction::StartDeleteOperator,
            VimEditAction::StartChangeOperator,
            VimEditAction::StartYankOperator,
            VimEditAction::StartFormatOperator,
            VimEditAction::VisualFormat,
            VimEditAction::ToggleLineComment,
            VimEditAction::VisualToggleComment,
            VimEditAction::Append,
            VimEditAction::AppendLineEnd,
            VimEditAction::InsertLineStart,
            VimEditAction::OpenLineBelow,
            VimEditAction::OpenLineAbove,
            VimEditAction::Undo,
            VimEditAction::Redo,
            VimEditAction::MulticursorAddNextMatch,
            VimEditAction::MulticursorSelectAllMatches,
            VimEditAction::StartGPrefix,
            VimEditAction::StartFindForward,
            VimEditAction::StartFindBackward,
            VimEditAction::StartTillForward,
            VimEditAction::StartTillBackward,
            VimEditAction::RepeatFindNext,
            VimEditAction::RepeatFindPrevious,
            VimEditAction::StartSearchForward,
            VimEditAction::StartSearchBackward,
            VimEditAction::SearchWordForward,
            VimEditAction::SearchWordBackward,
            VimEditAction::RepeatSearchNext,
            VimEditAction::RepeatSearchPrevious,
            VimEditAction::SelectRegister,
            VimEditAction::SetMark,
            VimEditAction::GotoMarkLine,
            VimEditAction::GotoMark,
            VimEditAction::ToggleMacroRecord,
            VimEditAction::StartMacroPlayback,
            VimEditAction::PutAfter,
            VimEditAction::PutBefore,
            VimEditAction::VisualPutAfter,
            VimEditAction::VisualPutBefore,
            VimEditAction::VisualDelete,
            VimEditAction::VisualChange,
            VimEditAction::VisualReplaceChar,
            VimEditAction::VisualBlockInsert,
            VimEditAction::VisualBlockAppend,
            VimEditAction::VisualYank,
            VimEditAction::VisualToggleCase,
            VimEditAction::VisualLowercase,
            VimEditAction::VisualUppercase,
            VimEditAction::VisualIndent,
            VimEditAction::VisualOutdent,
            VimEditAction::VisualJoin,
            VimEditAction::VisualMoveDown,
            VimEditAction::VisualMoveUp,
            VimEditAction::VisualSwapAnchor,
            VimEditAction::StartVisualInnerTextObject,
            VimEditAction::StartVisualAroundTextObject,
        ] {
            assert_eq!(
                VimEditAction::from_hook_detail(action.hook_detail()),
                Some(action)
            );
        }
        assert_eq!(VimEditAction::from_hook_detail("__unknown__"), None);
    }

    #[test]
    fn vim_action_context_carries_typed_spec() {
        let context = VimActionContext::new(VimActionSpec::DeleteChar);
        assert_eq!(context.action(), VimEditAction::DeleteChar);
    }
}
