mod acp;
mod browser;
mod clipboard;
mod command_line;
mod diagnostics;
mod directory;
mod git;
mod pdf;
mod picker;
mod render;
mod terminal;
mod ui_overlays;
mod workspace_search;

use browser::*;
use command_line::*;
use diagnostics::*;
use directory::*;
use git::*;
use pdf::*;
use render::*;
use terminal::*;
use workspace_search::*;

#[cfg(test)]
mod tests;

use std::{
    any::Any,
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env, fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Sender},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use agent_client_protocol::{
    ContentBlock, MaybeUndefined, Plan, PlanEntry, PlanEntryPriority, PlanEntryStatus,
    SessionInfoUpdate, ToolCall, ToolCallContent, ToolCallStatus, ToolCallUpdate, ToolKind,
};
use base64::Engine as _;
use clipboard::*;
use lopdf::{Document as PdfDocument, PdfMetadata};
use notify::{
    Event as NotifyEvent, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    recommended_watcher,
};
use ui_overlays::*;

use crate::browser_host::{
    BrowserBufferPlan, BrowserHostEvent, BrowserHostService, BrowserLocationUpdate,
    BrowserSurfacePlan, BrowserSyncPlan, BrowserViewportRect,
};
use crate::config::{ShellConfig, ShellError, ShellSummary, TypingProfileSummary};
use crate::state::{
    BlockInsertState, BlockSelection, FormatterRegistry, FormatterSpec, InputMode, LastFind,
    LastSearch, MulticursorState, ScrollCommand, ShellMotion, VimBufferState, VimFindKind, VimMark,
    VimOperator, VimPending, VimRecordedInput, VimSearchDirection, VimState, VimTarget,
    VimTextObjectKind, VimVisualSnapshot, VisualSelection, VisualSelectionKind, YankFlash,
    YankRegister,
};
use editor_buffer::{TextBuffer, TextPoint, TextRange, TextSnapshot, WordKind};
use editor_core::{
    Buffer, BufferId, BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope,
    KeymapVimMode, PaneId, SectionAction, SectionCollapseState, SectionRenderLine,
    SectionRenderLineKind, WorkspaceId, builtins,
};
use editor_fs::{DirectoryBuffer, DirectoryEntry, DirectoryEntryKind};
use editor_git::{
    GitLogEntry, GitStatusSnapshot, detect_in_progress, list_repository_files, parse_log_oneline,
    parse_stash_list, parse_status,
};
use editor_jobs::{JobManager, JobSpec};
use editor_lsp::{
    Diagnostic as LspDiagnostic, DiagnosticSeverity as LspDiagnosticSeverity,
    LanguageServerRegistry, LspClientError, LspClientManager, LspCodeAction, LspFormattingOptions,
    LspLocation, LspLogEntry, LspLogSnapshot, LspNotificationLevel, LspNotificationSnapshot,
    LspTextEdit,
};
use editor_picker::{PickerItem, PickerResultOrder, PickerSession};
use editor_plugin_api::{
    GhostTextContext as HostGhostTextContext, LspDiagnosticsInfo as PluginLspDiagnosticsInfo,
    OilDefaults, OilKeyAction, PluginBufferSectionUpdate, PluginBufferSections, autocomplete_hooks,
    browser_hooks, buffer_kinds, git_actions, git_hooks, git_sections, hover_hooks, image_hooks,
    lsp_hooks, oil_hooks, oil_protocol, pdf_hooks, plugin_hooks,
};
use editor_plugin_host::{
    NullUserLibrary, StatuslineContext as HostStatuslineContext, UserLibrary,
    load_auto_loaded_packages,
};
use editor_render::{
    DrawCommand, PixelRect, RenderBackend, RenderColor, centered_rect, find_font_by_name,
    find_system_monospace_font, horizontal_pane_rects, vertical_pane_rects,
};
use editor_syntax::{
    HighlightWindow, LanguageConfiguration, SyntaxError, SyntaxParseSession, SyntaxRegistry,
    SyntaxSnapshot,
};
use editor_terminal::{
    LiveTerminalConfig, LiveTerminalSession, TerminalKey, TerminalRenderSnapshot,
    TerminalViewportScroll,
};
use editor_theme::{Color as ThemeColor, ThemeRegistry};
use fontdue::Font as RasterFont;
use rustybuzz::{
    Face as ShapeFace, Feature as ShapeFeature, UnicodeBuffer, shape, ttf_parser::Tag,
};
use sdl3::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::{MouseButton, MouseWheelDirection},
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{Canvas, FPoint, RenderTarget, ScaleMode, Texture, TextureCreator},
    surface::Surface,
    ttf::Font,
    video::{Window, WindowContext},
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
const HOOK_VIM_COMMAND_LINE: &str = "editor.vim.command-line";
const HOOK_BUFFER_SAVE: &str = "buffer.save";
const HOOK_BUFFER_CLOSE: &str = "buffer.close";
const HOOK_WORKSPACE_SAVE: &str = "workspace.save";
const HOOK_WORKSPACE_FORMAT: &str = "workspace.format";
const HOOK_WORKSPACE_FORMATTER_REGISTER: &str = "workspace.formatter.register";
const HOOK_PICKER_OPEN: &str = "ui.picker.open";
const HOOK_PICKER_NEXT: &str = "ui.picker.next";
const HOOK_PICKER_PREVIOUS: &str = "ui.picker.previous";
const HOOK_PICKER_SUBMIT: &str = "ui.picker.submit";
const HOOK_PICKER_CANCEL: &str = "ui.picker.cancel";
const HOOK_AUTOCOMPLETE_TRIGGER: &str = autocomplete_hooks::TRIGGER;
const HOOK_AUTOCOMPLETE_NEXT: &str = autocomplete_hooks::NEXT;
const HOOK_AUTOCOMPLETE_PREVIOUS: &str = autocomplete_hooks::PREVIOUS;
const HOOK_AUTOCOMPLETE_ACCEPT: &str = autocomplete_hooks::ACCEPT;
const HOOK_AUTOCOMPLETE_CANCEL: &str = autocomplete_hooks::CANCEL;
const HOOK_HOVER_TOGGLE: &str = hover_hooks::TOGGLE;
const HOOK_HOVER_FOCUS: &str = hover_hooks::FOCUS;
const HOOK_HOVER_NEXT: &str = hover_hooks::NEXT;
const HOOK_HOVER_PREVIOUS: &str = hover_hooks::PREVIOUS;
const HOOK_POPUP_TOGGLE: &str = "ui.popup.toggle";
const HOOK_POPUP_NEXT: &str = "ui.popup.next";
const HOOK_POPUP_PREVIOUS: &str = "ui.popup.previous";
const HOOK_ACP_DISCONNECT: &str = "ui.acp.disconnect";
const HOOK_ACP_PERMISSION_APPROVE: &str = "ui.acp.permission-approve";
const HOOK_ACP_PERMISSION_DENY: &str = "ui.acp.permission-deny";
const HOOK_ACP_PICK_SESSION: &str = "ui.acp.pick-session";
const HOOK_ACP_NEW_SESSION: &str = "ui.acp.new-session";
const HOOK_ACP_PICK_MODE: &str = "ui.acp.pick-mode";
const HOOK_ACP_PICK_MODEL: &str = "ui.acp.pick-model";
const HOOK_ACP_CYCLE_MODE: &str = "ui.acp.cycle-mode";
const HOOK_ACP_SWITCH_PANE: &str = "ui.acp.switch-pane";
const HOOK_ACP_COMPLETE_SLASH: &str = "ui.acp.complete-slash";
const HOOK_ACP_FOCUS_INPUT: &str = "ui.acp.focus-input";
const HOOK_PANE_SPLIT_HORIZONTAL: &str = "ui.pane.split-horizontal";
const HOOK_PANE_SPLIT_VERTICAL: &str = "ui.pane.split-vertical";
const HOOK_PANE_CLOSE: &str = "ui.pane.close";
const HOOK_PANE_SWITCH_SPLIT: &str = "ui.pane.switch-split";
const HOOK_WORKSPACE_WINDOW_LEFT: &str = "ui.workspace.window-left";
const HOOK_WORKSPACE_WINDOW_DOWN: &str = "ui.workspace.window-down";
const HOOK_WORKSPACE_WINDOW_UP: &str = "ui.workspace.window-up";
const HOOK_WORKSPACE_WINDOW_RIGHT: &str = "ui.workspace.window-right";
const INTERACTIVE_READONLY_KIND: &str = "interactive-readonly";
const INTERACTIVE_INPUT_KIND: &str = "interactive-input";
const ACP_BUFFER_KIND: &str = buffer_kinds::ACP;
const BROWSER_KIND: &str = buffer_kinds::BROWSER;
const PDF_BUFFER_KIND: &str = buffer_kinds::PDF;
const HOOK_BROWSER_URL: &str = browser_hooks::URL;
const HOOK_BROWSER_FOCUS_INPUT: &str = "ui.browser.focus-input";
const HOOK_IMAGE_ZOOM_IN: &str = image_hooks::ZOOM_IN;
const HOOK_IMAGE_ZOOM_OUT: &str = image_hooks::ZOOM_OUT;
const HOOK_IMAGE_ZOOM_RESET: &str = image_hooks::ZOOM_RESET;
const HOOK_IMAGE_TOGGLE_MODE: &str = image_hooks::TOGGLE_MODE;
const HOOK_PDF_NEXT_PAGE: &str = pdf_hooks::NEXT_PAGE;
const HOOK_PDF_PREVIOUS_PAGE: &str = pdf_hooks::PREVIOUS_PAGE;
const HOOK_PDF_ROTATE_CLOCKWISE: &str = pdf_hooks::ROTATE_CLOCKWISE;
const HOOK_PDF_DELETE_PAGE: &str = pdf_hooks::DELETE_PAGE;
const AUTOCOMPLETE_BUFFER_PROVIDER: &str = "buffer";
const AUTOCOMPLETE_LSP_PROVIDER: &str = "lsp";
const HOVER_PROVIDER_TEST: &str = "test-hover";
const HOVER_PROVIDER_LSP: &str = "lsp";
const HOVER_PROVIDER_SIGNATURE_HELP: &str = "signature-help";
const HOVER_PROVIDER_DIAGNOSTICS: &str = "diagnostics";
const HOOK_LSP_START: &str = lsp_hooks::START;
const HOOK_LSP_STOP: &str = lsp_hooks::STOP;
const HOOK_LSP_RESTART: &str = lsp_hooks::RESTART;
const HOOK_LSP_LOG: &str = lsp_hooks::LOG;
const HOOK_LSP_DEFINITION: &str = lsp_hooks::DEFINITION;
const HOOK_LSP_REFERENCES: &str = lsp_hooks::REFERENCES;
const HOOK_LSP_IMPLEMENTATION: &str = lsp_hooks::IMPLEMENTATION;
const HOOK_LSP_CODE_ACTIONS: &str = lsp_hooks::CODE_ACTIONS;
const ACP_INPUT_PLACEHOLDER: &str =
    "Type @ to mention files, # for issues/PRs, / for commands, or ? for shortcuts";
const GIT_STATUS_KIND: &str = buffer_kinds::GIT_STATUS;
const GIT_COMMIT_KIND: &str = buffer_kinds::GIT_COMMIT;
const GIT_DIFF_KIND: &str = buffer_kinds::GIT_DIFF;
const GIT_LOG_KIND: &str = buffer_kinds::GIT_LOG;
const GIT_STASH_KIND: &str = buffer_kinds::GIT_STASH;
const HOOK_PLUGIN_EVALUATE: &str = plugin_hooks::EVALUATE;
const PLUGIN_EVALUATE_SEPARATOR_PREFIX: &str = plugin_hooks::EVALUATE_SEPARATOR_PREFIX;
const HOOK_PLUGIN_RUN_COMMAND: &str = plugin_hooks::RUN_COMMAND;
const HOOK_PLUGIN_RERUN_COMMAND: &str = plugin_hooks::RERUN_COMMAND;
const HOOK_PLUGIN_SWITCH_PANE: &str = plugin_hooks::SWITCH_PANE;
const HOOK_GIT_STATUS_OPEN_POPUP: &str = git_hooks::STATUS_OPEN_POPUP;
const HOOK_GIT_DIFF_OPEN: &str = git_hooks::DIFF_OPEN;
const HOOK_GIT_LOG_OPEN: &str = git_hooks::LOG_OPEN;
const HOOK_GIT_STASH_LIST_OPEN: &str = git_hooks::STASH_LIST_OPEN;
const HOOK_OIL_OPEN: &str = oil_hooks::OPEN;
const HOOK_OIL_OPEN_PARENT: &str = oil_hooks::OPEN_PARENT;
const GIT_ACTION_STAGE_FILE: &str = git_actions::STAGE_FILE;
const GIT_ACTION_UNSTAGE_FILE: &str = git_actions::UNSTAGE_FILE;
const GIT_ACTION_SHOW_COMMIT: &str = git_actions::SHOW_COMMIT;
const GIT_ACTION_SHOW_STASH: &str = git_actions::SHOW_STASH;
const GIT_SECTION_HEADERS: &str = git_sections::HEADERS;
const GIT_SECTION_IN_PROGRESS: &str = git_sections::IN_PROGRESS;
const GIT_SECTION_STAGED: &str = git_sections::STAGED;
const GIT_SECTION_UNSTAGED: &str = git_sections::UNSTAGED;
const GIT_SECTION_UNTRACKED: &str = git_sections::UNTRACKED;
const GIT_SECTION_STASHES: &str = git_sections::STASHES;
const GIT_SECTION_UNPULLED: &str = git_sections::UNPULLED;
const GIT_SECTION_UNPUSHED: &str = git_sections::UNPUSHED;
const GIT_SECTION_COMMIT: &str = git_sections::COMMIT;
const PDF_ROTATION_FULL_CIRCLE: i64 = 360;
const TOKEN_GIT_STATUS_SECTION_HEADER: &str = "git.status.section.header";
const TOKEN_GIT_STATUS_SECTION_COUNT: &str = "git.status.section.count";
const TOKEN_GIT_STATUS_HEADER_LABEL: &str = "git.status.header.label";
const TOKEN_GIT_STATUS_HEADER_VALUE: &str = "git.status.header.value";
const TOKEN_GIT_STATUS_HEADER_HASH: &str = "git.status.header.hash";
const TOKEN_GIT_STATUS_HEADER_SUMMARY: &str = "git.status.header.summary";
const TOKEN_GIT_STATUS_IN_PROGRESS: &str = "git.status.in-progress";
const TOKEN_GIT_STATUS_ENTRY_ADDED: &str = "git.status.entry.added";
const TOKEN_GIT_STATUS_ENTRY_MODIFIED: &str = "git.status.entry.modified";
const TOKEN_GIT_STATUS_ENTRY_DELETED: &str = "git.status.entry.deleted";
const TOKEN_GIT_STATUS_ENTRY_RENAMED: &str = "git.status.entry.renamed";
const TOKEN_GIT_STATUS_ENTRY_COPIED: &str = "git.status.entry.copied";
const TOKEN_GIT_STATUS_ENTRY_UPDATED: &str = "git.status.entry.updated";
const TOKEN_GIT_STATUS_ENTRY_CHANGED: &str = "git.status.entry.changed";
const TOKEN_GIT_STATUS_ENTRY_UNTRACKED: &str = "git.status.entry.untracked";
const TOKEN_GIT_STATUS_ENTRY_PATH: &str = "git.status.entry.path";
const TOKEN_GIT_STATUS_COMMIT_HASH: &str = "git.status.commit.hash";
const TOKEN_GIT_STATUS_COMMIT_SUMMARY: &str = "git.status.commit.summary";
const TOKEN_GIT_STATUS_STASH_NAME: &str = "git.status.stash.name";
const TOKEN_GIT_STATUS_STASH_SUMMARY: &str = "git.status.stash.summary";
const TOKEN_GIT_STATUS_COMMAND: &str = "git.status.command";
const TOKEN_GIT_STATUS_MESSAGE: &str = "git.status.message";
const TOKEN_STATUSLINE_ACTIVE: &str = "ui.statusline.active";
const TOKEN_STATUSLINE_INACTIVE: &str = "ui.statusline.inactive";
const MOUSE_WHEEL_SCROLL_LINES: i32 = 3;
const OIL_BUFFER_NAME: &str = "*oil*";
const OIL_PREVIEW_BUFFER_NAME: &str = "*oil-preview*";
const OIL_HELP_BUFFER_NAME: &str = "*oil-help*";
const LSP_LOG_BUFFER_PREFIX: &str = "*lsp-log ";
const OIL_PREVIEW_KIND: &str = "oil-preview";
const OIL_HELP_KIND: &str = "oil-help";
const HOOK_INPUT_SUBMIT: &str = "ui.input.submit";
const HOOK_INPUT_CLEAR: &str = "ui.input.clear";
const OPTION_LINE_NUMBER_RELATIVE: &str = "ui.line-number.relative";
const OPTION_FONT: &str = "font";
const OPTION_FONT_SIZE: &str = "font_size";
const OPTION_CURSOR_ROUNDNESS: &str = "cursor_roundness";
const OPTION_PICKER_ROUNDNESS: &str = "picker_roundness";
const OPTION_SCROLL_OFF: &str = "scrolloff";
const SEARCH_PICKER_ITEM_LIMIT: usize = 512;
const GIT_LOG_LIMIT: usize = 10;
const GIT_LOG_VIEW_LIMIT: usize = 200;
const BUNDLED_ICON_FONT_SEARCH_DEPTH: usize = 6;
const BUNDLED_ICON_FONT_DIR_CANDIDATES: &[&[&str]] =
    &[&["crates", "volt", "assets", "font"], &["assets", "font"]];
const BUNDLED_ICON_FONT_FILES: &[&str] = &[
    "NFM.ttf",
    "all-the-icons.ttf",
    "file-icons.ttf",
    "fontawesome.ttf",
    "material-design-icons.ttf",
    "octicons.ttf",
    "weathericons.ttf",
];
#[cfg(target_os = "windows")]
const SYSTEM_ICON_FONT_CANDIDATES: &[&str] = &[r"C:\Windows\Fonts\seguisym.ttf"];
#[cfg(target_os = "macos")]
const SYSTEM_ICON_FONT_CANDIDATES: &[&str] = &[
    "/System/Library/Fonts/Supplemental/Apple Symbols.ttf",
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
];
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
const SYSTEM_ICON_FONT_CANDIDATES: &[&str] = &[
    "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
    "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/opentype/noto/NotoSansSymbols2-Regular.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];
const WINDOW_ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../volt/assets/logo.png"
));
const ERROR_LOG_MAX_ENTRIES: usize = 200;
const ERROR_LOG_FILE_NAME: &str = "errors.log";
const ACTIVE_THEME_STATE_FILE_NAME: &str = "active-theme.txt";
const TYPING_PROFILE_LOG_FILE_NAME: &str = "typing-profile.log";
const TYPING_PROFILE_MAX_FRAMES: usize = 10_000;
const TYPING_PROFILE_SLOW_FRAME_THRESHOLD: Duration = Duration::from_millis(8);
const FRAME_PACING_TARGET_120FPS: Duration = Duration::from_nanos(8_333_333);
const FRAME_PACING_YIELD_THRESHOLD: Duration = Duration::from_millis(1);
const FRAME_PACING_TYPING_IDLE_THRESHOLD: Duration = Duration::from_millis(150);
const TYPING_EVENT_BATCH_LIMIT: usize = 24;
const TYPING_EVENT_BATCH_TIME_BUDGET: Duration = Duration::from_millis(2);
const GIT_SUMMARY_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const GIT_FRINGE_REFRESH_DEBOUNCE: Duration = Duration::from_millis(150);
const GIT_REFRESH_TYPING_IDLE_THRESHOLD: Duration = Duration::from_millis(750);
const SYNTAX_WINDOW_MIN_LINES: usize = 256;
const SYNTAX_WINDOW_MARGIN_LINES: usize = 96;
const NOTIFICATION_AUTO_DISMISS: Duration = Duration::from_secs(5);
const NOTIFICATION_VISIBLE_LIMIT: usize = 3;
const NOTIFICATION_MAX_STORED: usize = 12;
const NOTIFICATION_STACK_GAP: i32 = 10;
const NOTIFICATION_MAX_BODY_LINES: usize = 4;
const IMAGE_ZOOM_STEP: f32 = 1.25;
const IMAGE_ZOOM_MIN: f32 = 0.1;
const IMAGE_ZOOM_MAX: f32 = 8.0;

// ─── Local constants (formerly from user modules) ────────────────────────────
const BROWSER_BUFFER_NAME: &str = "*browser*";
const AUTOCOMPLETE_NEXT_CHORD: &str = "Ctrl+n";
const AUTOCOMPLETE_PREVIOUS_CHORD: &str = "Ctrl+p";
const HOVER_NEXT_CHORD: &str = "Ctrl+n";
const HOVER_PREVIOUS_CHORD: &str = "Ctrl+p";

/// Newtype wrapper so `Arc<dyn UserLibrary>` can be stored in the runtime's
/// type-erased service map.
struct UserLibraryService(Arc<dyn UserLibrary>);

/// Returns a clone of the user library stored in the runtime service map.
fn shell_user_library(runtime: &EditorRuntime) -> Arc<dyn UserLibrary> {
    runtime
        .services()
        .get::<UserLibraryService>()
        .expect("UserLibraryService not registered in runtime")
        .0
        .clone()
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThemeRuntimeSettings {
    font_request: Option<String>,
    font_size: u32,
}

struct IconFont<'ttf> {
    name: String,
    font: Font<'ttf>,
    raster_font: RasterFont,
}

struct FontSetInit<'ttf> {
    primary: Font<'ttf>,
    primary_raster_font: RasterFont,
    primary_shape_face: ShapeFace<'static>,
    primary_pixel_size: f32,
    ligatures_enabled: bool,
    icon_fonts: Vec<(String, Font<'ttf>, RasterFont)>,
    icon_chars: BTreeSet<char>,
    icon_pixel_size: f32,
    cell_width: i32,
}

struct FontSet<'ttf> {
    primary: Font<'ttf>,
    primary_raster_font: RasterFont,
    primary_shape_face: ShapeFace<'static>,
    primary_pixel_size: f32,
    ligatures_enabled: bool,
    icon_fonts: Vec<IconFont<'ttf>>,
    icon_chars: BTreeSet<char>,
    icon_pixel_size: f32,
    cell_width: i32,
}

impl<'ttf> FontSet<'ttf> {
    fn new(init: FontSetInit<'ttf>) -> Self {
        let icon_fonts = init
            .icon_fonts
            .into_iter()
            .map(|(name, font, raster_font)| IconFont {
                name,
                font,
                raster_font,
            })
            .collect();
        Self {
            primary: init.primary,
            primary_raster_font: init.primary_raster_font,
            primary_shape_face: init.primary_shape_face,
            primary_pixel_size: init.primary_pixel_size,
            ligatures_enabled: init.ligatures_enabled,
            icon_fonts,
            icon_chars: init.icon_chars,
            icon_pixel_size: init.icon_pixel_size,
            cell_width: init.cell_width.max(1),
        }
    }

    fn primary(&self) -> &Font<'ttf> {
        &self.primary
    }

    fn primary_raster_font(&self) -> &RasterFont {
        &self.primary_raster_font
    }

    fn primary_shape_face(&self) -> &ShapeFace<'static> {
        &self.primary_shape_face
    }

    fn primary_pixel_size(&self) -> f32 {
        self.primary_pixel_size
    }

    fn ligatures_enabled(&self) -> bool {
        self.ligatures_enabled
    }

    fn icon_font(&self, index: usize) -> Option<&IconFont<'ttf>> {
        self.icon_fonts.get(index)
    }

    fn icon_font_index_for_char(&self, character: char) -> Option<usize> {
        self.icon_fonts
            .iter()
            .position(|font| font.font.find_glyph(character).is_some())
    }

    fn icon_fonts(&self) -> &[IconFont<'ttf>] {
        &self.icon_fonts
    }

    fn icon_pixel_size(&self) -> f32 {
        self.icon_pixel_size
    }

    fn cell_width(&self) -> i32 {
        self.cell_width
    }

    fn prefers_icon_font(&self, character: char) -> bool {
        self.icon_chars.contains(&character)
    }
}

#[derive(Debug, Clone)]
struct LineSyntaxSpan {
    start: usize,
    end: usize,
    capture_name: String,
    theme_token: String,
}

type IndexedSyntaxLines = BTreeMap<usize, Vec<LineSyntaxSpan>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SyntaxLineWindow {
    start_line: usize,
    line_count: usize,
}

impl SyntaxLineWindow {
    fn new(start_line: usize, line_count: usize) -> Option<Self> {
        (line_count > 0).then_some(Self {
            start_line,
            line_count,
        })
    }

    fn end_line_exclusive(self) -> usize {
        self.start_line.saturating_add(self.line_count)
    }

    fn contains(self, other: Self) -> bool {
        self.start_line <= other.start_line
            && self.end_line_exclusive() >= other.end_line_exclusive()
    }

    fn to_highlight_window(self) -> HighlightWindow {
        HighlightWindow::new(self.start_line, self.line_count)
    }
}

#[derive(Debug, Clone, Copy)]
struct LineWrapSegment {
    start_col: usize,
    end_col: usize,
}

#[derive(Debug, Clone)]
struct LineCharMap {
    bytes: Vec<usize>,
    whitespace: Vec<bool>,
}

impl LineCharMap {
    fn new(line: &str) -> Self {
        let mut bytes = Vec::new();
        let mut whitespace = Vec::new();
        for (byte_index, character) in line.char_indices() {
            bytes.push(byte_index);
            whitespace.push(character.is_whitespace());
        }
        bytes.push(line.len());
        Self { bytes, whitespace }
    }

    fn len(&self) -> usize {
        self.whitespace.len()
    }

    fn slice<'a>(&self, line: &'a str, start_col: usize, end_col: usize) -> &'a str {
        if start_col >= end_col {
            return "";
        }
        let len = self.len();
        let start = start_col.min(len);
        let end = end_col.min(len);
        let start_byte = self.bytes.get(start).copied().unwrap_or(line.len());
        let end_byte = self.bytes.get(end).copied().unwrap_or(line.len());
        &line[start_byte..end_byte]
    }
}

fn theme_lang_indent(theme_registry: Option<&ThemeRegistry>, language_id: Option<&str>) -> usize {
    let Some(registry) = theme_registry else {
        return 0;
    };
    let Some(language_id) = language_id else {
        return 0;
    };
    let key = format!("langs.{language_id}.indent");
    registry
        .resolve_number(&key)
        .map(|value| value.max(0.0).round() as usize)
        .unwrap_or(0)
}

fn theme_lang_format_on_save(
    theme_registry: Option<&ThemeRegistry>,
    language_id: Option<&str>,
) -> bool {
    let Some(registry) = theme_registry else {
        return false;
    };
    let Some(language_id) = language_id else {
        return false;
    };
    let key = format!("langs.{language_id}.format_on_save");
    registry.resolve_bool(&key).unwrap_or(false)
}

fn theme_lang_use_tabs(theme_registry: Option<&ThemeRegistry>, language_id: Option<&str>) -> bool {
    let Some(registry) = theme_registry else {
        return false;
    };
    let Some(language_id) = language_id else {
        return false;
    };
    let key = format!("langs.{language_id}.use_tabs");
    registry.resolve_bool(&key).unwrap_or(false)
}

fn theme_scrolloff(theme_registry: Option<&ThemeRegistry>) -> usize {
    theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_SCROLL_OFF))
        .map(|value| value.max(0.0).round() as usize)
        .unwrap_or(0)
}

fn buffer_context_overlay_snapshot(
    buffer: &ShellBuffer,
    active: bool,
    user_library: &dyn UserLibrary,
    theme_registry: Option<&ThemeRegistry>,
) -> Option<BufferContextOverlaySnapshot> {
    active.then(|| buffer.context_overlay_snapshot(user_library, theme_scrolloff(theme_registry)))
}

fn buffer_headerline_rows(
    buffer: &ShellBuffer,
    active: bool,
    user_library: &dyn UserLibrary,
    theme_registry: Option<&ThemeRegistry>,
    visible_rows: usize,
) -> usize {
    buffer_context_overlay_snapshot(buffer, active, user_library, theme_registry)
        .map(|snapshot| count_visible_headerline_lines(&snapshot.headerline_lines, visible_rows))
        .unwrap_or(0)
}

fn lsp_formatting_options(
    runtime: &EditorRuntime,
    language_id: Option<&str>,
) -> LspFormattingOptions {
    let theme_registry = runtime.services().get::<ThemeRegistry>();
    let indent_size = theme_lang_indent(theme_registry, language_id);
    let tab_size = if indent_size == 0 { 4 } else { indent_size } as u32;
    let insert_spaces = !theme_lang_use_tabs(theme_registry, language_id);
    LspFormattingOptions::new(tab_size, insert_spaces)
}

fn theme_color(theme_registry: Option<&ThemeRegistry>, token: &str, fallback: Color) -> Color {
    theme_registry
        .and_then(|registry| registry.resolve(token))
        .map(to_sdl_color)
        .unwrap_or(fallback)
}

fn is_dark_color(color: Color) -> bool {
    let luminance =
        0.2126 * f32::from(color.r) + 0.7152 * f32::from(color.g) + 0.0722 * f32::from(color.b);
    luminance < 128.0
}

fn adjust_color(color: Color, delta: i16) -> Color {
    let adjust = |channel: u8| -> u8 { (i16::from(channel) + delta).clamp(0, 255) as u8 };
    Color::RGBA(adjust(color.r), adjust(color.g), adjust(color.b), color.a)
}

fn blend_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let blend = |a: u8, b: u8| -> u8 {
        (f32::from(a) * inv + f32::from(b) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color::RGBA(
        blend(a.r, b.r),
        blend(a.g, b.g),
        blend(a.b, b.b),
        blend(a.a, b.a),
    )
}

fn normalize_tabs<'a>(text: &'a str, indent_size: usize, use_tabs: bool) -> Cow<'a, str> {
    if use_tabs || !text.contains('\t') {
        return Cow::Borrowed(text);
    }
    let indent_size = indent_size.max(1);
    Cow::Owned(text.replace('\t', &" ".repeat(indent_size)))
}

fn leading_whitespace_info(line: &str, tab_width: usize) -> (usize, usize) {
    let tab_width = tab_width.max(1);
    let mut columns = 0usize;
    let mut end = 0usize;
    for (index, character) in line.char_indices() {
        if !character.is_whitespace() {
            end = index;
            return (columns, end);
        }
        if character == '\t' {
            columns = columns.saturating_add(tab_width);
        } else {
            columns = columns.saturating_add(1);
        }
        end = index + character.len_utf8();
    }
    (columns, end)
}

fn leading_indent_string(line: &str, indent_size: usize) -> String {
    let (_, end) = leading_whitespace_info(line, indent_size);
    line[..end].to_owned()
}

fn trim_indent_unit(indent: &str, indent_size: usize) -> String {
    if indent.is_empty() || indent_size == 0 {
        return indent.to_owned();
    }
    let tab_width = indent_size.max(1);
    let mut total_cols = 0usize;
    for character in indent.chars() {
        total_cols = total_cols.saturating_add(if character == '\t' { tab_width } else { 1 });
    }
    let target_cols = total_cols.saturating_sub(indent_size);
    let mut cols = 0usize;
    let mut cut = 0usize;
    for (index, character) in indent.char_indices() {
        let width = if character == '\t' { tab_width } else { 1 };
        if cols.saturating_add(width) > target_cols {
            break;
        }
        cols = cols.saturating_add(width);
        cut = index + character.len_utf8();
    }
    indent[..cut].to_owned()
}

fn desired_indent_for_line(
    buffer: &ShellBuffer,
    line_index: usize,
    indent_size: usize,
    use_tabs: bool,
) -> String {
    let mut base_line = buffer.text.line(line_index).unwrap_or_default();
    let mut search_index = line_index;
    while search_index > 0 && base_line.trim().is_empty() {
        search_index = search_index.saturating_sub(1);
        let Some(line) = buffer.text.line(search_index) else {
            continue;
        };
        if !line.trim().is_empty() {
            base_line = line;
            break;
        }
    }
    let mut indent = leading_indent_string(&base_line, indent_size);
    if indent_size > 0 && base_line.trim_end().ends_with('{') {
        if use_tabs {
            indent.push('\t');
        } else {
            indent.push_str(&" ".repeat(indent_size));
        }
    }
    let current_line = buffer.text.line(line_index).unwrap_or_default();
    if current_line.trim_start().starts_with('}') {
        indent = trim_indent_unit(&indent, indent_size);
    }
    indent
}

fn apply_line_indent(
    buffer: &mut ShellBuffer,
    line_index: usize,
    indent_size: usize,
    indent: &str,
) {
    let line = buffer.text.line(line_index).unwrap_or_default();
    let (_, end) = leading_whitespace_info(&line, indent_size);
    let current_indent = &line[..end];
    if current_indent == indent {
        return;
    }
    let end_col = current_indent.chars().count();
    buffer.replace_range(
        TextRange::new(
            TextPoint::new(line_index, 0),
            TextPoint::new(line_index, end_col),
        ),
        indent,
    );
    let cursor = buffer.cursor_point();
    if cursor.line == line_index {
        let new_indent_cols = indent.chars().count();
        let delta = new_indent_cols as isize - end_col as isize;
        let new_col = if cursor.column <= end_col {
            new_indent_cols
        } else {
            let adjusted = cursor.column as isize + delta;
            if adjusted < new_indent_cols as isize {
                new_indent_cols
            } else {
                adjusted as usize
            }
        };
        buffer.set_cursor(TextPoint::new(line_index, new_col));
    }
}

fn format_current_line_indent(buffer: &mut ShellBuffer, indent_size: usize, use_tabs: bool) {
    let line_index = buffer.cursor_row();
    let indent = desired_indent_for_line(buffer, line_index, indent_size, use_tabs);
    apply_line_indent(buffer, line_index, indent_size, &indent);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkdownTableAlignment {
    None,
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownTableLine {
    prefix: String,
    cells: Vec<String>,
    is_delimiter: bool,
    alignments: Vec<MarkdownTableAlignment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownTable {
    start_line: usize,
    rows: Vec<MarkdownTableLine>,
    column_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MarkdownTableCursorTarget {
    row_index: usize,
    cell_index: usize,
    content_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownTableRender {
    lines: Vec<String>,
    widths: Vec<usize>,
}

fn detect_markdown_table(buffer: &ShellBuffer) -> Option<MarkdownTable> {
    if buffer.language_id() != Some("markdown") || buffer.line_count() < 2 {
        return None;
    }

    let cursor_line = buffer
        .cursor_row()
        .min(buffer.line_count().saturating_sub(1));
    parse_markdown_table_line(&buffer.text.line(cursor_line)?)
        .as_ref()
        .map(|_| ())?;

    let mut start_line = cursor_line;
    while start_line > 0 {
        let previous = start_line.saturating_sub(1);
        let Some(line) = buffer.text.line(previous) else {
            break;
        };
        if parse_markdown_table_line(&line).is_none() {
            break;
        }
        start_line = previous;
    }

    let mut end_line = cursor_line;
    while end_line + 1 < buffer.line_count() {
        let next = end_line + 1;
        let Some(line) = buffer.text.line(next) else {
            break;
        };
        if parse_markdown_table_line(&line).is_none() {
            break;
        }
        end_line = next;
    }

    let mut rows = Vec::new();
    let mut column_count = 0usize;
    for line_index in start_line..=end_line {
        let (prefix, cells) = parse_markdown_table_line(&buffer.text.line(line_index)?)?;
        column_count = column_count.max(cells.len());
        rows.push(MarkdownTableLine {
            prefix,
            cells,
            is_delimiter: false,
            alignments: Vec::new(),
        });
    }

    if rows.len() < 2 || column_count < 2 {
        return None;
    }

    let delimiter_cells = rows[1].cells.clone();
    if !is_markdown_table_delimiter_row_candidate(&delimiter_cells) {
        return None;
    }

    rows[1].is_delimiter = true;
    rows[1].alignments = (0..column_count)
        .map(|index| markdown_table_alignment(delimiter_cells.get(index).map(String::as_str)))
        .collect();

    Some(MarkdownTable {
        start_line,
        rows,
        column_count,
    })
}

fn parse_markdown_table_line(line: &str) -> Option<(String, Vec<String>)> {
    let trimmed_end = line.trim_end();
    let body = trimmed_end.trim_start();
    let prefix = trimmed_end[..trimmed_end.len().saturating_sub(body.len())].to_owned();
    if !body.starts_with('|') || !body.ends_with('|') || body.chars().count() < 3 {
        return None;
    }
    let inner = body.get(1..body.len().saturating_sub(1))?;
    let cells = inner.split('|').map(str::to_owned).collect::<Vec<_>>();
    (!cells.is_empty()).then_some((prefix, cells))
}

fn markdown_table_alignment(cell: Option<&str>) -> MarkdownTableAlignment {
    let trimmed = cell.unwrap_or_default().trim();
    if trimmed.starts_with(':') && trimmed.ends_with(':') && trimmed.chars().count() >= 2 {
        MarkdownTableAlignment::Center
    } else if trimmed.starts_with(':') {
        MarkdownTableAlignment::Left
    } else if trimmed.ends_with(':') {
        MarkdownTableAlignment::Right
    } else {
        MarkdownTableAlignment::None
    }
}

fn is_markdown_table_delimiter_row_candidate(cells: &[String]) -> bool {
    let mut has_bootstrap_cell = false;
    let mut has_any_content = false;
    for cell in cells {
        let trimmed = cell.trim();
        if trimmed.is_empty() {
            continue;
        }
        has_any_content = true;
        let without_prefix = trimmed.strip_prefix(':').unwrap_or(trimmed);
        let stripped = without_prefix.strip_suffix(':').unwrap_or(without_prefix);
        if stripped.is_empty() || !stripped.chars().all(|character| character == '-') {
            return false;
        }
        if stripped.chars().count() >= 2 {
            has_bootstrap_cell = true;
        }
    }
    has_any_content && has_bootstrap_cell
}

fn render_markdown_table(table: &MarkdownTable) -> MarkdownTableRender {
    let mut widths = vec![1usize; table.column_count];
    for row in table.rows.iter().filter(|row| !row.is_delimiter) {
        for (index, width) in widths.iter_mut().enumerate() {
            let content_width = row
                .cells
                .get(index)
                .map(|cell| cell.trim().chars().count())
                .unwrap_or(0)
                .max(1);
            *width = (*width).max(content_width);
        }
    }

    let lines = table
        .rows
        .iter()
        .map(|row| {
            let mut line = row.prefix.clone();
            for (column_index, base_width) in widths.iter().copied().enumerate() {
                line.push('|');
                line.push(' ');
                let width = if row.is_delimiter {
                    base_width.max(3)
                } else {
                    base_width
                };
                if row.is_delimiter {
                    line.push_str(&render_markdown_table_delimiter(
                        width,
                        row.alignments
                            .get(column_index)
                            .copied()
                            .unwrap_or(MarkdownTableAlignment::None),
                    ));
                } else {
                    let content = row
                        .cells
                        .get(column_index)
                        .map(|cell| cell.trim())
                        .unwrap_or_default();
                    line.push_str(content);
                    line.push_str(&" ".repeat(width.saturating_sub(content.chars().count())));
                }
                line.push(' ');
            }
            line.push('|');
            line
        })
        .collect();

    MarkdownTableRender { lines, widths }
}

fn render_markdown_table_delimiter(width: usize, alignment: MarkdownTableAlignment) -> String {
    let width = width.max(3);
    match alignment {
        MarkdownTableAlignment::None => "-".repeat(width),
        MarkdownTableAlignment::Left => format!(":{}", "-".repeat(width.saturating_sub(1))),
        MarkdownTableAlignment::Right => format!("{}:", "-".repeat(width.saturating_sub(1))),
        MarkdownTableAlignment::Center => {
            format!(":{}:", "-".repeat(width.saturating_sub(2)))
        }
    }
}

fn markdown_table_cursor_target(
    buffer: &ShellBuffer,
    table: &MarkdownTable,
    point: TextPoint,
) -> Option<MarkdownTableCursorTarget> {
    if point.line < table.start_line || point.line >= table.start_line + table.rows.len() {
        return None;
    }
    let row_index = point.line.saturating_sub(table.start_line);
    let line = buffer.text.line(point.line)?;
    let (prefix, cells) = parse_markdown_table_line(&line)?;
    let prefix_width = prefix.chars().count();
    let mut column = prefix_width;

    for cell_index in 0..table.column_count {
        column = column.saturating_add(1);
        let cell = cells
            .get(cell_index)
            .map(String::as_str)
            .unwrap_or_default();
        let raw_len = cell.chars().count();
        let cell_start = column;
        let cell_end = cell_start + raw_len;
        let (editable_start, editable_len) =
            markdown_table_editable_cell_bounds(cell, cell_start, cell_end);

        if point.column <= cell_start {
            return Some(MarkdownTableCursorTarget {
                row_index,
                cell_index,
                content_offset: 0,
            });
        }
        if point.column <= cell_end {
            return Some(MarkdownTableCursorTarget {
                row_index,
                cell_index,
                content_offset: point
                    .column
                    .saturating_sub(editable_start)
                    .min(editable_len),
            });
        }

        column = cell_end;
        if point.column <= column.saturating_add(1) {
            return Some(MarkdownTableCursorTarget {
                row_index,
                cell_index: (cell_index + 1).min(table.column_count.saturating_sub(1)),
                content_offset: 0,
            });
        }
    }

    Some(MarkdownTableCursorTarget {
        row_index,
        cell_index: table.column_count.saturating_sub(1),
        content_offset: 0,
    })
}

fn markdown_table_editable_cell_bounds(
    cell: &str,
    cell_start: usize,
    cell_end: usize,
) -> (usize, usize) {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        let start = if cell_start < cell_end {
            cell_start.saturating_add(1).min(cell_end)
        } else {
            cell_start
        };
        return (start, 0);
    }
    let leading = cell
        .chars()
        .take_while(|character| character.is_whitespace())
        .count();
    (cell_start + leading, trimmed.chars().count())
}

fn markdown_table_point_for_target(
    table: &MarkdownTable,
    render: &MarkdownTableRender,
    target: MarkdownTableCursorTarget,
) -> TextPoint {
    let row_index = target.row_index.min(table.rows.len().saturating_sub(1));
    let cell_index = target.cell_index.min(table.column_count.saturating_sub(1));
    let row = &table.rows[row_index];
    let mut column = row.prefix.chars().count();
    for current_cell in 0..table.column_count {
        column = column.saturating_add(1);
        let editable_start = column.saturating_add(1);
        let editable_len = if row.is_delimiter {
            render.widths[current_cell].max(3)
        } else {
            row.cells
                .get(current_cell)
                .map(|cell| cell.trim().chars().count())
                .unwrap_or(0)
        };
        if current_cell == cell_index {
            return TextPoint::new(
                table.start_line + row_index,
                editable_start + target.content_offset.min(editable_len),
            );
        }
        let display_width = if row.is_delimiter {
            render.widths[current_cell].max(3)
        } else {
            render.widths[current_cell]
        };
        column = editable_start + display_width + 1;
    }
    TextPoint::new(table.start_line + row_index, row.prefix.chars().count())
}

fn apply_markdown_table_update(
    buffer: &mut ShellBuffer,
    original: &MarkdownTable,
    updated: &MarkdownTable,
    target: MarkdownTableCursorTarget,
) -> bool {
    let Some(range) = buffer.line_span_range(original.start_line, original.rows.len()) else {
        return false;
    };
    let original_text = buffer.slice(range);
    let render = render_markdown_table(updated);
    let mut replacement = render.lines.join("\n");
    if original_text.ends_with('\n') {
        replacement.push('\n');
    }
    let changed = original_text != replacement;
    if changed {
        buffer.replace_range(range, &replacement);
    }
    buffer.set_cursor(markdown_table_point_for_target(updated, &render, target));
    changed
}

fn format_markdown_table_at_cursor(buffer: &mut ShellBuffer) -> Option<bool> {
    let table = detect_markdown_table(buffer)?;
    let target = markdown_table_cursor_target(buffer, &table, buffer.cursor_point())?;
    Some(apply_markdown_table_update(buffer, &table, &table, target))
}

fn should_defer_table_format(buffer: &ShellBuffer, text: &str) -> bool {
    if text.is_empty() || !text.chars().all(|character| character == ' ') {
        return false;
    }
    let Some(table) = detect_markdown_table(buffer) else {
        return false;
    };
    markdown_table_cursor_target(buffer, &table, buffer.cursor_point())
        .and_then(|target| table.rows.get(target.row_index))
        .is_some_and(|row| !row.is_delimiter)
}

fn insert_markdown_table_row_at_cursor(buffer: &mut ShellBuffer) -> Option<bool> {
    let table = detect_markdown_table(buffer)?;
    let current = markdown_table_cursor_target(buffer, &table, buffer.cursor_point())?;
    let insert_after = if current.row_index <= 1 {
        1
    } else {
        current.row_index
    };
    let mut updated = table.clone();
    updated.rows.insert(
        insert_after + 1,
        MarkdownTableLine {
            prefix: updated
                .rows
                .get(insert_after)
                .map(|row| row.prefix.clone())
                .unwrap_or_default(),
            cells: vec![String::new(); updated.column_count],
            is_delimiter: false,
            alignments: Vec::new(),
        },
    );
    Some(apply_markdown_table_update(
        buffer,
        &table,
        &updated,
        MarkdownTableCursorTarget {
            row_index: insert_after + 1,
            cell_index: 0,
            content_offset: 0,
        },
    ))
}

fn advance_markdown_table_insert_tab(buffer: &mut ShellBuffer) -> Option<bool> {
    let table = detect_markdown_table(buffer)?;
    let current = markdown_table_cursor_target(buffer, &table, buffer.cursor_point())?;
    let mut updated = table.clone();
    let target = if current.cell_index + 1 < updated.column_count {
        MarkdownTableCursorTarget {
            row_index: current.row_index,
            cell_index: current.cell_index + 1,
            content_offset: 0,
        }
    } else {
        updated.column_count = updated.column_count.saturating_add(1);
        for row in &mut updated.rows {
            row.cells.resize(updated.column_count, String::new());
            if row.is_delimiter {
                row.alignments
                    .resize(updated.column_count, MarkdownTableAlignment::None);
            }
        }
        MarkdownTableCursorTarget {
            row_index: current.row_index,
            cell_index: updated.column_count.saturating_sub(1),
            content_offset: 0,
        }
    };
    Some(apply_markdown_table_update(
        buffer, &table, &updated, target,
    ))
}

fn advance_markdown_table_normal_tab(buffer: &mut ShellBuffer) -> Option<bool> {
    let table = detect_markdown_table(buffer)?;
    let current = markdown_table_cursor_target(buffer, &table, buffer.cursor_point())?;
    let mut targets = Vec::new();
    for row_index in 0..table.rows.len() {
        if table.rows[row_index].is_delimiter {
            continue;
        }
        for cell_index in 0..table.column_count {
            targets.push((row_index, cell_index));
        }
    }
    let target = if let Some(position) = targets.iter().position(|&(row_index, cell_index)| {
        row_index == current.row_index && cell_index == current.cell_index
    }) {
        let (row_index, cell_index) = targets[(position + 1) % targets.len()];
        MarkdownTableCursorTarget {
            row_index,
            cell_index,
            content_offset: 0,
        }
    } else {
        let (row_index, cell_index) = targets
            .into_iter()
            .find(|&(row_index, cell_index)| {
                (row_index, cell_index) > (current.row_index, current.cell_index)
            })
            .unwrap_or((0, 0));
        MarkdownTableCursorTarget {
            row_index,
            cell_index,
            content_offset: 0,
        }
    };
    Some(apply_markdown_table_update(buffer, &table, &table, target))
}

fn dedent_block_end(buffer: &mut ShellBuffer, indent_size: usize) -> bool {
    if indent_size == 0 {
        return false;
    }
    let cursor = buffer.cursor_point();
    let line = buffer.text.line(cursor.line).unwrap_or_default();
    if !line
        .chars()
        .take(cursor.column)
        .all(|character| character.is_whitespace())
    {
        return false;
    }
    let (leading_cols, leading_end) = leading_whitespace_info(&line, indent_size);
    if leading_cols == 0 {
        return false;
    }
    let target_remove_cols = indent_size.min(leading_cols);
    let mut removed_cols = 0usize;
    let mut remove_end = 0usize;
    for (index, character) in line[..leading_end].char_indices() {
        let width = if character == '\t' {
            indent_size.max(1)
        } else {
            1
        };
        if removed_cols + width > target_remove_cols {
            break;
        }
        removed_cols += width;
        remove_end = index + character.len_utf8();
        if removed_cols >= target_remove_cols {
            break;
        }
    }
    if remove_end == 0 {
        return false;
    }
    let removed_chars = line[..remove_end].chars().count();
    buffer.delete_range(TextRange::new(
        TextPoint::new(cursor.line, 0),
        TextPoint::new(cursor.line, removed_chars),
    ));
    buffer.set_cursor(TextPoint::new(
        cursor.line,
        cursor.column.saturating_sub(removed_chars),
    ));
    true
}

fn wrap_columns_for_width(width: u32, cell_width: i32) -> usize {
    let cell_width = cell_width.max(1) as u32;
    let line_number_width = cell_width.saturating_mul(5);
    let fringe_width = cell_width;
    let right_padding = cell_width;
    let padding = 12u32 + line_number_width + fringe_width + right_padding;
    let available = width.saturating_sub(padding).max(cell_width);
    (available / cell_width).max(1) as usize
}

fn wrap_line_segments(
    map: &LineCharMap,
    first_cols: usize,
    continuation_cols: usize,
) -> Vec<LineWrapSegment> {
    let first_cols = first_cols.max(1);
    let continuation_cols = continuation_cols.max(1);
    let len = map.len();
    if len == 0 {
        return vec![LineWrapSegment {
            start_col: 0,
            end_col: 0,
        }];
    }

    let mut segments = Vec::new();
    let mut start = 0;
    let mut max_cols = first_cols;
    while start < len {
        let remaining = len - start;
        if remaining <= max_cols {
            segments.push(LineWrapSegment {
                start_col: start,
                end_col: len,
            });
            break;
        }

        let wrap_limit = start + max_cols;
        let mut break_at = None;
        for idx in (start..wrap_limit).rev() {
            if map.whitespace.get(idx).copied().unwrap_or(false) {
                break_at = Some(idx + 1);
                break;
            }
        }
        if break_at.is_none() {
            for idx in wrap_limit..len {
                if map.whitespace.get(idx).copied().unwrap_or(false) {
                    break_at = Some(idx + 1);
                    break;
                }
            }
        }

        let end = break_at.unwrap_or(wrap_limit);
        segments.push(LineWrapSegment {
            start_col: start,
            end_col: end,
        });
        start = end;
        max_cols = continuation_cols;
    }

    if segments.is_empty() {
        segments.push(LineWrapSegment {
            start_col: 0,
            end_col: 0,
        });
    }

    segments
}

fn wrap_line_segments_for_line(
    line: &str,
    wrap_cols: usize,
    indent_size: usize,
) -> Vec<LineWrapSegment> {
    let char_map = LineCharMap::new(line);
    let (leading_indent_cols, _) = leading_whitespace_info(line, indent_size);
    let continuation_indent_cols = leading_indent_cols.saturating_add(indent_size);
    let continuation_cols = wrap_cols.saturating_sub(continuation_indent_cols).max(1);
    wrap_line_segments(&char_map, wrap_cols, continuation_cols)
}

fn line_wrap_row_count(line: &str, wrap_cols: usize, indent_size: usize) -> usize {
    wrap_line_segments_for_line(line, wrap_cols, indent_size)
        .len()
        .max(1)
}

fn segment_index_for_column(segments: &[LineWrapSegment], column: usize) -> usize {
    if segments.is_empty() {
        return 0;
    }
    for (index, segment) in segments.iter().enumerate() {
        if column < segment.end_col || index == segments.len().saturating_sub(1) {
            return index;
        }
    }
    segments.len().saturating_sub(1)
}

#[derive(Debug, Clone)]
struct UndoSnapshot {
    text: String,
    cursor: TextPoint,
}

impl UndoSnapshot {
    fn from_buffer(buffer: &TextBuffer) -> Self {
        Self {
            text: buffer.text(),
            cursor: buffer.cursor(),
        }
    }

    fn preview_line(&self) -> Option<String> {
        let line = self.text.lines().next().unwrap_or("");
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    }
}

#[derive(Debug, Clone)]
struct UndoNode {
    parent: Option<usize>,
    children: Vec<usize>,
    snapshot: UndoSnapshot,
    sequence: u64,
    last_child: Option<usize>,
}

#[derive(Debug, Clone)]
struct UndoTree {
    nodes: Vec<UndoNode>,
    current: usize,
    next_sequence: u64,
    last_revision: u64,
}

#[derive(Debug, Clone)]
struct UndoTreeEntry {
    node_id: usize,
    label: String,
    detail: String,
    preview: Option<String>,
}

impl UndoTree {
    fn new(buffer: &TextBuffer) -> Self {
        let snapshot = UndoSnapshot::from_buffer(buffer);
        Self {
            nodes: vec![UndoNode {
                parent: None,
                children: Vec::new(),
                snapshot,
                sequence: 0,
                last_child: None,
            }],
            current: 0,
            next_sequence: 1,
            last_revision: buffer.revision(),
        }
    }

    fn update_revision(&mut self, revision: u64) {
        self.last_revision = revision;
    }

    fn record_snapshot(&mut self, buffer: &TextBuffer) -> bool {
        let revision = buffer.revision();
        if revision == self.last_revision {
            return false;
        }
        let snapshot = UndoSnapshot::from_buffer(buffer);
        let parent = self.current;
        let node_id = self.nodes.len();
        let sequence = self.next_sequence;
        self.nodes.push(UndoNode {
            parent: Some(parent),
            children: Vec::new(),
            snapshot,
            sequence,
            last_child: None,
        });
        if let Some(parent_node) = self.nodes.get_mut(parent) {
            parent_node.children.push(node_id);
            parent_node.last_child = Some(node_id);
        }
        self.current = node_id;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.last_revision = revision;
        true
    }

    fn undo(&mut self) -> Option<UndoSnapshot> {
        let parent = self.nodes.get(self.current)?.parent?;
        let current = self.current;
        if let Some(parent_node) = self.nodes.get_mut(parent) {
            parent_node.last_child = Some(current);
        }
        self.current = parent;
        self.nodes
            .get(self.current)
            .map(|node| node.snapshot.clone())
    }

    fn redo(&mut self) -> Option<UndoSnapshot> {
        let next = {
            let node = self.nodes.get(self.current)?;
            node.last_child.or_else(|| node.children.last().copied())
        }?;
        self.current = next;
        self.nodes
            .get(self.current)
            .map(|node| node.snapshot.clone())
    }

    fn select(&mut self, node_id: usize) -> Option<UndoSnapshot> {
        if node_id >= self.nodes.len() {
            return None;
        }
        if let Some(parent) = self.nodes[node_id].parent
            && let Some(parent_node) = self.nodes.get_mut(parent)
        {
            parent_node.last_child = Some(node_id);
        }
        self.current = node_id;
        self.nodes.get(node_id).map(|node| node.snapshot.clone())
    }

    fn picker_entries(&self) -> (Vec<UndoTreeEntry>, usize) {
        let mut entries = Vec::new();
        let mut selected_index = None;
        if !self.nodes.is_empty() {
            self.collect_entries(0, 0, &mut entries, &mut selected_index);
        }
        (entries, selected_index.unwrap_or(0))
    }

    fn collect_entries(
        &self,
        node_id: usize,
        depth: usize,
        entries: &mut Vec<UndoTreeEntry>,
        selected_index: &mut Option<usize>,
    ) {
        let Some(node) = self.nodes.get(node_id) else {
            return;
        };
        let is_current = node_id == self.current;
        if is_current {
            *selected_index = Some(entries.len());
        }
        let indent = "  ".repeat(depth);
        let prefix = if is_current { "* " } else { "  " };
        let cursor = node.snapshot.cursor;
        let label = if node.parent.is_none() {
            format!(
                "{indent}{prefix}root line {}, col {}",
                cursor.line + 1,
                cursor.column + 1
            )
        } else {
            format!(
                "{indent}{prefix}{} line {}, col {}",
                node.sequence,
                cursor.line + 1,
                cursor.column + 1
            )
        };
        let detail = if is_current {
            format!("current | children: {}", node.children.len())
        } else if node.parent.is_none() {
            format!("root | children: {}", node.children.len())
        } else {
            format!("children: {}", node.children.len())
        };
        entries.push(UndoTreeEntry {
            node_id,
            label,
            detail,
            preview: node.snapshot.preview_line(),
        });
        for child in &node.children {
            self.collect_entries(*child, depth + 1, entries, selected_index);
        }
    }
}

#[derive(Debug, Clone)]
struct InputField {
    prompt: String,
    text: String,
    placeholder: Option<String>,
    hint: Option<String>,
    cursor: usize,
    selection_anchor: Option<usize>,
}

impl InputField {
    fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            text: String::new(),
            placeholder: None,
            hint: None,
            cursor: 0,
            selection_anchor: None,
        }
    }

    fn prompt(&self) -> &str {
        &self.prompt
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    fn set_placeholder(&mut self, placeholder: Option<String>) {
        self.placeholder = placeholder;
    }

    fn text_line_count(&self) -> usize {
        self.line_starts().len().max(1)
    }

    fn wrapped_visual_rows(&self, available_cols: usize) -> Vec<String> {
        let prompt_len = self.prompt.chars().count();
        let cols_per_row = available_cols.saturating_sub(prompt_len).max(1);
        let mut rows = Vec::new();
        for line in self.text.split('\n') {
            let chars: Vec<char> = line.chars().collect();
            if chars.is_empty() {
                rows.push(String::new());
            } else {
                let mut start = 0;
                while start < chars.len() {
                    let end = (start + cols_per_row).min(chars.len());
                    rows.push(chars[start..end].iter().collect());
                    start = end;
                }
            }
        }
        if rows.is_empty() {
            rows.push(String::new());
        }
        rows
    }

    fn visual_line_count(&self, available_cols: usize) -> usize {
        self.wrapped_visual_rows(available_cols).len().max(1)
    }

    fn cursor_visual_row_col(&self, available_cols: usize) -> (usize, usize) {
        self.visual_row_col_for_cursor(self.cursor_char(), available_cols)
    }

    fn visual_row_col_for_cursor(
        &self,
        cursor_char: usize,
        available_cols: usize,
    ) -> (usize, usize) {
        let prompt_len = self.prompt.chars().count();
        let cols_per_row = available_cols.saturating_sub(prompt_len).max(1);
        let (logical_line, col_in_logical_line) = self.line_col_for_char(cursor_char);
        let mut visual_row = 0usize;
        for (idx, line) in self.text.split('\n').enumerate() {
            if idx == logical_line {
                break;
            }
            let char_count = line.chars().count();
            visual_row += if char_count == 0 {
                1
            } else {
                char_count.div_ceil(cols_per_row)
            };
        }
        let wrap_row = col_in_logical_line / cols_per_row;
        let col_in_wrap_row = col_in_logical_line % cols_per_row;
        visual_row += wrap_row;
        (visual_row, col_in_wrap_row)
    }

    fn line_col_for_char(&self, cursor_char: usize) -> (usize, usize) {
        let mut consumed = 0usize;
        for (line_index, line) in self.text.split('\n').enumerate() {
            let line_len = line.chars().count();
            if cursor_char <= consumed + line_len {
                return (line_index, cursor_char.saturating_sub(consumed));
            }
            consumed = consumed.saturating_add(line_len + 1);
        }
        self.cursor_line_col()
    }

    fn append_text(&mut self, text: &str) {
        self.insert_text(text);
    }

    fn set_text(&mut self, text: &str) {
        let filtered: String = text
            .chars()
            .filter(|character| *character != '\r')
            .collect();
        self.text.clear();
        self.text.push_str(&filtered);
        self.cursor = self.text.chars().count();
        self.selection_anchor = None;
    }

    fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor == 0 {
            return false;
        }
        let start = self.cursor.saturating_sub(1);
        let end = self.cursor;
        self.delete_range(start, end);
        self.cursor = start;
        true
    }

    fn delete_forward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor >= self.text.chars().count() {
            return false;
        }
        let end = self.cursor.saturating_add(1);
        self.delete_range(self.cursor, end);
        true
    }

    fn move_left(&mut self) -> bool {
        self.selection_anchor = None;
        if self.cursor == 0 {
            return false;
        }
        self.cursor = self.cursor.saturating_sub(1);
        true
    }

    fn move_right(&mut self) -> bool {
        self.selection_anchor = None;
        let total = self.text.chars().count();
        if self.cursor >= total {
            return false;
        }
        self.cursor = (self.cursor + 1).min(total);
        true
    }

    fn move_up(&mut self) -> bool {
        self.selection_anchor = None;
        let starts = self.line_starts();
        let total = self.text.chars().count();
        let (line, col) = self.cursor_line_col_with_starts(&starts);
        if line == 0 {
            return false;
        }
        let prev_line = line.saturating_sub(1);
        let prev_len = Self::line_len_for(&starts, total, prev_line);
        let new_col = col.min(prev_len);
        self.cursor = starts[prev_line] + new_col;
        true
    }

    fn move_down(&mut self) -> bool {
        self.selection_anchor = None;
        let starts = self.line_starts();
        let total = self.text.chars().count();
        let (line, col) = self.cursor_line_col_with_starts(&starts);
        let next_line = line.saturating_add(1);
        if next_line >= starts.len() {
            return false;
        }
        let next_len = Self::line_len_for(&starts, total, next_line);
        let new_col = col.min(next_len);
        self.cursor = starts[next_line] + new_col;
        true
    }

    fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.selection_anchor = None;
    }

    fn insert_text(&mut self, text: &str) {
        let filtered: String = text
            .chars()
            .filter(|character| *character != '\r')
            .collect();
        if filtered.is_empty() {
            return;
        }
        let _ = self.delete_selection();
        let insert_at = self.byte_index_for_char(self.cursor);
        self.text.insert_str(insert_at, &filtered);
        self.cursor = self.cursor.saturating_add(filtered.chars().count());
    }

    fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    fn cursor_char(&self) -> usize {
        self.cursor.min(self.char_count())
    }

    fn cursor_point(&self) -> TextPoint {
        let buffer = TextBuffer::from_text(&self.text);
        buffer.point_from_char_index(self.cursor_char())
    }

    fn move_line_start(&mut self) -> bool {
        self.selection_anchor = None;
        let before = self.cursor_char();
        let (line, _) = self.cursor_line_col();
        let starts = self.line_starts();
        self.cursor = starts.get(line).copied().unwrap_or(0);
        self.cursor != before
    }

    fn move_line_end(&mut self) -> bool {
        self.selection_anchor = None;
        let before = self.cursor_char();
        let starts = self.line_starts();
        let total = self.char_count();
        let (line, _) = self.cursor_line_col_with_starts(&starts);
        self.cursor = starts[line] + Self::line_len_for(&starts, total, line);
        self.cursor != before
    }

    fn start_selection(&mut self) {
        self.selection_anchor = Some(self.cursor_char());
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn selected_char_range(&self, kind: VisualSelectionKind) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        let total = self.char_count();
        if total == 0 {
            return None;
        }
        match kind {
            VisualSelectionKind::Character => {
                let head = self.cursor_char().min(total.saturating_sub(1));
                if head >= anchor {
                    Some((anchor, (head + 1).min(total)))
                } else {
                    Some((head, (anchor + 1).min(total)))
                }
            }
            VisualSelectionKind::Line => {
                let anchor_point = {
                    let buffer = TextBuffer::from_text(&self.text);
                    buffer.point_from_char_index(anchor.min(total))
                };
                let head_point = {
                    let buffer = TextBuffer::from_text(&self.text);
                    buffer.point_from_char_index(self.cursor_char().min(total))
                };
                let starts = self.line_starts();
                let start_line = anchor_point.line.min(head_point.line);
                let end_line = anchor_point.line.max(head_point.line);
                let start = starts.get(start_line).copied().unwrap_or(0);
                let end = if end_line + 1 < starts.len() {
                    starts[end_line + 1]
                } else {
                    total
                };
                Some((start, end))
            }
            VisualSelectionKind::Block => None,
        }
    }

    fn selected_text(&self, kind: VisualSelectionKind) -> Option<String> {
        let (start, end) = self.selected_char_range(kind)?;
        let start_byte = self.byte_index_for_char(start);
        let end_byte = self.byte_index_for_char(end);
        (start_byte < end_byte).then(|| self.text[start_byte..end_byte].to_owned())
    }

    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selected_char_range(VisualSelectionKind::Character) else {
            return false;
        };
        self.delete_range(start, end);
        self.cursor = start;
        self.selection_anchor = None;
        true
    }

    fn selection_visual_ranges(
        &self,
        kind: VisualSelectionKind,
        available_cols: usize,
    ) -> Vec<(usize, usize, usize)> {
        let Some((start, end)) = self.selected_char_range(kind) else {
            return Vec::new();
        };
        let prompt_len = self.prompt.chars().count();
        let cols_per_row = available_cols.saturating_sub(prompt_len).max(1);
        let starts = self.line_starts();
        let total = self.char_count();
        let mut visual_row_offsets = Vec::with_capacity(starts.len());
        let mut visual_row = 0usize;
        for line_index in 0..starts.len() {
            visual_row_offsets.push(visual_row);
            let line_len = Self::line_len_for(&starts, total, line_index);
            visual_row += if line_len == 0 {
                1
            } else {
                line_len.div_ceil(cols_per_row)
            };
        }
        let mut ranges = Vec::new();
        for (line_index, line_start) in starts.iter().copied().enumerate() {
            let line_len = Self::line_len_for(&starts, total, line_index);
            let line_end = line_start + line_len;
            let line_selection_start = start.max(line_start);
            let line_selection_end = end.min(line_end);
            if line_selection_start >= line_selection_end {
                continue;
            }
            let start_col = line_selection_start - line_start;
            let end_col = line_selection_end - line_start;
            let start_row = start_col / cols_per_row;
            let end_row = end_col.saturating_sub(1) / cols_per_row;
            for row in start_row..=end_row {
                let row_start = row * cols_per_row;
                let row_end = ((row + 1) * cols_per_row).min(line_len.max(1));
                let selection_start = start_col.max(row_start);
                let selection_end = end_col.min(row_end);
                if selection_start < selection_end {
                    ranges.push((
                        visual_row_offsets[line_index] + row,
                        selection_start - row_start,
                        selection_end - row_start,
                    ));
                }
            }
        }
        ranges
    }

    fn cursor_line_col(&self) -> (usize, usize) {
        let starts = self.line_starts();
        self.cursor_line_col_with_starts(&starts)
    }

    fn cursor_line_col_with_starts(&self, starts: &[usize]) -> (usize, usize) {
        let line = starts
            .iter()
            .rposition(|start| *start <= self.cursor)
            .unwrap_or(0);
        let col = self.cursor.saturating_sub(starts[line]);
        (line, col)
    }

    fn line_starts(&self) -> Vec<usize> {
        let mut starts = vec![0];
        for (index, character) in self.text.chars().enumerate() {
            if character == '\n' {
                starts.push(index.saturating_add(1));
            }
        }
        starts
    }

    fn line_len_for(starts: &[usize], total: usize, line: usize) -> usize {
        let start = starts.get(line).copied().unwrap_or(0);
        let end = starts
            .get(line.saturating_add(1))
            .copied()
            .map(|next| next.saturating_sub(1))
            .unwrap_or(total);
        end.saturating_sub(start)
    }

    fn byte_index_for_char(&self, char_index: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_index)
            .map(|(index, _)| index)
            .unwrap_or(self.text.len())
    }

    fn delete_range(&mut self, start: usize, end: usize) {
        let start_byte = self.byte_index_for_char(start);
        let end_byte = self.byte_index_for_char(end);
        if start_byte < end_byte {
            self.text.replace_range(start_byte..end_byte, "");
        }
    }
}

#[derive(Debug, Clone)]
struct SectionLineMeta {
    section_id: String,
    kind: SectionRenderLineKind,
    action: Option<SectionAction>,
}

#[derive(Debug, Clone, Default)]
struct SectionedBufferState {
    collapsed: SectionCollapseState,
    lines: Vec<SectionLineMeta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BufferContextOverlayCacheKey {
    buffer_revision: u64,
    buffer_name: String,
    language_id: Option<String>,
    viewport_top_line: usize,
    cursor_line: usize,
    cursor_column: usize,
}

#[derive(Debug, Clone)]
struct BufferContextOverlaySnapshot {
    key: BufferContextOverlayCacheKey,
    headerline_lines: Vec<String>,
    ghost_text_by_line: BTreeMap<usize, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ShellBuffer {
    id: BufferId,
    pub(crate) name: String,
    pub(crate) kind: BufferKind,
    read_only: bool,
    input: Option<InputField>,
    section_state: Option<SectionedBufferState>,
    plugin_section_state: Option<PluginSectionBufferState>,
    image_state: Option<ImageBufferState>,
    pdf_state: Option<PdfBufferState>,
    acp_state: Option<AcpBufferState>,
    git_snapshot: Option<GitStatusSnapshot>,
    git_view: Option<GitViewState>,
    git_fringe: Option<GitFringeState>,
    git_fringe_dirty: bool,
    git_fringe_last_edit_at: Option<Instant>,
    browser_state: Option<BrowserBufferState>,
    directory_state: Option<DirectoryViewState>,
    terminal_render: Option<TerminalRenderSnapshot>,
    pub(crate) text: TextBuffer,
    backing_file_fingerprint: Option<BackingFileFingerprint>,
    backing_file_reload_pending: bool,
    backing_file_check_in_flight: bool,
    undo_tree: UndoTree,
    language_id: Option<String>,
    pub(crate) scroll_row: usize,
    viewport_lines: usize,
    wrap_cache: Option<WrapRowCache>,
    context_overlay_cache: Arc<Mutex<Option<BufferContextOverlaySnapshot>>>,
    syntax_error: Option<String>,
    syntax_lines: BTreeMap<usize, Vec<LineSyntaxSpan>>,
    syntax_dirty: bool,
    syntax_requested_revision: Option<u64>,
    syntax_requested_window: Option<SyntaxLineWindow>,
    syntax_applied_window: Option<SyntaxLineWindow>,
    lsp_enabled: bool,
    lsp_diagnostics: Vec<LspDiagnostic>,
    lsp_diagnostic_lines: BTreeMap<usize, Box<[DiagnosticLineSpan]>>,
    lsp_diagnostics_revision: u64,
    last_edit_at: Option<Instant>,
    vim_buffer_state: VimBufferState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcpPane {
    Plan,
    Output,
    Input,
    Footer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BackingFileFingerprint {
    modified_at: Option<SystemTime>,
    len: u64,
}

impl BackingFileFingerprint {
    fn read(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        Ok(Self {
            modified_at: metadata.modified().ok(),
            len: metadata.len(),
        })
    }
}

#[derive(Debug, Clone)]
struct PluginSectionBufferState {
    base_title: String,
    base_writable: bool,
    base_min_rows: Option<usize>,
    base_update: PluginBufferSectionUpdate,
    active_section: usize,
    evaluate_target_section: usize,
    attached_sections: Vec<PluginTextPaneState>,
}

#[derive(Debug, Clone)]
struct PluginTextPaneState {
    title: String,
    writable: bool,
    min_rows: Option<usize>,
    update: PluginBufferSectionUpdate,
    text: TextBuffer,
    scroll_row: usize,
    viewport_rows: usize,
    wrap_cols: usize,
}

#[derive(Debug, Clone)]
struct AcpBufferState {
    session_title: Option<String>,
    active_pane: AcpPane,
    plan_entries: Vec<PlanEntry>,
    output_items: Vec<AcpOutputItem>,
    tool_item_indices: BTreeMap<String, usize>,
    plan_pane: AcpPaneState,
    output_pane: AcpPaneState,
    input: InputField,
    footer_pane: PluginTextPaneState,
}

#[derive(Debug, Clone)]
struct AcpPaneState {
    text: TextBuffer,
    render_lines: Vec<AcpRenderedLine>,
    scroll_row: usize,
    viewport_rows: usize,
    wrap_cols: usize,
}

#[derive(Debug, Clone)]
enum AcpOutputItem {
    UserPrompt(String),
    AgentBlocks(Vec<ContentBlock>),
    ToolCall(ToolCall),
    SystemMessage(String),
}

#[derive(Debug, Clone)]
enum AcpRenderedLine {
    Text(AcpRenderedTextLine),
    Image(AcpRenderedImageLine),
    ImageContinuation,
    Spacer,
}

#[derive(Debug, Clone)]
struct AcpRenderedTextLine {
    prefix: Vec<AcpRenderedSegment>,
    text: String,
    text_role: AcpColorRole,
}

#[derive(Debug, Clone)]
struct AcpRenderedSegment {
    text: String,
    role: AcpColorRole,
    animate: bool,
}

#[derive(Debug, Clone)]
struct AcpRenderedImageLine {
    label: String,
    image: Option<AcpDecodedImage>,
    rows: usize,
}

#[derive(Debug, Clone)]
struct AcpDecodedImage {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

type DecodedImage = AcpDecodedImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageBufferFormat {
    Raster,
    Svg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageBufferMode {
    Rendered,
    Source,
}

#[derive(Debug, Clone)]
struct ImageBufferState {
    format: ImageBufferFormat,
    mode: ImageBufferMode,
    decoded: DecodedImage,
    zoom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcpColorRole {
    Default,
    Muted,
    Accent,
    Success,
    Warning,
    Error,
    PriorityHigh,
    PriorityMedium,
    PriorityLow,
}

const ACP_IMAGE_ROWS: usize = 12;

impl Default for PluginTextPaneState {
    fn default() -> Self {
        Self {
            title: String::new(),
            writable: false,
            min_rows: None,
            update: PluginBufferSectionUpdate::Replace,
            text: TextBuffer::new(),
            scroll_row: 0,
            viewport_rows: 1,
            wrap_cols: 1,
        }
    }
}

impl PluginSectionBufferState {
    fn new(config: PluginBufferSections, evaluate_target_section: Option<&str>) -> Option<Self> {
        let mut sections = config.items().iter();
        let base = sections.next()?;
        let attached_sections = sections
            .map(|section| {
                let mut pane = PluginTextPaneState {
                    title: section.name().to_owned(),
                    writable: section.writable(),
                    min_rows: section.min_lines(),
                    update: section.update(),
                    ..PluginTextPaneState::default()
                };
                pane.replace_lines(
                    section
                        .initial_lines()
                        .iter()
                        .map(|line| line.to_string())
                        .collect(),
                    true,
                );
                pane
            })
            .collect::<Vec<_>>();
        let evaluate_target_section = evaluate_target_section
            .and_then(|name| {
                config
                    .items()
                    .iter()
                    .position(|section| section.name() == name)
            })
            .or_else(|| {
                config
                    .items()
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, section)| (!section.writable()).then_some(index))
            })
            .unwrap_or_else(|| config.items().len().saturating_sub(1));
        Some(Self {
            base_title: base.name().to_owned(),
            base_writable: base.writable(),
            base_min_rows: base.min_lines(),
            base_update: base.update(),
            active_section: 0,
            evaluate_target_section,
            attached_sections,
        })
    }

    fn section_count(&self) -> usize {
        self.attached_sections.len().saturating_add(1)
    }

    fn active_section_writable(&self) -> bool {
        if self.active_section == 0 {
            self.base_writable
        } else {
            self.attached_sections
                .get(self.active_section.saturating_sub(1))
                .map(|pane| pane.writable)
                .unwrap_or(false)
        }
    }

    fn active_attached_section(&self) -> Option<&PluginTextPaneState> {
        self.active_section
            .checked_sub(1)
            .and_then(|index| self.attached_sections.get(index))
    }

    fn has_active_attached_section(&self) -> bool {
        self.active_section > 0
    }

    fn active_attached_section_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        self.active_section
            .checked_sub(1)
            .and_then(|index| self.attached_sections.get_mut(index))
    }

    fn attached_section(&self, section_index: usize) -> Option<&PluginTextPaneState> {
        section_index
            .checked_sub(1)
            .and_then(|index| self.attached_sections.get(index))
    }

    fn attached_section_mut(&mut self, section_index: usize) -> Option<&mut PluginTextPaneState> {
        section_index
            .checked_sub(1)
            .and_then(|index| self.attached_sections.get_mut(index))
    }
}

impl Default for AcpPaneState {
    fn default() -> Self {
        Self {
            text: TextBuffer::new(),
            render_lines: Vec::new(),
            scroll_row: 0,
            viewport_rows: 1,
            wrap_cols: 1,
        }
    }
}

impl AcpBufferState {
    fn new(client_label: String) -> Self {
        let _ = client_label;
        let mut input = InputField::new("> ");
        input.set_placeholder(Some(ACP_INPUT_PLACEHOLDER.to_owned()));
        Self {
            session_title: None,
            active_pane: AcpPane::Output,
            plan_entries: Vec::new(),
            output_items: Vec::new(),
            tool_item_indices: BTreeMap::new(),
            plan_pane: AcpPaneState::default(),
            output_pane: AcpPaneState::default(),
            input,
            footer_pane: PluginTextPaneState {
                min_rows: Some(1),
                ..PluginTextPaneState::default()
            },
        }
    }
}

impl AcpPaneState {
    fn line_count(&self) -> usize {
        self.text.line_count()
    }

    fn line_len_chars(&self, line_index: usize) -> usize {
        self.text.line_len_chars(line_index).unwrap_or(0)
    }

    fn cursor(&self) -> TextPoint {
        self.text.cursor()
    }

    fn set_cursor(&mut self, point: TextPoint) {
        self.text.set_cursor(point);
    }

    fn visible_rows(&self) -> usize {
        self.viewport_rows.max(1)
    }

    fn wrap_cols(&self) -> usize {
        self.wrap_cols.max(1)
    }

    fn set_view_metrics(&mut self, visible_rows: usize, wrap_cols: usize) {
        self.viewport_rows = visible_rows.max(1);
        self.wrap_cols = wrap_cols.max(1);
        self.scroll_row = self.scroll_row.min(self.max_scroll_row());
    }

    fn max_scroll_row_for(&self, visible_rows: usize) -> usize {
        if self.render_lines.is_empty() {
            return 0;
        }
        let visible_rows = visible_rows.max(1);
        let mut rows = 0usize;
        for line_index in (0..self.render_lines.len()).rev() {
            let row_count =
                acp_rendered_line_row_count(&self.render_lines[line_index], self.wrap_cols());
            if rows.saturating_add(row_count) > visible_rows {
                return if rows == 0 {
                    line_index
                } else {
                    line_index.saturating_add(1)
                };
            }
            rows = rows.saturating_add(row_count);
        }
        0
    }

    fn max_scroll_row(&self) -> usize {
        self.max_scroll_row_for(self.visible_rows())
    }

    fn should_follow_output(&self, visible_rows: usize) -> bool {
        if self.render_lines.is_empty() {
            return true;
        }
        self.scroll_row >= self.max_scroll_row_for(visible_rows)
    }

    fn replace_render_lines(
        &mut self,
        render_lines: Vec<AcpRenderedLine>,
        follow_output: bool,
        visible_rows: usize,
    ) {
        let cursor = self.cursor();
        let scroll_row = self.scroll_row;
        let lines = render_lines
            .iter()
            .map(AcpRenderedLine::plain_text)
            .collect::<Vec<_>>();
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.viewport_rows = visible_rows.max(1);
        self.text = text;
        self.text.mark_clean();
        self.render_lines = render_lines;
        let line_count = self.line_count();
        if line_count == 0 {
            self.text.set_cursor(TextPoint::default());
            self.scroll_row = 0;
            return;
        }
        let line = cursor.line.min(line_count.saturating_sub(1));
        let column = cursor.column.min(self.line_len_chars(line));
        self.text.set_cursor(TextPoint::new(line, column));
        if follow_output {
            self.scroll_row = self.max_scroll_row();
        } else {
            self.scroll_row = scroll_row.min(self.max_scroll_row());
        }
    }

    fn line_at_viewport_offset(&self, offset: usize) -> usize {
        if self.render_lines.is_empty() {
            return 0;
        }
        let mut line_index = self
            .scroll_row
            .min(self.render_lines.len().saturating_sub(1));
        let mut remaining = offset;
        while line_index + 1 < self.render_lines.len() {
            let row_count =
                acp_rendered_line_row_count(&self.render_lines[line_index], self.wrap_cols());
            if remaining < row_count {
                return line_index;
            }
            remaining = remaining.saturating_sub(row_count);
            line_index = line_index.saturating_add(1);
        }
        line_index
    }

    fn cursor_viewport_offset(&self) -> usize {
        if self.render_lines.is_empty() {
            return 0;
        }
        let cursor = self.cursor();
        if cursor.line < self.scroll_row {
            return 0;
        }
        let mut offset = 0usize;
        for line_index in self.scroll_row..cursor.line {
            offset = offset.saturating_add(acp_rendered_line_row_count(
                &self.render_lines[line_index],
                self.wrap_cols(),
            ));
        }
        let cursor_segment = self
            .render_lines
            .get(cursor.line)
            .and_then(|line| match line {
                AcpRenderedLine::Text(line) => Some(segment_index_for_column(
                    &acp_rendered_text_segments(line, self.wrap_cols()),
                    cursor.column,
                )),
                _ => None,
            })
            .unwrap_or(0);
        offset.saturating_add(cursor_segment)
    }

    fn ensure_cursor_visible(&mut self) {
        if self.render_lines.is_empty() {
            self.scroll_row = 0;
            return;
        }
        let cursor = self.cursor();
        if cursor.line < self.scroll_row {
            self.scroll_row = cursor.line;
            return;
        }
        let visible_rows = self.visible_rows();
        let mut offset = self.cursor_viewport_offset();
        if offset < visible_rows {
            return;
        }
        let mut new_scroll = self.scroll_row;
        while offset >= visible_rows && new_scroll < cursor.line {
            offset = offset.saturating_sub(acp_rendered_line_row_count(
                &self.render_lines[new_scroll],
                self.wrap_cols(),
            ));
            new_scroll = new_scroll.saturating_add(1);
        }
        self.scroll_row = new_scroll.min(self.max_scroll_row());
    }
}

impl PluginTextPaneState {
    fn line_count(&self) -> usize {
        self.text.line_count()
    }

    fn line_len_chars(&self, line_index: usize) -> usize {
        self.text.line_len_chars(line_index).unwrap_or(0)
    }

    fn cursor(&self) -> TextPoint {
        self.text.cursor()
    }

    fn set_cursor(&mut self, point: TextPoint) {
        self.text.set_cursor(point);
    }

    fn visible_rows(&self) -> usize {
        self.viewport_rows.max(1)
    }

    fn wrap_cols(&self) -> usize {
        self.wrap_cols.max(1)
    }

    fn set_view_metrics(&mut self, visible_rows: usize, wrap_cols: usize) {
        self.viewport_rows = visible_rows.max(1);
        self.wrap_cols = wrap_cols.max(1);
        self.scroll_row = self.scroll_row.min(self.max_scroll_row());
    }

    fn row_count_for_line(&self, line_index: usize) -> usize {
        let line = self.text.line(line_index).unwrap_or_default();
        wrap_line_segments(&LineCharMap::new(&line), self.wrap_cols(), self.wrap_cols())
            .len()
            .max(1)
    }

    fn max_scroll_row_for(&self, visible_rows: usize) -> usize {
        let line_count = self.line_count();
        if line_count == 0 {
            return 0;
        }
        let visible_rows = visible_rows.max(1);
        let mut rows = 0usize;
        for line_index in (0..line_count).rev() {
            let row_count = self.row_count_for_line(line_index);
            if rows.saturating_add(row_count) > visible_rows {
                return if rows == 0 {
                    line_index
                } else {
                    line_index.saturating_add(1)
                };
            }
            rows = rows.saturating_add(row_count);
        }
        0
    }

    fn max_scroll_row(&self) -> usize {
        self.max_scroll_row_for(self.visible_rows())
    }

    fn should_follow_output(&self) -> bool {
        self.scroll_row >= self.max_scroll_row()
    }

    fn replace_lines(&mut self, lines: Vec<String>, follow_output: bool) {
        let cursor = self.cursor();
        let scroll_row = self.scroll_row;
        self.text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.text.mark_clean();
        if self.line_count() == 0 {
            self.text.set_cursor(TextPoint::default());
            self.scroll_row = 0;
            return;
        }
        let line = cursor.line.min(self.line_count().saturating_sub(1));
        let column = cursor.column.min(self.line_len_chars(line));
        self.text.set_cursor(TextPoint::new(line, column));
        if follow_output {
            self.scroll_row = self.max_scroll_row();
        } else {
            self.scroll_row = scroll_row.min(self.max_scroll_row());
        }
    }

    fn append_lines(&mut self, mut lines: Vec<String>, follow_output: bool) {
        if lines.is_empty() {
            return;
        }
        let mut existing = (0..self.line_count())
            .map(|line_index| self.text.line(line_index).unwrap_or_default().to_owned())
            .collect::<Vec<_>>();
        existing.append(&mut lines);
        self.replace_lines(existing, follow_output);
    }

    fn line_at_viewport_offset(&self, offset: usize) -> usize {
        let line_count = self.line_count();
        if line_count == 0 {
            return 0;
        }
        let mut line_index = self.scroll_row.min(line_count.saturating_sub(1));
        let mut remaining = offset;
        while line_index + 1 < line_count {
            let row_count = self.row_count_for_line(line_index);
            if remaining < row_count {
                return line_index;
            }
            remaining = remaining.saturating_sub(row_count);
            line_index = line_index.saturating_add(1);
        }
        line_index
    }

    fn cursor_viewport_offset(&self) -> usize {
        let line_count = self.line_count();
        if line_count == 0 {
            return 0;
        }
        let cursor = self.cursor();
        if cursor.line < self.scroll_row {
            return 0;
        }
        let mut offset = 0usize;
        for line_index in self.scroll_row..cursor.line {
            offset = offset.saturating_add(self.row_count_for_line(line_index));
        }
        let line = self.text.line(cursor.line).unwrap_or_default();
        let segments =
            wrap_line_segments(&LineCharMap::new(&line), self.wrap_cols(), self.wrap_cols());
        offset.saturating_add(segment_index_for_column(&segments, cursor.column))
    }

    fn ensure_cursor_visible(&mut self) {
        let line_count = self.line_count();
        if line_count == 0 {
            self.scroll_row = 0;
            return;
        }
        let cursor = self.cursor();
        if cursor.line < self.scroll_row {
            self.scroll_row = cursor.line;
            return;
        }
        let visible_rows = self.visible_rows();
        let mut offset = self.cursor_viewport_offset();
        if offset < visible_rows {
            return;
        }
        let mut new_scroll = self.scroll_row;
        while offset >= visible_rows && new_scroll < cursor.line {
            offset = offset.saturating_sub(self.row_count_for_line(new_scroll));
            new_scroll = new_scroll.saturating_add(1);
        }
        self.scroll_row = new_scroll.min(self.max_scroll_row());
    }
}

impl AcpRenderedLine {
    fn plain_text(&self) -> String {
        match self {
            Self::Text(line) => line.text.clone(),
            Self::Image(line) => line.label.clone(),
            Self::ImageContinuation | Self::Spacer => String::new(),
        }
    }
}

fn acp_rendered_text_wrap_cols(line: &AcpRenderedTextLine, wrap_cols: usize) -> usize {
    wrap_cols
        .saturating_sub(acp_prefix_columns(&line.prefix, acp_spinner_frame()))
        .max(1)
}

fn acp_rendered_text_segments(
    line: &AcpRenderedTextLine,
    wrap_cols: usize,
) -> Vec<LineWrapSegment> {
    let text_wrap_cols = acp_rendered_text_wrap_cols(line, wrap_cols);
    wrap_line_segments(
        &LineCharMap::new(&line.text),
        text_wrap_cols,
        text_wrap_cols,
    )
}

fn acp_rendered_line_row_count(line: &AcpRenderedLine, wrap_cols: usize) -> usize {
    match line {
        AcpRenderedLine::Text(line) => acp_rendered_text_segments(line, wrap_cols).len().max(1),
        AcpRenderedLine::Image(image) => image.rows.max(1),
        AcpRenderedLine::ImageContinuation | AcpRenderedLine::Spacer => 1,
    }
}

fn acp_pane_content_rows(pane: &AcpPaneState, wrap_cols: usize) -> usize {
    pane.render_lines
        .iter()
        .map(|line| acp_rendered_line_row_count(line, wrap_cols))
        .sum()
}

fn acp_text_segment(text: impl Into<String>, role: AcpColorRole) -> AcpRenderedSegment {
    AcpRenderedSegment {
        text: text.into(),
        role,
        animate: false,
    }
}

fn acp_spinner_segment(role: AcpColorRole) -> AcpRenderedSegment {
    AcpRenderedSegment {
        text: editor_icons::symbols::fa::FA_SPINNER.to_owned(),
        role,
        animate: true,
    }
}

#[derive(Debug, Clone)]
struct WrapRowCache {
    wrap_cols: usize,
    indent_size: usize,
    line_count: usize,
    prefix_rows: Vec<usize>,
}

impl WrapRowCache {
    fn build(buffer: &ShellBuffer, wrap_cols: usize, indent_size: usize) -> Self {
        let line_count = buffer.line_count();
        let mut prefix_rows: Vec<usize> = Vec::with_capacity(line_count + 1);
        prefix_rows.push(0);
        for line_index in 0..line_count {
            let line = buffer.text.line(line_index).unwrap_or_default();
            let rows = line_wrap_row_count(&line, wrap_cols, indent_size);
            let next = prefix_rows
                .last()
                .copied()
                .unwrap_or(0)
                .saturating_add(rows);
            prefix_rows.push(next);
        }
        Self {
            wrap_cols,
            indent_size,
            line_count,
            prefix_rows,
        }
    }

    fn matches(&self, wrap_cols: usize, indent_size: usize, line_count: usize) -> bool {
        self.wrap_cols == wrap_cols
            && self.indent_size == indent_size
            && self.line_count == line_count
    }
}

impl ShellBuffer {
    fn from_runtime_buffer(
        buffer: &Buffer,
        lines: Vec<String>,
        user_library: &dyn UserLibrary,
    ) -> Self {
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        let undo_tree = UndoTree::new(&text);
        let (read_only, input) = buffer_interaction(buffer.kind(), user_library);
        let plugin_section_state = plugin_section_state_for_kind(buffer.kind(), user_library);
        let browser_state = browser_state_for_kind(buffer.kind(), user_library);
        let vim_target = default_vim_target(input.is_some() || browser_state.is_some());

        Self {
            id: buffer.id(),
            name: buffer.name().to_owned(),
            kind: buffer.kind().clone(),
            read_only,
            input,
            section_state: None,
            plugin_section_state,
            image_state: None,
            pdf_state: None,
            acp_state: None,
            git_snapshot: None,
            git_view: None,
            git_fringe: None,
            git_fringe_dirty: false,
            git_fringe_last_edit_at: None,
            browser_state,
            directory_state: None,
            terminal_render: None,
            text,
            backing_file_fingerprint: None,
            backing_file_reload_pending: false,
            backing_file_check_in_flight: false,
            undo_tree,
            language_id: None,
            scroll_row: 0,
            viewport_lines: 1,
            wrap_cache: None,
            context_overlay_cache: Arc::new(Mutex::new(None)),
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            syntax_requested_revision: None,
            syntax_requested_window: None,
            syntax_applied_window: None,
            lsp_enabled: true,
            lsp_diagnostics: Vec::new(),
            lsp_diagnostic_lines: BTreeMap::new(),
            lsp_diagnostics_revision: 0,
            last_edit_at: None,
            vim_buffer_state: VimBufferState {
                target: vim_target,
                ..VimBufferState::default()
            },
        }
    }

    fn from_text_buffer(buffer: &Buffer, text: TextBuffer, user_library: &dyn UserLibrary) -> Self {
        let undo_tree = UndoTree::new(&text);
        let (read_only, input) = buffer_interaction(buffer.kind(), user_library);
        let plugin_section_state = plugin_section_state_for_kind(buffer.kind(), user_library);
        let browser_state = browser_state_for_kind(buffer.kind(), user_library);
        let vim_target = default_vim_target(input.is_some() || browser_state.is_some());
        let git_fringe = if matches!(buffer.kind(), BufferKind::File) && text.path().is_some() {
            Some(GitFringeState::new())
        } else {
            None
        };
        let git_fringe_dirty = git_fringe.is_some();
        let git_fringe_last_edit_at = git_fringe_dirty.then(Instant::now);
        let backing_file_fingerprint = text
            .path()
            .and_then(|path| BackingFileFingerprint::read(path).ok());
        Self {
            id: buffer.id(),
            name: buffer.name().to_owned(),
            kind: buffer.kind().clone(),
            read_only,
            input,
            section_state: None,
            plugin_section_state,
            image_state: None,
            pdf_state: None,
            acp_state: None,
            git_snapshot: None,
            git_view: None,
            git_fringe,
            git_fringe_dirty,
            git_fringe_last_edit_at,
            browser_state,
            directory_state: None,
            terminal_render: None,
            text,
            backing_file_fingerprint,
            backing_file_reload_pending: false,
            backing_file_check_in_flight: false,
            undo_tree,
            language_id: None,
            scroll_row: 0,
            viewport_lines: 1,
            wrap_cache: None,
            context_overlay_cache: Arc::new(Mutex::new(None)),
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            syntax_requested_revision: None,
            syntax_requested_window: None,
            syntax_applied_window: None,
            lsp_enabled: true,
            lsp_diagnostics: Vec::new(),
            lsp_diagnostic_lines: BTreeMap::new(),
            lsp_diagnostics_revision: 0,
            last_edit_at: None,
            vim_buffer_state: VimBufferState {
                target: vim_target,
                ..VimBufferState::default()
            },
        }
    }

    fn placeholder(
        buffer_id: BufferId,
        name: &str,
        kind: BufferKind,
        user_library: &dyn UserLibrary,
    ) -> Self {
        let lines = placeholder_lines(name, &kind, user_library);
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        let undo_tree = UndoTree::new(&text);
        let (read_only, input) = buffer_interaction(&kind, user_library);
        let browser_state = browser_state_for_kind(&kind, user_library);
        let plugin_section_state = plugin_section_state_for_kind(&kind, user_library);
        let vim_target = default_vim_target(input.is_some() || browser_state.is_some());

        Self {
            id: buffer_id,
            name: name.to_owned(),
            kind,
            read_only,
            input,
            section_state: None,
            plugin_section_state,
            image_state: None,
            pdf_state: None,
            acp_state: None,
            git_snapshot: None,
            git_view: None,
            git_fringe: None,
            git_fringe_dirty: false,
            git_fringe_last_edit_at: None,
            browser_state,
            directory_state: None,
            terminal_render: None,
            text,
            backing_file_fingerprint: None,
            backing_file_reload_pending: false,
            backing_file_check_in_flight: false,
            undo_tree,
            language_id: None,
            scroll_row: 0,
            viewport_lines: 1,
            wrap_cache: None,
            context_overlay_cache: Arc::new(Mutex::new(None)),
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            syntax_requested_revision: None,
            syntax_requested_window: None,
            syntax_applied_window: None,
            lsp_enabled: true,
            lsp_diagnostics: Vec::new(),
            lsp_diagnostic_lines: BTreeMap::new(),
            lsp_diagnostics_revision: 0,
            last_edit_at: None,
            vim_buffer_state: VimBufferState {
                target: vim_target,
                ..VimBufferState::default()
            },
        }
    }

    pub(crate) fn id(&self) -> BufferId {
        self.id
    }

    pub(crate) fn display_name(&self) -> &str {
        &self.name
    }

    fn context_overlay_snapshot(
        &self,
        user_library: &dyn UserLibrary,
        scrolloff: usize,
    ) -> BufferContextOverlaySnapshot {
        let key = BufferContextOverlayCacheKey {
            buffer_revision: self.text.revision(),
            buffer_name: self.display_name().to_owned(),
            language_id: self.language_id.clone(),
            viewport_top_line: self.scroll_row.saturating_add(scrolloff),
            cursor_line: self.cursor_row(),
            cursor_column: self.cursor_col(),
        };
        if let Ok(cache) = self.context_overlay_cache.lock()
            && let Some(snapshot) = cache
                .as_ref()
                .filter(|snapshot| snapshot.key == key)
                .cloned()
        {
            return snapshot;
        }

        let buffer_text = self.text.text();
        let buffer_name = key.buffer_name.clone();
        let language_id = key.language_id.clone();
        let context = HostGhostTextContext {
            buffer_id: self.id().get(),
            buffer_revision: key.buffer_revision,
            buffer_name: &buffer_name,
            language_id: language_id.as_deref(),
            buffer_text: &buffer_text,
            viewport_top_line: key.viewport_top_line,
            cursor_line: key.cursor_line,
            cursor_column: key.cursor_column,
        };
        let snapshot = BufferContextOverlaySnapshot {
            key,
            headerline_lines: user_library.headerline_lines(&context),
            ghost_text_by_line: user_library
                .ghost_text_lines(&context)
                .into_iter()
                .map(|line| (line.line, line.text))
                .collect(),
        };
        if let Ok(mut cache) = self.context_overlay_cache.lock() {
            *cache = Some(snapshot.clone());
        }
        snapshot
    }

    fn is_read_only(&self) -> bool {
        self.read_only
            || self
                .plugin_section_state
                .as_ref()
                .is_some_and(|state| !state.active_section_writable())
            || self.acp_active_pane_is_read_only()
            || self.browser_active_pane_is_read_only()
            || (self.kind == BufferKind::Image && !self.is_svg_source_mode())
    }

    fn has_input_field(&self) -> bool {
        self.input_field().is_some()
    }

    fn has_plugin_sections(&self) -> bool {
        self.plugin_section_state.is_some()
    }

    #[cfg(test)]
    fn plugin_active_section_index(&self) -> Option<usize> {
        self.plugin_section_state
            .as_ref()
            .map(|state| state.active_section)
    }

    fn plugin_attached_pane_state(&self) -> Option<&PluginTextPaneState> {
        self.plugin_section_state
            .as_ref()
            .and_then(PluginSectionBufferState::active_attached_section)
    }

    fn plugin_attached_pane_state_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        self.plugin_section_state
            .as_mut()
            .and_then(PluginSectionBufferState::active_attached_section_mut)
    }

    fn plugin_switch_pane(&mut self) -> bool {
        let Some(state) = self.plugin_section_state.as_mut() else {
            return false;
        };
        if state.section_count() <= 1 {
            return false;
        }
        state.active_section = (state.active_section + 1) % state.section_count();
        true
    }

    fn plugin_sections(&self) -> Option<&PluginSectionBufferState> {
        self.plugin_section_state.as_ref()
    }

    fn image_state(&self) -> Option<&ImageBufferState> {
        self.image_state.as_ref()
    }

    fn image_state_mut(&mut self) -> Option<&mut ImageBufferState> {
        self.image_state.as_mut()
    }

    fn pdf_state(&self) -> Option<&PdfBufferState> {
        self.pdf_state.as_ref()
    }

    fn pdf_state_mut(&mut self) -> Option<&mut PdfBufferState> {
        self.pdf_state.as_mut()
    }

    fn is_pdf_buffer(&self) -> bool {
        self.pdf_state.is_some()
            || matches!(&self.kind, BufferKind::Plugin(kind) if kind == PDF_BUFFER_KIND)
    }

    fn is_rendered_image_buffer(&self) -> bool {
        self.image_state()
            .is_some_and(|state| state.mode == ImageBufferMode::Rendered)
    }

    fn is_svg_source_mode(&self) -> bool {
        self.image_state().is_some_and(|state| {
            state.format == ImageBufferFormat::Svg && state.mode == ImageBufferMode::Source
        })
    }

    fn supports_text_file_actions(&self) -> bool {
        self.kind == BufferKind::File || self.is_svg_source_mode()
    }

    fn set_image_state(&mut self, state: ImageBufferState) {
        self.image_state = Some(state);
    }

    fn set_pdf_state(&mut self, state: PdfBufferState) {
        self.pdf_state = Some(state);
    }

    fn image_zoom_in(&mut self) -> bool {
        let Some(state) = self.image_state_mut() else {
            return false;
        };
        if state.mode != ImageBufferMode::Rendered {
            return false;
        }
        let next = (state.zoom * IMAGE_ZOOM_STEP).clamp(IMAGE_ZOOM_MIN, IMAGE_ZOOM_MAX);
        if (next - state.zoom).abs() < f32::EPSILON {
            return false;
        }
        state.zoom = next;
        true
    }

    fn image_zoom_out(&mut self) -> bool {
        let Some(state) = self.image_state_mut() else {
            return false;
        };
        if state.mode != ImageBufferMode::Rendered {
            return false;
        }
        let next = (state.zoom / IMAGE_ZOOM_STEP).clamp(IMAGE_ZOOM_MIN, IMAGE_ZOOM_MAX);
        if (next - state.zoom).abs() < f32::EPSILON {
            return false;
        }
        state.zoom = next;
        true
    }

    fn reset_image_zoom(&mut self) -> bool {
        let Some(state) = self.image_state_mut() else {
            return false;
        };
        if state.mode != ImageBufferMode::Rendered || (state.zoom - 1.0).abs() < f32::EPSILON {
            return false;
        }
        state.zoom = 1.0;
        true
    }

    fn refresh_pdf_preview(&mut self) {
        let Some(state) = self.pdf_state().cloned() else {
            return;
        };
        let display_name = self.display_name().to_owned();
        let path = self.path().map(Path::to_path_buf);
        self.replace_with_lines_preserve_view(pdf_buffer_lines(
            display_name.as_str(),
            path.as_deref(),
            &state,
        ));
        if let Some(path) = path {
            self.text.set_path(path);
        }
        self.text.mark_clean();
    }

    fn pdf_next_page(&mut self) -> bool {
        let Some(state) = self.pdf_state_mut() else {
            return false;
        };
        let page_count = state.page_count();
        if state.current_page >= page_count {
            return false;
        }
        state.current_page += 1;
        self.refresh_pdf_preview();
        true
    }

    fn pdf_previous_page(&mut self) -> bool {
        let Some(state) = self.pdf_state_mut() else {
            return false;
        };
        if state.current_page <= 1 {
            return false;
        }
        state.current_page -= 1;
        self.refresh_pdf_preview();
        true
    }

    fn pdf_rotate_clockwise(&mut self) -> Result<bool, String> {
        let Some(state) = self.pdf_state_mut() else {
            return Ok(false);
        };
        let page_id = state
            .document
            .get_pages()
            .get(&state.current_page)
            .copied()
            .ok_or_else(|| format!("missing page {}", state.current_page))?;
        let page = state
            .document
            .get_dictionary_mut(page_id)
            .map_err(|error| error.to_string())?;
        let next_rotation = page
            .get(b"Rotate")
            .ok()
            .and_then(|rotation| rotation.as_i64().ok())
            .unwrap_or(0)
            + 90;
        page.set("Rotate", next_rotation.rem_euclid(PDF_ROTATION_FULL_CIRCLE));
        state.dirty = true;
        self.refresh_pdf_preview();
        Ok(true)
    }

    fn pdf_delete_current_page(&mut self) -> Result<bool, String> {
        let Some(state) = self.pdf_state_mut() else {
            return Ok(false);
        };
        if state.page_count() <= 1 {
            return Err("cannot delete the last remaining PDF page".to_owned());
        }
        state.document.delete_pages(&[state.current_page]);
        state.document.prune_objects();
        if state.current_page > state.page_count() {
            state.current_page = state.page_count().max(1);
        }
        state.metadata.page_count = state.page_count();
        state.dirty = true;
        self.refresh_pdf_preview();
        Ok(true)
    }

    fn toggle_svg_image_mode(&mut self) -> Result<bool, String> {
        let Some(state) = self.image_state.as_mut() else {
            return Ok(false);
        };
        if state.format != ImageBufferFormat::Svg {
            return Ok(false);
        }
        match state.mode {
            ImageBufferMode::Rendered => {
                state.mode = ImageBufferMode::Source;
                Ok(true)
            }
            ImageBufferMode::Source => {
                let path = self.text.path().map(Path::to_path_buf);
                let decoded = rasterize_svg_text(&self.text.text(), path.as_deref())?;
                state.decoded = decoded;
                state.mode = ImageBufferMode::Rendered;
                Ok(true)
            }
        }
    }

    fn set_plugin_output_lines(&mut self, lines: Vec<String>) {
        let Some((target_section, base_update)) = self
            .plugin_section_state
            .as_ref()
            .map(|state| (state.evaluate_target_section, state.base_update))
        else {
            return;
        };
        if target_section == 0 {
            let follow_output = self.should_follow_output();
            match base_update {
                PluginBufferSectionUpdate::Replace => self.replace_with_lines(lines),
                PluginBufferSectionUpdate::Append => self.append_output_lines(&lines),
            }
            if follow_output {
                self.scroll_output_to_end();
            }
            return;
        }
        let Some(state) = self.plugin_section_state.as_mut() else {
            return;
        };
        let Some(pane) = state.attached_section_mut(target_section) else {
            return;
        };
        let follow_output = pane.should_follow_output();
        match pane.update {
            PluginBufferSectionUpdate::Replace => pane.replace_lines(lines, follow_output),
            PluginBufferSectionUpdate::Append => pane.append_lines(lines, follow_output),
        }
    }

    fn is_acp_buffer(&self) -> bool {
        self.acp_state.is_some()
    }

    pub(crate) fn init_acp_view(&mut self, client_label: &str) {
        self.text = TextBuffer::new();
        self.undo_tree = UndoTree::new(&self.text);
        self.scroll_row = 0;
        self.wrap_cache = None;
        self.acp_state = Some(AcpBufferState::new(client_label.to_owned()));
        self.acp_push_system_message(format!(
            "{} Connected to {client_label}.",
            editor_icons::symbols::cod::COD_ROCKET
        ));
    }

    pub(crate) fn acp_switch_pane(&mut self) -> bool {
        let Some(state) = self.acp_state.as_mut() else {
            return false;
        };
        state.active_pane = match state.active_pane {
            AcpPane::Plan => AcpPane::Output,
            AcpPane::Output => AcpPane::Input,
            AcpPane::Input => AcpPane::Footer,
            AcpPane::Footer => AcpPane::Plan,
        };
        true
    }

    fn focus_acp_input(&mut self) -> bool {
        let Some(state) = self.acp_state.as_mut() else {
            return false;
        };
        state.active_pane = AcpPane::Input;
        true
    }

    fn acp_active_pane(&self) -> Option<AcpPane> {
        self.acp_state.as_ref().map(|state| state.active_pane)
    }

    fn acp_plan_viewport_lines(&self) -> usize {
        self.acp_state
            .as_ref()
            .map(|state| state.plan_pane.visible_rows())
            .unwrap_or(1)
    }

    fn acp_output_viewport_lines(&self) -> usize {
        self.acp_state
            .as_ref()
            .map(|state| state.output_pane.visible_rows())
            .unwrap_or(1)
    }

    fn acp_active_pane_state(&self) -> Option<&AcpPaneState> {
        let state = self.acp_state.as_ref()?;
        Some(match state.active_pane {
            AcpPane::Plan => &state.plan_pane,
            AcpPane::Output => &state.output_pane,
            AcpPane::Input | AcpPane::Footer => return None,
        })
    }

    fn current_scroll_row(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.scroll_row;
        }
        self.acp_active_pane_state()
            .map(|pane| pane.scroll_row)
            .unwrap_or(self.scroll_row)
    }

    fn acp_active_pane_state_mut(&mut self) -> Option<&mut AcpPaneState> {
        let state = self.acp_state.as_mut()?;
        Some(match state.active_pane {
            AcpPane::Plan => &mut state.plan_pane,
            AcpPane::Output => &mut state.output_pane,
            AcpPane::Input | AcpPane::Footer => return None,
        })
    }

    fn acp_footer_pane(&self) -> Option<&PluginTextPaneState> {
        self.acp_state.as_ref().map(|state| &state.footer_pane)
    }

    fn acp_footer_pane_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        self.acp_state.as_mut().map(|state| &mut state.footer_pane)
    }

    fn browser_active_pane(&self) -> Option<BrowserPane> {
        self.browser_state.as_ref().map(|state| state.active_pane)
    }

    fn focus_browser_input(&mut self) -> bool {
        let Some(state) = self.browser_state.as_mut() else {
            return false;
        };
        state.active_pane = BrowserPane::Input;
        true
    }

    fn acp_active_pane_is_read_only(&self) -> bool {
        matches!(
            self.acp_active_pane(),
            Some(AcpPane::Plan | AcpPane::Output | AcpPane::Footer)
        )
    }

    fn browser_active_pane_is_read_only(&self) -> bool {
        matches!(self.browser_active_pane(), Some(BrowserPane::Footer))
    }

    fn browser_footer_pane(&self) -> Option<&PluginTextPaneState> {
        self.browser_state.as_ref().map(|state| &state.footer_pane)
    }

    fn browser_footer_pane_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        self.browser_state
            .as_mut()
            .map(|state| &mut state.footer_pane)
    }

    fn active_aux_text_pane_state(&self) -> Option<&PluginTextPaneState> {
        if let Some(pane) = self.plugin_attached_pane_state() {
            return Some(pane);
        }
        if matches!(self.acp_active_pane(), Some(AcpPane::Footer)) {
            return self.acp_footer_pane();
        }
        if matches!(self.browser_active_pane(), Some(BrowserPane::Footer)) {
            return self.browser_footer_pane();
        }
        None
    }

    fn active_aux_text_pane_state_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        if self
            .plugin_section_state
            .as_ref()
            .is_some_and(PluginSectionBufferState::has_active_attached_section)
        {
            return self.plugin_attached_pane_state_mut();
        }
        if matches!(self.acp_active_pane(), Some(AcpPane::Footer)) {
            return self.acp_footer_pane_mut();
        }
        if matches!(self.browser_active_pane(), Some(BrowserPane::Footer)) {
            return self.browser_footer_pane_mut();
        }
        None
    }

    pub(crate) fn acp_push_user_prompt(&mut self, prompt: impl Into<String>) {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            state
                .output_items
                .push(AcpOutputItem::UserPrompt(prompt.into()));
        }
        self.acp_rebuild_output_view(follow_output);
    }

    pub(crate) fn acp_push_system_message(&mut self, message: impl Into<String>) {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            state
                .output_items
                .push(AcpOutputItem::SystemMessage(message.into()));
        }
        self.acp_rebuild_output_view(follow_output);
    }

    pub(crate) fn acp_append_agent_chunk(&mut self, content: ContentBlock) {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            match state.output_items.last_mut() {
                Some(AcpOutputItem::AgentBlocks(blocks)) => match (blocks.last_mut(), content) {
                    (Some(ContentBlock::Text(existing)), ContentBlock::Text(text)) => {
                        existing.text.push_str(&text.text);
                    }
                    (_, content) => blocks.push(content),
                },
                _ => state
                    .output_items
                    .push(AcpOutputItem::AgentBlocks(vec![content])),
            }
        }
        self.acp_rebuild_output_view(follow_output);
    }

    pub(crate) fn acp_set_plan(&mut self, plan: Plan) {
        if let Some(state) = self.acp_state.as_mut() {
            state.plan_entries = plan.entries;
        }
        self.acp_rebuild_plan_view();
    }

    pub(crate) fn acp_set_session_info(&mut self, update: &SessionInfoUpdate) {
        if let Some(state) = self.acp_state.as_mut() {
            match &update.title {
                MaybeUndefined::Value(title) => state.session_title = Some(title.clone()),
                MaybeUndefined::Null => state.session_title = None,
                MaybeUndefined::Undefined => {}
            }
        }
    }

    pub(crate) fn acp_upsert_tool_call(&mut self, tool_call: ToolCall) {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            let tool_key = tool_call.tool_call_id.to_string();
            if let Some(index) = state.tool_item_indices.get(tool_key.as_str()).copied() {
                state.output_items[index] = AcpOutputItem::ToolCall(tool_call);
            } else {
                let index = state.output_items.len();
                state.tool_item_indices.insert(tool_key, index);
                state.output_items.push(AcpOutputItem::ToolCall(tool_call));
            }
        }
        self.acp_rebuild_output_view(follow_output);
    }

    pub(crate) fn acp_update_tool_call(&mut self, update: ToolCallUpdate) {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            let tool_key = update.tool_call_id.to_string();
            if let Some(index) = state.tool_item_indices.get(tool_key.as_str()).copied() {
                if let Some(AcpOutputItem::ToolCall(tool_call)) = state.output_items.get_mut(index)
                {
                    tool_call.update(update.fields.clone());
                }
            } else {
                let tool_call = ToolCall::try_from(update.clone())
                    .unwrap_or_else(|_| acp_tool_call_from_partial_update(&update));
                let index = state.output_items.len();
                state.tool_item_indices.insert(tool_key, index);
                state.output_items.push(AcpOutputItem::ToolCall(tool_call));
            }
        }
        self.acp_rebuild_output_view(follow_output);
    }

    fn acp_rebuild_plan_view(&mut self) {
        let visible_rows = self.acp_plan_viewport_lines();
        let Some(state) = self.acp_state.as_mut() else {
            return;
        };
        let render_lines = acp_build_plan_lines(&state.plan_entries);
        state
            .plan_pane
            .replace_render_lines(render_lines, false, visible_rows);
    }

    fn acp_rebuild_output_view(&mut self, follow_output: bool) {
        let visible_rows = self.acp_output_viewport_lines();
        let Some(state) = self.acp_state.as_mut() else {
            return;
        };
        let render_lines = acp_build_output_lines(&state.output_items);
        state
            .output_pane
            .replace_render_lines(render_lines, follow_output, visible_rows);
    }

    fn input_field(&self) -> Option<&InputField> {
        self.standalone_input_field()
            .or_else(|| self.acp_state.as_ref().map(|state| &state.input))
            .or_else(|| self.browser_state.as_ref().map(|state| &state.input))
    }

    fn input_field_mut(&mut self) -> Option<&mut InputField> {
        if let Some(input) = self.input.as_mut() {
            return Some(input);
        }
        if let Some(state) = self.acp_state.as_mut() {
            return Some(&mut state.input);
        }
        self.browser_state.as_mut().map(|state| &mut state.input)
    }

    fn standalone_input_field(&self) -> Option<&InputField> {
        self.input.as_ref()
    }

    fn clear_input(&mut self) -> bool {
        if let Some(input) = self.input_field_mut() {
            input.clear();
            return true;
        }
        false
    }

    fn section_state(&self) -> Option<&SectionedBufferState> {
        self.section_state.as_ref()
    }

    fn ensure_section_state(&mut self) -> &mut SectionedBufferState {
        self.section_state
            .get_or_insert_with(SectionedBufferState::default)
    }

    fn section_line_meta(&self, line_index: usize) -> Option<&SectionLineMeta> {
        self.section_state
            .as_ref()
            .and_then(|state| state.lines.get(line_index))
    }

    fn git_snapshot(&self) -> Option<&GitStatusSnapshot> {
        self.git_snapshot.as_ref()
    }

    fn set_git_snapshot(&mut self, snapshot: GitStatusSnapshot) {
        self.git_snapshot = Some(snapshot);
    }

    fn git_view(&self) -> Option<&GitViewState> {
        self.git_view.as_ref()
    }

    fn set_git_view(&mut self, view: GitViewState) {
        self.git_view = Some(view);
    }

    fn git_fringe_state(&self) -> Option<&GitFringeState> {
        self.git_fringe.as_ref()
    }

    fn git_fringe_kind(&self, line_index: usize) -> Option<GitFringeKind> {
        self.git_fringe_state()
            .and_then(|state| state.try_line_kind(line_index))
    }

    fn git_fringe_revision(&self) -> Option<u64> {
        self.git_fringe_state()
            .map(GitFringeState::snapshot_revision)
    }

    fn mark_git_fringe_dirty(&mut self) {
        if matches!(self.kind, BufferKind::File) && self.git_fringe.is_some() {
            self.git_fringe_dirty = true;
            self.git_fringe_last_edit_at = Some(Instant::now());
        }
    }

    fn git_fringe_refresh_due(&self, now: Instant, typing_active: bool) -> bool {
        !typing_active
            && self.git_fringe_dirty
            && self
                .git_fringe_last_edit_at
                .map(|last| now.duration_since(last) >= GIT_FRINGE_REFRESH_DEBOUNCE)
                .unwrap_or(true)
    }

    fn clear_git_fringe_dirty(&mut self) {
        self.git_fringe_dirty = false;
    }

    fn directory_state(&self) -> Option<&DirectoryViewState> {
        self.directory_state.as_ref()
    }

    fn set_directory_state(&mut self, state: DirectoryViewState) {
        self.directory_state = Some(state);
    }

    fn clear_directory_state(&mut self) {
        self.directory_state = None;
    }

    fn terminal_render(&self) -> Option<&TerminalRenderSnapshot> {
        self.terminal_render.as_ref()
    }

    fn set_terminal_render(&mut self, snapshot: TerminalRenderSnapshot) {
        self.terminal_render = Some(snapshot);
    }

    fn clear_terminal_render(&mut self) {
        self.terminal_render = None;
    }

    fn set_section_lines(&mut self, lines: Vec<SectionRenderLine>) {
        let is_git_status = buffer_is_git_status(&self.kind);
        let mut text_lines = Vec::with_capacity(lines.len());
        let mut meta = Vec::with_capacity(lines.len());
        let mut syntax_lines = BTreeMap::new();
        for (line_index, line) in lines.into_iter().enumerate() {
            let formatted_line = format_section_line(&line);
            if is_git_status {
                let spans = git_status_line_spans(&line, &formatted_line);
                if !spans.is_empty() {
                    syntax_lines.insert(line_index, spans);
                }
            }
            text_lines.push(formatted_line);
            meta.push(SectionLineMeta {
                section_id: line.section_id,
                kind: line.kind,
                action: line.action,
            });
        }
        let state = self.ensure_section_state();
        state.lines = meta;
        self.replace_with_lines_preserve_view(text_lines);
        if is_git_status {
            self.syntax_lines = syntax_lines;
            self.syntax_dirty = false;
            self.last_edit_at = None;
        }
    }

    fn append_output_lines(&mut self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        let original_cursor = self.cursor_point();
        let original_scroll = self.scroll_row;
        let follow_output = self.should_follow_output();
        let insert_text = lines.join("\n");
        if self.line_count() == 0 {
            self.text.set_cursor(TextPoint::new(0, 0));
            self.text.insert_text(&insert_text);
        } else {
            let last_line = self.line_count().saturating_sub(1);
            let column = self.line_len_chars(last_line);
            self.text.set_cursor(TextPoint::new(last_line, column));
            self.text.insert_text(&format!("\n{insert_text}"));
        }
        self.text.mark_clean();
        self.text.set_cursor(original_cursor);
        if follow_output {
            self.scroll_output_to_end();
        } else {
            self.scroll_row = original_scroll.min(self.line_count().saturating_sub(1));
        }
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        if self.language_id.is_some() {
            self.mark_syntax_dirty();
        } else {
            self.syntax_dirty = false;
            self.last_edit_at = None;
        }
        self.invalidate_wrap_cache();
    }

    fn language_id(&self) -> Option<&str> {
        self.language_id.as_deref()
    }

    fn kind_label(&self) -> String {
        buffer_kind_label(&self.kind)
    }

    pub(crate) fn cursor_row(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.cursor().line;
        }
        self.acp_active_pane_state()
            .map(|pane| pane.cursor().line)
            .unwrap_or_else(|| self.text.cursor().line)
    }

    pub(crate) fn cursor_col(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.cursor().column;
        }
        self.acp_active_pane_state()
            .map(|pane| pane.cursor().column)
            .unwrap_or_else(|| self.text.cursor().column)
    }

    pub(crate) fn cursor_point(&self) -> TextPoint {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.cursor();
        }
        self.acp_active_pane_state()
            .map(AcpPaneState::cursor)
            .unwrap_or_else(|| self.text.cursor())
    }

    fn line_count(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.line_count();
        }
        self.acp_active_pane_state()
            .map(AcpPaneState::line_count)
            .unwrap_or_else(|| self.text.line_count())
    }

    fn line_len_chars(&self, line_index: usize) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.line_len_chars(line_index);
        }
        self.acp_active_pane_state()
            .map(|pane| pane.line_len_chars(line_index))
            .unwrap_or_else(|| self.text.line_len_chars(line_index).unwrap_or(0))
    }

    fn should_follow_output(&self) -> bool {
        if let Some(state) = self.acp_state.as_ref() {
            return state
                .output_pane
                .should_follow_output(self.acp_output_viewport_lines());
        }
        if self.line_count() == 0 {
            return true;
        }
        self.line_at_viewport_offset(self.viewport_lines().saturating_sub(1)) + 1
            >= self.line_count()
    }

    fn scroll_output_to_end(&mut self) {
        if let Some(state) = self.acp_state.as_mut() {
            state.output_pane.scroll_row = state.output_pane.max_scroll_row();
            return;
        }
        self.scroll_row = self.line_count().saturating_sub(self.viewport_lines());
    }

    fn path(&self) -> Option<&Path> {
        if self.active_aux_text_pane_state().is_some() {
            return None;
        }
        self.text.path()
    }

    fn is_dirty(&self) -> bool {
        self.text.is_dirty() || self.pdf_state().is_some_and(|state| state.dirty)
    }

    fn save_to_path(&mut self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(state) = self.pdf_state_mut() {
            state
                .document
                .save(path)
                .map_err(|error| io::Error::other(error.to_string()))?;
            state.dirty = false;
            state.metadata = PdfDocument::load_metadata(path)
                .map_err(|error| io::Error::other(error.to_string()))?;
            self.text.set_path(path.to_path_buf());
            self.backing_file_fingerprint = BackingFileFingerprint::read(path).ok();
            self.backing_file_reload_pending = false;
            self.backing_file_check_in_flight = false;
            return Ok(());
        }
        self.text.save_to_path(path)?;
        self.backing_file_fingerprint = BackingFileFingerprint::read(path).ok();
        self.backing_file_reload_pending = false;
        self.backing_file_check_in_flight = false;
        Ok(())
    }

    fn mark_backing_file_reload_pending(&mut self) {
        if (self.kind == BufferKind::File || self.is_pdf_buffer()) && self.text.path().is_some() {
            self.backing_file_reload_pending = true;
        }
    }

    fn file_reload_request(&mut self) -> Option<FileReloadWorkerRequest> {
        if self.kind != BufferKind::File
            || self.text.is_dirty()
            || self.backing_file_check_in_flight
            || !self.backing_file_reload_pending
        {
            return None;
        }
        let path = self.text.path().map(Path::to_path_buf)?;
        self.backing_file_reload_pending = false;
        self.backing_file_check_in_flight = true;
        Some(FileReloadWorkerRequest {
            buffer_id: self.id,
            buffer_revision: self.text.revision(),
            path,
            loaded_fingerprint: self.backing_file_fingerprint,
        })
    }

    fn finish_file_reload_request(&mut self) {
        self.backing_file_check_in_flight = false;
    }

    fn apply_reloaded_file_buffer(
        &mut self,
        fingerprint: BackingFileFingerprint,
        reloaded: TextBuffer,
    ) -> bool {
        self.backing_file_fingerprint = Some(fingerprint);
        if !self.text.reload_from_buffer(reloaded) {
            return false;
        }

        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.force_syntax_refresh();
        self.scroll_row = self.scroll_row.min(self.line_count().saturating_sub(1));
        self.invalidate_wrap_cache();
        if self.git_fringe.is_some() {
            self.git_fringe_dirty = true;
            self.git_fringe_last_edit_at = None;
        }
        true
    }

    fn set_syntax_snapshot(&mut self, syntax: Option<SyntaxSnapshot>) {
        let syntax_window = syntax.as_ref().and_then(|_| self.full_syntax_window());
        self.set_indexed_syntax_lines(syntax.map(index_syntax_lines), syntax_window);
    }

    fn set_indexed_syntax_lines(
        &mut self,
        syntax_lines: Option<IndexedSyntaxLines>,
        syntax_window: Option<SyntaxLineWindow>,
    ) {
        self.syntax_lines = syntax_lines.unwrap_or_default();
        self.syntax_dirty = false;
        self.syntax_requested_revision = Some(self.text.revision());
        self.syntax_requested_window = syntax_window;
        self.syntax_applied_window = syntax_window;
        self.last_edit_at = None;
    }

    fn set_language_id(&mut self, language_id: Option<String>) {
        self.language_id = language_id;
    }

    fn lsp_diagnostics(&self) -> &[LspDiagnostic] {
        &self.lsp_diagnostics
    }

    fn lsp_enabled(&self) -> bool {
        self.lsp_enabled
    }

    fn lsp_diagnostic_line_spans(&self, line_index: usize) -> &[DiagnosticLineSpan] {
        self.lsp_diagnostic_lines
            .get(&line_index)
            .map(Box::as_ref)
            .unwrap_or(&[])
    }

    fn set_lsp_enabled(&mut self, enabled: bool) {
        self.lsp_enabled = enabled;
    }

    fn lsp_diagnostics_revision(&self) -> u64 {
        self.lsp_diagnostics_revision
    }

    fn set_lsp_diagnostics(&mut self, diagnostics: Vec<LspDiagnostic>) -> bool {
        if self.lsp_diagnostics == diagnostics {
            return false;
        }
        self.lsp_diagnostic_lines = diagnostic_line_spans_for_diagnostics(&diagnostics);
        self.lsp_diagnostics = diagnostics;
        self.lsp_diagnostics_revision = self.lsp_diagnostics_revision.saturating_add(1);
        true
    }

    fn lsp_diagnostic_severity(&self, line_index: usize) -> Option<LspDiagnosticSeverity> {
        self.lsp_diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.range().start().line <= line_index)
            .filter(|diagnostic| diagnostic.range().end().line >= line_index)
            .map(LspDiagnostic::severity)
            .min_by_key(|severity| diagnostic_severity_rank(*severity))
    }

    fn invalidate_wrap_cache(&mut self) {
        self.wrap_cache = None;
    }

    fn set_syntax_error(&mut self, error: Option<String>) {
        self.syntax_error = error;
    }

    fn reload_from_disk_if_changed(&mut self, force: bool) -> Result<bool, String> {
        if !matches!(self.kind, BufferKind::File) && !self.is_pdf_buffer() {
            return Ok(false);
        }
        if self.kind == BufferKind::File && self.text.is_dirty() {
            return Ok(false);
        }
        if self.is_pdf_buffer() && self.pdf_state().is_some_and(|state| state.dirty) {
            return Ok(false);
        }
        let Some(path) = self.text.path().map(Path::to_path_buf) else {
            return Ok(false);
        };
        if !force && !self.backing_file_reload_pending {
            return Ok(false);
        }
        self.backing_file_reload_pending = false;
        self.backing_file_check_in_flight = false;

        let current_fingerprint = match BackingFileFingerprint::read(&path) {
            Ok(fingerprint) => fingerprint,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(format!("failed to stat `{}`: {error}", path.display()));
            }
        };
        let Some(loaded_fingerprint) = self.backing_file_fingerprint else {
            self.backing_file_fingerprint = Some(current_fingerprint);
            return Ok(false);
        };
        if current_fingerprint == loaded_fingerprint {
            return Ok(false);
        }

        if self.is_pdf_buffer() {
            let current_page = self
                .pdf_state()
                .map(|state| state.current_page)
                .unwrap_or(1);
            let fit_mode = self
                .pdf_state()
                .map(|state| state.fit_mode)
                .unwrap_or(PdfFitMode::Page);
            let zoom_percent = self
                .pdf_state()
                .map(|state| state.zoom_percent)
                .unwrap_or(100);
            let mut state = load_pdf_buffer_state(&path)
                .map_err(|error| format!("failed to reload `{}`: {error}", path.display()))?;
            state.current_page = current_page;
            state.fit_mode = fit_mode;
            state.zoom_percent = zoom_percent;
            state.clamp_current_page();
            self.pdf_state = Some(state);
            self.refresh_pdf_preview();
            self.backing_file_fingerprint = Some(current_fingerprint);
            self.backing_file_reload_pending = false;
            self.backing_file_check_in_flight = false;
            return Ok(true);
        }

        let reloaded = match self.text.reload_from_path() {
            Ok(reloaded) => reloaded,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(format!("failed to reload `{}`: {error}", path.display()));
            }
        };
        self.backing_file_fingerprint = Some(current_fingerprint);
        if !reloaded {
            return Ok(false);
        }

        self.finish_file_reload_request();
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.force_syntax_refresh();
        self.scroll_row = self.scroll_row.min(self.line_count().saturating_sub(1));
        self.invalidate_wrap_cache();
        if self.git_fringe.is_some() {
            self.git_fringe_dirty = true;
            self.git_fringe_last_edit_at = None;
        }
        Ok(true)
    }

    fn replace_with_lines(&mut self, lines: Vec<String>) {
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.text = text;
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.syntax_dirty = false;
        self.syntax_requested_revision = None;
        self.syntax_requested_window = None;
        self.syntax_applied_window = None;
        self.last_edit_at = None;
        self.scroll_row = 0;
        self.invalidate_wrap_cache();
    }

    fn replace_with_lines_preserve_view(&mut self, lines: Vec<String>) {
        let cursor = self.cursor_point();
        let scroll_row = self.scroll_row;
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.text = text;
        self.text.mark_clean();
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.syntax_dirty = false;
        self.syntax_requested_revision = None;
        self.syntax_requested_window = None;
        self.syntax_applied_window = None;
        self.last_edit_at = None;
        let line_count = self.line_count();
        if line_count == 0 {
            self.text.set_cursor(TextPoint::default());
            self.scroll_row = 0;
            return;
        }
        let line = cursor.line.min(line_count.saturating_sub(1));
        let column = cursor.column.min(self.line_len_chars(line));
        self.text.set_cursor(TextPoint::new(line, column));
        let max_scroll = line_count.saturating_sub(1);
        self.scroll_row = scroll_row.min(max_scroll);
        self.invalidate_wrap_cache();
    }

    fn replace_with_lines_follow_output(&mut self, lines: Vec<String>) {
        let cursor = self.cursor_point();
        let scroll_row = self.scroll_row;
        let follow_output = self.should_follow_output();
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.text = text;
        self.text.mark_clean();
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.syntax_dirty = false;
        self.syntax_requested_revision = None;
        self.syntax_requested_window = None;
        self.syntax_applied_window = None;
        self.last_edit_at = None;
        let line_count = self.line_count();
        if line_count == 0 {
            self.text.set_cursor(TextPoint::default());
            self.scroll_row = 0;
            return;
        }
        let line = cursor.line.min(line_count.saturating_sub(1));
        let column = cursor.column.min(self.line_len_chars(line));
        self.text.set_cursor(TextPoint::new(line, column));
        if follow_output {
            self.scroll_output_to_end();
        } else {
            let max_scroll = line_count.saturating_sub(1);
            self.scroll_row = scroll_row.min(max_scroll);
        }
        self.invalidate_wrap_cache();
    }

    fn mark_syntax_dirty(&mut self) {
        if self.kind == BufferKind::File || self.language_id.is_some() {
            self.syntax_dirty = true;
            self.syntax_requested_window = None;
            self.syntax_applied_window = None;
            self.last_edit_at = Some(Instant::now());
            self.mark_git_fringe_dirty();
        }
    }

    fn force_syntax_refresh(&mut self) {
        if self.kind == BufferKind::File || self.language_id.is_some() {
            self.syntax_dirty = true;
            self.syntax_requested_revision = None;
            self.syntax_requested_window = None;
            self.syntax_applied_window = None;
            self.last_edit_at = None;
        }
    }

    fn mark_syntax_refresh_requested(&mut self, syntax_window: Option<SyntaxLineWindow>) {
        self.syntax_requested_revision = Some(self.text.revision());
        self.syntax_requested_window = syntax_window;
    }

    fn syntax_refresh_due(&self, now: Instant) -> bool {
        const SYNTAX_REFRESH_COLD_DEBOUNCE: Duration = Duration::from_millis(75);
        const SYNTAX_REFRESH_INCREMENTAL_DEBOUNCE: Duration = Duration::from_millis(8);
        let debounce = if self.syntax_applied_window.is_some() {
            SYNTAX_REFRESH_INCREMENTAL_DEBOUNCE
        } else {
            SYNTAX_REFRESH_COLD_DEBOUNCE
        };
        self.syntax_dirty
            && self.syntax_requested_revision != Some(self.text.revision())
            && self
                .last_edit_at
                .map(|last_edit_at| now.duration_since(last_edit_at) >= debounce)
                .unwrap_or(true)
    }

    fn line_syntax_spans(&self, line_index: usize) -> Option<&[LineSyntaxSpan]> {
        self.syntax_lines.get(&line_index).map(Vec::as_slice)
    }

    fn full_syntax_window(&self) -> Option<SyntaxLineWindow> {
        SyntaxLineWindow::new(0, self.line_count())
    }

    fn desired_syntax_window(&self) -> Option<SyntaxLineWindow> {
        if self.kind != BufferKind::File && self.language_id.is_none() {
            return None;
        }
        let line_count = self.line_count();
        if line_count == 0 {
            return None;
        }
        let visible_lines = self.viewport_lines();
        let target_lines = visible_lines
            .saturating_add(SYNTAX_WINDOW_MARGIN_LINES.saturating_mul(2))
            .max(SYNTAX_WINDOW_MIN_LINES)
            .min(line_count);
        let centered_margin = target_lines.saturating_sub(visible_lines) / 2;
        let max_start_line = line_count.saturating_sub(target_lines);
        let start_line = self
            .scroll_row
            .saturating_sub(centered_margin)
            .min(max_start_line);
        SyntaxLineWindow::new(start_line, target_lines)
    }

    fn ensure_visible_syntax_window(&mut self) {
        let Some(desired_window) = self.desired_syntax_window() else {
            return;
        };
        let current_revision = self.text.revision();
        let applied_matches = self.syntax_requested_revision == Some(current_revision)
            && self
                .syntax_applied_window
                .map(|window| window.contains(desired_window))
                .unwrap_or(false);
        let requested_matches = self.syntax_requested_revision == Some(current_revision)
            && self
                .syntax_requested_window
                .map(|window| window.contains(desired_window))
                .unwrap_or(false);
        if applied_matches || requested_matches {
            return;
        }
        self.syntax_dirty = true;
        self.syntax_requested_revision = None;
        self.syntax_requested_window = None;
        self.last_edit_at = None;
    }

    fn insert_text(&mut self, text: &str) {
        self.text.insert_text(text);
        self.invalidate_wrap_cache();
    }

    fn replace_mode_text(&mut self, text: &str) {
        let mut changed = false;
        for character in text.chars() {
            if character == '\n' {
                self.text.insert_newline();
                changed = true;
                continue;
            }

            let point = self.cursor_point();
            let Some(next) = self.point_after(point) else {
                self.text.insert_text(&character.to_string());
                changed = true;
                continue;
            };

            let current = self.slice(TextRange::new(point, next));
            if current == "\n" {
                self.text.insert_text(&character.to_string());
                changed = true;
            } else {
                self.text
                    .replace(TextRange::new(point, next), &character.to_string());
                changed = true;
            }
        }
        if changed {
            self.invalidate_wrap_cache();
        }
    }

    fn backspace(&mut self) {
        let _ = self.text.backspace();
        self.invalidate_wrap_cache();
    }

    fn delete_forward(&mut self) {
        let _ = self.text.delete_forward();
        self.invalidate_wrap_cache();
    }

    fn move_left(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_left();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_left();
        }
        self.text.move_left()
    }

    fn move_right(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_right();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_right();
        }
        self.text.move_right()
    }

    fn move_up(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_up();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_up();
        }
        self.text.move_up()
    }

    fn move_down(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_down();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_down();
        }
        self.text.move_down()
    }

    fn move_word_forward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_word_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_word_forward();
        }
        self.text.move_word_forward()
    }

    fn move_big_word_forward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_big_word_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_big_word_forward();
        }
        self.text.move_big_word_forward()
    }

    fn move_word_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_word_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_word_backward();
        }
        self.text.move_word_backward()
    }

    fn move_big_word_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_big_word_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_big_word_backward();
        }
        self.text.move_big_word_backward()
    }

    fn move_word_end(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_word_end_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_word_end_forward();
        }
        self.text.move_word_end_forward()
    }

    fn move_big_word_end(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_big_word_end_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_big_word_end_forward();
        }
        self.text.move_big_word_end_forward()
    }

    fn move_word_end_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_word_end_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_word_end_backward();
        }
        self.text.move_word_end_backward()
    }

    fn move_big_word_end_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_big_word_end_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_big_word_end_backward();
        }
        self.text.move_big_word_end_backward()
    }

    fn move_matching_delimiter(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_matching_delimiter();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_matching_delimiter();
        }
        self.text.move_matching_delimiter()
    }

    fn move_sentence_forward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_sentence_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_sentence_forward();
        }
        self.text.move_sentence_forward()
    }

    fn move_sentence_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_sentence_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_sentence_backward();
        }
        self.text.move_sentence_backward()
    }

    fn move_paragraph_forward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_paragraph_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_paragraph_forward();
        }
        self.text.move_paragraph_forward()
    }

    fn move_paragraph_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_paragraph_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_paragraph_backward();
        }
        self.text.move_paragraph_backward()
    }

    pub(crate) fn set_cursor(&mut self, point: TextPoint) {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            pane.set_cursor(point);
            return;
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            pane.set_cursor(point);
            return;
        }
        self.text.set_cursor(point);
    }

    fn point_after(&self, point: TextPoint) -> Option<TextPoint> {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.text.point_after(point);
        }
        self.acp_active_pane_state()
            .map(|pane| pane.text.point_after(point))
            .unwrap_or_else(|| self.text.point_after(point))
    }

    fn move_line_start(&mut self) -> bool {
        let before = self.cursor_point();
        self.set_cursor(editor_buffer::TextPoint::new(self.cursor_row(), 0));
        self.cursor_point() != before
    }

    fn move_line_first_non_blank(&mut self) -> bool {
        let before = self.cursor_point();
        let row = self.cursor_row();
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(row) {
                pane.text.set_cursor(point);
            }
        } else if let Some(pane) = self.acp_active_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(row) {
                pane.text.set_cursor(point);
            }
        } else if let Some(point) = self.text.first_non_blank_in_line(row) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn move_line_end(&mut self) -> bool {
        let before = self.cursor_point();
        let line = self.cursor_row();
        let column = self.line_len_chars(line).saturating_sub(1);
        self.set_cursor(editor_buffer::TextPoint::new(line, column));
        self.cursor_point() != before
    }

    fn goto_first_line(&mut self) -> bool {
        let before = self.cursor_point();
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(0) {
                pane.text.set_cursor(point);
            }
        } else if let Some(pane) = self.acp_active_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(0) {
                pane.text.set_cursor(point);
            }
        } else if let Some(point) = self.text.first_non_blank_in_line(0) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn goto_last_line(&mut self) -> bool {
        let before = self.cursor_point();
        let line = self.line_count().saturating_sub(1);
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(line) {
                pane.text.set_cursor(point);
            }
        } else if let Some(pane) = self.acp_active_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(line) {
                pane.text.set_cursor(point);
            }
        } else if let Some(point) = self.text.first_non_blank_in_line(line) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn goto_line(&mut self, line_index: usize) -> bool {
        let before = self.cursor_point();
        let line = line_index.min(self.line_count().saturating_sub(1));
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            let point = pane
                .text
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            pane.text.set_cursor(point);
        } else if let Some(pane) = self.acp_active_pane_state_mut() {
            let point = pane
                .text
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            pane.text.set_cursor(point);
        } else {
            let point = self
                .text
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            self.text.set_cursor(point);
        }
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
        self.invalidate_wrap_cache();
    }

    fn open_line_above(&mut self) {
        let line = self.cursor_row();
        self.text.set_cursor(editor_buffer::TextPoint::new(line, 0));
        self.text.insert_newline();
        let _ = self.text.move_up();
        self.invalidate_wrap_cache();
    }

    fn undo(&mut self) {
        let _ = self.undo_tree_undo();
    }

    fn redo(&mut self) {
        let _ = self.undo_tree_redo();
    }

    fn record_undo_snapshot(&mut self) {
        let _ = self.undo_tree.record_snapshot(&self.text);
    }

    fn undo_tree_undo(&mut self) -> bool {
        let Some(snapshot) = self.undo_tree.undo() else {
            return false;
        };
        self.apply_undo_snapshot(&snapshot);
        true
    }

    fn undo_tree_redo(&mut self) -> bool {
        let Some(snapshot) = self.undo_tree.redo() else {
            return false;
        };
        self.apply_undo_snapshot(&snapshot);
        true
    }

    fn undo_tree_select(&mut self, node_id: usize) -> bool {
        let Some(snapshot) = self.undo_tree.select(node_id) else {
            return false;
        };
        self.apply_undo_snapshot(&snapshot);
        true
    }

    fn undo_tree_entries(&self) -> (Vec<UndoTreeEntry>, usize) {
        self.undo_tree.picker_entries()
    }

    fn delete_range(&mut self, range: TextRange) {
        self.text.delete(range);
        self.invalidate_wrap_cache();
    }

    fn replace_range(&mut self, range: TextRange, text: &str) {
        self.text.replace(range, text);
        self.invalidate_wrap_cache();
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
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.text.slice(range);
        }
        self.acp_active_pane_state()
            .map(|pane| pane.text.slice(range))
            .unwrap_or_else(|| self.text.slice(range))
    }

    pub(crate) fn line_range(&self, line_index: usize) -> Option<TextRange> {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.text.line_range(line_index);
        }
        self.acp_active_pane_state()
            .and_then(|pane| pane.text.line_range(line_index))
            .or_else(|| self.text.line_range(line_index))
    }

    pub(crate) fn line_span_range(&self, start_line: usize, count: usize) -> Option<TextRange> {
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

    fn full_range(&self) -> TextRange {
        if self.line_count() == 0 {
            return TextRange::new(TextPoint::default(), TextPoint::default());
        }
        let start = self.line_range(0).map(TextRange::start).unwrap_or_default();
        let end = self
            .line_range(self.line_count().saturating_sub(1))
            .map(TextRange::end)
            .unwrap_or(start);
        TextRange::new(start, end)
    }

    fn apply_undo_snapshot(&mut self, snapshot: &UndoSnapshot) {
        let range = self.full_range();
        self.replace_range(range, &snapshot.text);
        self.set_cursor(snapshot.cursor);
        self.undo_tree.update_revision(self.text.revision());
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
        self.invalidate_wrap_cache();
    }

    fn scroll_by(&mut self, delta: i32) {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            let max_scroll = pane.max_scroll_row() as i32;
            let next = (pane.scroll_row as i32 + delta).clamp(0, max_scroll);
            pane.scroll_row = next as usize;
            return;
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            let max_scroll = pane.max_scroll_row() as i32;
            let next = (pane.scroll_row as i32 + delta).clamp(0, max_scroll);
            pane.scroll_row = next as usize;
            return;
        }
        let max_scroll = self.line_count().saturating_sub(1) as i32;
        let next = (self.scroll_row as i32 + delta).clamp(0, max_scroll);
        self.scroll_row = next as usize;
    }

    pub(crate) fn set_viewport_lines(&mut self, visible_lines: usize) {
        self.viewport_lines = visible_lines.max(1);
        if let Some(state) = self.plugin_section_state.as_mut() {
            for pane in &mut state.attached_sections {
                let rows = pane.visible_rows();
                let wrap_cols = pane.wrap_cols();
                pane.set_view_metrics(rows, wrap_cols);
            }
        }
        if let Some(state) = self.acp_state.as_mut() {
            let plan_rows = state.plan_pane.visible_rows();
            let plan_wrap_cols = state.plan_pane.wrap_cols();
            state.plan_pane.set_view_metrics(plan_rows, plan_wrap_cols);
            let output_rows = state.output_pane.visible_rows();
            let output_wrap_cols = state.output_pane.wrap_cols();
            state
                .output_pane
                .set_view_metrics(output_rows, output_wrap_cols);
        }
    }

    fn sync_acp_viewport_metrics(
        &mut self,
        width: u32,
        height: u32,
        cell_width: i32,
        line_height: i32,
    ) {
        let rect = PixelRectToRect::rect(0, 0, width.max(1), height.max(1));
        let layout = buffer_footer_layout(self, rect, line_height, cell_width);
        self.viewport_lines = layout.visible_rows.max(1);
        let Some(acp_layout) = acp_buffer_layout(self, rect, layout, cell_width, line_height)
        else {
            return;
        };
        if let Some(state) = self.acp_state.as_mut() {
            state
                .plan_pane
                .set_view_metrics(acp_layout.plan.visible_rows, acp_layout.plan.wrap_cols);
            state
                .output_pane
                .set_view_metrics(acp_layout.output.visible_rows, acp_layout.output.wrap_cols);
        }
    }

    fn sync_plugin_section_viewport_metrics(
        &mut self,
        width: u32,
        height: u32,
        cell_width: i32,
        line_height: i32,
    ) {
        let rect = PixelRectToRect::rect(0, 0, width.max(1), height.max(1));
        let layout = buffer_footer_layout(self, rect, line_height, cell_width);
        let Some(section_layout) =
            plugin_section_buffer_layout(self, rect, layout, cell_width, line_height)
        else {
            return;
        };
        self.viewport_lines = section_layout
            .panes
            .first()
            .map(|pane| pane.visible_rows)
            .unwrap_or(1)
            .max(1);
        if let Some(state) = self.plugin_section_state.as_mut() {
            for (index, pane_layout) in section_layout.panes.iter().enumerate().skip(1) {
                if let Some(pane) = state.attached_section_mut(index) {
                    pane.set_view_metrics(pane_layout.visible_rows, pane_layout.wrap_cols);
                }
            }
        }
    }

    fn viewport_lines(&self) -> usize {
        if let Some(state) = self.plugin_section_state.as_ref() {
            if state.active_section == 0 {
                return self.viewport_lines.max(1);
            }
            return state
                .active_attached_section()
                .map(PluginTextPaneState::visible_rows)
                .unwrap_or(1);
        }
        match self.acp_active_pane() {
            Some(AcpPane::Plan) => self.acp_plan_viewport_lines(),
            Some(AcpPane::Output) => self.acp_output_viewport_lines(),
            Some(AcpPane::Input | AcpPane::Footer) => self.viewport_lines.max(1),
            None => self.viewport_lines.max(1),
        }
    }

    fn line_at_viewport_offset(&self, offset: usize) -> usize {
        let max_line = self.line_count().saturating_sub(1);
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.line_at_viewport_offset(offset).min(max_line);
        }
        if let Some(pane) = self.acp_active_pane_state() {
            return pane.line_at_viewport_offset(offset).min(max_line);
        }
        self.scroll_row.saturating_add(offset).min(max_line)
    }

    fn cursor_viewport_offset(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.cursor_viewport_offset();
        }
        self.acp_active_pane_state()
            .map(AcpPaneState::cursor_viewport_offset)
            .unwrap_or_else(|| self.cursor_row().saturating_sub(self.scroll_row))
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

    fn max_scroll_row_for_wrapped_rows(
        &self,
        visible_rows: usize,
        wrap_cols: usize,
        indent_size: usize,
    ) -> usize {
        let line_count = self.line_count();
        if line_count == 0 {
            return 0;
        }
        let visible_rows = visible_rows.max(1);
        let mut rows = 0usize;
        for line_index in (0..line_count).rev() {
            let line = self.text.line(line_index).unwrap_or_default();
            let row_count = line_wrap_row_count(&line, wrap_cols, indent_size);
            if rows.saturating_add(row_count) > visible_rows {
                return if rows == 0 {
                    line_index
                } else {
                    line_index.saturating_add(1)
                };
            }
            rows = rows.saturating_add(row_count);
        }
        0
    }

    fn scroll_row_for_top_margin(
        &self,
        cursor_row: usize,
        cursor_segment_index: usize,
        min_cursor_row: usize,
        wrap_cols: usize,
        indent_size: usize,
    ) -> usize {
        let mut target = cursor_row;
        let mut offset = cursor_segment_index;
        while target > 0 && offset < min_cursor_row {
            target = target.saturating_sub(1);
            let line = self.text.line(target).unwrap_or_default();
            offset = offset.saturating_add(line_wrap_row_count(&line, wrap_cols, indent_size));
        }
        target
    }

    fn ensure_visible(
        &mut self,
        visible_rows: usize,
        wrap_cols: usize,
        indent_size: usize,
        reserved_top_rows: usize,
        scrolloff: usize,
    ) {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            pane.ensure_cursor_visible();
            return;
        }
        if self.is_acp_buffer() {
            if let Some(pane) = self.acp_active_pane_state_mut() {
                pane.ensure_cursor_visible();
            }
            return;
        }
        let visible_rows = visible_rows.max(1);
        let reserved_top_rows = reserved_top_rows.min(visible_rows.saturating_sub(1));
        let content_rows = visible_rows.saturating_sub(reserved_top_rows).max(1);
        let min_cursor_row = scrolloff.min(content_rows.saturating_sub(1) / 2);
        let max_cursor_row = content_rows
            .saturating_sub(1)
            .saturating_sub(min_cursor_row);
        let cursor_row = self.cursor_row();
        if self.line_count() == 0 {
            self.scroll_row = 0;
            return;
        }
        let max_scroll_row =
            self.max_scroll_row_for_wrapped_rows(content_rows, wrap_cols, indent_size);
        self.scroll_row = self.scroll_row.min(max_scroll_row);

        let cursor_col = self.cursor_col();
        let cursor_line = self.text.line(cursor_row).unwrap_or_default();
        let cursor_segments = wrap_line_segments_for_line(&cursor_line, wrap_cols, indent_size);
        let cursor_segment_index = segment_index_for_column(&cursor_segments, cursor_col);

        let line_count = self.line_count();
        let distance = cursor_row.abs_diff(self.scroll_row);
        let threshold = content_rows.saturating_mul(4).max(256);
        let cache_valid = match self.wrap_cache.as_ref() {
            Some(cache) => cache.matches(wrap_cols, indent_size, line_count),
            None => false,
        };
        if !cache_valid {
            self.wrap_cache = None;
        }
        if self.wrap_cache.is_none() && distance >= threshold {
            self.wrap_cache = Some(WrapRowCache::build(self, wrap_cols, indent_size));
        }

        if let Some(cache) = self.wrap_cache.as_ref() {
            let base = cache
                .prefix_rows
                .get(cursor_row)
                .copied()
                .unwrap_or(0)
                .saturating_add(cursor_segment_index);
            let top_target = cache
                .prefix_rows
                .partition_point(|&value| value <= base.saturating_sub(min_cursor_row))
                .saturating_sub(1)
                .min(cursor_row)
                .min(max_scroll_row);
            let current_offset =
                base.saturating_sub(cache.prefix_rows.get(self.scroll_row).copied().unwrap_or(0));
            if cursor_row < self.scroll_row || current_offset < min_cursor_row {
                self.scroll_row = top_target;
                return;
            }
            if current_offset <= max_cursor_row {
                return;
            }
            let bottom_target = cache
                .prefix_rows
                .partition_point(|&value| value < base.saturating_sub(max_cursor_row))
                .min(cursor_row)
                .min(max_scroll_row);
            self.scroll_row = bottom_target;
            return;
        }

        if cursor_row < self.scroll_row {
            self.scroll_row = self
                .scroll_row_for_top_margin(
                    cursor_row,
                    cursor_segment_index,
                    min_cursor_row,
                    wrap_cols,
                    indent_size,
                )
                .min(max_scroll_row);
            return;
        }

        let mut row_offset = 0usize;
        let mut row_counts = Vec::with_capacity(distance);
        for line_index in self.scroll_row..cursor_row {
            let line = self.text.line(line_index).unwrap_or_default();
            let row_count = line_wrap_row_count(&line, wrap_cols, indent_size);
            row_offset = row_offset.saturating_add(row_count);
            row_counts.push(row_count);
        }
        row_offset = row_offset.saturating_add(cursor_segment_index);
        if row_offset < min_cursor_row {
            self.scroll_row = self
                .scroll_row_for_top_margin(
                    cursor_row,
                    cursor_segment_index,
                    min_cursor_row,
                    wrap_cols,
                    indent_size,
                )
                .min(max_scroll_row);
            return;
        }
        if row_offset <= max_cursor_row {
            return;
        }

        let mut offset = row_offset;
        let mut new_scroll = self.scroll_row;
        for row_count in row_counts {
            if offset <= max_cursor_row || new_scroll >= cursor_row {
                break;
            }
            offset = offset.saturating_sub(row_count);
            new_scroll = new_scroll.saturating_add(1);
        }
        self.scroll_row = new_scroll.min(max_scroll_row);
    }
}

fn acp_tool_call_from_partial_update(update: &ToolCallUpdate) -> ToolCall {
    let mut tool_call = ToolCall::new(
        update.tool_call_id.clone(),
        update
            .fields
            .title
            .clone()
            .unwrap_or_else(|| "Tool call".to_owned()),
    );
    if let Some(kind) = update.fields.kind {
        tool_call.kind = kind;
    }
    if let Some(status) = update.fields.status {
        tool_call.status = status;
    }
    if let Some(content) = update.fields.content.clone() {
        tool_call.content = content;
    }
    if let Some(locations) = update.fields.locations.clone() {
        tool_call.locations = locations;
    }
    if let Some(raw_input) = update.fields.raw_input.clone() {
        tool_call.raw_input = Some(raw_input);
    }
    if let Some(raw_output) = update.fields.raw_output.clone() {
        tool_call.raw_output = Some(raw_output);
    }
    tool_call
}

fn acp_build_plan_lines(entries: &[PlanEntry]) -> Vec<AcpRenderedLine> {
    if entries.is_empty() {
        return vec![AcpRenderedLine::Text(AcpRenderedTextLine {
            prefix: vec![acp_icon_segment(
                editor_icons::symbols::cod::COD_NOTEBOOK,
                AcpColorRole::Muted,
            )],
            text: " Waiting for plan updates...".to_owned(),
            text_role: AcpColorRole::Muted,
        })];
    }
    entries
        .iter()
        .map(|entry| {
            let mut prefix = acp_plan_status_segments(entry.status.clone(), entry.priority.clone());
            prefix.push(acp_text_segment(" ", AcpColorRole::Default));
            AcpRenderedLine::Text(AcpRenderedTextLine {
                prefix,
                text: entry.content.clone(),
                text_role: AcpColorRole::Default,
            })
        })
        .collect()
}

fn acp_build_output_lines(items: &[AcpOutputItem]) -> Vec<AcpRenderedLine> {
    let mut lines = Vec::new();
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            lines.push(AcpRenderedLine::Spacer);
        }
        match item {
            AcpOutputItem::UserPrompt(prompt) => {
                let prefix = vec![
                    acp_icon_segment(editor_icons::symbols::cod::COD_PERSON, AcpColorRole::Accent),
                    acp_text_segment(" ", AcpColorRole::Default),
                ];
                lines.extend(acp_multiline_text_lines(
                    prefix,
                    prompt,
                    AcpColorRole::Default,
                ));
            }
            AcpOutputItem::SystemMessage(message) => {
                let prefix = vec![
                    acp_icon_segment(editor_icons::symbols::cod::COD_INFO, AcpColorRole::Muted),
                    acp_text_segment(" ", AcpColorRole::Default),
                ];
                lines.extend(acp_multiline_text_lines(
                    prefix,
                    message,
                    AcpColorRole::Muted,
                ));
            }
            AcpOutputItem::AgentBlocks(blocks) => {
                for block in blocks {
                    lines.extend(acp_render_content_block(
                        block,
                        vec![
                            acp_icon_segment(
                                editor_icons::symbols::cod::COD_COMMENT,
                                AcpColorRole::Accent,
                            ),
                            acp_text_segment(" ", AcpColorRole::Default),
                        ],
                        AcpColorRole::Default,
                    ));
                }
            }
            AcpOutputItem::ToolCall(tool_call) => {
                let mut prefix = acp_status_segments(tool_call.status);
                prefix.push(acp_text_segment(" ", AcpColorRole::Default));
                prefix.push(acp_icon_segment(
                    acp_tool_kind_icon(tool_call.kind),
                    AcpColorRole::Accent,
                ));
                prefix.push(acp_text_segment(" ", AcpColorRole::Default));
                lines.push(AcpRenderedLine::Text(AcpRenderedTextLine {
                    prefix,
                    text: tool_call.title.clone(),
                    text_role: AcpColorRole::Default,
                }));
                for content in &tool_call.content {
                    lines.extend(acp_render_tool_content(content));
                }
                for location in &tool_call.locations {
                    let line = location
                        .line
                        .map(|line| format!("{}:{line}", location.path.display()))
                        .unwrap_or_else(|| location.path.display().to_string());
                    lines.push(AcpRenderedLine::Text(AcpRenderedTextLine {
                        prefix: vec![
                            acp_icon_segment(
                                editor_icons::symbols::cod::COD_SEARCH,
                                AcpColorRole::Muted,
                            ),
                            acp_text_segment(" ", AcpColorRole::Default),
                        ],
                        text: line,
                        text_role: AcpColorRole::Muted,
                    }));
                }
            }
        }
    }
    if lines.is_empty() {
        lines.push(AcpRenderedLine::Text(AcpRenderedTextLine {
            prefix: vec![acp_icon_segment(
                editor_icons::symbols::cod::COD_HISTORY,
                AcpColorRole::Muted,
            )],
            text: " Waiting for session output...".to_owned(),
            text_role: AcpColorRole::Muted,
        }));
    }
    lines
}

fn acp_render_tool_content(content: &ToolCallContent) -> Vec<AcpRenderedLine> {
    match content {
        ToolCallContent::Content(content) => acp_render_content_block(
            &content.content,
            vec![
                acp_icon_segment(
                    editor_icons::symbols::cod::COD_CHEVRON_RIGHT,
                    AcpColorRole::Muted,
                ),
                acp_text_segment(" ", AcpColorRole::Default),
            ],
            AcpColorRole::Default,
        ),
        ToolCallContent::Diff(diff) => vec![AcpRenderedLine::Text(AcpRenderedTextLine {
            prefix: vec![
                acp_icon_segment(
                    editor_icons::symbols::cod::COD_DIFF_MODIFIED,
                    AcpColorRole::Warning,
                ),
                acp_text_segment(" ", AcpColorRole::Default),
            ],
            text: diff.path.display().to_string(),
            text_role: AcpColorRole::Default,
        })],
        ToolCallContent::Terminal(terminal) => vec![AcpRenderedLine::Text(AcpRenderedTextLine {
            prefix: vec![
                acp_icon_segment(
                    editor_icons::symbols::cod::COD_TERMINAL,
                    AcpColorRole::Accent,
                ),
                acp_text_segment(" ", AcpColorRole::Default),
            ],
            text: format!("terminal {}", terminal.terminal_id),
            text_role: AcpColorRole::Default,
        })],
        _ => vec![AcpRenderedLine::Text(AcpRenderedTextLine {
            prefix: vec![
                acp_icon_segment(
                    editor_icons::symbols::cod::COD_WARNING,
                    AcpColorRole::Warning,
                ),
                acp_text_segment(" ", AcpColorRole::Default),
            ],
            text: "Unsupported tool content".to_owned(),
            text_role: AcpColorRole::Warning,
        })],
    }
}

fn acp_render_content_block(
    block: &ContentBlock,
    prefix: Vec<AcpRenderedSegment>,
    text_role: AcpColorRole,
) -> Vec<AcpRenderedLine> {
    match block {
        ContentBlock::Text(text) => acp_multiline_text_lines(prefix, &text.text, text_role),
        ContentBlock::Image(image) => match acp_decode_image(image) {
            Ok(decoded) => {
                let mut lines = vec![AcpRenderedLine::Image(AcpRenderedImageLine {
                    label: format!(
                        "{} {}",
                        editor_icons::symbols::fa::FA_IMAGE,
                        image.mime_type
                    ),
                    image: Some(decoded),
                    rows: ACP_IMAGE_ROWS,
                })];
                lines.extend(std::iter::repeat_n(
                    AcpRenderedLine::ImageContinuation,
                    ACP_IMAGE_ROWS.saturating_sub(1),
                ));
                lines
            }
            Err(error) => acp_multiline_text_lines(
                vec![
                    acp_icon_segment(
                        editor_icons::symbols::cod::COD_WARNING,
                        AcpColorRole::Warning,
                    ),
                    acp_text_segment(" ", AcpColorRole::Default),
                ],
                format!("Image decode failed: {error}"),
                AcpColorRole::Warning,
            ),
        },
        ContentBlock::Audio(_) | ContentBlock::ResourceLink(_) | ContentBlock::Resource(_) => {
            acp_multiline_text_lines(
                vec![
                    acp_icon_segment(
                        editor_icons::symbols::cod::COD_WARNING,
                        AcpColorRole::Warning,
                    ),
                    acp_text_segment(" ", AcpColorRole::Default),
                ],
                "Unsupported ACP content block",
                AcpColorRole::Warning,
            )
        }
        _ => acp_multiline_text_lines(
            vec![
                acp_icon_segment(
                    editor_icons::symbols::cod::COD_WARNING,
                    AcpColorRole::Warning,
                ),
                acp_text_segment(" ", AcpColorRole::Default),
            ],
            "Unsupported ACP content block",
            AcpColorRole::Warning,
        ),
    }
}

fn acp_multiline_text_lines(
    prefix: Vec<AcpRenderedSegment>,
    text: impl AsRef<str>,
    text_role: AcpColorRole,
) -> Vec<AcpRenderedLine> {
    let text = text.as_ref();
    let mut lines = Vec::new();
    let continuation_prefix = acp_padding_prefix(&prefix);
    let parts = if text.is_empty() {
        vec![String::new()]
    } else {
        text.split('\n').map(str::to_owned).collect::<Vec<_>>()
    };
    for (index, line) in parts.into_iter().enumerate() {
        lines.push(AcpRenderedLine::Text(AcpRenderedTextLine {
            prefix: if index == 0 {
                prefix.clone()
            } else {
                continuation_prefix.clone()
            },
            text: line,
            text_role,
        }));
    }
    lines
}

fn acp_padding_prefix(prefix: &[AcpRenderedSegment]) -> Vec<AcpRenderedSegment> {
    let width = prefix
        .iter()
        .map(|segment| segment.text.chars().count())
        .sum();
    if width == 0 {
        return Vec::new();
    }
    vec![acp_text_segment(" ".repeat(width), AcpColorRole::Muted)]
}

fn acp_icon_segment(icon: &str, role: AcpColorRole) -> AcpRenderedSegment {
    acp_text_segment(icon, role)
}

fn acp_status_segments(status: ToolCallStatus) -> Vec<AcpRenderedSegment> {
    match status {
        ToolCallStatus::Pending => vec![acp_icon_segment(
            editor_icons::symbols::dev::DEV_CIRCLECI,
            AcpColorRole::Muted,
        )],
        ToolCallStatus::InProgress => vec![acp_spinner_segment(AcpColorRole::Accent)],
        ToolCallStatus::Completed => vec![acp_icon_segment(
            editor_icons::symbols::fa::FA_CHECK,
            AcpColorRole::Success,
        )],
        ToolCallStatus::Failed => vec![acp_icon_segment(
            editor_icons::symbols::cod::COD_ERROR,
            AcpColorRole::Error,
        )],
        _ => vec![acp_icon_segment(
            editor_icons::symbols::cod::COD_WARNING,
            AcpColorRole::Warning,
        )],
    }
}

fn acp_plan_status_segments(
    status: PlanEntryStatus,
    priority: PlanEntryPriority,
) -> Vec<AcpRenderedSegment> {
    match status {
        PlanEntryStatus::Pending => vec![acp_icon_segment(
            editor_icons::symbols::dev::DEV_CIRCLECI,
            acp_priority_color_role(priority),
        )],
        PlanEntryStatus::InProgress => vec![acp_spinner_segment(AcpColorRole::Accent)],
        PlanEntryStatus::Completed => vec![acp_icon_segment(
            editor_icons::symbols::fa::FA_CHECK,
            AcpColorRole::Success,
        )],
        _ => vec![acp_icon_segment(
            editor_icons::symbols::cod::COD_WARNING,
            AcpColorRole::Warning,
        )],
    }
}

fn acp_priority_color_role(priority: PlanEntryPriority) -> AcpColorRole {
    match priority {
        PlanEntryPriority::High => AcpColorRole::PriorityHigh,
        PlanEntryPriority::Medium => AcpColorRole::PriorityMedium,
        PlanEntryPriority::Low => AcpColorRole::PriorityLow,
        _ => AcpColorRole::Muted,
    }
}

fn acp_tool_kind_icon(kind: ToolKind) -> &'static str {
    match kind {
        ToolKind::Read => editor_icons::symbols::cod::COD_NOTEBOOK,
        ToolKind::Edit => editor_icons::symbols::cod::COD_EDIT,
        ToolKind::Delete => editor_icons::symbols::cod::COD_DIFF_REMOVED,
        ToolKind::Move => editor_icons::symbols::cod::COD_ARROW_SWAP,
        ToolKind::Search => editor_icons::symbols::cod::COD_SEARCH,
        ToolKind::Execute => editor_icons::symbols::cod::COD_TERMINAL,
        ToolKind::Think => editor_icons::symbols::cod::COD_LIGHTBULB,
        ToolKind::Fetch => editor_icons::symbols::cod::COD_CLOUD_DOWNLOAD,
        ToolKind::SwitchMode => editor_icons::symbols::cod::COD_SYNC,
        ToolKind::Other => editor_icons::symbols::cod::COD_TOOLS,
        _ => editor_icons::symbols::cod::COD_TOOLS,
    }
}

fn acp_decode_image(image: &agent_client_protocol::ImageContent) -> Result<DecodedImage, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(image.data.as_bytes())
        .map_err(|error| error.to_string())?;
    decode_raster_image_bytes(&bytes)
}

fn decode_raster_image_bytes(bytes: &[u8]) -> Result<DecodedImage, String> {
    let decoded = image::load_from_memory(bytes).map_err(|error| error.to_string())?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(DecodedImage {
        width,
        height,
        pixels: Arc::<[u8]>::from(rgba.into_raw()),
    })
}

fn decode_raster_image_path(path: &Path) -> Result<DecodedImage, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    decode_raster_image_bytes(&bytes)
}

fn rasterize_svg_text(text: &str, path: Option<&Path>) -> Result<DecodedImage, String> {
    let mut options = resvg::usvg::Options {
        resources_dir: path.and_then(Path::parent).map(Path::to_path_buf),
        ..resvg::usvg::Options::default()
    };
    options.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(text, &options).map_err(|error| error.to_string())?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| "failed to allocate SVG render target".to_owned())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    Ok(DecodedImage {
        width: pixmap.width(),
        height: pixmap.height(),
        pixels: Arc::<[u8]>::from(pixmap.take()),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneSplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowMoveDirection {
    Left,
    Down,
    Up,
    Right,
}

#[derive(Debug, Clone, Copy)]
enum GitBranchActionKind {
    Checkout,
    MergePlain,
    MergeEdit,
    MergeNoCommit,
    MergeSquash,
    MergePreview,
    RebaseOnto,
    RebaseInteractive,
}

#[derive(Debug, Clone, Copy)]
enum GitCommitActionKind {
    CherryPick,
    CherryPickNoCommit,
    Revert,
    RevertNoCommit,
    ResetMixed,
    ResetSoft,
    ResetHard,
    ResetKeep,
}

#[derive(Debug, Clone, Copy)]
enum GitSequenceKind {
    CherryPick,
    Revert,
}

#[derive(Debug, Clone, Copy)]
enum GitResetMode {
    Mixed,
    Soft,
    Hard,
    Keep,
}

#[derive(Debug, Clone, Copy)]
struct ShellPane {
    pane_id: PaneId,
    buffer_id: BufferId,
}

#[derive(Debug, Clone)]
enum PickerAction {
    NoOp,
    ExecuteCommand(String),
    ApplyLspCodeAction {
        workspace_id: WorkspaceId,
        buffer_id: BufferId,
        path: PathBuf,
        code_action: LspCodeAction,
    },
    FocusBuffer(BufferId),
    CloseBuffer(BufferId),
    CloseBufferSave(BufferId),
    CloseBufferDiscard(BufferId),
    OpenFile(PathBuf),
    OpenFileLocation {
        path: PathBuf,
        target: TextPoint,
    },
    OpenAcpClient(String),
    CreateWorkspaceFile {
        root: PathBuf,
    },
    ActivateTheme(String),
    UndoTreeNode {
        buffer_id: BufferId,
        node_id: usize,
    },
    VimSearch(VimSearchDirection),
    VimSearchResult {
        direction: VimSearchDirection,
        target: TextPoint,
    },
    InstallTreeSitterLanguage(String),
    CreateWorkspace {
        name: String,
        root: PathBuf,
    },
    SwitchWorkspace(WorkspaceId),
    DeleteWorkspace(WorkspaceId),
    GitPushRemote(String),
    GitFetchRemote(String),
    GitBranchAction {
        action: GitBranchActionKind,
        branch: String,
    },
    GitCommitAction {
        action: GitCommitActionKind,
        commit: String,
    },
    AcpInsertSlashCommand {
        buffer_id: BufferId,
        command: String,
    },
    AcpLoadSession {
        buffer_id: BufferId,
        session_id: String,
    },
    AcpSetMode {
        buffer_id: BufferId,
        mode_id: String,
    },
    AcpSetModel {
        buffer_id: BufferId,
        model_id: String,
    },
    AcpResolvePermission {
        request_id: u64,
        option_id: String,
    },
    CopyToClipboard(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickerKind {
    Generic,
    AcpPermission { request_id: u64 },
}

#[derive(Debug, Clone)]
struct PickerEntry {
    item: PickerItem,
    action: PickerAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutocompleteProviderKind {
    Buffer,
    Lsp,
    Manual,
}

#[derive(Debug, Clone)]
pub(crate) struct PickerOverlay {
    session: PickerSession,
    actions: BTreeMap<String, PickerAction>,
    submit_action: Option<PickerAction>,
    mode: PickerMode,
    kind: PickerKind,
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
            mode: PickerMode::Static,
            kind: PickerKind::Generic,
        }
    }

    fn with_result_order(mut self, result_order: PickerResultOrder) -> Self {
        self.session = self.session.with_result_order(result_order);
        self
    }

    fn search(
        title: impl Into<String>,
        direction: VimSearchDirection,
        entries: Vec<PickerEntry>,
    ) -> Self {
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
            mode: PickerMode::VimSearch(direction),
            kind: PickerKind::Generic,
        }
    }

    fn workspace_search(title: impl Into<String>, root: PathBuf) -> Self {
        Self {
            session: PickerSession::new(title.into(), Vec::new())
                .with_result_limit(48)
                .with_preserve_order(),
            actions: BTreeMap::new(),
            submit_action: Some(PickerAction::NoOp),
            mode: PickerMode::WorkspaceSearch { root },
            kind: PickerKind::Generic,
        }
    }

    pub(crate) fn session(&self) -> &PickerSession {
        &self.session
    }

    fn kind(&self) -> PickerKind {
        self.kind
    }

    fn with_kind(mut self, kind: PickerKind) -> Self {
        self.kind = kind;
        self
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
        match self.mode {
            PickerMode::VimSearch(direction) => Some(direction),
            _ => None,
        }
    }

    fn workspace_search_root(&self) -> Option<&Path> {
        match &self.mode {
            PickerMode::WorkspaceSearch { root } => Some(root.as_path()),
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
    active_buffer: BufferId,
}

#[derive(Debug, Clone)]
struct ShellWorkspaceView {
    buffer_ids: Vec<BufferId>,
    panes: Vec<ShellPane>,
    active_pane: usize,
    split_buffer_id: BufferId,
    split_direction: Option<PaneSplitDirection>,
}

impl ShellWorkspaceView {
    fn new(
        primary_pane_id: PaneId,
        primary_buffer_id: BufferId,
        split_buffer_id: BufferId,
        buffer_ids: Vec<BufferId>,
    ) -> Self {
        Self {
            buffer_ids,
            panes: vec![ShellPane {
                pane_id: primary_pane_id,
                buffer_id: primary_buffer_id,
            }],
            active_pane: 0,
            split_buffer_id,
            split_direction: None,
        }
    }
}

#[derive(Debug, Clone)]
struct DirectoryPrefixState {
    started_at: Instant,
}

#[derive(Debug, Clone)]
struct KeySequenceState {
    tokens: Vec<String>,
    started_at: Instant,
}

pub(crate) struct ShellUiState {
    buffers: Vec<ShellBuffer>,
    workspace_views: BTreeMap<WorkspaceId, ShellWorkspaceView>,
    active_workspace: WorkspaceId,
    previous_workspace: Option<WorkspaceId>,
    default_workspace: WorkspaceId,
    input_mode: InputMode,
    vim: VimState,
    pending_ctrl_c: Option<Instant>,
    pending_git_prefix: Option<GitPrefixState>,
    pending_directory_prefix: Option<DirectoryPrefixState>,
    pending_key_sequence: Option<KeySequenceState>,
    attached_lsp_servers: BTreeMap<WorkspaceId, String>,
    picker: Option<PickerOverlay>,
    command_line: Option<CommandLineOverlay>,
    autocomplete: Option<AutocompleteOverlay>,
    hover: Option<HoverOverlay>,
    notifications: NotificationCenter,
    last_lsp_notification_revision: u64,
    popup_focus: bool,
    popup_buffer_id: Option<BufferId>,
    yank_flash: Option<YankFlash>,
    git_summary: GitSummaryState,
    autocomplete_worker: AutocompleteWorkerState,
    vim_search_worker: VimSearchWorkerState,
    workspace_search_worker: WorkspaceSearchWorkerState,
    file_reload_worker: FileReloadWorkerState,
    syntax_refresh_worker: SyntaxRefreshWorkerState,
    /// Per-workspace last-used build command.  Set when the user runs
    /// `workspace.compile`; reused by `workspace.recompile`.
    compile_commands: BTreeMap<WorkspaceId, String>,
}

impl ShellUiState {
    fn new(
        default_workspace: WorkspaceId,
        primary_pane_id: PaneId,
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
                primary_pane_id,
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
            pending_ctrl_c: None,
            pending_git_prefix: None,
            pending_directory_prefix: None,
            pending_key_sequence: None,
            attached_lsp_servers: BTreeMap::new(),
            picker: None,
            command_line: None,
            autocomplete: None,
            hover: None,
            notifications: NotificationCenter::default(),
            last_lsp_notification_revision: 0,
            popup_focus: false,
            popup_buffer_id: None,
            yank_flash: None,
            git_summary: GitSummaryState::new(),
            autocomplete_worker: AutocompleteWorkerState::new(),
            vim_search_worker: VimSearchWorkerState::new(),
            workspace_search_worker: WorkspaceSearchWorkerState::new(),
            file_reload_worker: FileReloadWorkerState::new(),
            syntax_refresh_worker: SyntaxRefreshWorkerState::disabled(),
            compile_commands: BTreeMap::new(),
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

    fn command_line_visible(&self) -> bool {
        self.command_line.is_some()
    }

    fn focused_buffer_id(&self) -> Option<BufferId> {
        if self.popup_focus {
            self.popup_buffer_id.or_else(|| self.active_buffer_id())
        } else {
            self.active_buffer_id()
        }
    }

    fn set_popup_buffer(&mut self, buffer_id: BufferId) {
        if self.popup_buffer_id == Some(buffer_id) {
            return;
        }
        if self.popup_focus {
            if let Some(previous_buffer_id) = self.popup_buffer_id {
                self.persist_buffer_vim_state(previous_buffer_id);
            }
            self.popup_buffer_id = Some(buffer_id);
            self.restore_buffer_vim_state(buffer_id);
        } else {
            self.popup_buffer_id = Some(buffer_id);
        }
    }

    fn clear_popup_buffer(&mut self) {
        self.popup_buffer_id = None;
    }

    fn set_popup_focus(&mut self, focus: bool) {
        if self.popup_focus == focus {
            return;
        }
        self.persist_active_buffer_vim_state();
        self.popup_focus = focus;
        self.restore_active_buffer_vim_state();
    }

    fn popup_focus_allowed(&self, popup: &RuntimePopupSnapshot) -> bool {
        if let Some(buffer) = self.buffer(popup.active_buffer) {
            return !buffer_is_oil_preview(&buffer.kind);
        }
        true
    }

    fn popup_focus_active(&self, popup: &RuntimePopupSnapshot) -> bool {
        self.popup_focus && self.popup_focus_allowed(popup)
    }

    fn git_summary(&self) -> Option<GitSummarySnapshot> {
        self.git_summary.snapshot()
    }

    fn git_summary_revision(&self) -> u64 {
        self.git_summary.snapshot_revision()
    }

    fn git_summary_refresh_due(&self, now: Instant) -> bool {
        self.git_summary.refresh_due(now)
    }

    fn git_summary_state(&self) -> GitSummaryState {
        self.git_summary.clone()
    }

    fn mark_git_summary_refreshed(&mut self, now: Instant) {
        self.git_summary.mark_refreshed(now);
    }

    fn clear_git_summary(&self) {
        self.git_summary.set_snapshot(None);
    }

    fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    fn input_mode_for_buffer(&self, buffer_id: BufferId, active: bool) -> InputMode {
        if active {
            self.input_mode
        } else {
            self.buffer(buffer_id)
                .map(|buffer| buffer.vim_buffer_state.input_mode)
                .unwrap_or(InputMode::Normal)
        }
    }

    fn vim_target_for_buffer(&self, buffer_id: BufferId, active: bool) -> VimTarget {
        if active {
            self.vim.target
        } else {
            self.buffer(buffer_id)
                .map(|buffer| buffer.vim_buffer_state.target)
                .unwrap_or(VimTarget::Buffer)
        }
    }

    fn visual_selection_for_buffer(
        &self,
        buffer: &ShellBuffer,
        active: bool,
    ) -> Option<VisualSelection> {
        let (input_mode, target, visual_anchor, visual_kind) = if active {
            (
                self.input_mode,
                self.vim.target,
                self.vim.visual_anchor,
                self.vim.visual_kind,
            )
        } else {
            (
                buffer.vim_buffer_state.input_mode,
                buffer.vim_buffer_state.target,
                buffer.vim_buffer_state.visual_anchor,
                buffer.vim_buffer_state.visual_kind,
            )
        };
        if input_mode != InputMode::Visual || target == VimTarget::Input {
            return None;
        }
        visual_anchor.and_then(|anchor| visual_selection(buffer, anchor, visual_kind))
    }

    fn multicursor_for_buffer(
        &self,
        buffer_id: BufferId,
        active: bool,
    ) -> Option<&MulticursorState> {
        if active {
            self.vim.multicursor.as_ref()
        } else {
            self.buffer(buffer_id)
                .and_then(|buffer| buffer.vim_buffer_state.multicursor.as_ref())
        }
    }

    fn persist_buffer_vim_state(&mut self, buffer_id: BufferId) {
        let state = self.vim.active_buffer_state(self.input_mode);
        if let Some(buffer) = self.buffer_mut(buffer_id) {
            buffer.vim_buffer_state = state;
        }
    }

    fn persist_active_buffer_vim_state(&mut self) {
        if let Some(buffer_id) = self.focused_buffer_id() {
            self.persist_buffer_vim_state(buffer_id);
        }
    }

    fn restore_buffer_vim_state(&mut self, buffer_id: BufferId) {
        let state = self
            .buffer(buffer_id)
            .map(|buffer| buffer.vim_buffer_state.clone())
            .unwrap_or_default();
        self.vim
            .apply_active_buffer_state(&mut self.input_mode, &state);
    }

    fn restore_active_buffer_vim_state(&mut self) {
        if let Some(buffer_id) = self.focused_buffer_id() {
            self.restore_buffer_vim_state(buffer_id);
        } else {
            self.vim
                .apply_active_buffer_state(&mut self.input_mode, &VimBufferState::default());
        }
    }

    fn enter_normal_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.vim.visual_anchor = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.multicursor = None;
        self.vim.clear_transient();
        self.persist_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn enter_insert_mode(&mut self) {
        self.input_mode = InputMode::Insert;
        self.vim.visual_anchor = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.clear_transient();
        self.persist_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn enter_replace_mode(&mut self) {
        self.input_mode = InputMode::Replace;
        self.vim.visual_anchor = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.clear_transient();
        self.persist_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn enter_visual_mode(&mut self, anchor: TextPoint, kind: VisualSelectionKind) {
        self.input_mode = InputMode::Visual;
        self.vim.visual_anchor = Some(anchor);
        self.vim.visual_kind = kind;
        self.vim.clear_transient();
        self.persist_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    pub(crate) fn vim(&self) -> &VimState {
        &self.vim
    }

    fn vim_mut(&mut self) -> &mut VimState {
        &mut self.vim
    }

    fn active_buffer_targets_input(&self) -> bool {
        self.focused_buffer_id()
            .and_then(|buffer_id| self.buffer(buffer_id))
            .is_some_and(|buffer| buffer.has_input_field() && self.vim.target == VimTarget::Input)
    }

    fn set_active_vim_target(&mut self, target: VimTarget) {
        self.vim.target = target;
        self.persist_active_buffer_vim_state();
    }

    pub(crate) fn active_workspace(&self) -> WorkspaceId {
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
        self.persist_active_buffer_vim_state();
        if self.active_workspace != workspace_id {
            self.previous_workspace = Some(self.active_workspace);
            self.active_workspace = workspace_id;
        }
        self.restore_active_buffer_vim_state();
        self.close_picker();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn add_workspace(
        &mut self,
        workspace_id: WorkspaceId,
        primary_pane_id: PaneId,
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
                primary_pane_id,
                primary_buffer_id,
                split_buffer_id,
                vec![primary_buffer_id, secondary_buffer_id],
            ),
        );
    }

    fn remove_workspace(&mut self, workspace_id: WorkspaceId) {
        let active_workspace_removed = self.active_workspace == workspace_id;
        if active_workspace_removed {
            self.persist_active_buffer_vim_state();
        }
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
            self.restore_active_buffer_vim_state();
        }
    }

    fn set_attached_lsp_server(
        &mut self,
        workspace_id: WorkspaceId,
        attached_lsp_server: Option<String>,
    ) -> bool {
        let previous = self.attached_lsp_servers.get(&workspace_id).cloned();
        match attached_lsp_server.clone() {
            Some(server) => {
                self.attached_lsp_servers.insert(workspace_id, server);
            }
            None => {
                self.attached_lsp_servers.remove(&workspace_id);
            }
        }
        previous != attached_lsp_server
    }

    fn panes(&self) -> Option<&[ShellPane]> {
        self.workspace_view().map(|view| view.panes.as_slice())
    }

    fn active_pane_index(&self) -> usize {
        self.workspace_view()
            .map(|view| view.active_pane)
            .unwrap_or(0)
    }

    fn active_pane_id(&self) -> Option<PaneId> {
        self.workspace_view()
            .and_then(|view| view.panes.get(view.active_pane))
            .map(|pane| pane.pane_id)
    }

    fn focus_pane(&mut self, pane_id: PaneId) {
        self.persist_active_buffer_vim_state();
        if let Some(view) = self.workspace_view_mut()
            && let Some(index) = view.panes.iter().position(|pane| pane.pane_id == pane_id)
        {
            view.active_pane = index;
        }
        self.restore_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn split_buffer_id(&self) -> Option<BufferId> {
        self.workspace_view().map(|view| view.split_buffer_id)
    }

    fn pane_split_direction(&self) -> PaneSplitDirection {
        self.workspace_view()
            .and_then(|view| view.split_direction)
            .unwrap_or(PaneSplitDirection::Horizontal)
    }

    fn active_workspace_buffer_ids(&self) -> Option<&[BufferId]> {
        self.workspace_view().map(|view| view.buffer_ids.as_slice())
    }

    pub(crate) fn attached_lsp_server(&self) -> Option<&str> {
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

    fn insert_buffer(&mut self, mut buffer: ShellBuffer) {
        let new_watch_path = shell_buffer_watch_path(&buffer);
        if let Some(existing) = self
            .buffers
            .iter_mut()
            .find(|existing| existing.id() == buffer.id())
        {
            let old_watch_path = shell_buffer_watch_path(existing);
            buffer.vim_buffer_state = existing.vim_buffer_state.clone();
            *existing = buffer;
            sync_file_reload_watch(
                &mut self.file_reload_worker,
                old_watch_path.as_deref(),
                new_watch_path.as_deref(),
            );
        } else {
            self.buffers.push(buffer);
            sync_file_reload_watch(
                &mut self.file_reload_worker,
                None,
                new_watch_path.as_deref(),
            );
        }
    }

    fn remove_buffer(&mut self, buffer_id: BufferId) {
        let removed_active_buffer = self.active_buffer_id() == Some(buffer_id);
        let removed_watch_path = self
            .buffers
            .iter()
            .find(|buffer| buffer.id() == buffer_id)
            .and_then(shell_buffer_watch_path);
        if !removed_active_buffer {
            self.persist_active_buffer_vim_state();
        }
        self.buffers.retain(|buffer| buffer.id() != buffer_id);
        if let Some(path) = removed_watch_path.as_deref() {
            self.file_reload_worker.unwatch_path(path);
        }
        for view in self.workspace_views.values_mut() {
            if view.buffer_ids.contains(&buffer_id) {
                view.buffer_ids.retain(|id| *id != buffer_id);
                if let Some(fallback) = view.buffer_ids.first().copied() {
                    if view.split_buffer_id == buffer_id {
                        view.split_buffer_id = fallback;
                    }
                    for pane in view.panes.iter_mut() {
                        if pane.buffer_id == buffer_id {
                            pane.buffer_id = fallback;
                        }
                    }
                }
            }
            if view.active_pane >= view.panes.len() {
                view.active_pane = 0;
            }
        }
        if removed_active_buffer {
            self.restore_active_buffer_vim_state();
        }
    }

    pub(crate) fn picker(&self) -> Option<&PickerOverlay> {
        self.picker.as_ref()
    }

    fn picker_kind(&self) -> Option<PickerKind> {
        self.picker.as_ref().map(PickerOverlay::kind)
    }

    fn picker_mut(&mut self) -> Option<&mut PickerOverlay> {
        self.picker.as_mut()
    }

    fn set_picker(&mut self, picker: PickerOverlay) {
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
        self.vim_search_worker.clear_pending();
        self.workspace_search_worker.clear_pending();
        self.picker = Some(picker);
    }

    fn close_picker(&mut self) {
        self.vim_search_worker.clear_pending();
        self.workspace_search_worker.clear_pending();
        self.picker = None;
    }

    fn command_line(&self) -> Option<&CommandLineOverlay> {
        self.command_line.as_ref()
    }

    fn command_line_mut(&mut self) -> Option<&mut CommandLineOverlay> {
        self.command_line.as_mut()
    }

    fn set_command_line(&mut self, command_line: CommandLineOverlay) {
        self.close_picker();
        self.close_autocomplete();
        self.close_hover();
        self.command_line = Some(command_line);
    }

    fn close_command_line(&mut self) {
        self.command_line = None;
    }

    fn autocomplete(&self) -> Option<&AutocompleteOverlay> {
        self.autocomplete.as_ref()
    }

    fn autocomplete_mut(&mut self) -> Option<&mut AutocompleteOverlay> {
        self.autocomplete.as_mut()
    }

    fn set_autocomplete(&mut self, autocomplete: AutocompleteOverlay) {
        self.close_picker();
        self.close_command_line();
        self.close_hover();
        self.autocomplete_worker.clear_pending();
        self.autocomplete = Some(autocomplete);
    }

    fn close_autocomplete(&mut self) {
        self.autocomplete_worker.clear_pending();
        self.autocomplete = None;
    }

    fn hover(&self) -> Option<&HoverOverlay> {
        self.hover.as_ref()
    }

    fn hover_mut(&mut self) -> Option<&mut HoverOverlay> {
        self.hover.as_mut()
    }

    fn set_hover(&mut self, hover: HoverOverlay) {
        self.close_picker();
        self.close_command_line();
        self.close_autocomplete();
        self.hover = Some(hover);
    }

    fn close_hover(&mut self) {
        self.hover = None;
    }

    fn apply_notification(&mut self, update: NotificationUpdate, now: Instant) -> bool {
        self.notifications.apply(update, now)
    }

    fn prune_notifications(&mut self, now: Instant) -> bool {
        self.notifications.prune_expired(now)
    }

    fn visible_notifications(&self, now: Instant) -> Vec<&ShellNotification> {
        self.notifications.visible(now)
    }

    fn notification_revision(&self) -> u64 {
        self.notifications.revision()
    }

    fn notification_deadline(&self, now: Instant) -> Option<Instant> {
        self.notifications.next_deadline(now)
    }

    fn last_lsp_notification_revision(&self) -> u64 {
        self.last_lsp_notification_revision
    }

    fn set_last_lsp_notification_revision(&mut self, revision: u64) {
        self.last_lsp_notification_revision = revision;
    }

    fn configure_syntax_refresh_worker(
        &mut self,
        configs: Vec<LanguageConfiguration>,
        install_root: PathBuf,
    ) {
        self.syntax_refresh_worker.configure(configs, install_root);
    }

    fn set_yank_flash(&mut self, buffer_id: BufferId, selection: VisualSelection) {
        const YANK_FLASH_DURATION: Duration = Duration::from_millis(140);
        self.yank_flash = Some(YankFlash {
            buffer_id,
            selection,
            until: Instant::now() + YANK_FLASH_DURATION,
        });
    }

    pub(crate) fn yank_flash(&self, buffer_id: BufferId, now: Instant) -> Option<VisualSelection> {
        self.yank_flash.and_then(|flash| {
            (flash.buffer_id == buffer_id && now <= flash.until).then_some(flash.selection)
        })
    }

    fn yank_flash_deadline(&self, now: Instant) -> Option<Instant> {
        self.yank_flash
            .and_then(|flash| (now <= flash.until).then_some(flash.until))
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
        user_library: &dyn UserLibrary,
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

        self.buffers.push(ShellBuffer::placeholder(
            buffer_id,
            name,
            kind,
            user_library,
        ));
        let index = self.buffers.len() - 1;
        &mut self.buffers[index]
    }

    fn ensure_popup_buffer(
        &mut self,
        buffer_id: BufferId,
        name: &str,
        kind: BufferKind,
        user_library: &dyn UserLibrary,
    ) -> &mut ShellBuffer {
        if let Some(index) = self
            .buffers
            .iter()
            .position(|buffer| buffer.id() == buffer_id)
        {
            return &mut self.buffers[index];
        }

        self.buffers.push(ShellBuffer::placeholder(
            buffer_id,
            name,
            kind,
            user_library,
        ));
        let index = self.buffers.len() - 1;
        &mut self.buffers[index]
    }

    fn active_buffer_id(&self) -> Option<BufferId> {
        self.workspace_view()?
            .panes
            .get(self.active_pane_index())
            .map(|pane| pane.buffer_id)
    }

    fn focus_buffer_in_active_pane(&mut self, buffer_id: BufferId) {
        self.persist_active_buffer_vim_state();
        if self.buffers.iter().any(|buffer| buffer.id() == buffer_id)
            && let Some(view) = self.workspace_view_mut()
            && let Some(pane) = view.panes.get_mut(view.active_pane)
        {
            if !view.buffer_ids.contains(&buffer_id) {
                view.buffer_ids.push(buffer_id);
            }
            pane.buffer_id = buffer_id;
        }
        self.restore_active_buffer_vim_state();
    }

    fn focus_buffer(&mut self, buffer_id: BufferId) {
        self.focus_buffer_in_active_pane(buffer_id);
        self.close_picker();
        self.close_autocomplete();
        self.close_hover();
    }

    fn split_pane(&mut self, pane_id: PaneId, buffer_id: BufferId, direction: PaneSplitDirection) {
        if let Some(view) = self.workspace_view_mut()
            && view.panes.len() == 1
        {
            if !view.buffer_ids.contains(&buffer_id) {
                view.buffer_ids.push(buffer_id);
            }
            view.panes.push(ShellPane { pane_id, buffer_id });
            view.split_direction = Some(direction);
        }
    }

    fn close_pane(&mut self, pane_id: PaneId) {
        self.persist_active_buffer_vim_state();
        if let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
            && let Some(index) = view.panes.iter().position(|pane| pane.pane_id == pane_id)
        {
            view.panes.remove(index);
            if view.panes.len() == 1 {
                view.split_direction = None;
            }
            if index < view.active_pane {
                view.active_pane = view.active_pane.saturating_sub(1);
            } else if index == view.active_pane {
                view.active_pane = view.active_pane.min(view.panes.len().saturating_sub(1));
            }
        }
        self.restore_active_buffer_vim_state();
        self.close_autocomplete();
        self.close_hover();
    }

    fn switch_split(&mut self) -> bool {
        let active_pane_id = if let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
        {
            let active_pane_id = view.panes.get(view.active_pane).map(|pane| pane.pane_id);
            view.panes.reverse();
            if let Some(active_pane_id) = active_pane_id
                && let Some(index) = view
                    .panes
                    .iter()
                    .position(|pane| pane.pane_id == active_pane_id)
            {
                view.active_pane = index;
            }
            active_pane_id
        } else {
            None
        };
        if active_pane_id.is_none() {
            return false;
        }
        self.close_autocomplete();
        self.close_hover();
        true
    }

    fn shift_active_pane(&mut self, delta: isize) -> Option<PaneId> {
        self.persist_active_buffer_vim_state();
        if !self.picker_visible()
            && let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
        {
            let pane_count = view.panes.len() as isize;
            let next = (view.active_pane as isize + delta).rem_euclid(pane_count);
            view.active_pane = next as usize;
        }
        self.restore_active_buffer_vim_state();
        self.active_pane_id()
    }

    fn cycle_active_pane(&mut self) -> Option<PaneId> {
        self.persist_active_buffer_vim_state();
        if !self.picker_visible()
            && let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
        {
            view.active_pane = (view.active_pane + 1) % view.panes.len();
        }
        self.restore_active_buffer_vim_state();
        self.active_pane_id()
    }
}

fn shell_buffer_watch_path(buffer: &ShellBuffer) -> Option<PathBuf> {
    (buffer.kind == BufferKind::File || buffer.is_pdf_buffer())
        .then_some(())
        .and_then(|_| buffer.path().map(Path::to_path_buf))
}

fn sync_file_reload_watch(
    worker: &mut FileReloadWorkerState,
    previous: Option<&Path>,
    current: Option<&Path>,
) {
    if previous == current {
        return;
    }
    if let Some(previous) = previous {
        worker.unwatch_path(previous);
    }
    if let Some(current) = current {
        worker.watch_path(current.to_path_buf());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorSeverity {
    Error,
}

impl ErrorSeverity {
    fn label(self) -> &'static str {
        "error"
    }
}

#[derive(Debug, Clone)]
struct ErrorEntry {
    timestamp: SystemTime,
    severity: ErrorSeverity,
    source: String,
    message: String,
}

impl ErrorEntry {
    fn new(severity: ErrorSeverity, source: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            severity,
            source: source.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug)]
struct ErrorLog {
    entries: Vec<ErrorEntry>,
    buffer_id: BufferId,
    log_file_path: PathBuf,
    file_logging_enabled: bool,
    max_entries: usize,
}

impl ErrorLog {
    fn new(buffer_id: BufferId, log_file_path: PathBuf, file_logging_enabled: bool) -> Self {
        Self {
            entries: Vec::new(),
            buffer_id,
            log_file_path,
            file_logging_enabled,
            max_entries: ERROR_LOG_MAX_ENTRIES,
        }
    }

    fn record(&mut self, entry: ErrorEntry) -> Vec<String> {
        self.push_entry(entry.clone());
        if self.file_logging_enabled
            && let Err(error) = append_error_log(&self.log_file_path, &entry)
        {
            self.file_logging_enabled = false;
            self.push_entry(ErrorEntry::new(
                ErrorSeverity::Error,
                "error-log",
                format!(
                    "failed to write error log to `{}`: {error}",
                    self.log_file_path.display()
                ),
            ));
        }
        errors_buffer_lines(&self.entries, &self.log_file_path)
    }

    fn push_entry(&mut self, entry: ErrorEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            let overflow = self.entries.len() - self.max_entries;
            self.entries.drain(0..overflow);
        }
    }
}

#[derive(Debug, Default)]
struct LspLogBufferState {
    buffer_ids: BTreeMap<WorkspaceId, BTreeMap<String, BufferId>>,
    applied_revision: u64,
}

impl LspLogBufferState {
    fn buffer_id(&self, workspace_id: WorkspaceId, server_id: &str) -> Option<BufferId> {
        self.buffer_ids
            .get(&workspace_id)
            .and_then(|buffers| buffers.get(server_id))
            .copied()
    }

    fn insert_buffer(&mut self, workspace_id: WorkspaceId, server_id: String, buffer_id: BufferId) {
        self.buffer_ids
            .entry(workspace_id)
            .or_default()
            .insert(server_id, buffer_id);
    }

    fn buffers_for_workspace(&self, workspace_id: WorkspaceId) -> Vec<(String, BufferId)> {
        self.buffer_ids
            .get(&workspace_id)
            .into_iter()
            .flat_map(|buffers| buffers.iter())
            .map(|(server_id, buffer_id)| (server_id.clone(), *buffer_id))
            .collect()
    }

    fn remove_workspace(&mut self, workspace_id: WorkspaceId) {
        self.buffer_ids.remove(&workspace_id);
    }
}

#[derive(Debug, Clone)]
struct TypingFrameProfile {
    frame_index: u32,
    timestamp: SystemTime,
    frame_pacing_sleep: Duration,
    polled_events: usize,
    keydown_events: usize,
    text_input_events: usize,
    text_preview: String,
    handle_event_total: Duration,
    keydown_handle_total: Duration,
    text_input_handle_total: Duration,
    text_input_inner_total: Duration,
    picker_refresh: Duration,
    syntax_refresh: Duration,
    syntax_worker_compute: Duration,
    syntax_result_count: usize,
    syntax_highlight_spans: usize,
    git_refresh: Duration,
    acp_refresh: Duration,
    render: Duration,
    present: Duration,
    frame_total: Duration,
    first_text_to_present: Option<Duration>,
    last_text_to_present: Option<Duration>,
}

#[derive(Debug)]
struct ActiveTypingFrameProfile {
    frame_index: u32,
    timestamp: SystemTime,
    frame_pacing_sleep: Duration,
    polled_events: usize,
    keydown_events: usize,
    text_input_events: usize,
    text_preview: String,
    handle_event_total: Duration,
    keydown_handle_total: Duration,
    text_input_handle_total: Duration,
    text_input_inner_total: Duration,
    picker_refresh: Duration,
    syntax_refresh: Duration,
    syntax_worker_compute: Duration,
    syntax_result_count: usize,
    syntax_highlight_spans: usize,
    git_refresh: Duration,
    acp_refresh: Duration,
    render: Duration,
    present: Duration,
    first_text_input_started_at: Option<Instant>,
    last_text_input_started_at: Option<Instant>,
}

impl ActiveTypingFrameProfile {
    fn new(frame_index: u32, frame_pacing_sleep: Duration) -> Self {
        Self {
            frame_index,
            timestamp: SystemTime::now(),
            frame_pacing_sleep,
            polled_events: 0,
            keydown_events: 0,
            text_input_events: 0,
            text_preview: String::new(),
            handle_event_total: Duration::from_secs(0),
            keydown_handle_total: Duration::from_secs(0),
            text_input_handle_total: Duration::from_secs(0),
            text_input_inner_total: Duration::from_secs(0),
            picker_refresh: Duration::from_secs(0),
            syntax_refresh: Duration::from_secs(0),
            syntax_worker_compute: Duration::from_secs(0),
            syntax_result_count: 0,
            syntax_highlight_spans: 0,
            git_refresh: Duration::from_secs(0),
            acp_refresh: Duration::from_secs(0),
            render: Duration::from_secs(0),
            present: Duration::from_secs(0),
            first_text_input_started_at: None,
            last_text_input_started_at: None,
        }
    }

    fn record_event(
        &mut self,
        metadata: &TypingEventMetadata,
        handle_event_total: Duration,
        text_input_inner_total: Option<Duration>,
    ) {
        self.polled_events = self.polled_events.saturating_add(1);
        self.handle_event_total += handle_event_total;
        match metadata {
            TypingEventMetadata::KeyDown => {
                self.keydown_events = self.keydown_events.saturating_add(1);
                self.keydown_handle_total += handle_event_total;
            }
            TypingEventMetadata::TextInput { text, received_at } => {
                self.text_input_events = self.text_input_events.saturating_add(1);
                self.text_input_handle_total += handle_event_total;
                self.text_input_inner_total +=
                    text_input_inner_total.unwrap_or_else(|| Duration::from_secs(0));
                if self.first_text_input_started_at.is_none() {
                    self.first_text_input_started_at = Some(*received_at);
                }
                self.last_text_input_started_at = Some(*received_at);
                self.push_text_preview(text);
            }
            TypingEventMetadata::Other => {}
        }
    }

    fn push_text_preview(&mut self, text: &str) {
        const MAX_PREVIEW_CHARS: usize = 24;
        if self.text_preview.chars().count() >= MAX_PREVIEW_CHARS {
            return;
        }
        let sanitized = sanitize_typing_preview(text);
        if !self.text_preview.is_empty() {
            self.text_preview.push('|');
        }
        for character in sanitized.chars() {
            if self.text_preview.chars().count() >= MAX_PREVIEW_CHARS {
                self.text_preview.push('…');
                break;
            }
            self.text_preview.push(character);
        }
    }

    fn finish(self, frame_total: Duration, presented_at: Instant) -> TypingFrameProfile {
        TypingFrameProfile {
            frame_index: self.frame_index,
            timestamp: self.timestamp,
            frame_pacing_sleep: self.frame_pacing_sleep,
            polled_events: self.polled_events,
            keydown_events: self.keydown_events,
            text_input_events: self.text_input_events,
            text_preview: self.text_preview,
            handle_event_total: self.handle_event_total,
            keydown_handle_total: self.keydown_handle_total,
            text_input_handle_total: self.text_input_handle_total,
            text_input_inner_total: self.text_input_inner_total,
            picker_refresh: self.picker_refresh,
            syntax_refresh: self.syntax_refresh,
            syntax_worker_compute: self.syntax_worker_compute,
            syntax_result_count: self.syntax_result_count,
            syntax_highlight_spans: self.syntax_highlight_spans,
            git_refresh: self.git_refresh,
            acp_refresh: self.acp_refresh,
            render: self.render,
            present: self.present,
            frame_total,
            first_text_to_present: self
                .first_text_input_started_at
                .map(|received_at| presented_at.duration_since(received_at)),
            last_text_to_present: self
                .last_text_input_started_at
                .map(|received_at| presented_at.duration_since(received_at)),
        }
    }
}

#[derive(Debug)]
enum TypingEventMetadata {
    KeyDown,
    TextInput { text: String, received_at: Instant },
    Other,
}

impl TypingEventMetadata {
    fn from_event(event: &Event) -> Self {
        match event {
            Event::KeyDown { .. } => Self::KeyDown,
            Event::TextInput { text, .. } => Self::TextInput {
                text: text.clone(),
                received_at: Instant::now(),
            },
            _ => Self::Other,
        }
    }
}

#[derive(Debug)]
struct TypingProfiler {
    log_path: PathBuf,
    frames: Vec<TypingFrameProfile>,
    max_frames: usize,
    dropped_frames: usize,
}

impl TypingProfiler {
    fn new(log_path: PathBuf) -> Self {
        Self {
            log_path,
            frames: Vec::new(),
            max_frames: TYPING_PROFILE_MAX_FRAMES,
            dropped_frames: 0,
        }
    }

    fn record_frame(&mut self, frame: TypingFrameProfile) {
        if frame.text_input_events == 0
            && frame.keydown_events == 0
            && frame.syntax_result_count == 0
            && frame.frame_total < TYPING_PROFILE_SLOW_FRAME_THRESHOLD
        {
            return;
        }
        self.frames.push(frame);
        if self.frames.len() > self.max_frames {
            let overflow = self.frames.len() - self.max_frames;
            self.frames.drain(0..overflow);
            self.dropped_frames = self.dropped_frames.saturating_add(overflow);
        }
    }

    fn write_report(&self) -> Result<TypingProfileSummary, String> {
        ensure_log_directory(&self.log_path)?;
        let mut file = fs::File::create(&self.log_path).map_err(|error| {
            format!(
                "failed to create typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        let input_frames = self
            .frames
            .iter()
            .filter(|frame| frame.text_input_events > 0)
            .collect::<Vec<_>>();
        let input_frame_times = input_frames
            .iter()
            .map(|frame| frame.frame_total)
            .collect::<Vec<_>>();
        let syntax_result_frames = self
            .frames
            .iter()
            .filter(|frame| frame.syntax_result_count > 0)
            .collect::<Vec<_>>();
        let syntax_worker_times = syntax_result_frames
            .iter()
            .map(|frame| frame.syntax_worker_compute)
            .collect::<Vec<_>>();
        let syntax_apply_times = syntax_result_frames
            .iter()
            .map(|frame| frame.syntax_refresh)
            .collect::<Vec<_>>();
        let slowest_frame = self
            .frames
            .iter()
            .map(|frame| frame.frame_total)
            .max()
            .unwrap_or_else(|| Duration::from_secs(0));

        writeln!(file, "Volt typing profile").map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "Frames captured: {}", self.frames.len()).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "Frames with text input: {}", input_frames.len()).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "Dropped frames: {}", self.dropped_frames).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        if !input_frame_times.is_empty() {
            writeln!(
                file,
                "Input frame total: avg={}, p50={}, p95={}, max={}",
                format_duration_ms(average_duration(&input_frame_times)),
                format_duration_ms(percentile_duration(&input_frame_times, 50)),
                format_duration_ms(percentile_duration(&input_frame_times, 95)),
                format_duration_ms(
                    *input_frame_times
                        .iter()
                        .max()
                        .unwrap_or(&Duration::from_secs(0))
                ),
            )
            .map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
        }
        if !syntax_result_frames.is_empty() {
            let syntax_result_total = syntax_result_frames
                .iter()
                .map(|frame| frame.syntax_result_count)
                .sum::<usize>();
            let syntax_span_total = syntax_result_frames
                .iter()
                .map(|frame| frame.syntax_highlight_spans)
                .sum::<usize>();
            writeln!(
                file,
                "Syntax worker compute: avg={}, p50={}, p95={}, max={} (frames={}, results={}, spans={})",
                format_duration_ms(average_duration(&syntax_worker_times)),
                format_duration_ms(percentile_duration(&syntax_worker_times, 50)),
                format_duration_ms(percentile_duration(&syntax_worker_times, 95)),
                format_duration_ms(
                    *syntax_worker_times
                        .iter()
                        .max()
                        .unwrap_or(&Duration::from_secs(0))
                ),
                syntax_result_frames.len(),
                syntax_result_total,
                syntax_span_total,
            )
            .map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
            writeln!(
                file,
                "Syntax UI apply: avg={}, p50={}, p95={}, max={}",
                format_duration_ms(average_duration(&syntax_apply_times)),
                format_duration_ms(percentile_duration(&syntax_apply_times, 50)),
                format_duration_ms(percentile_duration(&syntax_apply_times, 95)),
                format_duration_ms(
                    *syntax_apply_times
                        .iter()
                        .max()
                        .unwrap_or(&Duration::from_secs(0))
                ),
            )
            .map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
        }
        writeln!(file).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "Slowest captured frames").map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        let mut slowest_frames = self.frames.iter().collect::<Vec<_>>();
        slowest_frames.sort_by_key(|frame| std::cmp::Reverse(frame.frame_total));
        for frame in slowest_frames.into_iter().take(20) {
            writeln!(file, "{}", format_typing_frame_profile(frame)).map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
        }
        writeln!(file).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "All captured frames").map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        for frame in &self.frames {
            writeln!(file, "{}", format_typing_frame_profile(frame)).map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
        }

        Ok(TypingProfileSummary {
            log_path: self.log_path.display().to_string(),
            frames_captured: self.frames.len(),
            input_frames_captured: input_frames.len(),
            slowest_frame_micros: slowest_frame.as_micros(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellVisualRefreshKey {
    render_width: u32,
    render_height: u32,
    theme_settings: ThemeRuntimeSettings,
    git_summary_revision: u64,
    git_fringe_revisions: Vec<(BufferId, u64)>,
    lsp_diagnostics_revisions: Vec<(BufferId, u64)>,
    active_lsp_server: Option<String>,
    active_lsp_workspace_loaded: bool,
    notification_revision: u64,
    notification_deadline: Option<Instant>,
    yank_flash_until: Option<Instant>,
}

pub(crate) struct ShellState {
    pub(crate) runtime: EditorRuntime,
    pub(crate) user_library: Arc<dyn UserLibrary>,
    typing_profiler: Option<TypingProfiler>,
    last_text_input_profile: Option<Duration>,
    last_text_input_at: Option<Instant>,
    pending_suppressed_text_input: Option<SuppressedTextInput>,
    mouse_drag: Option<MouseDragState>,
    browser_host: BrowserHostService,
}

#[derive(Debug)]
struct SuppressedTextInput {
    text: String,
    expires_at: Instant,
}

#[derive(Debug, Clone, Copy)]
struct MouseDragState {
    buffer_id: BufferId,
    rect: PixelRect,
    anchor: TextPoint,
    kind: VisualSelectionKind,
}

impl ShellState {
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, ShellError> {
        let user_library: Arc<dyn UserLibrary> = Arc::new(NullUserLibrary);
        Self::new_with_user_library(default_error_log_path(), false, user_library)
    }

    pub(crate) fn new_with_user_library(
        log_file_path: PathBuf,
        profile_input_latency: bool,
        user_library: Arc<dyn UserLibrary>,
    ) -> Result<Self, ShellError> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("volt");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "default", None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;

        register_shell_hooks(&mut runtime).map_err(ShellError::Runtime)?;
        register_git_status_commands(&mut runtime).map_err(ShellError::Runtime)?;

        let notes_id = runtime
            .model_mut()
            .create_buffer(workspace_id, "*notes*", BufferKind::Scratch, None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let scratch_id = runtime
            .model_mut()
            .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let errors_id = runtime
            .model_mut()
            .create_popup_buffer(workspace_id, "*errors*", BufferKind::Diagnostics, None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let (scratch, notes, primary_pane_id) = {
            let workspace = runtime
                .model()
                .workspace(workspace_id)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            let pane_id = workspace.active_pane_id().ok_or_else(|| {
                ShellError::Runtime("default workspace has no active pane".to_owned())
            })?;
            let scratch = workspace.buffer(scratch_id).ok_or_else(|| {
                ShellError::Runtime("scratch buffer missing after bootstrap".to_owned())
            })?;
            let notes = workspace.buffer(notes_id).ok_or_else(|| {
                ShellError::Runtime("notes buffer missing after bootstrap".to_owned())
            })?;
            (
                ShellBuffer::from_runtime_buffer(scratch, initial_scratch_lines(), &*user_library),
                ShellBuffer::from_runtime_buffer(notes, initial_notes_lines(), &*user_library),
                pane_id,
            )
        };

        let mut ui_state =
            ShellUiState::new(workspace_id, primary_pane_id, scratch, notes, notes_id);
        ui_state
            .ensure_buffer(
                errors_id,
                "*errors*",
                BufferKind::Diagnostics,
                &*user_library,
            )
            .replace_with_lines(initial_errors_lines(Some(&log_file_path)));
        runtime.services_mut().insert(ui_state);

        let log_dir_error = ensure_log_directory(&log_file_path).err();
        runtime.services_mut().insert(ErrorLog::new(
            errors_id,
            log_file_path,
            log_dir_error.is_none(),
        ));
        if let Some(error) = log_dir_error {
            record_runtime_error(&mut runtime, "error-log", error);
        }
        runtime.services_mut().insert(LspLogBufferState::default());
        runtime
            .services_mut()
            .insert(Mutex::new(TerminalBufferState::default()));
        runtime.services_mut().insert(FormatterRegistry::default());
        runtime.services_mut().insert(Mutex::new(JobManager::new()));
        acp::init_acp_manager(&mut runtime)?;
        runtime
            .services_mut()
            .insert(AutocompleteRegistry::from_user_config(&*user_library));
        runtime
            .services_mut()
            .insert(HoverRegistry::from_user_config(&*user_library));
        let mut lsp_registry = LanguageServerRegistry::new();
        lsp_registry
            .register_all(user_library.language_servers())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        runtime
            .services_mut()
            .insert(Arc::new(LspClientManager::new(lsp_registry)));
        let mut syntax_registry = SyntaxRegistry::new();
        syntax_registry
            .register_all(user_library.syntax_languages())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        runtime.services_mut().insert(syntax_registry);
        configure_syntax_refresh_worker(&mut runtime).map_err(ShellError::Runtime)?;
        let mut theme_registry = ThemeRegistry::new();
        theme_registry
            .register_all(user_library.themes())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        if let Err(error) =
            restore_saved_theme_selection(&mut theme_registry, &active_theme_state_path())
        {
            record_runtime_error(&mut runtime, "theme.restore", error);
        }
        runtime.services_mut().insert(theme_registry);
        runtime
            .services_mut()
            .insert(UserLibraryService(Arc::clone(&user_library)));
        load_auto_loaded_packages(&mut runtime, &user_library.packages())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        picker::ensure_picker_keybindings(&mut runtime).map_err(ShellError::Runtime)?;
        register_lsp_status_hooks(&mut runtime).map_err(ShellError::Runtime)?;

        Ok(Self {
            runtime,
            user_library,
            typing_profiler: profile_input_latency
                .then(|| TypingProfiler::new(default_typing_profile_log_path())),
            last_text_input_profile: None,
            last_text_input_at: None,
            pending_suppressed_text_input: None,
            mouse_drag: None,
            browser_host: BrowserHostService::new(),
        })
    }

    fn record_error(&mut self, source: &str, message: impl Into<String>) {
        record_runtime_error(&mut self.runtime, source, message);
    }

    fn record_shell_error(&mut self, source: &str, error: ShellError) {
        self.record_error(source, error.to_string());
    }

    fn begin_typing_frame(
        &self,
        frame_index: u32,
        frame_pacing_sleep: Duration,
    ) -> Option<ActiveTypingFrameProfile> {
        self.typing_profiler
            .as_ref()
            .map(|_| ActiveTypingFrameProfile::new(frame_index, frame_pacing_sleep))
    }

    fn record_typing_frame(&mut self, frame: TypingFrameProfile) {
        if let Some(profiler) = self.typing_profiler.as_mut() {
            profiler.record_frame(frame);
        }
    }

    fn take_last_text_input_profile(&mut self) -> Option<Duration> {
        self.last_text_input_profile.take()
    }

    fn secondary_refresh_deferred_for_typing(&self, now: Instant) -> bool {
        git_refresh_deferred_for_typing(self.last_text_input_at, now)
    }

    fn frame_pacing_deferred_for_typing(&self, now: Instant) -> bool {
        frame_pacing_deferred_for_typing(self.last_text_input_at, now)
    }

    fn finish_typing_profile(&mut self) -> Result<Option<TypingProfileSummary>, String> {
        self.typing_profiler
            .as_ref()
            .map(TypingProfiler::write_report)
            .transpose()
    }

    fn handle_event(
        &mut self,
        event: Event,
        render_width: u32,
        render_height: u32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<bool, ShellError> {
        let active_buffer =
            active_buffer_event_context(&self.runtime).map_err(ShellError::Runtime)?;
        let visible_rows = shell_buffer(&self.runtime, active_buffer.buffer_id)
            .map_err(ShellError::Runtime)?
            .viewport_lines();
        let page_rows = visible_rows as i32;
        let (input_mode, picker_visible) = {
            let ui = self.ui()?;
            (ui.input_mode(), ui.picker_visible())
        };
        match event {
            Event::Quit { .. } => return Ok(true),
            Event::MouseButtonDown {
                mouse_btn,
                clicks,
                x,
                y,
                ..
            } => {
                let mouse_x = x as i32;
                let mouse_y = y as i32;
                let now = Instant::now();
                if mouse_btn == MouseButton::Left
                    && let Some(action) = notification_action_at_point(
                        self.ui()?,
                        render_width,
                        render_height,
                        cell_width,
                        line_height,
                        now,
                        (mouse_x, mouse_y),
                    )
                {
                    match action {
                        NotificationAction::OpenAcpPermissionPicker { request_id } => {
                            acp::acp_open_permission_request(&mut self.runtime, request_id)
                                .map_err(ShellError::Runtime)?;
                        }
                    }
                    return Ok(false);
                }
                if picker_visible {
                    return Ok(false);
                }
                let runtime_popup = self.runtime_popup()?;
                let popup_height = runtime_popup
                    .as_ref()
                    .map(|_| popup_window_height(render_height, line_height))
                    .unwrap_or(0);
                let pane_height = render_height.saturating_sub(popup_height);
                let browser_plan = browser_sync_plan(
                    self.ui()?,
                    runtime_popup.as_ref(),
                    render_width,
                    render_height,
                    cell_width,
                    line_height,
                    Instant::now(),
                )?;
                let clicked_browser_buffer =
                    browser_surface_buffer_at_point(&browser_plan, mouse_x, mouse_y);
                if runtime_popup.is_some() && mouse_y >= pane_height as i32 {
                    self.mouse_drag = None;
                    if let Some(popup) = runtime_popup.as_ref() {
                        let ui = self.ui_mut()?;
                        ui.set_popup_buffer(popup.active_buffer);
                        ui.set_popup_focus(true);
                    }
                    if let Some(buffer_id) = clicked_browser_buffer {
                        self.browser_host
                            .focus_buffer(buffer_id)
                            .map_err(ShellError::Runtime)?;
                    } else {
                        self.browser_host
                            .focus_parent()
                            .map_err(ShellError::Runtime)?;
                    }
                    return Ok(false);
                }
                if let Some((pane_id, buffer_id, pane_rect)) =
                    self.pane_surface_at_point(render_width, pane_height, mouse_x, mouse_y)?
                {
                    self.focus_runtime_pane(pane_id)?;
                    if let Some(buffer_id) = clicked_browser_buffer {
                        self.browser_host
                            .focus_buffer(buffer_id)
                            .map_err(ShellError::Runtime)?;
                    } else {
                        self.browser_host
                            .focus_parent()
                            .map_err(ShellError::Runtime)?;
                    }
                    if mouse_btn == MouseButton::Left {
                        let kind = shell_buffer(&self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?
                            .kind
                            .clone();
                        if !buffer_is_browser(&kind) && !buffer_is_terminal(&kind) {
                            self.begin_mouse_selection(
                                buffer_id,
                                pane_rect,
                                mouse_x,
                                mouse_y,
                                clicks,
                                cell_width,
                                line_height,
                            )?;
                        } else {
                            self.mouse_drag = None;
                        }
                    } else {
                        self.mouse_drag = None;
                    }
                } else if mouse_btn == MouseButton::Left {
                    self.mouse_drag = None;
                }
            }
            Event::MouseMotion { x, y, .. } => {
                self.update_mouse_selection(x as i32, y as i32, cell_width, line_height)?;
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                self.finish_mouse_selection()?;
            }
            Event::MouseButtonUp { .. } => {}
            Event::MouseWheel {
                y,
                direction,
                mouse_x,
                mouse_y,
                ..
            } => {
                if picker_visible {
                    return Ok(false);
                }
                let wheel_delta = match direction {
                    MouseWheelDirection::Normal => y.round() as i32,
                    MouseWheelDirection::Flipped => -(y.round() as i32),
                    _ => y.round() as i32,
                };
                if wheel_delta == 0 {
                    return Ok(false);
                }
                let mouse_x = mouse_x as i32;
                let mouse_y = mouse_y as i32;
                let runtime_popup = self.runtime_popup()?;
                let popup_height = runtime_popup
                    .as_ref()
                    .map(|_| popup_window_height(render_height, line_height))
                    .unwrap_or(0);
                let pane_height = render_height.saturating_sub(popup_height);
                if runtime_popup.is_some() && mouse_y >= pane_height as i32 {
                    return Ok(false);
                }
                let browser_plan = browser_sync_plan(
                    self.ui()?,
                    runtime_popup.as_ref(),
                    render_width,
                    render_height,
                    cell_width,
                    line_height,
                    Instant::now(),
                )?;
                if browser_surface_buffer_at_point(&browser_plan, mouse_x, mouse_y).is_some() {
                    return Ok(false);
                }
                let Some((pane_id, _, _)) =
                    self.pane_surface_at_point(render_width, pane_height, mouse_x, mouse_y)?
                else {
                    return Ok(false);
                };
                self.focus_runtime_pane(pane_id)?;
                self.browser_host
                    .focus_parent()
                    .map_err(ShellError::Runtime)?;
                let scroll_lines = wheel_delta.saturating_mul(MOUSE_WHEEL_SCROLL_LINES);
                let active_buffer_id =
                    active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                let active_kind = shell_buffer(&self.runtime, active_buffer_id)
                    .map_err(ShellError::Runtime)?
                    .kind
                    .clone();
                if buffer_is_terminal(&active_kind) {
                    scroll_active_terminal_view(
                        &mut self.runtime,
                        TerminalViewportScroll::LineDelta(scroll_lines),
                    )
                    .map_err(ShellError::Runtime)?;
                } else if !buffer_is_browser(&active_kind) {
                    scroll_buffer_viewport_only(
                        shell_buffer_mut(&mut self.runtime, active_buffer_id)
                            .map_err(ShellError::Runtime)?,
                        -scroll_lines,
                    );
                }
            }
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                repeat: _,
                ..
            } => {
                let runtime_surface_before =
                    active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
                let is_ctrl_c = keymod.intersects(ctrl_mod()) && keycode == Keycode::C;
                let is_ctrl_k = keymod.intersects(ctrl_mod()) && keycode == Keycode::K;
                let is_ctrl_key = matches!(keycode, Keycode::LCtrl | Keycode::RCtrl);
                if active_buffer.is_git_commit {
                    let mut should_commit = false;
                    let mut should_cancel = false;
                    let mut consume = false;
                    {
                        let ui = self.ui_mut()?;
                        if ui.pending_ctrl_c.is_some() {
                            if is_ctrl_c {
                                ui.pending_ctrl_c = None;
                                should_commit = true;
                                consume = true;
                            } else if is_ctrl_k {
                                ui.pending_ctrl_c = None;
                                should_cancel = true;
                                consume = true;
                            } else if is_ctrl_key {
                                consume = true;
                            } else {
                                ui.pending_ctrl_c = None;
                            }
                        } else if is_ctrl_c {
                            ui.pending_ctrl_c = Some(Instant::now());
                            consume = true;
                        }
                    }
                    if should_commit || should_cancel {
                        if should_commit {
                            commit_git_buffer(&mut self.runtime, active_buffer.buffer_id)
                                .map_err(ShellError::Runtime)?;
                        } else {
                            cancel_git_commit_buffer(&mut self.runtime, active_buffer.buffer_id)
                                .map_err(ShellError::Runtime)?;
                        }
                        return Ok(false);
                    }
                    if consume {
                        return Ok(false);
                    }
                }
                if active_buffer.is_plugin_evaluatable {
                    let mut should_evaluate = false;
                    let mut consume = false;
                    {
                        let ui = self.ui_mut()?;
                        if ui.pending_ctrl_c.is_some() {
                            if is_ctrl_c {
                                ui.pending_ctrl_c = None;
                                should_evaluate = true;
                                consume = true;
                            } else if is_ctrl_key {
                                consume = true;
                            } else {
                                ui.pending_ctrl_c = None;
                            }
                        } else if is_ctrl_c {
                            ui.pending_ctrl_c = Some(Instant::now());
                            consume = true;
                        }
                    }
                    if should_evaluate {
                        evaluate_active_plugin_buffer(&mut self.runtime, active_buffer.buffer_id)
                            .map_err(ShellError::Runtime)?;
                        return Ok(false);
                    }
                    if consume {
                        return Ok(false);
                    }
                }
                if active_buffer.has_input && (active_buffer.is_acp || active_buffer.is_browser) {
                    let mut should_submit = false;
                    let mut consume = false;
                    {
                        let ui = self.ui_mut()?;
                        if ui.pending_ctrl_c.is_some() {
                            if is_ctrl_c {
                                ui.pending_ctrl_c = None;
                                should_submit = true;
                                consume = true;
                            } else if is_ctrl_key {
                                consume = true;
                            } else {
                                ui.pending_ctrl_c = None;
                            }
                        } else if is_ctrl_c {
                            ui.pending_ctrl_c = Some(Instant::now());
                            consume = true;
                        }
                    }
                    if should_submit {
                        submit_input_buffer(&mut self.runtime).map_err(ShellError::Runtime)?;
                        return Ok(false);
                    }
                    if consume {
                        return Ok(false);
                    }
                }
                if !is_ctrl_c
                    && !is_ctrl_k
                    && !is_ctrl_key
                    && let Ok(ui) = self.ui_mut()
                {
                    ui.pending_ctrl_c = None;
                }
                if !picker_visible
                    && active_buffer.is_browser
                    && browser_devtools_shortcut_requested(keycode, keymod)
                {
                    self.browser_host
                        .open_devtools(active_buffer.buffer_id)
                        .map_err(ShellError::Runtime)?;
                    return Ok(false);
                }
                if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                    && active_buffer.has_input
                    && input_field_paste_shortcut_requested(keycode, keymod)
                {
                    if let Some(text) = read_system_clipboard() {
                        paste_text_into_active_input_buffer(&mut self.runtime, &text)
                            .map_err(ShellError::Runtime)?;
                    }
                    return Ok(false);
                }
                if keymod.intersects(ctrl_mod())
                    && keycode == Keycode::J
                    && matches!(input_mode, InputMode::Insert | InputMode::Replace)
                    && active_buffer.has_input
                    && active_buffer.is_acp
                {
                    if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                        input.append_text("\n");
                    }
                    return Ok(false);
                }
                if self.handle_command_line_keydown(keycode, keymod)? {
                    return Ok(false);
                }
                if self.handle_focused_hover_keydown(keycode, keymod)? {
                    return Ok(false);
                }
                if self.handle_autocomplete_keydown(keycode, keymod)? {
                    return Ok(false);
                }
                if self.try_runtime_keybinding_cached(
                    keycode,
                    keymod,
                    input_mode,
                    picker_visible,
                    active_buffer.is_directory,
                )? {
                    self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
                    return Ok(false);
                }

                if keymod.intersects(ctrl_mod()) && keycode == Keycode::Q {
                    return Ok(true);
                }

                if picker_visible {
                    if matches!(keycode, Keycode::Return | Keycode::KpEnter) {
                        self.runtime
                            .execute_command("picker.submit")
                            .map_err(|error| ShellError::Runtime(error.to_string()))?;
                        self.sync_active_buffer().map_err(ShellError::Runtime)?;
                        return Ok(false);
                    }
                    if keycode == Keycode::Backspace
                        && let Some(picker) = self.ui_mut()?.picker_mut()
                    {
                        picker.backspace_query();
                        self.schedule_picker_search_refresh()?;
                    }
                    return Ok(false);
                }

                if active_buffer.is_terminal
                    && matches!(input_mode, InputMode::Insert | InputMode::Replace)
                    && !active_buffer.has_input
                    && let Some(terminal_key) = terminal_key_for_event(keycode, keymod)
                {
                    write_active_terminal_key(&mut self.runtime, terminal_key)
                        .map_err(ShellError::Runtime)?;
                    return Ok(false);
                }

                let mut refresh_autocomplete = false;
                let mut close_autocomplete = false;
                match keycode {
                    Keycode::Left => {
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                        {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.move_left();
                            }
                        } else {
                            let _ = self.active_buffer_mut()?.move_left();
                            refresh_autocomplete =
                                matches!(input_mode, InputMode::Insert | InputMode::Replace);
                        }
                    }
                    Keycode::Right => {
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                        {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.move_right();
                            }
                        } else {
                            let _ = self.active_buffer_mut()?.move_right();
                            refresh_autocomplete =
                                matches!(input_mode, InputMode::Insert | InputMode::Replace);
                        }
                    }
                    Keycode::Up => {
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                        {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.move_up();
                            }
                        } else if active_buffer.is_terminal {
                            scroll_active_terminal_view(
                                &mut self.runtime,
                                TerminalViewportScroll::LineDelta(1),
                            )
                            .map_err(ShellError::Runtime)?;
                        } else {
                            let _ = self.active_buffer_mut()?.move_up();
                            refresh_autocomplete =
                                matches!(input_mode, InputMode::Insert | InputMode::Replace);
                        }
                    }
                    Keycode::Down => {
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                        {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.move_down();
                            }
                        } else if active_buffer.is_terminal {
                            scroll_active_terminal_view(
                                &mut self.runtime,
                                TerminalViewportScroll::LineDelta(-1),
                            )
                            .map_err(ShellError::Runtime)?;
                        } else {
                            let _ = self.active_buffer_mut()?.move_down();
                            refresh_autocomplete =
                                matches!(input_mode, InputMode::Insert | InputMode::Replace);
                        }
                    }
                    Keycode::PageDown if active_buffer.is_terminal => {
                        scroll_active_terminal_view(
                            &mut self.runtime,
                            TerminalViewportScroll::PageDown,
                        )
                        .map_err(ShellError::Runtime)?;
                    }
                    Keycode::PageDown => self.active_buffer_mut()?.scroll_by(page_rows),
                    Keycode::PageUp if active_buffer.is_terminal => {
                        scroll_active_terminal_view(
                            &mut self.runtime,
                            TerminalViewportScroll::PageUp,
                        )
                        .map_err(ShellError::Runtime)?;
                    }
                    Keycode::PageUp => self.active_buffer_mut()?.scroll_by(-page_rows),
                    Keycode::Return | Keycode::KpEnter
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace) =>
                    {
                        if self.ui()?.vim().multicursor.is_some() && !active_buffer.has_input {
                            apply_multicursor_insert_text(
                                &mut self.runtime,
                                "\n",
                                matches!(input_mode, InputMode::Replace),
                            )
                            .map_err(ShellError::Runtime)?;
                            close_autocomplete = true;
                        } else if active_buffer.has_input {
                            submit_input_buffer(&mut self.runtime).map_err(ShellError::Runtime)?;
                        } else if !active_buffer.is_read_only {
                            let changed = {
                                let buffer = self.active_buffer_mut()?;
                                insert_markdown_table_row_at_cursor(buffer)
                            };
                            if let Some(changed) = changed {
                                if changed {
                                    self.mark_active_buffer_syntax_dirty()?;
                                }
                                close_autocomplete = true;
                            } else {
                                let (indent_size, use_tabs) = {
                                    let ui = self.ui()?;
                                    let buffer_id = active_shell_buffer_id(&self.runtime)
                                        .map_err(ShellError::Runtime)?;
                                    let language_id = ui
                                        .buffer(buffer_id)
                                        .and_then(|buffer| buffer.language_id());
                                    let theme_registry =
                                        self.runtime.services().get::<ThemeRegistry>();
                                    (
                                        theme_lang_indent(theme_registry, language_id),
                                        theme_lang_use_tabs(theme_registry, language_id),
                                    )
                                };
                                {
                                    let buffer = self.active_buffer_mut()?;
                                    buffer.insert_text("\n");
                                    format_current_line_indent(buffer, indent_size, use_tabs);
                                }
                                self.mark_active_buffer_syntax_dirty()?;
                                close_autocomplete = true;
                            }
                        }
                    }
                    Keycode::Backspace
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace) =>
                    {
                        if self.ui()?.vim().multicursor.is_some() && !active_buffer.has_input {
                            apply_multicursor_delete(&mut self.runtime, true)
                                .map_err(ShellError::Runtime)?;
                            refresh_autocomplete = true;
                        } else if active_buffer.has_input {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.backspace();
                            }
                            if active_buffer.is_acp {
                                acp::maybe_open_slash_completion(
                                    &mut self.runtime,
                                    active_buffer.buffer_id,
                                )
                                .map_err(ShellError::Runtime)?;
                                acp::refresh_acp_input_hint(
                                    &mut self.runtime,
                                    active_buffer.buffer_id,
                                )
                                .map_err(ShellError::Runtime)?;
                            }
                        } else if !active_buffer.is_read_only {
                            self.active_buffer_mut()?.backspace();
                            {
                                let buffer = self.active_buffer_mut()?;
                                let _ = format_markdown_table_at_cursor(buffer);
                            }
                            self.mark_active_buffer_syntax_dirty()?;
                            refresh_autocomplete = true;
                        }
                    }
                    Keycode::Delete
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace) =>
                    {
                        if self.ui()?.vim().multicursor.is_some() && !active_buffer.has_input {
                            apply_multicursor_delete(&mut self.runtime, false)
                                .map_err(ShellError::Runtime)?;
                            refresh_autocomplete = true;
                        } else if active_buffer.has_input {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.delete_forward();
                            }
                            if active_buffer.is_acp {
                                acp::maybe_open_slash_completion(
                                    &mut self.runtime,
                                    active_buffer.buffer_id,
                                )
                                .map_err(ShellError::Runtime)?;
                                acp::refresh_acp_input_hint(
                                    &mut self.runtime,
                                    active_buffer.buffer_id,
                                )
                                .map_err(ShellError::Runtime)?;
                            }
                        } else if !active_buffer.is_read_only {
                            self.active_buffer_mut()?.delete_forward();
                            {
                                let buffer = self.active_buffer_mut()?;
                                let _ = format_markdown_table_at_cursor(buffer);
                            }
                            self.mark_active_buffer_syntax_dirty()?;
                            refresh_autocomplete = true;
                        }
                    }
                    Keycode::Tab => {
                        if !keymod.intersects(shift_mod())
                            && matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                            && active_buffer.is_acp
                        {
                            acp::acp_complete_slash(&mut self.runtime)
                                .map_err(ShellError::Runtime)?;
                        } else if !matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.is_git_status
                        {
                            toggle_git_section(&mut self.runtime).map_err(ShellError::Runtime)?;
                        } else if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && !active_buffer.has_input
                            && !active_buffer.is_read_only
                        {
                            let changed = {
                                let buffer = self.active_buffer_mut()?;
                                advance_markdown_table_insert_tab(buffer)
                            };
                            if let Some(changed) = changed {
                                if changed {
                                    self.mark_active_buffer_syntax_dirty()?;
                                }
                                close_autocomplete = true;
                            } else {
                                cycle_runtime_pane(&mut self.runtime)
                                    .map_err(ShellError::Runtime)?;
                                close_autocomplete = true;
                            }
                        } else {
                            cycle_runtime_pane(&mut self.runtime).map_err(ShellError::Runtime)?;
                            close_autocomplete = true;
                        }
                    }
                    Keycode::F2 => {
                        split_runtime_pane(&mut self.runtime, PaneSplitDirection::Horizontal)
                            .map_err(ShellError::Runtime)?;
                    }
                    _ => {}
                }
                if close_autocomplete {
                    self.ui_mut()?.close_autocomplete();
                } else if refresh_autocomplete {
                    self.schedule_autocomplete_refresh_if_active()?;
                }
            }
            Event::TextInput { text, .. } => {
                self.handle_text_input(&text)?;
            }
            _ => {}
        }

        Ok(false)
    }

    fn queue_suppressed_text_input_for_chord(&mut self, chord: &str) {
        const SUPPRESSED_TEXT_INPUT_WINDOW: Duration = Duration::from_millis(50);
        let suppressed = match chord {
            "Ctrl+Space" => Some(" "),
            _ => chord
                .strip_prefix("Ctrl+")
                .filter(|suffix| suffix.len() == 1)
                .filter(|suffix| {
                    suffix
                        .chars()
                        .all(|character| character.is_ascii_lowercase())
                }),
        };
        if let Some(text) = suppressed {
            self.pending_suppressed_text_input = Some(SuppressedTextInput {
                text: text.to_owned(),
                expires_at: Instant::now() + SUPPRESSED_TEXT_INPUT_WINDOW,
            });
        }
    }

    fn should_suppress_text_input(&mut self, text: &str) -> bool {
        self.pending_suppressed_text_input
            .take()
            .is_some_and(|pending| Instant::now() <= pending.expires_at && pending.text == text)
    }

    fn pane_surface_at_point(
        &self,
        width: u32,
        pane_height: u32,
        x: i32,
        y: i32,
    ) -> Result<Option<(PaneId, BufferId, PixelRect)>, ShellError> {
        if x < 0 || y < 0 || y >= pane_height as i32 {
            return Ok(None);
        }
        let ui = self.ui()?;
        let Some(panes) = ui.panes() else {
            return Ok(None);
        };
        let pane_rects = match ui.pane_split_direction() {
            PaneSplitDirection::Vertical => vertical_pane_rects(width, pane_height, panes.len()),
            PaneSplitDirection::Horizontal => {
                horizontal_pane_rects(width, pane_height, panes.len())
            }
        };
        Ok(panes
            .iter()
            .zip(pane_rects.iter())
            .find_map(|(pane, rect)| {
                pixel_rect_contains_point(*rect, x, y).then_some((
                    pane.pane_id,
                    pane.buffer_id,
                    *rect,
                ))
            }))
    }

    fn focus_runtime_pane(&mut self, pane_id: PaneId) -> Result<(), ShellError> {
        let workspace_id = self
            .runtime
            .model()
            .active_workspace_id()
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        self.runtime
            .model_mut()
            .focus_pane(workspace_id, pane_id)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let ui = self.ui_mut()?;
        ui.set_popup_focus(false);
        ui.focus_pane(pane_id);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_mouse_selection(
        &mut self,
        buffer_id: BufferId,
        rect: PixelRect,
        mouse_x: i32,
        mouse_y: i32,
        clicks: u8,
        cell_width: i32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let point = {
            let theme_registry = self.runtime.services().get::<ThemeRegistry>();
            let buffer = shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?;
            buffer_point_at_screen(
                buffer,
                PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
                &*shell_user_library(&self.runtime),
                theme_registry,
                mouse_x,
                mouse_y,
                cell_width,
                line_height,
                false,
            )
        };
        let Some(point) = point else {
            self.mouse_drag = None;
            return Ok(());
        };

        shell_buffer_mut(&mut self.runtime, buffer_id)
            .map_err(ShellError::Runtime)?
            .set_cursor(point);

        let kind = if clicks >= 2 {
            VisualSelectionKind::Line
        } else {
            if self.ui()?.input_mode() == InputMode::Visual {
                self.ui_mut()?.enter_normal_mode();
            }
            VisualSelectionKind::Character
        };
        if kind == VisualSelectionKind::Line {
            self.ui_mut()?.enter_visual_mode(point, kind);
        }
        self.mouse_drag = Some(MouseDragState {
            buffer_id,
            rect,
            anchor: point,
            kind,
        });
        Ok(())
    }

    fn update_mouse_selection(
        &mut self,
        mouse_x: i32,
        mouse_y: i32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let Some(drag) = self.mouse_drag else {
            return Ok(());
        };
        let point = {
            let theme_registry = self.runtime.services().get::<ThemeRegistry>();
            let buffer =
                shell_buffer(&self.runtime, drag.buffer_id).map_err(ShellError::Runtime)?;
            buffer_point_at_screen(
                buffer,
                PixelRectToRect::rect(drag.rect.x, drag.rect.y, drag.rect.width, drag.rect.height),
                &*shell_user_library(&self.runtime),
                theme_registry,
                mouse_x,
                mouse_y,
                cell_width,
                line_height,
                true,
            )
        };
        let Some(point) = point else {
            return Ok(());
        };
        let (input_mode, visual_anchor, visual_kind) = {
            let ui = self.ui()?;
            (
                ui.input_mode(),
                ui.vim().visual_anchor,
                ui.vim().visual_kind,
            )
        };
        if drag.kind == VisualSelectionKind::Character
            && point == drag.anchor
            && input_mode != InputMode::Visual
        {
            return Ok(());
        }
        if input_mode != InputMode::Visual
            || visual_anchor != Some(drag.anchor)
            || visual_kind != drag.kind
        {
            self.ui_mut()?.enter_visual_mode(drag.anchor, drag.kind);
        }
        shell_buffer_mut(&mut self.runtime, drag.buffer_id)
            .map_err(ShellError::Runtime)?
            .set_cursor(point);
        Ok(())
    }

    fn finish_mouse_selection(&mut self) -> Result<(), ShellError> {
        let Some(drag) = self.mouse_drag.take() else {
            return Ok(());
        };
        if drag.kind != VisualSelectionKind::Character {
            return Ok(());
        }
        let should_exit_visual = {
            let ui = self.ui()?;
            let buffer =
                shell_buffer(&self.runtime, drag.buffer_id).map_err(ShellError::Runtime)?;
            ui.input_mode() == InputMode::Visual
                && ui.vim().visual_anchor == Some(drag.anchor)
                && buffer.cursor_point() == drag.anchor
        };
        if should_exit_visual {
            self.ui_mut()?.enter_normal_mode();
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        target: &mut DrawTarget<'_>,
        fonts: &FontSet<'_>,
        width: u32,
        height: u32,
        cell_width: i32,
        line_height: i32,
        ascent: i32,
    ) -> Result<(), ShellError> {
        let runtime_popup = self.runtime_popup()?;
        let ui = self.ui()?;
        let acp_connected = acp::acp_connected(&self.runtime).unwrap_or(false);
        let lsp_workspace_loaded = active_lsp_workspace_loaded(&self.runtime, ui);
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
            fonts,
            ui,
            runtime_popup.as_ref(),
            &*self.user_library,
            &workspace_name,
            ui.attached_lsp_server(),
            lsp_workspace_loaded,
            acp_connected,
            theme_registry,
            width,
            height,
            cell_width,
            line_height,
            ascent,
            Instant::now(),
        )
    }

    fn sync_browser_hosts(
        &mut self,
        window: &Window,
        width: u32,
        height: u32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let runtime_popup = self.runtime_popup()?;
        let plan = browser_sync_plan(
            self.ui()?,
            runtime_popup.as_ref(),
            width,
            height,
            cell_width,
            line_height,
            Instant::now(),
        )?;
        let updates = self
            .browser_host
            .sync_window(window, &plan)
            .map_err(ShellError::Runtime)?;
        if !updates.is_empty() {
            apply_browser_location_updates(&mut self.runtime, &updates)
                .map_err(ShellError::Runtime)?;
        }
        let events = self
            .browser_host
            .drain_events()
            .map_err(ShellError::Runtime)?;
        if !events.is_empty() {
            self.apply_browser_host_events(&events)?;
        }
        Ok(())
    }

    fn apply_browser_host_events(&mut self, events: &[BrowserHostEvent]) -> Result<(), ShellError> {
        for event in events {
            match event {
                BrowserHostEvent::FocusParentRequested { .. } => {
                    self.browser_host
                        .focus_parent()
                        .map_err(ShellError::Runtime)?;
                    self.ui_mut()?.enter_normal_mode();
                }
                BrowserHostEvent::OpenDevtoolsRequested { buffer_id } => {
                    self.browser_host
                        .open_devtools(*buffer_id)
                        .map_err(ShellError::Runtime)?;
                }
            }
        }
        Ok(())
    }

    fn pane_count(&self) -> Result<usize, ShellError> {
        Ok(self.ui()?.pane_count())
    }

    pub(crate) fn picker_visible(&self) -> Result<bool, ShellError> {
        Ok(self.ui()?.picker_visible())
    }

    pub(crate) fn command_line_visible(&self) -> Result<bool, ShellError> {
        Ok(self.ui()?.command_line_visible())
    }

    fn popup_visible(&mut self) -> Result<bool, ShellError> {
        Ok(self.picker_visible()? || self.runtime_popup()?.is_some())
    }

    pub(crate) fn ui(&self) -> Result<&ShellUiState, ShellError> {
        shell_ui(&self.runtime).map_err(ShellError::Runtime)
    }

    fn ui_mut(&mut self) -> Result<&mut ShellUiState, ShellError> {
        shell_ui_mut(&mut self.runtime).map_err(ShellError::Runtime)
    }

    pub(crate) fn input_mode(&self) -> Result<InputMode, ShellError> {
        Ok(self.ui()?.input_mode())
    }

    pub(crate) fn active_buffer_mut(&mut self) -> Result<&mut ShellBuffer, ShellError> {
        let buffer_id = active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
        ensure_shell_buffer(&mut self.runtime, buffer_id).map_err(ShellError::Runtime)?;
        let ui = self.ui_mut()?;
        ui.buffer_mut(buffer_id)
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
                        if self.handle_pending_g_sequence(operator, line_target, chord)? {
                            return Ok(true);
                        }
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
                    let buffer_id =
                        active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                    let point = self.active_buffer_mut()?.cursor_point();
                    self.ui_mut()?
                        .vim_mut()
                        .marks
                        .insert(mark, VimMark { buffer_id, point });
                }
                self.ui_mut()?.vim_mut().clear_transient();
                Ok(true)
            }
            VimPending::MarkJump { linewise } => {
                if let Some(mark) = chord.chars().next() {
                    jump_to_mark(&mut self.runtime, mark, linewise).map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::MacroRecord => {
                if let Some(register) = chord.chars().next() {
                    start_macro_record(&mut self.runtime, register).map_err(ShellError::Runtime)?;
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
        let runtime_surface_before =
            active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;

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

        if self.try_plugin_buffer_keybinding(chord, vim_mode)? {
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
            self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
            self.clear_stale_vim_count()?;
        }

        Ok(())
    }

    fn handle_pending_g_sequence(
        &mut self,
        operator: Option<VimOperator>,
        line_target: Option<usize>,
        chord: &str,
    ) -> Result<bool, ShellError> {
        if operator.is_some() {
            return Ok(false);
        }
        let vim_mode = keymap_vim_mode(self.input_mode()?);
        let prefix = match self.ui()?.vim().pending_change_prefix.clone() {
            Some(VimRecordedInput::Chord(chord)) => chord,
            Some(VimRecordedInput::Text(text)) => text,
            None => "g".to_owned(),
        };
        let candidate = format!("{prefix} {chord}");
        if self
            .runtime
            .keymaps()
            .contains_for_mode(&KeymapScope::Workspace, vim_mode, &candidate)
        {
            let runtime_surface_before =
                active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
            self.ui_mut()?.vim_mut().pending_change_prefix = None;
            self.ui_mut()?.vim_mut().clear_transient();
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &candidate)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
            self.clear_stale_vim_count()?;
            return Ok(true);
        }

        let tokens = candidate
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if self.runtime.keymaps().has_sequence_prefix_for_mode(
            &KeymapScope::Workspace,
            vim_mode,
            &tokens,
        ) {
            self.ui_mut()?.vim_mut().pending_change_prefix =
                Some(VimRecordedInput::Chord(candidate));
            self.ui_mut()?.vim_mut().pending = Some(VimPending::GPrefix {
                operator,
                line_target,
            });
            return Ok(true);
        }

        Ok(false)
    }

    fn handle_key_sequence(
        &mut self,
        token: &str,
        scope: KeymapScope,
        vim_mode: KeymapVimMode,
    ) -> Result<bool, ShellError> {
        let mut tokens = take_key_sequence(&mut self.runtime)
            .map_err(ShellError::Runtime)?
            .unwrap_or_default();
        tokens.push(token.to_owned());
        let chord = tokens.join(" ");

        if self
            .runtime
            .keymaps()
            .contains_for_mode(&scope, vim_mode, &chord)
        {
            let runtime_surface_before =
                active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
            self.runtime
                .execute_key_binding_for_mode(&scope, vim_mode, &chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
            self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
            self.clear_stale_vim_count()?;
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if self
            .runtime
            .keymaps()
            .has_sequence_prefix_for_mode(&scope, vim_mode, &tokens)
        {
            set_key_sequence(&mut self.runtime, tokens).map_err(ShellError::Runtime)?;
            return Ok(true);
        }

        clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
        Ok(false)
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

    pub(crate) fn handle_text_input(&mut self, text: &str) -> Result<(), ShellError> {
        if self.should_suppress_text_input(text) {
            return Ok(());
        }
        self.last_text_input_at = Some(Instant::now());
        self.last_text_input_profile = None;
        let profile_started = self.typing_profiler.as_ref().map(|_| Instant::now());
        let hover_before = self.ui()?.hover().cloned();
        let result = self.handle_text_input_inner(text);
        let hover_changed = result.is_ok() && self.ui()?.hover().cloned() != hover_before;
        let result = result.and_then(|()| {
            if hover_changed {
                Ok(())
            } else {
                self.refresh_hover_state().map(|_| ())
            }
        });
        if let Some(profile_started) = profile_started {
            self.last_text_input_profile = Some(profile_started.elapsed());
        }
        result
    }

    fn handle_text_input_inner(&mut self, text: &str) -> Result<(), ShellError> {
        if self.command_line_visible()? {
            clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
            if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                command_line.append_text(text);
            }
            return Ok(());
        }
        if self.picker_visible()? {
            clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
            if let Some(picker) = self.ui_mut()?.picker_mut() {
                picker.append_query(text);
            }
            self.schedule_picker_search_refresh()?;
            return Ok(());
        }
        let active_buffer =
            active_buffer_event_context(&self.runtime).map_err(ShellError::Runtime)?;

        match self.input_mode()? {
            InputMode::Insert => {
                clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
                if self.ui()?.vim().multicursor.is_some() && !active_buffer.vim_targets_input {
                    apply_multicursor_insert_text(&mut self.runtime, text, false)
                        .map_err(ShellError::Runtime)?;
                    self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                    self.maybe_finish_change_after_input()?;
                    return Ok(());
                }
                if active_buffer.is_terminal {
                    write_active_terminal_text(&mut self.runtime, text)
                        .map_err(ShellError::Runtime)?;
                    return Ok(());
                }
                if active_buffer.vim_targets_input {
                    let buffer_id =
                        active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                    let handled = {
                        let buffer = self.active_buffer_mut()?;
                        if let Some(input) = buffer.input_field_mut() {
                            input.append_text(text);
                            true
                        } else {
                            false
                        }
                    };
                    if handled {
                        acp::maybe_open_slash_completion(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        acp::refresh_acp_input_hint(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                        self.maybe_finish_change_after_input()?;
                    }
                    return Ok(());
                }
                if active_shell_buffer_read_only(&self.runtime).map_err(ShellError::Runtime)? {
                    return Ok(());
                }
                let buffer_id =
                    active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                let (indent_size, use_tabs) = {
                    let ui = self.ui()?;
                    let language_id = ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
                    let theme_registry = self.runtime.services().get::<ThemeRegistry>();
                    (
                        theme_lang_indent(theme_registry, language_id),
                        theme_lang_use_tabs(theme_registry, language_id),
                    )
                };
                let normalized = normalize_tabs(text, indent_size, use_tabs);
                let defer_markdown_table_format = should_defer_table_format(
                    shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?,
                    text,
                );
                {
                    let buffer = self.active_buffer_mut()?;
                    if text == "}" {
                        dedent_block_end(buffer, indent_size);
                    }
                    buffer.insert_text(normalized.as_ref());
                    if !defer_markdown_table_format {
                        let _ = format_markdown_table_at_cursor(buffer);
                    }
                }
                self.mark_active_buffer_syntax_dirty()?;
                self.schedule_autocomplete_refresh_if_active()?;
                self.record_vim_input(VimRecordedInput::Text(normalized.to_string()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            InputMode::Replace => {
                clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
                if self.ui()?.vim().multicursor.is_some() && !active_buffer.vim_targets_input {
                    apply_multicursor_insert_text(&mut self.runtime, text, true)
                        .map_err(ShellError::Runtime)?;
                    self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                    self.maybe_finish_change_after_input()?;
                    return Ok(());
                }
                if active_buffer.is_terminal {
                    write_active_terminal_text(&mut self.runtime, text)
                        .map_err(ShellError::Runtime)?;
                    return Ok(());
                }
                if active_buffer.vim_targets_input {
                    let buffer_id =
                        active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                    let handled = {
                        let buffer = self.active_buffer_mut()?;
                        if let Some(input) = buffer.input_field_mut() {
                            input.append_text(text);
                            true
                        } else {
                            false
                        }
                    };
                    if handled {
                        acp::maybe_open_slash_completion(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        acp::refresh_acp_input_hint(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                        self.maybe_finish_change_after_input()?;
                    }
                    return Ok(());
                }
                if active_shell_buffer_read_only(&self.runtime).map_err(ShellError::Runtime)? {
                    return Ok(());
                }
                let buffer_id =
                    active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                let (indent_size, use_tabs) = {
                    let ui = self.ui()?;
                    let language_id = ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
                    let theme_registry = self.runtime.services().get::<ThemeRegistry>();
                    (
                        theme_lang_indent(theme_registry, language_id),
                        theme_lang_use_tabs(theme_registry, language_id),
                    )
                };
                let normalized = normalize_tabs(text, indent_size, use_tabs);
                let defer_markdown_table_format = should_defer_table_format(
                    shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?,
                    text,
                );
                {
                    let buffer = self.active_buffer_mut()?;
                    if text == "}" {
                        dedent_block_end(buffer, indent_size);
                    }
                    buffer.replace_mode_text(normalized.as_ref());
                    if !defer_markdown_table_format {
                        let _ = format_markdown_table_at_cursor(buffer);
                    }
                }
                self.mark_active_buffer_syntax_dirty()?;
                self.schedule_autocomplete_refresh_if_active()?;
                self.record_vim_input(VimRecordedInput::Text(normalized.to_string()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            _ => {}
        }

        if let Some(chord) = text_chord(text) {
            if self.handle_focused_hover_text_input(&chord)? {
                return Ok(());
            }
            let vim_mode = keymap_vim_mode(self.input_mode()?);
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && active_buffer.is_git_status
                && chord == "V"
                && self.runtime.keymaps().contains_for_mode(
                    &KeymapScope::Workspace,
                    vim_mode,
                    &chord,
                )
            {
                let runtime_surface_before =
                    active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
                self.runtime
                    .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
                    .map_err(|error| ShellError::Runtime(error.to_string()))?;
                self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
                self.clear_stale_vim_count()?;
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && handle_git_status_chord(&mut self.runtime, &chord)
                    .map_err(ShellError::Runtime)?
            {
                self.ui_mut()?.vim_mut().clear_transient();
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && handle_git_view_chord(&mut self.runtime, &chord).map_err(ShellError::Runtime)?
            {
                self.ui_mut()?.vim_mut().clear_transient();
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && handle_directory_chord(&mut self.runtime, &chord).map_err(ShellError::Runtime)?
            {
                self.ui_mut()?.vim_mut().clear_transient();
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && active_buffer.is_compilation
                && chord == "<CR>"
            {
                jump_to_compilation_error(&mut self.runtime).map_err(ShellError::Runtime)?;
                self.ui_mut()?.vim_mut().clear_transient();
                return Ok(());
            }
            if self.handle_vim_pending_text(&chord)? || self.handle_vim_count_input(&chord)? {
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }

            let token = normalize_text_token(&chord);
            if self.handle_key_sequence(&token, KeymapScope::Workspace, vim_mode)? {
                return Ok(());
            }

            if chord == "." && !self.picker_visible()? {
                self.repeat_last_change()?;
                return Ok(());
            }

            if self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
            {
                let runtime_surface_before =
                    active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
                self.runtime
                    .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
                    .map_err(|error| ShellError::Runtime(error.to_string()))?;
                self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
                self.clear_stale_vim_count()?;
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
            }
        }

        Ok(())
    }

    fn schedule_picker_search_refresh(&mut self) -> Result<(), ShellError> {
        enum DynamicPickerSearch {
            Vim {
                buffer_id: BufferId,
                buffer_revision: u64,
                text: TextSnapshot,
                direction: VimSearchDirection,
                query: String,
            },
            Workspace {
                root: PathBuf,
                query: String,
            },
        }

        let pending = {
            let ui = self.ui()?;
            let Some(picker) = ui.picker() else {
                return Ok(());
            };
            let query = picker.session().query().to_owned();
            if let Some(direction) = picker.vim_search_direction() {
                let buffer_id = ui
                    .active_buffer_id()
                    .ok_or_else(|| ShellError::Runtime("active buffer is missing".to_owned()))?;
                let buffer = ui.buffer(buffer_id).ok_or_else(|| {
                    ShellError::Runtime("active shell buffer is missing".to_owned())
                })?;
                Some(DynamicPickerSearch::Vim {
                    buffer_id,
                    buffer_revision: buffer.text.revision(),
                    text: buffer.text.snapshot(),
                    direction,
                    query,
                })
            } else {
                picker
                    .workspace_search_root()
                    .map(|root| DynamicPickerSearch::Workspace {
                        root: root.to_path_buf(),
                        query,
                    })
            }
        };

        match pending {
            Some(DynamicPickerSearch::Vim {
                buffer_id,
                buffer_revision,
                text,
                direction,
                query,
            }) => {
                self.ui_mut()?.vim_search_worker.schedule(
                    buffer_id,
                    buffer_revision,
                    text,
                    direction,
                    query,
                );
            }
            Some(DynamicPickerSearch::Workspace { root, query }) => {
                self.ui_mut()?.workspace_search_worker.schedule(root, query);
            }
            None => {}
        }

        Ok(())
    }

    fn refresh_pending_picker_searches(&mut self) -> Result<bool, ShellError> {
        let now = Instant::now();
        {
            let ui = self.ui_mut()?;
            ui.vim_search_worker.dispatch_due(now);
            ui.workspace_search_worker.dispatch_due(now);
        }

        let mut changed = false;
        if let Some(result) = self.ui()?.vim_search_worker.take_latest_result() {
            let should_apply = {
                let ui = self.ui()?;
                if let Some(picker) = ui.picker()
                    && let Some(buffer) = ui.buffer(result.buffer_id)
                {
                    picker.vim_search_direction() == Some(result.direction)
                        && picker.session().query() == result.query
                        && ui.active_buffer_id() == Some(result.buffer_id)
                        && buffer.text.revision() == result.buffer_revision
                        && result.request_id == ui.vim_search_worker.next_request_id
                } else {
                    false
                }
            };
            if should_apply
                && let Some(picker) = self.ui_mut()?.picker_mut()
                && picker.vim_search_direction() == Some(result.direction)
            {
                picker.set_entries(result.data.entries, result.data.selected_index);
                changed = true;
            }
        }

        if let Some(result) = self.ui()?.workspace_search_worker.take_latest_result() {
            let should_apply = {
                let ui = self.ui()?;
                if let Some(picker) = ui.picker()
                    && let Some(root) = picker.workspace_search_root()
                {
                    picker.session().query() == result.query
                        && root == result.root.as_path()
                        && result.request_id == ui.workspace_search_worker.next_request_id
                } else {
                    false
                }
            };
            if should_apply
                && let Some(picker) = self.ui_mut()?.picker_mut()
                && picker.workspace_search_root().is_some()
            {
                picker.set_entries(result.data.entries, result.data.selected_index);
                changed = true;
            }
        }

        Ok(changed)
    }

    fn schedule_autocomplete_refresh_if_active(&mut self) -> Result<(), ShellError> {
        let registry = self
            .runtime
            .services()
            .get::<AutocompleteRegistry>()
            .cloned()
            .ok_or_else(|| {
                ShellError::Runtime("autocomplete registry service missing".to_owned())
            })?;
        let lsp_client = self
            .runtime
            .services()
            .get::<Arc<LspClientManager>>()
            .cloned();
        let request = {
            let ui = self.ui()?;
            let Some(buffer_id) = ui.active_buffer_id() else {
                return Ok(());
            };
            let Some(buffer) = ui.buffer(buffer_id) else {
                return Ok(());
            };
            let root = if let Some(path) = buffer.path() {
                workspace_root_for_path(&self.runtime, path).map_err(ShellError::Runtime)?
            } else {
                None
            };
            autocomplete_request_for_buffer(
                buffer_id,
                buffer,
                root,
                &registry,
                lsp_client.clone(),
                false,
            )
        };
        let ui = self.ui_mut()?;
        match request {
            Some(request) => {
                if ui
                    .autocomplete()
                    .map(|autocomplete| autocomplete.buffer_id != request.buffer_id)
                    .unwrap_or(true)
                {
                    ui.set_autocomplete(AutocompleteOverlay::new(
                        request.buffer_id,
                        request.buffer_revision,
                        request.query.clone(),
                    ));
                } else if let Some(autocomplete) = ui.autocomplete_mut() {
                    autocomplete.mark_loading(request.buffer_revision, request.query.clone());
                }
                ui.autocomplete_worker.schedule(request);
            }
            None => ui.close_autocomplete(),
        }
        Ok(())
    }

    fn refresh_pending_autocomplete(&mut self) -> Result<bool, ShellError> {
        let now = Instant::now();
        self.ui_mut()?.autocomplete_worker.dispatch_due(now);
        let Some(result) = self.ui()?.autocomplete_worker.take_latest_result() else {
            return Ok(false);
        };
        let should_apply = {
            let ui = self.ui()?;
            if let Some(autocomplete) = ui.autocomplete()
                && let Some(buffer) = ui.buffer(result.buffer_id)
            {
                autocomplete.buffer_id == result.buffer_id
                    && result.buffer_revision >= autocomplete.buffer_revision
                    && ui.active_buffer_id() == Some(result.buffer_id)
                    && buffer.text.revision() == result.buffer_revision
                    && result.request_id == ui.autocomplete_worker.next_request_id
            } else {
                false
            }
        };
        if !should_apply {
            return Ok(false);
        }
        if result.entries.is_empty() {
            self.ui_mut()?.close_autocomplete();
            return Ok(true);
        }
        if let Some(autocomplete) = self.ui_mut()?.autocomplete_mut() {
            autocomplete.buffer_revision = result.buffer_revision;
            autocomplete.query = result.query;
            autocomplete.set_entries(result.entries);
            return Ok(true);
        }
        Ok(false)
    }

    fn refresh_hover_state(&mut self) -> Result<bool, ShellError> {
        let should_close = {
            let ui = self.ui()?;
            let Some(hover) = ui.hover() else {
                return Ok(false);
            };
            let Some(buffer) = ui.buffer(hover.buffer_id) else {
                return Ok(true);
            };
            buffer.cursor_point() != hover.anchor
        };
        if should_close {
            self.ui_mut()?.close_hover();
            return Ok(true);
        }
        Ok(false)
    }

    fn handle_focused_hover_keydown(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        if !self
            .ui()?
            .hover()
            .map(|hover| hover.focused)
            .unwrap_or(false)
        {
            return Ok(false);
        }
        match keycode {
            Keycode::Escape => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    hover.focused = false;
                    hover.clear_navigation_state();
                }
                Ok(true)
            }
            Keycode::N if keymod.intersects(ctrl_mod()) => {
                cycle_hover_provider(&mut self.runtime, true).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            Keycode::P if keymod.intersects(ctrl_mod()) => {
                cycle_hover_provider(&mut self.runtime, false).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            Keycode::Down => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover.take_count_or_one() as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::Up => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover.take_count_or_one() as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::PageDown => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::PageUp => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::D if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .half_page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::U if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .half_page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::F if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::B if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::E if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover.take_count_or_one() as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::Y if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover.take_count_or_one() as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::Home => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    hover.scroll_to_start();
                }
                Ok(true)
            }
            Keycode::End => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    hover.scroll_to_end();
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_focused_hover_text_input(&mut self, chord: &str) -> Result<bool, ShellError> {
        if matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace) {
            return Ok(false);
        }
        if !self
            .ui()?
            .hover()
            .map(|hover| hover.focused)
            .unwrap_or(false)
        {
            return Ok(false);
        }
        if chord.chars().count() != 1 {
            return Ok(false);
        }
        let Some(character) = chord.chars().next() else {
            return Ok(false);
        };
        let Some(hover) = self.ui_mut()?.hover_mut() else {
            return Ok(false);
        };
        match character {
            '1'..='9' => {
                hover.push_count_digit(character.to_digit(10).unwrap_or_default() as usize);
                Ok(true)
            }
            '0' => {
                if hover.count.is_some() {
                    hover.push_count_digit(0);
                } else {
                    hover.scroll_to_start();
                }
                Ok(true)
            }
            'j' => {
                let lines = hover.take_count_or_one() as i32;
                hover.pending_g_prefix = false;
                hover.scroll_by(lines);
                Ok(true)
            }
            'k' => {
                let lines = hover.take_count_or_one() as i32;
                hover.pending_g_prefix = false;
                hover.scroll_by(-lines);
                Ok(true)
            }
            'g' => {
                if hover.pending_g_prefix {
                    if let Some(line) = hover.take_count().map(|count| count.saturating_sub(1)) {
                        hover.scroll_to_line(line);
                    } else {
                        hover.scroll_to_start();
                    }
                } else {
                    hover.pending_g_prefix = true;
                }
                Ok(true)
            }
            'G' => {
                if let Some(line) = hover.take_count().map(|count| count.saturating_sub(1)) {
                    hover.scroll_to_line(line);
                } else {
                    hover.scroll_to_end();
                }
                Ok(true)
            }
            '{' | '(' | 'H' => {
                let lines = hover
                    .page_scroll_lines()
                    .saturating_mul(hover.take_count_or_one()) as i32;
                hover.pending_g_prefix = false;
                hover.scroll_by(-lines);
                Ok(true)
            }
            '}' | ')' | 'L' | '$' => {
                let lines = hover
                    .page_scroll_lines()
                    .saturating_mul(hover.take_count_or_one()) as i32;
                hover.pending_g_prefix = false;
                hover.scroll_by(lines);
                Ok(true)
            }
            'h' | 'l' | 'w' | 'W' | 'b' | 'B' | 'e' | 'E' | '^' | 'M' => {
                hover.clear_navigation_state();
                Ok(true)
            }
            _ => {
                hover.clear_navigation_state();
                Ok(false)
            }
        }
    }

    fn handle_autocomplete_keydown(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        let Some(chord) = keydown_chord(keycode, keymod) else {
            return Ok(false);
        };
        let handled = {
            let Some(autocomplete) = self.ui_mut()?.autocomplete_mut() else {
                return Ok(false);
            };
            if !autocomplete.is_visible() {
                return Ok(false);
            }
            if chord == AUTOCOMPLETE_NEXT_CHORD {
                autocomplete.select_next();
                true
            } else if chord == AUTOCOMPLETE_PREVIOUS_CHORD {
                autocomplete.select_previous();
                true
            } else {
                false
            }
        };
        if handled {
            self.queue_suppressed_text_input_for_chord(&chord);
        }
        Ok(handled)
    }

    fn handle_command_line_keydown(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        if !self.command_line_visible()? {
            return Ok(false);
        }
        match keycode {
            Keycode::Escape => {
                self.ui_mut()?.close_command_line();
                Ok(true)
            }
            Keycode::Return | Keycode::KpEnter => {
                submit_vim_command_line(&mut self.runtime).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            Keycode::Backspace => {
                if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                    command_line.backspace();
                }
                Ok(true)
            }
            Keycode::Delete => {
                if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                    command_line.delete_forward();
                }
                Ok(true)
            }
            Keycode::Left => {
                if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                    command_line.move_left();
                }
                Ok(true)
            }
            Keycode::Right => {
                if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                    command_line.move_right();
                }
                Ok(true)
            }
            Keycode::Tab => {
                cycle_vim_command_line_completion(
                    &mut self.runtime,
                    keymod.intersects(shift_mod()),
                )
                .map_err(ShellError::Runtime)?;
                Ok(true)
            }
            _ => Ok(keydown_chord(keycode, keymod).is_some()),
        }
    }

    #[cfg(test)]
    pub(crate) fn try_runtime_keybinding(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        if self.handle_command_line_keydown(keycode, keymod)? {
            return Ok(true);
        }
        let active_buffer =
            active_buffer_event_context(&self.runtime).map_err(ShellError::Runtime)?;
        let (input_mode, picker_visible) = {
            let ui = self.ui()?;
            (ui.input_mode(), ui.picker_visible())
        };
        self.try_runtime_keybinding_cached(
            keycode,
            keymod,
            input_mode,
            picker_visible,
            active_buffer.is_directory,
        )
    }

    #[cfg(test)]
    pub(crate) fn replace_active_buffer_text_for_test(
        &mut self,
        text: &str,
    ) -> Result<(), ShellError> {
        let lines = if text.is_empty() {
            Vec::new()
        } else {
            text.split('\n').map(str::to_owned).collect()
        };
        self.active_buffer_mut()?.replace_with_lines(lines);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn flush_picker_searches_for_test(&mut self) -> Result<(), ShellError> {
        const SEARCH_WAIT_STEP: Duration = Duration::from_millis(5);
        const SEARCH_WAIT_ATTEMPTS: usize = 40;

        {
            let ui = self.ui_mut()?;
            let due = Instant::now() + Duration::from_secs(1);
            ui.vim_search_worker.dispatch_due(due);
            ui.workspace_search_worker.dispatch_due(due);
        }

        for _ in 0..SEARCH_WAIT_ATTEMPTS {
            if self.refresh_pending_picker_searches()? {
                return Ok(());
            }
            std::thread::sleep(SEARCH_WAIT_STEP);
        }

        Ok(())
    }

    fn try_runtime_keybinding_cached(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
        input_mode: InputMode,
        picker_visible: bool,
        active_buffer_is_directory: bool,
    ) -> Result<bool, ShellError> {
        let Some(chord) = keydown_chord(keycode, keymod) else {
            return Ok(false);
        };

        clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
        let vim_mode = keymap_vim_mode(input_mode);
        let in_text_insert_mode = matches!(input_mode, InputMode::Insert | InputMode::Replace);
        let hover_visible = self.ui()?.hover().is_some();

        if !picker_visible && !in_text_insert_mode && chord == "Tab" {
            let handled = {
                let buffer_id =
                    active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                let buffer =
                    shell_buffer_mut(&mut self.runtime, buffer_id).map_err(ShellError::Runtime)?;
                advance_markdown_table_normal_tab(buffer).is_some()
            };
            if handled {
                self.queue_suppressed_text_input_for_chord(&chord);
                self.record_vim_input(VimRecordedInput::Chord(chord))?;
                self.maybe_finish_change_after_input()?;
                return Ok(true);
            }
        }

        if !picker_visible
            && !in_text_insert_mode
            && active_buffer_is_directory
            && handle_directory_keydown_chord(&mut self.runtime, &chord)
                .map_err(ShellError::Runtime)?
        {
            self.queue_suppressed_text_input_for_chord(&chord);
            self.ui_mut()?.vim_mut().clear_transient();
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if !picker_visible && !in_text_insert_mode && hover_visible && chord == "Tab" {
            trigger_hover_focus(&mut self.runtime).map_err(ShellError::Runtime)?;
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if !picker_visible
            && !in_text_insert_mode
            && hover_visible
            && matches!(chord.as_str(), HOVER_NEXT_CHORD | HOVER_PREVIOUS_CHORD)
            && self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if picker_visible
            && self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Popup, vim_mode, &chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Popup, vim_mode, &chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.queue_suppressed_text_input_for_chord(&chord);
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
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if self.try_plugin_buffer_keybinding(&chord, vim_mode)? {
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if !picker_visible
            && !in_text_insert_mode
            && self
                .runtime
                .keymaps()
                .contains_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
        {
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn try_plugin_buffer_keybinding(
        &mut self,
        chord: &str,
        vim_mode: KeymapVimMode,
    ) -> Result<bool, ShellError> {
        let buffer_id = match self.ui()?.active_buffer_id() {
            Some(buffer_id) => buffer_id,
            None => return Ok(false),
        };
        let plugin_kind = {
            let Some(buffer) = self.ui()?.buffer(buffer_id) else {
                return Ok(false);
            };
            match &buffer.kind {
                BufferKind::Plugin(kind) => kind.clone(),
                _ => return Ok(false),
            }
        };
        let binding = shell_user_library(&self.runtime)
            .plugin_buffer_key_bindings(&plugin_kind)
            .into_iter()
            .find(|binding| {
                matches!(
                    binding.scope(),
                    editor_plugin_api::PluginKeymapScope::Workspace
                        | editor_plugin_api::PluginKeymapScope::Global
                ) && plugin_vim_mode_matches(binding.vim_mode(), vim_mode)
                    && binding.chord() == chord
            });
        let Some(binding) = binding else {
            return Ok(false);
        };
        self.runtime
            .execute_command(binding.command_name())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        Ok(true)
    }

    #[cfg(test)]
    pub(crate) fn wait_for_autocomplete_results(&mut self) -> Result<(), ShellError> {
        for _ in 0..40 {
            self.ui_mut()?
                .autocomplete_worker
                .dispatch_due(Instant::now());
            if self.refresh_pending_autocomplete()? {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn autocomplete_visible(&self) -> Result<bool, ShellError> {
        Ok(self
            .ui()?
            .autocomplete()
            .is_some_and(AutocompleteOverlay::is_visible))
    }

    #[cfg(test)]
    pub(crate) fn command_line_text(&self) -> Result<Option<String>, ShellError> {
        Ok(self
            .ui()?
            .command_line()
            .map(|command_line| command_line.text().to_owned()))
    }

    #[cfg(test)]
    pub(crate) fn autocomplete_entries(&self) -> Result<Vec<String>, ShellError> {
        Ok(self
            .ui()?
            .autocomplete()
            .filter(|autocomplete| autocomplete.is_visible())
            .map(|autocomplete| {
                autocomplete
                    .entries()
                    .iter()
                    .map(|entry| entry.replacement.clone())
                    .collect()
            })
            .unwrap_or_default())
    }

    #[cfg(test)]
    pub(crate) fn autocomplete_selected(&self) -> Result<Option<String>, ShellError> {
        Ok(self
            .ui()?
            .autocomplete()
            .filter(|autocomplete| autocomplete.is_visible())
            .and_then(|autocomplete| {
                autocomplete
                    .selected()
                    .map(|entry| entry.replacement.clone())
            }))
    }

    #[cfg(test)]
    pub(crate) fn hover_visible(&self) -> Result<bool, ShellError> {
        Ok(self.ui()?.hover().is_some())
    }

    #[cfg(test)]
    pub(crate) fn hover_focused(&self) -> Result<bool, ShellError> {
        Ok(self
            .ui()?
            .hover()
            .map(|hover| hover.focused)
            .unwrap_or(false))
    }

    #[cfg(test)]
    pub(crate) fn hover_provider_label(&self) -> Result<Option<String>, ShellError> {
        Ok(self.ui()?.hover().and_then(|hover| {
            hover
                .current_provider()
                .map(|provider| provider.provider_label.clone())
        }))
    }

    #[cfg(test)]
    pub(crate) fn refresh_pending_file_reloads_for_test(&mut self) -> Result<bool, ShellError> {
        refresh_pending_file_reloads(&mut self.runtime, Instant::now(), true)
            .map_err(ShellError::Runtime)
    }

    fn sync_active_buffer(&mut self) -> Result<(), String> {
        sync_active_buffer(&mut self.runtime)
    }

    fn sync_active_buffer_if_surface_changed(
        &mut self,
        previous_surface: Option<(PaneId, BufferId)>,
    ) -> Result<(), ShellError> {
        let runtime_surface = active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
        if runtime_surface != previous_surface {
            self.sync_active_buffer().map_err(ShellError::Runtime)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn sync_active_viewport(
        &mut self,
        viewport_height: u32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let buffer_id = active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
        let visible_rows = {
            let buffer = shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?;
            buffer_visible_rows_for_height(buffer, viewport_height, line_height)
        };
        self.active_buffer_mut()?.set_viewport_lines(visible_rows);
        Ok(())
    }

    #[cfg(test)]
    fn active_viewport_height(
        &mut self,
        render_width: u32,
        render_height: u32,
        line_height: i32,
    ) -> Result<u32, ShellError> {
        let runtime_popup = self.runtime_popup()?;
        let ui = self.ui()?;
        if let Some(popup) = runtime_popup.as_ref()
            && ui.popup_focus_active(popup)
        {
            return Ok(popup_window_height(render_height, line_height).max(1));
        }
        let popup_height = runtime_popup
            .as_ref()
            .map(|_| popup_window_height(render_height, line_height))
            .unwrap_or(0);
        let pane_height = render_height.saturating_sub(popup_height);
        let panes = ui
            .panes()
            .ok_or_else(|| ShellError::Runtime("active workspace view is missing".to_owned()))?;
        let pane_rects = match ui.pane_split_direction() {
            PaneSplitDirection::Vertical => {
                vertical_pane_rects(render_width, pane_height, panes.len())
            }
            PaneSplitDirection::Horizontal => {
                horizontal_pane_rects(render_width, pane_height, panes.len())
            }
        };
        let rect = pane_rects
            .get(ui.active_pane_index())
            .ok_or_else(|| ShellError::Runtime("active pane rect is missing".to_owned()))?;
        Ok(rect.height.max(1))
    }

    #[cfg(test)]
    fn sync_active_viewport_for_render_size(
        &mut self,
        render_width: u32,
        render_height: u32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let viewport_height =
            self.active_viewport_height(render_width, render_height, line_height)?;
        self.sync_active_viewport(viewport_height, line_height)
    }

    fn sync_visible_buffer_layouts(
        &mut self,
        render_width: u32,
        render_height: u32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let runtime_popup = self.runtime_popup()?;
        let ui = self.ui()?;
        let popup_height = runtime_popup
            .as_ref()
            .map(|_| popup_window_height(render_height, line_height))
            .unwrap_or(0);
        let pane_height = render_height.saturating_sub(popup_height);
        let panes = ui
            .panes()
            .ok_or_else(|| ShellError::Runtime("active workspace view is missing".to_owned()))?;
        let pane_rects = match ui.pane_split_direction() {
            PaneSplitDirection::Vertical => {
                vertical_pane_rects(render_width, pane_height, panes.len())
            }
            PaneSplitDirection::Horizontal => {
                horizontal_pane_rects(render_width, pane_height, panes.len())
            }
        };
        let mut visible_buffers = panes
            .iter()
            .zip(pane_rects.iter())
            .enumerate()
            .map(|(pane_index, (pane, rect))| {
                (
                    pane.buffer_id,
                    rect.width,
                    rect.height,
                    pane_index == ui.active_pane_index()
                        && !ui.picker_visible()
                        && !runtime_popup
                            .as_ref()
                            .map(|popup| ui.popup_focus_active(popup))
                            .unwrap_or(false),
                )
            })
            .collect::<Vec<_>>();
        if let Some(popup) = runtime_popup.as_ref() {
            visible_buffers.push((
                popup.active_buffer,
                render_width,
                popup_height.max(1),
                ui.popup_focus_active(popup),
            ));
        }
        for (buffer_id, width, height, active) in visible_buffers {
            let (
                language_id,
                visible_rows,
                is_acp,
                has_plugin_sections,
                reserved_top_rows,
                scrolloff,
            ) = {
                let theme_registry = self.runtime.services().get::<ThemeRegistry>();
                let buffer = self.ui()?.buffer(buffer_id).ok_or_else(|| {
                    ShellError::Runtime(format!("buffer `{buffer_id}` is missing"))
                })?;
                let visible_rows = buffer_visible_rows_for_height(buffer, height, line_height);
                let is_acp = buffer.is_acp_buffer();
                let has_plugin_sections = buffer.has_plugin_sections();
                (
                    buffer.language_id().map(str::to_owned),
                    visible_rows,
                    is_acp,
                    has_plugin_sections,
                    if active && !is_acp && !has_plugin_sections {
                        buffer_headerline_rows(
                            buffer,
                            active,
                            &*shell_user_library(&self.runtime),
                            theme_registry,
                            visible_rows,
                        )
                    } else {
                        0
                    },
                    if !is_acp && !has_plugin_sections {
                        theme_scrolloff(theme_registry)
                    } else {
                        0
                    },
                )
            };
            let wrap_cols = wrap_columns_for_width(width, cell_width);
            let indent_size = theme_lang_indent(
                self.runtime.services().get::<ThemeRegistry>(),
                language_id.as_deref(),
            );
            let buffer = self
                .ui_mut()?
                .buffer_mut(buffer_id)
                .ok_or_else(|| ShellError::Runtime(format!("buffer `{buffer_id}` is missing")))?;
            if is_acp {
                buffer.sync_acp_viewport_metrics(width, height, cell_width, line_height);
            } else if has_plugin_sections {
                buffer.sync_plugin_section_viewport_metrics(width, height, cell_width, line_height);
            } else {
                buffer.set_viewport_lines(visible_rows);
            }
            buffer.ensure_visible(
                visible_rows,
                wrap_cols,
                indent_size,
                reserved_top_rows,
                scrolloff,
            );
        }
        Ok(())
    }

    fn runtime_popup(&mut self) -> Result<Option<RuntimePopupSnapshot>, ShellError> {
        let popup = active_runtime_popup(&self.runtime).map_err(ShellError::Runtime)?;
        if let Some(popup) = popup.as_ref() {
            self.ui_mut()?.set_popup_buffer(popup.active_buffer);
            ensure_shell_buffer(&mut self.runtime, popup.active_buffer)
                .map_err(ShellError::Runtime)?;
            if ensure_terminal_session(&mut self.runtime, popup.active_buffer)
                .map_err(ShellError::Runtime)?
            {
                self.ui_mut()?.enter_insert_mode();
            }
        } else {
            let ui = self.ui_mut()?;
            ui.set_popup_focus(false);
            ui.clear_popup_buffer();
        }
        Ok(popup)
    }

    fn mark_active_buffer_syntax_dirty(&mut self) -> Result<(), ShellError> {
        self.active_buffer_mut()?.mark_syntax_dirty();
        Ok(())
    }

    fn refresh_pending_file_reloads(&mut self, now: Instant) -> Result<bool, ShellError> {
        refresh_pending_file_reloads(&mut self.runtime, now, false).map_err(ShellError::Runtime)
    }

    fn refresh_pending_syntax(&mut self) -> Result<SyntaxRefreshStats, ShellError> {
        self.active_buffer_mut()?.ensure_visible_syntax_window();
        refresh_pending_syntax(&mut self.runtime).map_err(ShellError::Runtime)
    }

    fn refresh_pending_git(&mut self, now: Instant, typing_active: bool) -> Result<(), ShellError> {
        refresh_pending_git(&mut self.runtime, now, typing_active).map_err(ShellError::Runtime)
    }

    fn refresh_pending_terminal(
        &mut self,
        render_width: u32,
        render_height: u32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<bool, ShellError> {
        refresh_pending_terminal(
            &mut self.runtime,
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(ShellError::Runtime)
    }

    fn refresh_pending_lsp(&mut self) -> Result<bool, ShellError> {
        refresh_pending_lsp(&mut self.runtime).map_err(ShellError::Runtime)
    }

    fn refresh_notifications(&mut self, now: Instant) -> Result<bool, ShellError> {
        Ok(self.ui_mut()?.prune_notifications(now))
    }

    fn refresh_pending_acp(
        &mut self,
        render_width: u32,
        render_height: u32,
        line_height: i32,
        cell_width: i32,
    ) -> Result<bool, ShellError> {
        self.sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)?;
        let active_buffer_id =
            active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
        let followed_output = {
            let buffer =
                shell_buffer(&self.runtime, active_buffer_id).map_err(ShellError::Runtime)?;
            buffer.has_input_field() && buffer.should_follow_output()
        };
        let changed = acp::refresh_pending_acp(&mut self.runtime).map_err(ShellError::Runtime)?;
        self.sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)?;
        if changed
            && followed_output
            && active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?
                == active_buffer_id
        {
            self.active_buffer_mut()?.scroll_output_to_end();
        }
        Ok(changed)
    }

    fn visual_refresh_key(
        &self,
        render_width: u32,
        render_height: u32,
        theme_settings: &ThemeRuntimeSettings,
        now: Instant,
    ) -> Result<ShellVisualRefreshKey, ShellError> {
        let ui = self.ui()?;
        let active_lsp_workspace_loaded = active_lsp_workspace_loaded(&self.runtime, ui);
        Ok(ShellVisualRefreshKey {
            render_width,
            render_height,
            theme_settings: theme_settings.clone(),
            git_summary_revision: ui.git_summary_revision(),
            git_fringe_revisions: ui
                .buffers
                .iter()
                .filter_map(|buffer| {
                    buffer
                        .git_fringe_revision()
                        .map(|revision| (buffer.id(), revision))
                })
                .collect(),
            lsp_diagnostics_revisions: ui
                .buffers
                .iter()
                .map(|buffer| (buffer.id(), buffer.lsp_diagnostics_revision()))
                .collect(),
            active_lsp_server: ui.attached_lsp_server().map(str::to_owned),
            active_lsp_workspace_loaded,
            notification_revision: ui.notification_revision(),
            notification_deadline: ui.notification_deadline(now),
            yank_flash_until: ui.yank_flash_deadline(now),
        })
    }
}

fn active_lsp_workspace_loaded(runtime: &EditorRuntime, ui: &ShellUiState) -> bool {
    let Some(path) = ui
        .active_buffer_id()
        .and_then(|buffer_id| ui.buffer(buffer_id))
        .and_then(ShellBuffer::path)
    else {
        return false;
    };
    runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .map(|manager| manager.has_live_sessions_for_path(path))
        .unwrap_or(false)
}

fn frame_pacing_remaining(frame_started: Instant, now: Instant) -> Duration {
    let elapsed = now
        .checked_duration_since(frame_started)
        .unwrap_or_else(|| Duration::from_secs(0));
    FRAME_PACING_TARGET_120FPS.saturating_sub(elapsed)
}

fn pace_frame_to_120fps(frame_started: Instant) -> Duration {
    let mut now = Instant::now();
    let mut remaining = frame_pacing_remaining(frame_started, now);
    if remaining.is_zero() {
        return Duration::from_secs(0);
    }
    let sleep_started = now;
    while !remaining.is_zero() {
        if remaining > FRAME_PACING_YIELD_THRESHOLD {
            std::thread::sleep(remaining.saturating_sub(FRAME_PACING_YIELD_THRESHOLD));
        } else {
            std::thread::yield_now();
        }
        now = Instant::now();
        remaining = frame_pacing_remaining(frame_started, now);
    }
    sleep_started.elapsed()
}

fn git_refresh_deferred_for_typing(last_text_input_at: Option<Instant>, now: Instant) -> bool {
    last_text_input_at
        .map(|last| {
            now.checked_duration_since(last)
                .unwrap_or_else(|| Duration::from_secs(0))
                < GIT_REFRESH_TYPING_IDLE_THRESHOLD
        })
        .unwrap_or(false)
}

fn frame_pacing_deferred_for_typing(last_text_input_at: Option<Instant>, now: Instant) -> bool {
    last_text_input_at
        .map(|last| {
            now.checked_duration_since(last)
                .unwrap_or_else(|| Duration::from_secs(0))
                < FRAME_PACING_TYPING_IDLE_THRESHOLD
        })
        .unwrap_or(false)
}

fn should_yield_after_typing_batch(
    text_input_events: usize,
    events_processed: usize,
    batch_started: Instant,
) -> bool {
    text_input_events > 0
        && (events_processed >= TYPING_EVENT_BATCH_LIMIT
            || batch_started.elapsed() >= TYPING_EVENT_BATCH_TIME_BUDGET)
}

/// Runs the SDL3 + SDL_ttf demo shell.
pub fn run_demo_shell(config: ShellConfig) -> Result<ShellSummary, ShellError> {
    let log_file_path = default_error_log_path();
    install_panic_hook(log_file_path.clone());

    let sdl_context = sdl3::init().map_err(|error| ShellError::Sdl(error.to_string()))?;
    let video = sdl_context
        .video()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    register_clipboard_context(video.clone());
    let ttf = sdl3::ttf::init().map_err(|error| ShellError::Sdl(error.to_string()))?;

    let user_library: Arc<dyn UserLibrary> = config
        .user_library
        .clone()
        .unwrap_or_else(|| Arc::new(NullUserLibrary));
    let mut state = ShellState::new_with_user_library(
        log_file_path,
        config.profile_input_latency,
        Arc::clone(&user_library),
    )?;
    let mut theme_settings =
        theme_runtime_settings(state.runtime.services().get::<ThemeRegistry>(), &config);
    let (mut fonts, mut font_path) = load_font_set(&ttf, &theme_settings, &*user_library)?;
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
    let mut line_height = fonts.primary().height().max(1) as usize;
    let mut ascent = fonts.primary().ascent();
    let mut cell_width = fonts.cell_width();

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();
    let mut text_texture_cache = TextTextureCache::new();
    let renderer_name = canvas.renderer_name.clone();
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let mut frames_rendered = 0;
    let mut last_scene: Option<Vec<DrawCommand>> = None;
    let mut last_visual_key: Option<ShellVisualRefreshKey> = None;

    enum FrameOutcome {
        Continue,
        Quit,
    }

    let mut frame_pacing_sleep = Duration::from_secs(0);
    loop {
        let frame_started = Instant::now();
        let frame_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> FrameOutcome {
                let mut typing_frame =
                    state.begin_typing_frame(frames_rendered, frame_pacing_sleep);
                let fonts_changed = match update_theme_runtime(
                    &ttf,
                    &state,
                    &config,
                    &mut theme_settings,
                    &mut fonts,
                    &mut font_path,
                    &mut text_texture_cache,
                    &mut line_height,
                    &mut ascent,
                    &mut cell_width,
                ) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.update-theme", error);
                        false
                    }
                };

                let (render_width, render_height) = match canvas.output_size() {
                    Ok(size) => size,
                    Err(error) => {
                        state.record_shell_error(
                            "shell.output-size",
                            ShellError::Sdl(error.to_string()),
                        );
                        return FrameOutcome::Continue;
                    }
                };
                if let Err(error) = state.sync_visible_buffer_layouts(
                    render_width,
                    render_height,
                    cell_width,
                    line_height as i32,
                ) {
                    state.record_shell_error("shell.sync-visible-buffer-layouts", error);
                }
                let mut had_events = false;
                let event_batch_started = Instant::now();
                let mut frame_polled_events = 0usize;
                let mut frame_text_input_events = 0usize;

                for event in event_pump.poll_iter() {
                    had_events = true;
                    frame_polled_events = frame_polled_events.saturating_add(1);
                    let profiled_event = typing_frame
                        .as_ref()
                        .map(|_| TypingEventMetadata::from_event(&event));
                    let event_started = typing_frame.as_ref().map(|_| Instant::now());
                    match state.handle_event(
                        event,
                        render_width,
                        render_height,
                        cell_width,
                        line_height as i32,
                    ) {
                        Ok(true) => return FrameOutcome::Quit,
                        Ok(false) => {}
                        Err(error) => state.record_shell_error("shell.handle-event", error),
                    }
                    if matches!(profiled_event, Some(TypingEventMetadata::TextInput { .. })) {
                        frame_text_input_events = frame_text_input_events.saturating_add(1);
                    }
                    if let Some(frame) = typing_frame.as_mut()
                        && let Some(profiled_event) = profiled_event.as_ref()
                        && let Some(event_started) = event_started
                    {
                        frame.record_event(
                            profiled_event,
                            event_started.elapsed(),
                            state.take_last_text_input_profile(),
                        );
                    }
                    if should_yield_after_typing_batch(
                        frame_text_input_events,
                        frame_polled_events,
                        event_batch_started,
                    ) {
                        break;
                    }
                }
                if had_events
                    && let Err(error) = state.sync_visible_buffer_layouts(
                        render_width,
                        render_height,
                        cell_width,
                        line_height as i32,
                    )
                {
                    state.record_shell_error("shell.sync-visible-buffer-layouts", error);
                }

                let refresh_now = Instant::now();
                let typing_active = state.secondary_refresh_deferred_for_typing(refresh_now);
                let text_texture_cache_mode = if typing_active {
                    TextTextureCacheMode::ReuseOnly
                } else {
                    TextTextureCacheMode::ReadWrite
                };
                let file_reload_changed = match state.refresh_pending_file_reloads(refresh_now) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.file-reload-refresh", error);
                        false
                    }
                };
                let picker_refresh_started = Instant::now();
                let picker_changed = match state.refresh_pending_picker_searches() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.picker-search-refresh", error);
                        false
                    }
                };
                let lsp_changed = match state.refresh_pending_lsp() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.lsp-refresh", error);
                        false
                    }
                };
                let notification_changed = match state.refresh_notifications(refresh_now) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.notification-refresh", error);
                        false
                    }
                };
                let autocomplete_changed = match state.refresh_pending_autocomplete() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.autocomplete-refresh", error);
                        false
                    }
                };
                let hover_changed = match state.refresh_hover_state() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.hover-refresh", error);
                        false
                    }
                };
                let terminal_changed = match state.refresh_pending_terminal(
                    render_width,
                    render_height,
                    cell_width,
                    line_height as i32,
                ) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.terminal-refresh", error);
                        false
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.picker_refresh = picker_refresh_started.elapsed();
                }
                let syntax_refresh_started = Instant::now();
                let syntax_stats = match state.refresh_pending_syntax() {
                    Ok(stats) => stats,
                    Err(error) => {
                        state.record_shell_error("shell.syntax-refresh", error);
                        SyntaxRefreshStats::default()
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.syntax_refresh = syntax_refresh_started.elapsed();
                    frame.syntax_worker_compute = syntax_stats.worker_compute;
                    frame.syntax_result_count = syntax_stats.result_count;
                    frame.syntax_highlight_spans = syntax_stats.highlight_spans;
                }
                let syntax_changed = syntax_stats.changed;
                let git_refresh_started = Instant::now();
                if let Err(error) = state.refresh_pending_git(refresh_now, typing_active) {
                    state.record_shell_error("shell.git-refresh", error);
                }
                if let Some(frame) = typing_frame.as_mut() {
                    frame.git_refresh = git_refresh_started.elapsed();
                }
                let acp_refresh_started = Instant::now();
                let acp_changed = match state.refresh_pending_acp(
                    render_width,
                    render_height,
                    line_height as i32,
                    cell_width,
                ) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.acp-refresh", error);
                        false
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.acp_refresh = acp_refresh_started.elapsed();
                }

                let visual_key = match state.visual_refresh_key(
                    render_width,
                    render_height,
                    &theme_settings,
                    Instant::now(),
                ) {
                    Ok(key) => key,
                    Err(error) => {
                        state.record_shell_error("shell.visual-refresh-key", error);
                        return FrameOutcome::Continue;
                    }
                };
                let should_render = last_scene.is_none()
                    || had_events
                    || fonts_changed
                    || file_reload_changed
                    || picker_changed
                    || lsp_changed
                    || notification_changed
                    || autocomplete_changed
                    || hover_changed
                    || terminal_changed
                    || syntax_changed
                    || acp_changed
                    || last_visual_key.as_ref() != Some(&visual_key);
                let presented_at = if should_render {
                    let mut scene = Vec::new();
                    let render_started = Instant::now();
                    if let Err(error) = state.render(
                        &mut DrawTarget::Scene(&mut scene),
                        &fonts,
                        render_width,
                        render_height,
                        cell_width,
                        line_height as i32,
                        ascent,
                    ) {
                        state.record_shell_error("shell.render", error);
                        return FrameOutcome::Continue;
                    }
                    if let Some(frame) = typing_frame.as_mut() {
                        frame.render = render_started.elapsed();
                    }
                    if fonts_changed || last_scene.as_ref() != Some(&scene) {
                        let present_started = Instant::now();
                        if let Err(error) = present_scene_to_canvas(
                            &mut canvas,
                            &texture_creator,
                            &mut text_texture_cache,
                            text_texture_cache_mode,
                            &fonts,
                            &scene,
                        ) {
                            state.record_shell_error("shell.present", error);
                        } else if let Some(frame) = typing_frame.as_mut() {
                            frame.present = present_started.elapsed();
                        }
                        last_scene = Some(scene);
                    }
                    last_visual_key = Some(visual_key);
                    Instant::now()
                } else {
                    Instant::now()
                };
                if let Err(error) = state.sync_browser_hosts(
                    canvas.window(),
                    render_width,
                    render_height,
                    cell_width,
                    line_height as i32,
                ) {
                    state.record_shell_error("shell.browser-host", error);
                }
                if let Some(frame) = typing_frame.take() {
                    state.record_typing_frame(frame.finish(frame_started.elapsed(), presented_at));
                }

                FrameOutcome::Continue
            }));

        match frame_result {
            Ok(FrameOutcome::Quit) => {
                return Ok(build_shell_summary(
                    &mut state,
                    frames_rendered,
                    renderer_name.clone(),
                    &font_path,
                ));
            }
            Ok(FrameOutcome::Continue) => {
                frames_rendered += 1;
                if let Some(frame_limit) = config.frame_limit
                    && frames_rendered >= frame_limit
                {
                    break;
                }
            }
            Err(payload) => {
                state.record_error("panic", panic_payload_message(payload));
            }
        }

        frame_pacing_sleep = if state.frame_pacing_deferred_for_typing(Instant::now()) {
            Duration::from_secs(0)
        } else {
            pace_frame_to_120fps(frame_started)
        };
    }

    Ok(build_shell_summary(
        &mut state,
        frames_rendered,
        renderer_name,
        &font_path,
    ))
}

#[allow(clippy::too_many_arguments)]
fn update_theme_runtime<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    state: &ShellState,
    config: &ShellConfig,
    theme_settings: &mut ThemeRuntimeSettings,
    fonts: &mut FontSet<'ttf>,
    font_path: &mut PathBuf,
    text_texture_cache: &mut TextTextureCache<'_>,
    line_height: &mut usize,
    ascent: &mut i32,
    cell_width: &mut i32,
) -> Result<bool, ShellError> {
    let updated = theme_runtime_settings(state.runtime.services().get::<ThemeRegistry>(), config);
    if &updated == theme_settings {
        return Ok(false);
    }

    let mut fonts_changed = false;
    if updated.font_size != theme_settings.font_size
        || updated.font_request != theme_settings.font_request
    {
        let (next_fonts, next_font_path) = load_font_set(ttf, &updated, &*state.user_library)?;
        *font_path = next_font_path;
        *fonts = next_fonts;
        text_texture_cache.clear();
        *line_height = fonts.primary().height().max(1) as usize;
        *ascent = fonts.primary().ascent();
        *cell_width = fonts.cell_width();
        fonts_changed = true;
    }

    *theme_settings = updated;
    Ok(fonts_changed)
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
    ThemeRuntimeSettings {
        font_request,
        font_size,
    }
}

fn resolve_font_path(request: Option<&str>) -> Result<PathBuf, ShellError> {
    if let Some(request) = request.and_then(|value| (!value.is_empty()).then_some(value))
        && let Some(path) = resolve_font_request(request)
    {
        return Ok(path);
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

fn asset_path_from_parts(base: &Path, parts: &[&str]) -> PathBuf {
    parts
        .iter()
        .fold(base.to_path_buf(), |candidate, part| candidate.join(part))
}

fn resolve_bundled_icon_font_dir() -> Result<PathBuf, ShellError> {
    let mut roots = Vec::new();
    if let Ok(exe_path) = env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        roots.extend(
            exe_dir
                .ancestors()
                .take(BUNDLED_ICON_FONT_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }
    if let Ok(cwd) = env::current_dir() {
        roots.extend(
            cwd.ancestors()
                .take(BUNDLED_ICON_FONT_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }
    for root in roots {
        for parts in BUNDLED_ICON_FONT_DIR_CANDIDATES {
            let candidate = asset_path_from_parts(&root, parts);
            if candidate.is_dir() {
                return Ok(candidate);
            }
        }
    }
    let candidates = BUNDLED_ICON_FONT_DIR_CANDIDATES
        .iter()
        .map(|parts| parts.join("\\"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(ShellError::Runtime(format!(
        "bundled icon font directory not found; looked for {candidates}"
    )))
}

fn resolve_bundled_icon_font_paths() -> Result<Vec<PathBuf>, ShellError> {
    let font_dir = resolve_bundled_icon_font_dir()?;
    BUNDLED_ICON_FONT_FILES
        .iter()
        .map(|name| {
            let path = font_dir.join(name);
            if path.is_file() {
                Ok(path)
            } else {
                Err(ShellError::Runtime(format!(
                    "bundled icon font `{name}` is missing from `{}`",
                    font_dir.display()
                )))
            }
        })
        .collect()
}

fn resolve_system_icon_font_paths() -> Vec<PathBuf> {
    SYSTEM_ICON_FONT_CANDIDATES
        .iter()
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .collect()
}

fn resolve_icon_font_paths() -> Result<Vec<PathBuf>, ShellError> {
    let mut paths = resolve_bundled_icon_font_paths()?;
    let mut seen = paths.iter().cloned().collect::<BTreeSet<_>>();
    for path in resolve_system_icon_font_paths() {
        if seen.insert(path.clone()) {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn validate_bundled_icon_fonts(
    fonts: &FontSet<'_>,
    user_library: &dyn UserLibrary,
) -> Result<(), ShellError> {
    let mut missing_count = 0usize;
    let mut examples = Vec::new();
    for symbol in user_library.icon_symbols() {
        let supported = symbol
            .glyph
            .chars()
            .all(|character| fonts.icon_font_index_for_char(character).is_some());
        if supported {
            continue;
        }
        missing_count += 1;
        if examples.len() < 12 {
            examples.push(format!("{} ({})", symbol.id(), symbol.codepoint_label()));
        }
    }
    if missing_count == 0 {
        return Ok(());
    }
    let loaded_fonts = fonts
        .icon_fonts()
        .iter()
        .map(|font| font.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(ShellError::Runtime(format!(
        "bundled icon font validation failed: {missing_count} exported icons are missing from the startup icon-font stack ({loaded_fonts}). examples: {}",
        examples.join(", ")
    )))
}

fn load_font_set<'ttf>(
    ttf: &'ttf sdl3::ttf::Sdl3TtfContext,
    settings: &ThemeRuntimeSettings,
    user_library: &dyn UserLibrary,
) -> Result<(FontSet<'ttf>, PathBuf), ShellError> {
    let primary_path = resolve_font_path(settings.font_request.as_deref())?;
    let primary_font_data: &'static [u8] = Box::leak(
        fs::read(&primary_path)
            .map_err(|error| {
                ShellError::Runtime(format!(
                    "failed to read primary font `{}`: {error}",
                    primary_path.display()
                ))
            })?
            .into_boxed_slice(),
    );
    let primary_raster_font = RasterFont::from_bytes(
        primary_font_data,
        fontdue::FontSettings {
            scale: settings.font_size.max(1) as f32,
            ..fontdue::FontSettings::default()
        },
    )
    .map_err(|error| {
        ShellError::Runtime(format!(
            "failed to parse primary font `{}`: {error}",
            primary_path.display()
        ))
    })?;
    let primary_shape_face = ShapeFace::from_slice(primary_font_data, 0).ok_or_else(|| {
        ShellError::Runtime(format!(
            "failed to parse shaping data for primary font `{}`",
            primary_path.display()
        ))
    })?;
    let primary = ttf
        .load_font(&primary_path, settings.font_size as f32)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let primary_pixel_size = primary_raster_font
        .horizontal_line_metrics(settings.font_size.max(1) as f32)
        .map(|metrics| metrics.ascent - metrics.descent)
        .filter(|height| *height > f32::EPSILON)
        .map(|height| settings.font_size.max(1) as f32 * primary.height().max(1) as f32 / height)
        .unwrap_or(settings.font_size.max(1) as f32);
    let cell_width = primary
        .size_of_char('M')
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0
        .max(1) as i32;
    let icon_fonts = resolve_icon_font_paths()?
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .unwrap_or("icon-font")
                .to_owned();
            let bytes = fs::read(&path).map_err(|error| {
                ShellError::Runtime(format!(
                    "failed to read bundled icon font `{}`: {error}",
                    path.display()
                ))
            })?;
            let raster_font = RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
                .map_err(|error| {
                    ShellError::Runtime(format!(
                        "failed to parse bundled icon font `{}`: {error}",
                        path.display()
                    ))
                })?;
            let font = ttf
                .load_font(&path, settings.font_size as f32)
                .map_err(|error| ShellError::Sdl(error.to_string()))?;
            Ok((name, font, raster_font))
        })
        .collect::<Result<Vec<_>, ShellError>>()?;
    let icon_chars = user_library
        .icon_symbols()
        .iter()
        .flat_map(|symbol| symbol.glyph.chars())
        .collect();
    let fonts = FontSet::new(FontSetInit {
        primary,
        primary_raster_font,
        primary_shape_face,
        primary_pixel_size,
        ligatures_enabled: user_library.ligature_config().enabled,
        icon_fonts,
        icon_chars,
        icon_pixel_size: settings.font_size as f32,
        cell_width,
    });
    validate_bundled_icon_fonts(&fonts, user_library)?;
    Ok((fonts, primary_path))
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
    register_hook(
        runtime,
        HOOK_VIM_COMMAND_LINE,
        "Opens the Vim command line under the active status line.",
    )?;
    register_hook(
        runtime,
        HOOK_BUFFER_SAVE,
        "Saves the active file-backed buffer.",
    )?;
    register_hook(runtime, HOOK_BUFFER_CLOSE, "Closes the active buffer.")?;
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
        HOOK_AUTOCOMPLETE_TRIGGER,
        "Opens autocomplete for the active insert buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_NEXT,
        "Moves to the next autocomplete suggestion.",
    )?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_PREVIOUS,
        "Moves to the previous autocomplete suggestion.",
    )?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_ACCEPT,
        "Accepts the selected autocomplete suggestion.",
    )?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_CANCEL,
        "Closes the active autocomplete window.",
    )?;
    register_hook(
        runtime,
        HOOK_HOVER_TOGGLE,
        "Shows or closes the hover overlay at the cursor without focusing it.",
    )?;
    register_hook(
        runtime,
        HOOK_HOVER_FOCUS,
        "Moves focus into the hover overlay at the cursor.",
    )?;
    register_hook(
        runtime,
        HOOK_HOVER_NEXT,
        "Moves to the next hover provider tab.",
    )?;
    register_hook(
        runtime,
        HOOK_HOVER_PREVIOUS,
        "Moves to the previous hover provider tab.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_DISCONNECT,
        "Disconnects the active ACP session.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PERMISSION_APPROVE,
        "Approves the latest ACP permission request.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PERMISSION_DENY,
        "Denies the latest ACP permission request.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PICK_SESSION,
        "Opens the ACP session picker for the active client.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_NEW_SESSION,
        "Creates a new ACP session for the active client in a new buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PICK_MODE,
        "Opens the ACP mode picker for the active session.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PICK_MODEL,
        "Opens the ACP model picker for the active session.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_CYCLE_MODE,
        "Cycles to the next ACP session mode.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_SWITCH_PANE,
        "Switches focus between the ACP plan and output panes.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_COMPLETE_SLASH,
        "Opens ACP slash command completion for the active input.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_FOCUS_INPUT,
        "Focuses the ACP input section.",
    )?;
    register_hook(
        runtime,
        HOOK_IMAGE_ZOOM_IN,
        "Zooms the active native image buffer in.",
    )?;
    register_hook(
        runtime,
        HOOK_IMAGE_ZOOM_OUT,
        "Zooms the active native image buffer out.",
    )?;
    register_hook(
        runtime,
        HOOK_IMAGE_ZOOM_RESET,
        "Resets the active native image buffer to its fitted zoom.",
    )?;
    register_hook(
        runtime,
        HOOK_IMAGE_TOGGLE_MODE,
        "Toggles the active SVG image buffer between preview and source mode.",
    )?;
    register_hook(
        runtime,
        HOOK_PDF_PREVIOUS_PAGE,
        "Moves the active PDF buffer to the previous page.",
    )?;
    register_hook(
        runtime,
        HOOK_PDF_NEXT_PAGE,
        "Moves the active PDF buffer to the next page.",
    )?;
    register_hook(
        runtime,
        HOOK_PDF_ROTATE_CLOCKWISE,
        "Rotates the active PDF page clockwise.",
    )?;
    register_hook(
        runtime,
        HOOK_PDF_DELETE_PAGE,
        "Deletes the active PDF page.",
    )?;
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
    register_hook(runtime, HOOK_POPUP_NEXT, "Cycles to the next popup buffer.")?;
    register_hook(
        runtime,
        HOOK_POPUP_PREVIOUS,
        "Cycles to the previous popup buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_PANE_SPLIT_HORIZONTAL,
        "Splits the active workspace horizontally.",
    )?;
    register_hook(
        runtime,
        HOOK_PANE_SPLIT_VERTICAL,
        "Splits the active workspace vertically.",
    )?;
    register_hook(
        runtime,
        HOOK_PANE_CLOSE,
        "Closes the currently focused split.",
    )?;
    register_hook(
        runtime,
        HOOK_PANE_SWITCH_SPLIT,
        "Swaps the current split positions.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WINDOW_LEFT,
        "Moves focus to the window on the left.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WINDOW_DOWN,
        "Moves focus to the window below.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WINDOW_UP,
        "Moves focus to the window above.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WINDOW_RIGHT,
        "Moves focus to the window on the right.",
    )?;
    register_hook(
        runtime,
        HOOK_GIT_STATUS_OPEN_POPUP,
        "Opens the git status buffer in the popup window.",
    )?;
    register_hook(
        runtime,
        HOOK_BROWSER_URL,
        "Detects a URL in the active buffer and opens it in the popup browser.",
    )?;
    register_hook(
        runtime,
        HOOK_BROWSER_FOCUS_INPUT,
        "Focuses the browser input section.",
    )?;
    register_hook(runtime, HOOK_GIT_DIFF_OPEN, "Opens the git diff buffer.")?;
    register_hook(runtime, HOOK_GIT_LOG_OPEN, "Opens the git log buffer.")?;
    register_hook(
        runtime,
        HOOK_GIT_STASH_LIST_OPEN,
        "Opens the git stash list buffer.",
    )?;
    register_hook(runtime, HOOK_OIL_OPEN, "Opens the oil directory buffer.")?;
    register_hook(
        runtime,
        HOOK_OIL_OPEN_PARENT,
        "Opens the oil parent directory buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_INPUT_SUBMIT,
        "Submits the active input buffer prompt.",
    )?;
    register_hook(
        runtime,
        HOOK_INPUT_CLEAR,
        "Clears the active input buffer prompt.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_EVALUATE,
        "Evaluates the active plugin buffer's input section and writes the output section.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_RUN_COMMAND,
        "Opens the compilation buffer and runs (or prompts for) the workspace build command.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_RERUN_COMMAND,
        "Re-runs the last build command for the active workspace.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_SWITCH_PANE,
        "Switches focus between the active plugin buffer's split panes.",
    )?;

    runtime
        .subscribe_hook(
            HOOK_PLUGIN_EVALUATE,
            "shell.plugin-evaluate",
            |event, runtime| {
                let buffer_id = event.buffer_id.unwrap_or(active_shell_buffer_id(runtime)?);
                evaluate_active_plugin_buffer(runtime, buffer_id)
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PLUGIN_SWITCH_PANE,
            "shell.plugin-switch-pane",
            |event, runtime| switch_active_plugin_pane(runtime, event.buffer_id),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PLUGIN_RUN_COMMAND,
            "shell.plugin-run-command",
            |event, runtime| open_compile_buffer(runtime, event.detail.as_deref()),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PLUGIN_RERUN_COMMAND,
            "shell.plugin-rerun-command",
            |_, runtime| rerun_compile_command(runtime),
        )
        .map_err(|error| error.to_string())?;
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
            let is_terminal = active_shell_buffer_is_terminal(runtime)?;
            if active_shell_buffer_read_only(runtime)?
                && !active_shell_buffer_has_input(runtime)?
                && !is_terminal
            {
                report_read_only(runtime, "insert mode blocked");
                return Ok(());
            }
            if is_terminal {
                shell_ui_mut(runtime)?.enter_insert_mode();
                return Ok(());
            }
            if active_shell_buffer_has_input(runtime)? {
                let buffer_id = active_shell_buffer_id(runtime)?;
                let buffer = shell_buffer_mut(runtime, buffer_id)?;
                if buffer_is_acp(&buffer.kind) {
                    let _ = buffer.focus_acp_input();
                } else if buffer_is_browser(&buffer.kind) {
                    let _ = buffer.focus_browser_input();
                }
                shell_ui_mut(runtime)?.set_active_vim_target(VimTarget::Input);
            }
            start_change_recording(runtime)?;
            mark_change_finish_on_normal(runtime)?;
            shell_ui_mut(runtime)?.enter_insert_mode();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MODE_NORMAL, "shell.enter-normal-mode", |_, runtime| {
            let previous_mode = shell_ui(runtime)?.input_mode();
            let buffer_id = active_shell_buffer_id(runtime)?;
            let is_directory = buffer_is_directory(&shell_buffer(runtime, buffer_id)?.kind);
            let cursor_point = active_shell_buffer_mut(runtime)?.cursor_point();
            let has_input = active_shell_buffer_has_input(runtime)?;
            let targeted_input = active_shell_buffer_vim_targets_input(runtime)?;
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
            apply_pending_block_insert(runtime)?;
            if has_input && previous_mode == InputMode::Normal && targeted_input {
                let ui = shell_ui_mut(runtime)?;
                ui.set_active_vim_target(VimTarget::Buffer);
                ui.enter_normal_mode();
                active_shell_buffer_mut(runtime)?.set_cursor(cursor_point);
                return Ok(());
            }
            shell_ui_mut(runtime)?.enter_normal_mode();
            active_shell_buffer_mut(runtime)?.set_cursor(cursor_point);
            if targeted_input
                && has_input
                && let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut()
            {
                if previous_mode == InputMode::Visual {
                    input.clear_selection();
                } else if matches!(previous_mode, InputMode::Insert | InputMode::Replace)
                    && input.cursor_char() > 0
                {
                    let _ = input.move_left();
                }
            }
            if let Some((anchor, head, kind)) = visual_snapshot {
                store_last_visual_selection(runtime, anchor, head, kind)?;
            }
            if finish_change {
                finish_change_recording(runtime)?;
            }
            if is_directory
                && matches!(previous_mode, InputMode::Insert | InputMode::Replace)
                && let Err(error) = apply_directory_edit_queue(runtime, buffer_id)
            {
                record_runtime_error(runtime, "oil.directory", error.clone());
                return Err(error);
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_VIM_EDIT, "shell.vim-edit", |event, runtime| {
            let detail = event.detail.as_deref().unwrap_or_default();
            if handle_terminal_vim_edit(runtime, detail)? {
                return Ok(());
            }
            if vim_edit_requires_write(detail)
                && active_shell_buffer_read_only(runtime)?
                && !vim_edit_targets_input(runtime, detail)?
            {
                let action = format!("{detail} blocked");
                report_read_only(runtime, &action);
                return Ok(());
            }
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
                    if shell_ui(runtime)?.vim().multicursor.is_some() {
                        let ui = shell_ui_mut(runtime)?;
                        ui.input_mode = InputMode::Replace;
                        ui.vim_mut().clear_transient();
                        return Ok(());
                    }
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        shell_ui_mut(runtime)?.enter_replace_mode();
                        return Ok(());
                    }
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    shell_ui_mut(runtime)?.enter_replace_mode();
                }
                "toggle-case" => {
                    toggle_case_chars(runtime)?;
                }
                "append" => {
                    if shell_ui(runtime)?.vim().multicursor.is_some() {
                        let offset = {
                            let state = shell_ui(runtime)?
                                .vim()
                                .multicursor
                                .as_ref()
                                .ok_or_else(|| "multicursor state is missing".to_owned())?;
                            if state.match_text.is_empty() {
                                0
                            } else {
                                state
                                    .cursor_offset
                                    .saturating_add(1)
                                    .min(state.match_text.chars().count())
                            }
                        };
                        set_multicursor_cursor_offset(runtime, offset)?;
                        let ui = shell_ui_mut(runtime)?;
                        ui.input_mode = InputMode::Insert;
                        ui.vim_mut().clear_transient();
                        return Ok(());
                    }
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                            let _ = input.move_right();
                        }
                        shell_ui_mut(runtime)?.enter_insert_mode();
                        return Ok(());
                    }
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.append_after_cursor();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "append-line-end" => {
                    if shell_ui(runtime)?.vim().multicursor.is_some() {
                        let offset = shell_ui(runtime)?
                            .vim()
                            .multicursor
                            .as_ref()
                            .map(|state| state.match_text.chars().count())
                            .unwrap_or_default();
                        set_multicursor_cursor_offset(runtime, offset)?;
                        let ui = shell_ui_mut(runtime)?;
                        ui.input_mode = InputMode::Insert;
                        ui.vim_mut().clear_transient();
                        return Ok(());
                    }
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                            input.move_line_end();
                        }
                        shell_ui_mut(runtime)?.enter_insert_mode();
                        return Ok(());
                    }
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.append_line_end();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "insert-line-start" => {
                    if shell_ui(runtime)?.vim().multicursor.is_some() {
                        set_multicursor_cursor_offset(runtime, 0)?;
                        let ui = shell_ui_mut(runtime)?;
                        ui.input_mode = InputMode::Insert;
                        ui.vim_mut().clear_transient();
                        return Ok(());
                    }
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                            input.move_line_start();
                        }
                        shell_ui_mut(runtime)?.enter_insert_mode();
                        return Ok(());
                    }
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.insert_line_start();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "open-line-below" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    let (indent_size, use_tabs) = {
                        let ui = shell_ui(runtime)?;
                        let buffer_id = active_shell_buffer_id(runtime)?;
                        let language_id =
                            ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
                        let theme_registry = runtime.services().get::<ThemeRegistry>();
                        (
                            theme_lang_indent(theme_registry, language_id),
                            theme_lang_use_tabs(theme_registry, language_id),
                        )
                    };
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.open_line_below();
                    format_current_line_indent(buffer, indent_size, use_tabs);
                    buffer.mark_syntax_dirty();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                "open-line-above" => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    let (indent_size, reference_indent) = {
                        let ui = shell_ui(runtime)?;
                        let buffer_id = active_shell_buffer_id(runtime)?;
                        let buffer = ui
                            .buffer(buffer_id)
                            .ok_or_else(|| "active buffer is missing".to_owned())?;
                        let language_id = buffer.language_id();
                        let theme_registry = runtime.services().get::<ThemeRegistry>();
                        let indent_size = theme_lang_indent(theme_registry, language_id);
                        let line = buffer.text.line(buffer.cursor_row()).unwrap_or_default();
                        (indent_size, leading_indent_string(&line, indent_size))
                    };
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.open_line_above();
                    apply_line_indent(buffer, buffer.cursor_row(), indent_size, &reference_indent);
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
                "multicursor-add-next-match" => {
                    add_next_multicursor_match(runtime)?;
                }
                "multicursor-select-all-matches" => {
                    add_next_multicursor_match(runtime)?;
                    while shell_ui(runtime)?.vim().multicursor.is_some() {
                        let before = shell_ui(runtime)?
                            .vim()
                            .multicursor
                            .as_ref()
                            .map(|state| state.ranges.len())
                            .unwrap_or_default();
                        add_next_multicursor_match(runtime)?;
                        let after = shell_ui(runtime)?
                            .vim()
                            .multicursor
                            .as_ref()
                            .map(|state| state.ranges.len())
                            .unwrap_or_default();
                        if after <= before {
                            break;
                        }
                    }
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
                    shell_ui_mut(runtime)?.vim_mut().pending =
                        Some(VimPending::MarkJump { linewise: true });
                }
                "goto-mark" => {
                    shell_ui_mut(runtime)?.vim_mut().pending =
                        Some(VimPending::MarkJump { linewise: false });
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
        .subscribe_hook(
            HOOK_WORKSPACE_FORMAT,
            "shell.workspace-format",
            |_, runtime| {
                format_workspace(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BUFFER_SAVE, "shell.buffer-save", |event, runtime| {
            let workspace_id = event
                .workspace_id
                .or_else(|| active_shell_workspace_id(runtime))
                .or_else(|| runtime.model().active_workspace_id().ok())
                .ok_or_else(|| "buffer.save hook missing workspace".to_owned())?;
            let buffer_id = event
                .buffer_id
                .or_else(|| active_shell_buffer_id(runtime).ok())
                .ok_or_else(|| "buffer.save hook missing buffer".to_owned())?;
            save_buffer(runtime, workspace_id, buffer_id)?;
            let _ = refresh_git_status_buffers(runtime);
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BUFFER_CLOSE, "shell.buffer-close", |event, runtime| {
            let buffer_id = event.buffer_id.unwrap_or(active_shell_buffer_id(runtime)?);
            close_buffer_with_prompt(runtime, buffer_id)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_SAVE,
            "shell.workspace-save",
            |event, runtime| {
                let workspace_id = event
                    .workspace_id
                    .or_else(|| active_shell_workspace_id(runtime))
                    .or_else(|| runtime.model().active_workspace_id().ok())
                    .ok_or_else(|| "workspace.save hook missing workspace".to_owned())?;
                save_workspace(runtime, workspace_id)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_OPEN, "shell.open-picker", |event, runtime| {
            let picker =
                picker::picker_overlay(runtime, event.detail.as_deref().unwrap_or("commands"))?;
            shell_ui_mut(runtime)?.set_picker(picker);
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_VIM_COMMAND_LINE,
            "shell.vim-command-line",
            |_, runtime| open_vim_command_line(runtime),
        )
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
            let picker_kind = shell_ui(runtime)?.picker_kind();
            shell_ui_mut(runtime)?.close_picker();
            if let Some(PickerKind::AcpPermission { request_id }) = picker_kind {
                acp::acp_permission_picker_closed(runtime, request_id)?;
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_TRIGGER,
            "shell.autocomplete-trigger",
            |_, runtime| {
                trigger_autocomplete(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_NEXT,
            "shell.autocomplete-next",
            |_, runtime| {
                if let Some(autocomplete) = shell_ui_mut(runtime)?.autocomplete_mut()
                    && autocomplete.is_visible()
                {
                    autocomplete.select_next();
                }
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_PREVIOUS,
            "shell.autocomplete-previous",
            |_, runtime| {
                if let Some(autocomplete) = shell_ui_mut(runtime)?.autocomplete_mut()
                    && autocomplete.is_visible()
                {
                    autocomplete.select_previous();
                }
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_ACCEPT,
            "shell.autocomplete-accept",
            |_, runtime| {
                accept_autocomplete(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_CANCEL,
            "shell.autocomplete-cancel",
            |_, runtime| {
                shell_ui_mut(runtime)?.close_autocomplete();
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_HOVER_TOGGLE, "shell.hover-toggle", |_, runtime| {
            trigger_hover_toggle(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_HOVER_FOCUS, "shell.hover-focus", |_, runtime| {
            trigger_hover_focus(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_HOVER_NEXT, "shell.hover-next", |_, runtime| {
            cycle_hover_provider(runtime, true)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_HOVER_PREVIOUS, "shell.hover-previous", |_, runtime| {
            cycle_hover_provider(runtime, false)?;
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
        .subscribe_hook(HOOK_POPUP_NEXT, "shell.popup-next", |_, runtime| {
            cycle_runtime_popup_buffer(runtime, true)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_POPUP_PREVIOUS, "shell.popup-previous", |_, runtime| {
            cycle_runtime_popup_buffer(runtime, false)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PANE_SPLIT_HORIZONTAL,
            "shell.pane-split-horizontal",
            |_, runtime| {
                split_runtime_pane(runtime, PaneSplitDirection::Horizontal)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PANE_SPLIT_VERTICAL,
            "shell.pane-split-vertical",
            |_, runtime| {
                split_runtime_pane(runtime, PaneSplitDirection::Vertical)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PANE_CLOSE, "shell.pane-close", |_, runtime| {
            close_runtime_pane(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PANE_SWITCH_SPLIT,
            "shell.pane-switch-split",
            |_, runtime| {
                switch_runtime_split(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WINDOW_LEFT,
            "shell.workspace-window-left",
            |_, runtime| {
                move_workspace_window(runtime, WindowMoveDirection::Left)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WINDOW_DOWN,
            "shell.workspace-window-down",
            |_, runtime| {
                move_workspace_window(runtime, WindowMoveDirection::Down)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WINDOW_UP,
            "shell.workspace-window-up",
            |_, runtime| {
                move_workspace_window(runtime, WindowMoveDirection::Up)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WINDOW_RIGHT,
            "shell.workspace-window-right",
            |_, runtime| {
                move_workspace_window(runtime, WindowMoveDirection::Right)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_GIT_STATUS_OPEN_POPUP,
            "shell.git-status-open-popup",
            |_, runtime| {
                open_git_status_popup(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BROWSER_URL, "shell.browser-url", |_, runtime| {
            open_detected_browser_url(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_BROWSER_FOCUS_INPUT,
            "shell.browser-focus-input",
            |_, runtime| {
                focus_browser_input_section(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_GIT_DIFF_OPEN, "shell.git-diff-open", |_, runtime| {
            open_git_diff_worktree(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_GIT_LOG_OPEN, "shell.git-log-open", |_, runtime| {
            open_git_log_current(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_GIT_STASH_LIST_OPEN,
            "shell.git-stash-list-open",
            |_, runtime| {
                open_git_stash_list_buffer(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_OIL_OPEN, "shell.oil-open", |_, runtime| {
            let root = oil_default_root(runtime)?;
            open_oil_directory(runtime, root)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_OIL_OPEN_PARENT,
            "shell.oil-open-parent",
            |_, runtime| {
                let root = oil_default_root(runtime)?;
                open_oil_directory(runtime, root)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(builtins::PANE_SWITCH, "shell.pane-switch", |_, runtime| {
            shell_ui_mut(runtime)?.close_autocomplete();
            refresh_git_status_if_active(runtime)?;
            ensure_directory_buffer(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            builtins::BUFFER_SWITCH,
            "shell.buffer-switch",
            |_, runtime| {
                shell_ui_mut(runtime)?.close_autocomplete();
                refresh_git_status_if_active(runtime)?;
                ensure_directory_buffer(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_INPUT_SUBMIT, "shell.input-submit", |_, runtime| {
            submit_input_buffer(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_INPUT_CLEAR, "shell.input-clear", |_, runtime| {
            clear_input_buffer(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_DISCONNECT, "shell.acp-disconnect", |_, runtime| {
            acp::acp_disconnect(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_PERMISSION_APPROVE,
            "shell.acp-permission-approve",
            |_, runtime| {
                acp::acp_permission_approve(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_PERMISSION_DENY,
            "shell.acp-permission-deny",
            |_, runtime| {
                acp::acp_permission_deny(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_PICK_SESSION,
            "shell.acp-pick-session",
            |_, runtime| {
                acp::acp_pick_session(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_NEW_SESSION,
            "shell.acp-new-session",
            |_, runtime| {
                acp::acp_new_session(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_PICK_MODE, "shell.acp-pick-mode", |_, runtime| {
            acp::acp_pick_mode(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_PICK_MODEL, "shell.acp-pick-model", |_, runtime| {
            acp::acp_pick_model(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_CYCLE_MODE, "shell.acp-cycle-mode", |_, runtime| {
            acp::acp_cycle_mode(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_SWITCH_PANE,
            "shell.acp-switch-pane",
            |_, runtime| {
                acp::acp_switch_pane(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_COMPLETE_SLASH,
            "shell.acp-complete-slash",
            |_, runtime| {
                acp::acp_complete_slash(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_FOCUS_INPUT,
            "shell.acp-focus-input",
            |_, runtime| {
                focus_acp_input_section(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_IMAGE_ZOOM_IN, "shell.image-zoom-in", |_, runtime| {
            zoom_active_image_buffer_in(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_IMAGE_ZOOM_OUT, "shell.image-zoom-out", |_, runtime| {
            zoom_active_image_buffer_out(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_IMAGE_ZOOM_RESET,
            "shell.image-zoom-reset",
            |_, runtime| {
                reset_active_image_buffer_zoom(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_IMAGE_TOGGLE_MODE,
            "shell.image-toggle-mode",
            |_, runtime| {
                toggle_active_image_buffer_mode(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PDF_PREVIOUS_PAGE,
            "shell.pdf-previous-page",
            |_, runtime| {
                pdf_previous_page(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PDF_NEXT_PAGE, "shell.pdf-next-page", |_, runtime| {
            pdf_next_page(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PDF_ROTATE_CLOCKWISE,
            "shell.pdf-rotate-clockwise",
            |_, runtime| {
                pdf_rotate_clockwise(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PDF_DELETE_PAGE,
            "shell.pdf-delete-page",
            |_, runtime| {
                pdf_delete_page(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_SUBMIT, "shell.picker-submit", |_, runtime| {
            let (action, query, picker_kind) = {
                let ui = shell_ui_mut(runtime)?;
                let action = ui
                    .picker()
                    .and_then(PickerOverlay::selected_action)
                    .ok_or_else(|| "picker has no selected item".to_owned())?;
                let query = ui
                    .picker()
                    .map(|picker| picker.session().query().to_owned())
                    .unwrap_or_default();
                let picker_kind = ui.picker_kind();
                ui.close_picker();
                (action, query, picker_kind)
            };

            match action {
                PickerAction::NoOp => {}
                PickerAction::ExecuteCommand(command_name) => {
                    runtime
                        .execute_command(&command_name)
                        .map_err(|error| error.to_string())?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::ApplyLspCodeAction {
                    workspace_id,
                    buffer_id,
                    path,
                    code_action,
                } => {
                    apply_lsp_code_action(runtime, workspace_id, buffer_id, &path, &code_action)?;
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
                PickerAction::CloseBuffer(buffer_id) => {
                    close_buffer_with_prompt(runtime, buffer_id)?;
                }
                PickerAction::CloseBufferSave(buffer_id) => {
                    close_buffer_save(runtime, buffer_id)?;
                }
                PickerAction::CloseBufferDiscard(buffer_id) => {
                    close_buffer_discard(runtime, buffer_id)?;
                }
                PickerAction::OpenFile(path) => {
                    open_workspace_file(runtime, &path)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::OpenFileLocation { path, target } => {
                    open_workspace_file_at(runtime, &path, target)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::OpenAcpClient(client_id) => {
                    acp::open_acp_client(runtime, &client_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::CreateWorkspaceFile { root } => {
                    create_workspace_file_from_query(runtime, &root, &query)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::ActivateTheme(theme_id) => {
                    {
                        let registry = runtime
                            .services_mut()
                            .get_mut::<ThemeRegistry>()
                            .ok_or_else(|| "theme registry service missing".to_owned())?;
                        registry
                            .activate(&theme_id)
                            .map_err(|error| error.to_string())?;
                    }
                    if let Err(error) =
                        write_saved_theme_selection(&active_theme_state_path(), &theme_id)
                    {
                        record_runtime_error(runtime, "theme.save", error);
                    }
                }
                PickerAction::UndoTreeNode { buffer_id, node_id } => {
                    apply_undo_tree_node(runtime, buffer_id, node_id)?;
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
                PickerAction::GitPushRemote(remote) => {
                    push_git_remote(runtime, &remote)?;
                }
                PickerAction::GitFetchRemote(remote) => {
                    fetch_git_remote(runtime, &remote)?;
                }
                PickerAction::GitBranchAction { action, branch } => match action {
                    GitBranchActionKind::Checkout => {
                        checkout_git_branch(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergePlain => {
                        merge_git_plain(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergeEdit => {
                        merge_git_edit(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergeNoCommit => {
                        merge_git_no_commit(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergeSquash => {
                        merge_git_squash(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergePreview => {
                        merge_git_preview(runtime, &branch)?;
                    }
                    GitBranchActionKind::RebaseOnto => {
                        rebase_git_onto(runtime, &branch)?;
                    }
                    GitBranchActionKind::RebaseInteractive => {
                        rebase_git_interactive_onto(runtime, &branch)?;
                    }
                },
                PickerAction::GitCommitAction { action, commit } => match action {
                    GitCommitActionKind::CherryPick => {
                        cherry_pick_git_commit(runtime, &commit)?;
                    }
                    GitCommitActionKind::CherryPickNoCommit => {
                        cherry_pick_git_commit_no_commit(runtime, &commit)?;
                    }
                    GitCommitActionKind::Revert => {
                        revert_git_commit(runtime, &commit)?;
                    }
                    GitCommitActionKind::RevertNoCommit => {
                        revert_git_commit_no_commit(runtime, &commit)?;
                    }
                    GitCommitActionKind::ResetMixed => {
                        reset_git_commit(runtime, &commit, GitResetMode::Mixed)?;
                    }
                    GitCommitActionKind::ResetSoft => {
                        reset_git_commit(runtime, &commit, GitResetMode::Soft)?;
                    }
                    GitCommitActionKind::ResetHard => {
                        reset_git_commit(runtime, &commit, GitResetMode::Hard)?;
                    }
                    GitCommitActionKind::ResetKeep => {
                        reset_git_commit(runtime, &commit, GitResetMode::Keep)?;
                    }
                },
                PickerAction::AcpInsertSlashCommand { buffer_id, command } => {
                    acp::acp_insert_slash_command(runtime, buffer_id, &command)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpLoadSession {
                    buffer_id,
                    session_id,
                } => {
                    acp::acp_load_session(runtime, buffer_id, &session_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpSetMode { buffer_id, mode_id } => {
                    acp::acp_set_mode(runtime, buffer_id, &mode_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpSetModel {
                    buffer_id,
                    model_id,
                } => {
                    acp::acp_set_model(runtime, buffer_id, &model_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpResolvePermission {
                    request_id,
                    option_id,
                } => {
                    acp::acp_resolve_permission_option(runtime, request_id, &option_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::CopyToClipboard(text) => {
                    write_system_clipboard(&text);
                }
            }

            if let Some(PickerKind::AcpPermission { request_id }) = picker_kind {
                acp::acp_permission_picker_submitted(runtime, request_id)?;
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
    if runtime.hooks().contains(HOOK_LSP_START) {
        runtime
            .subscribe_hook(
                HOOK_LSP_START,
                "shell.track-lsp-server",
                |event, runtime| start_lsp_for_active_buffer(runtime, event.detail.as_deref()),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_STOP) {
        runtime
            .subscribe_hook(HOOK_LSP_STOP, "shell.stop-lsp-server", |_, runtime| {
                stop_lsp_for_active_buffer(runtime)
            })
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_RESTART) {
        runtime
            .subscribe_hook(
                HOOK_LSP_RESTART,
                "shell.restart-lsp-server",
                |_, runtime| restart_lsp_for_active_buffer(runtime, None),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_LOG) {
        runtime
            .subscribe_hook(HOOK_LSP_LOG, "shell.open-lsp-log", |_, runtime| {
                open_lsp_log_buffer(runtime)
            })
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_DEFINITION) {
        runtime
            .subscribe_hook(HOOK_LSP_DEFINITION, "shell.lsp-definition", |_, runtime| {
                goto_lsp_definition(runtime)
            })
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_REFERENCES) {
        runtime
            .subscribe_hook(HOOK_LSP_REFERENCES, "shell.lsp-references", |_, runtime| {
                goto_lsp_references(runtime)
            })
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_IMPLEMENTATION) {
        runtime
            .subscribe_hook(
                HOOK_LSP_IMPLEMENTATION,
                "shell.lsp-implementation",
                |_, runtime| goto_lsp_implementation(runtime),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_CODE_ACTIONS) {
        runtime
            .subscribe_hook(
                HOOK_LSP_CODE_ACTIONS,
                "shell.lsp-code-actions",
                |_, runtime| open_lsp_code_actions(runtime),
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn start_lsp_for_active_buffer(
    runtime: &mut EditorRuntime,
    preferred_server_id: Option<&str>,
) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let manager = lsp_client;
    if let Some(server_id) = preferred_server_id {
        let supported = manager
            .registered_server_ids_for_path(&context.path)
            .into_iter()
            .any(|registered| registered == server_id);
        if !supported {
            return Err(format!(
                "language server `{server_id}` is not registered for `{}`",
                context.path.display()
            ));
        }
    } else if !manager.supports_path(&context.path) {
        return Err(format!(
            "no language server is registered for `{}`",
            context.path.display()
        ));
    }
    let labels = if let Some(server_id) = preferred_server_id {
        manager.start_buffer_server(
            &context.path,
            &context.text,
            context.revision,
            context.root.as_deref(),
            server_id,
        )
    } else {
        manager.sync_buffer(
            &context.path,
            &context.text,
            context.revision,
            context.root.as_deref(),
        )
    }
    .map_err(|error| error.to_string())?;
    if let Some(buffer) = shell_ui_mut(runtime)?.buffer_mut(context.buffer_id) {
        buffer.set_lsp_enabled(true);
    }
    let attached = (!labels.is_empty()).then(|| labels.join(", "));
    shell_ui_mut(runtime)?.set_attached_lsp_server(context.workspace_id, attached);
    Ok(())
}

fn stop_lsp_for_active_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    lsp_client
        .stop_buffer(&context.path)
        .map_err(|error| error.to_string())?;
    let ui = shell_ui_mut(runtime)?;
    if let Some(buffer) = ui.buffer_mut(context.buffer_id) {
        buffer.set_lsp_enabled(false);
        buffer.set_lsp_diagnostics(Vec::new());
    }
    ui.set_attached_lsp_server(context.workspace_id, None);
    Ok(())
}

fn restart_lsp_for_active_buffer(
    runtime: &mut EditorRuntime,
    preferred_server_id: Option<&str>,
) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let manager = lsp_client;
    if let Some(server_id) = preferred_server_id {
        let supported = manager
            .registered_server_ids_for_path(&context.path)
            .into_iter()
            .any(|registered| registered == server_id);
        if !supported {
            return Err(format!(
                "language server `{server_id}` is not registered for `{}`",
                context.path.display()
            ));
        }
    } else if !manager.supports_path(&context.path) {
        return Err(format!(
            "no language server is registered for `{}`",
            context.path.display()
        ));
    }
    let labels = manager
        .restart_buffer(
            &context.path,
            &context.text,
            context.revision,
            context.root.as_deref(),
            preferred_server_id,
        )
        .map_err(|error| error.to_string())?;
    let ui = shell_ui_mut(runtime)?;
    if let Some(buffer) = ui.buffer_mut(context.buffer_id) {
        buffer.set_lsp_enabled(true);
        buffer.set_lsp_diagnostics(Vec::new());
    }
    ui.set_attached_lsp_server(
        context.workspace_id,
        (!labels.is_empty()).then(|| labels.join(", ")),
    );
    Ok(())
}

fn open_lsp_log_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let server_id = preferred_lsp_log_server(runtime, workspace_id)?
        .ok_or_else(|| "no active LSP server log is available".to_owned())?;
    let buffer_id = ensure_lsp_log_buffer(runtime, workspace_id, &server_id)?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let ui = shell_ui_mut(runtime)?;
    ui.focus_buffer_in_active_pane(buffer_id);
    ui.enter_normal_mode();
    Ok(())
}

fn goto_lsp_definition(runtime: &mut EditorRuntime) -> Result<(), String> {
    navigate_to_lsp_locations(runtime, "Definitions", LspClientManager::definitions)
}

fn goto_lsp_references(runtime: &mut EditorRuntime) -> Result<(), String> {
    navigate_to_lsp_locations(runtime, "References", LspClientManager::references)
}

fn goto_lsp_implementation(runtime: &mut EditorRuntime) -> Result<(), String> {
    navigate_to_lsp_locations(
        runtime,
        "Implementations",
        LspClientManager::implementations,
    )
}

fn open_lsp_code_actions(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let range = active_lsp_code_action_range(runtime, context.buffer_id)?;
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let (labels, code_actions) = {
        let labels = lsp_client
            .sync_buffer(
                &context.path,
                &context.text,
                context.revision,
                context.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        let code_actions = lsp_client
            .code_actions(&context.path, range)
            .map_err(|error| error.to_string())?;
        (labels, code_actions)
    };
    sync_lsp_buffer_state(runtime, context.workspace_id, context.buffer_id, &labels)?;
    if code_actions.is_empty() {
        return Err("no code actions available at the cursor".to_owned());
    }
    let picker = lsp_code_actions_picker_overlay(
        context.workspace_id,
        context.buffer_id,
        &context.path,
        &code_actions,
    );
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

fn active_lsp_code_action_range(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<TextRange, String> {
    let ui = shell_ui(runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())?;
    match ui.visual_selection_for_buffer(buffer, true) {
        Some(VisualSelection::Range(range)) => Ok(range),
        Some(VisualSelection::Block(_)) => {
            Err("LSP code actions do not support block selections".to_owned())
        }
        None => {
            let cursor = buffer.cursor_point();
            Ok(TextRange::new(cursor, cursor))
        }
    }
}

fn navigate_to_lsp_locations(
    runtime: &mut EditorRuntime,
    title: &str,
    request: fn(&LspClientManager, &Path, TextPoint) -> Result<Vec<LspLocation>, LspClientError>,
) -> Result<(), String> {
    let context = active_lsp_buffer_context(runtime)?;
    let position = shell_buffer(runtime, context.buffer_id)?.cursor_point();
    let lsp_client = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())?;
    let (labels, locations) = {
        let labels = lsp_client
            .sync_buffer(
                &context.path,
                &context.text,
                context.revision,
                context.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        let locations =
            request(&lsp_client, &context.path, position).map_err(|error| error.to_string())?;
        (labels, locations)
    };
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(context.buffer_id) {
            buffer.set_lsp_enabled(true);
        }
        ui.set_attached_lsp_server(
            context.workspace_id,
            (!labels.is_empty()).then(|| labels.join(", ")),
        );
    }
    open_lsp_locations(runtime, title, locations)
}

fn open_lsp_locations(
    runtime: &mut EditorRuntime,
    title: &str,
    locations: Vec<LspLocation>,
) -> Result<(), String> {
    let Some(location) = locations.first() else {
        return Err(format!("no {} found at cursor", title.to_ascii_lowercase()));
    };
    if locations.len() == 1 {
        open_workspace_file_at(runtime, location.path(), location.range().start())?;
        sync_active_buffer(runtime)?;
        return Ok(());
    }
    let picker = lsp_locations_picker_overlay(runtime, title, &locations);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

fn ensure_lsp_log_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    server_id: &str,
) -> Result<BufferId, String> {
    if let Some(buffer_id) = runtime
        .services()
        .get::<LspLogBufferState>()
        .and_then(|state| state.buffer_id(workspace_id, server_id))
    {
        return Ok(buffer_id);
    }

    let snapshot = current_lsp_log_snapshot(runtime)?;
    let entries = lsp_log_entries_for_server(snapshot.entries(), server_id);
    let buffer_name = lsp_log_buffer_name(server_id);
    let buffer_id = runtime
        .model_mut()
        .create_popup_buffer(workspace_id, &buffer_name, BufferKind::Diagnostics, None)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(
            buffer_id,
            &buffer_name,
            BufferKind::Diagnostics,
            &*user_library,
        )
        .replace_with_lines_follow_output(lsp_log_buffer_lines(server_id, &entries));
    }
    runtime
        .services_mut()
        .get_mut::<LspLogBufferState>()
        .ok_or_else(|| "LSP log buffer service missing".to_owned())?
        .insert_buffer(workspace_id, server_id.to_owned(), buffer_id);
    Ok(buffer_id)
}

fn current_lsp_log_snapshot(runtime: &EditorRuntime) -> Result<LspLogSnapshot, String> {
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>() else {
        return Ok(LspLogSnapshot::default());
    };
    Ok(lsp_client.log_snapshot())
}

fn preferred_lsp_log_server(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<Option<String>, String> {
    if let Ok(context) = active_lsp_buffer_context(runtime)
        && let Some(manager) = runtime.services().get::<Arc<LspClientManager>>()
        && let Some(server_id) = manager
            .session_labels_for_path(&context.path)
            .into_iter()
            .next()
    {
        return Ok(Some(server_id));
    }
    if let Some(server_id) = shell_ui(runtime)?
        .attached_lsp_servers
        .get(&workspace_id)
        .and_then(|labels| labels.split(", ").next())
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
    {
        return Ok(Some(server_id));
    }
    Ok(runtime
        .services()
        .get::<LspLogBufferState>()
        .and_then(|state| state.buffers_for_workspace(workspace_id).into_iter().next())
        .map(|(server_id, _)| server_id))
}

fn lsp_log_buffer_name(server_id: &str) -> String {
    format!("{LSP_LOG_BUFFER_PREFIX}{server_id}*")
}

fn trigger_autocomplete(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let active_buffer_is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer_is_acp(&buffer.kind) && buffer.has_input_field()
    };
    if active_buffer_is_acp {
        return acp::acp_complete_slash(runtime);
    }

    let registry = runtime
        .services()
        .get::<AutocompleteRegistry>()
        .cloned()
        .ok_or_else(|| "autocomplete registry service missing".to_owned())?;
    let lsp_client = runtime.services().get::<Arc<LspClientManager>>().cloned();
    let request = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        if buffer.is_read_only() || buffer.has_input_field() {
            return Ok(());
        }
        let root = if let Some(path) = buffer.path() {
            workspace_root_for_path(runtime, path)?
        } else {
            None
        };
        autocomplete_request_for_buffer(buffer_id, buffer, root, &registry, lsp_client, true)
    };
    let Some(request) = request else {
        shell_ui_mut(runtime)?.close_autocomplete();
        return Ok(());
    };
    let overlay =
        AutocompleteOverlay::new(buffer_id, request.buffer_revision, request.query.clone());
    let ui = shell_ui_mut(runtime)?;
    ui.set_autocomplete(overlay);
    ui.autocomplete_worker.schedule(request);
    Ok(())
}

fn trigger_hover_toggle(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let same_anchor = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        if buffer.has_input_field() {
            return Ok(());
        }
        let cursor = buffer.cursor_point();
        ui.hover()
            .filter(|hover| hover.buffer_id == buffer_id && hover.anchor == cursor)
            .is_some()
    };
    if same_anchor {
        shell_ui_mut(runtime)?.close_hover();
        return Ok(());
    }
    show_hover_overlay(runtime, false)
}

fn trigger_hover_focus(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let same_anchor_focus = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        if buffer.has_input_field() {
            return Ok(());
        }
        let cursor = buffer.cursor_point();
        ui.hover()
            .filter(|hover| hover.buffer_id == buffer_id && hover.anchor == cursor)
            .map(|hover| hover.focused)
    };
    match same_anchor_focus {
        Some(true) => return Ok(()),
        Some(false) => {
            if let Some(hover) = shell_ui_mut(runtime)?.hover_mut() {
                hover.focused = true;
            }
            return Ok(());
        }
        None => {}
    }

    show_hover_overlay(runtime, true)
}

fn cycle_hover_provider(runtime: &mut EditorRuntime, next: bool) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let Some(hover) = shell_ui_mut(runtime)?.hover_mut() else {
        return Ok(());
    };
    if next {
        hover.select_next_provider();
    } else {
        hover.select_previous_provider();
    }
    Ok(())
}

fn show_hover_overlay(runtime: &mut EditorRuntime, focused: bool) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let registry = runtime
        .services()
        .get::<HoverRegistry>()
        .cloned()
        .ok_or_else(|| "hover registry service missing".to_owned())?;
    let lsp_client = runtime.services().get::<Arc<LspClientManager>>().cloned();
    let lsp_context = active_lsp_buffer_context(runtime).ok();
    let overlay = {
        let ui = shell_ui(runtime)?;
        let Some(buffer) = ui.buffer(buffer_id) else {
            return Ok(());
        };
        hover_overlay_for_buffer(
            buffer_id,
            buffer,
            &registry,
            lsp_client.as_ref(),
            lsp_context.as_ref(),
            &*shell_user_library(runtime),
        )
    };
    let ui = shell_ui_mut(runtime)?;
    if let Some(mut overlay) = overlay {
        overlay.focused = focused;
        ui.set_hover(overlay);
    } else {
        ui.close_hover();
    }
    Ok(())
}

fn accept_autocomplete(runtime: &mut EditorRuntime) -> Result<(), String> {
    let selected = {
        let ui = shell_ui(runtime)?;
        ui.autocomplete()
            .filter(|autocomplete| autocomplete.is_visible())
            .and_then(|autocomplete| autocomplete.selected().cloned())
    };
    let Some(selected) = selected else {
        return Ok(());
    };

    let buffer_id = active_shell_buffer_id(runtime)?;
    let ui = shell_ui_mut(runtime)?;
    let Some(buffer) = ui.buffer_mut(buffer_id) else {
        ui.close_autocomplete();
        return Ok(());
    };
    if buffer.is_read_only() || buffer.has_input_field() {
        ui.close_autocomplete();
        return Ok(());
    }
    let snapshot = buffer.text.snapshot();
    let Some(query) = autocomplete_query(&snapshot, true) else {
        ui.close_autocomplete();
        return Ok(());
    };
    buffer.replace_range(query.replace_range, &selected.replacement);
    buffer.mark_syntax_dirty();
    ui.close_autocomplete();
    Ok(())
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
        if let Ok(ui) = shell_ui(runtime) {
            if ui.popup_focus_active(&popup) {
                return Ok(popup.active_buffer);
            }
        } else {
            return Ok(popup.active_buffer);
        }
    }

    shell_ui(runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active shell buffer is missing".to_owned())
}

fn active_shell_workspace_id(runtime: &EditorRuntime) -> Option<WorkspaceId> {
    shell_ui(runtime).ok().map(ShellUiState::active_workspace)
}

fn active_shell_buffer_mut(runtime: &mut EditorRuntime) -> Result<&mut ShellBuffer, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    shell_ui_mut(runtime)?
        .buffer_mut(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())
}

fn active_shell_buffer_read_only(runtime: &EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    Ok(shell_buffer(runtime, buffer_id)?.is_read_only())
}

fn active_shell_buffer_has_input(runtime: &EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    Ok(shell_buffer(runtime, buffer_id)?.has_input_field())
}

fn zoom_active_image_buffer_in(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.image_zoom_in();
    Ok(())
}

fn zoom_active_image_buffer_out(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.image_zoom_out();
    Ok(())
}

fn reset_active_image_buffer_zoom(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.reset_image_zoom();
    Ok(())
}

fn toggle_active_image_buffer_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let switched_to_source = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if !buffer.toggle_svg_image_mode()? {
            return Ok(());
        }
        buffer.is_svg_source_mode()
    };
    if switched_to_source {
        queue_buffer_syntax_refresh(runtime, buffer_id)?;
    }
    Ok(())
}

fn enter_insert_mode_for_input_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let has_input = shell_ui(runtime)?
        .buffer(buffer_id)
        .map(ShellBuffer::has_input_field)
        .unwrap_or(false);
    if has_input {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if buffer_is_acp(&buffer.kind) {
            let _ = buffer.focus_acp_input();
        } else if buffer_is_browser(&buffer.kind) {
            let _ = buffer.focus_browser_input();
        }
        let ui = shell_ui_mut(runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    Ok(())
}

fn active_shell_buffer_vim_targets_input(runtime: &EditorRuntime) -> Result<bool, String> {
    Ok(shell_ui(runtime)?.active_buffer_targets_input())
}

fn vim_edit_targets_input(runtime: &EditorRuntime, detail: &str) -> Result<bool, String> {
    if !active_shell_buffer_has_input(runtime)? {
        return Ok(false);
    }
    Ok(match detail {
        "enter-replace-mode" | "append" | "append-line-end" | "insert-line-start" => {
            active_shell_buffer_vim_targets_input(runtime)?
        }
        _ => false,
    })
}

fn active_buffer_event_context(
    runtime: &EditorRuntime,
) -> Result<ActiveBufferEventContext, String> {
    let ui = shell_ui(runtime)?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())?;
    Ok(ActiveBufferEventContext {
        buffer_id,
        has_input: buffer.has_input_field(),
        vim_targets_input: ui.active_buffer_targets_input(),
        is_read_only: buffer.is_read_only(),
        is_git_status: buffer_is_git_status(&buffer.kind),
        is_git_commit: buffer_is_git_commit(&buffer.kind),
        is_acp: buffer_is_acp(&buffer.kind),
        is_directory: buffer_is_directory(&buffer.kind),
        is_browser: buffer_is_browser(&buffer.kind),
        is_terminal: buffer_is_terminal(&buffer.kind),
        is_plugin_evaluatable: plugin_evaluatable_kind(&buffer.kind, runtime),
        is_compilation: buffer_is_compilation(&buffer.kind),
    })
}

fn active_lsp_buffer_context(runtime: &EditorRuntime) -> Result<ActiveLspBufferContext, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    lsp_buffer_context(runtime, workspace_id, buffer_id)
}

fn lsp_buffer_context(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
) -> Result<ActiveLspBufferContext, String> {
    let ui = shell_ui(runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "active shell buffer is missing".to_owned())?;
    let path = buffer
        .path()
        .map(Path::to_path_buf)
        .ok_or_else(|| "active buffer does not have a file path for LSP".to_owned())?;
    Ok(ActiveLspBufferContext {
        workspace_id,
        buffer_id,
        path: path.clone(),
        text: buffer.text.text(),
        revision: buffer.text.revision(),
        root: workspace_root_for_path(runtime, &path)?,
    })
}

fn buffer_is_git_status(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STATUS_KIND)
}

fn buffer_is_git_commit(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_COMMIT_KIND)
}

fn buffer_is_acp(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND)
}

fn buffer_is_browser(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == BROWSER_KIND)
}

fn buffer_is_compilation(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Compilation)
}

/// Returns `true` when the user library has an evaluator for the given buffer
/// kind.  Used to decide whether Ctrl+c Ctrl+c should trigger evaluation.
fn plugin_evaluatable_kind(kind: &BufferKind, runtime: &EditorRuntime) -> bool {
    if let BufferKind::Plugin(plugin_kind) = kind {
        shell_user_library(runtime).supports_plugin_evaluate(plugin_kind)
    } else {
        false
    }
}

fn active_shell_buffer_is_terminal(runtime: &EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    Ok(buffer_is_terminal(&shell_buffer(runtime, buffer_id)?.kind))
}

fn buffer_is_directory(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Directory)
}

fn buffer_is_oil_preview(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == OIL_PREVIEW_KIND)
}

fn report_read_only(runtime: &mut EditorRuntime, action: &str) {
    let message = match active_shell_buffer_id(runtime)
        .ok()
        .and_then(|buffer_id| shell_buffer(runtime, buffer_id).ok())
    {
        Some(buffer) => format!("buffer `{}` is read-only; {action}", buffer.display_name()),
        None => format!("read-only buffer; {action}"),
    };
    record_runtime_error(runtime, "buffer.read-only", message);
}

fn vim_edit_requires_write(detail: &str) -> bool {
    matches!(
        detail,
        "delete-char"
            | "delete-char-before"
            | "delete-line-end"
            | "change-line-end"
            | "substitute-char"
            | "substitute-line"
            | "replace-char"
            | "enter-replace-mode"
            | "toggle-case"
            | "append"
            | "append-line-end"
            | "insert-line-start"
            | "open-line-below"
            | "open-line-above"
            | "undo"
            | "redo"
            | "start-delete-operator"
            | "start-change-operator"
            | "start-format-operator"
            | "put-after"
            | "put-before"
            | "visual-delete"
            | "visual-change"
            | "visual-format"
            | "visual-toggle-case"
            | "visual-lowercase"
            | "visual-uppercase"
            | "visual-block-insert"
            | "visual-block-append"
    )
}

fn handle_terminal_vim_edit(runtime: &mut EditorRuntime, detail: &str) -> Result<bool, String> {
    if !active_shell_buffer_is_terminal(runtime)? {
        return Ok(false);
    }
    match detail {
        "append" | "append-line-end" | "insert-line-start" | "open-line-below"
        | "open-line-above" | "substitute-char" | "substitute-line" => {
            shell_ui_mut(runtime)?.enter_insert_mode();
            Ok(true)
        }
        "enter-replace-mode" | "replace-char" => {
            shell_ui_mut(runtime)?.enter_replace_mode();
            Ok(true)
        }
        "put-after" => {
            put_yank(runtime, true)?;
            Ok(true)
        }
        "put-before" => {
            put_yank(runtime, false)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn shell_buffer(runtime: &EditorRuntime, buffer_id: BufferId) -> Result<&ShellBuffer, String> {
    shell_ui(runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))
}

fn shell_buffer_mut(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<&mut ShellBuffer, String> {
    shell_ui_mut(runtime)?
        .buffer_mut(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))
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
    (start_char..char_count).find(|&char_index| {
        char_at_index(buffer, char_index).map(|ch| normalize_search_char(ch, case_sensitive))
            == Some(target)
    })
}

fn fuzzy_match_end(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
) -> Option<usize> {
    if pattern.is_empty()
        || char_at_index(buffer, start_char).map(|ch| normalize_search_char(ch, case_sensitive))
            != Some(pattern[0])
    {
        return None;
    }

    let mut last_index = start_char;
    let mut next_index = start_char.saturating_add(1);
    for target in pattern.iter().skip(1) {
        let found = find_char_forward(buffer, next_index, *target, case_sensitive)?;
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
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive)
            else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.is_none_or(|(_, best_span)| span < best_span) {
                best = Some((char_index, span));
            }
        }
    }
    if best.is_some() {
        return best.map(|(start, _)| buffer.text.point_from_char_index(start));
    }

    if wrap {
        for char_index in 0..first_pass_start.min(max_start.saturating_add(1)) {
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive)
            else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.is_none_or(|(_, best_span)| span < best_span) {
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
        if best.is_none_or(|(_, best_span)| span < best_span) {
            best = Some((char_index, span));
        }
    }
    if best.is_some() {
        return best.map(|(start, _)| buffer.text.point_from_char_index(start));
    }

    if wrap && first_pass_start < max_start {
        for char_index in ((first_pass_start + 1)..=max_start).rev() {
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive)
            else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.is_none_or(|(_, best_span)| span < best_span) {
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
struct AutocompleteBufferRequest {
    buffer_id: BufferId,
    buffer_revision: u64,
    text: TextSnapshot,
    plugin_kind: Option<String>,
    path: Option<PathBuf>,
    root: Option<PathBuf>,
    cursor: TextPoint,
    query: AutocompleteQuery,
    providers: Vec<AutocompleteProviderSpec>,
    result_limit: usize,
    lsp_client: Option<Arc<LspClientManager>>,
}

struct PendingAutocompleteRequest {
    due_at: Instant,
    request: AutocompleteWorkerRequest,
}

struct AutocompleteWorkerRequest {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    text: TextSnapshot,
    plugin_kind: Option<String>,
    path: Option<PathBuf>,
    root: Option<PathBuf>,
    cursor: TextPoint,
    query: AutocompleteQuery,
    providers: Vec<AutocompleteProviderSpec>,
    result_limit: usize,
    lsp_client: Option<Arc<LspClientManager>>,
}

struct AutocompleteWorkerResult {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    query: AutocompleteQuery,
    entries: Vec<AutocompleteEntry>,
}

struct AutocompleteWorkerState {
    pending: Option<PendingAutocompleteRequest>,
    next_request_id: u64,
    request_tx: Sender<AutocompleteWorkerRequest>,
    results: Arc<Mutex<Vec<AutocompleteWorkerResult>>>,
}

impl AutocompleteWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<AutocompleteWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(mut request) = request_rx.recv() {
                while let Ok(newer_request) = request_rx.try_recv() {
                    request = newer_request;
                }
                let entries = autocomplete_entries(&request);
                if let Ok(mut results) = worker_results.lock() {
                    results.push(AutocompleteWorkerResult {
                        request_id: request.request_id,
                        buffer_id: request.buffer_id,
                        buffer_revision: request.buffer_revision,
                        query: request.query,
                        entries,
                    });
                } else {
                    return;
                }
            }
        });

        Self {
            pending: None,
            next_request_id: 0,
            request_tx,
            results,
        }
    }

    fn clear_pending(&mut self) {
        self.pending = None;
    }

    fn schedule(&mut self, request: AutocompleteBufferRequest) {
        let debounce = if cfg!(test) {
            Duration::from_millis(0)
        } else {
            Duration::from_millis(45)
        };
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.pending = Some(PendingAutocompleteRequest {
            due_at: Instant::now() + debounce,
            request: AutocompleteWorkerRequest {
                request_id: self.next_request_id,
                buffer_id: request.buffer_id,
                buffer_revision: request.buffer_revision,
                text: request.text,
                plugin_kind: request.plugin_kind,
                path: request.path,
                root: request.root,
                cursor: request.cursor,
                query: request.query,
                providers: request.providers,
                result_limit: request.result_limit,
                lsp_client: request.lsp_client,
            },
        });
    }

    fn dispatch_due(&mut self, now: Instant) {
        let Some(pending) = self.pending.as_ref() else {
            return;
        };
        if now < pending.due_at {
            return;
        }
        let request = self.pending.take().map(|pending| pending.request);
        if let Some(request) = request {
            let _ = self.request_tx.send(request);
        }
    }

    fn take_latest_result(&self) -> Option<AutocompleteWorkerResult> {
        let mut results = self.results.lock().ok()?;
        results.drain(..).next_back()
    }
}

#[derive(Debug)]
struct RankedAutocompleteEntry {
    entry: AutocompleteEntry,
    score: i64,
    provider_index: usize,
}

fn autocomplete_entries(request: &AutocompleteWorkerRequest) -> Vec<AutocompleteEntry> {
    let mut ranked = Vec::new();
    let mut satisfied_or_groups = BTreeSet::new();
    for (provider_index, provider) in request.providers.iter().enumerate() {
        if provider
            .or_group
            .as_ref()
            .is_some_and(|group| satisfied_or_groups.contains(group))
        {
            continue;
        }
        let entries = match provider.kind {
            AutocompleteProviderKind::Buffer => {
                buffer_autocomplete_entries(&request.text, &request.query, provider)
            }
            AutocompleteProviderKind::Lsp => {
                lsp_autocomplete_entries(request, &request.query, provider)
            }
            AutocompleteProviderKind::Manual => {
                manual_autocomplete_entries(&request.plugin_kind, &request.query, provider)
            }
        };
        if !entries.is_empty()
            && let Some(group) = provider.or_group.as_ref()
        {
            satisfied_or_groups.insert(group.clone());
        }
        ranked.extend(
            entries
                .into_iter()
                .map(|(entry, score)| RankedAutocompleteEntry {
                    entry,
                    score,
                    provider_index,
                }),
        );
    }
    ranked.sort_by(|left, right| {
        left.provider_index
            .cmp(&right.provider_index)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| {
                left.entry
                    .replacement
                    .chars()
                    .count()
                    .cmp(&right.entry.replacement.chars().count())
            })
            .then_with(|| left.entry.replacement.cmp(&right.entry.replacement))
    });
    if ranked.len() > request.result_limit {
        ranked.truncate(request.result_limit);
    }
    ranked.into_iter().map(|entry| entry.entry).collect()
}

fn buffer_autocomplete_entries(
    snapshot: &TextSnapshot,
    query: &AutocompleteQuery,
    provider: &AutocompleteProviderSpec,
) -> Vec<(AutocompleteEntry, i64)> {
    let text = snapshot.text();
    let counts = collect_autocomplete_token_counts(&text);
    let prefix_lower = query.prefix.to_ascii_lowercase();
    counts
        .into_iter()
        .filter_map(|(token, frequency)| {
            let token_lower = token.to_ascii_lowercase();
            if !prefix_lower.is_empty() && !token_lower.starts_with(&prefix_lower) {
                return None;
            }
            if !query.token.is_empty() && token == query.token {
                return None;
            }
            let score = autocomplete_score(&token, frequency, query);
            Some((
                AutocompleteEntry {
                    provider_id: provider.id.clone(),
                    provider_label: provider.label.clone(),
                    provider_icon: provider.icon.clone(),
                    item_icon: provider.item_icon.clone(),
                    label: token.clone(),
                    replacement: token,
                    detail: None,
                    documentation: None,
                },
                score,
            ))
        })
        .collect()
}

fn lsp_kind_icon(kind: Option<editor_lsp::LspCompletionKind>) -> &'static str {
    use editor_icons::symbols::cod::*;
    use editor_lsp::LspCompletionKind;
    match kind {
        Some(LspCompletionKind::Text) => COD_TEXT_SIZE,
        Some(LspCompletionKind::Method)
        | Some(LspCompletionKind::Function)
        | Some(LspCompletionKind::Constructor) => COD_SYMBOL_METHOD,
        Some(LspCompletionKind::Field) => COD_SYMBOL_FIELD,
        Some(LspCompletionKind::Variable) => COD_SYMBOL_VARIABLE,
        Some(LspCompletionKind::Class) => COD_SYMBOL_CLASS,
        Some(LspCompletionKind::Interface) => COD_SYMBOL_INTERFACE,
        Some(LspCompletionKind::Module) => COD_SYMBOL_NAMESPACE,
        Some(LspCompletionKind::Property) => COD_SYMBOL_PROPERTY,
        Some(LspCompletionKind::Unit) => COD_SYMBOL_RULER,
        Some(LspCompletionKind::Value) => COD_SYMBOL_NUMERIC,
        Some(LspCompletionKind::Enum) => COD_SYMBOL_ENUM,
        Some(LspCompletionKind::Keyword) => COD_SYMBOL_KEYWORD,
        Some(LspCompletionKind::Snippet) => COD_SYMBOL_SNIPPET,
        Some(LspCompletionKind::Color) => COD_SYMBOL_COLOR,
        Some(LspCompletionKind::File) => COD_FILE,
        Some(LspCompletionKind::Reference) => COD_REFERENCES,
        Some(LspCompletionKind::Folder) => COD_FOLDER,
        Some(LspCompletionKind::EnumMember) => COD_SYMBOL_ENUM_MEMBER,
        Some(LspCompletionKind::Constant) => COD_SYMBOL_CONSTANT,
        Some(LspCompletionKind::Struct) => COD_SYMBOL_STRUCTURE,
        Some(LspCompletionKind::Event) => COD_SYMBOL_EVENT,
        Some(LspCompletionKind::Operator) => COD_SYMBOL_OPERATOR,
        Some(LspCompletionKind::TypeParameter) => COD_SYMBOL_PARAMETER,
        None => COD_SYMBOL_MISC,
    }
}

fn lsp_autocomplete_entries(
    request: &AutocompleteWorkerRequest,
    query: &AutocompleteQuery,
    provider: &AutocompleteProviderSpec,
) -> Vec<(AutocompleteEntry, i64)> {
    let Some(path) = request.path.as_deref() else {
        return Vec::new();
    };
    let Some(lsp_client) = request.lsp_client.as_ref() else {
        return Vec::new();
    };
    let text = request.text.text();
    let completions = lsp_client
        .sync_buffer(
            path,
            &text,
            request.buffer_revision,
            request.root.as_deref(),
        )
        .ok()
        .and_then(|_| lsp_client.completions(path, request.cursor).ok())
        .unwrap_or_default();
    let prefix_lower = query.prefix.to_ascii_lowercase();
    completions
        .into_iter()
        .filter_map(|item| {
            let replacement = item.insert_text().to_owned();
            let label = item.label().to_owned();
            let candidate = if label.is_empty() {
                replacement.clone()
            } else {
                label.clone()
            };
            let candidate_lower = candidate.to_ascii_lowercase();
            if !prefix_lower.is_empty() && !candidate_lower.starts_with(&prefix_lower) {
                return None;
            }
            if !query.token.is_empty() && replacement == query.token {
                return None;
            }
            Some((
                AutocompleteEntry {
                    provider_id: provider.id.clone(),
                    provider_label: provider.label.clone(),
                    provider_icon: provider.icon.clone(),
                    item_icon: lsp_kind_icon(item.kind()).to_owned(),
                    label: candidate.clone(),
                    replacement,
                    detail: item.detail().map(str::to_owned),
                    documentation: item.documentation().map(str::to_owned),
                },
                autocomplete_score(&candidate, 2, query) + 40,
            ))
        })
        .collect()
}

fn manual_autocomplete_entries(
    plugin_kind: &Option<String>,
    query: &AutocompleteQuery,
    provider: &AutocompleteProviderSpec,
) -> Vec<(AutocompleteEntry, i64)> {
    if provider.buffer_kind.as_ref() != plugin_kind.as_ref() {
        return Vec::new();
    }
    let prefix_lower = query.prefix.to_ascii_lowercase();
    provider
        .items
        .iter()
        .filter_map(|item| {
            let label_lower = item.label.to_ascii_lowercase();
            let replacement_lower = item.replacement.to_ascii_lowercase();
            if !prefix_lower.is_empty()
                && !label_lower.starts_with(&prefix_lower)
                && !replacement_lower.starts_with(&prefix_lower)
            {
                return None;
            }
            if !query.token.is_empty() && item.replacement == query.token {
                return None;
            }
            Some((
                AutocompleteEntry {
                    provider_id: provider.id.clone(),
                    provider_label: provider.label.clone(),
                    provider_icon: provider.icon.clone(),
                    item_icon: provider.item_icon.clone(),
                    label: item.label.clone(),
                    replacement: item.replacement.clone(),
                    detail: item.detail.clone(),
                    documentation: item.documentation.clone(),
                },
                autocomplete_score(&item.replacement, 1, query) + 80,
            ))
        })
        .collect()
}

fn collect_autocomplete_token_counts(text: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    let mut token = String::new();
    for character in text.chars() {
        if is_completion_word_char(character) {
            token.push(character);
            continue;
        }
        if !token.is_empty() {
            *counts.entry(std::mem::take(&mut token)).or_insert(0) += 1;
        }
    }
    if !token.is_empty() {
        *counts.entry(token).or_insert(0) += 1;
    }
    counts
}

fn autocomplete_score(token: &str, frequency: usize, query: &AutocompleteQuery) -> i64 {
    let starts_with_exact_case =
        usize::from(!query.prefix.is_empty() && token.starts_with(&query.prefix));
    (frequency as i64 * 100)
        + (starts_with_exact_case as i64 * 24)
        + (query.prefix.chars().count() as i64 * 8)
        - token.chars().count() as i64
}

fn autocomplete_query(snapshot: &TextSnapshot, allow_empty: bool) -> Option<AutocompleteQuery> {
    let cursor = snapshot.cursor();
    let line = snapshot.line(cursor.line)?;
    let characters = line.chars().collect::<Vec<_>>();
    let cursor_col = cursor.column.min(characters.len());
    let mut start = cursor_col;
    while start > 0 && is_completion_word_char(characters[start - 1]) {
        start -= 1;
    }
    let mut end = cursor_col;
    while end < characters.len() && is_completion_word_char(characters[end]) {
        end += 1;
    }
    if !allow_empty && start == cursor_col && end == cursor_col {
        return None;
    }
    let prefix = characters[start..cursor_col].iter().collect::<String>();
    if !allow_empty && prefix.is_empty() {
        return None;
    }
    let token = characters[start..end].iter().collect::<String>();
    Some(AutocompleteQuery {
        prefix,
        token,
        replace_range: TextRange::new(
            TextPoint::new(cursor.line, start),
            TextPoint::new(cursor.line, end),
        ),
    })
}

fn is_completion_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn completion_token_at_cursor(buffer: &ShellBuffer) -> Option<(TextRange, String)> {
    let cursor = buffer.cursor_point();
    let line = buffer.text.line(cursor.line)?;
    let characters = line.chars().collect::<Vec<_>>();
    let cursor_col = cursor.column.min(characters.len());
    let token_col =
        if cursor_col < characters.len() && is_completion_word_char(characters[cursor_col]) {
            cursor_col
        } else if cursor_col > 0 && is_completion_word_char(characters[cursor_col - 1]) {
            cursor_col - 1
        } else {
            return None;
        };
    let mut start = token_col;
    while start > 0 && is_completion_word_char(characters[start - 1]) {
        start -= 1;
    }
    let mut end = token_col + 1;
    while end < characters.len() && is_completion_word_char(characters[end]) {
        end += 1;
    }
    let token = characters[start..end].iter().collect::<String>();
    Some((
        TextRange::new(
            TextPoint::new(cursor.line, start),
            TextPoint::new(cursor.line, end),
        ),
        token,
    ))
}

fn autocomplete_request_for_buffer(
    buffer_id: BufferId,
    buffer: &ShellBuffer,
    root: Option<PathBuf>,
    registry: &AutocompleteRegistry,
    lsp_client: Option<Arc<LspClientManager>>,
    allow_empty_query: bool,
) -> Option<AutocompleteBufferRequest> {
    if registry.providers.is_empty() {
        return None;
    }
    let text = buffer.text.snapshot();
    let query = autocomplete_query(&text, allow_empty_query)?;
    Some(AutocompleteBufferRequest {
        buffer_id,
        buffer_revision: buffer.text.revision(),
        text,
        plugin_kind: match &buffer.kind {
            BufferKind::Plugin(kind) => Some(kind.clone()),
            _ => None,
        },
        path: buffer.path().map(Path::to_path_buf),
        root,
        cursor: buffer.cursor_point(),
        query,
        providers: registry.providers.clone(),
        result_limit: registry.result_limit,
        lsp_client,
    })
}

fn hover_overlay_for_buffer(
    buffer_id: BufferId,
    buffer: &ShellBuffer,
    registry: &HoverRegistry,
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
    user_library: &dyn UserLibrary,
) -> Option<HoverOverlay> {
    if registry.providers.is_empty() {
        return None;
    }
    let anchor = buffer.cursor_point();
    let token_info = completion_token_at_cursor(buffer);
    let token = token_info
        .as_ref()
        .map(|(_, token)| token.clone())
        .filter(|token| !token.is_empty())
        .unwrap_or_else(|| "Cursor".to_owned());
    let providers = registry
        .providers
        .iter()
        .filter_map(|provider| {
            let lines = match provider.kind {
                HoverProviderKind::TestHover => {
                    hover_test_provider_lines(buffer, token_info.as_ref())
                }
                HoverProviderKind::Lsp => hover_lsp_provider_lines(buffer, lsp_client, lsp_context),
                HoverProviderKind::SignatureHelp => {
                    hover_signature_provider_lines(buffer, lsp_client, lsp_context)
                }
                HoverProviderKind::Diagnostics => {
                    hover_diagnostic_provider_lines(buffer, user_library)
                }
                HoverProviderKind::Manual => hover_manual_provider_lines(buffer, provider),
            };
            (!lines.is_empty()).then(|| HoverProviderContent {
                provider_label: provider.label.clone(),
                provider_icon: provider.icon.clone(),
                lines,
            })
        })
        .collect::<Vec<_>>();
    let providers = if providers.is_empty() {
        vec![HoverProviderContent {
            provider_label: "Hover".to_owned(),
            provider_icon: editor_icons::symbols::md::MD_HELP_CIRCLE_OUTLINE.to_owned(),
            lines: hover_empty_provider_lines(buffer, token_info.as_ref()),
        }]
    } else {
        providers
    };
    Some(HoverOverlay {
        buffer_id,
        anchor,
        token,
        providers,
        provider_index: 0,
        scroll_offset: 0,
        focused: false,
        line_limit: registry.line_limit,
        pending_g_prefix: false,
        count: None,
    })
}

fn hover_test_provider_lines(
    buffer: &ShellBuffer,
    token_info: Option<&(TextRange, String)>,
) -> Vec<String> {
    let mut lines = vec![
        format!("Buffer: {}", buffer.display_name()),
        format!(
            "Line: {}, Column: {}",
            buffer.cursor_row() + 1,
            buffer.cursor_col() + 1
        ),
    ];
    if let Some((range, token)) = token_info {
        lines.extend([
            format!("Token: {token}"),
            format!(
                "Range: {}:{}-{}:{}",
                range.start().line + 1,
                range.start().column + 1,
                range.end().line + 1,
                range.end().column + 1
            ),
            format!("Characters: {}", token.chars().count()),
            format!("Uppercase: {}", token.to_uppercase()),
            format!("Lowercase: {}", token.to_lowercase()),
        ]);
    } else {
        lines.extend([
            "No symbol under the cursor yet.".to_owned(),
            "Move onto an identifier to inspect token details.".to_owned(),
        ]);
    }
    if let Some(span) = hover_syntax_span_at_cursor(buffer, token_info) {
        let capture_name = if span.capture_name.starts_with('@') {
            span.capture_name.clone()
        } else {
            format!("@{}", span.capture_name)
        };
        lines.extend([
            format!("Theme color: {}", span.theme_token),
            format!("Tree-sitter token: {capture_name}"),
        ]);
    }
    lines
}

fn hover_syntax_span_at_cursor<'a>(
    buffer: &'a ShellBuffer,
    token_info: Option<&(TextRange, String)>,
) -> Option<&'a LineSyntaxSpan> {
    let point = token_info
        .map(|(range, _)| range.start())
        .unwrap_or_else(|| buffer.cursor_point());
    buffer
        .line_syntax_spans(point.line)?
        .iter()
        .filter(|span| point.column >= span.start && point.column < span.end)
        .min_by_key(|span| span.end.saturating_sub(span.start))
}

fn hover_empty_provider_lines(
    buffer: &ShellBuffer,
    token_info: Option<&(TextRange, String)>,
) -> Vec<String> {
    let mut lines = vec![
        format!("Buffer: {}", buffer.display_name()),
        format!(
            "Line: {}, Column: {}",
            buffer.cursor_row() + 1,
            buffer.cursor_col() + 1
        ),
    ];
    if let Some((_, token)) = token_info {
        lines.push(format!("No hover details are available for `{token}` yet."));
    } else {
        lines.push("No symbol is under the cursor.".to_owned());
    }
    lines.push(
        "Try moving onto an identifier or waiting for LSP/diagnostics to refresh.".to_owned(),
    );
    lines
}

fn hover_manual_provider_lines(buffer: &ShellBuffer, provider: &HoverProviderSpec) -> Vec<String> {
    let plugin_kind = match &buffer.kind {
        BufferKind::Plugin(kind) => Some(kind.as_str()),
        _ => None,
    };
    if provider.buffer_kind.as_deref() != plugin_kind {
        return Vec::new();
    }
    completion_token_at_cursor(buffer)
        .and_then(|(_, token)| {
            provider
                .topics
                .iter()
                .find(|topic| topic.token == token)
                .map(|topic| topic.lines.clone())
        })
        .unwrap_or_default()
}

fn hover_lsp_provider_lines(
    buffer: &ShellBuffer,
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
) -> Vec<String> {
    let hovers = synced_hover_lsp_request(buffer, lsp_client, lsp_context, LspClientManager::hover);
    let show_server_labels = hovers.len() > 1;
    let mut lines = Vec::new();
    for hover in hovers {
        if show_server_labels {
            lines.push(format!(
                "{} {}",
                editor_icons::symbols::cod::COD_INFO,
                hover.server_id()
            ));
        }
        lines.extend(hover.lines().iter().cloned());
    }
    lines
}

fn hover_signature_provider_lines(
    buffer: &ShellBuffer,
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
) -> Vec<String> {
    let signatures = synced_hover_lsp_request(
        buffer,
        lsp_client,
        lsp_context,
        LspClientManager::signature_help,
    );
    let show_server_labels = signatures.len() > 1;
    let mut lines = Vec::new();
    for signature in signatures {
        if show_server_labels {
            lines.push(format!(
                "{} {}",
                editor_icons::symbols::md::MD_SIGNATURE,
                signature.server_id()
            ));
        }
        lines.extend(signature.lines().iter().cloned());
    }
    lines
}

fn synced_hover_lsp_request<T>(
    buffer: &ShellBuffer,
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
    request: fn(&LspClientManager, &Path, TextPoint) -> Result<Vec<T>, LspClientError>,
) -> Vec<T> {
    let Some(lsp_client) = lsp_client else {
        return Vec::new();
    };
    let Some(context) = lsp_context else {
        return Vec::new();
    };
    lsp_client
        .sync_buffer(
            &context.path,
            &context.text,
            context.revision,
            context.root.as_deref(),
        )
        .ok()
        .and_then(|_| request(lsp_client, &context.path, buffer.cursor_point()).ok())
        .unwrap_or_default()
}

fn hover_diagnostic_provider_lines(
    buffer: &ShellBuffer,
    user_library: &dyn UserLibrary,
) -> Vec<String> {
    let cursor = buffer.cursor_point();
    let diagnostic_icon = user_library.lsp_diagnostic_icon();
    let diagnostic_line_limit = user_library.lsp_diagnostic_line_limit();
    let matching = buffer
        .lsp_diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic_matches_cursor_line(diagnostic, cursor))
        .take(diagnostic_line_limit)
        .map(|diagnostic| {
            let source = diagnostic.source();
            if source.is_empty() {
                format!("{diagnostic_icon} {}", diagnostic.message())
            } else {
                format!("{diagnostic_icon} {} ({source})", diagnostic.message())
            }
        })
        .collect::<Vec<_>>();
    if !matching.is_empty() {
        return matching;
    }
    buffer
        .lsp_diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.range().start().line == cursor.line)
        .take(diagnostic_line_limit)
        .map(|diagnostic| format!("{diagnostic_icon} {}", diagnostic.message()))
        .collect()
}

fn diagnostic_matches_cursor_line(diagnostic: &LspDiagnostic, cursor: TextPoint) -> bool {
    let range = diagnostic.range().normalized();
    if cursor.line < range.start().line || cursor.line > range.end().line {
        return false;
    }
    if range.start().line == range.end().line {
        return cursor.column >= range.start().column && cursor.column <= range.end().column;
    }
    if cursor.line == range.start().line {
        return cursor.column >= range.start().column;
    }
    if cursor.line == range.end().line {
        return cursor.column <= range.end().column;
    }
    true
}

struct PendingVimSearchRequest {
    due_at: Instant,
    request: VimSearchWorkerRequest,
}

struct VimSearchWorkerRequest {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    text: TextSnapshot,
    direction: VimSearchDirection,
    query: String,
}

struct VimSearchWorkerResult {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    direction: VimSearchDirection,
    query: String,
    data: SearchPickerData,
}

struct VimSearchWorkerState {
    pending: Option<PendingVimSearchRequest>,
    next_request_id: u64,
    request_tx: Sender<VimSearchWorkerRequest>,
    results: Arc<Mutex<Vec<VimSearchWorkerResult>>>,
}

impl VimSearchWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<VimSearchWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(mut request) = request_rx.recv() {
                while let Ok(newer_request) = request_rx.try_recv() {
                    request = newer_request;
                }
                let data = vim_search_entries(&request.text, request.direction, &request.query);
                if let Ok(mut results) = worker_results.lock() {
                    results.push(VimSearchWorkerResult {
                        request_id: request.request_id,
                        buffer_id: request.buffer_id,
                        buffer_revision: request.buffer_revision,
                        direction: request.direction,
                        query: request.query,
                        data,
                    });
                } else {
                    return;
                }
            }
        });

        Self {
            pending: None,
            next_request_id: 0,
            request_tx,
            results,
        }
    }

    fn clear_pending(&mut self) {
        self.pending = None;
    }

    fn schedule(
        &mut self,
        buffer_id: BufferId,
        buffer_revision: u64,
        text: TextSnapshot,
        direction: VimSearchDirection,
        query: String,
    ) {
        const SEARCH_REFRESH_DEBOUNCE: Duration = Duration::from_millis(100);
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.pending = Some(PendingVimSearchRequest {
            due_at: Instant::now() + SEARCH_REFRESH_DEBOUNCE,
            request: VimSearchWorkerRequest {
                request_id: self.next_request_id,
                buffer_id,
                buffer_revision,
                text,
                direction,
                query,
            },
        });
    }

    fn dispatch_due(&mut self, now: Instant) {
        let Some(pending) = self.pending.as_ref() else {
            return;
        };
        if now < pending.due_at {
            return;
        }
        let request = self.pending.take().map(|pending| pending.request);
        if let Some(request) = request {
            let _ = self.request_tx.send(request);
        }
    }

    fn take_latest_result(&self) -> Option<VimSearchWorkerResult> {
        let mut results = self.results.lock().ok()?;
        results.drain(..).next_back()
    }
}

struct PendingWorkspaceSearchRequest {
    due_at: Instant,
    request: WorkspaceSearchWorkerRequest,
}

struct WorkspaceSearchWorkerRequest {
    request_id: u64,
    root: PathBuf,
    query: String,
}

struct WorkspaceSearchWorkerResult {
    request_id: u64,
    root: PathBuf,
    query: String,
    data: SearchPickerData,
}

struct WorkspaceSearchWorkerState {
    pending: Option<PendingWorkspaceSearchRequest>,
    next_request_id: u64,
    request_tx: Sender<WorkspaceSearchWorkerRequest>,
    results: Arc<Mutex<Vec<WorkspaceSearchWorkerResult>>>,
}

impl WorkspaceSearchWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<WorkspaceSearchWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(mut request) = request_rx.recv() {
                while let Ok(newer_request) = request_rx.try_recv() {
                    request = newer_request;
                }
                let data = workspace_search_entries(&request.root, &request.query);
                if let Ok(mut results) = worker_results.lock() {
                    results.push(WorkspaceSearchWorkerResult {
                        request_id: request.request_id,
                        root: request.root,
                        query: request.query,
                        data,
                    });
                } else {
                    return;
                }
            }
        });

        Self {
            pending: None,
            next_request_id: 0,
            request_tx,
            results,
        }
    }

    fn clear_pending(&mut self) {
        self.pending = None;
    }

    fn schedule(&mut self, root: PathBuf, query: String) {
        const SEARCH_REFRESH_DEBOUNCE: Duration = Duration::from_millis(50);
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.pending = Some(PendingWorkspaceSearchRequest {
            due_at: Instant::now() + SEARCH_REFRESH_DEBOUNCE,
            request: WorkspaceSearchWorkerRequest {
                request_id: self.next_request_id,
                root,
                query,
            },
        });
    }

    fn dispatch_due(&mut self, now: Instant) {
        let Some(pending) = self.pending.as_ref() else {
            return;
        };
        if now < pending.due_at {
            return;
        }
        let request = self.pending.take().map(|pending| pending.request);
        if let Some(request) = request {
            let _ = self.request_tx.send(request);
        }
    }

    fn take_latest_result(&self) -> Option<WorkspaceSearchWorkerResult> {
        let mut results = self.results.lock().ok()?;
        results.drain(..).next_back()
    }
}

struct FileReloadWorkerRequest {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: PathBuf,
    loaded_fingerprint: Option<BackingFileFingerprint>,
}

enum FileReloadWorkerOutcome {
    Missing,
    Unchanged {
        fingerprint: BackingFileFingerprint,
    },
    Reloaded {
        fingerprint: BackingFileFingerprint,
        text: TextBuffer,
    },
}

struct FileReloadWorkerResult {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: PathBuf,
    outcome: Result<FileReloadWorkerOutcome, String>,
}

enum FileReloadWorkerCommand {
    WatchPath(PathBuf),
    UnwatchPath(PathBuf),
    Reload(FileReloadWorkerRequest),
}

struct FileReloadWorkerState {
    command_tx: Sender<FileReloadWorkerCommand>,
    changed_paths: Arc<Mutex<Vec<PathBuf>>>,
    results: Arc<Mutex<Vec<FileReloadWorkerResult>>>,
    errors: Arc<Mutex<Vec<String>>>,
    watched_paths: HashMap<PathBuf, usize>,
}

impl FileReloadWorkerState {
    fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel::<FileReloadWorkerCommand>();
        let changed_paths = Arc::new(Mutex::new(Vec::new()));
        let results = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let watcher_changed_paths = Arc::clone(&changed_paths);
        let worker_results = Arc::clone(&results);
        let worker_errors = Arc::clone(&errors);
        std::thread::spawn(move || {
            let mut watcher =
                match create_file_reload_watcher(watcher_changed_paths, Arc::clone(&worker_errors))
                {
                    Ok(watcher) => watcher,
                    Err(error) => {
                        push_file_reload_worker_error(
                            &worker_errors,
                            format!("failed to start file watcher: {error}"),
                        );
                        return;
                    }
                };
            while let Ok(command) = command_rx.recv() {
                match command {
                    FileReloadWorkerCommand::WatchPath(path) => {
                        if let Err(error) =
                            watcher.watch(path.as_path(), RecursiveMode::NonRecursive)
                        {
                            push_file_reload_worker_error(
                                &worker_errors,
                                format!("failed to watch `{}`: {error}", path.display()),
                            );
                        }
                    }
                    FileReloadWorkerCommand::UnwatchPath(path) => {
                        if let Err(error) = watcher.unwatch(path.as_path()) {
                            push_file_reload_worker_error(
                                &worker_errors,
                                format!("failed to stop watching `{}`: {error}", path.display()),
                            );
                        }
                    }
                    FileReloadWorkerCommand::Reload(request) => {
                        let result = process_file_reload_request(request);
                        if let Ok(mut results) = worker_results.lock() {
                            results.push(result);
                        } else {
                            return;
                        }
                    }
                }
            }
        });

        Self {
            command_tx,
            changed_paths,
            results,
            errors,
            watched_paths: HashMap::new(),
        }
    }

    fn watch_path(&mut self, path: PathBuf) {
        let entry = self.watched_paths.entry(path.clone()).or_default();
        *entry = entry.saturating_add(1);
        if *entry == 1 {
            let _ = self
                .command_tx
                .send(FileReloadWorkerCommand::WatchPath(path));
        }
    }

    fn unwatch_path(&mut self, path: &Path) {
        let Some(count) = self.watched_paths.get_mut(path) else {
            return;
        };
        if *count > 1 {
            *count -= 1;
            return;
        }
        self.watched_paths.remove(path);
        let _ = self
            .command_tx
            .send(FileReloadWorkerCommand::UnwatchPath(path.to_path_buf()));
    }

    fn send(&self, request: FileReloadWorkerRequest) {
        let _ = self
            .command_tx
            .send(FileReloadWorkerCommand::Reload(request));
    }

    fn take_changed_paths(&self) -> Vec<PathBuf> {
        let Ok(mut changed_paths) = self.changed_paths.lock() else {
            return Vec::new();
        };
        changed_paths.drain(..).collect()
    }

    fn take_results(&self) -> Vec<FileReloadWorkerResult> {
        let Ok(mut results) = self.results.lock() else {
            return Vec::new();
        };
        results.drain(..).collect()
    }

    fn take_errors(&self) -> Vec<String> {
        let Ok(mut errors) = self.errors.lock() else {
            return Vec::new();
        };
        errors.drain(..).collect()
    }

    #[cfg(test)]
    fn record_changed_path_for_test(&self, path: PathBuf) {
        if let Ok(mut changed_paths) = self.changed_paths.lock() {
            changed_paths.push(path);
        }
    }
}

fn process_file_reload_request(request: FileReloadWorkerRequest) -> FileReloadWorkerResult {
    let outcome = match BackingFileFingerprint::read(&request.path) {
        Ok(fingerprint) => match request.loaded_fingerprint {
            Some(loaded_fingerprint) if fingerprint == loaded_fingerprint => {
                Ok(FileReloadWorkerOutcome::Unchanged { fingerprint })
            }
            None => Ok(FileReloadWorkerOutcome::Unchanged { fingerprint }),
            Some(_) => match TextBuffer::load_from_path(&request.path) {
                Ok(text) => Ok(FileReloadWorkerOutcome::Reloaded { fingerprint, text }),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    Ok(FileReloadWorkerOutcome::Missing)
                }
                Err(error) => Err(format!(
                    "failed to reload `{}`: {error}",
                    request.path.display()
                )),
            },
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(FileReloadWorkerOutcome::Missing)
        }
        Err(error) => Err(format!(
            "failed to stat `{}`: {error}",
            request.path.display()
        )),
    };

    FileReloadWorkerResult {
        buffer_id: request.buffer_id,
        buffer_revision: request.buffer_revision,
        path: request.path,
        outcome,
    }
}

fn create_file_reload_watcher(
    changed_paths: Arc<Mutex<Vec<PathBuf>>>,
    errors: Arc<Mutex<Vec<String>>>,
) -> notify::Result<RecommendedWatcher> {
    recommended_watcher(move |event: notify::Result<NotifyEvent>| match event {
        Ok(event) => enqueue_file_reload_event(event, &changed_paths),
        Err(error) => {
            push_file_reload_worker_error(
                &errors,
                format!("failed to receive file watcher event: {error}"),
            );
        }
    })
}

fn enqueue_file_reload_event(event: NotifyEvent, changed_paths: &Arc<Mutex<Vec<PathBuf>>>) {
    if !matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return;
    }
    if let Ok(mut queued_paths) = changed_paths.lock() {
        queued_paths.extend(event.paths);
    }
}

fn push_file_reload_worker_error(errors: &Arc<Mutex<Vec<String>>>, message: String) {
    if let Ok(mut errors) = errors.lock() {
        errors.push(message);
    }
}

struct SyntaxRefreshWorkerRequest {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: Option<PathBuf>,
    buffer_language_id: Option<String>,
    syntax_window: Option<SyntaxLineWindow>,
    text: TextBuffer,
}

struct SyntaxRefreshWorkerResult {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: Option<PathBuf>,
    buffer_language_id: Option<String>,
    language_id: Option<String>,
    syntax_window: Option<SyntaxLineWindow>,
    compute_elapsed: Duration,
    highlight_span_count: usize,
    syntax_result: Option<Result<IndexedSyntaxLines, String>>,
}

#[derive(Debug, Default, Clone, Copy)]
struct SyntaxRefreshStats {
    changed: bool,
    worker_compute: Duration,
    result_count: usize,
    highlight_spans: usize,
}

struct SyntaxRefreshWorkerState {
    request_tx: Option<Sender<SyntaxRefreshWorkerRequest>>,
    results: Arc<Mutex<Vec<SyntaxRefreshWorkerResult>>>,
}

impl SyntaxRefreshWorkerState {
    fn disabled() -> Self {
        Self {
            request_tx: None,
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn configure(&mut self, configs: Vec<LanguageConfiguration>, install_root: PathBuf) {
        let (request_tx, request_rx) = mpsc::channel::<SyntaxRefreshWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            let mut registry = SyntaxRegistry::with_install_root(install_root);
            let mut parse_sessions: BTreeMap<BufferId, Option<SyntaxParseSession>> =
                BTreeMap::new();
            if registry.register_all(configs).is_err() {
                return;
            }

            while let Ok(request) = request_rx.recv() {
                let mut latest_by_buffer = BTreeMap::new();
                latest_by_buffer.insert(request.buffer_id, request);
                while let Ok(newer_request) = request_rx.try_recv() {
                    latest_by_buffer.insert(newer_request.buffer_id, newer_request);
                }

                for request in latest_by_buffer.into_values() {
                    let compute_started = Instant::now();
                    let parse_session = parse_sessions.entry(request.buffer_id).or_insert(None);
                    let (language_id, syntax_result) = compute_buffer_syntax(
                        &mut registry,
                        request.path.as_deref(),
                        &request.text,
                        request.buffer_language_id.as_deref(),
                        request.syntax_window,
                        parse_session,
                    );
                    if language_id.is_none() {
                        parse_sessions.remove(&request.buffer_id);
                    }
                    let (highlight_span_count, syntax_result) = match syntax_result {
                        Some(Ok(snapshot)) => {
                            let highlight_span_count = snapshot.highlight_count();
                            (highlight_span_count, Some(Ok(index_syntax_lines(snapshot))))
                        }
                        Some(Err(error)) => (0, Some(Err(error.to_string()))),
                        None => (0, None),
                    };
                    if let Ok(mut results) = worker_results.lock() {
                        results.push(SyntaxRefreshWorkerResult {
                            buffer_id: request.buffer_id,
                            buffer_revision: request.buffer_revision,
                            path: request.path,
                            buffer_language_id: request.buffer_language_id,
                            language_id,
                            syntax_window: request.syntax_window,
                            compute_elapsed: compute_started.elapsed(),
                            highlight_span_count,
                            syntax_result,
                        });
                    } else {
                        return;
                    }
                }
            }
        });

        self.request_tx = Some(request_tx);
        self.results = results;
    }

    fn is_configured(&self) -> bool {
        self.request_tx.is_some()
    }

    fn send(&self, request: SyntaxRefreshWorkerRequest) {
        if let Some(request_tx) = self.request_tx.as_ref() {
            let _ = request_tx.send(request);
        }
    }

    fn take_results(&self) -> Vec<SyntaxRefreshWorkerResult> {
        let Ok(mut results) = self.results.lock() else {
            return Vec::new();
        };
        results.drain(..).collect()
    }
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
        if pattern.iter().enumerate().all(|(offset, expected)| {
            normalize_search_char(chars[start + offset], case_sensitive) == *expected
        }) {
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
        let found = chars
            .get(next_index..)
            .and_then(|slice| {
                slice
                    .iter()
                    .position(|ch| normalize_search_char(*ch, case_sensitive) == *target)
            })
            .map(|offset| next_index + offset)?;
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

fn search_start_char(
    buffer: &TextSnapshot,
    direction: VimSearchDirection,
    pattern_len: usize,
) -> usize {
    let cursor = buffer.cursor();
    match direction {
        VimSearchDirection::Forward => buffer
            .point_after(cursor)
            .map(|point| buffer.point_to_char_index(point))
            .unwrap_or(buffer.char_count()),
        VimSearchDirection::Backward => buffer
            .point_before(cursor)
            .map(|point| buffer.point_to_char_index(point))
            .unwrap_or_else(|| buffer.char_count().saturating_sub(pattern_len)),
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
    buffer: &TextSnapshot,
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
    let line_count = buffer.line_count();
    let mut matches = Vec::new();

    for line_index in 0..line_count {
        let Some(line) = buffer.line(line_index) else {
            continue;
        };
        let chars: Vec<char> = line.chars().collect();
        let positions = exact_match_positions_in_chars(&chars, &pattern, case_sensitive);
        for start in positions {
            let point = TextPoint::new(line_index, start);
            let char_index = buffer.point_to_char_index(point);
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
            let Some(line) = buffer.line(line_index) else {
                continue;
            };
            let chars: Vec<char> = line.chars().collect();
            let positions = fuzzy_match_positions_in_chars(&chars, &pattern, case_sensitive);
            for (start, span) in positions {
                let point = TextPoint::new(line_index, start);
                let char_index = buffer.point_to_char_index(point);
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
                    matched.line_text.trim().to_owned(),
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
    let repeat = count.unwrap_or(1);
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

fn move_input_with_motion(
    input: &mut InputField,
    motion: ShellMotion,
    count: Option<usize>,
) -> bool {
    let repeat = count.unwrap_or(1).max(1);
    let original_anchor = input.selection_anchor;
    let original_cursor = input.cursor_char();
    let original_point = input.cursor_point();
    let mut buffer = TextBuffer::from_text(input.text());
    buffer.set_cursor(input.cursor_point());
    let moved = match motion {
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
        ShellMotion::WordEnd => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_forward() || moved)
        }
        ShellMotion::BigWordEnd => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_forward() || moved
        }),
        ShellMotion::WordEndBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_backward() || moved)
        }
        ShellMotion::BigWordEndBackward => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_backward() || moved
        }),
        ShellMotion::LineStart => {
            buffer.set_cursor(TextPoint::new(buffer.cursor().line, 0));
            buffer.cursor() != original_point
        }
        ShellMotion::LineFirstNonBlank => {
            let line = buffer.line(buffer.cursor().line).unwrap_or_default();
            let column = line
                .chars()
                .take_while(|character| character.is_whitespace())
                .count();
            buffer.set_cursor(TextPoint::new(buffer.cursor().line, column));
            buffer.cursor() != original_point
        }
        ShellMotion::LineEnd => {
            let line = buffer.cursor().line;
            let line_repeat = repeat.saturating_sub(1);
            let moved_line = if line_repeat == 0 {
                false
            } else {
                (0..line_repeat).fold(false, |moved, _| buffer.move_down() || moved)
            };
            let line_len = buffer.line_len_chars(buffer.cursor().line).unwrap_or(0);
            buffer.set_cursor(TextPoint::new(buffer.cursor().line, line_len));
            moved_line || buffer.cursor().line != line || line_len != original_point.column
        }
        ShellMotion::FirstLine => {
            let line = count.unwrap_or(1).saturating_sub(1);
            buffer.set_cursor(TextPoint::new(line, 0));
            buffer.cursor() != original_point
        }
        ShellMotion::LastLine => {
            let line = count
                .map(|value| value.saturating_sub(1))
                .unwrap_or_else(|| buffer.line_count().saturating_sub(1));
            buffer.set_cursor(TextPoint::new(line, 0));
            buffer.cursor() != original_point
        }
        ShellMotion::SentenceForward
        | ShellMotion::SentenceBackward
        | ShellMotion::ParagraphForward
        | ShellMotion::ParagraphBackward
        | ShellMotion::MatchPair
        | ShellMotion::ScreenTop
        | ShellMotion::ScreenMiddle
        | ShellMotion::ScreenBottom => false,
    };
    input.cursor = buffer.point_to_char_index(buffer.cursor());
    if original_anchor.is_none() {
        input.selection_anchor = None;
    } else {
        input.selection_anchor = original_anchor;
    }
    moved || input.cursor_char() != original_cursor
}

fn advance_point_by_text(mut point: TextPoint, text: &str) -> TextPoint {
    for character in text.chars() {
        if character == '\n' {
            point.line = point.line.saturating_add(1);
            point.column = 0;
        } else {
            point.column = point.column.saturating_add(1);
        }
    }
    point
}

fn statusline_mode_label(input_mode: InputMode, multicursor: bool) -> &'static str {
    if multicursor {
        match input_mode {
            InputMode::Normal => "MC NORMAL",
            InputMode::Insert => "MC INSERT",
            InputMode::Replace => "MC REPLACE",
            InputMode::Visual => "MC VISUAL",
        }
    } else {
        input_mode.label()
    }
}

fn move_text_buffer_with_motion(
    buffer: &mut TextBuffer,
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
        ShellMotion::WordEnd => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_forward() || moved)
        }
        ShellMotion::BigWordEnd => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_forward() || moved
        }),
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
        ShellMotion::LineStart => {
            buffer.set_cursor(TextPoint::new(buffer.cursor().line, 0));
            true
        }
        ShellMotion::LineFirstNonBlank => {
            let point = buffer
                .first_non_blank_in_line(buffer.cursor().line)
                .unwrap_or(TextPoint::new(buffer.cursor().line, 0));
            let moved = point != buffer.cursor();
            buffer.set_cursor(point);
            moved
        }
        ShellMotion::LineEnd => {
            let line_repeat = repeat.saturating_sub(1);
            let moved_line = if line_repeat == 0 {
                false
            } else {
                (0..line_repeat).fold(false, |moved, _| buffer.move_down() || moved)
            };
            let line = buffer.cursor().line;
            let column = buffer.line_len_chars(line).unwrap_or(0);
            let moved = buffer.cursor().column != column;
            buffer.set_cursor(TextPoint::new(line, column));
            moved || moved_line
        }
        ShellMotion::FirstLine => {
            let line = count.unwrap_or(1).saturating_sub(1);
            let point = buffer
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            let moved = point != buffer.cursor();
            buffer.set_cursor(point);
            moved
        }
        ShellMotion::LastLine => {
            let line = count
                .map(|value| value.saturating_sub(1))
                .unwrap_or_else(|| buffer.line_count().saturating_sub(1));
            let point = buffer
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            let moved = point != buffer.cursor();
            buffer.set_cursor(point);
            moved
        }
        ShellMotion::ScreenTop | ShellMotion::ScreenMiddle | ShellMotion::ScreenBottom => false,
    }
}

fn find_multicursor_seed_range(buffer: &ShellBuffer) -> Option<(String, TextRange)> {
    let point = buffer.cursor_point();
    let range = buffer.text.word_range_at(point, false, 1).or_else(|| {
        buffer
            .text
            .word_range_at_kind(point, WordKind::BigWord, false, 1)
    })?;
    let text = buffer.slice(range);
    (!text.is_empty()).then_some((text, range))
}

fn find_next_multicursor_match(
    buffer: &ShellBuffer,
    needle: &str,
    after_char_index: usize,
    existing: &[TextRange],
) -> Option<TextRange> {
    if needle.is_empty() {
        return None;
    }
    let haystack = buffer.text.text().chars().collect::<Vec<_>>();
    let needle_chars = needle.chars().collect::<Vec<_>>();
    if needle_chars.is_empty() || haystack.len() < needle_chars.len() {
        return None;
    }
    let existing = existing
        .iter()
        .map(|range| {
            (
                buffer.text.point_to_char_index(range.start()),
                buffer.text.point_to_char_index(range.end()),
            )
        })
        .collect::<Vec<_>>();
    let search_range = |start: usize, end: usize| {
        (start..end).find_map(|candidate| {
            let candidate_end = candidate.saturating_add(needle_chars.len());
            if candidate_end > haystack.len()
                || haystack[candidate..candidate_end] != needle_chars[..]
                || existing.iter().any(|&(existing_start, existing_end)| {
                    existing_start == candidate && existing_end == candidate_end
                })
            {
                return None;
            }
            let start_point = buffer.text.point_from_char_index(candidate);
            let end_point = buffer.text.point_from_char_index(candidate_end);
            let range = TextRange::new(start_point, end_point);
            let exact = buffer
                .text
                .word_range_at(start_point, false, 1)
                .or_else(|| {
                    buffer
                        .text
                        .word_range_at_kind(start_point, WordKind::BigWord, false, 1)
                });
            (exact == Some(range)).then_some(range)
        })
    };
    search_range(
        after_char_index.min(
            haystack
                .len()
                .saturating_sub(needle_chars.len())
                .saturating_add(1),
        ),
        haystack
            .len()
            .saturating_sub(needle_chars.len())
            .saturating_add(1),
    )
    .or_else(|| {
        search_range(
            0,
            after_char_index.min(
                haystack
                    .len()
                    .saturating_sub(needle_chars.len())
                    .saturating_add(1),
            ),
        )
    })
}

fn sync_multicursor_primary_cursor(runtime: &mut EditorRuntime) -> Result<(), String> {
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let Some(primary_range) = state.ranges.get(state.primary).copied() else {
        return Ok(());
    };
    let prefix = state
        .match_text
        .chars()
        .take(state.cursor_offset.min(state.match_text.chars().count()))
        .collect::<String>();
    let point = advance_point_by_text(primary_range.start(), &prefix);
    active_shell_buffer_mut(runtime)?.set_cursor(point);
    Ok(())
}

fn replace_multicursor_ranges(
    runtime: &mut EditorRuntime,
    text: &str,
    cursor_offset: usize,
    visual_anchor_offset: Option<usize>,
) -> Result<(), String> {
    let mut state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let new_text_len = text.chars().count();
    let original_ranges = {
        let buffer = active_shell_buffer_mut(runtime)?;
        state
            .ranges
            .iter()
            .map(|range| {
                let start = buffer.text.point_to_char_index(range.start());
                let end = buffer.text.point_to_char_index(range.end());
                (start, end)
            })
            .collect::<Vec<_>>()
    };
    let mut adjusted_starts = Vec::with_capacity(original_ranges.len());
    let mut delta = 0isize;
    for (start, end) in &original_ranges {
        adjusted_starts.push(start.saturating_add_signed(delta));
        delta += new_text_len as isize - end.saturating_sub(*start) as isize;
    }
    {
        let buffer = active_shell_buffer_mut(runtime)?;
        for range in state.ranges.iter().rev().copied() {
            buffer.replace_range(range, text);
        }
        buffer.mark_syntax_dirty();
        state.ranges = adjusted_starts
            .into_iter()
            .map(|start| {
                TextRange::new(
                    buffer.text.point_from_char_index(start),
                    buffer
                        .text
                        .point_from_char_index(start.saturating_add(new_text_len)),
                )
            })
            .collect();
    }
    let text_len = text.chars().count();
    state.match_text = text.to_owned();
    state.cursor_offset = cursor_offset.min(text_len);
    state.visual_anchor_offset = visual_anchor_offset.map(|offset| offset.min(text_len));
    shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state);
    sync_multicursor_primary_cursor(runtime)?;
    Ok(())
}

fn apply_multicursor_motion(
    runtime: &mut EditorRuntime,
    motion: ShellMotion,
) -> Result<bool, String> {
    let mut state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let mut buffer = TextBuffer::from_text(&state.match_text);
    buffer.set_cursor(buffer.point_from_char_index(state.cursor_offset));
    let moved = move_text_buffer_with_motion(
        &mut buffer,
        motion,
        shell_ui_mut(runtime)?.vim_mut().take_count(),
    );
    state.cursor_offset = buffer.point_to_char_index(buffer.cursor());
    shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state);
    sync_multicursor_primary_cursor(runtime)?;
    Ok(moved)
}

fn set_multicursor_cursor_offset(runtime: &mut EditorRuntime, offset: usize) -> Result<(), String> {
    let mut state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    state.cursor_offset = offset.min(state.match_text.chars().count());
    state.visual_anchor_offset = None;
    shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state);
    sync_multicursor_primary_cursor(runtime)
}

fn add_next_multicursor_match(runtime: &mut EditorRuntime) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? || active_shell_buffer_is_terminal(runtime)?
    {
        return Ok(());
    }
    let mut state = if let Some(existing) = shell_ui(runtime)?.vim().multicursor.clone() {
        existing
    } else {
        let buffer = active_shell_buffer_mut(runtime)?;
        let Some((match_text, range)) = find_multicursor_seed_range(buffer) else {
            return Ok(());
        };
        MulticursorState {
            match_text,
            ranges: vec![range],
            primary: 0,
            cursor_offset: buffer
                .text
                .point_to_char_index(buffer.cursor_point())
                .saturating_sub(buffer.text.point_to_char_index(range.start())),
            visual_anchor_offset: None,
        }
    };
    let after_char = active_shell_buffer_mut(runtime)?
        .text
        .point_to_char_index(state.ranges[state.primary].end());
    let next = {
        let buffer = active_shell_buffer_mut(runtime)?;
        find_next_multicursor_match(buffer, &state.match_text, after_char, &state.ranges)
    };
    if shell_ui(runtime)?.vim().multicursor.is_none() {
        shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state.clone());
        sync_multicursor_primary_cursor(runtime)?;
        return Ok(());
    }
    let Some(next) = next else {
        return Ok(());
    };
    state.ranges.push(next);
    let buffer = active_shell_buffer_mut(runtime)?;
    state
        .ranges
        .sort_by_key(|range| buffer.text.point_to_char_index(range.start()));
    state.primary = state
        .ranges
        .iter()
        .position(|range| *range == next)
        .unwrap_or(state.primary);
    shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state);
    sync_multicursor_primary_cursor(runtime)?;
    Ok(())
}

fn multicursor_selection_offsets(
    state: &MulticursorState,
    input_mode: InputMode,
) -> Option<(usize, usize)> {
    if input_mode == InputMode::Visual {
        state.visual_anchor_offset.map(|anchor| {
            let start = anchor.min(state.cursor_offset);
            let end = anchor.max(state.cursor_offset);
            (start, end)
        })
    } else {
        Some((0, state.match_text.chars().count()))
    }
}

fn apply_multicursor_insert_text(
    runtime: &mut EditorRuntime,
    text: &str,
    replace: bool,
) -> Result<(), String> {
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let mut buffer = TextBuffer::from_text(&state.match_text);
    buffer.set_cursor(buffer.point_from_char_index(state.cursor_offset));
    if replace {
        let cursor = buffer.cursor();
        let next = buffer.point_after(cursor).unwrap_or(cursor);
        if next != cursor {
            buffer.replace(TextRange::new(cursor, next), text);
        } else {
            buffer.insert_text(text);
        }
    } else {
        buffer.insert_text(text);
    }
    let new_text = buffer.text();
    let new_offset = buffer.point_to_char_index(buffer.cursor());
    replace_multicursor_ranges(runtime, &new_text, new_offset, None)
}

fn apply_multicursor_delete(runtime: &mut EditorRuntime, backward: bool) -> Result<(), String> {
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let mut buffer = TextBuffer::from_text(&state.match_text);
    buffer.set_cursor(buffer.point_from_char_index(state.cursor_offset));
    let changed = if backward {
        buffer.backspace()
    } else {
        buffer.delete_forward()
    };
    if !changed {
        return Ok(());
    }
    let new_text = buffer.text();
    let new_offset = buffer.point_to_char_index(buffer.cursor());
    replace_multicursor_ranges(runtime, &new_text, new_offset, None)
}

fn toggle_multicursor_visual_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let input_mode = shell_ui(runtime)?.input_mode();
    let mut state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    if input_mode == InputMode::Visual {
        state.visual_anchor_offset = None;
        let ui = shell_ui_mut(runtime)?;
        ui.input_mode = InputMode::Normal;
        ui.vim_mut().multicursor = Some(state);
        ui.vim_mut().clear_transient();
        return Ok(());
    }
    state.visual_anchor_offset = Some(state.cursor_offset);
    let ui = shell_ui_mut(runtime)?;
    ui.input_mode = InputMode::Visual;
    ui.vim_mut().multicursor = Some(state);
    ui.vim_mut().clear_transient();
    Ok(())
}

fn apply_multicursor_visual_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
) -> Result<(), String> {
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let Some((start, end)) = multicursor_selection_offsets(&state, InputMode::Visual) else {
        return Ok(());
    };
    if start == end {
        let ui = shell_ui_mut(runtime)?;
        ui.input_mode = InputMode::Normal;
        ui.vim_mut().multicursor = Some(state);
        return Ok(());
    }
    let selected = state
        .match_text
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect::<String>();
    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        store_yank_register(runtime, YankRegister::Character(selected.clone()), true)?;
    }
    match operator {
        VimOperator::Delete | VimOperator::Change => {
            let prefix = state.match_text.chars().take(start).collect::<String>();
            let suffix = state.match_text.chars().skip(end).collect::<String>();
            let replacement = format!("{prefix}{suffix}");
            replace_multicursor_ranges(runtime, &replacement, start, None)?;
            if operator == VimOperator::Change {
                let ui = shell_ui_mut(runtime)?;
                ui.input_mode = InputMode::Insert;
                ui.vim_mut().clear_transient();
            } else {
                let ui = shell_ui_mut(runtime)?;
                ui.input_mode = InputMode::Normal;
                ui.vim_mut().clear_transient();
            }
        }
        VimOperator::Yank => {
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Normal;
            ui.vim_mut().clear_transient();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
            let prefix = state.match_text.chars().take(start).collect::<String>();
            let middle = transform_case_text(&selected, operator);
            let suffix = state.match_text.chars().skip(end).collect::<String>();
            let replacement = format!("{prefix}{middle}{suffix}");
            replace_multicursor_ranges(runtime, &replacement, end, None)?;
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Normal;
            ui.vim_mut().clear_transient();
        }
    }
    Ok(())
}

fn apply_multicursor_text_object_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    kind: VimTextObjectKind,
) -> Result<bool, String> {
    if !matches!(kind, VimTextObjectKind::Word | VimTextObjectKind::BigWord) {
        return Ok(false);
    }
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let selected = state.match_text.clone();
    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        store_yank_register(runtime, YankRegister::Character(selected), true)?;
    }
    match operator {
        VimOperator::Delete => {
            replace_multicursor_ranges(runtime, "", 0, None)?;
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Normal;
            ui.vim_mut().clear_transient();
        }
        VimOperator::Change => {
            replace_multicursor_ranges(runtime, "", 0, None)?;
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Insert;
            ui.vim_mut().clear_transient();
            mark_change_finish_on_normal(runtime)?;
        }
        VimOperator::Yank => {
            let ui = shell_ui_mut(runtime)?;
            ui.vim_mut().clear_transient();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {}
    }
    Ok(true)
}

fn input_field_paste_shortcut_requested(keycode: Keycode, keymod: Mod) -> bool {
    keycode == Keycode::V
        && keymod.intersects(ctrl_mod())
        && keymod.intersects(shift_mod())
        && !keymod.intersects(alt_mod() | gui_mod())
}

fn paste_text_into_active_input_buffer(
    runtime: &mut EditorRuntime,
    text: &str,
) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer_is_acp(&buffer.kind)
    };
    let handled = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if let Some(input) = buffer.input_field_mut() {
            input.insert_text(text);
            true
        } else {
            false
        }
    };
    if handled && is_acp {
        acp::maybe_open_slash_completion(runtime, buffer_id)?;
        acp::refresh_acp_input_hint(runtime, buffer_id)?;
    }
    Ok(handled)
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

fn store_yank_register(
    runtime: &mut EditorRuntime,
    yank: YankRegister,
    sync_to_system_clipboard: bool,
) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    vim.yank = Some(yank.clone());
    if let Some(register) = vim.active_register.take() {
        vim.registers.insert(register, yank.clone());
    }
    if sync_to_system_clipboard {
        let text = yank_to_clipboard_text(&yank);
        write_system_clipboard(text.as_ref());
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
    let record_snapshot = {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        if vim.recording_change {
            if !vim.change_buffer.is_empty() {
                vim.last_change = vim.change_buffer.clone();
            }
            vim.change_buffer.clear();
            vim.recording_change = false;
            vim.finish_change_on_normal = false;
            vim.finish_change_after_input = false;
            true
        } else {
            false
        }
    };
    if record_snapshot {
        record_undo_tree_snapshot(runtime)?;
    }
    Ok(())
}

fn record_undo_tree_snapshot(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer = active_shell_buffer_mut(runtime)?;
    buffer.record_undo_snapshot();
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

fn jump_to_mark(runtime: &mut EditorRuntime, mark: char, linewise: bool) -> Result<(), String> {
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
        store_yank_register(runtime, yank, true)?;
    }

    match operator {
        VimOperator::Delete => {
            let buffer = active_shell_buffer_mut(runtime)?;
            buffer.delete_range(range);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            apply_directory_delete_if_needed(runtime)?;
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
        store_yank_register(runtime, YankRegister::Block(yanked), true)?;
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
            apply_directory_delete_if_needed(runtime)?;
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
            prepare_block_insert_state(
                runtime,
                selection,
                selection.start_col,
                target_cursor.line,
            )?;
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

fn apply_directory_delete_if_needed(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    if !buffer_is_directory(&shell_buffer(runtime, buffer_id)?.kind) {
        return Ok(());
    }
    apply_directory_edit_queue(runtime, buffer_id)
}

fn apply_visual_operator(runtime: &mut EditorRuntime, operator: VimOperator) -> Result<(), String> {
    if shell_ui(runtime)?.vim().multicursor.is_some()
        && !active_shell_buffer_vim_targets_input(runtime)?
    {
        return apply_multicursor_visual_operator(runtime, operator);
    }
    if active_shell_buffer_vim_targets_input(runtime)? {
        let kind = shell_ui(runtime)?.vim().visual_kind;
        let selected = {
            let buffer = active_shell_buffer_mut(runtime)?;
            let Some(input) = buffer.input_field_mut() else {
                return Ok(());
            };
            input.selected_text(kind)
        };
        let Some(selected) = selected else {
            return Ok(());
        };
        match operator {
            VimOperator::Yank => {
                let yank = match kind {
                    VisualSelectionKind::Line => YankRegister::Line(selected),
                    VisualSelectionKind::Character => YankRegister::Character(selected),
                    VisualSelectionKind::Block => return Ok(()),
                };
                store_yank_register(runtime, yank, true)?;
                if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                    input.clear_selection();
                }
                shell_ui_mut(runtime)?.enter_normal_mode();
                return Ok(());
            }
            VimOperator::Delete | VimOperator::Change => {
                if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                    input.delete_selection();
                }
                if operator == VimOperator::Change {
                    shell_ui_mut(runtime)?.enter_insert_mode();
                } else {
                    shell_ui_mut(runtime)?.enter_normal_mode();
                }
                return Ok(());
            }
            VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
                return Ok(());
            }
        }
    }
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

fn submit_input_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (prompt, text, kind) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let Some(input) = buffer.input_field() else {
            return Ok(());
        };
        (
            input.prompt().to_owned(),
            input.text().to_owned(),
            buffer.kind.clone(),
        )
    };
    if text.trim().is_empty() {
        return Ok(());
    }
    if buffer_is_acp(&kind) {
        return acp::submit_acp_prompt(runtime, buffer_id, &prompt, &text);
    }
    if buffer_is_browser(&kind) {
        return navigate_browser_buffer(runtime, buffer_id, &text);
    }
    if buffer_is_compilation(&kind) {
        return run_compile_command_in_buffer(runtime, buffer_id, &text);
    }
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!("{prompt}{text}")]);
        buffer.clear_input();
    }
    Ok(())
}

fn clear_input_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer_is_acp(&buffer.kind)
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.clear_input();
    if is_acp {
        acp::refresh_acp_input_hint(runtime, buffer_id)?;
    }
    Ok(())
}

fn focus_acp_input_section(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    if shell_buffer_mut(runtime, buffer_id)?.focus_acp_input() {
        start_change_recording(runtime)?;
        mark_change_finish_on_normal(runtime)?;
        let ui = shell_ui_mut(runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    Ok(())
}

fn save_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
) -> Result<(), String> {
    let path = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let buffer = workspace
            .buffer(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        buffer.path().map(Path::to_path_buf)
    };

    let supports_text_file_actions = shell_buffer(runtime, buffer_id)?.supports_text_file_actions();
    let is_pdf_buffer = shell_buffer(runtime, buffer_id)?.is_pdf_buffer();
    if !supports_text_file_actions && !is_pdf_buffer {
        return Ok(());
    }

    let path = path.ok_or_else(|| "buffer.save requires a file path".to_owned())?;
    if is_pdf_buffer {
        return save_buffer_inner(runtime, workspace_id, buffer_id, &path);
    }
    let language_id = language_id_for_path(runtime, &path).ok();
    if theme_lang_format_on_save(
        runtime.services().get::<ThemeRegistry>(),
        language_id.as_deref(),
    ) {
        if let Err(error) = format_buffer_on_save(runtime, workspace_id, buffer_id, &path) {
            record_runtime_error(
                runtime,
                "buffer.save.format-on-save",
                format!(
                    "format-on-save failed for `{}`: {error}; saving without formatting",
                    path.display()
                ),
            );
        }
    }
    save_buffer_inner(runtime, workspace_id, buffer_id, &path)
}

fn save_buffer_inner(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
) -> Result<(), String> {
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
            .save_to_path(path)
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
    if let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>() {
        lsp_client
            .save_buffer(path)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn close_buffer_with_prompt(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let (is_dirty, name) = {
        let ui = shell_ui(runtime)?;
        let buffer = ui
            .buffer(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing from the shell UI"))?;
        (buffer.is_dirty(), buffer.display_name().to_owned())
    };
    if is_dirty {
        let picker = picker::buffer_close_confirm_overlay(buffer_id, &name);
        shell_ui_mut(runtime)?.set_picker(picker);
        return Ok(());
    }
    close_buffer_immediate(runtime, buffer_id)
}

fn close_buffer_save(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    save_buffer(runtime, workspace_id, buffer_id)?;
    close_buffer_immediate(runtime, buffer_id)
}

fn close_buffer_discard(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    close_buffer_immediate(runtime, buffer_id)
}

fn close_buffer_immediate(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if let Some(path) = shell_ui(runtime)?
        .buffer(buffer_id)
        .and_then(|buffer| buffer.path().map(Path::to_path_buf))
        && let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>()
    {
        lsp_client
            .close_buffer(&path)
            .map_err(|error| error.to_string())?;
    }
    acp::close_acp_buffer(runtime, buffer_id)?;
    close_terminal_buffer(runtime, buffer_id)?;
    runtime
        .model_mut()
        .close_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.remove_buffer(buffer_id);
    sync_active_buffer(runtime)?;
    Ok(())
}

fn close_lsp_buffers_for_workspace(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>() else {
        return Ok(());
    };
    let paths = {
        let ui = shell_ui(runtime)?;
        ui.workspace_views
            .get(&workspace_id)
            .map(|view| {
                view.buffer_ids
                    .iter()
                    .filter_map(|buffer_id| {
                        ui.buffer(*buffer_id)
                            .and_then(|buffer| buffer.path().map(Path::to_path_buf))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    for path in paths {
        lsp_client
            .close_buffer(&path)
            .map_err(|error| error.to_string())?;
    }
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
        let path = {
            let workspace = runtime
                .model()
                .workspace(workspace_id)
                .map_err(|error| error.to_string())?;
            let buffer = workspace
                .buffer(buffer_id)
                .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
            buffer.path().map(Path::to_path_buf)
        };

        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer.supports_text_file_actions() && !buffer.is_pdf_buffer() {
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

        let path =
            path.ok_or_else(|| format!("text-editable buffer `{buffer_id}` is missing a path"))?;
        save_buffer(runtime, workspace_id, buffer_id)
            .map_err(|error| format!("failed to save `{}`: {error}", path.display()))?;
    }

    Ok(())
}

fn format_workspace(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (path, extension, original_cursor, selection, supports_text_actions) = {
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
        (
            path.to_path_buf(),
            extension,
            original_cursor,
            selection,
            buffer.supports_text_file_actions(),
        )
    };

    if !supports_text_actions {
        return Err("workspace.format only supports file buffers".to_owned());
    }

    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .or_else(|| active_workspace_root(runtime).ok().flatten());

    start_change_recording(runtime)?;

    if let Some((selection, anchor, head, kind)) = selection {
        store_last_visual_selection(runtime, anchor, head, kind)?;
        if try_format_visual_selection_with_lsp(
            runtime,
            workspace_id,
            buffer_id,
            &path,
            selection,
            original_cursor,
        )? {
            finish_format_command(runtime)?;
            return Ok(());
        }
        let formatter = formatter_for_path(runtime, &path)?;
        format_visual_selection_with_formatter(
            runtime,
            &formatter,
            selection,
            extension.as_deref(),
            cwd.as_deref(),
            original_cursor,
        )?;
    } else {
        if try_format_buffer_entire_with_lsp(
            runtime,
            workspace_id,
            buffer_id,
            &path,
            original_cursor,
        )? {
            finish_format_command(runtime)?;
            return Ok(());
        }
        let formatter = formatter_for_path(runtime, &path)?;
        format_entire_buffer_with_formatter(
            runtime,
            &formatter,
            extension.as_deref(),
            cwd.as_deref(),
            original_cursor,
        )?;
    }

    Ok(())
}

fn format_buffer_on_save(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
) -> Result<(), String> {
    let original_cursor = shell_buffer(runtime, buffer_id)?.cursor_point();
    if try_format_buffer_entire_with_lsp(runtime, workspace_id, buffer_id, path, original_cursor)? {
        return Ok(());
    }

    let formatter = formatter_for_path(runtime, path)?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_owned);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .or_else(|| active_workspace_root(runtime).ok().flatten());
    format_buffer_entire_with_formatter(
        runtime,
        buffer_id,
        &formatter,
        extension.as_deref(),
        cwd.as_deref(),
        original_cursor,
    )
}

fn formatter_for_path(runtime: &EditorRuntime, path: &Path) -> Result<FormatterSpec, String> {
    let language_id = language_id_for_path(runtime, path)?;
    let formatter = formatter_registry(runtime)?
        .formatter_for_language(&language_id)
        .ok_or_else(|| format!("no formatter registered for language `{language_id}`"))?;
    Ok(formatter.clone())
}

fn language_id_for_path(runtime: &EditorRuntime, path: &Path) -> Result<String, String> {
    let syntax = runtime
        .services()
        .get::<SyntaxRegistry>()
        .ok_or_else(|| "syntax registry service missing".to_owned())?;
    let language = syntax
        .language_for_path(path)
        .ok_or_else(|| format!("no syntax language registered for `{}`", path.display()))?;
    Ok(language.id().to_owned())
}

fn finish_format_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.enter_normal_mode();
    schedule_finish_change(runtime)
}

fn format_entire_buffer_with_formatter(
    runtime: &mut EditorRuntime,
    formatter: &FormatterSpec,
    extension: Option<&str>,
    cwd: Option<&Path>,
    original_cursor: TextPoint,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    format_buffer_entire_with_formatter(
        runtime,
        buffer_id,
        formatter,
        extension,
        cwd,
        original_cursor,
    )?;
    shell_ui_mut(runtime)?.enter_normal_mode();
    schedule_finish_change(runtime)?;
    Ok(())
}

fn format_buffer_entire_with_formatter(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    formatter: &FormatterSpec,
    extension: Option<&str>,
    cwd: Option<&Path>,
    original_cursor: TextPoint,
) -> Result<(), String> {
    let input = { shell_buffer(runtime, buffer_id)?.text.text() };
    let formatted = format_text_with_formatter(runtime, formatter, &input, extension, cwd)?;
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    if formatted != input {
        let end = buffer.text.point_from_char_index(buffer.text.char_count());
        buffer.replace_range(TextRange::new(TextPoint::default(), end), &formatted);
        buffer.mark_syntax_dirty();
    }
    buffer.set_cursor(original_cursor);
    Ok(())
}

fn format_visual_selection_with_formatter(
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

fn try_format_buffer_entire_with_lsp(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    original_cursor: TextPoint,
) -> Result<bool, String> {
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        return Ok(false);
    };
    let context = lsp_buffer_context(runtime, workspace_id, buffer_id)?;
    let language_id = language_id_for_path(runtime, path).ok();
    let options = lsp_formatting_options(runtime, language_id.as_deref());
    let (labels, edits) = {
        if !lsp_client.supports_path(&context.path) {
            return Ok(false);
        }
        let labels = lsp_client
            .sync_buffer(
                &context.path,
                &context.text,
                context.revision,
                context.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        let edits = lsp_client
            .formatting(&context.path, options)
            .map_err(|error| error.to_string())?;
        (labels, edits)
    };
    sync_lsp_buffer_state(runtime, workspace_id, buffer_id, &labels)?;
    let Some(edits) = edits else {
        return Ok(false);
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    apply_lsp_text_edits(buffer, &edits);
    buffer.set_cursor(original_cursor);
    Ok(true)
}

fn try_format_visual_selection_with_lsp(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    selection: VisualSelection,
    original_cursor: TextPoint,
) -> Result<bool, String> {
    let VisualSelection::Range(range) = selection else {
        return Ok(false);
    };
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        return Ok(false);
    };
    let context = lsp_buffer_context(runtime, workspace_id, buffer_id)?;
    let language_id = language_id_for_path(runtime, path).ok();
    let options = lsp_formatting_options(runtime, language_id.as_deref());
    let (labels, edits) = {
        if !lsp_client.supports_path(&context.path) {
            return Ok(false);
        }
        let labels = lsp_client
            .sync_buffer(
                &context.path,
                &context.text,
                context.revision,
                context.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        let edits = lsp_client
            .range_formatting(&context.path, range, options)
            .map_err(|error| error.to_string())?;
        (labels, edits)
    };
    sync_lsp_buffer_state(runtime, workspace_id, buffer_id, &labels)?;
    let Some(edits) = edits else {
        return Ok(false);
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    apply_lsp_text_edits(buffer, &edits);
    buffer.set_cursor(original_cursor);
    Ok(true)
}

fn sync_lsp_buffer_state(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    labels: &[String],
) -> Result<(), String> {
    let active_buffer_id = active_shell_buffer_id(runtime).ok();
    let ui = shell_ui_mut(runtime)?;
    if let Some(buffer) = ui.buffer_mut(buffer_id) {
        buffer.set_lsp_enabled(!labels.is_empty());
    }
    if active_buffer_id == Some(buffer_id) {
        ui.set_attached_lsp_server(
            workspace_id,
            (!labels.is_empty()).then(|| labels.join(", ")),
        );
    }
    Ok(())
}

fn apply_lsp_text_edits(buffer: &mut ShellBuffer, edits: &[LspTextEdit]) {
    let mut ordered = edits.to_vec();
    ordered.sort_by(|left, right| {
        right
            .range()
            .start()
            .cmp(&left.range().start())
            .then_with(|| right.range().end().cmp(&left.range().end()))
    });
    for edit in ordered {
        buffer.replace_range(edit.range(), edit.new_text());
    }
    if !edits.is_empty() {
        buffer.mark_syntax_dirty();
    }
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
    if shell_ui(runtime)?.vim().multicursor.is_some()
        && apply_multicursor_text_object_operator(runtime, operator, kind)?
    {
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
        // Multicursor text objects intentionally operate on the linked token set as a whole, so
        // the per-command around/count modifiers do not change that mirrored scope yet.
        let _ = around;
        let _ = count;
        return Ok(());
    }
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

    if shell_ui(runtime)?.vim().multicursor.is_some()
        && !active_shell_buffer_vim_targets_input(runtime)?
    {
        let _ = apply_multicursor_motion(runtime, motion)?;
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
        return Ok(());
    }

    let count = shell_ui_mut(runtime)?.vim_mut().take_count();
    if let Some(scroll) = terminal_scroll_for_motion(motion, count)
        && scroll_active_terminal_view(runtime, scroll)?
    {
        return Ok(());
    }
    let input_mode = shell_ui(runtime)?.input_mode();
    let target_input = active_shell_buffer_vim_targets_input(runtime)?;
    let handled_input = {
        let buffer = active_shell_buffer_mut(runtime)?;
        if target_input {
            if let Some(input) = buffer.input_field_mut() {
                if matches!(input_mode, InputMode::Visual) && input.selection_anchor.is_none() {
                    input.start_selection();
                }
                Some(move_input_with_motion(input, motion, count))
            } else {
                None
            }
        } else {
            None
        }
    };
    if handled_input.is_none() {
        move_buffer_with_motion(active_shell_buffer_mut(runtime)?, motion, count);
    }
    Ok(())
}

fn apply_scroll_command(runtime: &mut EditorRuntime, command: ScrollCommand) -> Result<(), String> {
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    let terminal_scroll = match command {
        ScrollCommand::HalfPageDown => Some(TerminalViewportScroll::HalfPageDown),
        ScrollCommand::HalfPageUp => Some(TerminalViewportScroll::HalfPageUp),
        ScrollCommand::PageDown => Some(TerminalViewportScroll::PageDown),
        ScrollCommand::PageUp => Some(TerminalViewportScroll::PageUp),
        ScrollCommand::LineDown => Some(TerminalViewportScroll::LineDelta(-(count as i32))),
        ScrollCommand::LineUp => Some(TerminalViewportScroll::LineDelta(count as i32)),
    };
    if let Some(scroll) = terminal_scroll
        && scroll_active_terminal_view(runtime, scroll)?
    {
        return Ok(());
    }
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
    let screen_offset = buffer.cursor_viewport_offset();
    buffer.scroll_by(delta);
    let target_line = buffer.line_at_viewport_offset(screen_offset);
    let _ = buffer.goto_line(target_line);
}

fn scroll_buffer_viewport_only(buffer: &mut ShellBuffer, delta: i32) {
    buffer.scroll_by(delta);
    let top = buffer.current_scroll_row();
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

fn open_vim_command_line(runtime: &mut EditorRuntime) -> Result<(), String> {
    if !shell_user_library(runtime).commandline_enabled() {
        let picker = picker::picker_overlay(runtime, "commands")?;
        shell_ui_mut(runtime)?.set_picker(picker);
        return Ok(());
    }
    clear_key_sequence(runtime)?;
    shell_ui_mut(runtime)?.set_command_line(CommandLineOverlay::new());
    Ok(())
}

fn cycle_vim_command_line_completion(
    runtime: &mut EditorRuntime,
    reverse: bool,
) -> Result<(), String> {
    let seed = shell_ui(runtime)?
        .command_line()
        .map(|command_line| command_line.text().to_owned())
        .unwrap_or_default();
    let matches = vim_command_line_completion_matches(runtime, &seed);
    if matches.is_empty() {
        return Ok(());
    }
    if let Some(command_line) = shell_ui_mut(runtime)?.command_line_mut() {
        command_line.cycle_completion(matches, reverse);
    }
    Ok(())
}

fn vim_command_line_completion_matches(runtime: &EditorRuntime, seed: &str) -> Vec<String> {
    let trimmed = seed.trim();
    if trimmed.starts_with('!') {
        return Vec::new();
    }
    if trimmed.starts_with('%') {
        let candidate = "%s///g";
        return candidate
            .starts_with(trimmed)
            .then_some(candidate.to_owned())
            .into_iter()
            .collect();
    }
    runtime
        .commands()
        .command_names()
        .into_iter()
        .filter(|name| name.starts_with(trimmed))
        .map(str::to_owned)
        .collect()
}

fn submit_vim_command_line(runtime: &mut EditorRuntime) -> Result<(), String> {
    let text = shell_ui(runtime)?
        .command_line()
        .map(|command_line| command_line.text().trim().to_owned())
        .unwrap_or_default();
    shell_ui_mut(runtime)?.close_command_line();
    execute_vim_command_line(runtime, &text)
}

fn execute_vim_command_line(runtime: &mut EditorRuntime, command: &str) -> Result<(), String> {
    let command = command.trim();
    if command.is_empty() {
        return Ok(());
    }
    if let Some(shell_command) = command.strip_prefix('!') {
        return run_shell_command_from_vim_command_line(runtime, shell_command.trim());
    }
    if command.starts_with("%s") {
        return apply_vim_substitute_command(runtime, command);
    }
    if runtime.commands().contains(command) {
        runtime
            .execute_command(command)
            .map_err(|error| error.to_string())?;
        sync_active_buffer(runtime)?;
        return Ok(());
    }
    let matches = runtime
        .commands()
        .command_names()
        .into_iter()
        .filter(|name| name.starts_with(command))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [matched] => {
            runtime
                .execute_command(matched)
                .map_err(|error| error.to_string())?;
            sync_active_buffer(runtime)?;
            Ok(())
        }
        [] => Err(format!("unknown command `{command}`")),
        _ => Err(format!("ambiguous command `{command}`")),
    }
}

fn apply_vim_substitute_command(runtime: &mut EditorRuntime, command: &str) -> Result<(), String> {
    let (pattern, replacement, flags) = parse_vim_substitute_command(command)?;
    if pattern.is_empty() {
        return Err(":%s requires a search pattern".to_owned());
    }
    let replace_all = flags.contains('g');
    if flags.chars().any(|flag| flag != 'g') {
        return Err(format!("unsupported :%s flags `{flags}`"));
    }
    let (original_cursor, end, replaced, replacements) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        if buffer.is_read_only() {
            return Err(":%s is blocked for read-only buffers".to_owned());
        }
        let original_cursor = buffer.cursor_point();
        let end = if buffer.line_count() == 0 {
            TextPoint::default()
        } else {
            let last_line = buffer.line_count().saturating_sub(1);
            TextPoint::new(last_line, buffer.line_len_chars(last_line))
        };
        let (replaced, replacements) =
            substitute_buffer_text(&buffer.text.text(), &pattern, &replacement, replace_all);
        (original_cursor, end, replaced, replacements)
    };
    if replacements == 0 {
        return Err(format!("no matches found for `{pattern}`"));
    }
    let buffer = active_shell_buffer_mut(runtime)?;
    buffer.replace_range(TextRange::new(TextPoint::default(), end), &replaced);
    buffer.set_cursor(original_cursor);
    buffer.mark_syntax_dirty();
    Ok(())
}

fn parse_vim_substitute_command(command: &str) -> Result<(String, String, String), String> {
    let rest = command
        .strip_prefix("%s")
        .ok_or_else(|| ":%s command must start with `%s`".to_owned())?;
    let Some(delimiter) = rest.chars().next() else {
        return Err(":%s requires a delimiter".to_owned());
    };
    let mut remaining = &rest[delimiter.len_utf8()..];
    let (pattern, next) = split_vim_substitute_segment(remaining, delimiter)?;
    remaining = next;
    let (replacement, next) = split_vim_substitute_segment(remaining, delimiter)?;
    remaining = next;
    Ok((pattern, replacement, remaining.trim().to_owned()))
}

fn split_vim_substitute_segment(input: &str, delimiter: char) -> Result<(String, &str), String> {
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == delimiter {
            let segment = unescape_vim_substitute_segment(&input[..index]);
            let remaining = &input[index + delimiter.len_utf8()..];
            return Ok((segment, remaining));
        }
    }
    Err(format!("missing closing `{delimiter}` in :%s command"))
}

fn unescape_vim_substitute_segment(segment: &str) -> String {
    let mut text = String::new();
    let mut escaped = false;
    for character in segment.chars() {
        if escaped {
            text.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        text.push(character);
    }
    if escaped {
        text.push('\\');
    }
    text
}

fn substitute_buffer_text(
    text: &str,
    pattern: &str,
    replacement: &str,
    replace_all: bool,
) -> (String, usize) {
    let mut replacements = 0usize;
    let lines = text
        .split('\n')
        .map(|line| {
            let (updated, count) = substitute_line_text(line, pattern, replacement, replace_all);
            replacements = replacements.saturating_add(count);
            updated
        })
        .collect::<Vec<_>>();
    (lines.join("\n"), replacements)
}

fn substitute_line_text(
    line: &str,
    pattern: &str,
    replacement: &str,
    replace_all: bool,
) -> (String, usize) {
    if pattern.is_empty() {
        return (line.to_owned(), 0);
    }
    if !replace_all {
        if let Some(index) = line.find(pattern) {
            let mut updated = String::new();
            updated.push_str(&line[..index]);
            updated.push_str(replacement);
            updated.push_str(&line[index + pattern.len()..]);
            return (updated, 1);
        }
        return (line.to_owned(), 0);
    }
    let mut remaining = line;
    let mut updated = String::new();
    let mut replacements = 0usize;
    while let Some(index) = remaining.find(pattern) {
        updated.push_str(&remaining[..index]);
        updated.push_str(replacement);
        remaining = &remaining[index + pattern.len()..];
        replacements = replacements.saturating_add(1);
    }
    if replacements == 0 {
        return (line.to_owned(), 0);
    }
    updated.push_str(remaining);
    (updated, replacements)
}

fn open_vim_search_prompt(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
) -> Result<(), String> {
    let title = match direction {
        VimSearchDirection::Forward => "Search /",
        VimSearchDirection::Backward => "Search ?",
    };
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::search(title, direction, Vec::new()));
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
        VimOperator::Delete
            | VimOperator::Change
            | VimOperator::ToggleCase
            | VimOperator::Lowercase
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
    if active_shell_buffer_vim_targets_input(runtime)? {
        let cursor = {
            let buffer = active_shell_buffer_mut(runtime)?;
            let Some(input) = buffer.input_field_mut() else {
                return Ok(());
            };
            input.start_selection();
            input.cursor_point()
        };
        shell_ui_mut(runtime)?.enter_visual_mode(cursor, kind);
        return Ok(());
    }
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
    if shell_ui(runtime)?.vim().multicursor.is_some()
        && !active_shell_buffer_vim_targets_input(runtime)?
    {
        return toggle_multicursor_visual_mode(runtime);
    }
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

fn resolve_put_yank(runtime: &mut EditorRuntime) -> Result<Option<YankRegister>, String> {
    let (active_register, fallback_yank) = {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        (vim.active_register.take(), vim.yank.clone())
    };
    let yank = if let Some(register) = active_register {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        vim.registers.get(&register).cloned().or(fallback_yank)
    } else {
        let clipboard_text = read_system_clipboard();
        let clipboard_yank = clipboard_text.as_deref().and_then(yank_from_clipboard_text);
        // Prefer internal block yanks when the clipboard matches them, since block shapes
        // cannot be reconstructed from clipboard text alone.
        let prefer_internal_block = match (fallback_yank.as_ref(), clipboard_text.as_deref()) {
            (Some(block @ YankRegister::Block(_)), Some(text)) => {
                let block_text = yank_to_clipboard_text(block);
                text == block_text.as_ref()
            }
            (Some(YankRegister::Block(_)), None) => true,
            _ => false,
        };
        if prefer_internal_block {
            fallback_yank
        } else if let Some(clipboard) = clipboard_yank {
            shell_ui_mut(runtime)?.vim_mut().yank = Some(clipboard.clone());
            Some(clipboard)
        } else {
            fallback_yank
        }
    };
    Ok(yank)
}

fn put_yank(runtime: &mut EditorRuntime, after: bool) -> Result<(), String> {
    let Some(yank) = resolve_put_yank(runtime)? else {
        return Ok(());
    };
    if active_shell_buffer_is_terminal(runtime)? {
        let text = yank_to_clipboard_text(&yank);
        write_active_terminal_text(runtime, text.as_ref())?;
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
        return Ok(());
    }

    start_change_recording(runtime)?;
    let (indent_size, use_tabs) = {
        let ui = shell_ui(runtime)?;
        let buffer_id = active_shell_buffer_id(runtime)?;
        let language_id = ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
        let theme_registry = runtime.services().get::<ThemeRegistry>();
        (
            theme_lang_indent(theme_registry, language_id),
            theme_lang_use_tabs(theme_registry, language_id),
        )
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
        if buffer.supports_text_file_actions() {
            format_current_line_indent(buffer, indent_size, use_tabs);
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
    let Some((pane_id, buffer_id, buffer_name, buffer_kind)) = active_runtime_buffer(runtime)?
    else {
        return Ok(());
    };
    let is_git_commit = buffer_is_git_commit(&buffer_kind);
    let is_git_status = buffer_is_git_status(&buffer_kind);
    let is_directory = buffer_is_directory(&buffer_kind);
    let is_terminal = buffer_is_terminal(&buffer_kind);

    let (previous_pane, previous_buffer) = {
        let ui = shell_ui(runtime)?;
        (ui.active_pane_id(), ui.active_buffer_id())
    };
    let should_enter_insert = {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        if previous_pane != Some(pane_id) {
            ui.focus_pane(pane_id);
        } else if previous_buffer != Some(buffer_id) {
            ui.close_autocomplete();
            ui.close_hover();
        }
        let has_input = ui
            .ensure_buffer(buffer_id, &buffer_name, buffer_kind, &*user_library)
            .has_input_field();
        ui.focus_buffer_in_active_pane(buffer_id);
        if !is_git_commit {
            ui.pending_ctrl_c = None;
        }
        if !is_git_status {
            ui.pending_git_prefix = None;
        }
        if !is_directory {
            ui.pending_directory_prefix = None;
        }
        previous_buffer != Some(buffer_id) && has_input
    };
    let terminal_created = if is_terminal {
        ensure_terminal_session(runtime, buffer_id)?
    } else {
        false
    };
    if should_enter_insert || terminal_created {
        shell_ui_mut(runtime)?.enter_insert_mode();
    }
    if previous_buffer != Some(buffer_id) {
        let workspace_id = runtime
            .model()
            .active_workspace_id()
            .map_err(|error| error.to_string())?;
        let window_id = active_window_id(runtime)?;
        runtime
            .emit_hook(
                builtins::BUFFER_SWITCH,
                HookEvent::new()
                    .with_window(window_id)
                    .with_workspace(workspace_id)
                    .with_pane(pane_id)
                    .with_buffer(buffer_id),
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn active_runtime_surface(runtime: &EditorRuntime) -> Result<Option<(PaneId, BufferId)>, String> {
    Ok(active_runtime_buffer(runtime)?.map(|(pane_id, buffer_id, _, _)| (pane_id, buffer_id)))
}

fn ensure_shell_buffer(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let (buffer_name, buffer_kind) = {
        let workspace_id = runtime
            .model()
            .active_workspace_id()
            .map_err(|error| error.to_string())?;
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let buffer = workspace
            .buffer(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        (buffer.name().to_owned(), buffer.kind().clone())
    };
    let user_library = shell_user_library(runtime);
    shell_ui_mut(runtime)?.ensure_popup_buffer(
        buffer_id,
        &buffer_name,
        buffer_kind,
        &*user_library,
    );
    Ok(())
}

fn find_shell_buffer_by_kind(ui: &ShellUiState, kind: &str) -> Option<BufferId> {
    ui.buffers.iter().find_map(|buffer| {
        if matches!(&buffer.kind, BufferKind::Plugin(plugin_kind) if plugin_kind == kind) {
            Some(buffer.id())
        } else {
            None
        }
    })
}

fn find_oil_buffer(ui: &ShellUiState) -> Option<BufferId> {
    ui.buffers.iter().find_map(|buffer| {
        if matches!(&buffer.kind, BufferKind::Directory) && buffer.display_name() == OIL_BUFFER_NAME
        {
            Some(buffer.id())
        } else {
            None
        }
    })
}

fn active_shell_buffer_path(runtime: &EditorRuntime) -> Result<Option<PathBuf>, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    Ok(shell_buffer(runtime, buffer_id)?
        .path()
        .map(Path::to_path_buf))
}

fn active_directory_root(runtime: &EditorRuntime) -> Result<Option<PathBuf>, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    Ok(buffer.directory_state().map(|state| state.root.clone()))
}

fn oil_workspace_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_workspace_root(runtime)? {
        return Ok(root);
    }
    env::current_dir().map_err(|error| format!("oil requires a workspace root: {error}"))
}

fn oil_default_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_directory_root(runtime)? {
        return Ok(root);
    }
    if let Some(path) = active_shell_buffer_path(runtime)?
        && let Some(parent) = path.parent()
    {
        return Ok(parent.to_path_buf());
    }
    oil_workspace_root(runtime)
}

fn oil_parent_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_directory_root(runtime)? {
        return Ok(root.parent().unwrap_or(root.as_path()).to_path_buf());
    }
    let root = oil_default_root(runtime)?;
    Ok(root.parent().unwrap_or(root.as_path()).to_path_buf())
}

fn open_oil_directory(runtime: &mut EditorRuntime, root: PathBuf) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let existing = shell_ui(runtime).ok().and_then(find_oil_buffer);
    let buffer_id = if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        existing
    } else {
        runtime
            .model_mut()
            .create_buffer(workspace_id, OIL_BUFFER_NAME, BufferKind::Directory, None)
            .map_err(|error| error.to_string())?
    };
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(
            buffer_id,
            OIL_BUFFER_NAME,
            BufferKind::Directory,
            &*user_library,
        );
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    set_directory_root(runtime, buffer_id, root)?;
    Ok(())
}

fn ensure_directory_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_directory(&buffer.kind) || buffer.directory_state().is_some() {
        return Ok(());
    }
    let root = oil_default_root(runtime)?;
    set_directory_root(runtime, buffer_id, root)
}

fn refresh_directory_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let (root, show_hidden, sort_mode, trash_enabled) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let Some(state) = buffer.directory_state() else {
            let root = oil_default_root(runtime)?;
            return set_directory_root(runtime, buffer_id, root);
        };
        (
            state.root.clone(),
            state.show_hidden,
            state.sort_mode,
            state.trash_enabled,
        )
    };
    let entries = match DirectoryBuffer::read(&root) {
        Ok(buffer) => buffer.entries().to_vec(),
        Err(error) => {
            let message = format!("failed to read `{}`: {error}", root.display());
            set_directory_error(runtime, buffer_id, &message)?;
            return Err(message);
        }
    };
    let defaults = shell_user_library(runtime).oil_defaults();
    let mut state = DirectoryViewState::new(root, entries, defaults);
    state.show_hidden = show_hidden;
    state.sort_mode = sort_mode;
    state.trash_enabled = trash_enabled;
    apply_directory_state(runtime, buffer_id, state)
}

fn set_directory_root(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    root: PathBuf,
) -> Result<(), String> {
    let defaults = shell_user_library(runtime).oil_defaults();
    let (show_hidden, sort_mode, trash_enabled, previous_root) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let state = buffer.directory_state();
        (
            state
                .map(|state| state.show_hidden)
                .unwrap_or(defaults.show_hidden),
            state
                .map(|state| state.sort_mode)
                .unwrap_or(defaults.sort_mode),
            state
                .map(|state| state.trash_enabled)
                .unwrap_or(defaults.trash_enabled),
            state.map(|state| state.root.clone()),
        )
    };
    let root_for_compare = root.clone();
    let entries = match DirectoryBuffer::read(&root) {
        Ok(buffer) => buffer.entries().to_vec(),
        Err(error) => {
            let message = format!("failed to read `{}`: {error}", root.display());
            set_directory_error(runtime, buffer_id, &message)?;
            return Err(message);
        }
    };
    let mut state = DirectoryViewState::new(root, entries, defaults);
    state.show_hidden = show_hidden;
    state.sort_mode = sort_mode;
    state.trash_enabled = trash_enabled;
    apply_directory_state(runtime, buffer_id, state)?;
    if previous_root.as_ref() != Some(&root_for_compare) {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let target_line = if buffer.line_count() > 1 { 1 } else { 0 };
        buffer.goto_line(target_line);
        buffer.scroll_row = 0;
    }
    Ok(())
}

fn set_directory_error(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    message: &str,
) -> Result<(), String> {
    record_runtime_error(runtime, "oil.directory", message.to_owned());
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.clear_directory_state();
    buffer.section_state = None;
    buffer.replace_with_lines(vec![
        "Directory view unavailable.".to_owned(),
        message.to_owned(),
    ]);
    Ok(())
}

fn open_file_in_split(
    runtime: &mut EditorRuntime,
    path: &Path,
    direction: PaneSplitDirection,
    focus: bool,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let original_pane_id = shell_ui(runtime)?
        .active_pane_id()
        .ok_or_else(|| "active pane is missing".to_owned())?;
    if shell_ui(runtime)?.pane_count() < 2 {
        split_runtime_pane(runtime, direction)?;
    }
    let target_pane_id = shell_ui(runtime)?
        .panes()
        .and_then(|panes| {
            panes
                .iter()
                .find(|pane| pane.pane_id != original_pane_id)
                .map(|pane| pane.pane_id)
        })
        .ok_or_else(|| "split pane is missing".to_owned())?;
    runtime
        .model_mut()
        .focus_pane(workspace_id, target_pane_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_pane(target_pane_id);
    open_workspace_file(runtime, path)?;
    if !focus {
        runtime
            .model_mut()
            .focus_pane(workspace_id, original_pane_id)
            .map_err(|error| error.to_string())?;
        shell_ui_mut(runtime)?.focus_pane(original_pane_id);
    }
    Ok(())
}

fn open_oil_preview_popup(runtime: &mut EditorRuntime, path: &Path) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let existing = shell_ui(runtime)
        .ok()
        .and_then(|ui| find_shell_buffer_by_kind(ui, OIL_PREVIEW_KIND));
    let buffer_id = if let Some(existing) = existing {
        existing
    } else {
        runtime
            .model_mut()
            .create_popup_buffer(
                workspace_id,
                OIL_PREVIEW_BUFFER_NAME,
                BufferKind::Plugin(OIL_PREVIEW_KIND.to_owned()),
                None,
            )
            .map_err(|error| error.to_string())?
    };
    runtime
        .model_mut()
        .open_popup_buffer(workspace_id, "Preview", buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.set_popup_focus(false);
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let text = TextBuffer::load_from_path(path)
        .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    let user_library = shell_user_library(runtime);
    let shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
    shell_ui_mut(runtime)?.insert_buffer(shell_buffer);
    queue_buffer_syntax_refresh(runtime, buffer_id)?;
    Ok(())
}

fn open_oil_help_popup(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let existing = shell_ui(runtime)
        .ok()
        .and_then(|ui| find_shell_buffer_by_kind(ui, OIL_HELP_KIND));
    let buffer_id = if let Some(existing) = existing {
        existing
    } else {
        runtime
            .model_mut()
            .create_popup_buffer(
                workspace_id,
                OIL_HELP_BUFFER_NAME,
                BufferKind::Plugin(OIL_HELP_KIND.to_owned()),
                None,
            )
            .map_err(|error| error.to_string())?
    };
    runtime
        .model_mut()
        .open_popup_buffer(workspace_id, "Oil Help", buffer_id)
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui_mut(runtime)?;
        ui.set_popup_buffer(buffer_id);
        ui.set_popup_focus(true);
    }
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let shell_buffer =
        ShellBuffer::from_runtime_buffer(buffer, user_library.oil_help_lines(), &*user_library);
    shell_ui_mut(runtime)?.insert_buffer(shell_buffer);
    Ok(())
}

fn open_external_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("path does not exist: {}", path.display()));
    }
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("cmd");
        configure_background_command(&mut command);
        command
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    }
    Ok(())
}

/// Evaluate the input section of any evaluatable plugin buffer and replace the
/// output section with the result.  Called both by the generic Ctrl+c Ctrl+c
/// handler and by the `plugin.evaluate` hook subscriber.
fn evaluate_active_plugin_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    // Read the buffer kind (needed to route the evaluate call).
    let (input_lines, sep_line, kind_str, has_plugin_sections) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let kind_str = if let BufferKind::Plugin(k) = &buffer.kind {
            k.clone()
        } else {
            return Ok(()); // not a plugin buffer; nothing to do
        };
        if buffer.has_plugin_sections() {
            let line_count = buffer.text.line_count();
            let all_lines: Vec<String> = (0..line_count)
                .map(|i| buffer.text.line(i).unwrap_or_default().to_owned())
                .collect();
            (all_lines, String::new(), kind_str, true)
        } else {
            let line_count = buffer.text.line_count();
            let all_lines: Vec<String> = (0..line_count)
                .map(|i| buffer.text.line(i).unwrap_or_default().to_owned())
                .collect();
            if let Some(idx) = all_lines
                .iter()
                .position(|l| l.starts_with(PLUGIN_EVALUATE_SEPARATOR_PREFIX))
            {
                let input = all_lines[..idx].to_vec();
                let sep = all_lines[idx].clone();
                (input, sep, kind_str, false)
            } else {
                // No separator — treat everything as input; add a fresh separator.
                let sep = format!("{} {}", PLUGIN_EVALUATE_SEPARATOR_PREFIX, "─".repeat(48));
                (all_lines, sep, kind_str, false)
            }
        }
    };

    let input_text = input_lines.join("\n");

    // Call user library evaluator (no mutable borrow of runtime required).
    let output = shell_user_library(runtime).handle_plugin_evaluate(&kind_str, &input_text);

    if has_plugin_sections {
        shell_buffer_mut(runtime, buffer_id)?.set_plugin_output_lines(output);
        return Ok(());
    }

    // Rebuild: input + separator + output.
    let mut new_lines = input_lines;
    new_lines.push(sep_line);
    new_lines.extend(output);

    shell_buffer_mut(runtime, buffer_id)?.replace_with_lines(new_lines);
    Ok(())
}

fn switch_active_plugin_pane(
    runtime: &mut EditorRuntime,
    buffer_id: Option<BufferId>,
) -> Result<(), String> {
    let buffer_id = buffer_id.unwrap_or(active_shell_buffer_id(runtime)?);
    let switched_to_read_only = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if buffer.plugin_switch_pane() {
            buffer.is_read_only()
        } else {
            return acp::acp_switch_pane(runtime);
        }
    };
    if switched_to_read_only {
        shell_ui_mut(runtime)?.enter_normal_mode();
    }
    Ok(())
}

// ─── Generic compile / build infrastructure ───────────────────────────────────

/// Buffer name pattern for the compilation popup.
fn compile_buffer_name(workspace_name: &str) -> String {
    format!("*compile {workspace_name}*")
}

fn command_output_buffer_name(workspace_name: &str) -> String {
    format!("*command {workspace_name}*")
}

/// Open (or focus) the `*compile <workspace>*` compilation buffer and
/// pre-fill its input field with the default build command for `language`
/// (obtained from the user library).  The user can edit the command and press
/// Ctrl+Enter to run it.
///
/// Called by the `plugin.run-command` hook subscriber.
fn open_compile_buffer(
    runtime: &mut EditorRuntime,
    language_hint: Option<&str>,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace_name = runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?
        .name()
        .to_owned();
    let buf_name = compile_buffer_name(&workspace_name);

    // Reuse an existing buffer if present.
    let existing = shell_ui(runtime).ok().and_then(|ui| {
        ui.buffers
            .iter()
            .find(|b| b.display_name() == buf_name)
            .map(|b| b.id())
    });

    let buffer_id = if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.focus_buffer_in_active_pane(existing);
        ui.enter_normal_mode();
        existing
    } else {
        let id = runtime
            .model_mut()
            .create_buffer(workspace_id, &buf_name, BufferKind::Compilation, None)
            .map_err(|error| error.to_string())?;
        let buffer = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|e| e.to_string())?
            .buffer(id)
            .ok_or_else(|| format!("buffer `{id}` is missing"))?;
        let user_library = shell_user_library(runtime);
        let initial = vec![format!("# {workspace_name} — compilation output")];
        let mut shell_buf = ShellBuffer::from_runtime_buffer(buffer, initial, &*user_library);
        // Pre-fill the input field with the default build command.
        let default_cmd = language_hint
            .and_then(|lang| user_library.default_build_command(lang))
            .unwrap_or_default();
        if let Some(input) = shell_buf.input_field_mut() {
            input.set_text(&default_cmd);
        }
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buf);
        ui.focus_buffer_in_active_pane(id);
        ui.enter_normal_mode();
        id
    };

    // If the buffer already has a stored command for this workspace, pre-fill it.
    let stored = shell_ui(runtime)
        .ok()
        .and_then(|ui| ui.compile_commands.get(&workspace_id).cloned());
    if let Some(cmd) = stored
        && let Some(buf) = shell_ui_mut(runtime)
            .ok()
            .and_then(|ui| ui.buffer_mut(buffer_id))
        && let Some(input) = buf.input_field_mut()
    {
        input.set_text(&cmd);
    }

    Ok(())
}

fn open_command_output_buffer(runtime: &mut EditorRuntime) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace_name = runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?
        .name()
        .to_owned();
    let buf_name = command_output_buffer_name(&workspace_name);
    let existing = shell_ui(runtime).ok().and_then(|ui| {
        ui.buffers
            .iter()
            .find(|buffer| buffer.display_name() == buf_name)
            .map(ShellBuffer::id)
    });
    if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.focus_buffer_in_active_pane(existing);
        ui.enter_normal_mode();
        return Ok(existing);
    }
    let id = runtime
        .model_mut()
        .create_buffer(workspace_id, &buf_name, BufferKind::Compilation, None)
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(id)
        .ok_or_else(|| format!("buffer `{id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let initial = vec![format!("# {workspace_name} — command output")];
    let shell_buf = ShellBuffer::from_runtime_buffer(buffer, initial, &*user_library);
    let ui = shell_ui_mut(runtime)?;
    ui.insert_buffer(shell_buf);
    ui.focus_buffer_in_active_pane(id);
    ui.enter_normal_mode();
    Ok(id)
}

fn run_shell_command_from_vim_command_line(
    runtime: &mut EditorRuntime,
    command: &str,
) -> Result<(), String> {
    let command = command.trim();
    if command.is_empty() {
        return Err(":! requires a shell command".to_owned());
    }
    let buffer_id = open_command_output_buffer(runtime)?;
    run_shell_command_in_buffer(runtime, buffer_id, command)
}

fn run_shell_command_in_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    command: &str,
) -> Result<(), String> {
    let command = command.trim().to_owned();
    if command.is_empty() {
        return Ok(());
    }
    let terminal_config = shell_user_library(runtime).terminal_config();
    let cwd = active_workspace_root(runtime)
        .ok()
        .flatten()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut args = terminal_config.args;
    let shell_program = terminal_config.program;
    args.push(shell_command_eval_flag(&shell_program).to_owned());
    args.push(command.clone());
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!("$ {command}"), String::new()]);
        buffer.clear_input();
    }
    let spec = JobSpec::command("command", shell_program, args).with_cwd(cwd);
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
    let transcript = result.transcript();
    let output_lines: Vec<String> = transcript.lines().map(str::to_owned).collect();
    let status_line = if result.succeeded() {
        "── ✓ Command succeeded ────────────────────────────────────────────────".to_owned()
    } else {
        format!(
            "── ✗ Command failed (exit {}) ──────────────────────────────────────",
            result.exit_code().unwrap_or(-1)
        )
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.append_output_lines(&output_lines);
    buffer.append_output_lines(&[status_line]);
    Ok(())
}

fn shell_command_eval_flag(program: &str) -> &'static str {
    let shell = Path::new(program)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    if cfg!(target_os = "windows") {
        if shell.eq_ignore_ascii_case("cmd") {
            "/C"
        } else if shell.eq_ignore_ascii_case("powershell") || shell.eq_ignore_ascii_case("pwsh") {
            "-Command"
        } else {
            "-c"
        }
    } else {
        "-c"
    }
}

/// Re-run the last stored build command for the active workspace.
/// If no command has been stored yet, falls back to opening the compile buffer.
fn rerun_compile_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let stored = shell_ui(runtime)
        .ok()
        .and_then(|ui| ui.compile_commands.get(&workspace_id).cloned());
    if let Some(cmd) = stored {
        let workspace_name = runtime
            .model()
            .active_workspace()
            .map_err(|error| error.to_string())?
            .name()
            .to_owned();
        let buf_name = compile_buffer_name(&workspace_name);
        let buf_id = shell_ui(runtime).ok().and_then(|ui| {
            ui.buffers
                .iter()
                .find(|b| b.display_name() == buf_name)
                .map(|b| b.id())
        });
        if let Some(buffer_id) = buf_id {
            run_compile_command_in_buffer(runtime, buffer_id, &cmd)
        } else {
            open_compile_buffer(runtime, None)
        }
    } else {
        open_compile_buffer(runtime, None)
    }
}

/// Run `command` in the compilation buffer `buffer_id`, capturing stdout +
/// stderr into it.  Stores the command as the active workspace's last command.
fn run_compile_command_in_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    command: &str,
) -> Result<(), String> {
    let command = command.trim().to_owned();
    if command.is_empty() {
        return Ok(());
    }

    // Parse into program + args.
    let mut parts = command.split_whitespace();
    let program = parts.next().unwrap_or("").to_owned();
    let args: Vec<String> = parts.map(str::to_owned).collect();

    // Determine working directory (workspace root or cwd).
    let cwd = active_workspace_root(runtime)
        .ok()
        .flatten()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Store command for this workspace.
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    if let Ok(ui) = shell_ui_mut(runtime) {
        ui.compile_commands.insert(workspace_id, command.clone());
    }

    // Write header to buffer.
    {
        let buf = shell_buffer_mut(runtime, buffer_id)?;
        buf.append_output_lines(&[format!("$ {command}"), String::new()]);
        buf.clear_input();
    }

    // Spawn the job and wait (synchronously — same pattern as git commands).
    let spec = JobSpec::command("compile", &program, args).with_cwd(cwd);
    let manager = runtime
        .services()
        .get::<Mutex<JobManager>>()
        .ok_or_else(|| "job manager service missing".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "job manager lock poisoned".to_owned())?;
    let handle = manager.spawn(spec).map_err(|e| e.to_string())?;
    drop(manager);
    let result = handle.wait().map_err(|e| e.to_string())?;

    // Write output to the buffer.
    let transcript = result.transcript();
    let output_lines: Vec<String> = transcript.lines().map(str::to_owned).collect();
    let status_line = if result.succeeded() {
        "── ✓ Build succeeded ──────────────────────────────────────────────────".to_owned()
    } else {
        format!(
            "── ✗ Build failed (exit {}) ─────────────────────────────────────────",
            result.exit_code().unwrap_or(-1)
        )
    };
    let buf = shell_buffer_mut(runtime, buffer_id)?;
    buf.append_output_lines(&output_lines);
    buf.append_output_lines(&[status_line]);
    Ok(())
}

/// In a compilation buffer, jump to the file location on the current line
/// by parsing `path:line` or `path:line:col`.
fn jump_to_compilation_error(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let line_text = {
        let buf = shell_buffer(runtime, buffer_id)?;
        let cursor_line = buf.cursor_point().line;
        buf.text.line(cursor_line).unwrap_or_default().to_owned()
    };

    // Parse the error line via the user library's compile module pattern.
    // We use the same logic as user::compile::parse_error_location but replicated
    // here generically so the shell does not depend on user code at parse time.
    let parsed = parse_compilation_error_line(&line_text);
    let (path, line_num, _col) = match parsed {
        Some(loc) => loc,
        None => return Ok(()), // not an error line, silently ignore
    };

    // Determine the absolute path (relative to workspace root if needed).
    let root = active_workspace_root(runtime).ok().flatten();
    let abs_path = if std::path::Path::new(&path).is_absolute() {
        PathBuf::from(&path)
    } else if let Some(ref root) = root {
        root.join(&path)
    } else {
        PathBuf::from(&path)
    };

    // Find or open the file buffer and navigate to the line.
    open_file_at_line(runtime, &abs_path, line_num)
}

/// Generic compilation error line parser.  Handles:
/// - `path:line:col`
/// - `path:line`
/// - `  --> path:line:col` (Rust rustc style)
fn parse_compilation_error_line(line: &str) -> Option<(String, u32, u32)> {
    let line = line.trim();
    let line = line.strip_prefix("-->").map(str::trim).unwrap_or(line);
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    match parts.as_slice() {
        [path, line_str, col_str, ..] => {
            let line_num = line_str.trim().parse::<u32>().ok()?;
            let col_num = col_str
                .trim()
                .split_once(|c: char| !c.is_ascii_digit())
                .and_then(|(n, _)| n.parse().ok())
                .or_else(|| col_str.trim().parse().ok())
                .unwrap_or(1);
            if !path.is_empty() && line_num > 0 {
                return Some(((*path).to_owned(), line_num, col_num));
            }
            None
        }
        [path, line_str] => {
            let line_num = line_str.trim().parse::<u32>().ok()?;
            if !path.is_empty() && line_num > 0 {
                return Some(((*path).to_owned(), line_num, 1));
            }
            None
        }
        _ => None,
    }
}

/// Open `path` in the most-recently-active non-compilation buffer and move
/// the cursor to `line_num`.
fn open_file_at_line(
    runtime: &mut EditorRuntime,
    path: &Path,
    line_num: u32,
) -> Result<(), String> {
    open_workspace_file_at(
        runtime,
        path,
        TextPoint::new(line_num.saturating_sub(1) as usize, 0),
    )?;
    shell_ui_mut(runtime)?.enter_normal_mode();
    Ok(())
}

fn handle_directory_keydown_chord(
    runtime: &mut EditorRuntime,
    chord: &str,
) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_directory(&buffer.kind) {
        return Ok(false);
    }
    shell_ui_mut(runtime)?.pending_directory_prefix = None;
    let user_library = shell_user_library(runtime);
    match user_library.oil_keydown_action(chord) {
        Some(OilKeyAction::OpenEntry) => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(runtime, buffer_id, entry, DirectoryOpenMode::Current)?;
            Ok(true)
        }
        Some(OilKeyAction::OpenVerticalSplit) => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(runtime, buffer_id, entry, DirectoryOpenMode::SplitVertical)?;
            Ok(true)
        }
        Some(OilKeyAction::OpenHorizontalSplit) => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(
                runtime,
                buffer_id,
                entry,
                DirectoryOpenMode::SplitHorizontal,
            )?;
            Ok(true)
        }
        Some(OilKeyAction::OpenNewPane) => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(runtime, buffer_id, entry, DirectoryOpenMode::NewPane)?;
            Ok(true)
        }
        Some(OilKeyAction::PreviewEntry) => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(runtime, buffer_id, entry, DirectoryOpenMode::Preview)?;
            Ok(true)
        }
        Some(OilKeyAction::Refresh) => {
            refresh_directory_buffer(runtime, buffer_id)?;
            Ok(true)
        }
        Some(OilKeyAction::Close) => {
            close_buffer_discard(runtime, buffer_id)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn handle_directory_chord(runtime: &mut EditorRuntime, chord: &str) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_directory(&buffer.kind) {
        return Ok(false);
    }
    let had_prefix = take_directory_prefix(runtime)?;
    let user_library = shell_user_library(runtime);
    match user_library.oil_chord_action(had_prefix, chord) {
        Some(OilKeyAction::ShowHelp) => {
            open_oil_help_popup(runtime)?;
            Ok(true)
        }
        Some(OilKeyAction::ToggleHidden) => {
            update_directory_state(runtime, buffer_id, |state| {
                state.show_hidden = !state.show_hidden;
            })?;
            Ok(true)
        }
        Some(OilKeyAction::ToggleTrash) => {
            update_directory_state(runtime, buffer_id, |state| {
                state.trash_enabled = !state.trash_enabled;
            })?;
            Ok(true)
        }
        Some(OilKeyAction::CycleSort) => {
            update_directory_state(runtime, buffer_id, |state| {
                state.sort_mode = state.sort_mode.cycle();
            })?;
            Ok(true)
        }
        Some(OilKeyAction::OpenExternal) => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_external_path(entry.path())?;
            Ok(true)
        }
        Some(OilKeyAction::SetTabLocalRoot) | Some(OilKeyAction::SetRoot) => {
            directory_cd_from_cursor(runtime, buffer_id)?;
            Ok(true)
        }
        Some(OilKeyAction::StartPrefix) => {
            set_directory_prefix(runtime)?;
            Ok(true)
        }
        Some(OilKeyAction::OpenParent) => {
            let root = oil_parent_root(runtime)?;
            set_directory_root(runtime, buffer_id, root)?;
            Ok(true)
        }
        Some(OilKeyAction::OpenWorkspaceRoot) => {
            let root = oil_workspace_root(runtime)?;
            set_directory_root(runtime, buffer_id, root)?;
            Ok(true)
        }
        None if had_prefix => {
            let prefix = user_library.oil_keybindings().prefix;
            record_runtime_error(
                runtime,
                "oil.directory",
                format!("unknown oil {prefix} action `{chord}`"),
            );
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn refresh_pending_syntax(runtime: &mut EditorRuntime) -> Result<SyntaxRefreshStats, String> {
    let mut stats = SyntaxRefreshStats::default();
    if let Some(buffer_id) = shell_ui(runtime)?.active_buffer_id()
        && let Some(buffer) = shell_ui_mut(runtime)?.buffer_mut(buffer_id)
    {
        buffer.ensure_visible_syntax_window();
    }
    let syntax_results = shell_ui(runtime)?.syntax_refresh_worker.take_results();
    if !syntax_results.is_empty() {
        let ui = shell_ui_mut(runtime)?;
        for result in syntax_results {
            let Some(buffer) = ui.buffer_mut(result.buffer_id) else {
                continue;
            };
            let current_path = buffer.path().map(Path::to_path_buf);
            let current_language_id = buffer.language_id().map(str::to_owned);
            if buffer.text.revision() != result.buffer_revision
                || current_path.as_deref() != result.path.as_deref()
                || current_language_id.as_deref() != result.buffer_language_id.as_deref()
            {
                continue;
            }
            stats.changed = true;
            stats.worker_compute += result.compute_elapsed;
            stats.result_count = stats.result_count.saturating_add(1);
            stats.highlight_spans = stats
                .highlight_spans
                .saturating_add(result.highlight_span_count);
            buffer.set_language_id(result.language_id.clone());
            match result.syntax_result {
                Some(Ok(syntax_lines)) => {
                    buffer.set_indexed_syntax_lines(Some(syntax_lines), result.syntax_window);
                    buffer.set_syntax_error(None);
                }
                Some(Err(error)) => {
                    let error_label = result
                        .path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .or(result.language_id.clone())
                        .unwrap_or_else(|| "buffer".to_owned());
                    eprintln!("tree-sitter syntax refresh failed for `{error_label}`: {error}");
                    buffer.set_syntax_snapshot(None);
                    buffer.set_syntax_error(Some(error));
                }
                None => {
                    buffer.set_syntax_snapshot(None);
                    buffer.set_syntax_error(None);
                    buffer.set_language_id(None);
                }
            }
        }
    }

    if !shell_ui(runtime)?.syntax_refresh_worker.is_configured() {
        let now = Instant::now();
        let buffer_ids = {
            let ui = shell_ui(runtime)?;
            ui.buffers
                .iter()
                .filter(|buffer| buffer.syntax_refresh_due(now))
                .map(ShellBuffer::id)
                .collect::<Vec<_>>()
        };
        let had_due_buffers = !buffer_ids.is_empty();

        for buffer_id in buffer_ids {
            refresh_buffer_syntax(runtime, buffer_id)?;
        }

        stats.changed = stats.changed || had_due_buffers;
        return Ok(stats);
    }

    let now = Instant::now();
    let requests = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .filter(|buffer| buffer.syntax_refresh_due(now))
            .map(|buffer| SyntaxRefreshWorkerRequest {
                buffer_id: buffer.id(),
                buffer_revision: buffer.text.revision(),
                path: buffer.path().map(Path::to_path_buf),
                buffer_language_id: buffer.language_id().map(str::to_owned),
                syntax_window: buffer.desired_syntax_window(),
                text: buffer.text.clone(),
            })
            .collect::<Vec<_>>()
    };

    if requests.is_empty() {
        return Ok(stats);
    }

    {
        let ui = shell_ui_mut(runtime)?;
        for request in &requests {
            if let Some(buffer) = ui.buffer_mut(request.buffer_id) {
                buffer.mark_syntax_refresh_requested(request.syntax_window);
            }
        }
        for request in requests {
            ui.syntax_refresh_worker.send(request);
        }
    }

    Ok(stats)
}

fn refresh_pending_file_reloads(
    runtime: &mut EditorRuntime,
    _now: Instant,
    force: bool,
) -> Result<bool, String> {
    let mut changed = false;
    if force {
        let buffer_ids = {
            let ui = shell_ui(runtime)?;
            ui.buffers.iter().map(ShellBuffer::id).collect::<Vec<_>>()
        };
        for buffer_id in buffer_ids {
            let did_reload = {
                let ui = shell_ui_mut(runtime)?;
                let Some(buffer) = ui.buffer_mut(buffer_id) else {
                    continue;
                };
                buffer.reload_from_disk_if_changed(true)?
            };
            changed |= did_reload;
        }
        return Ok(changed);
    }

    let mut first_error = None;
    let watcher_errors = shell_ui(runtime)?.file_reload_worker.take_errors();
    if first_error.is_none() {
        first_error = watcher_errors.into_iter().next();
    }
    let changed_paths = shell_ui(runtime)?.file_reload_worker.take_changed_paths();
    if !changed_paths.is_empty() {
        let changed_paths = changed_paths.into_iter().collect::<HashSet<_>>();
        let ui = shell_ui_mut(runtime)?;
        for buffer in &mut ui.buffers {
            let Some(path) = shell_buffer_watch_path(buffer) else {
                continue;
            };
            if changed_paths.contains(&path) {
                buffer.mark_backing_file_reload_pending();
            }
        }
    }
    {
        let ui = shell_ui_mut(runtime)?;
        for buffer in &mut ui.buffers {
            if !buffer.is_pdf_buffer() || !buffer.backing_file_reload_pending {
                continue;
            }
            changed |= buffer.reload_from_disk_if_changed(false)?;
        }
    }
    let results = shell_ui(runtime)?.file_reload_worker.take_results();
    if !results.is_empty() {
        let ui = shell_ui_mut(runtime)?;
        for result in results {
            let Some(buffer) = ui.buffer_mut(result.buffer_id) else {
                continue;
            };
            buffer.finish_file_reload_request();
            let current_path = buffer.path().map(Path::to_path_buf);
            if buffer.text.revision() != result.buffer_revision
                || current_path.as_deref() != Some(result.path.as_path())
            {
                continue;
            }
            match result.outcome {
                Ok(FileReloadWorkerOutcome::Missing) => {}
                Ok(FileReloadWorkerOutcome::Unchanged { fingerprint }) => {
                    buffer.backing_file_fingerprint = Some(fingerprint);
                }
                Ok(FileReloadWorkerOutcome::Reloaded { fingerprint, text }) => {
                    changed |= buffer.apply_reloaded_file_buffer(fingerprint, text);
                }
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }
    }

    let requests = {
        let ui = shell_ui_mut(runtime)?;
        ui.buffers
            .iter_mut()
            .filter_map(ShellBuffer::file_reload_request)
            .collect::<Vec<_>>()
    };
    if !requests.is_empty() {
        let ui = shell_ui(runtime)?;
        for request in requests {
            ui.file_reload_worker.send(request);
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }

    Ok(changed)
}

fn refresh_pending_git(
    runtime: &mut EditorRuntime,
    now: Instant,
    typing_active: bool,
) -> Result<(), String> {
    refresh_pending_git_fringe(runtime, now, typing_active)?;
    refresh_pending_git_summary(runtime, now, typing_active)?;
    Ok(())
}

#[derive(Debug)]
struct LspBufferRefreshRequest {
    path: PathBuf,
    revision: u64,
    text: String,
    root: Option<PathBuf>,
}

fn refresh_pending_lsp(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let now = Instant::now();
    let Some(lsp_client) = runtime.services().get::<Arc<LspClientManager>>().cloned() else {
        return Ok(false);
    };

    let requests = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .map(|buffer| {
                if !buffer.lsp_enabled() {
                    return Ok(None);
                }
                let Some(path) = buffer.path().map(Path::to_path_buf) else {
                    return Ok(None);
                };
                if path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_none()
                {
                    return Ok(None);
                }
                if !lsp_client.needs_sync(&path, buffer.text.revision()) {
                    return Ok(None);
                }
                Ok(Some(LspBufferRefreshRequest {
                    path: path.clone(),
                    revision: buffer.text.revision(),
                    text: buffer.text.text(),
                    root: workspace_root_for_path(runtime, &path)?,
                }))
            })
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    };

    for request in requests {
        lsp_client
            .sync_buffer(
                &request.path,
                &request.text,
                request.revision,
                request.root.as_deref(),
            )
            .map_err(|error| error.to_string())?;
    }

    let (
        diagnostic_updates,
        active_workspace_id,
        active_server_label,
        log_snapshot,
        notification_snapshot,
    ) = {
        let ui = shell_ui(runtime)?;
        let updates = ui
            .buffers
            .iter()
            .filter(|buffer| buffer.lsp_enabled())
            .filter_map(|buffer| {
                let path = buffer.path()?;
                Some((buffer.id(), lsp_client.diagnostics_for_path(path)))
            })
            .collect::<Vec<_>>();
        let active_server_label = ui
            .active_buffer_id()
            .and_then(|buffer_id| ui.buffer(buffer_id))
            .and_then(ShellBuffer::path)
            .map(|path| lsp_client.session_labels_for_path(path))
            .filter(|labels| !labels.is_empty())
            .map(|labels| labels.join(", "));
        (
            updates,
            ui.active_workspace(),
            active_server_label,
            lsp_client.log_snapshot(),
            lsp_client.notification_snapshot(),
        )
    };

    let mut changed = false;
    {
        let ui = shell_ui_mut(runtime)?;
        for (buffer_id, diagnostics) in diagnostic_updates {
            if let Some(buffer) = ui.buffer_mut(buffer_id) {
                changed |= buffer.set_lsp_diagnostics(diagnostics);
            }
        }
        changed |= ui.set_attached_lsp_server(active_workspace_id, active_server_label);
        changed |= apply_lsp_notifications(ui, &notification_snapshot, now);
    }
    changed |= refresh_lsp_log_buffers(runtime, &log_snapshot)?;
    Ok(changed)
}

fn notification_severity(level: LspNotificationLevel) -> NotificationSeverity {
    match level {
        LspNotificationLevel::Info => NotificationSeverity::Info,
        LspNotificationLevel::Success => NotificationSeverity::Success,
        LspNotificationLevel::Warning => NotificationSeverity::Warning,
        LspNotificationLevel::Error => NotificationSeverity::Error,
    }
}

fn apply_lsp_notifications(
    ui: &mut ShellUiState,
    snapshot: &LspNotificationSnapshot,
    now: Instant,
) -> bool {
    let mut changed = false;
    let last_seen = ui.last_lsp_notification_revision();
    for entry in snapshot.entries() {
        if entry.revision() <= last_seen {
            continue;
        }
        let notification = entry.notification();
        let progress = notification
            .progress()
            .map(|progress| NotificationProgress {
                percentage: progress
                    .percentage()
                    .and_then(|percentage| u8::try_from(percentage.min(u32::from(u8::MAX))).ok()),
            });
        changed |= ui.apply_notification(
            NotificationUpdate {
                key: notification.key().to_owned(),
                severity: notification_severity(notification.level()),
                title: notification.title().to_owned(),
                body_lines: notification.body_lines().to_vec(),
                progress,
                active: notification.active(),
                action: None,
            },
            now,
        );
    }
    ui.set_last_lsp_notification_revision(snapshot.revision());
    changed
}

fn refresh_lsp_log_buffers(
    runtime: &mut EditorRuntime,
    snapshot: &LspLogSnapshot,
) -> Result<bool, String> {
    let (workspace_buffers, applied_revision) = {
        let Some(state) = runtime.services().get::<LspLogBufferState>() else {
            return Ok(false);
        };
        (
            state
                .buffer_ids
                .iter()
                .map(|(workspace_id, buffers)| {
                    (
                        *workspace_id,
                        buffers
                            .iter()
                            .map(|(server_id, buffer_id)| (server_id.clone(), *buffer_id))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
            state.applied_revision,
        )
    };
    if snapshot.revision() == applied_revision {
        return Ok(false);
    }
    let active_workspace = runtime.model().active_workspace_id().ok();
    let server_ids = snapshot
        .entries()
        .iter()
        .map(LspLogEntry::server_id)
        .collect::<BTreeSet<_>>();
    if let Some(workspace_id) = active_workspace {
        for server_id in server_ids {
            let _ = ensure_lsp_log_buffer(runtime, workspace_id, server_id)?;
        }
    }
    let had_buffers = !workspace_buffers.is_empty();
    {
        let ui = shell_ui_mut(runtime)?;
        for (_workspace_id, buffers) in &workspace_buffers {
            for (server_id, buffer_id) in buffers {
                let entries = lsp_log_entries_for_server(snapshot.entries(), server_id);
                if let Some(buffer) = ui.buffer_mut(*buffer_id) {
                    buffer.replace_with_lines_follow_output(lsp_log_buffer_lines(
                        server_id, &entries,
                    ));
                }
            }
        }
    }
    if let Some(state) = runtime.services_mut().get_mut::<LspLogBufferState>() {
        state.applied_revision = snapshot.revision();
    }
    Ok(had_buffers || !snapshot.entries().is_empty())
}

fn refresh_pending_git_fringe(
    runtime: &mut EditorRuntime,
    now: Instant,
    typing_active: bool,
) -> Result<(), String> {
    let buffer_ids = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .filter(|buffer| buffer.git_fringe_refresh_due(now, typing_active))
            .map(ShellBuffer::id)
            .collect::<Vec<_>>()
    };

    for buffer_id in buffer_ids {
        refresh_git_fringe(runtime, buffer_id)?;
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

fn workspace_root_for_path(
    runtime: &EditorRuntime,
    path: &Path,
) -> Result<Option<PathBuf>, String> {
    let window = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?;
    Ok(window
        .workspaces()
        .filter_map(|workspace| workspace.root())
        .filter(|root| path.starts_with(root))
        .max_by_key(|root| root.components().count())
        .map(Path::to_path_buf))
}

fn workspace_root_readme_path(root: &Path) -> Result<Option<PathBuf>, String> {
    let mut candidates = fs::read_dir(root)
        .map_err(|error| {
            format!(
                "failed to read workspace root `{}`: {error}",
                root.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_file() {
                return None;
            }

            let path = entry.path();
            let stem = path.file_stem()?.to_str()?;
            stem.eq_ignore_ascii_case("readme").then_some(path)
        })
        .collect::<Vec<_>>();
    candidates.sort_by_cached_key(|path| readme_path_priority(path));
    Ok(candidates.into_iter().next())
}

fn readme_path_priority(path: &Path) -> (u8, String) {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let priority = match file_name.as_str() {
        "readme.md" => 0,
        "readme" => 1,
        _ => 2,
    };
    (priority, file_name)
}

fn git_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_workspace_root(runtime)? {
        return Ok(root);
    }
    env::current_dir().map_err(|error| format!("git status requires a workspace root: {error}"))
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

pub(crate) fn switch_runtime_workspace(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    runtime
        .model_mut()
        .switch_workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.switch_workspace(workspace_id);
    let window_id = active_window_id(runtime)?;
    let workspace_name = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .name()
        .to_owned();
    runtime
        .emit_hook(
            builtins::WORKSPACE_SWITCH,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_detail(workspace_name),
        )
        .map_err(|error| error.to_string())?;
    sync_active_buffer(runtime)
}

pub(crate) fn open_workspace_from_project(
    runtime: &mut EditorRuntime,
    name: &str,
    root: &std::path::Path,
) -> Result<WorkspaceId, String> {
    if let Some(workspace_id) = find_workspace_by_root(runtime, root)? {
        switch_runtime_workspace(runtime, workspace_id)?;
        return Ok(workspace_id);
    }

    let initial_readme_path = workspace_root_readme_path(root)?;
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

    let (scratch, notes, primary_pane_id) = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let pane_id = workspace
            .active_pane_id()
            .ok_or_else(|| "new workspace has no active pane".to_owned())?;
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
                &*shell_user_library(runtime),
            ),
            ShellBuffer::from_runtime_buffer(
                notes,
                workspace_notes_lines(workspace.name(), workspace.root()),
                &*shell_user_library(runtime),
            ),
            pane_id,
        )
    };

    {
        let ui = shell_ui_mut(runtime)?;
        ui.add_workspace(workspace_id, primary_pane_id, scratch, notes, notes_id);
        ui.switch_workspace(workspace_id);
    }

    if let Some(readme_path) = initial_readme_path {
        open_workspace_file(runtime, &readme_path)?;
    }

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

pub(crate) fn delete_runtime_workspace(
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
    close_lsp_buffers_for_workspace(runtime, workspace_id)?;
    close_terminal_buffers_for_workspace(runtime, workspace_id)?;
    acp::close_acp_workspace_buffers(runtime, workspace_id)?;
    let removed = runtime
        .model_mut()
        .close_workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.remove_workspace(workspace_id);
    if let Some(state) = runtime.services_mut().get_mut::<LspLogBufferState>() {
        state.remove_workspace(workspace_id);
    }
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
        let ui = shell_ui_mut(runtime)?;
        ui.set_popup_focus(false);
        ui.clear_popup_buffer();
        return Ok(());
    }

    let buffer_id = runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup*", BufferKind::Diagnostics, None)
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![buffer_id], buffer_id)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_popup_buffer(
            buffer_id,
            "*popup*",
            BufferKind::Diagnostics,
            &*user_library,
        );
        ui.set_popup_buffer(buffer_id);
    }
    shell_ui_mut(runtime)?.set_popup_focus(true);
    Ok(())
}

fn cycle_runtime_popup_buffer(runtime: &mut EditorRuntime, forward: bool) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = runtime
        .model_mut()
        .cycle_popup_buffer(workspace_id, forward)
        .map_err(|error| error.to_string())?;
    let Some(buffer_id) = buffer_id else {
        return Ok(());
    };
    ensure_shell_buffer(runtime, buffer_id)?;
    shell_ui_mut(runtime)?.set_popup_buffer(buffer_id);
    Ok(())
}

fn split_runtime_pane(
    runtime: &mut EditorRuntime,
    direction: PaneSplitDirection,
) -> Result<(), String> {
    let split_buffer_id = {
        let ui = shell_ui(runtime)?;
        if ui.pane_count() > 1 {
            return Ok(());
        }
        ui.split_buffer_id()
            .ok_or_else(|| "active workspace view is missing".to_owned())?
    };
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let pane_id = runtime
        .model_mut()
        .split_pane(workspace_id, split_buffer_id)
        .map_err(|error| error.to_string())?;
    let (buffer_name, buffer_kind) = {
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let buffer = workspace
            .buffer(split_buffer_id)
            .ok_or_else(|| format!("buffer `{split_buffer_id}` is missing"))?;
        (buffer.name().to_owned(), buffer.kind().clone())
    };
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(split_buffer_id, &buffer_name, buffer_kind, &*user_library);
        ui.split_pane(pane_id, split_buffer_id, direction);
    }
    let window_id = active_window_id(runtime)?;
    let hook_name = match direction {
        PaneSplitDirection::Horizontal => builtins::PANE_SPLIT_HORIZONTAL,
        PaneSplitDirection::Vertical => builtins::PANE_SPLIT_VERTICAL,
    };
    runtime
        .emit_hook(
            hook_name,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_pane(pane_id)
                .with_buffer(split_buffer_id),
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn close_runtime_pane(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let pane_id = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .active_pane_id()
        .ok_or_else(|| format!("workspace `{workspace_id}` has no active pane"))?;
    runtime
        .model_mut()
        .close_pane(workspace_id, pane_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.close_pane(pane_id);
    sync_active_buffer(runtime)?;

    let (active_pane_id, active_buffer_id) = active_runtime_buffer(runtime)?
        .map(|(active_pane_id, active_buffer_id, _, _)| (active_pane_id, active_buffer_id))
        .ok_or_else(|| "active runtime surface is missing after closing pane".to_owned())?;
    let window_id = active_window_id(runtime)?;
    runtime
        .emit_hook(
            builtins::PANE_SWITCH,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id)
                .with_pane(active_pane_id)
                .with_buffer(active_buffer_id),
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn switch_runtime_split(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui_mut(runtime)?.switch_split() {
        return Ok(());
    }
    Err("switch split requires an active split".to_owned())
}

fn cycle_runtime_pane(runtime: &mut EditorRuntime) -> Result<(), String> {
    let pane_id = shell_ui_mut(runtime)?.cycle_active_pane();
    let Some(pane_id) = pane_id else {
        return Ok(());
    };
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_pane(workspace_id, pane_id)
        .map_err(|error| error.to_string())?;
    let window_id = active_window_id(runtime)?;
    let buffer_id = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .pane(pane_id)
        .and_then(|pane| pane.active_buffer());
    let mut event = HookEvent::new()
        .with_window(window_id)
        .with_workspace(workspace_id)
        .with_pane(pane_id);
    if let Some(buffer_id) = buffer_id {
        event = event.with_buffer(buffer_id);
    }
    runtime
        .emit_hook(builtins::PANE_SWITCH, event)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn move_workspace_window(
    runtime: &mut EditorRuntime,
    direction: WindowMoveDirection,
) -> Result<(), String> {
    if let Some(popup) = active_runtime_popup(runtime)? {
        let (focus_allowed, focus_active) = {
            let ui = shell_ui(runtime)?;
            (
                ui.popup_focus_allowed(&popup),
                ui.popup_focus_active(&popup),
            )
        };
        if focus_allowed {
            let emit_switch = |runtime: &mut EditorRuntime| -> Result<(), String> {
                let window_id = active_window_id(runtime)?;
                let workspace_id = runtime
                    .model()
                    .active_workspace_id()
                    .map_err(|error| error.to_string())?;
                runtime
                    .emit_hook(
                        builtins::PANE_SWITCH,
                        HookEvent::new()
                            .with_window(window_id)
                            .with_workspace(workspace_id),
                    )
                    .map_err(|error| error.to_string())
            };
            if !focus_active && direction == WindowMoveDirection::Down {
                let ui = shell_ui_mut(runtime)?;
                ui.set_popup_buffer(popup.active_buffer);
                ui.set_popup_focus(true);
                emit_switch(runtime)?;
                return Ok(());
            }
            if focus_active
                && matches!(
                    direction,
                    WindowMoveDirection::Up | WindowMoveDirection::Left
                )
            {
                shell_ui_mut(runtime)?.set_popup_focus(false);
                emit_switch(runtime)?;
                return Ok(());
            }
            if focus_active {
                return Ok(());
            }
        }
    }

    let delta = match direction {
        WindowMoveDirection::Left | WindowMoveDirection::Up => -1,
        WindowMoveDirection::Right | WindowMoveDirection::Down => 1,
    };
    let pane_id = shell_ui_mut(runtime)?.shift_active_pane(delta);
    let Some(pane_id) = pane_id else {
        return Ok(());
    };
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_pane(workspace_id, pane_id)
        .map_err(|error| error.to_string())?;
    let window_id = active_window_id(runtime)?;
    let buffer_id = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .pane(pane_id)
        .and_then(|pane| pane.active_buffer());
    let mut event = HookEvent::new()
        .with_window(window_id)
        .with_workspace(workspace_id)
        .with_pane(pane_id);
    if let Some(buffer_id) = buffer_id {
        event = event.with_buffer(buffer_id);
    }
    runtime
        .emit_hook(builtins::PANE_SWITCH, event)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn active_runtime_buffer(
    runtime: &EditorRuntime,
) -> Result<Option<(PaneId, BufferId, String, BufferKind)>, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let Some(pane_id) = workspace.active_pane_id() else {
        return Ok(None);
    };
    let pane = workspace
        .pane(pane_id)
        .ok_or_else(|| format!("pane `{pane_id}` is missing"))?;
    let Some(buffer_id) = pane.active_buffer() else {
        return Ok(None);
    };
    let buffer = workspace
        .buffer(buffer_id)
        .ok_or_else(|| format!("runtime buffer `{buffer_id}` is missing"))?;
    Ok(Some((
        pane_id,
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
    {
        let ui = shell_ui_mut(runtime)?;
        for buffer_id in buffer_ids {
            if let Some(buffer) = ui.buffer_mut(buffer_id) {
                buffer.force_syntax_refresh();
            }
        }
    }
    refresh_pending_syntax(runtime).map(|_| ())
}

fn queue_buffer_syntax_refresh(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    {
        let ui = shell_ui_mut(runtime)?;
        let Some(buffer) = ui.buffer_mut(buffer_id) else {
            return Ok(());
        };
        buffer.force_syntax_refresh();
    }
    refresh_pending_syntax(runtime).map(|_| ())
}

fn refresh_buffer_syntax(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let (path, text, buffer_language_id, syntax_window) = {
        let Some(buffer) = shell_ui(runtime)?.buffer(buffer_id) else {
            return Ok(());
        };
        (
            buffer.path().map(|path| path.to_path_buf()),
            buffer.text.clone(),
            buffer
                .language_id()
                .map(|language_id| language_id.to_owned()),
            buffer.desired_syntax_window(),
        )
    };

    let mut parse_session = None;
    let (language_id, syntax_result) = compute_buffer_syntax(
        syntax_registry_mut(runtime)?,
        path.as_deref(),
        &text,
        buffer_language_id.as_deref(),
        syntax_window,
        &mut parse_session,
    );

    let ui = shell_ui_mut(runtime)?;
    if let Some(buffer) = ui.buffer_mut(buffer_id) {
        match syntax_result {
            Some(Ok(snapshot)) => {
                buffer.set_language_id(language_id.clone());
                buffer.set_indexed_syntax_lines(Some(index_syntax_lines(snapshot)), syntax_window);
                buffer.set_syntax_error(None);
            }
            Some(Err(error)) => {
                let error_label = path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .or(language_id.clone())
                    .unwrap_or_else(|| "buffer".to_owned());
                eprintln!("tree-sitter syntax refresh failed for `{error_label}`: {error}");
                buffer.set_language_id(language_id.clone());
                buffer.set_syntax_snapshot(None);
                buffer.set_syntax_error(Some(error.to_string()));
            }
            None => {
                buffer.set_syntax_snapshot(None);
                buffer.set_syntax_error(None);
                buffer.set_language_id(None);
            }
        }
    }

    Ok(())
}

fn compute_buffer_syntax(
    registry: &mut SyntaxRegistry,
    path: Option<&Path>,
    text: &TextBuffer,
    buffer_language_id: Option<&str>,
    syntax_window: Option<SyntaxLineWindow>,
    parse_session: &mut Option<SyntaxParseSession>,
) -> (Option<String>, Option<Result<SyntaxSnapshot, SyntaxError>>) {
    let highlight_window = syntax_window.map(SyntaxLineWindow::to_highlight_window);
    let language_id = path
        .and_then(|path| {
            registry
                .language_for_path(path)
                .map(|language| language.id().to_owned())
        })
        .or_else(|| buffer_language_id.map(str::to_owned));
    let Some(language_id) = language_id else {
        *parse_session = None;
        return (None, None);
    };
    let syntax_result = match match highlight_window {
        Some(window) => registry.highlight_buffer_for_language_window_with_session(
            &language_id,
            text,
            window,
            parse_session,
        ),
        None => {
            registry.highlight_buffer_for_language_with_session(&language_id, text, parse_session)
        }
    } {
        Ok(snapshot) => Ok(snapshot),
        Err(SyntaxError::GrammarNotInstalled {
            language_id: missing_language_id,
            ..
        }) => {
            if let Err(error) = registry.install_language(&missing_language_id) {
                Err(error)
            } else {
                match highlight_window {
                    Some(window) => registry.highlight_buffer_for_language_window_with_session(
                        &language_id,
                        text,
                        window,
                        parse_session,
                    ),
                    None => registry.highlight_buffer_for_language_with_session(
                        &language_id,
                        text,
                        parse_session,
                    ),
                }
            }
        }
        Err(error) => Err(error),
    };
    (Some(language_id), Some(syntax_result))
}

fn configure_syntax_refresh_worker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let (install_root, configs) = {
        let registry = runtime
            .services()
            .get::<SyntaxRegistry>()
            .ok_or_else(|| "syntax registry service missing".to_owned())?;
        (
            registry.install_root().to_path_buf(),
            registry.languages().cloned().collect::<Vec<_>>(),
        )
    };
    shell_ui_mut(runtime)?.configure_syntax_refresh_worker(configs, install_root);
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

fn image_format_for_path(path: &Path) -> Option<ImageBufferFormat> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "svg" => Some(ImageBufferFormat::Svg),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "bmp" | "tif" | "tiff" => {
            Some(ImageBufferFormat::Raster)
        }
        _ => None,
    }
}

fn open_image_workspace_file(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    display_name: &str,
    path: &Path,
    format: ImageBufferFormat,
) -> Result<BufferId, String> {
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            display_name,
            BufferKind::Image,
            Some(path.to_path_buf()),
        )
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("new image buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let shell_buffer = match format {
        ImageBufferFormat::Raster => {
            let decoded = decode_raster_image_path(path)?;
            let mut text = TextBuffer::new();
            text.set_path(path.to_path_buf());
            let mut shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
            shell_buffer.set_image_state(ImageBufferState {
                format,
                mode: ImageBufferMode::Rendered,
                decoded,
                zoom: 1.0,
            });
            shell_buffer
        }
        ImageBufferFormat::Svg => {
            let text = TextBuffer::load_from_path(path)
                .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
            let decoded = rasterize_svg_text(&text.text(), Some(path))?;
            let mut shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
            shell_buffer.set_image_state(ImageBufferState {
                format,
                mode: ImageBufferMode::Rendered,
                decoded,
                zoom: 1.0,
            });
            shell_buffer.set_language_id(language_id_for_path(runtime, path).ok());
            shell_buffer
        }
    };

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

    if format == ImageBufferFormat::Svg {
        queue_buffer_syntax_refresh(runtime, buffer_id)?;
    }

    Ok(buffer_id)
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
        return Ok(existing);
    }

    let workspace_root = active_workspace_root(runtime)?;
    let display_name = workspace_relative_path(workspace_root.as_deref(), path);
    if let Some(format) = image_format_for_path(path) {
        return open_image_workspace_file(
            runtime,
            workspace_id,
            display_name.as_str(),
            path,
            format,
        );
    }
    if is_pdf_path(path) {
        return open_pdf_workspace_file(runtime, workspace_id, display_name.as_str(), path);
    }
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
    let user_library = shell_user_library(runtime);
    let shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);

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
    queue_buffer_syntax_refresh(runtime, buffer_id)?;

    Ok(buffer_id)
}

fn open_workspace_file_at(
    runtime: &mut EditorRuntime,
    path: &Path,
    target: TextPoint,
) -> Result<(), String> {
    let buffer_id = open_workspace_file(runtime, path)?;
    if let Some(buffer) = shell_ui_mut(runtime)?.buffer_mut(buffer_id) {
        buffer.set_cursor(target);
    }
    Ok(())
}

fn create_workspace_file_from_query(
    runtime: &mut EditorRuntime,
    root: &Path,
    query: &str,
) -> Result<(), String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let input_path = PathBuf::from(trimmed);
    if input_path.is_absolute() {
        return Err(format!(
            "workspace file path must be relative: {}",
            input_path.display()
        ));
    }

    if input_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("workspace file path must not contain `..`".to_owned());
    }

    if input_path.file_name().is_none() {
        return Err("workspace file path must include a file name".to_owned());
    }

    let path = root.join(input_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
    }

    if !path.exists() {
        fs::File::create(&path)
            .map_err(|error| format!("failed to create `{}`: {error}", path.display()))?;
    }

    open_workspace_file(runtime, &path)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn workspace_switch_picker_overlay(
    runtime: &EditorRuntime,
) -> Result<PickerOverlay, String> {
    picker::workspace_switch_picker_overlay(runtime)
}

#[cfg(test)]
pub(crate) fn workspace_delete_picker_overlay(
    runtime: &EditorRuntime,
) -> Result<PickerOverlay, String> {
    picker::workspace_delete_picker_overlay(runtime)
}

fn apply_undo_tree_node(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    node_id: usize,
) -> Result<(), String> {
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    if buffer.undo_tree_select(node_id) {
        buffer.mark_syntax_dirty();
        Ok(())
    } else {
        Err("undo tree node is missing".to_owned())
    }
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
            Keycode::Equals | Keycode::KpPlus => Some("Ctrl+=".to_owned()),
            Keycode::Minus | Keycode::KpMinus => Some("Ctrl+-".to_owned()),
            Keycode::_0 | Keycode::Kp0 => Some("Ctrl+0".to_owned()),
            Keycode::B => Some("Ctrl+b".to_owned()),
            Keycode::C => Some("Ctrl+c".to_owned()),
            Keycode::D => Some("Ctrl+d".to_owned()),
            Keycode::E => Some("Ctrl+e".to_owned()),
            Keycode::F => Some("Ctrl+f".to_owned()),
            Keycode::H => Some("Ctrl+h".to_owned()),
            Keycode::J => Some("Ctrl+j".to_owned()),
            Keycode::K => Some("Ctrl+k".to_owned()),
            Keycode::L => Some("Ctrl+l".to_owned()),
            Keycode::N => Some("Ctrl+n".to_owned()),
            Keycode::P => Some("Ctrl+p".to_owned()),
            Keycode::R => Some("Ctrl+r".to_owned()),
            Keycode::S => Some("Ctrl+s".to_owned()),
            Keycode::T => Some("Ctrl+t".to_owned()),
            Keycode::U => Some("Ctrl+u".to_owned()),
            Keycode::V => Some("Ctrl+v".to_owned()),
            Keycode::Y => Some("Ctrl+y".to_owned()),
            Keycode::Space => Some("Ctrl+Space".to_owned()),
            Keycode::Grave => Some("Ctrl+`".to_owned()),
            Keycode::Period | Keycode::KpPeriod => Some("Ctrl+.".to_owned()),
            Keycode::Return | Keycode::KpEnter => Some("Ctrl+Enter".to_owned()),
            Keycode::Tab => Some("Ctrl+Tab".to_owned()),
            _ => None,
        };
    }

    if keymod.intersects(alt_mod()) && !keymod.intersects(ctrl_mod() | gui_mod()) {
        return match keycode {
            Keycode::X => Some("Alt+x".to_owned()),
            _ => None,
        };
    }

    if keymod.intersects(shift_mod()) && matches!(keycode, Keycode::Tab) {
        return Some("Shift+Tab".to_owned());
    }

    match keycode {
        Keycode::F3 => Some("F3".to_owned()),
        Keycode::F4 => Some("F4".to_owned()),
        Keycode::F5 => Some("F5".to_owned()),
        Keycode::F6 => Some("F6".to_owned()),
        Keycode::Tab => Some("Tab".to_owned()),
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

fn normalize_text_token(chord: &str) -> String {
    if chord == " " {
        "Space".to_owned()
    } else {
        chord.to_owned()
    }
}

fn ctrl_mod() -> Mod {
    Mod::LCTRLMOD | Mod::RCTRLMOD
}

fn shift_mod() -> Mod {
    Mod::LSHIFTMOD | Mod::RSHIFTMOD
}

fn alt_mod() -> Mod {
    Mod::LALTMOD | Mod::RALTMOD
}

fn gui_mod() -> Mod {
    Mod::LGUIMOD | Mod::RGUIMOD
}

fn browser_devtools_shortcut_requested(keycode: Keycode, keymod: Mod) -> bool {
    if !keymod.intersects(alt_mod() | gui_mod()) && keycode == Keycode::F12 {
        return true;
    }
    keycode == Keycode::I
        && keymod.intersects(ctrl_mod())
        && keymod.intersects(shift_mod())
        && !keymod.intersects(alt_mod() | gui_mod())
}

fn keymap_vim_mode(input_mode: InputMode) -> KeymapVimMode {
    match input_mode {
        InputMode::Normal => KeymapVimMode::Normal,
        InputMode::Insert | InputMode::Replace => KeymapVimMode::Insert,
        InputMode::Visual => KeymapVimMode::Visual,
    }
}

fn plugin_vim_mode_matches(
    binding_mode: editor_plugin_api::PluginVimMode,
    active_mode: KeymapVimMode,
) -> bool {
    match binding_mode {
        editor_plugin_api::PluginVimMode::Any => true,
        editor_plugin_api::PluginVimMode::Normal => active_mode == KeymapVimMode::Normal,
        editor_plugin_api::PluginVimMode::Insert => active_mode == KeymapVimMode::Insert,
        editor_plugin_api::PluginVimMode::Visual => active_mode == KeymapVimMode::Visual,
    }
}

fn default_volt_state_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        let base = env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        base.join("volt")
    } else {
        let base = env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("state"))
            })
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        base.join("volt")
    }
}

fn default_error_log_path() -> PathBuf {
    default_volt_state_dir().join(ERROR_LOG_FILE_NAME)
}

fn active_theme_state_path() -> PathBuf {
    default_volt_state_dir().join(ACTIVE_THEME_STATE_FILE_NAME)
}

fn default_typing_profile_log_path() -> PathBuf {
    default_volt_state_dir().join(TYPING_PROFILE_LOG_FILE_NAME)
}

fn read_saved_theme_selection(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let theme_id = contents.trim();
            if theme_id.is_empty() {
                Ok(None)
            } else {
                Ok(Some(theme_id.to_owned()))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "failed to read saved theme from `{}`: {error}",
            path.display()
        )),
    }
}

fn write_saved_theme_selection(path: &Path, theme_id: &str) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err(format!(
            "saved theme path `{}` does not have a parent directory",
            path.display()
        ));
    };
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create theme state directory `{}`: {error}",
            parent.display()
        )
    })?;
    fs::write(path, format!("{theme_id}\n")).map_err(|error| {
        format!(
            "failed to write saved theme to `{}`: {error}",
            path.display()
        )
    })
}

fn clear_saved_theme_selection(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to clear saved theme `{}`: {error}",
            path.display()
        )),
    }
}

fn restore_saved_theme_selection(
    theme_registry: &mut ThemeRegistry,
    path: &Path,
) -> Result<(), String> {
    let Some(theme_id) = read_saved_theme_selection(path)? else {
        return Ok(());
    };
    if let Err(error) = theme_registry.activate(&theme_id) {
        clear_saved_theme_selection(path)?;
        return Err(format!(
            "saved theme `{theme_id}` is no longer available and was removed from `{}`: {error}",
            path.display()
        ));
    }
    Ok(())
}

fn ensure_log_directory(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create log directory `{}`: {error}",
            parent.display()
        )
    })
}

fn install_panic_hook(log_file_path: PathBuf) {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let entry = ErrorEntry::new(ErrorSeverity::Error, "panic", info.to_string());
        if let Err(error) = append_error_log(&log_file_path, &entry) {
            eprintln!("Failed to write panic log: {error}");
        }
        default_hook(info);
    }));
}

fn panic_payload_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic payload is not a string".to_owned()
    }
}

fn format_timestamp(timestamp: SystemTime) -> String {
    match timestamp.duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}.{:03}", duration.as_secs(), duration.subsec_millis()),
        Err(_) => "0.000".to_owned(),
    }
}

fn format_duration_ms(duration: Duration) -> String {
    let micros = duration.as_micros();
    format!("{}.{:03}ms", micros / 1_000, micros % 1_000)
}

fn average_duration(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    let total_micros = durations
        .iter()
        .map(|duration| duration.as_micros())
        .sum::<u128>();
    let average_micros = total_micros / durations.len() as u128;
    let seconds = average_micros / 1_000_000;
    let nanos = ((average_micros % 1_000_000) * 1_000) as u32;
    Duration::new(seconds.min(u64::MAX as u128) as u64, nanos)
}

fn percentile_duration(durations: &[Duration], percentile: usize) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    let mut sorted = durations.to_vec();
    sorted.sort();
    let clamped = percentile.min(100);
    let index = ((sorted.len().saturating_sub(1)) * clamped) / 100;
    sorted[index]
}

fn format_typing_frame_profile(frame: &TypingFrameProfile) -> String {
    let timestamp = format_timestamp(frame.timestamp);
    let preview = if frame.text_preview.is_empty() {
        "<none>".to_owned()
    } else {
        frame.text_preview.clone()
    };
    let first_to_present = frame
        .first_text_to_present
        .map(format_duration_ms)
        .unwrap_or_else(|| "-".to_owned());
    let last_to_present = frame
        .last_text_to_present
        .map(format_duration_ms)
        .unwrap_or_else(|| "-".to_owned());
    format!(
        "[{timestamp}] frame={} pacing_sleep={} events={} keydowns={} text_inputs={} preview=\"{}\" handle={} keydown_handle={} text_handle={} text_inner={} picker={} syntax_apply={} syntax_worker={} syntax_results={} syntax_spans={} git={} acp={} render={} present={} total={} first_text_to_present={} last_text_to_present={}",
        frame.frame_index,
        format_duration_ms(frame.frame_pacing_sleep),
        frame.polled_events,
        frame.keydown_events,
        frame.text_input_events,
        preview,
        format_duration_ms(frame.handle_event_total),
        format_duration_ms(frame.keydown_handle_total),
        format_duration_ms(frame.text_input_handle_total),
        format_duration_ms(frame.text_input_inner_total),
        format_duration_ms(frame.picker_refresh),
        format_duration_ms(frame.syntax_refresh),
        format_duration_ms(frame.syntax_worker_compute),
        frame.syntax_result_count,
        frame.syntax_highlight_spans,
        format_duration_ms(frame.git_refresh),
        format_duration_ms(frame.acp_refresh),
        format_duration_ms(frame.render),
        format_duration_ms(frame.present),
        format_duration_ms(frame.frame_total),
        first_to_present,
        last_to_present,
    )
}

fn sanitize_typing_preview(text: &str) -> String {
    let mut sanitized = String::new();
    for character in text.chars() {
        match character {
            '\n' => sanitized.push_str("\\n"),
            '\r' => sanitized.push_str("\\r"),
            '\t' => sanitized.push_str("\\t"),
            other => sanitized.push(other),
        }
    }
    sanitized
}

fn format_error_entry_lines(entry: &ErrorEntry) -> Vec<String> {
    let timestamp = format_timestamp(entry.timestamp);
    let mut lines = Vec::new();
    let mut message_lines = entry.message.lines();
    if let Some(first) = message_lines.next() {
        lines.push(format!(
            "[{timestamp}] {} {}: {first}",
            entry.severity.label(),
            entry.source
        ));
        for line in message_lines {
            lines.push(format!("    {line}"));
        }
    } else {
        lines.push(format!(
            "[{timestamp}] {} {}: <empty>",
            entry.severity.label(),
            entry.source
        ));
    }
    lines
}

fn append_error_log(path: &Path, entry: &ErrorEntry) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to open log `{}`: {error}", path.display()))?;
    for line in format_error_entry_lines(entry) {
        writeln!(file, "{line}")
            .map_err(|error| format!("failed to write log `{}`: {error}", path.display()))?;
    }
    Ok(())
}

fn errors_buffer_lines(entries: &[ErrorEntry], log_path: &Path) -> Vec<String> {
    let mut lines = initial_errors_lines(Some(log_path));
    if entries.is_empty() {
        return lines;
    }
    lines.push(String::new());
    lines.push(format!("Recent errors ({})", entries.len()));
    for entry in entries {
        lines.extend(format_error_entry_lines(entry));
    }
    lines
}

fn record_runtime_error(runtime: &mut EditorRuntime, source: &str, message: impl Into<String>) {
    let entry = ErrorEntry::new(ErrorSeverity::Error, source, message);
    let (buffer_id, lines) = {
        let Some(log) = runtime.services_mut().get_mut::<ErrorLog>() else {
            eprintln!("Error log service missing for: {}.", entry.message);
            return;
        };
        let lines = log.record(entry);
        (log.buffer_id, lines)
    };
    if let Err(error) = update_error_buffer(runtime, buffer_id, lines) {
        eprintln!("Failed to update errors buffer: {error}");
    }
}

fn update_error_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    lines: Vec<String>,
) -> Result<(), String> {
    let user_library = shell_user_library(runtime);
    let ui = shell_ui_mut(runtime)?;
    let buffer = ui.ensure_buffer(
        buffer_id,
        "*errors*",
        BufferKind::Diagnostics,
        &*user_library,
    );
    buffer.replace_with_lines(lines);
    Ok(())
}

fn format_lsp_log_entry_lines(entry: &LspLogEntry) -> Vec<String> {
    let timestamp = format_timestamp(entry.timestamp());
    let mut body_lines = entry
        .body()
        .lines()
        .map(str::trim_end)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let prefix = format!(
        "[{timestamp}] {} {}",
        entry.direction().label(),
        entry.server_id()
    );
    let mut lines = Vec::with_capacity(body_lines.len().saturating_add(2));
    if let Some(first) = body_lines.first() {
        lines.push(format!("{prefix}: {first}"));
        for line in body_lines.drain(1..) {
            lines.push(format!("    {line}"));
        }
    } else {
        lines.push(format!("{prefix}: <empty>"));
    }
    lines.push(String::new());
    lines
}

fn lsp_log_entries_for_server(entries: &[LspLogEntry], server_id: &str) -> Vec<LspLogEntry> {
    entries
        .iter()
        .filter(|entry| entry.server_id() == server_id)
        .cloned()
        .collect()
}

fn lsp_log_buffer_lines(server_id: &str, entries: &[LspLogEntry]) -> Vec<String> {
    let mut lines = initial_lsp_log_lines(server_id);
    if entries.is_empty() {
        return lines;
    }
    lines.push(String::new());
    lines.push(format!("Recent transport entries ({})", entries.len()));
    for entry in entries {
        lines.extend(format_lsp_log_entry_lines(entry));
    }
    lines
}

fn build_shell_summary(
    state: &mut ShellState,
    frames_rendered: u32,
    renderer_name: String,
    font_path: &Path,
) -> ShellSummary {
    let typing_profile = match state.finish_typing_profile() {
        Ok(profile) => profile,
        Err(error) => {
            state.record_error("typing-profile", error);
            None
        }
    };
    let pane_count = match state.pane_count() {
        Ok(count) => count,
        Err(error) => {
            state.record_shell_error("shell.summary.pane-count", error);
            0
        }
    };
    let popup_visible = match state.popup_visible() {
        Ok(visible) => visible,
        Err(error) => {
            state.record_shell_error("shell.summary.popup-visible", error);
            false
        }
    };
    ShellSummary {
        frames_rendered,
        pane_count,
        popup_visible,
        render_backend: RenderBackend::SdlCanvas,
        renderer_name,
        font_path: font_path.display().to_string(),
        typing_profile,
    }
}

fn initial_errors_lines(log_path: Option<&Path>) -> Vec<String> {
    let mut lines = vec![
        "*errors* captures runtime failures and panics.".to_owned(),
        "The shell continues running while logging errors here.".to_owned(),
        "Open the buffer picker (F4) to revisit this buffer.".to_owned(),
    ];
    if let Some(path) = log_path {
        lines.push(format!("Log file: {}", path.display()));
    } else {
        lines.push("Log file: <pending>".to_owned());
    }
    lines
}

fn initial_lsp_log_lines(server_id: &str) -> Vec<String> {
    vec![
        format!(
            "{} captures live JSON-RPC traffic for `{server_id}`.",
            lsp_log_buffer_name(server_id)
        ),
        "Run `lsp.log` from a buffer using that server, or open the buffer picker (F4) to focus this buffer.".to_owned(),
        "Requests, notifications, responses, and disconnect events are appended here.".to_owned(),
    ]
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

fn buffer_interaction(
    kind: &BufferKind,
    _user_library: &dyn UserLibrary,
) -> (bool, Option<InputField>) {
    match kind {
        BufferKind::Image => (false, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_READONLY_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_INPUT_KIND => {
            (true, Some(InputField::new("Ask > ")))
        }
        BufferKind::Plugin(plugin_kind) if plugin_kind == BROWSER_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == PDF_BUFFER_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STATUS_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_DIFF_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_LOG_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STASH_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_COMMIT_KIND => (false, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == OIL_PREVIEW_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == OIL_HELP_KIND => (true, None),
        BufferKind::Terminal => (true, None),
        BufferKind::Directory => (false, None),
        BufferKind::Compilation => {
            let mut input = InputField::new("$ ");
            input.set_placeholder(Some(
                "Enter build command (e.g. cargo build) then press Ctrl+Enter".to_owned(),
            ));
            (true, Some(input))
        }
        _ => (false, None),
    }
}

fn plugin_section_state_for_kind(
    kind: &BufferKind,
    user_library: &dyn UserLibrary,
) -> Option<PluginSectionBufferState> {
    let BufferKind::Plugin(plugin_kind) = kind else {
        return None;
    };
    let buffer = user_library.plugin_buffer(plugin_kind)?;
    let sections = buffer.sections()?.clone();
    PluginSectionBufferState::new(sections, buffer.evaluate_target_section())
}

fn placeholder_lines(name: &str, kind: &BufferKind, user_library: &dyn UserLibrary) -> Vec<String> {
    match name {
        "*scratch*" => initial_scratch_lines(),
        "*notes*" => initial_notes_lines(),
        "*errors*" => initial_errors_lines(None),
        _ => match kind {
            BufferKind::Image => vec![
                format!("{name} is a native image buffer."),
                "Supported image files open directly into a centered preview.".to_owned(),
            ],
            BufferKind::Scratch => vec![
                format!("{name} is a scratch buffer created by the runtime."),
                "This buffer can be focused from the generic buffer picker.".to_owned(),
            ],
            BufferKind::Picker => vec![
                format!("{name} is a picker-backed buffer."),
                "The SDL shell renders picker state through the popup search UI.".to_owned(),
            ],
            BufferKind::Terminal => vec![
                format!("{name} is launching the configured shell."),
                "Press i to enter terminal input mode, or stay in Normal mode to navigate scrollback."
                    .to_owned(),
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
            BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_READONLY_KIND => vec![
                format!("{name} is an interactive read-only buffer."),
                "Keybindings still run, but edits are blocked.".to_owned(),
                "Use this as a starting point for magit-style interfaces.".to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_INPUT_KIND => vec![
                format!("{name} is an interactive input buffer."),
                "Type into the prompt to submit commands or text.".to_owned(),
                "Use Ctrl+Enter to submit or Ctrl+l to clear.".to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == BROWSER_KIND => {
                user_library.browser_buffer_lines(None)
            }
            BufferKind::Plugin(plugin_kind) if plugin_kind == PDF_BUFFER_KIND => vec![
                format!("{name} is a native PDF buffer."),
                "Open a .pdf file to inspect its metadata, page text, and structural state."
                    .to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND => vec![
                format!("{name} is an ACP session buffer."),
                "Use acp.pick-client to start an ACP agent.".to_owned(),
                "Type into the prompt and press Ctrl+Enter to send.".to_owned(),
                "Use / for slash commands, Ctrl+Space/Tab for completion, Shift+Tab to cycle modes, Ctrl+Tab to switch ACP panes, acp.pick-mode to choose a mode, acp.pick-model to choose a model, and Ctrl+j for a newline."
                    .to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STATUS_KIND => Vec::new(),
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_DIFF_KIND => Vec::new(),
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_LOG_KIND => Vec::new(),
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STASH_KIND => Vec::new(),
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_COMMIT_KIND => {
                user_library.git_commit_template()
            }
            BufferKind::File => vec![
                format!("{name} is a file-backed buffer placeholder."),
                "File loading is not yet wired into the SDL shell event loop.".to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) => {
                // Ask the user library for initial content.  If none is provided,
                // fall back to the generic plugin placeholder message.
                let initial = user_library.plugin_buffer_initial_lines(plugin_kind);
                if initial.is_empty() {
                    vec![
                        format!("{name} was opened for plugin kind `{plugin_kind}`."),
                        "Users can change this behavior by editing the matching user package and recompiling.".to_owned(),
                    ]
                } else {
                    initial
                }
            }
        },
    }
}

fn buffer_kind_label(kind: &BufferKind) -> String {
    match kind {
        BufferKind::File => "file".to_owned(),
        BufferKind::Image => "image".to_owned(),
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

fn popup_window_height(content_height: u32, line_height: i32) -> u32 {
    let row_height = line_height.max(1) as u32;
    if content_height <= row_height {
        return content_height;
    }

    let desired = (content_height.saturating_mul(2) / 5).max(row_height * 4);
    let max_height = content_height.saturating_sub(row_height).max(row_height);
    let clamped = desired.min(max_height);
    (clamped / row_height).max(1) * row_height
}

fn pixel_rect_contains_point(rect: PixelRect, x: i32, y: i32) -> bool {
    let right = rect.x.saturating_add(rect.width as i32);
    let bottom = rect.y.saturating_add(rect.height as i32);
    x >= rect.x && x < right && y >= rect.y && y < bottom
}
