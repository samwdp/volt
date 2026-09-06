mod acp;
mod acp_dock;
mod autocomplete_tokens;
mod browser;
mod clipboard;
mod command_line;
mod command_stream;
mod dap;
mod db;
mod diagnostics;
mod directory;
mod draw;
mod git;
mod git_editor;
mod idle;
mod issues;
mod markdown_pretty;
mod pdf;
mod picker;
mod render;
mod terminal;
mod tool_install;
mod treesitter_install;
mod ui_overlays;
mod workspace_dock;
mod workspace_search;

use acp_dock::*;
use autocomplete_tokens::{AutocompleteTokenCache, is_completion_word_char};
use browser::*;
use command_line::*;
use command_stream::*;
use dap::*;
use db::*;
use diagnostics::*;
use directory::*;
use draw::*;
use git::*;
use git_editor::*;
use idle::*;
use issues::*;
use pdf::*;
use render::*;
use terminal::*;
use workspace_dock::*;
use workspace_search::*;

pub use idle::{IDLE_WAIT_CAP, idle_wait_timeout};

#[cfg(test)]
mod tests;

use std::{
    any::Any,
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    env, fs,
    io::{self, Write},
    panic::{self, AssertUnwindSafe},
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Sender},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use abi_stable::library::RootModule;
use agent_client_protocol::{
    ContentBlock, Diff, MaybeUndefined, Plan, PlanEntry, PlanEntryPriority, PlanEntryStatus,
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
    BlockInsertState, BlockSelection, DirectoryYankEntry, FormatterRegistry, FormatterSpec,
    InputMode, LastFind, LastSearch, MulticursorState, ScrollCommand, ShellMotion, VimBufferState,
    VimFindKind, VimMark, VimOperator, VimPending, VimRecordedInput, VimSearchDirection, VimState,
    VimTarget, VimTextObjectKind, VimVisualSnapshot, VisualSelection, VisualSelectionKind,
    YankFlash, YankRegister,
};
use crate::window_effects::{
    WindowEffects, apply_window_effects, configure_window_opacity_driver,
    current_window_effect_settings, update_window_effects, window_creation_flags,
};
use editor_buffer::{TextBuffer, TextEdit, TextPoint, TextRange, TextSnapshot, WordKind};
use editor_core::{
    Buffer, BufferId, BufferKind, CommandSource, CycleDirection, EditorRuntime, HookEvent,
    KeySequenceOptions, KeySequencePush, KeySequenceTick, KeymapScope, KeymapVimMode, MarkList,
    MarkedWorkspaceJump, PaneId, PendingKeySequence, SectionAction, SectionCollapseState,
    SectionRenderLine, SectionRenderLineKind, WorkspaceId, builtins, cycle_project_workspace,
    marked_workspace_jump, normalize_project_root_path, plan_worktree_remove, project_roots_equal,
    push_key_sequence, tick_key_sequence,
};
use editor_dap::{
    BreakpointState, DapClientError, DapClientManager, DapEvaluateContext, DapLogDirection,
    DapLogSnapshot, DapSessionEvent, DebugAdapterRegistry, DebugConfiguration,
    DebugConfigurationCandidate, DebugInferContext, DebugRequestKind,
    collect_configuration_candidates, configuration_holes, infer_compile_heuristic,
};
use editor_db::{
    DbActionOutcome, DbAutocompleteCandidate, DbBrowserBufferView, DbExecutionOutput, DbService,
    DbSessionId, QualifiedName, parse_db_connect_prompt, split_sql_statements, sql_scope_from_text,
};
use editor_fs::{
    DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, ProjectSearchRoot,
    project_discovery_background_tick, project_discovery_forget_candidate,
    project_discovery_rescan_cached_roots, project_discovery_snapshot,
};
use editor_git::{
    GitLogEntry, GitStatusSnapshot, detect_in_progress, git_probe_snapshot,
    git_probe_snapshot_with_numstat, invalidate_git_probe_cache_for,
    invalidate_repository_file_list_cache_for, list_repository_files, parse_log_oneline,
    parse_stash_list, parse_status, repository_file_preview,
};
use editor_jobs::{JobManager, JobSpec};
use editor_lsp::{
    Diagnostic as LspDiagnostic, DiagnosticSeverity as LspDiagnosticSeverity,
    LanguageServerRegistry, LspClientError, LspClientManager, LspCodeAction, LspFormattingOptions,
    LspInlineCompletionItem, LspLiveSession, LspLocation, LspLogEntry, LspLogSnapshot,
    LspNotificationAction, LspNotificationLevel, LspNotificationSnapshot, LspTextEdit,
    LspWorkspaceDiagnostic,
};
use editor_picker::{
    PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind, PickerItem, PickerOneShotContext,
    PickerResultOrder, PickerSelectedRow, PickerSession, resolve_picker_extra,
};
use editor_plugin_api::{
    AcpActionSpec, AcpPickerContext, AcpPickerItemSpec, AcpPickerKind, AcpPickerOption,
    DbBrowserKind, GhostTextContext as HostGhostTextContext,
    LspDiagnosticsInfo as PluginLspDiagnosticsInfo, ModelineAlignment, ModelineSegment,
    OilDefaults, OilKeyAction, PdfOpenMode, PickerAcpClientContext, PickerActionSpec,
    PickerBufferContext, PickerCommandContext, PickerIconContext, PickerKeybindingContext,
    PickerProviderContext, PickerProviderSpec, PickerSource, PickerSyntaxLanguageContext,
    PickerThemeContext, PickerTruncateStrategy, PickerUndoTreeContext, PickerWorkspaceContext,
    PluginBufferLayout, PluginBufferLayoutAxis, PluginBufferLayoutNode, PluginBufferSectionUpdate,
    PluginBufferSections, StatuslineSpan, VimEditAction, WorkspaceDockSide,
    abi::{
        AbiDirectoryEntry, AbiGhostTextContext, AbiGitStatusPrefix, AbiStatuslineContext,
        UserLibraryModuleRef,
    },
    autocomplete_hooks, browser_hooks, buffer_kinds, dap_hooks, db_hooks, decode_modeline,
    flatten_modeline_text, flatten_modeline_to_spans, git_actions, git_hooks, git_sections,
    hover_hooks, image_hooks, input_hooks, lsp_hooks, oil_hooks, oil_protocol, pdf_hooks,
    plugin_hooks, terminal_hooks,
};
use editor_plugin_host::{
    NullUserLibrary, StatuslineContext as HostStatuslineContext, UserLibrary,
    load_auto_loaded_packages, reload_user_packages,
};
use editor_render::{
    DrawCommand, PixelRect, RenderBackend, RenderColor, SplitAxis, SplitChild, SplitNode,
    TextStyle, centered_rect, find_font_by_name, find_system_monospace_font,
    horizontal_pane_rects_for_active, layout_split_tree, pane_rects_with_weights,
    vertical_pane_rects_for_active,
};
use editor_syntax::{
    HighlightWindow, LanguageConfiguration, SyntaxError, SyntaxParseSession, SyntaxRegistry,
    SyntaxSnapshot, apply_rainbow_delimiter_spans_for_buffer,
};
use editor_terminal::{
    LiveTerminalConfig, LiveTerminalSession, TerminalKey, TerminalRenderSnapshot,
    TerminalViewportScroll,
};
use editor_theme::{Color as ThemeColor, ThemeRegistry, ThemeStyle};
use editor_ui::{
    OverlayCard, PanelFrame, ScrollbarThumb, paint_left_accent,
    paint_overlay_card as paint_ui_overlay_card, paint_panel_frame, paint_right_band,
    paint_scrollbar_thumb, paint_selection_highlight, paint_top_header_band,
};
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
    ttf::{Font, FontStyle, Hinting},
    video::{Window, WindowContext},
};
use sdl3_ttf_sys as _;

include!("layout.rs");
include!("constants.rs");
include!("user_library.rs");
include!("draw_target.rs");
include!("theme_settings.rs");
include!("fonts.rs");
include!("text_layout.rs");
include!("markdown_table.rs");
include!("wrap.rs");
include!("undo.rs");
include!("input_field.rs");
include!("buffer_types.rs");
include!("shell_buffer_construct.rs");
include!("shell_buffer_media.rs");
include!("shell_buffer_acp.rs");
include!("shell_buffer_edit.rs");
include!("acp_view.rs");
include!("ui_state.rs");
include!("telemetry.rs");
include!("shell_state.rs");
include!("shell_state_events.rs");
include!("shell_state_render.rs");
include!("shell_state_vim.rs");
include!("shell_state_input.rs");
include!("shell_state_keys.rs");
include!("shell_state_refresh.rs");
include!("run_loop.rs");
include!("theme_reload.rs");
include!("hooks.rs");
include!("dap_commands.rs");
include!("lsp_commands.rs");
include!("overlay_commands.rs");
include!("buffer_access.rs");
include!("vim_search.rs");
include!("workers_autocomplete.rs");
include!("hover.rs");
include!("workers.rs");
include!("vim_motion.rs");
include!("vim_visual.rs");
include!("comments.rs");
include!("yank.rs");
include!("db_commands.rs");
include!("save.rs");
include!("vim_operators.rs");
include!("vim_ex.rs");
include!("oil.rs");
include!("compile.rs");
include!("refresh.rs");
include!("workspace.rs");
include!("syntax_refresh.rs");
include!("files.rs");
include!("input_chords.rs");
include!("paths_logs.rs");
