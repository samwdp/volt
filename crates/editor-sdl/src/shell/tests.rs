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

#[derive(Debug, Default)]
struct CommandLog(Vec<String>);

fn rust_test_language() -> editor_syntax::Language {
    tree_sitter_rust::LANGUAGE.into()
}

fn register_rust_highlight_test_language(runtime: &mut EditorRuntime) -> Result<(), String> {
    syntax_registry_mut(runtime)?
        .register(editor_syntax::LanguageConfiguration::new(
            "rust",
            ["rs"],
            rust_test_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [
                editor_syntax::CaptureThemeMapping::new("keyword", "syntax.keyword"),
                editor_syntax::CaptureThemeMapping::new("function", "syntax.function"),
            ],
        ))
        .map_err(|error| error.to_string())
}

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        Self {
            path: env::temp_dir().join(format!("volt-shell-{name}-{unique}")),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn wait_for_buffer_syntax_refresh(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_syntax(runtime)?;
        if !shell_buffer(runtime, buffer_id)?.syntax_dirty {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    let worker_configured = shell_ui(runtime)?.syntax_refresh_worker.is_configured();
    let pending_results = shell_ui(runtime)?
        .syntax_refresh_worker
        .results
        .lock()
        .map(|results| results.len())
        .unwrap_or(usize::MAX);
    let buffer = shell_buffer(runtime, buffer_id)?;
    Err(format!(
        "syntax refresh did not complete for buffer `{}` (worker_configured={worker_configured}, pending_results={pending_results}, language_id={:?}, syntax_error={:?})",
        buffer_id.get(),
        buffer.language_id(),
        buffer.syntax_error
    ))
}

include!("tests_01.rs");
include!("tests_02.rs");
include!("tests_03.rs");
include!("tests_04.rs");
include!("tests_05.rs");
include!("tests_06.rs");
include!("tests_07.rs");
include!("tests_08.rs");
include!("tests_09.rs");
include!("tests_10.rs");
include!("tests_11.rs");
include!("tests_12.rs");
include!("tests_13.rs");
include!("tests_14.rs");
include!("tests_15.rs");
include!("tests_16.rs");
include!("tests_17.rs");
include!("tests_18.rs");
include!("tests_19.rs");
