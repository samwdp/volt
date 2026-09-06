use super::autocomplete_tokens::AutocompleteTokenScanKind;
use super::*;
use agent_client_protocol::{
    Diff, Plan, PlanEntry, PlanEntryPriority, PlanEntryStatus, TextContent, ToolCall,
    ToolCallContent, ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields, ToolKind,
};
use editor_lsp::{
    Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LspClientManager, LspLiveSession,
    LspLogDirection,
};
use editor_plugin_api::{
    AcpClient, AutocompleteProvider, DebugAdapterSpec, GhostTextContext, HoverProvider,
    LanguageConfiguration, LanguageServerSpec, LigatureConfig, MarkdownPrettyConfig, OilDefaults,
    OilKeyAction, OilKeybindings, PaneConfig, PdfOpenMode, PluginBuffer, PluginBufferSections,
    RainbowParensConfig, TerminalConfig, Theme, WorkspaceDockConfig, WorkspaceDockSide,
    WorkspaceRoot,
};
use editor_plugin_host::StatuslineContext;
use editor_render::{horizontal_pane_rects, vertical_pane_rects};
use sdl3::mouse::MouseState;
use sdl3::video::WindowFlags;
use std::{
    collections::BTreeMap,
    env, fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

mod helpers;
use helpers::*;

mod acp_protocol;
mod browser_ui;
mod compile_reload;
mod database;
mod debug;
mod draw_text;
mod font_atlas;
mod git_status;
mod hover_ui;
mod idle_pacing;
mod input_shortcuts;
mod keys;
mod line_wrap;
mod markdown_paint;
mod misc;
mod oil_buffer;
mod picker_ui;
mod popup;
mod render_chrome;
mod render_overlays;
mod render_plugins;
mod search;
mod syntax_highlight;
mod terminal_ui;
mod text_layout;
mod vim;
mod workspace_nav;
