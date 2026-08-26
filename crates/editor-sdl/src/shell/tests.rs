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

#[test]
fn stalled_syntax_request_becomes_due_again() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*stalled-syntax-request*",
        vec!["fn main() {}".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.set_language_id(Some("rust".to_owned()));
    buffer.force_syntax_refresh();
    buffer.mark_syntax_refresh_requested(buffer.full_syntax_window());
    let requested_at = buffer
        .syntax_requested_at
        .ok_or_else(|| "syntax request timestamp missing".to_owned())?;

    assert!(!buffer.syntax_refresh_due(requested_at));
    assert!(buffer.syntax_refresh_due(
        requested_at + SYNTAX_REFRESH_REQUEST_TIMEOUT + Duration::from_millis(1)
    ));
    Ok(())
}

#[test]
fn disconnected_syntax_worker_restarts_without_stranding_buffers() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*disconnected-syntax-worker*",
        vec!["fn main() {}".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.force_syntax_refresh();
    }
    let (request_tx, request_rx) = mpsc::channel();
    drop(request_rx);
    shell_ui_mut(&mut state.runtime)?
        .syntax_refresh_worker
        .request_tx = Some(request_tx);

    refresh_pending_syntax(&mut state.runtime)?;
    assert!(
        shell_ui(&state.runtime)?
            .syntax_refresh_worker
            .is_configured()
    );
    assert!(
        shell_ui(&state.runtime)?
            .syntax_refresh_worker
            .has_live_worker()
    );
    Ok(())
}

#[test]
fn syntax_refresh_reuses_shared_worker_across_buffers() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first = install_text_test_buffer(
        &mut state,
        "*shared-syntax-worker-a*",
        vec!["fn main() {}".to_owned()],
    )?;
    let second = install_text_test_buffer(
        &mut state,
        "*shared-syntax-worker-b*",
        vec!["fn other() {}".to_owned()],
    )?;
    for buffer_id in [first, second] {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.force_syntax_refresh();
    }

    refresh_pending_syntax(&mut state.runtime)?;
    assert!(
        shell_ui(&state.runtime)?
            .syntax_refresh_worker
            .has_live_worker()
    );
    refresh_pending_syntax(&mut state.runtime)?;
    assert!(
        shell_ui(&state.runtime)?
            .syntax_refresh_worker
            .has_live_worker(),
        "second buffer must reuse the same shared worker"
    );
    Ok(())
}

#[test]
fn preload_languages_returns_without_waiting_for_worker_done() {
    let (request_tx, request_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut worker = SyntaxRefreshWorkerState::disabled();
        worker.configure(Vec::new(), PathBuf::from("."), None);
        worker.request_tx = Some(request_tx);
        let ok = worker.preload_languages(["rust"]);
        let _ = result_tx.send(ok);
    });

    let _queued = request_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("preload should queue a worker message");

    let returned = result_rx.recv_timeout(Duration::from_millis(500));
    assert!(
        returned.is_ok(),
        "preload_languages must return without waiting for the worker to finish loading"
    );
    assert!(
        returned.expect("preload result"),
        "queueing preload must succeed even when the worker has not finished"
    );
}

#[test]
fn preload_languages_still_completes_on_worker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    assert!(
        shell_ui_mut(&mut state.runtime)?
            .syntax_refresh_worker
            .preload_languages(["rust"])
    );
    assert!(
        shell_ui_mut(&mut state.runtime)?
            .syntax_refresh_worker
            .wait_for_pending_preloads(Duration::from_secs(5)),
        "test-only wait hook must observe worker preload completion"
    );
    Ok(())
}

fn sync_active_buffer_layout_for_test(state: &mut ShellState) -> Result<(), String> {
    const RENDER_WIDTH: u32 = 960;
    const RENDER_HEIGHT: u32 = 640;
    const CELL_WIDTH: i32 = 8;
    const LINE_HEIGHT: i32 = 16;

    state
        .sync_active_viewport_for_render_size(RENDER_WIDTH, RENDER_HEIGHT, LINE_HEIGHT)
        .map_err(|error| error.to_string())?;
    state
        .sync_visible_buffer_layouts(RENDER_WIDTH, RENDER_HEIGHT, CELL_WIDTH, LINE_HEIGHT)
        .map_err(|error| error.to_string())
}

struct HeaderlineTestUserLibrary {
    scrolloff: f64,
    headerline_lines: Vec<String>,
    headerline_requires_scrolled_viewport: bool,
    headerline_call_count: Arc<AtomicUsize>,
    pdf_open_mode: PdfOpenMode,
    markdown_pretty: MarkdownPrettyConfig,
}

impl Default for HeaderlineTestUserLibrary {
    fn default() -> Self {
        Self {
            scrolloff: 1.0,
            headerline_lines: vec!["fn render(value: usize)".to_owned()],
            headerline_requires_scrolled_viewport: false,
            headerline_call_count: Arc::new(AtomicUsize::new(0)),
            pdf_open_mode: PdfOpenMode::Rendered,
            markdown_pretty: MarkdownPrettyConfig::default(),
        }
    }
}

impl HeaderlineTestUserLibrary {
    fn with_scrolloff(scrolloff: f64) -> Self {
        Self {
            scrolloff,
            ..Self::default()
        }
    }

    fn with_pdf_open_mode(pdf_open_mode: PdfOpenMode) -> Self {
        Self {
            pdf_open_mode,
            ..Self::default()
        }
    }

    fn headerline_call_count(&self) -> usize {
        self.headerline_call_count.load(Ordering::Relaxed)
    }
}

impl UserLibrary for HeaderlineTestUserLibrary {
    fn packages(&self) -> Vec<editor_plugin_api::PluginPackage> {
        Vec::new()
    }

    fn themes(&self) -> Vec<Theme> {
        vec![Theme::new("default", "Default").with_option("scrolloff", self.scrolloff)]
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
        editor_icons::symbols::cod::COD_SYMBOL_MISC
    }

    fn hover_providers(&self) -> Vec<HoverProvider> {
        Vec::new()
    }

    fn hover_line_limit(&self) -> usize {
        10
    }

    fn hover_token_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_INFO
    }

    fn hover_signature_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_SIGNATURE
    }

    fn acp_clients(&self) -> Vec<AcpClient> {
        Vec::new()
    }

    fn acp_client_by_id(&self, _id: &str) -> Option<AcpClient> {
        None
    }

    fn workspace_roots(&self) -> Vec<WorkspaceRoot> {
        Vec::new()
    }

    fn terminal_config(&self) -> TerminalConfig {
        TerminalConfig {
            program: "powershell.exe".to_owned(),
            args: vec!["-NoLogo".to_owned()],
        }
    }

    fn commandline_enabled(&self) -> bool {
        true
    }

    fn markdown_pretty_config(&self) -> MarkdownPrettyConfig {
        self.markdown_pretty.clone()
    }

    fn pane_config(&self) -> PaneConfig {
        PaneConfig {
            golden_ratio: false,
        }
    }

    fn ligature_config(&self) -> LigatureConfig {
        LigatureConfig { enabled: false }
    }

    fn rainbow_parens_config(&self) -> RainbowParensConfig {
        RainbowParensConfig { enabled: true }
    }

    fn show_paren_config(&self) -> editor_plugin_api::ShowParenConfig {
        editor_plugin_api::ShowParenConfig { enabled: true }
    }

    fn oil_defaults(&self) -> OilDefaults {
        OilDefaults {
            show_hidden: false,
            sort_mode: editor_plugin_api::OilSortMode::TypeThenName,
            trash_enabled: false,
        }
    }

    fn oil_keybindings(&self) -> OilKeybindings {
        OilKeybindings {
            open_entry: "Enter",
            open_vertical_split: "s",
            open_horizontal_split: "S",
            open_new_pane: "p",
            preview_entry: "-",
            refresh: "gr",
            close: "q",
            prefix: "g",
            open_parent: "..",
            open_workspace_root: "~",
            set_root: "cd",
            show_help: "?",
            cycle_sort: "gs",
            toggle_hidden: "gh",
            toggle_trash: "gt",
            open_external: "gx",
            set_tab_local_root: "gl",
            create_git_worktree: "gwn",
        }
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
        _entries: &[editor_fs::DirectoryEntry],
        _show_hidden: bool,
        _sort_mode: editor_plugin_api::OilSortMode,
        _trash_enabled: bool,
    ) -> editor_core::SectionTree {
        editor_core::SectionTree::default()
    }

    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str {
        label
    }

    fn git_status_sections(
        &self,
        _snapshot: &editor_git::GitStatusSnapshot,
    ) -> editor_core::SectionTree {
        editor_core::SectionTree::default()
    }

    fn git_commit_template(&self, _snapshot: &editor_git::GitStatusSnapshot) -> Vec<String> {
        Vec::new()
    }

    fn git_prefix_for_chord(&self, _chord: &str) -> Option<editor_plugin_api::GitStatusPrefix> {
        None
    }

    fn git_command_for_chord(
        &self,
        _prefix: Option<editor_plugin_api::GitStatusPrefix>,
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
        "URL > ".to_owned()
    }

    fn browser_url_placeholder(&self) -> String {
        "https://example.com".to_owned()
    }

    fn pdf_open_mode(&self) -> PdfOpenMode {
        self.pdf_open_mode
    }

    fn headerline_lines(&self, context: &GhostTextContext<'_>) -> Vec<String> {
        self.headerline_call_count.fetch_add(1, Ordering::Relaxed);
        if self.headerline_requires_scrolled_viewport && context.viewport_top_line == 0 {
            return Vec::new();
        }
        self.headerline_lines.clone()
    }

    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String {
        format!(
            " {} | {}:{} ",
            context.buffer_name, context.line, context.column
        )
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

    fn icon_symbols(&self) -> &'static [editor_icons::IconFontSymbol] {
        editor_icons::all_symbols()
    }

    fn run_plugin_buffer_evaluator(&self, _handler: &str, _input: &str) -> Vec<String> {
        Vec::new()
    }

    fn plugin_buffer(&self, _kind: &str) -> Option<PluginBuffer> {
        None
    }

    fn plugin_buffer_sections(&self, _kind: &str) -> Option<PluginBufferSections> {
        None
    }

    fn default_build_command(&self, _language: &str) -> Option<String> {
        None
    }
}

#[test]
fn resolve_default_workspace_root_prefers_existing_executable_relative_user_dir() {
    let temp_root = TempTestDir::new("default-workspace-root");
    let exe_dir = temp_root.path().join("target").join("debug").join("deps");
    let bundled_user_dir = temp_root.path().join("target").join("debug").join("user");
    fs::create_dir_all(&exe_dir).expect("create fake executable directory");
    fs::create_dir_all(&bundled_user_dir).expect("create bundled user directory");

    let resolved = resolve_default_workspace_root(Some(&exe_dir.join("volt-tests")), None);
    assert_eq!(resolved, Some(bundled_user_dir));
}

#[test]
fn file_open_detail_returns_basenames_for_extension_and_extensionless_files() {
    assert_eq!(
        file_open_detail(Path::new("src\\main.rs")).as_deref(),
        Some("main.rs")
    );
    assert_eq!(
        file_open_detail(Path::new("Makefile")).as_deref(),
        Some("Makefile")
    );
}

#[test]
fn resolve_default_workspace_root_falls_back_to_executable_user_dir() {
    let temp_root = TempTestDir::new("default-workspace-fallback");
    let exe_dir = temp_root.path().join("bin");
    assert_eq!(
        resolve_default_workspace_root(Some(&exe_dir.join("volt")), Some(temp_root.path())),
        Some(exe_dir.join("user"))
    );
}

#[cfg(windows)]
#[test]
fn normalize_git_output_path_converts_git_for_windows_drive_roots() {
    assert_eq!(
        normalize_git_output_path("/p/volt/target/release/user"),
        PathBuf::from(r"P:\volt\target\release\user")
    );
    assert_eq!(
        normalize_git_output_path(r"P:\volt\target\release\user"),
        PathBuf::from(r"P:\volt\target\release\user")
    );
    assert_eq!(
        normalize_git_output_path("w:/w/ftc-ui-web"),
        PathBuf::from(r"W:\w\ftc-ui-web")
    );
    assert_eq!(normalize_git_output_path(".git"), PathBuf::from(".git"));
}

#[cfg(windows)]
#[test]
fn worktree_remove_uses_porcelain_raw_path_not_normalized_windows_path() {
    let entries = parse_git_worktree_list(
        "\
worktree /w/ftc-ui-web
bare

worktree /w/ftc-ui-web/map
HEAD a3dcf8f90bfe54a1bffb3c505ec878c8566986fd
branch refs/heads/feature/TASK-5645-mapchanges

worktree W:\\ftc-ui-web/main
HEAD 3811c5ec536197500efa15290940d47f3f55cff5
branch refs/heads/origin-main
",
    )
    .expect("porcelain parses");

    let invocation =
        worktree_remove_git_invocation_for_entries(&entries, Path::new(r"W:\ftc-ui-web\map"));
    assert_eq!(
        invocation,
        WorktreeRemoveGitInvocation::Remove {
            cli_path: "/w/ftc-ui-web/map".to_owned(),
        },
        "Git only recognizes the registered MSYS spelling"
    );
    assert_eq!(
        worktree_remove_git_args(&invocation),
        vec![
            "worktree".to_owned(),
            "remove".to_owned(),
            "/w/ftc-ui-web/map".to_owned(),
            "--force".to_owned(),
        ]
    );
}

#[cfg(windows)]
#[test]
fn worktree_remove_prunes_when_porcelain_marks_entry_prunable() {
    let entries = parse_git_worktree_list(
        "\
worktree /w/ftc-ui-web/map
HEAD a3dcf8f90bfe54a1bffb3c505ec878c8566986fd
branch refs/heads/feature/TASK-5645-mapchanges
prunable gitdir file points to non-existent location
",
    )
    .expect("porcelain parses");

    let invocation =
        worktree_remove_git_invocation_for_entries(&entries, Path::new(r"W:\ftc-ui-web\map"));
    assert_eq!(invocation, WorktreeRemoveGitInvocation::Prune);
    assert_eq!(
        worktree_remove_git_args(&invocation),
        vec!["worktree".to_owned(), "prune".to_owned(), "-v".to_owned()]
    );
}

#[test]
fn shell_state_uses_default_workspace_root() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    let root = state
        .runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?
        .root()
        .map(Path::to_path_buf);
    assert_eq!(root, default_workspace_root());
    Ok(())
}

fn slice_by_columns(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn syntax_span_segments(line: &str, spans: &[LineSyntaxSpan]) -> Vec<(String, String)> {
    spans
        .iter()
        .map(|span| {
            (
                span.theme_token.to_string(),
                slice_by_columns(line, span.start, span.end),
            )
        })
        .collect()
}

fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!(
        "volt-shell-tests-{label}-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to create temp dir `{}`: {error}", dir.display()));
    dir
}

fn write_test_png(path: &Path) -> Result<(), String> {
    let image = image::RgbaImage::from_pixel(40, 20, image::Rgba([255, 0, 0, 255]));
    image.save(path).map_err(|error| error.to_string())
}

fn write_test_svg(path: &Path) -> Result<(), String> {
    std::fs::write(
        path,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20" viewBox="0 0 40 20">
  <rect width="40" height="20" fill="#1f6feb"/>
  <circle cx="10" cy="10" r="6" fill="#f2cc60"/>
</svg>"##,
    )
    .map_err(|error| error.to_string())
}

fn write_test_pdf(path: &Path, page_texts: &[&str]) -> Result<(), String> {
    use lopdf::content::{Content, Operation};
    use lopdf::dictionary;
    use lopdf::{Document as PdfDocument, Object as PdfObject, Stream};

    let mut document = PdfDocument::with_version("1.5");
    let info_id = document.add_object(lopdf::dictionary! {
        "Title" => PdfObject::string_literal("Volt PDF Test"),
        "Creator" => PdfObject::string_literal("volt"),
    });
    let pages_id = document.new_object_id();
    let font_id = document.add_object(lopdf::dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    });
    let resources_id = document.add_object(lopdf::dictionary! {
        "Font" => lopdf::dictionary! {
            "F1" => font_id,
        },
    });
    let pages = page_texts.iter().map(|text| {
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 24.into()]),
                Operation::new("Td", vec![72.into(), 720.into()]),
                Operation::new("Tj", vec![PdfObject::string_literal(*text)]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = document.add_object(Stream::new(
            lopdf::dictionary! {},
            content.encode().map_err(|error| error.to_string())?,
        ));
        Ok::<PdfObject, String>(
            document
                .add_object(lopdf::dictionary! {
                    "Type" => "Page",
                    "Parent" => pages_id,
                    "Contents" => content_id,
                })
                .into(),
        )
    });
    let kids = pages.collect::<Result<Vec<_>, _>>()?;
    document.objects.insert(
        pages_id,
        PdfObject::Dictionary(lopdf::dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => page_texts.len() as i64,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        }),
    );
    let catalog_id = document.add_object(lopdf::dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    document.trailer.set("Root", catalog_id);
    document.trailer.set("Info", info_id);
    document.compress();
    document.save(path).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn write_test_pdf_creates_extractable_pages() -> Result<(), String> {
    let root = unique_temp_dir("write-test-pdf");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["alpha", "bravo"])?;

    let document = lopdf::Document::load(&path).map_err(|error| error.to_string())?;
    assert_eq!(document.get_pages().len(), 2);
    assert_eq!(
        document
            .extract_text(&[1])
            .map_err(|error| error.to_string())?,
        "alpha\n"
    );
    assert_eq!(
        document
            .extract_text(&[2])
            .map_err(|error| error.to_string())?,
        "bravo\n"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn pdf_helpers_parse_paths_state_and_render_lines() -> Result<(), String> {
    let root = unique_temp_dir("pdf-helpers");
    let path = root.join("sample.PDF");
    write_test_pdf(&path, &["page one", "page two"])?;

    assert!(is_pdf_path(&path));
    assert!(!is_pdf_path(Path::new("sample.txt")));
    assert!(!is_pdf_path(Path::new("sample")));
    assert_eq!(pdf_fit_mode_label(PdfFitMode::Page), "fit page");

    let mut state = load_pdf_buffer_state(&path)?;
    assert_eq!(state.page_count(), 2);
    assert_eq!(state.metadata.page_count, 2);
    assert_eq!(pdf_page_rotation(&state.document, 1), None);
    assert_eq!(
        pdf_page_media_box(&state.document, 1).as_deref(),
        Some("0 0 595 842")
    );
    assert_eq!(pdf_page_text(&state.document, 1), "page one");
    assert_eq!(pdf_page_text(&state.document, 99), "");

    let second_page_id = state
        .document
        .get_pages()
        .get(&2)
        .copied()
        .ok_or_else(|| "second page missing".to_owned())?;
    state
        .document
        .get_dictionary_mut(second_page_id)
        .map_err(|error| error.to_string())?
        .set("Rotate", 90);
    state.current_page = 2;
    state.dirty = true;
    state.render_error = Some("missing renderer".to_owned());
    let lines = pdf_buffer_lines("sample.pdf", Some(&path), &state);
    let body = lines.join("\n");
    assert!(body.contains("Page 2/2"));
    assert!(body.contains("rotation 90°"));
    assert!(body.contains("page two"));
    assert!(body.contains("Modified: yes"));
    assert!(body.contains("Rendered preview unavailable: missing renderer"));

    state.open_mode = PdfOpenMode::Markdown;
    let markdown = pdf_buffer_lines("sample.pdf", Some(&path), &state).join("\n");
    assert!(markdown.contains("# sample.pdf"));
    assert!(markdown.contains("## Page 1"));
    assert!(markdown.contains("## Page 2"));

    state.open_mode = PdfOpenMode::Latex;
    let latex = pdf_buffer_lines("sample.pdf", Some(&path), &state).join("\n");
    assert!(latex.contains(r"\section*{sample.pdf}"));
    assert!(latex.contains(r"\subsection*{Page 2}"));

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn load_pdf_buffer_state_rejects_missing_and_invalid_files() {
    let root = unique_temp_dir("pdf-invalid");
    let missing = root.join("missing.pdf");
    assert!(load_pdf_buffer_state(&missing).is_err());

    let invalid = root.join("invalid.pdf");
    std::fs::write(&invalid, "not a pdf").expect("write invalid pdf");
    assert!(load_pdf_buffer_state(&invalid).is_err());

    std::fs::remove_dir_all(&root).expect("remove temp dir");
}

const MATERIAL_ICONS_FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../volt/assets/font/material-design-icons.ttf"
));

fn berkeley_mono_font() -> Option<&'static [u8]> {
    static BERKELEY_MONO_FONT: std::sync::OnceLock<Option<Box<[u8]>>> = std::sync::OnceLock::new();
    BERKELEY_MONO_FONT
        .get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../LigaBerkeleyMono-Regular.ttf");
            std::fs::read(path).ok().map(Vec::into_boxed_slice)
        })
        .as_deref()
}

const BERKELEY_MONO_TEST_CELL_WIDTH: i32 = 11;

fn berkeley_mono_ligature_test_assets() -> Option<(ShapeFace<'static>, RasterFont)> {
    let bytes = berkeley_mono_font()?;
    Some((
        ShapeFace::from_slice(bytes, 0)?,
        RasterFont::from_bytes(bytes, fontdue::FontSettings::default()).ok()?,
    ))
}

fn configure_file_buffer(
    state: &mut ShellState,
    buffer_id: BufferId,
    path: &Path,
) -> Result<(), String> {
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        buffer.kind = BufferKind::File;
        buffer.name = path.display().to_string();
        buffer.text = TextBuffer::load_from_path(path).map_err(|error| error.to_string())?;
        buffer.backing_file_fingerprint = BackingFileFingerprint::read(path).ok();
        buffer.backing_file_reload_pending = false;
        buffer.backing_file_check_in_flight = false;
    }
    shell_ui_mut(&mut state.runtime)?
        .file_reload_worker
        .watch_path(path.to_path_buf());
    Ok(())
}

fn active_and_secondary_buffer_ids(
    runtime: &EditorRuntime,
) -> Result<(BufferId, BufferId), String> {
    let ui = shell_ui(runtime)?;
    let active_buffer_id = ui
        .active_buffer_id()
        .ok_or_else(|| "active buffer is missing".to_owned())?;
    let secondary_buffer_id = ui
        .active_workspace_buffer_ids()
        .and_then(|buffer_ids| {
            buffer_ids
                .iter()
                .copied()
                .find(|buffer_id| *buffer_id != active_buffer_id)
        })
        .ok_or_else(|| "secondary buffer is missing".to_owned())?;
    Ok((active_buffer_id, secondary_buffer_id))
}

fn wait_for_file_reload_worker(
    state: &mut ShellState,
    buffer_ids: &[BufferId],
) -> Result<(), String> {
    for _ in 0..200 {
        let _ = refresh_pending_file_reloads(&mut state.runtime, Instant::now(), false)?;
        if buffer_ids.iter().copied().all(|buffer_id| {
            shell_buffer(&state.runtime, buffer_id)
                .map(|buffer| !buffer.backing_file_check_in_flight)
                .unwrap_or(true)
        }) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("timed out waiting for the file reload worker".to_owned())
}

fn wait_for_file_reload_change(state: &mut ShellState) -> Result<bool, String> {
    for _ in 0..200 {
        if refresh_pending_file_reloads(&mut state.runtime, Instant::now(), false)? {
            return Ok(true);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(false)
}

fn record_file_reload_event(state: &ShellState, path: &Path) -> Result<(), String> {
    shell_ui(&state.runtime)?
        .file_reload_worker
        .record_changed_path_for_test(path.to_path_buf());
    Ok(())
}

#[test]
fn ligature_shaping_collapses_material_icon_label_when_enabled() {
    let face = rustybuzz::Face::from_slice(MATERIAL_ICONS_FONT, 0)
        .unwrap_or_else(|| panic!("failed to parse bundled Material Icons font"));
    let shaped = shape_ascii_ligature_run_with_face(&face, 18.0, true, "face")
        .unwrap_or_else(|| panic!("expected `face` ligature to shape"));

    assert!(shaped.glyphs.len() < "face".chars().count());
}

#[test]
fn ligature_shaping_is_disabled_by_user_toggle() {
    let face = rustybuzz::Face::from_slice(MATERIAL_ICONS_FONT, 0)
        .unwrap_or_else(|| panic!("failed to parse bundled Material Icons font"));

    assert!(shape_ascii_ligature_run_with_face(&face, 18.0, false, "face").is_none());
}

#[test]
fn ligature_shaping_accepts_same_length_contextual_substitutions() {
    let Some(berkeley_mono_font) = berkeley_mono_font() else {
        eprintln!("skipping: Berkeley Mono test font is unavailable");
        return;
    };
    let face = rustybuzz::Face::from_slice(berkeley_mono_font, 0)
        .unwrap_or_else(|| panic!("failed to parse Berkeley Mono test font"));
    let shaped = shape_ascii_ligature_run_with_face(&face, 18.0, true, "=>")
        .unwrap_or_else(|| panic!("expected `=>` to shape"));
    let nominal_font =
        fontdue::Font::from_bytes(berkeley_mono_font, fontdue::FontSettings::default())
            .unwrap_or_else(|error| panic!("failed to parse Berkeley Mono raster font: {error}"));

    assert_eq!(shaped.glyphs.len(), 2);
    assert!(
        shaped
            .glyphs
            .iter()
            .zip("=>".chars())
            .any(|(glyph, character)| nominal_font.lookup_glyph_index(character) != glyph.glyph_id)
    );
    assert!(shaped_run_uses_cell_grid("=>", &shaped));
}

#[test]
fn same_length_inline_ligatures_stay_layout_safe_on_cell_grid() {
    let Some(berkeley_mono_font) = berkeley_mono_font() else {
        eprintln!("skipping: Berkeley Mono test font is unavailable");
        return;
    };
    let face = rustybuzz::Face::from_slice(berkeley_mono_font, 0)
        .unwrap_or_else(|| panic!("failed to parse Berkeley Mono test font"));
    let shaped = shape_ascii_ligature_run_with_face(&face, 18.0, true, "a => b")
        .unwrap_or_else(|| panic!("expected inline ligature to shape"));

    assert!(shaped_run_uses_cell_grid("a => b", &shaped));
    assert!(shaped_run_preserves_monospace_layout("a => b", &shaped, 11));
}

#[test]
fn ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text() {
    let Some((face, raster_font)) = berkeley_mono_ligature_test_assets() else {
        eprintln!("skipping: Berkeley Mono test font is unavailable");
        return;
    };

    assert_eq!(
        ascii_ligature_byte_ranges_with_face(
            &face,
            &raster_font,
            18.0,
            true,
            "a => b",
            BERKELEY_MONO_TEST_CELL_WIDTH,
        ),
        vec![2..4]
    );
}

#[test]
fn split_primary_text_by_ligature_ranges_keeps_whole_line_surrounding_text_on_primary_path() {
    let Some((face, raster_font)) = berkeley_mono_ligature_test_assets() else {
        eprintln!("skipping: Berkeley Mono test font is unavailable");
        return;
    };
    let ligature_ranges = ascii_ligature_byte_ranges_with_face(
        &face,
        &raster_font,
        18.0,
        true,
        "a => b",
        BERKELEY_MONO_TEST_CELL_WIDTH,
    );

    assert_eq!(
        split_primary_text_by_ligature_ranges("a => b", &ligature_ranges),
        vec![
            PrimaryTextRun {
                render_mode: PrimaryTextRenderMode::Normal,
                text: "a ".to_owned(),
            },
            PrimaryTextRun {
                render_mode: PrimaryTextRenderMode::Ligature,
                text: "=>".to_owned(),
            },
            PrimaryTextRun {
                render_mode: PrimaryTextRenderMode::Normal,
                text: " b".to_owned(),
            },
        ]
    );
}

#[test]
fn split_primary_text_by_ligature_ranges_respects_preexisting_color_boundaries() {
    let Some((face, raster_font)) = berkeley_mono_ligature_test_assets() else {
        eprintln!("skipping: Berkeley Mono test font is unavailable");
        return;
    };

    assert_eq!(
        split_primary_text_by_ligature_ranges(
            "a ",
            &ascii_ligature_byte_ranges_with_face(
                &face,
                &raster_font,
                18.0,
                true,
                "a ",
                BERKELEY_MONO_TEST_CELL_WIDTH,
            ),
        ),
        vec![PrimaryTextRun {
            render_mode: PrimaryTextRenderMode::Normal,
            text: "a ".to_owned(),
        }]
    );
    assert_eq!(
        split_primary_text_by_ligature_ranges(
            "=>",
            &ascii_ligature_byte_ranges_with_face(
                &face,
                &raster_font,
                18.0,
                true,
                "=>",
                BERKELEY_MONO_TEST_CELL_WIDTH,
            ),
        ),
        vec![PrimaryTextRun {
            render_mode: PrimaryTextRenderMode::Ligature,
            text: "=>".to_owned(),
        }]
    );
    assert_eq!(
        split_primary_text_by_ligature_ranges(
            " b",
            &ascii_ligature_byte_ranges_with_face(
                &face,
                &raster_font,
                18.0,
                true,
                " b",
                BERKELEY_MONO_TEST_CELL_WIDTH,
            ),
        ),
        vec![PrimaryTextRun {
            render_mode: PrimaryTextRenderMode::Normal,
            text: " b".to_owned(),
        }]
    );
}

#[test]
fn styled_primary_font_path_prefers_real_style_files() {
    let temp_root = env::temp_dir().join(format!(
        "volt-styled-fonts-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_millis()
    ));
    fs::create_dir_all(&temp_root).expect("create temp font dir");
    let regular = temp_root.join("ExampleMono-Regular.ttf");
    let bold = temp_root.join("ExampleMono-Bold.ttf");
    fs::write(&regular, []).expect("write regular font marker");
    fs::write(&bold, []).expect("write bold font marker");

    assert_eq!(
        styled_primary_font_path(&regular, TextStyle::new(true, false)),
        bold
    );
    assert_eq!(
        styled_primary_font_path(&regular, TextStyle::new(false, true)),
        regular
    );

    fs::remove_dir_all(&temp_root).expect("cleanup temp font dir");
}

#[test]
fn text_texture_cache_keys_keep_same_text_separate_per_color() {
    let text = "=>".to_owned();

    assert_ne!(
        TextTextureCacheKey::Primary {
            text: text.clone(),
            color: render_color_cache_key(RenderColor::rgba(10, 20, 30, 255)),
            style: TextStyle::plain(),
        },
        TextTextureCacheKey::Primary {
            text: text.clone(),
            color: render_color_cache_key(RenderColor::rgba(10, 20, 31, 255)),
            style: TextStyle::plain(),
        }
    );
    assert_ne!(
        TextTextureCacheKey::Ligature {
            text: text.clone(),
            color: render_color_cache_key(RenderColor::rgba(10, 20, 30, 255)),
        },
        TextTextureCacheKey::Ligature {
            text,
            color: render_color_cache_key(RenderColor::rgba(10, 20, 31, 255)),
        }
    );
}

#[test]
fn contextual_ligature_raster_size_keeps_changed_glyphs_at_base_size() {
    let Some(berkeley_mono_font) = berkeley_mono_font() else {
        eprintln!("skipping: Berkeley Mono test font is unavailable");
        return;
    };
    let face = rustybuzz::Face::from_slice(berkeley_mono_font, 0)
        .unwrap_or_else(|| panic!("failed to parse Berkeley Mono test font"));
    let shaped = shape_ascii_ligature_run_with_face(&face, 18.0, true, "=>")
        .unwrap_or_else(|| panic!("expected `=>` to shape"));
    let raster_font =
        fontdue::Font::from_bytes(berkeley_mono_font, fontdue::FontSettings::default())
            .unwrap_or_else(|error| panic!("failed to parse Berkeley Mono raster font: {error}"));

    assert!(
        "=>".chars()
            .zip(shaped.glyphs.iter())
            .any(|(character, glyph)| {
                raster_font.lookup_glyph_index(character) != glyph.glyph_id
                    && adjusted_contextual_ligature_pixel_size(
                        &raster_font,
                        18.0,
                        character,
                        glyph.glyph_id,
                    ) == 18.0
            })
    );
}

#[test]
fn contextual_ligature_raster_size_never_upscales_smaller_substitute_glyphs() -> Result<(), String>
{
    let font_path = resolve_bundled_icon_font_dir()
        .map_err(|error| error.to_string())?
        .join("NFM.ttf");
    let bytes = fs::read(&font_path).map_err(|error| error.to_string())?;
    let raster_font = RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())?;
    let base_pixel_size = 18.0;
    let icon_characters = [
        editor_icons::symbols::md::MD_FORMAT_BOLD,
        editor_icons::symbols::cod::COD_DIFF_ADDED,
        editor_icons::symbols::dev::DEV_GIT_BRANCH,
        editor_icons::symbols::fa::FA_CONNECTDEVELOP,
        editor_icons::symbols::ple::PL_BRANCH,
    ]
    .into_iter()
    .map(|icon| {
        icon.chars()
            .next()
            .ok_or_else(|| "expected icon glyph".to_owned())
    })
    .collect::<Result<Vec<_>, _>>()?;
    let (nominal_character, substitute_glyph_id) = icon_characters
        .iter()
        .copied()
        .find_map(|nominal_character| {
            let nominal_metrics = raster_font.metrics(nominal_character, base_pixel_size);
            if nominal_metrics.width == 0 || nominal_metrics.height == 0 {
                return None;
            }
            let nominal_glyph_id = raster_font.lookup_glyph_index(nominal_character);
            icon_characters
                .iter()
                .copied()
                .find_map(|substitute_character| {
                    let substitute_glyph_id = raster_font.lookup_glyph_index(substitute_character);
                    if substitute_glyph_id == nominal_glyph_id {
                        return None;
                    }
                    let substitute_metrics =
                        raster_font.metrics_indexed(substitute_glyph_id, base_pixel_size);
                    if substitute_metrics.width == 0 || substitute_metrics.height == 0 {
                        return None;
                    }
                    let height_scale =
                        nominal_metrics.height as f32 / substitute_metrics.height as f32;
                    let width_scale =
                        nominal_metrics.width as f32 / substitute_metrics.width as f32;
                    (height_scale.max(width_scale) > 1.0)
                        .then_some((nominal_character, substitute_glyph_id))
                })
        })
        .ok_or_else(|| {
            "expected bundled NFM font to contain a smaller substitute glyph".to_owned()
        })?;

    assert_eq!(
        adjusted_contextual_ligature_pixel_size(
            &raster_font,
            base_pixel_size,
            nominal_character,
            substitute_glyph_id,
        ),
        base_pixel_size
    );
    Ok(())
}

#[test]
fn ligature_shape_cache_stores_negative_results() {
    let mut cache: TextTextureCache<'static> = TextTextureCache::new();

    assert!(cache.get_ligature_shape("plain").is_none());
    assert_eq!(
        cache.insert_ligature_shape("plain".to_owned(), LigatureShapeCacheValue::NotLigature),
        LigatureShapeCacheValue::NotLigature
    );
    assert_eq!(
        cache.get_ligature_shape("plain"),
        Some(LigatureShapeCacheValue::NotLigature)
    );
}

#[test]
fn ligature_shape_cache_stores_layout_results() {
    let mut cache: TextTextureCache<'static> = TextTextureCache::new();
    let layout = CachedLigatureLayout {
        glyphs: vec![CachedLigatureGlyphPlacement {
            glyph_id: 7,
            draw_x: -1,
            draw_y: 3,
            width: 8,
            height: 10,
            raster_px_64: encode_raster_px_64(18.0),
        }],
        offset_x: -1,
        offset_y: 3,
        width: 8,
        height: 10,
        advance: 11,
    };

    assert_eq!(
        cache.insert_ligature_shape(
            "=>".to_owned(),
            LigatureShapeCacheValue::Layout(layout.clone()),
        ),
        LigatureShapeCacheValue::Layout(layout.clone())
    );
    assert_eq!(
        cache.get_ligature_shape("=>"),
        Some(LigatureShapeCacheValue::Layout(layout))
    );
}

#[test]
fn primary_text_run_cache_stores_split_results() {
    let mut cache: TextTextureCache<'static> = TextTextureCache::new();
    let runs = vec![
        PrimaryTextRun {
            render_mode: PrimaryTextRenderMode::Normal,
            text: "a ".to_owned(),
        },
        PrimaryTextRun {
            render_mode: PrimaryTextRenderMode::Ligature,
            text: "=>".to_owned(),
        },
        PrimaryTextRun {
            render_mode: PrimaryTextRenderMode::Normal,
            text: " b".to_owned(),
        },
    ];

    assert!(cache.get_primary_text_runs("a => b").is_none());
    assert_eq!(
        cache.insert_primary_text_runs("a => b".to_owned(), runs.clone()),
        runs
    );
    assert_eq!(cache.get_primary_text_runs("a => b"), Some(runs));
}

#[test]
fn build_cached_text_layout_returns_empty_layout_when_no_glyphs() {
    let layout = build_cached_text_layout(Vec::new(), 17);

    assert_eq!(
        layout,
        CachedLigatureLayout {
            glyphs: Vec::new(),
            offset_x: 0,
            offset_y: 0,
            width: 0,
            height: 0,
            advance: 17,
        }
    );
}

#[test]
fn build_cached_text_layout_tracks_bounds_for_nominal_glyphs() {
    let layout = build_cached_text_layout(
        vec![
            CachedGlyphRasterPlacement {
                glyph_id: 7,
                draw_x: -1,
                draw_y: 3,
                width: 8,
                height: 10,
                raster_px_64: encode_raster_px_64(18.0),
            },
            CachedGlyphRasterPlacement {
                glyph_id: 8,
                draw_x: 10,
                draw_y: 5,
                width: 6,
                height: 7,
                raster_px_64: encode_raster_px_64(18.0),
            },
        ],
        22,
    );

    assert_eq!(
        layout,
        CachedLigatureLayout {
            glyphs: vec![
                CachedLigatureGlyphPlacement {
                    glyph_id: 7,
                    draw_x: -1,
                    draw_y: 3,
                    width: 8,
                    height: 10,
                    raster_px_64: encode_raster_px_64(18.0),
                },
                CachedLigatureGlyphPlacement {
                    glyph_id: 8,
                    draw_x: 10,
                    draw_y: 5,
                    width: 6,
                    height: 7,
                    raster_px_64: encode_raster_px_64(18.0),
                },
            ],
            offset_x: -1,
            offset_y: 3,
            width: 17,
            height: 10,
            advance: 22,
        }
    );
}

#[test]
fn composite_alpha_bitmap_preserves_straight_alpha_for_overlaps() {
    let mut surface = Surface::new(1, 1, PixelFormat::RGBA32)
        .unwrap_or_else(|error| panic!("failed to create surface: {error}"));
    surface
        .fill_rect(None, Color::RGBA(0, 0, 0, 0))
        .unwrap_or_else(|error| panic!("failed to clear surface: {error}"));

    composite_alpha_bitmap(
        &mut surface,
        0,
        0,
        1,
        1,
        &[128],
        RenderColor::rgba(10, 20, 30, 255),
    );
    composite_alpha_bitmap(
        &mut surface,
        0,
        0,
        1,
        1,
        &[128],
        RenderColor::rgba(10, 20, 30, 255),
    );

    surface.with_lock(|pixels| {
        assert_eq!(&pixels[..4], &[10, 20, 30, 191]);
    });
}

#[test]
fn render_primary_text_surface_preserves_straight_alpha_edge_colors() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 24,
            emoji_font_size: 24,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let color = RenderColor::rgba(61, 122, 211, 255);
    let surface = render_primary_text_surface(&fonts, "Volt", color, TextStyle::plain())
        .map_err(|error| error.to_string())?;
    assert_eq!(surface.pixel_format_enum(), PixelFormat::RGBA32);
    let width = surface.width() as usize;
    let height = surface.height() as usize;
    let pitch = surface.pitch() as usize;
    let mut partial_alpha_pixels = 0usize;

    surface.with_lock(|pixels| {
        for row in pixels.chunks(pitch).take(height) {
            let row_pixels = &row[..width.saturating_mul(4)];
            for rgba in row_pixels.as_chunks::<4>().0 {
                let alpha = rgba[3];
                if alpha != 0 && alpha != u8::MAX {
                    partial_alpha_pixels += 1;
                    assert_eq!(&rgba[..3], &[color.r, color.g, color.b]);
                }
            }
        }
    });

    assert!(
        partial_alpha_pixels > 0,
        "expected antialiased glyph edges with partial alpha coverage"
    );
    Ok(())
}

#[test]
fn compose_ligature_surface_uses_grayscale_glyph_coverage() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 18,
            emoji_font_size: 18,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let glyph_id = fonts.primary_raster_font().lookup_glyph_index('/');
    let pixel_size = decode_raster_px_64(encode_raster_px_64(fonts.primary_pixel_size()));
    let (metrics, bitmap) = fonts
        .primary_raster_font()
        .rasterize_indexed(glyph_id, pixel_size);
    assert!(metrics.width > 0 && metrics.height > 0);
    let layout = CachedLigatureLayout {
        glyphs: vec![CachedLigatureGlyphPlacement {
            glyph_id,
            draw_x: 0,
            draw_y: 0,
            width: metrics.width as u32,
            height: metrics.height as u32,
            raster_px_64: encode_raster_px_64(pixel_size),
        }],
        offset_x: 0,
        offset_y: 0,
        width: metrics.width as u32,
        height: metrics.height as u32,
        advance: metrics.width.max(1) as i32,
    };
    let surface = compose_ligature_surface(&fonts, &layout, RenderColor::rgba(10, 20, 30, 255))
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "expected composed ligature surface".to_owned())?;
    let width = metrics.width;
    let height = metrics.height;
    let pitch = surface.pitch() as usize;
    surface.with_lock(|pixels| {
        for row in 0..height {
            let row_start = row * pitch;
            let row_pixels = &pixels[row_start..row_start + width * 4];
            for col in 0..width {
                let alpha = bitmap[row * width + col];
                let pixel_start = col * 4;
                let expected = if alpha == 0 {
                    [0, 0, 0, 0]
                } else {
                    [10, 20, 30, alpha]
                };
                assert_eq!(&row_pixels[pixel_start..pixel_start + 4], &expected);
            }
        }
    });
    Ok(())
}

#[test]
fn keydown_chord_maps_alt_x() {
    assert_eq!(
        keydown_chord(Keycode::X, Mod::LALTMOD).as_deref(),
        Some("Alt+x")
    );
}

#[test]
fn keydown_chord_maps_ctrl_tab() {
    assert_eq!(
        keydown_chord(Keycode::Tab, ctrl_mod()).as_deref(),
        Some("Ctrl+Tab")
    );
}

#[test]
fn keydown_chord_maps_enter_variants() {
    for keycode in [Keycode::Return, Keycode::KpEnter, Keycode::Return2] {
        assert_eq!(
            keydown_chord(keycode, ctrl_mod()).as_deref(),
            Some("Ctrl+Enter")
        );
        assert_eq!(keydown_chord(keycode, Mod::NOMOD).as_deref(), Some("Enter"));
    }
}

#[test]
fn keydown_chord_maps_image_zoom_controls() {
    assert_eq!(
        keydown_chord(Keycode::Equals, ctrl_mod()).as_deref(),
        Some("Ctrl+=")
    );
    assert_eq!(
        keydown_chord(Keycode::Minus, ctrl_mod()).as_deref(),
        Some("Ctrl+-")
    );
    assert_eq!(
        keydown_chord(Keycode::_0, ctrl_mod()).as_deref(),
        Some("Ctrl+0")
    );
}

#[test]
fn keydown_chord_maps_shifted_letter_and_function_key_modifiers() {
    assert_eq!(
        keydown_chord(Keycode::F7, Mod::NOMOD).as_deref(),
        Some("F7")
    );
    assert_eq!(
        keydown_chord(
            Keycode::F7,
            ctrl_mod() | alt_mod() | shift_mod() | gui_mod()
        )
        .as_deref(),
        Some("Ctrl+Alt+Shift+Gui+F7")
    );
    assert_eq!(
        keydown_chord(Keycode::H, ctrl_mod() | shift_mod()).as_deref(),
        Some("Ctrl+Shift+h")
    );
}

#[test]
fn keydown_chord_maps_shifted_printable_aliases() {
    assert_eq!(
        keydown_chord(Keycode::Backslash, ctrl_mod() | shift_mod()).as_deref(),
        Some("Ctrl+|")
    );
    assert_eq!(
        keydown_chord(Keycode::Pipe, ctrl_mod() | shift_mod()).as_deref(),
        Some("Ctrl+|")
    );
    assert_eq!(
        keydown_chord(Keycode::M, ctrl_mod()).as_deref(),
        Some("Ctrl+m")
    );
    assert_eq!(
        keydown_chord(Keycode::PageDown, Mod::NOMOD).as_deref(),
        Some("PageDown")
    );
}

#[test]
fn terminal_key_for_event_maps_special_keys() {
    assert_eq!(
        terminal_key_for_event(Keycode::Tab, Mod::LSHIFTMOD),
        Some(TerminalKey::BackTab)
    );
    assert_eq!(
        terminal_key_for_event(Keycode::Return2, Mod::NOMOD),
        Some(TerminalKey::Enter)
    );
    assert_eq!(
        terminal_key_for_event(Keycode::C, ctrl_mod()),
        Some(TerminalKey::CtrlC)
    );
    assert_eq!(
        terminal_key_for_event(Keycode::PageDown, Mod::NOMOD),
        Some(TerminalKey::PageDown)
    );
}

#[test]
fn terminal_buffers_are_read_only_without_prompt_input() {
    let (read_only, input) = buffer_interaction(&BufferKind::Terminal, &NullUserLibrary);
    assert!(read_only);
    assert!(input.is_none());
}

#[test]
fn directory_view_state_uses_user_oil_defaults() {
    let defaults = user::UserLibraryImpl.oil_defaults();
    let state = DirectoryViewState::new(std::path::PathBuf::from("."), Vec::new(), defaults);

    assert_eq!(state.show_hidden, defaults.show_hidden);
    assert_eq!(state.sort_mode, defaults.sort_mode);
    assert_eq!(state.trash_enabled, defaults.trash_enabled);
}

#[test]
fn oil_insert_creates_directory_file_and_nested_paths_on_normal() -> Result<(), String> {
    let root = unique_temp_dir("oil-insert-create");
    std::fs::write(root.join("existing.txt"), "keep\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    oil_type_new_entry_and_leave_insert(&mut state, "Test/")?;
    oil_type_new_entry_and_leave_insert(&mut state, "abc.txt")?;
    oil_type_new_entry_and_leave_insert(&mut state, "nested/dir/file.txt")?;

    assert!(
        root.join("Test").is_dir(),
        "typing Test/ then leaving insert should create directory"
    );
    assert!(
        root.join("abc.txt").is_file(),
        "typing abc.txt then leaving insert should create file"
    );
    assert!(
        root.join("nested").join("dir").join("file.txt").is_file(),
        "typing nested/dir/file.txt then leaving insert should create nested directories and file"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_normal_mode_dd_applies_delete_immediately() -> Result<(), String> {
    let root = unique_temp_dir("oil-normal-delete");
    let file_path = root.join("alpha.txt");
    std::fs::write(&file_path, "alpha\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    let file_line = oil_line_index_containing(&state.runtime, buffer_id, "alpha.txt")?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(file_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    assert!(!file_path.exists());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "alpha.txt").is_err(),
        "deleted file should leave the oil listing"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_normal_mode_yy_p_copies_file_immediately() -> Result<(), String> {
    let root = unique_temp_dir("oil-normal-copy-file");
    let source = root.join("source");
    let dest = root.join("dest");
    std::fs::create_dir_all(&source).map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dest).map_err(|error| error.to_string())?;
    let source_file = source.join("alpha.txt");
    std::fs::write(&source_file, "alpha\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    open_workspace_from_project(&mut state.runtime, "oil-copy-file", &root)?;
    open_oil_directory(&mut state.runtime, source.clone())?;
    let source_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_buffer_mut(&mut state.runtime, source_buffer_id)?.set_cursor(TextPoint::new(1, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(source_buffer_id);

    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;

    open_oil_directory(&mut state.runtime, dest.clone())?;
    let dest_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(dest_buffer_id);
    state
        .handle_text_input("p")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(dest.join("alpha.txt")).map_err(|error| error.to_string())?,
        "alpha\n"
    );
    assert!(
        shell_buffer(&state.runtime, dest_buffer_id)?
            .directory_state()
            .ok_or_else(|| "destination directory state missing".to_owned())?
            .entries
            .iter()
            .any(|entry| entry.path() == dest.join("alpha.txt"))
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_visual_line_y_p_copies_multiple_entries_immediately() -> Result<(), String> {
    let root = unique_temp_dir("oil-visual-copy-multiple");
    let source = root.join("source");
    let dest = root.join("dest");
    std::fs::create_dir_all(source.join("folder")).map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dest).map_err(|error| error.to_string())?;
    std::fs::write(source.join("folder").join("nested.txt"), "nested\n")
        .map_err(|error| error.to_string())?;
    std::fs::write(source.join("plain.txt"), "plain\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    open_workspace_from_project(&mut state.runtime, "oil-copy-multiple", &root)?;
    open_oil_directory(&mut state.runtime, source.clone())?;
    let source_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_buffer_mut(&mut state.runtime, source_buffer_id)?.set_cursor(TextPoint::new(1, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(source_buffer_id);

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;

    open_oil_directory(&mut state.runtime, dest.clone())?;
    state
        .handle_text_input("p")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(dest.join("folder").join("nested.txt"))
            .map_err(|error| error.to_string())?,
        "nested\n"
    );
    assert_eq!(
        std::fs::read_to_string(dest.join("plain.txt")).map_err(|error| error.to_string())?,
        "plain\n"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_open_parent_command_uses_parent_root() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("oil-open-parent");
    let child = root.join("nested");
    std::fs::create_dir_all(&child).map_err(|error| error.to_string())?;

    open_workspace_from_project(&mut state.runtime, "oil-parent", &root)?;
    open_oil_directory(&mut state.runtime, child)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("oil.open-parent")
        .map_err(|error| error.to_string())?;

    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .directory_state()
            .ok_or_else(|| "directory state missing".to_owned())?
            .root,
        root
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_action_commands_are_registered_and_execute() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("oil-command-actions");
    std::fs::write(root.join(".hidden"), "hidden\n").map_err(|error| error.to_string())?;

    open_workspace_from_project(&mut state.runtime, "oil-command-actions", &root)?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    for command_name in [
        "oil.open-entry",
        "oil.open-vertical-split",
        "oil.open-horizontal-split",
        "oil.open-new-pane",
        "oil.preview-entry",
        "oil.refresh",
        "oil.close",
        "oil.open-workspace-root",
        "oil.set-root",
        "oil.show-help",
        "oil.cycle-sort",
        "oil.toggle-hidden",
        "oil.toggle-trash",
        "oil.open-external",
        "oil.set-tab-local-root",
    ] {
        assert!(
            state.runtime.commands().contains(command_name),
            "missing command {command_name}"
        );
    }

    state
        .runtime
        .execute_command("oil.toggle-hidden")
        .map_err(|error| error.to_string())?;

    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .directory_state()
            .ok_or_else(|| "directory state missing".to_owned())?
            .show_hidden
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_git_worktree_command_opens_branch_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let remote = unique_temp_dir("oil-worktree-remote");
    let repo = init_git_repo_with_commit("oil-worktree-repo")?;

    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &["remote", "add", "origin", remote.to_str().unwrap_or("")],
    )?;
    run_git_in_dir(&repo, &["push", "-u", "origin", "HEAD:master"])?;
    run_git_in_dir(&repo, &["checkout", "-qb", "feature/oil-worktree"])?;
    std::fs::write(repo.join("feature.txt"), "feature\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "feature.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "feature"])?;
    run_git_in_dir(&repo, &["push", "-u", "origin", "feature/oil-worktree"])?;
    run_git_in_dir(&repo, &["checkout", "-q", "master"])?;

    let workspace_root = repo
        .parent()
        .ok_or_else(|| "repo parent missing".to_owned())?
        .to_path_buf();
    open_workspace_from_project(&mut state.runtime, "oil-worktree", &workspace_root)?;
    open_oil_directory(&mut state.runtime, repo.clone())?;
    state
        .runtime
        .execute_command("oil.git-worktree")
        .map_err(|error| error.to_string())?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "oil.git-worktree did not open picker".to_owned())?;
    assert!(
        picker
            .session()
            .matches()
            .iter()
            .any(|entry| entry.item().label() == "New Branch")
    );
    assert!(
        picker
            .session()
            .matches()
            .iter()
            .any(|entry| entry.item().label() == "origin/feature/oil-worktree")
    );

    let _ = std::fs::remove_dir_all(&repo);
    let _ = std::fs::remove_dir_all(&remote);
    Ok(())
}

#[test]
fn oil_git_worktree_new_branch_prompts_for_name_then_directory() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let remote = unique_temp_dir("oil-worktree-new-remote");
    let repo = init_git_repo_with_commit("oil-worktree-new-repo")?;

    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &["remote", "add", "origin", remote.to_str().unwrap_or("")],
    )?;
    run_git_in_dir(&repo, &["push", "-u", "origin", "HEAD:master"])?;

    let workspace_root = repo
        .parent()
        .ok_or_else(|| "repo parent missing".to_owned())?
        .to_path_buf();
    open_workspace_from_project(&mut state.runtime, "oil-worktree-new", &workspace_root)?;
    open_oil_directory(&mut state.runtime, repo.clone())?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    state
        .runtime
        .execute_command("oil.git-worktree")
        .map_err(|error| error.to_string())?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "oil.git-worktree did not open picker".to_owned())?;
    assert_eq!(
        picker
            .session()
            .selected()
            .map(|entry| entry.item().label()),
        Some("New Branch")
    );

    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;

    assert!(
        shell_ui(&state.runtime)?
            .command_line()
            .is_some_and(|command_line| {
                matches!(
                    command_line.purpose(),
                    CommandLinePurpose::GitWorktreeNewBranch { .. }
                )
            }),
        "New Branch should open the branch-name command line"
    );
    assert!(shell_ui(&state.runtime)?.picker().is_none());

    state
        .handle_text_input("feature/new-oil-branch")
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Return, Mod::NOMOD)
        .map_err(|error| error.to_string())?;

    assert!(shell_ui(&state.runtime)?.command_line().is_none());
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    let pending = shell_buffer(&state.runtime, buffer_id)?
        .directory_state()
        .ok_or_else(|| "directory state missing".to_owned())?
        .pending_worktree
        .clone()
        .ok_or_else(|| "pending worktree request missing".to_owned())?;
    assert_eq!(pending.local_branch, "feature/new-oil-branch");
    assert_eq!(pending.remote_branch, "feature/new-oil-branch");
    assert!(pending.create_new_branch);

    let _ = std::fs::remove_dir_all(&repo);
    let _ = std::fs::remove_dir_all(&remote);
    Ok(())
}

#[test]
fn oil_open_directory_is_scoped_per_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("oil-workspace-first");
    let second_root = unique_temp_dir("oil-workspace-second");

    let first_workspace = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    open_oil_directory(&mut state.runtime, first_root.clone())?;
    let first_buffer_id = active_shell_buffer_id(&state.runtime)?;

    let second_workspace = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    open_oil_directory(&mut state.runtime, second_root.clone())?;
    let second_buffer_id = active_shell_buffer_id(&state.runtime)?;

    assert_ne!(first_workspace, second_workspace);
    assert_ne!(first_buffer_id, second_buffer_id);
    assert_eq!(
        shell_buffer(&state.runtime, first_buffer_id)?
            .directory_state()
            .ok_or_else(|| "first oil directory state missing".to_owned())?
            .root,
        first_root
    );
    assert_eq!(
        shell_buffer(&state.runtime, second_buffer_id)?
            .directory_state()
            .ok_or_else(|| "second oil directory state missing".to_owned())?
            .root,
        second_root
    );

    switch_runtime_workspace(&mut state.runtime, first_workspace)?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, first_buffer_id);

    switch_runtime_workspace(&mut state.runtime, second_workspace)?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, second_buffer_id);

    std::fs::remove_dir_all(&first_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&second_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn terminal_placeholder_lines_describe_shell_launch_not_vertical_slice() {
    let lines = placeholder_lines("*terminal*", &BufferKind::Terminal, &NullUserLibrary);
    let body = lines.join("\n");

    assert!(body.contains("*terminal* is launching the configured shell."));
    assert!(body.contains("Press i to enter terminal input mode"));
    assert!(!body.contains("vertical slice"));
    assert!(!body.contains("compiled terminal package"));
}

#[test]
fn open_workspace_file_routes_png_to_image_buffer() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("open-image-png");
    let path = root.join("sample.png");
    write_test_png(&path)?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let image_state = buffer
        .image_state()
        .ok_or_else(|| "image state missing".to_owned())?;

    assert_eq!(buffer.kind, BufferKind::Image);
    assert_eq!(buffer.path(), Some(path.as_path()));
    assert_eq!(image_state.format, ImageBufferFormat::Raster);
    assert_eq!(image_state.mode, ImageBufferMode::Rendered);
    assert!(buffer.is_read_only());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn open_workspace_file_routes_pdf_to_native_buffer() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("open-pdf");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["hello from page one", "hello from page two"])?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let pdf_state = buffer
        .pdf_state()
        .ok_or_else(|| "pdf state missing".to_owned())?;

    assert_eq!(buffer.kind, BufferKind::Plugin(PDF_BUFFER_KIND.to_owned()));
    assert_eq!(buffer.path(), Some(path.as_path()));
    assert_eq!(pdf_state.page_count(), 2);
    assert_eq!(pdf_state.current_page, 1);
    assert!(buffer.is_read_only());
    assert_eq!(pdf_state.open_mode, PdfOpenMode::Rendered);
    assert!(buffer.pdf_preview_url().is_none());
    assert!(!buffer.has_pdf_preview_surface());
    assert!(
        pdf_state.render_error.is_some() || buffer.image_state().is_some(),
        "rendered mode should either render an image or surface a renderer error"
    );
    assert!(buffer.text.text().contains("hello from page one"));

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn open_workspace_file_honors_markdown_pdf_mode() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(
        HeaderlineTestUserLibrary::with_pdf_open_mode(PdfOpenMode::Markdown),
    );
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let root = unique_temp_dir("open-pdf-markdown");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["hello from page one", "hello from page two"])?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let pdf_state = buffer
        .pdf_state()
        .ok_or_else(|| "pdf state missing".to_owned())?;

    assert_eq!(pdf_state.open_mode, PdfOpenMode::Markdown);
    assert_eq!(buffer.language_id(), Some("markdown"));
    assert!(buffer.image_state().is_none());
    assert!(buffer.text.text().contains("## Page 1"));
    assert!(buffer.text.text().contains("## Page 2"));

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn pdf_buffers_support_navigation_editing_and_save() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("edit-pdf");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["first page", "second page"])?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    pdf_next_page(&mut state.runtime)?;
    pdf_rotate_clockwise(&mut state.runtime)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let pdf_state = buffer
            .pdf_state()
            .ok_or_else(|| "pdf state missing".to_owned())?;
        assert_eq!(pdf_state.current_page, 2);
        assert!(pdf_state.dirty);
        assert!(buffer.text.text().contains("second page"));
    }

    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    save_buffer(&mut state.runtime, workspace_id, buffer_id)?;
    {
        let saved = lopdf::Document::load(&path).map_err(|error| error.to_string())?;
        let rotation = pdf_page_rotation(&saved, 2).unwrap_or_default();
        assert_eq!(rotation.rem_euclid(360), 90);
    }

    pdf_delete_page(&mut state.runtime)?;
    save_buffer(&mut state.runtime, workspace_id, buffer_id)?;
    {
        let saved = lopdf::Document::load(&path).map_err(|error| error.to_string())?;
        assert_eq!(saved.get_pages().len(), 1);
    }

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn buffer_save_command_writes_edited_file_buffer_to_disk() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("buffer-save-command");
    let path = root.join("sample.txt");
    std::fs::write(&path, "alpha\n").map_err(|error| error.to_string())?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nalpha\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_cycle_skips_non_default_workspace_without_project_root() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-cycle-first");
    let second_root = unique_temp_dir("workspace-cycle-second");
    let first = open_workspace_from_project(&mut state.runtime, "first", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "second", &second_root)?;
    let window_id = active_window_id(&state.runtime)?;
    let rootless = state
        .runtime
        .model_mut()
        .open_workspace(window_id, "rootless", None)
        .map_err(|error| error.to_string())?;

    let cycle_ids = open_project_workspace_ids(&state.runtime)?;
    assert_eq!(cycle_ids, vec![first, second]);
    assert!(!cycle_ids.contains(&rootless));

    switch_runtime_workspace(&mut state.runtime, first)?;
    state
        .runtime
        .execute_command("workspace.next")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    state
        .runtime
        .execute_command("workspace.next")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), first);

    std::fs::remove_dir_all(&first_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&second_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_mark_and_unmark_commands_persist_active_project_root() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-mark-state");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;
    let project_root = unique_temp_dir("workspace-mark-project");
    let canonical_root = canonicalize_project_root_path(&project_root);
    open_workspace_from_project(&mut state.runtime, "marked-project", &project_root)?;

    state
        .runtime
        .execute_command("workspace.mark")
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .execute_command("workspace.mark")
        .map_err(|error| error.to_string())?;

    let persisted = std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?;
    assert_eq!(persisted, format!("{}\n", canonical_root.display()));
    assert!(
        !persisted.contains(r"\\?\"),
        "Mark List must store stripped canonical roots, got {persisted}"
    );

    state
        .runtime
        .execute_command("workspace.unmark")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?,
        ""
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&project_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn find_workspace_by_root_matches_normalized_path_identity() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let project_root = unique_temp_dir("workspace-root-identity");
    let workspace_id =
        open_workspace_from_project(&mut state.runtime, "identity-project", &project_root)?;
    let verbatim = PathBuf::from(format!(r"\\?\{}", project_root.display()));

    assert_eq!(
        find_workspace_by_root(&state.runtime, &verbatim)?,
        Some(workspace_id)
    );

    std::fs::remove_dir_all(&project_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn mark_list_load_canonicalizes_existing_roots_and_keeps_missing_as_written() -> Result<(), String>
{
    let state_dir = unique_temp_dir("workspace-mark-load-normalize");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    let existing = unique_temp_dir("workspace-mark-load-existing");
    let missing = state_dir.join("missing-on-disk");
    let verbatim_existing = PathBuf::from(format!(r"\\?\{}", existing.display()));
    std::fs::write(
        &mark_list_path,
        format!("{}\n{}\n", verbatim_existing.display(), missing.display()),
    )
    .map_err(|error| error.to_string())?;

    let loaded = MarkListState::load(mark_list_path)?;
    assert_eq!(
        loaded.list.roots(),
        &[canonicalize_project_root_path(&existing), missing.clone(),]
    );
    assert!(
        !loaded.list.roots()[0].to_string_lossy().contains(r"\\?\"),
        "existing Mark List roots must strip verbatim prefixes"
    );

    let missing_verbatim =
        PathBuf::from(format!(r"\\?\{}", state_dir.join("also-missing").display()));
    let with_missing_verbatim =
        mark_list_from_persisted_text(&format!("{}\n", missing_verbatim.display()));
    assert_eq!(
        with_missing_verbatim.roots(),
        &[missing_verbatim],
        "missing paths must stay as-written when canonicalize cannot run"
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&existing).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_mark_refreshes_clean_open_mark_list_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-mark-open-list");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;
    let project_root = unique_temp_dir("workspace-mark-open-project");
    let canonical_root = canonicalize_project_root_path(&project_root);
    open_workspace_from_project(&mut state.runtime, "marked-project", &project_root)?;
    state
        .runtime
        .execute_command("workspace.marks")
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("workspace.mark")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.text.text(),
        format!("{}\n", canonical_root.display())
    );
    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?,
        format!("{}\n", canonical_root.display())
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&project_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_mark_on_default_workspace_notifies_without_mutating_list() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-mark-default");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;

    state
        .runtime
        .execute_command("workspace.mark")
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .execute_command("workspace.unmark")
        .map_err(|error| error.to_string())?;

    assert!(!mark_list_path.exists());
    assert!(mark_list_state(&state.runtime)?.list.roots().is_empty());
    let notifications = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
    assert!(
        notifications
            .iter()
            .any(|notification| notification.title == "Default Workspace has no project root")
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_marks_opens_real_file_and_save_reloads_normalized_list() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-marks-open");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    std::fs::write(&mark_list_path, "P:\\alpha\n").map_err(|error| error.to_string())?;
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;

    state
        .runtime
        .execute_command("workspace.marks")
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.path(),
        Some(mark_list_path.as_path())
    );
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let end = buffer.text.point_from_char_index(buffer.text.char_count());
        buffer.replace_range(
            TextRange::new(TextPoint::default(), end),
            "P:\\beta\n\n  \nP:\\gamma\n",
        );
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?,
        "P:\\beta\nP:\\gamma\n"
    );
    assert_eq!(
        mark_list_state(&state.runtime)?.list.roots(),
        &[PathBuf::from(r"P:\beta"), PathBuf::from(r"P:\gamma")]
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_marked_slot_jump_switches_open_opens_closed_and_handles_empty_missing()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-marked-jump-state");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;

    let open_root = unique_temp_dir("workspace-marked-jump-open");
    let closed_root = unique_temp_dir("workspace-marked-jump-closed");
    let missing_root = state_dir.join("missing-marked-workspace");
    open_workspace_from_project(&mut state.runtime, "open-project", &open_root)?;
    let open_workspace_id = shell_ui(&state.runtime)?.active_workspace();

    {
        let list = &mut mark_list_state_mut(&mut state.runtime)?.list;
        assert!(list.mark(&open_root));
        assert!(list.mark(&closed_root));
        assert!(list.mark(&missing_root));
    }
    persist_mark_list(mark_list_state(&state.runtime)?)?;

    let default_workspace = shell_ui(&state.runtime)?.default_workspace();
    switch_runtime_workspace(&mut state.runtime, default_workspace)?;
    assert_ne!(
        shell_ui(&state.runtime)?.active_workspace(),
        open_workspace_id
    );

    state
        .runtime
        .execute_command("workspace.marked-1")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        open_workspace_id
    );

    state
        .runtime
        .execute_command("workspace.marked-2")
        .map_err(|error| error.to_string())?;
    let closed_workspace_id = shell_ui(&state.runtime)?.active_workspace();
    assert_ne!(closed_workspace_id, open_workspace_id);
    assert_ne!(closed_workspace_id, default_workspace);
    assert_eq!(
        state
            .runtime
            .model()
            .workspace(closed_workspace_id)
            .map_err(|error| error.to_string())?
            .root(),
        Some(closed_root.as_path())
    );

    let before_missing = mark_list_state(&state.runtime)?.list.roots().to_vec();
    state
        .runtime
        .execute_command("workspace.marked-3")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        mark_list_state(&state.runtime)?.list.roots(),
        before_missing.as_slice()
    );
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        closed_workspace_id
    );
    let notifications = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
    assert!(
        notifications
            .iter()
            .any(|notification| notification.title == "Marked Workspace path missing")
    );

    switch_runtime_workspace(&mut state.runtime, open_workspace_id)?;
    state
        .runtime
        .execute_command("workspace.marked-4")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        open_workspace_id
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&open_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&closed_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_save_command_writes_all_dirty_workspace_files() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-save-command");
    let first = root.join("first.txt");
    let second = root.join("second.txt");
    std::fs::write(&first, "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "beta\n").map_err(|error| error.to_string())?;

    let first_buffer_id = open_workspace_file(&mut state.runtime, &first)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
        assert!(buffer.is_dirty());
    }

    let second_buffer_id = open_workspace_file(&mut state.runtime, &second)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, second_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("two\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("workspace.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );
    assert_eq!(
        std::fs::read_to_string(&second).map_err(|error| error.to_string())?,
        "two\nbeta\n"
    );
    assert!(!shell_buffer(&state.runtime, first_buffer_id)?.is_dirty());
    assert!(!shell_buffer(&state.runtime, second_buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale() -> Result<(), String>
{
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("buffer-save-stale-focus");
    let first = root.join("first.txt");
    let second = root.join("second.txt");
    std::fs::write(&first, "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "beta\n").map_err(|error| error.to_string())?;

    let first_buffer_id = open_workspace_file(&mut state.runtime, &first)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
    }

    let second_buffer_id = open_workspace_file(&mut state.runtime, &second)?;
    assert_ne!(first_buffer_id, second_buffer_id);

    shell_ui_mut(&mut state.runtime)?.focus_buffer(first_buffer_id);
    assert_eq!(
        shell_ui(&state.runtime)?.active_buffer_id(),
        Some(first_buffer_id)
    );

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn buffer_save_hook_prefers_explicit_event_buffer_over_shell_focus() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("buffer-save-explicit-buffer");
    let first = root.join("first.txt");
    let second = root.join("second.txt");
    std::fs::write(&first, "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "beta\n").map_err(|error| error.to_string())?;

    let first_buffer_id = open_workspace_file(&mut state.runtime, &first)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
    }

    let second_buffer_id = open_workspace_file(&mut state.runtime, &second)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, second_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("two\n");
    }

    shell_ui_mut(&mut state.runtime)?.focus_buffer(second_buffer_id);
    let workspace_id = shell_ui(&state.runtime)?.active_workspace();

    state
        .runtime
        .emit_hook(
            HOOK_BUFFER_SAVE,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_buffer(first_buffer_id),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );
    assert_eq!(
        std::fs::read_to_string(&second).map_err(|error| error.to_string())?,
        "beta\n"
    );
    assert!(!shell_buffer(&state.runtime, first_buffer_id)?.is_dirty());
    assert!(shell_buffer(&state.runtime, second_buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_save_command_uses_shell_active_workspace_when_runtime_workspace_is_stale()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-save-stale-a");
    let second_root = unique_temp_dir("workspace-save-stale-b");
    let first_workspace = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let first_path = first_root.join("alpha.txt");
    std::fs::write(&first_path, "alpha\n").map_err(|error| error.to_string())?;
    let first_buffer_id = open_workspace_file(&mut state.runtime, &first_path)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
    }

    let second_workspace = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    assert_ne!(first_workspace, second_workspace);
    shell_ui_mut(&mut state.runtime)?.switch_workspace(first_workspace);
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        first_workspace
    );

    state
        .runtime
        .execute_command("workspace.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first_path).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );

    std::fs::remove_dir_all(&first_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&second_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_save_hook_prefers_explicit_event_workspace_over_shell_focus() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-save-explicit-a");
    let second_root = unique_temp_dir("workspace-save-explicit-b");

    let first_workspace = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let first_path = first_root.join("alpha.txt");
    std::fs::write(&first_path, "alpha\n").map_err(|error| error.to_string())?;
    let first_buffer_id = open_workspace_file(&mut state.runtime, &first_path)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
    }

    let second_workspace = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    let second_path = second_root.join("beta.txt");
    std::fs::write(&second_path, "beta\n").map_err(|error| error.to_string())?;
    let second_buffer_id = open_workspace_file(&mut state.runtime, &second_path)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, second_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("two\n");
    }

    shell_ui_mut(&mut state.runtime)?.switch_workspace(second_workspace);

    state
        .runtime
        .emit_hook(
            HOOK_WORKSPACE_SAVE,
            HookEvent::new().with_workspace(first_workspace),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first_path).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );
    assert_eq!(
        std::fs::read_to_string(&second_path).map_err(|error| error.to_string())?,
        "beta\n"
    );
    assert!(!shell_buffer(&state.runtime, first_buffer_id)?.is_dirty());
    assert!(shell_buffer(&state.runtime, second_buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&first_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&second_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn picker_open_file_save_clears_dirty_state_and_closes_cleanly() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("picker-open-file-save");
    let path = root.join("sample.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "picker-save", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Files",
        vec![PickerEntry {
            item: PickerItem::new(
                path.display().to_string(),
                "sample.rs",
                "workspace root",
                Some(path.display().to_string()),
            ),
            action: PickerAction::OpenFile(path.clone()),
            quickfix: None,
        }],
    ));

    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;

    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.path(),
        Some(path.as_path())
    );

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nfn main() {}\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    close_buffer_with_prompt(&mut state.runtime, buffer_id)?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(shell_ui(&state.runtime)?.buffer(buffer_id).is_none());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn picker_open_file_location_save_clears_dirty_state_and_closes_cleanly() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("picker-open-location-save");
    let path = root.join("mod.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "picker-location-save", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Search",
        vec![PickerEntry {
            item: PickerItem::new(
                format!("{}:1:1", path.display()),
                "fn main() {}",
                "mod.rs | Ln 1, Col 1",
                Some(path.display().to_string()),
            ),
            action: PickerAction::OpenFileLocation {
                path: path.clone(),
                target: TextPoint::new(0, 0),
            },
            quickfix: None,
        }],
    ));

    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;

    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.path(),
        Some(path.as_path())
    );

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nfn main() {}\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    close_buffer_with_prompt(&mut state.runtime, buffer_id)?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(shell_ui(&state.runtime)?.buffer(buffer_id).is_none());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn buffer_save_still_writes_when_format_on_save_fails() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("save-format-failure");
    let path = root.join("mod.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "format-failure", &root)?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    state
        .runtime
        .services_mut()
        .insert(Arc::new(LspClientManager::new(
            LanguageServerRegistry::new(),
        )));
    state
        .runtime
        .services_mut()
        .insert(FormatterRegistry::default());
    formatter_registry_mut(&mut state.runtime)?.register(FormatterSpec {
        language_id: "rust".to_owned(),
        program: "definitely-missing-formatter".to_owned(),
        args: Vec::new(),
    })?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nfn main() {}\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn pdf_buffers_reload_when_backing_file_changes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("reload-pdf");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["before reload", "second page"])?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    write_test_pdf(&path, &["after reload"])?;

    let reloaded = {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.mark_backing_file_reload_pending();
        buffer.reload_from_disk_if_changed(true)?
    };
    assert!(reloaded);

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let pdf_state = buffer
        .pdf_state()
        .ok_or_else(|| "pdf state missing".to_owned())?;
    assert_eq!(pdf_state.page_count(), 1);
    assert!(!buffer.has_pdf_preview_surface());
    assert!(buffer.text.text().contains("after reload"));

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn svg_image_buffers_toggle_between_rendered_and_source_modes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("open-image-svg");
    let path = root.join("sample.svg");
    write_test_svg(&path)?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let image_state = buffer
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?;
        assert_eq!(buffer.kind, BufferKind::Image);
        assert_eq!(image_state.format, ImageBufferFormat::Svg);
        assert_eq!(image_state.mode, ImageBufferMode::Rendered);
        assert!(buffer.is_read_only());
    }

    toggle_active_image_buffer_mode(&mut state.runtime)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        assert!(buffer.is_svg_source_mode());
        assert!(buffer.supports_text_file_actions());
        assert!(!buffer.is_read_only());
        assert!(buffer.text.text().contains("<svg"));
    }

    toggle_active_image_buffer_mode(&mut state.runtime)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let image_state = buffer
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?;
        assert_eq!(image_state.mode, ImageBufferMode::Rendered);
        assert!(buffer.is_read_only());
    }

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn image_zoom_controls_adjust_zoom_multiplier() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("image-zoom");
    let path = root.join("sample.png");
    write_test_png(&path)?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?
            .zoom,
        1.0
    );

    zoom_active_image_buffer_in(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?
            .zoom,
        IMAGE_ZOOM_STEP
    );

    zoom_active_image_buffer_out(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?
            .zoom,
        1.0
    );

    zoom_active_image_buffer_in(&mut state.runtime)?;
    reset_active_image_buffer_zoom(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?
            .zoom,
        1.0
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn file_reload_notifications_target_only_matching_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-targeted");
    let active_path = root.join("src").join("main.rs");
    let hidden_path = root.join("src").join("lib.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&active_path, "fn main() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&hidden_path, "pub fn hidden() {}\n").map_err(|error| error.to_string())?;

    let (active_buffer_id, hidden_buffer_id) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, active_buffer_id, &active_path)?;
    configure_file_buffer(&mut state, hidden_buffer_id, &hidden_path)?;

    std::fs::write(
        &hidden_path,
        "pub fn hidden() {\n    println!(\"disk\");\n}\n",
    )
    .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &hidden_path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false
    )?);
    wait_for_file_reload_worker(&mut state, &[hidden_buffer_id])?;
    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, active_buffer_id)?.text.line(1),
        None
    );
    assert_eq!(
        shell_buffer(&state.runtime, hidden_buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"disk\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn file_reload_notifications_reload_hidden_buffers_without_focus_changes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-hidden");
    let active_path = root.join("src").join("main.rs");
    let hidden_path = root.join("src").join("lib.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&active_path, "fn main() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&hidden_path, "pub fn hidden() {}\n").map_err(|error| error.to_string())?;

    let (active_buffer_id, hidden_buffer_id) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, active_buffer_id, &active_path)?;
    configure_file_buffer(&mut state, hidden_buffer_id, &hidden_path)?;

    std::fs::write(
        &hidden_path,
        "pub fn hidden() {\n    println!(\"background\");\n}\n",
    )
    .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &hidden_path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false,
    )?);
    wait_for_file_reload_worker(&mut state, &[hidden_buffer_id])?;
    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, hidden_buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"background\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn file_reload_notifications_wait_for_dirty_buffers_to_become_clean() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-dirty");
    let path = root.join("src").join("main.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;

    let (buffer_id, _) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, buffer_id, &path)?;

    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
    }
    std::fs::write(&path, "fn main() {\n    println!(\"disk\");\n}\n")
        .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false,
    )?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(0)
            .as_deref(),
        Some("// local")
    );

    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        assert!(buffer.text.undo());
        assert!(!buffer.text.is_dirty());
    }

    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"disk\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn lsp_log_buffer_name_includes_server_name() {
    assert_eq!(lsp_log_buffer_name("csharp-ls"), "*lsp-log csharp-ls*");
}

#[test]
fn lsp_log_buffer_lines_only_include_entries_for_requested_server() {
    let entries = vec![
        LspLogEntry::new(LspLogDirection::Outgoing, "csharp-ls", "{\"id\":1}"),
        LspLogEntry::new(LspLogDirection::Incoming, "rust-analyzer", "{\"id\":2}"),
    ];
    let filtered = lsp_log_entries_for_server(&entries, "csharp-ls");
    let lines = lsp_log_buffer_lines("csharp-ls", &filtered);
    let body = lines.join("\n");

    assert!(body.contains("*lsp-log csharp-ls* captures live JSON-RPC traffic for `csharp-ls`."));
    assert!(body.contains("OUT csharp-ls"));
    assert!(!body.contains("rust-analyzer"));
}

#[test]
fn errors_buffer_updates_stay_in_the_background() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let active_before = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing".to_owned())?;

    assert_ne!(active_before.2, "*errors*");
    record_runtime_error(&mut state.runtime, "test.error", "boom");

    let active_after = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after logging".to_owned())?;
    assert_eq!(active_after.0, active_before.0);
    assert_eq!(active_after.1, active_before.1);
    assert_eq!(active_after.2, active_before.2);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, active_before.1);
    Ok(())
}

#[test]
fn lsp_log_buffers_stay_in_the_background_until_explicitly_focused() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let active_before = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing".to_owned())?;

    let buffer_id = ensure_lsp_log_buffer(&mut state.runtime, workspace_id, "rust-analyzer")?;
    let active_after_creation = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after creating log buffer".to_owned())?;

    assert_eq!(active_after_creation.0, active_before.0);
    assert_eq!(active_after_creation.1, active_before.1);
    assert_eq!(active_after_creation.2, active_before.2);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, active_before.1);

    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)?;

    let active_after_focus = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after focusing log buffer".to_owned())?;
    assert_eq!(active_after_focus.1, buffer_id);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);
    Ok(())
}

fn install_test_lsp_manager(
    runtime: &mut EditorRuntime,
    server_ids: &[&str],
) -> Result<Arc<LspClientManager>, String> {
    let mut registry = LanguageServerRegistry::new();
    for server_id in server_ids {
        registry
            .register(editor_lsp::LanguageServerSpec::new(
                *server_id,
                "rust",
                ["rs"],
                "dummy-lsp",
                std::iter::empty::<&str>(),
            ))
            .map_err(|error| error.to_string())?;
    }
    let manager = Arc::new(LspClientManager::new(registry));
    runtime.services_mut().insert(Arc::clone(&manager));
    Ok(manager)
}

fn install_lsp_enabled_file_buffer(
    state: &mut ShellState,
    name: &str,
    path: &Path,
    lines: Vec<String>,
) -> Result<BufferId, String> {
    let buffer_id = install_text_test_buffer(state, name, lines)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_path(path.to_path_buf());
        buffer.set_lsp_enabled(true);
        buffer.set_lsp_path(Some(path.to_path_buf()));
    }
    Ok(buffer_id)
}

fn sample_lsp_diagnostic(message: &str) -> Diagnostic {
    Diagnostic::new(
        "rustc",
        message,
        DiagnosticSeverity::Error,
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 4)),
    )
}

#[test]
fn apply_pending_lsp_state_skips_diagnostic_lookups_when_generation_unchanged() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let path = PathBuf::from("src").join("main.rs");
    let buffer_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-diag-skip*",
        &path,
        vec!["fn main() {}".to_owned()],
    )?;
    manager
        .attach_memory_session(
            "rust-analyzer",
            &path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;

    apply_pending_lsp_state(&mut state.runtime)?;
    let after_publish = shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision();
    assert_eq!(after_publish, 1);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics()[0].message(),
        "cannot find value `missing`"
    );
    let lookups_after_publish = manager.diagnostics_for_path_lookups();
    assert!(
        lookups_after_publish >= 1,
        "first apply should look up diagnostics"
    );

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision(),
        after_publish
    );
    assert_eq!(
        manager.diagnostics_for_path_lookups(),
        lookups_after_publish,
        "unchanged diagnostics generation must not clone diagnostics again"
    );

    manager
        .apply_published_diagnostics(&path, vec![sample_lsp_diagnostic("unused variable")])
        .map_err(|error| error.to_string())?;
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision(),
        after_publish + 1
    );
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics()[0].message(),
        "unused variable"
    );
    Ok(())
}

#[test]
fn apply_pending_lsp_state_refreshes_only_paths_whose_diagnostics_changed() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let main_path = PathBuf::from("src").join("main.rs");
    let lib_path = PathBuf::from("src").join("lib.rs");
    let main_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-diag-main*",
        &main_path,
        vec!["fn main() {}".to_owned()],
    )?;
    let lib_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-diag-lib*",
        &lib_path,
        vec!["pub fn lib() {}".to_owned()],
    )?;
    manager
        .attach_memory_session(
            "rust-analyzer",
            &main_path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;
    manager
        .attach_memory_session("rust-analyzer", &lib_path, Vec::new())
        .map_err(|error| error.to_string())?;
    let _ = manager.take_dirty_diagnostic_paths();
    manager
        .apply_published_diagnostics(
            &main_path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(manager.diagnostics_for_path_lookups(), 1);
    assert_eq!(
        shell_buffer(&state.runtime, main_id)?.lsp_diagnostics_revision(),
        1
    );
    assert_eq!(
        shell_buffer(&state.runtime, lib_id)?.lsp_diagnostics_revision(),
        0
    );

    manager
        .apply_published_diagnostics(&lib_path, vec![sample_lsp_diagnostic("unused variable")])
        .map_err(|error| error.to_string())?;
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(manager.diagnostics_for_path_lookups(), 2);
    assert_eq!(
        shell_buffer(&state.runtime, main_id)?.lsp_diagnostics_revision(),
        1
    );
    assert_eq!(
        shell_buffer(&state.runtime, lib_id)?.lsp_diagnostics_revision(),
        1
    );
    assert_eq!(
        shell_buffer(&state.runtime, lib_id)?.lsp_diagnostics()[0].message(),
        "unused variable"
    );
    Ok(())
}

#[test]
fn apply_pending_lsp_state_clears_diagnostics_after_session_disconnect() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let path = PathBuf::from("src").join("main.rs");
    let buffer_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-diag-disconnect*",
        &path,
        vec!["fn main() {}".to_owned()],
    )?;
    manager
        .attach_memory_session(
            "rust-analyzer",
            &path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision(),
        1
    );

    manager
        .disconnect_memory_sessions_for_path(&path)
        .map_err(|error| error.to_string())?;
    apply_pending_lsp_state(&mut state.runtime)?;
    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .lsp_diagnostics()
            .is_empty()
    );
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision(),
        2
    );
    Ok(())
}

#[test]
fn apply_pending_lsp_state_skips_log_snapshot_until_revision_moves() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = ensure_lsp_log_buffer(&mut state.runtime, workspace_id, "rust-analyzer")?;
    apply_pending_lsp_state(&mut state.runtime)?;
    let before = shell_buffer(&state.runtime, buffer_id)?.text.text();

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.text.text(), before);

    manager.record_transport_log_event("rust-analyzer", "started language server");
    apply_pending_lsp_state(&mut state.runtime)?;
    let after = shell_buffer(&state.runtime, buffer_id)?.text.text();
    assert_ne!(after, before);
    assert!(after.contains("started language server"));

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.text.text(), after);
    Ok(())
}

#[test]
fn apply_pending_lsp_state_toasts_only_when_notification_revision_moves() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    apply_pending_lsp_state(&mut state.runtime)?;
    let before = shell_ui(&state.runtime)?.notification_revision();

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);

    manager.record_show_message("rust-analyzer", "Indexing");
    apply_pending_lsp_state(&mut state.runtime)?;
    let after = shell_ui(&state.runtime)?.notification_revision();
    assert!(after > before);
    let now = Instant::now();
    assert!(
        shell_ui(&state.runtime)?
            .visible_notifications(now)
            .iter()
            .any(|notification| notification.title.contains("rust-analyzer")
                && notification
                    .body_lines
                    .iter()
                    .any(|line| line.contains("Indexing")))
    );

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), after);
    Ok(())
}

#[test]
fn apply_pending_lsp_state_refreshes_attached_server_label_when_session_set_changes()
-> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer", "biome"])?;
    let rust_path = PathBuf::from("src").join("main.rs");
    let biome_path = PathBuf::from("src").join("lib.rs");
    let rust_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-label-rust*",
        &rust_path,
        vec!["fn main() {}".to_owned()],
    )?;
    let biome_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-label-biome*",
        &biome_path,
        vec!["pub fn lib() {}".to_owned()],
    )?;
    manager
        .attach_memory_session("rust-analyzer", &rust_path, Vec::new())
        .map_err(|error| error.to_string())?;
    manager
        .attach_memory_session("biome", &biome_path, Vec::new())
        .map_err(|error| error.to_string())?;

    shell_ui_mut(&mut state.runtime)?.focus_buffer(rust_id);
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_ui(&state.runtime)?.attached_lsp_server(),
        Some("rust-analyzer")
    );

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_ui(&state.runtime)?.attached_lsp_server(),
        Some("rust-analyzer")
    );

    shell_ui_mut(&mut state.runtime)?.focus_buffer(biome_id);
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_ui(&state.runtime)?.attached_lsp_server(),
        Some("biome")
    );
    Ok(())
}

#[test]
fn apply_pending_lsp_state_does_nothing_without_lsp_enabled_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let path = PathBuf::from("src").join("main.rs");
    manager
        .attach_memory_session(
            "rust-analyzer",
            &path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        for buffer in &mut ui.buffers {
            buffer.set_lsp_enabled(false);
        }
    }

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(manager.diagnostics_for_path_lookups(), 0);
    Ok(())
}

#[test]
fn saved_theme_selection_round_trips() {
    let dir = unique_temp_dir("theme-save");
    let path = dir.join("active-theme.txt");
    write_saved_theme_selection(&path, "volt-dark")
        .unwrap_or_else(|error| panic!("unexpected save error: {error}"));

    assert_eq!(
        read_saved_theme_selection(&path)
            .unwrap_or_else(|error| panic!("unexpected read error: {error}")),
        Some("volt-dark".to_owned())
    );

    std::fs::remove_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to remove temp dir `{}`: {error}", dir.display()));
}

#[test]
fn restore_saved_theme_selection_activates_saved_theme() {
    let dir = unique_temp_dir("theme-restore");
    let path = dir.join("active-theme.txt");
    write_saved_theme_selection(&path, "amber")
        .unwrap_or_else(|error| panic!("unexpected save error: {error}"));

    let mut registry = ThemeRegistry::new();
    registry
        .register(editor_theme::Theme::new("volt-dark", "Volt Dark"))
        .unwrap_or_else(|error| panic!("unexpected register error: {error}"));
    registry
        .register(editor_theme::Theme::new("amber", "Amber"))
        .unwrap_or_else(|error| panic!("unexpected register error: {error}"));

    restore_saved_theme_selection(&mut registry, &path)
        .unwrap_or_else(|error| panic!("unexpected restore error: {error}"));

    assert_eq!(
        registry.active_theme().map(|theme| theme.id()),
        Some("amber")
    );

    std::fs::remove_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to remove temp dir `{}`: {error}", dir.display()));
}

#[test]
fn restore_saved_theme_selection_clears_unknown_theme() {
    let dir = unique_temp_dir("theme-stale");
    let path = dir.join("active-theme.txt");
    write_saved_theme_selection(&path, "missing-theme")
        .unwrap_or_else(|error| panic!("unexpected save error: {error}"));

    let mut registry = ThemeRegistry::new();
    registry
        .register(editor_theme::Theme::new("gruvbox-dark", "Gruvbox Dark"))
        .unwrap_or_else(|error| panic!("unexpected register error: {error}"));

    let error = restore_saved_theme_selection(&mut registry, &path)
        .expect_err("unknown saved theme should surface an error");
    assert!(error.contains("missing-theme"));
    assert!(!path.exists());
    assert_eq!(
        registry.active_theme().map(|theme| theme.id()),
        Some("gruvbox-dark")
    );

    std::fs::remove_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to remove temp dir `{}`: {error}", dir.display()));
}

#[test]
fn draw_buffer_text_keeps_cursor_line_as_one_text_run() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "abc";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: 3,
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "abc".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_expands_tabs_to_spaces() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "\tcargo";
    let char_map = LineCharMap::with_tab_width(line, 4);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "    cargo".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn line_char_map_treats_variation_selectors_as_zero_width() {
    let line = "⚛️x";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.display_cols_between(0, line.chars().count()), 3);
    assert_eq!(char_map.display_text_for_range(line, 0, 2), "⚛");
    assert_eq!(
        char_map.display_text_for_range(line, 0, line.chars().count()),
        "⚛x"
    );
}

#[test]
fn line_char_map_treats_byte_order_marks_as_zero_width() {
    let line = "\u{feff}<Project";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.display_cols_between(0, line.chars().count()), 8);
    assert_eq!(
        char_map.display_text_for_range(line, 0, line.chars().count()),
        "<Project"
    );
}

#[test]
fn line_char_map_renders_escape_as_caret_notation() {
    let line = "\u{1b}[31m";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.display_cols_between(0, line.chars().count()), 6);
    assert_eq!(
        char_map.display_text_for_range(line, 0, line.chars().count()),
        "^[[31m"
    );
}

#[test]
fn line_char_map_cursor_anchor_skips_variation_selectors() {
    let line = "⚛️x";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.cursor_anchor_col(0), 0);
    assert_eq!(char_map.cursor_anchor_col(1), 0);
    assert_eq!(char_map.cursor_anchor_col(2), 2);
}

#[test]
fn line_char_map_treats_emoji_as_double_width() {
    let line = "🙂x";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.display_cols_between(0, 1), 2);
    assert_eq!(char_map.display_cols_between(0, line.chars().count()), 3);
    assert_eq!(char_map.char_col_for_display_col(0), 0);
    assert_eq!(char_map.char_col_for_display_col(1), 0);
    assert_eq!(char_map.char_col_for_display_col(2), 1);
}

#[test]
fn draw_buffer_text_omits_variation_selectors_from_scene_text() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "⚛️";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "⚛".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_renders_escape_controls_as_caret_notation() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "\u{1b}[31mSet-PSReadLineOption";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "^[[31mSet-PSReadLineOption".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_omits_byte_order_mark_from_scene_text() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "\u{feff}<Project";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "<Project".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_skips_lines_that_only_contain_byte_order_marks() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "\u{feff}";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.is_empty());
    Ok(())
}

#[test]
fn monospace_text_width_ignores_variation_selectors() {
    assert_eq!(monospace_text_width("⚛️", 8), 8);
}

#[test]
fn draw_buffer_text_keeps_git_status_segments_aligned_with_icon_prefix() -> Result<(), String> {
    let line = SectionRenderLine {
        text: format!(
            "{} Head: master f9d8c15 Added some more keybinds",
            editor_icons::symbols::dev::DEV_GIT_BRANCH
        ),
        depth: 1,
        section_id: GIT_SECTION_HEADERS.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);
    let char_map = LineCharMap::new(&formatted);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line: &formatted,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: formatted.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: Some(&spans),
            default_color: Color::RGB(240, 240, 240),
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    let text_segments = scene
        .into_iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, .. } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        text_segments,
        vec![
            "  ".to_owned(),
            editor_icons::symbols::dev::DEV_GIT_BRANCH.to_owned(),
            " ".to_owned(),
            "Head:".to_owned(),
            " ".to_owned(),
            "master".to_owned(),
            " ".to_owned(),
            "f9d8c15".to_owned(),
            " ".to_owned(),
            "Added some more keybinds".to_owned(),
        ]
    );
    Ok(())
}

#[test]
fn draw_line_ghost_text_for_segment_draws_after_the_last_visible_column() -> Result<(), String> {
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let char_map = LineCharMap::new("a");

    draw_line_ghost_text_for_segment(
        &mut target,
        GhostTextSegmentDraw {
            x: 24,
            y: 8,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: 1,
            },
            char_map: &char_map,
            line_len: 1,
            ghost_text: Some(" render(value: usize)"),
            color: Color::RGB(140, 144, 152),
            cell_width: 8,
        },
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 40,
            y: 8,
            text: " render(value: usize)".to_owned(),
            color: to_render_color(Color::RGB(140, 144, 152)),
        }]
    );
    Ok(())
}

#[test]
fn draw_line_ghost_text_for_segment_skips_non_terminal_wrap_segments() -> Result<(), String> {
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let char_map = LineCharMap::new("alpha beta");

    draw_line_ghost_text_for_segment(
        &mut target,
        GhostTextSegmentDraw {
            x: 0,
            y: 0,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: 10,
            },
            char_map: &char_map,
            line_len: 24,
            ghost_text: Some("hidden"),
            color: Color::RGB(140, 144, 152),
            cell_width: 8,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.is_empty());
    Ok(())
}

#[test]
fn visible_headerline_lines_keeps_innermost_contexts_when_space_is_limited() {
    let lines = [
        "module app".to_owned(),
        "impl Demo".to_owned(),
        "render(value: usize)".to_owned(),
    ];
    assert_eq!(
        visible_headerline_lines(&lines, 3),
        vec!["impl Demo", "render(value: usize)"]
    );
}

#[test]
fn visible_headerline_lines_reserves_at_least_one_buffer_row() {
    let lines = ["render()".to_owned()];
    assert!(visible_headerline_lines(&lines, 1).is_empty());
    assert_eq!(visible_headerline_row_count(&lines, 1), 0);
}

#[test]
fn render_buffer_headerline_reserves_rows_above_buffer_body() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary::default();
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline-scrolloff*",
        vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.scroll_row = 1;
        buffer.set_cursor(TextPoint::new(1, 1));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: state.runtime.services().get::<ThemeRegistry>(),
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. } if *y == layout.body_y + 16 && text == "beta"
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. } if *y == layout.body_y && text == "beta"
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_keeps_cursor_below_sticky_row() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary::default();
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline*",
        vec!["abcdefghijklmnopqrstuvwxyz".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 25));

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let cursor_color = to_render_color(Color::RGB(110, 170, 255));
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y + 16 && text == "abcdefghijklmnopqrstuvwxyz"
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y && text == "fn render(value: usize)"
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.y == layout.body_y + 16 && *color == cursor_color
    )));
    let headerline_index = scene
        .iter()
        .position(|command| {
            matches!(
                command,
                DrawCommand::Text { y, text, .. }
                    if *y == layout.body_y && text == "fn render(value: usize)"
            )
        })
        .ok_or_else(|| "missing headerline draw".to_owned())?;
    let cursor_index = scene
        .iter()
        .position(|command| {
            matches!(
                command,
                DrawCommand::FillRoundedRect { rect, color, .. }
                    if rect.y == layout.body_y + 16 && *color == cursor_color
            )
        })
        .ok_or_else(|| "missing cursor draw".to_owned())?;
    assert!(cursor_index > headerline_index);
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.y == layout.body_y && *color == cursor_color
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_truncates_preserving_tail_scope() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec!["abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz".to_owned()],
        headerline_requires_scrolled_viewport: false,
        ..HeaderlineTestUserLibrary::default()
    };
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec!["abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz".to_owned()],
        headerline_requires_scrolled_viewport: false,
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*headerline-gap*", vec!["alpha".to_owned()])?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let cursor_color = to_render_color(Color::RGB(110, 170, 255));
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y && text.starts_with("...")
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.y == layout.body_y + 16 && *color == cursor_color
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_divider_sits_below_last_headerline_row() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec![
            "module app".to_owned(),
            "fn render(value: usize)".to_owned(),
        ],
        headerline_requires_scrolled_viewport: false,
        ..HeaderlineTestUserLibrary::default()
    };
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec![
            "module app".to_owned(),
            "fn render(value: usize)".to_owned(),
        ],
        headerline_requires_scrolled_viewport: false,
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*headerline-divider*", vec!["alpha".to_owned()])?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == 8
                && rect.y == layout.body_y + (2 * 16) - 1
                && rect.width == 304
                && rect.height == 1
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_only_activates_once_scope_header_leaves_viewport() -> Result<(), String>
{
    let render_user_library = HeaderlineTestUserLibrary {
        scrolloff: 3.0,
        headerline_lines: vec!["STICKY HEADER".to_owned()],
        headerline_requires_scrolled_viewport: true,
        ..HeaderlineTestUserLibrary::default()
    };
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 3.0,
        headerline_lines: vec!["STICKY HEADER".to_owned()],
        headerline_requires_scrolled_viewport: true,
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline-activation*",
        vec![
            "scope header".to_owned(),
            "body line".to_owned(),
            "return 'a'".to_owned(),
        ],
    )?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.scroll_row = 0;
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    let mut hidden_scope_scene = Vec::new();
    let mut hidden_scope_target = DrawTarget::Scene(&mut hidden_scope_scene);
    render_buffer(
        &mut hidden_scope_target,
        BufferDrawRequest {
            buffer: shell_buffer(&state.runtime, buffer_id)?,
            view_state: (shell_buffer(&state.runtime, buffer_id)?).view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;
    assert!(!hidden_scope_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { text, .. } if text == "STICKY HEADER"
    )));

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.scroll_row = 1;
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut sticky_scene = Vec::new();
    let mut sticky_target = DrawTarget::Scene(&mut sticky_scene);
    render_buffer(
        &mut sticky_target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;
    assert!(sticky_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y && text == "STICKY HEADER"
    )));
    Ok(())
}

#[test]
fn ensure_visible_scrolloff_keeps_cursor_off_bottom_edge() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*scrolloff-bottom*",
        (0..30).map(|index| format!("line {index}")).collect(),
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_viewport_lines(10);
        buffer.set_cursor(TextPoint::new(8, 0));
        buffer.ensure_visible(10, 80, 4, 0, 3);
    }

    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.scroll_row, 2);
    Ok(())
}

#[test]
fn ensure_visible_scrolloff_keeps_cursor_off_top_edge() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*scrolloff-top*",
        (0..30).map(|index| format!("line {index}")).collect(),
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_viewport_lines(10);
        buffer.scroll_row = 5;
        buffer.set_cursor(TextPoint::new(6, 0));
        buffer.ensure_visible(10, 80, 4, 0, 3);
    }

    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.scroll_row, 3);
    Ok(())
}

#[test]
fn ensure_visible_builds_wrap_cache_for_large_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let lines = (0..(LARGE_BUFFER_WRAP_CACHE_LINE_THRESHOLD + 2))
        .map(|index| {
            if index % 7 == 0 {
                "abcdef".to_owned()
            } else {
                "abcde".to_owned()
            }
        })
        .collect();
    let buffer_id = install_text_test_buffer(&mut state, "*large-wrap-cache*", lines)?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.set_viewport_lines(20);
    buffer.set_cursor(TextPoint::new(10, 0));
    buffer.ensure_visible(20, 5, 4, 0, 0);

    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was not built for large buffer".to_owned())?;
    assert_eq!(
        cache.max_scroll_row(20),
        buffer.max_scroll_row_for_wrapped_rows(20, 5, 4)
    );
    Ok(())
}

#[test]
fn worker_syntax_window_matches_visible_window() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*worker-syntax-window*",
        (0..600).map(|index| format!("line {index}")).collect(),
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.set_language_id(Some("rust".to_owned()));
    buffer.set_viewport_lines(12);
    buffer.scroll_row = 80;

    let desired = buffer
        .desired_syntax_window()
        .ok_or_else(|| "visible syntax window should exist".to_owned())?;
    let worker = buffer
        .worker_syntax_window()
        .ok_or_else(|| "worker syntax window should exist".to_owned())?;
    assert_eq!(worker, desired);
    Ok(())
}

#[test]
fn one_line_scroll_marks_visible_syntax_window_dirty() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*scroll-syntax-window*",
        (0..600).map(|index| format!("line {index}")).collect(),
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.set_language_id(Some("rust".to_owned()));
    buffer.set_viewport_lines(40);
    buffer.scroll_row = 200;
    let window = buffer
        .desired_syntax_window()
        .ok_or_else(|| "visible syntax window should exist".to_owned())?;
    buffer.set_indexed_syntax_lines(Some(BTreeMap::new()), Some(window));
    buffer.ensure_visible_syntax_window();
    assert!(
        !buffer.syntax_dirty,
        "applied window should cover the current visible window"
    );

    buffer.scroll_row = 201;
    buffer.ensure_visible_syntax_window();
    assert!(
        buffer.syntax_dirty,
        "one-line j/k scroll should request a new syntax window"
    );
    Ok(())
}

#[test]
fn single_line_insert_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*single-line-wrap-edit*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    let cache = WrapRowCache::build(buffer, 5, 4);
    buffer.wrap_cache = Some(cache);
    buffer.set_cursor(TextPoint::new(0, 5));

    buffer.insert_text("f");

    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was cleared after single-line insert".to_owned())?;
    assert_eq!(cache.prefix_rows, vec![0, 2, 3]);
    Ok(())
}

fn assert_wrap_cache_matches_cold_build(
    buffer: &ShellBuffer,
    wrap_cols: usize,
    indent_size: usize,
) -> Result<(), String> {
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was missing".to_owned())?;
    let cold = WrapRowCache::build(buffer, wrap_cols, indent_size);
    assert_eq!(cache.line_count, cold.line_count);
    assert_eq!(cache.prefix_rows, cold.prefix_rows);
    assert_eq!(cache.wrap_cols, wrap_cols);
    assert_eq!(cache.indent_size, indent_size);
    Ok(())
}

#[test]
fn insert_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*newline-wrap-edit*",
        vec![
            "abcde".to_owned(),
            "    wrappedtail".to_owned(),
            "end".to_owned(),
        ],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 3));

    buffer.insert_text("\n");

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)?;
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was cleared after newline insert".to_owned())?;
    assert_eq!(cache.prefix_rows.len(), 5);
    Ok(())
}

#[test]
fn join_lines_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*join-wrap-edit*",
        vec!["abcde".to_owned(), "fghij".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(1, 0));

    buffer.backspace();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)?;
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was cleared after join".to_owned())?;
    assert_eq!(cache.prefix_rows.len(), 3);
    Ok(())
}

#[test]
fn delete_forward_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*delete-newline-wrap-edit*",
        vec!["abcde".to_owned(), "fghij".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 5));

    buffer.delete_forward();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn newline_insert_does_not_create_wrap_cache() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*newline-no-wrap-cache*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = None;
    buffer.set_cursor(TextPoint::new(0, 2));

    buffer.insert_text("\n");

    assert!(
        buffer.wrap_cache.is_none(),
        "newline must not create a wrap cache by itself"
    );
    Ok(())
}

#[test]
fn replace_mode_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*replace-newline-wrap-edit*",
        vec!["hello".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 2));

    buffer.replace_mode_text("\n");

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn open_line_below_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*open-line-wrap-edit*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 1));

    buffer.open_line_below();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn same_line_replace_keeps_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*replace-range-wrap-edit*",
        vec!["hello".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));

    buffer.replace_range(
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 0)),
        "    ",
    );

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn undo_newline_wrap_cache_matches_cold_rebuild() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*undo-newline-wrap-edit*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 3));
    buffer.insert_text("\n");
    buffer.record_undo_snapshot();
    buffer.undo();

    assert_eq!(buffer.line_count(), 2);
    match buffer.wrap_cache.as_ref() {
        None => {}
        Some(_) => assert_wrap_cache_matches_cold_build(buffer, 8, 4)?,
    }
    Ok(())
}

#[test]
fn sync_visible_buffer_layouts_ignores_headerline_rows_for_scrolloff() -> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library: Arc<dyn UserLibrary> =
        Arc::new(HeaderlineTestUserLibrary::with_scrolloff(3.0));
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*scrolloff-theme*",
        (0..80).map(|index| format!("line {index}")).collect(),
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(30, 0));

    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, render_width, render_height);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let expected_scrolloff = 3usize.min(layout.visible_rows.saturating_sub(1) / 2);
    assert!(expected_scrolloff > 1);
    let anchor = buffer_cursor_screen_anchor(
        buffer,
        rect,
        &*user_library,
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
        false,
    )
    .ok_or_else(|| "buffer cursor screen anchor was missing".to_owned())?;
    let cursor_body_row = ((anchor.y - layout.body_y) / line_height) as usize;
    assert_eq!(
        cursor_body_row,
        layout
            .visible_rows
            .saturating_sub(1)
            .saturating_sub(expected_scrolloff)
    );
    Ok(())
}

#[test]
fn sync_visible_buffer_layouts_counts_markdown_pretty_image_rows_for_scrolloff()
-> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 3.0,
        headerline_lines: Vec::new(),
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
    let mut text = format!("![red](data:image/png;base64,{png})\n");
    for index in 1..80 {
        text.push_str(&format!("line {index}\n"));
    }
    let buffer_id = install_markdown_test_buffer(&mut state, "*pretty-image-scrolloff*", &text)?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(4, 0));

    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, render_width, render_height);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let wrap_cols = wrap_columns_for_width(render_width, cell_width);
    let text_width_px = (wrap_cols as i32 * cell_width).max(1) as u32;
    let pretty_paint = markdown_pretty_paint_plan(
        buffer,
        &*user_library,
        MarkdownPrettyPaintArgs {
            visible_start: 0,
            visible_end: buffer.line_count().max(1),
            visual_selection: None,
            input_mode: InputMode::Normal,
            pane_width_px: text_width_px,
            line_height,
        },
    );
    let image_rows = pretty_paint
        .images
        .get(&0)
        .map(|image| image.rows())
        .ok_or_else(|| "pretty image did not decode for scroll fixture".to_owned())?;
    assert!(
        image_rows > 1,
        "fixture image should occupy multiple visual rows, got {image_rows}"
    );
    let expected_scrolloff = 3usize.min(layout.visible_rows.saturating_sub(1) / 2);
    assert!(expected_scrolloff > 1);
    let cursor_body_row = pretty_cursor_body_row(
        buffer,
        rect,
        &*user_library,
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
    )
    .ok_or_else(|| "cursor went off screen before scrolloff".to_owned())?;
    assert!(
        cursor_body_row >= expected_scrolloff,
        "cursor visual row {cursor_body_row} is above scrolloff {expected_scrolloff}"
    );
    assert!(
        cursor_body_row
            <= layout
                .visible_rows
                .saturating_sub(1)
                .saturating_sub(expected_scrolloff),
        "cursor visual row {cursor_body_row} is below scrolloff in {} visible rows",
        layout.visible_rows
    );
    Ok(())
}

#[test]
fn sync_visible_buffer_layouts_reuses_headerline_snapshot_while_typing() -> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typing-headerline-cache*",
        vec!["alpha".to_owned()],
    )?;

    let before = user_library.headerline_call_count();
    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;
    let after_first = user_library.headerline_call_count();
    assert!(after_first > before);

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text("!");
    }
    state.last_text_input_at = Some(Instant::now());
    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;
    assert_eq!(user_library.headerline_call_count(), after_first);
    Ok(())
}

#[test]
fn acp_wrapped_text_uses_full_width_on_continuation_rows() {
    let line = AcpRenderedTextLine {
        prefix: vec![
            acp_icon_segment(editor_icons::symbols::cod::COD_COMMENT, AcpColorRole::Accent),
            acp_text_segment(" ", AcpColorRole::Default),
        ],
        text: "Excellent! Now let me gather more context about the project to inform the documentation content:".to_owned(),
        text_role: AcpColorRole::Default,
        syntax_spans: Vec::new(),
        row_fill: None,
        gutter: false,
        align: AcpChatAlign::Full,
        bubble: false,
        bubble_group: 0,
    };

    let segments = acp_rendered_text_segments(&line, 32);

    assert!(segments.len() > 1);
    assert!(segments[1].end_col.saturating_sub(segments[1].start_col) > 8);
}

#[test]
fn acp_multiline_text_lines_strip_carriage_returns() {
    let lines = acp_multiline_text_lines(
        vec![
            acp_icon_segment(
                editor_icons::symbols::cod::COD_COMMENT,
                AcpColorRole::Accent,
            ),
            acp_text_segment(" ", AcpColorRole::Default),
        ],
        "alpha\r\nbeta\r\n",
        AcpColorRole::Default,
    );

    let rendered = lines
        .into_iter()
        .map(|line| match line {
            AcpRenderedLine::Text(line) => line.text,
            _ => String::new(),
        })
        .collect::<Vec<_>>();

    assert_eq!(rendered, vec!["alpha", "beta", ""]);
}

#[test]
fn acp_agent_markdown_uses_shared_pipeline_pretty() {
    let config = editor_markdown::MarkdownPrettyConfig::default();
    let rendered = render_markdown_ephemeral_content(
        "# Title\n\nSee `EditorRuntime` and **Volt**.",
        &config,
        Some(true),
        None,
    );
    assert!(
        rendered
            .lines
            .first()
            .is_some_and(|line| line.contains("Title") && !line.starts_with("# ")),
        "heading markers should be pretty-concealed: {:?}",
        rendered.lines.first()
    );

    let items = vec![AcpOutputItem::AgentBlocks(vec![ContentBlock::Text(
        TextContent::new("# Title\nhello"),
    )])];
    let mut registry = SyntaxRegistry::new();
    let lines = acp_build_output_lines(
        &items,
        Some(AcpMarkdownPaint {
            registry: &mut registry,
            config: &config,
        }),
        Some(true),
    );
    let texts: Vec<_> = lines
        .iter()
        .filter_map(|line| match line {
            AcpRenderedLine::Text(line) => Some(line.text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        texts
            .iter()
            .any(|line| line.contains("Title") && !line.starts_with("# ")),
        "agent ACP lines should run through markdown pipeline: {texts:?}"
    );
}

#[test]
fn acp_output_speaker_roles_and_tool_chip() {
    let items = vec![
        AcpOutputItem::UserPrompt("hi".to_owned()),
        AcpOutputItem::AgentBlocks(vec![ContentBlock::Text(TextContent::new("hello"))]),
        AcpOutputItem::ToolCall(
            ToolCall::new("tool-1", "Read file")
                .kind(ToolKind::Read)
                .status(ToolCallStatus::InProgress)
                .content(vec![ToolCallContent::from("12 lines")]),
        ),
    ];
    let lines = acp_build_output_lines(&items, None, None);
    let texts: Vec<_> = lines
        .iter()
        .filter_map(|line| match line {
            AcpRenderedLine::Text(line) => Some((
                line.text.as_str(),
                line.text_role,
                line.row_fill,
                line.gutter,
                line.align,
            )),
            _ => None,
        })
        .collect();
    assert!(
        texts
            .iter()
            .any(|(text, role, ..)| *text == "hi" && *role == AcpColorRole::Accent),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|(text, _, _, _, align)| { *text == "hi" && *align == AcpChatAlign::End }),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|(text, role, ..)| *text == "hello" && *role == AcpColorRole::Default),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|(text, _, _, _, align)| { *text == "hello" && *align == AcpChatAlign::Start }),
        "{texts:?}"
    );
    assert!(
        texts.iter().any(|(text, _, fill, _, _)| {
            *text == "Read file" && *fill == Some(AcpColorRole::Accent)
        }),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|(text, _, _, gutter, _)| *text == "12 lines" && *gutter),
        "{texts:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| matches!(line, AcpRenderedLine::Spacer)),
        "turns should be separated by spacers"
    );
}

#[test]
fn acp_tool_diff_renders_added_and_removed_lines() {
    let items = vec![AcpOutputItem::ToolCall(
        ToolCall::new("tool-diff", "Edit file")
            .kind(ToolKind::Edit)
            .status(ToolCallStatus::Completed)
            .content(vec![ToolCallContent::Diff(
                Diff::new("src/main.rs", "fn main() {\n    println!(\"b\");\n}\n")
                    .old_text("fn main() {\n    println!(\"a\");\n}\n"),
            )]),
    )];
    let lines = acp_build_output_lines(&items, None, None);
    let texts: Vec<_> = lines
        .iter()
        .filter_map(|line| match line {
            AcpRenderedLine::Text(line) => Some((line.text.as_str(), line.text_role)),
            _ => None,
        })
        .collect();
    assert!(
        texts
            .iter()
            .any(|(text, role)| *text == "    println!(\"a\");" && *role == AcpColorRole::Error),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|(text, role)| *text == "    println!(\"b\");" && *role == AcpColorRole::Success),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|(text, role)| *text == "Edit file" && *role == AcpColorRole::Muted),
        "{texts:?}"
    );
}

#[test]
fn wrap_line_segments_keeps_unbroken_words_together() {
    let segments = wrap_line_segments(&LineCharMap::new("alpha betagamma delta"), 10, 10);

    assert_eq!(
        segments
            .into_iter()
            .map(|segment| (segment.start_col, segment.end_col))
            .collect::<Vec<_>>(),
        vec![(0, 6), (6, 16), (16, 21)]
    );
}

#[test]
fn input_field_wrap_keeps_words_intact() {
    let mut input = InputField::new("> ");
    input.set_text("prefix text Please see the screenshot of this input");
    let rows = input.wrapped_visual_rows(28);

    assert!(
        !rows.iter().any(|row| row == "Pl" || row == "ease"),
        "rows: {rows:?}"
    );
    assert!(
        rows.windows(2)
            .all(|pair| { !(pair[0].ends_with("Pl") && pair[1].starts_with("ease")) }),
        "rows: {rows:?}"
    );
}

#[test]
fn block_cursor_text_overlay_positions_multibyte_cursor_text() {
    let line = "aéz";
    let char_map = LineCharMap::new(line);
    let overlay = block_cursor_text_overlay(CursorOverlayQuery {
        x: 24,
        line,
        char_map: &char_map,
        segment: LineWrapSegment {
            start_col: 0,
            end_col: 3,
        },
        line_index: 0,
        cursor: TextPoint::new(0, 1),
        color: Some(Color::RGB(1, 2, 3)),
        cell_width: 8,
    })
    .expect("cursor on a multibyte character should produce an overlay");

    assert_eq!(overlay.draw_x, 32);
    assert_eq!(overlay.text, "é");
    assert_eq!(overlay.color, Color::RGB(1, 2, 3));
}

#[test]
fn block_cursor_text_overlay_uses_visible_glyph_for_variation_selector() {
    let line = "⚛️x";
    let char_map = LineCharMap::new(line);
    let overlay = block_cursor_text_overlay(CursorOverlayQuery {
        x: 24,
        line,
        char_map: &char_map,
        segment: LineWrapSegment {
            start_col: 0,
            end_col: line.chars().count(),
        },
        line_index: 0,
        cursor: TextPoint::new(0, 1),
        color: Some(Color::RGB(1, 2, 3)),
        cell_width: 8,
    })
    .expect("cursor on a variation selector should reuse the visible glyph");

    assert_eq!(overlay.draw_x, 24);
    assert_eq!(overlay.text, "⚛");
    assert_eq!(overlay.color, Color::RGB(1, 2, 3));
}

#[test]
fn statusline_lsp_diagnostics_counts_errors_and_warnings() {
    let diagnostics = vec![
        LspDiagnostic::new(
            "rust-analyzer",
            "error",
            LspDiagnosticSeverity::Error,
            TextRange::new(TextPoint::new(0, 1), TextPoint::new(0, 2)),
        ),
        LspDiagnostic::new(
            "rust-analyzer",
            "warning",
            LspDiagnosticSeverity::Warning,
            TextRange::new(TextPoint::new(1, 3), TextPoint::new(1, 5)),
        ),
        LspDiagnostic::new(
            "rust-analyzer",
            "info",
            LspDiagnosticSeverity::Information,
            TextRange::new(TextPoint::new(2, 0), TextPoint::new(2, 1)),
        ),
    ];

    assert_eq!(
        statusline_lsp_diagnostics(&diagnostics),
        Some(editor_plugin_api::LspDiagnosticsInfo {
            errors: 1,
            warnings: 1,
        })
    );
}

#[test]
fn diagnostic_underlines_clip_to_wrapped_segment_and_draw_errors_last() {
    let diagnostics = vec![
        LspDiagnostic::new(
            "rust-analyzer",
            "info",
            LspDiagnosticSeverity::Information,
            TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 4)),
        ),
        LspDiagnostic::new(
            "rust-analyzer",
            "error",
            LspDiagnosticSeverity::Error,
            TextRange::new(TextPoint::new(0, 1), TextPoint::new(0, 3)),
        ),
    ];
    let line_spans = diagnostic_line_spans_for_diagnostics(&diagnostics);

    assert_eq!(
        diagnostic_underlines_for_segment(
            line_spans.get(&0).map(Box::as_ref).unwrap_or(&[]),
            None,
            6,
            LineWrapSegment {
                start_col: 0,
                end_col: 4,
            },
        ),
        vec![
            DiagnosticUnderlineSpan {
                start_col: 0,
                end_col: 4,
                severity: LspDiagnosticSeverity::Information,
            },
            DiagnosticUnderlineSpan {
                start_col: 1,
                end_col: 3,
                severity: LspDiagnosticSeverity::Error,
            },
        ]
    );
}

#[test]
fn diagnostic_underlines_expand_to_cover_narrowest_syntax_token() {
    let diagnostics = vec![LspDiagnostic::new(
        "rust-analyzer",
        "warning",
        LspDiagnosticSeverity::Warning,
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 2)),
    )];
    let line_spans = diagnostic_line_spans_for_diagnostics(&diagnostics);
    let syntax_spans = vec![
        LineSyntaxSpan {
            start: 0,
            end: 10,
            capture_name: Arc::from("source_file"),
            theme_token: Arc::from("syntax.source"),
        },
        LineSyntaxSpan {
            start: 0,
            end: 3,
            capture_name: Arc::from("keyword"),
            theme_token: Arc::from("syntax.keyword"),
        },
    ];

    assert_eq!(
        diagnostic_underlines_for_segment(
            line_spans.get(&0).map(Box::as_ref).unwrap_or(&[]),
            Some(syntax_spans.as_slice()),
            10,
            LineWrapSegment {
                start_col: 0,
                end_col: 10,
            },
        ),
        vec![DiagnosticUnderlineSpan {
            start_col: 0,
            end_col: 3,
            severity: LspDiagnosticSeverity::Warning,
        }]
    );
}

#[test]
fn diagnostic_line_spans_cache_multiline_ranges_by_line() {
    let diagnostics = vec![LspDiagnostic::new(
        "rust-analyzer",
        "warning",
        LspDiagnosticSeverity::Warning,
        TextRange::new(TextPoint::new(1, 3), TextPoint::new(3, 2)),
    )];
    let line_spans = diagnostic_line_spans_for_diagnostics(&diagnostics);

    assert_eq!(
        line_spans.get(&1).map(Box::as_ref),
        Some(
            [DiagnosticLineSpan {
                start_col: Some(3),
                end_col: None,
                severity: LspDiagnosticSeverity::Warning,
            }]
            .as_slice()
        )
    );
    assert_eq!(
        line_spans.get(&2).map(Box::as_ref),
        Some(
            [DiagnosticLineSpan {
                start_col: None,
                end_col: None,
                severity: LspDiagnosticSeverity::Warning,
            }]
            .as_slice()
        )
    );
    assert_eq!(
        line_spans.get(&3).map(Box::as_ref),
        Some(
            [DiagnosticLineSpan {
                start_col: None,
                end_col: Some(2),
                severity: LspDiagnosticSeverity::Warning,
            }]
            .as_slice()
        )
    );
}

#[test]
fn draw_diagnostic_undercurl_emits_single_scene_command() -> Result<(), String> {
    let color = Color::RGB(224, 107, 117);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_diagnostic_undercurl(&mut target, 10, 20, 6, 10, color)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Undercurl {
            x: 10,
            y: 20,
            width: 6,
            line_height: 10,
            color: to_render_color(color),
        }]
    );
    Ok(())
}

fn install_acp_test_buffer(
    state: &mut ShellState,
    output_lines: usize,
    input_text: &str,
    hint: Option<&str>,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*acp test*",
            BufferKind::Plugin(ACP_BUFFER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP test buffer is missing".to_owned())?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, Vec::new(), &NullUserLibrary);
    shell_buffer.init_acp_view("Test ACP");
    for index in 1..=output_lines {
        shell_buffer.acp_push_system_message(format!("line {index}"));
    }
    if let Some(input) = shell_buffer.input_field_mut() {
        input.set_text(input_text);
    }
    if let Some(footer) = shell_buffer.acp_footer_pane_mut() {
        footer.replace_lines(hint.into_iter().map(str::to_owned).collect(), true);
    }
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn state_with_user_library() -> Result<ShellState, String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
    ShellState::new_with_user_library(default_error_log_path(), false, user_library)
        .map_err(|error| error.to_string())
}

fn focus_input_normal_mode(state: &mut ShellState, buffer_id: BufferId) -> Result<(), String> {
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn install_user_plugin_buffer(
    state: &mut ShellState,
    name: &str,
    kind: &str,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            name,
            BufferKind::Plugin(kind.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    sync_active_buffer(&mut state.runtime)?;
    Ok(buffer_id)
}

fn install_plugin_sections_test_buffer(
    state: &mut ShellState,
    input_lines: &[&str],
    output_lines: &[&str],
) -> Result<BufferId, String> {
    install_plugin_sections_test_buffer_with_update(
        state,
        input_lines,
        output_lines,
        editor_plugin_api::PluginBufferSectionUpdate::Replace,
    )
}

fn install_plugin_sections_test_buffer_with_update(
    state: &mut ShellState,
    input_lines: &[&str],
    output_lines: &[&str],
    update: editor_plugin_api::PluginBufferSectionUpdate,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*calculator test*",
            BufferKind::Plugin(buffer_kinds::CALCULATOR.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "plugin test buffer is missing".to_owned())?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(
        buffer,
        input_lines.iter().map(|line| (*line).to_owned()).collect(),
        &NullUserLibrary,
    );
    let output = if output_lines.is_empty() {
        vec!["(press Ctrl+c Ctrl+c to evaluate)".to_owned()]
    } else {
        output_lines.iter().map(|line| (*line).to_owned()).collect()
    };
    shell_buffer.plugin_section_state = PluginSectionBufferState::new(
        PluginBufferSections::new(vec![
            editor_plugin_api::PluginBufferSection::new("Input")
                .with_writable(true)
                .with_initial_lines(input_lines.iter().map(|line| (*line).to_owned()).collect()),
            editor_plugin_api::PluginBufferSection::new("Output")
                .with_min_lines(1)
                .with_initial_lines(output)
                .with_update(update),
        ]),
        Some("Output"),
    );
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn plugin_section_lines(buffer: &ShellBuffer, name: &str) -> Result<Vec<String>, String> {
    let state = buffer
        .plugin_sections()
        .ok_or_else(|| "plugin section state missing".to_owned())?;
    let index = state
        .section_index_by_name(name)
        .ok_or_else(|| format!("section `{name}` missing"))?;
    if index == 0 {
        return Ok((0..buffer.text.line_count())
            .filter_map(|line_index| buffer.text.line(line_index))
            .collect());
    }
    let pane = state
        .attached_section(index)
        .ok_or_else(|| format!("attached section `{name}` missing"))?;
    Ok((0..pane.line_count())
        .map(|line_index| pane.text.line(line_index).unwrap_or_default())
        .collect())
}

fn install_user_acp_test_buffer(
    state: &mut ShellState,
    input_text: &str,
) -> Result<BufferId, String> {
    let buffer_id = install_user_plugin_buffer(state, "*acp*", user::acp::ACP_BUFFER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.init_acp_view("Test ACP");
        let _ = buffer.focus_acp_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .set_text(input_text);
    }
    Ok(buffer_id)
}

fn install_scratch_test_buffer(state: &mut ShellState, name: &str) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, name, BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        name,
        BufferKind::Scratch,
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)?;
    Ok(buffer_id)
}

fn install_markdown_test_buffer(
    state: &mut ShellState,
    name: &str,
    text: &str,
) -> Result<BufferId, String> {
    let buffer_id = install_scratch_test_buffer(state, name)?;
    let lines = if text.is_empty() {
        Vec::new()
    } else {
        text.lines().map(str::to_owned).collect()
    };
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(lines);
        buffer.set_language_id(Some("markdown".to_owned()));
    }
    sync_active_buffer(&mut state.runtime)?;
    Ok(buffer_id)
}

const PRETTY_CACHE_FIXTURE: &str = "# Title\n- item\nplain\n";

fn markdown_pretty_paint_args(buffer: &ShellBuffer) -> MarkdownPrettyPaintArgs {
    MarkdownPrettyPaintArgs {
        visible_start: 0,
        visible_end: buffer.line_count().max(1),
        visual_selection: None,
        input_mode: InputMode::Normal,
        pane_width_px: 640,
        line_height: 16,
    }
}

fn park_cursor_on_plain_pretty_line(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 0));
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_reuses_plan_for_same_revision() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-cache-revision*", PRETTY_CACHE_FIXTURE)?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let args = markdown_pretty_paint_args(buffer);
    let first = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let first_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing cached plan after first paint")?;
    let second = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let second_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing cached plan after second paint")?;
    assert!(
        std::sync::Arc::ptr_eq(&first_plan, &second_plan),
        "same revision should reuse MarkdownPrettyPlan"
    );
    assert_eq!(first.text_overrides, second.text_overrides);
    let heading = first
        .text_overrides
        .get(&0)
        .ok_or("heading Pretty override missing")?;
    assert!(
        heading.contains("Title") && !heading.starts_with("# "),
        "heading should conceal markers: {heading:?}"
    );
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_rebuilds_after_edit() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-cache-edit*", PRETTY_CACHE_FIXTURE)?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let before_plan = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let args = markdown_pretty_paint_args(buffer);
        let paint = markdown_pretty_paint_plan(buffer, &*user_library, args);
        let plan = markdown_pretty::last_cached_pretty_plan(buffer)
            .ok_or("missing cached plan before edit")?;
        (paint.text_overrides, plan)
    };
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 7));
        buffer.insert_text("!");
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let args = markdown_pretty_paint_args(buffer);
    let after = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let after_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing cached plan after edit")?;
    assert!(!std::sync::Arc::ptr_eq(&before_plan.1, &after_plan));
    assert_ne!(before_plan.0, after.text_overrides);
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_cursor_anti_conceal_uses_source() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*pretty-anti-conceal-cursor*",
        "# Title\n- item\n",
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    assert!(
        !paint.text_overrides.contains_key(&0),
        "cursor line should paint Markdown Raw: {:?}",
        paint.text_overrides
    );
    assert!(
        paint.text_overrides.contains_key(&1),
        "non-cursor Pretty lines should still override: {:?}",
        paint.text_overrides
    );
    let plan = markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing cached plan")?;
    let reused =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let reused_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing reused plan")?;
    assert!(std::sync::Arc::ptr_eq(&plan, &reused_plan));
    assert_eq!(paint.text_overrides, reused.text_overrides);
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_visual_anti_conceal_then_restores() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*pretty-anti-conceal-visual*",
        PRETTY_CACHE_FIXTURE,
    )?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let visual = VisualSelection::Range(TextRange::new(TextPoint::new(0, 0), TextPoint::new(1, 6)));
    let visual_args = MarkdownPrettyPaintArgs {
        visual_selection: Some(visual),
        input_mode: InputMode::Visual,
        ..markdown_pretty_paint_args(buffer)
    };
    let visual_paint = markdown_pretty_paint_plan(buffer, &*user_library, visual_args);
    assert!(
        !visual_paint.text_overrides.contains_key(&0),
        "Visual selection should paint Markdown Raw: {:?}",
        visual_paint.text_overrides
    );
    assert!(
        !visual_paint.text_overrides.contains_key(&1),
        "Visual selection should paint Markdown Raw on selected lines: {:?}",
        visual_paint.text_overrides
    );
    let visual_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing plan during visual")?;
    let normal_paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let normal_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing plan after visual")?;
    assert!(std::sync::Arc::ptr_eq(&visual_plan, &normal_plan));
    assert!(normal_paint.text_overrides.contains_key(&0));
    assert!(normal_paint.text_overrides.contains_key(&1));
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_toggle_off_is_raw() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-toggle-off*", "# Title\n- item\n")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.toggle_markdown_pretty(true);
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    assert!(paint.text_overrides.is_empty());
    assert!(paint.images.is_empty());
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_kill_switch_skips() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        markdown_pretty: MarkdownPrettyConfig {
            kill_switch_enabled: true,
            kill_switch_max_lines: 0,
            ..MarkdownPrettyConfig::default()
        },
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-kill-switch*", "# Title\n- item\n")?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let first =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let first_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing kill-switch sentinel")?;
    assert!(first.text_overrides.is_empty());
    assert!(first_plan.skipped_by_kill_switch);
    let second =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let second_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing reused sentinel")?;
    assert!(std::sync::Arc::ptr_eq(&first_plan, &second_plan));
    assert_eq!(first.text_overrides, second.text_overrides);
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_forced_language_caches() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*pretty-forced-language*")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "# Title".to_owned(),
            "- item".to_owned(),
            "plain".to_owned(),
        ]);
        buffer.set_forced_language_id("markdown");
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let first =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let first_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing Forced Language plan")?;
    let second =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let second_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing reused Forced Language plan")?;
    assert!(std::sync::Arc::ptr_eq(&first_plan, &second_plan));
    assert_eq!(first.text_overrides, second.text_overrides);
    assert!(
        first
            .text_overrides
            .get(&0)
            .is_some_and(|line| line.contains("Title") && !line.starts_with("# ")),
        "Forced Language markdown should Pretty: {:?}",
        first.text_overrides
    );
    Ok(())
}

fn markdown_table_event_dimensions() -> (u32, u32, i32, i32) {
    (640, 240, 8, 16)
}

fn focus_test_buffer(state: &mut ShellState, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)
}

fn install_browser_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            BROWSER_BUFFER_NAME,
            BufferKind::Plugin(BROWSER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        BROWSER_BUFFER_NAME,
        BufferKind::Plugin(BROWSER_KIND.to_owned()),
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn install_terminal_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, "*terminal*", BufferKind::Terminal, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        "*terminal*",
        BufferKind::Terminal,
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn install_terminal_popup_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*terminal-popup*", BufferKind::Terminal, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Terminal", vec![buffer_id], buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_popup_buffer(
        buffer_id,
        "*terminal-popup*",
        BufferKind::Terminal,
        &NullUserLibrary,
    );
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_popup_buffer(buffer_id);
        ui.set_popup_focus(true);
    }
    Ok(buffer_id)
}

fn install_git_status_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*git-status*",
            BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        "*git-status*",
        BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn run_git_in_dir(root: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run git {:?}: {error}", args))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if stderr.is_empty() {
            format!("git {:?} failed with status {}", args, output.status)
        } else {
            format!("git {:?} failed: {stderr}", args)
        };
        return Err(message);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn init_git_repo(label: &str) -> Result<std::path::PathBuf, String> {
    let repo = unique_temp_dir(label);
    run_git_in_dir(&repo, &["init", "-q"])?;
    run_git_in_dir(&repo, &["config", "user.email", "volt-tests@example.com"])?;
    run_git_in_dir(&repo, &["config", "user.name", "Volt Tests"])?;
    run_git_in_dir(&repo, &["config", "commit.gpgsign", "false"])?;
    Ok(repo)
}

fn init_git_repo_with_commit(label: &str) -> Result<std::path::PathBuf, String> {
    let repo = init_git_repo(label)?;
    std::fs::write(repo.join("README.md"), "seed\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "README.md"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "initial"])?;
    Ok(repo)
}

fn install_git_hook(repo: &std::path::Path, hook_name: &str, script: &str) -> Result<(), String> {
    let hook_path = repo.join(".git").join("hooks").join(hook_name);
    std::fs::write(&hook_path, script).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        let mut permissions = std::fs::metadata(&hook_path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&hook_path, permissions).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn open_repo_git_status_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "git-test", root)?;
    let buffer_id = install_git_status_test_buffer(state)?;
    refresh_git_status_buffer(&mut state.runtime, buffer_id)?;
    Ok(buffer_id)
}

fn wait_for_streamed_command_output_line(
    state: &mut ShellState,
    buffer_id: BufferId,
    needle: &str,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        let matched = shell_ui(&state.runtime)?
            .buffer(buffer_id)
            .is_some_and(|buffer| {
                (0..buffer.line_count()).any(|line_index| {
                    buffer
                        .text
                        .line(line_index)
                        .unwrap_or_default()
                        .contains(needle)
                })
            });
        if tracked && matched {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "streamed command buffer `{buffer_id}` never emitted `{needle}` while running"
    ))
}

fn wait_for_streamed_command_buffer_close(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        let buffered = shell_ui(&state.runtime)?.buffer(buffer_id).is_some();
        if !tracked && !buffered {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    let tracked = terminal_buffer_state(&state.runtime)?.contains(buffer_id);
    let worker_tracked = shell_ui(&state.runtime)?
        .streamed_command_worker
        .contains(buffer_id);
    let buffered = shell_ui(&state.runtime)?.buffer(buffer_id).is_some();
    let popup_visible = active_runtime_popup(&state.runtime)?.is_some();
    Err(format!(
        "temporary streamed command buffer `{buffer_id}` did not close in time (terminal_tracked={tracked}, worker_tracked={worker_tracked}, buffered={buffered}, popup_visible={popup_visible})"
    ))
}

fn open_oil_test_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "oil-test", root)?;
    open_oil_directory(&mut state.runtime, root.to_path_buf())?;
    active_shell_buffer_id(&state.runtime)
}

fn oil_line_index_containing(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    needle: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|&index| buffer.text.line(index).unwrap_or_default().contains(needle))
        .ok_or_else(|| format!("oil buffer is missing line containing `{needle}`"))
}

fn oil_type_new_entry_and_leave_insert(state: &mut ShellState, entry: &str) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let last_line = shell_buffer(&state.runtime, buffer_id)?
        .line_count()
        .saturating_sub(1);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(last_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    if shell_ui(&state.runtime)?.input_mode() != InputMode::Insert {
        return Err(format!(
            "expected insert mode after o, got {:?}",
            shell_ui(&state.runtime)?.input_mode()
        ));
    }
    state
        .handle_text_input(entry)
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn install_text_test_buffer(
    state: &mut ShellState,
    name: &str,
    lines: Vec<String>,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, name, BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "text test buffer is missing".to_owned())?;
    let shell_buffer = ShellBuffer::from_runtime_buffer(buffer, lines, &NullUserLibrary);
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn screen_point_for_buffer_point(
    state: &mut ShellState,
    buffer_id: BufferId,
    point: TextPoint,
    render_width: u32,
    render_height: u32,
    cell_width: i32,
    line_height: i32,
) -> Result<(f32, f32), String> {
    let original_cursor = shell_buffer(&state.runtime, buffer_id)?.cursor_point();
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(point);
    let anchor = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer_cursor_screen_anchor(
            buffer,
            PixelRectToRect::rect(0, 0, render_width, render_height),
            &*shell_user_library(&state.runtime),
            state.runtime.services().get::<ThemeRegistry>(),
            cell_width,
            line_height,
            false,
        )
        .ok_or_else(|| "buffer cursor screen anchor was missing".to_owned())?
    };
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(original_cursor);
    Ok((
        (anchor.x + (cell_width / 2).max(1)) as f32,
        (anchor.y + (line_height / 2).max(1)) as f32,
    ))
}

fn git_status_line_for_action_detail(
    state: &ShellState,
    buffer_id: BufferId,
    action_id: &str,
    detail: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|line_index| {
            git_action_detail(buffer.section_line_meta(*line_index), action_id).as_deref()
                == Some(detail)
        })
        .ok_or_else(|| format!("git status line for `{detail}` and `{action_id}` was not found"))
}

fn git_status_header_line(
    state: &ShellState,
    buffer_id: BufferId,
    section_id: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|line_index| {
            buffer.section_line_meta(*line_index).is_some_and(|meta| {
                meta.section_id == section_id
                    && matches!(meta.kind, SectionRenderLineKind::Header { .. })
            })
        })
        .ok_or_else(|| format!("git status header line for section `{section_id}` was not found"))
}

fn set_git_status_visual_line_selection(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    let (start_line, end_line) = if start_line <= end_line {
        (start_line, end_line)
    } else {
        (end_line, start_line)
    };
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(start_line, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(end_line, 0));
    Ok(())
}

fn set_git_status_visual_block_selection_with_ctrl_v(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    assert!(
        state
            .try_runtime_keybinding(Keycode::V, ctrl_mod())
            .map_err(|error| error.to_string())?
    );

    state
        .handle_text_input("v")
        .map_err(|error| error.to_string())?;

    let motion = if end_line >= start_line { "j" } else { "k" };
    for _ in 0..start_line.abs_diff(end_line) {
        state
            .handle_text_input(motion)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Visual);
    assert_eq!(
        shell_ui(&state.runtime)?.vim().visual_kind,
        VisualSelectionKind::Block
    );
    Ok(())
}

fn set_git_status_visual_line_selection_with_shift_v(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;

    let motion = if end_line >= start_line { "j" } else { "k" };
    for _ in 0..start_line.abs_diff(end_line) {
        state
            .handle_text_input(motion)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Visual);
    assert_eq!(
        shell_ui(&state.runtime)?.vim().visual_kind,
        VisualSelectionKind::Line
    );
    Ok(())
}

type GitSnapshotPaths = (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>);

fn git_status_snapshot_paths(
    state: &ShellState,
    buffer_id: BufferId,
) -> Result<GitSnapshotPaths, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let snapshot = buffer
        .git_snapshot()
        .ok_or_else(|| "git snapshot missing".to_owned())?;
    let staged = snapshot
        .staged()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect();
    let unstaged = snapshot
        .unstaged()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect();
    let untracked = snapshot.untracked().iter().cloned().collect();
    Ok((staged, unstaged, untracked))
}

fn install_hover_test_overlay(state: &mut ShellState, focused: bool) -> Result<BufferId, String> {
    let buffer_id = shell_ui(&state.runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let anchor = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    shell_ui_mut(&mut state.runtime)?.set_hover(HoverOverlay {
        buffer_id,
        anchor,
        token: "hover".to_owned(),
        providers: vec![
            HoverProviderContent {
                provider_label: "Alpha".to_owned(),
                provider_icon: "A".to_owned(),
                lines: vec!["first".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
            HoverProviderContent {
                provider_label: "Beta".to_owned(),
                provider_icon: "B".to_owned(),
                lines: vec!["second".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
            HoverProviderContent {
                provider_label: "Gamma".to_owned(),
                provider_icon: "G".to_owned(),
                lines: vec!["third".to_owned()],
                syntax_lines: BTreeMap::new(),
            },
        ],
        provider_index: 0,
        scroll_offset: 0,
        focused,
        line_limit: 8,
        pending_g_prefix: false,
        count: None,
    });
    Ok(buffer_id)
}

fn install_scrollable_hover_test_overlay(
    state: &mut ShellState,
    focused: bool,
) -> Result<BufferId, String> {
    let buffer_id = shell_ui(&state.runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let anchor = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    let lines = (1..=12)
        .map(|line| format!("Line {line}"))
        .collect::<Vec<_>>();
    shell_ui_mut(&mut state.runtime)?.set_hover(HoverOverlay {
        buffer_id,
        anchor,
        token: "hover".to_owned(),
        providers: vec![HoverProviderContent {
            provider_label: "Scrollable".to_owned(),
            provider_icon: "S".to_owned(),
            lines,
            syntax_lines: BTreeMap::new(),
        }],
        provider_index: 0,
        scroll_offset: 0,
        focused,
        line_limit: 4,
        pending_g_prefix: false,
        count: None,
    });
    Ok(buffer_id)
}

fn hover_scroll_offset(state: &ShellState) -> Result<usize, String> {
    shell_ui(&state.runtime)?
        .hover()
        .map(|hover| hover.scroll_offset)
        .ok_or_else(|| "hover overlay missing".to_owned())
}

fn test_notification_update(
    key: &str,
    severity: NotificationSeverity,
    title: &str,
    body_lines: &[&str],
    progress: Option<u8>,
    active: bool,
) -> NotificationUpdate {
    NotificationUpdate {
        key: key.to_owned(),
        severity,
        title: title.to_owned(),
        body_lines: body_lines.iter().map(|line| (*line).to_owned()).collect(),
        progress: progress.map(|percentage| NotificationProgress {
            percentage: Some(percentage),
        }),
        active,
        action: None,
        workspace_id: None,
    }
}

#[test]
fn parse_rg_workspace_search_line_extracts_location() {
    let parsed = parse_rg_workspace_search_line(r"src\main.rs:12:7:let answer = compute();")
        .expect("rg output should parse into a workspace search match");
    assert_eq!(parsed.0, r"src\main.rs");
    assert_eq!(parsed.1, 12);
    assert_eq!(parsed.2, 7);
    assert_eq!(parsed.3, "let answer = compute();");
}

#[test]
fn parse_grep_workspace_search_line_finds_case_insensitive_column() {
    let parsed = parse_grep_workspace_search_line(r"src\lib.rs:3:Hello Workspace", "workspace")
        .expect("grep output should parse into a workspace search match");
    assert_eq!(parsed.0, r"src\lib.rs");
    assert_eq!(parsed.1, 3);
    assert_eq!(parsed.2, 7);
    assert_eq!(parsed.3, "Hello Workspace");
}

#[test]
fn workspace_search_char_column_handles_utf8_offsets() {
    assert_eq!(workspace_search_char_column("aébc", 0), 0);
    assert_eq!(workspace_search_char_column("aébc", 1), 1);
    assert_eq!(workspace_search_char_column("aébc", 3), 2);
}

#[test]
fn collect_search_output_stops_after_limit() {
    let (output, reached_limit) =
        collect_search_output(std::io::Cursor::new("one\ntwo\nthree\n"), 2)
            .expect("search output should be collected");
    assert_eq!(output, "one\ntwo\n");
    assert!(reached_limit);
}

#[test]
fn frame_pacing_remaining_clamps_to_120fps_budget() {
    let now = Instant::now();
    let remaining = frame_pacing_remaining(now - Duration::from_millis(2), now);
    assert!(remaining >= Duration::from_micros(6_000));
    assert_eq!(
        frame_pacing_remaining(now - Duration::from_millis(10), now),
        Duration::from_secs(0)
    );
}

#[test]
fn git_refresh_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(git_refresh_deferred_for_typing(Some(now), now));
    assert!(git_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!git_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!git_refresh_deferred_for_typing(None, now));
}

#[test]
fn secondary_refresh_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(secondary_refresh_deferred_for_typing(Some(now), now));
    assert!(secondary_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!secondary_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!secondary_refresh_deferred_for_typing(None, now));
}

#[test]
fn frame_pacing_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(frame_pacing_deferred_for_typing(Some(now), now));
    assert!(frame_pacing_deferred_for_typing(
        Some(now - FRAME_PACING_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!frame_pacing_deferred_for_typing(
        Some(now - FRAME_PACING_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!frame_pacing_deferred_for_typing(None, now));
}

#[test]
fn idle_wait_timeout_equals_next_deadline_when_idle() {
    let now = Instant::now();
    let deadline = now + Duration::from_millis(40);
    assert_eq!(
        idle_wait_timeout(now, &[deadline], false, false),
        Some(Duration::from_millis(40))
    );
}

#[test]
fn idle_wait_timeout_caps_and_skips_when_interacting() {
    let now = Instant::now();
    assert_eq!(
        idle_wait_timeout(now, &[], false, false),
        Some(IDLE_WAIT_CAP)
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_secs(5)], false, false),
        Some(IDLE_WAIT_CAP)
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_millis(40)], true, false),
        None
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_millis(40)], false, true),
        None
    );
}

#[test]
fn normal_mode_text_input_does_not_activate_typing_budget() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;

    assert!(!state.secondary_refresh_deferred_for_typing(Instant::now()));
    assert!(!state.typing_refresh_budget_active(Instant::now()));
    Ok(())
}

#[test]
fn insert_mode_text_input_activates_typing_budget() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    assert!(state.secondary_refresh_deferred_for_typing(Instant::now()));
    assert!(state.typing_refresh_budget_active(Instant::now()));
    Ok(())
}

#[test]
fn context_overlay_cache_reuses_stale_snapshot_while_typing() {
    let cached = Arc::new(BufferContextOverlaySnapshot {
        key: BufferContextOverlayCacheKey {
            buffer_revision: 41,
            buffer_name: "demo.rs".to_owned(),
            language_id: Some("rust".to_owned()),
            viewport_top_line: 10,
            cursor_line: 20,
            cursor_column: 4,
        },
        headerline_lines: vec!["fn demo".to_owned()],
        ghost_text_by_line: BTreeMap::new(),
    });
    let key = BufferContextOverlayCacheKey {
        buffer_revision: 42,
        buffer_name: "demo.rs".to_owned(),
        language_id: Some("rust".to_owned()),
        viewport_top_line: 11,
        cursor_line: 21,
        cursor_column: 5,
    };

    let snapshot =
        cached_context_overlay_snapshot(Some(&cached), &key, true).expect("stale snapshot");

    assert!(Arc::ptr_eq(&snapshot, &cached));
    assert_eq!(snapshot.key.buffer_revision, 41);
    assert_eq!(snapshot.headerline_lines, vec!["fn demo".to_owned()]);
}

#[test]
fn context_overlay_cache_requires_matching_buffer_identity() {
    let cached = Arc::new(BufferContextOverlaySnapshot {
        key: BufferContextOverlayCacheKey {
            buffer_revision: 1,
            buffer_name: "demo.rs".to_owned(),
            language_id: Some("rust".to_owned()),
            viewport_top_line: 0,
            cursor_line: 0,
            cursor_column: 0,
        },
        headerline_lines: vec!["fn demo".to_owned()],
        ghost_text_by_line: BTreeMap::new(),
    });
    let key = BufferContextOverlayCacheKey {
        buffer_revision: 2,
        buffer_name: "demo.py".to_owned(),
        language_id: Some("python".to_owned()),
        viewport_top_line: 0,
        cursor_line: 0,
        cursor_column: 0,
    };

    assert!(cached_context_overlay_snapshot(Some(&cached), &key, false).is_none());
    assert!(cached_context_overlay_snapshot(Some(&cached), &key, true).is_none());
}

#[test]
fn context_overlay_snapshot_reuses_same_arc_when_key_matches() -> Result<(), String> {
    let user_library = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline-arc-reuse*",
        vec!["alpha".to_owned()],
    )?;
    let first = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer.context_overlay_snapshot(&*user_library, false)
    };
    let second = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer.context_overlay_snapshot(&*user_library, false)
    };
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(user_library.headerline_call_count(), 1);
    Ok(())
}

#[test]
fn typing_event_batches_yield_once_budget_is_exhausted() {
    let now = Instant::now();
    assert!(!should_yield_after_typing_batch(
        0,
        TYPING_EVENT_BATCH_LIMIT,
        now
    ));
    assert!(!should_yield_after_typing_batch(
        1,
        TYPING_EVENT_BATCH_LIMIT - 1,
        now
    ));
    assert!(should_yield_after_typing_batch(
        1,
        TYPING_EVENT_BATCH_LIMIT,
        now
    ));
    assert!(should_yield_after_typing_batch(
        1,
        1,
        now - TYPING_EVENT_BATCH_TIME_BUDGET
    ));
}

#[test]
fn truncate_text_to_width_uses_cell_budget() {
    assert_eq!(truncate_text_to_width("abcdef", 24, 4), "abcdef");
    assert_eq!(truncate_text_to_width("abcdef", 20, 4), "ab...");
    assert_eq!(truncate_text_to_width("abcdef", 8, 4), "...");
}

#[test]
fn truncate_picker_label_shrink_directories_preserves_filename() {
    use editor_plugin_api::PickerTruncateStrategy;

    let path = "src/dir1/dir2/test.rs";
    assert_eq!(
        truncate_picker_label(path, 240, 4, PickerTruncateStrategy::ShrinkDirectories),
        "s/d/d/test.rs"
    );
}

#[test]
fn truncate_picker_label_start_ellipsis_preserves_tail() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label(
            "src/dir1/dir2/test.rs",
            56,
            4,
            PickerTruncateStrategy::StartEllipsis
        ),
        "...ir2/test.rs"
    );
}

#[test]
fn truncate_picker_label_middle_ellipsis_preserves_both_ends() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label(
            "src/dir1/dir2/test.rs",
            56,
            4,
            PickerTruncateStrategy::MiddleEllipsis
        ),
        "src...test.rs"
    );
}

#[test]
fn truncate_picker_label_auto_falls_back_to_start_ellipsis() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label("src/dir1/dir2/test.rs", 56, 4, PickerTruncateStrategy::Auto),
        "...ir2/test.rs"
    );
}

#[test]
fn truncate_picker_label_file_name_with_parent() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label(
            "src/dir1/dir2/test.rs",
            240,
            4,
            PickerTruncateStrategy::FileNameWithParent
        ),
        "dir2/test.rs"
    );
}

#[test]
fn truncate_picker_label_shrink_all_includes_stem() {
    use editor_plugin_api::PickerTruncateStrategy;

    assert_eq!(
        truncate_picker_label(
            "src/dir1/dir2/test.rs",
            240,
            4,
            PickerTruncateStrategy::ShrinkAll
        ),
        "s/d/d/t.rs"
    );
}

#[test]
fn git_status_header_spans_skip_leading_icons() {
    let line = SectionRenderLine {
        text: format!(
            "{} Head: master f9d8c15 Added some more keybinds",
            editor_icons::symbols::dev::DEV_GIT_BRANCH
        ),
        depth: 1,
        section_id: GIT_SECTION_HEADERS.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(),
                editor_icons::symbols::dev::DEV_GIT_BRANCH.to_owned(),
            ),
            (TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(), "Head:".to_owned()),
            (
                TOKEN_GIT_STATUS_HEADER_VALUE.to_owned(),
                "master".to_owned()
            ),
            (
                TOKEN_GIT_STATUS_HEADER_HASH.to_owned(),
                "f9d8c15".to_owned()
            ),
            (
                TOKEN_GIT_STATUS_HEADER_SUMMARY.to_owned(),
                "Added some more keybinds".to_owned(),
            ),
        ]
    );
}

#[test]
fn git_status_merge_header_spans_keep_tracking_counts() {
    let line = SectionRenderLine {
        text: format!(
            "{} Merge: origin/main (ahead 2, behind 1)",
            editor_icons::symbols::cod::COD_ARROW_DOWN
        ),
        depth: 1,
        section_id: GIT_SECTION_HEADERS.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(),
                editor_icons::symbols::cod::COD_ARROW_DOWN.to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(),
                "Merge:".to_owned()
            ),
            (
                TOKEN_GIT_STATUS_HEADER_VALUE.to_owned(),
                "origin/main".to_owned(),
            ),
            (TOKEN_GIT_STATUS_SECTION_COUNT.to_owned(), "2".to_owned()),
            (TOKEN_GIT_STATUS_SECTION_COUNT.to_owned(), "1".to_owned()),
        ]
    );
}

#[test]
fn git_status_entry_spans_skip_leading_icons() {
    let line = SectionRenderLine {
        text: format!(
            "{} crates/editor-sdl/src/shell.rs",
            editor_icons::symbols::cod::COD_DIFF_MODIFIED
        ),
        depth: 1,
        section_id: GIT_SECTION_UNSTAGED.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_ENTRY_MODIFIED.to_owned(),
                editor_icons::symbols::cod::COD_DIFF_MODIFIED.to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_ENTRY_PATH.to_owned(),
                "crates/editor-sdl/src/shell.rs".to_owned(),
            ),
        ]
    );
}

#[test]
fn git_status_stash_spans_handle_compact_stash_names() {
    let line = SectionRenderLine {
        text: format!(
            "{} stash[0] WIP on master: overnight todo",
            editor_icons::symbols::cod::COD_HISTORY
        ),
        depth: 1,
        section_id: GIT_SECTION_STASHES.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_STASH_NAME.to_owned(),
                editor_icons::symbols::cod::COD_HISTORY.to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_STASH_NAME.to_owned(),
                "stash[0]".to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_STASH_SUMMARY.to_owned(),
                "WIP on master: overnight todo".to_owned(),
            ),
        ]
    );
}

#[test]
fn git_status_uppercase_f_starts_pull_prefix() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_git_status_test_buffer(&mut state)?;

    assert!(handle_git_status_chord(&mut state.runtime, "F")?);
    assert_eq!(take_git_prefix(&mut state.runtime)?, Some(GitPrefix::Pull));
    Ok(())
}

#[test]
fn git_status_sequence_commands_are_registered() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;

    for &(name, _, _) in GIT_STATUS_COMMANDS {
        assert!(
            state.runtime.commands().contains(name),
            "missing command `{name}`"
        );
    }
    for name in ["git.diff", "git.log", "git.stash-list"] {
        assert!(
            state.runtime.commands().contains(name),
            "missing command `{name}`"
        );
    }

    Ok(())
}

#[test]
fn git_status_command_name_maps_sequences_to_picker_commands() {
    let user_library = user::UserLibraryImpl;
    assert_eq!(
        git_status_command_name(&user_library, None, "S"),
        Some("git.status.stage-all")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Pull), "u"),
        Some("git.status.pull-upstream")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Branch), "b"),
        Some("git.status.branches")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Diff), "w"),
        Some("git.diff")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Log), "l"),
        Some("git.log")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Stash), "l"),
        Some("git.stash-list")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Rebase), "f"),
        Some("git.status.rebase-autosquash")
    );
    assert_eq!(
        git_status_command_name(&user_library, Some(GitPrefix::Reset), "f"),
        Some("git.status.checkout-file")
    );
}

#[test]
fn git_status_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-visual-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection(&mut state, buffer_id, alpha, beta)?;

    assert!(handle_git_status_chord(&mut state.runtime, "s")?);

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-visual-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection(&mut state, buffer_id, alpha, beta)?;

    assert!(handle_git_status_chord(&mut state.runtime, "u")?);

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("s")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("u")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_x_deletes_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-delete")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(staged.is_empty());
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(!repo.join("alpha.txt").exists());
    assert!(!repo.join("beta.txt").exists());
    assert!(repo.join("gamma.txt").exists());
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("s")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("u")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_x_deletes_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-delete")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(staged.is_empty());
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(!repo.join("alpha.txt").exists());
    assert!(!repo.join("beta.txt").exists());
    assert!(repo.join("gamma.txt").exists());
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_buffer_supports_first_commit_on_fresh_repo() -> Result<(), String> {
    let repo = init_git_repo("git-status-fresh-repo")?;
    let branch = run_git_in_dir(&repo, &["symbolic-ref", "--short", "HEAD"])?
        .trim()
        .to_owned();
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let snapshot = buffer
        .git_snapshot()
        .ok_or_else(|| "git snapshot missing".to_owned())?;
    let has_commit_action = (0..buffer.line_count()).any(|line_index| {
        buffer
            .section_line_meta(line_index)
            .and_then(|meta| meta.action.as_ref())
            .is_some_and(|action| action.id() == editor_plugin_api::git_actions::COMMIT_OPEN)
    });

    assert_eq!(snapshot.branch(), Some(branch.as_str()));
    assert!(snapshot.head().is_none());
    assert!(has_commit_action);
    assert_eq!(staged, BTreeSet::from(["alpha.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert!(untracked.is_empty());

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_focus_refresh_reuses_recent_snapshot() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-focus-refresh-cache")?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let (_, _, untracked_before) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(untracked_before.is_empty());

    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;

    refresh_git_status_if_active_if_due(&mut state.runtime)?;
    let (_, _, untracked_throttled) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(untracked_throttled.is_empty());

    refresh_git_status_buffer(&mut state.runtime, buffer_id)?;
    let (_, _, untracked_after) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(untracked_after, BTreeSet::from(["beta.txt".to_owned()]));

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_tab_on_unstaged_file_opens_diff_buffer() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-tab-unstaged")?;
    std::fs::write(repo.join("alpha.txt"), "before\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "add alpha"])?;
    std::fs::write(repo.join("alpha.txt"), "after\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let status_buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha_line = git_status_line_for_action_detail(
        &state,
        status_buffer_id,
        GIT_ACTION_STAGE_FILE,
        "alpha.txt",
    )?;
    shell_buffer_mut(&mut state.runtime, status_buffer_id)?
        .set_cursor(TextPoint::new(alpha_line, 0));

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    let diff_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let diff_buffer = shell_buffer(&state.runtime, diff_buffer_id)?;
    assert_ne!(diff_buffer_id, status_buffer_id);
    assert!(matches!(
        &diff_buffer.kind,
        BufferKind::Plugin(kind) if kind == GIT_DIFF_KIND
    ));
    assert_eq!(diff_buffer.language_id(), Some("diff"));
    assert!((0..diff_buffer.line_count()).any(|line_index| {
        diff_buffer
            .text
            .line(line_index)
            .unwrap_or_default()
            .contains("diff --git")
    }));
    assert!((0..diff_buffer.line_count()).any(|line_index| {
        diff_buffer
            .text
            .line(line_index)
            .unwrap_or_default()
            .contains("alpha.txt")
    }));

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_tab_on_staged_file_opens_diff_buffer() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-tab-staged")?;
    std::fs::write(repo.join("alpha.txt"), "before\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "add alpha"])?;
    std::fs::write(repo.join("alpha.txt"), "after\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;

    let mut state = state_with_user_library()?;
    let status_buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha_line = git_status_line_for_action_detail(
        &state,
        status_buffer_id,
        GIT_ACTION_UNSTAGE_FILE,
        "alpha.txt",
    )?;
    shell_buffer_mut(&mut state.runtime, status_buffer_id)?
        .set_cursor(TextPoint::new(alpha_line, 0));

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    let diff_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let diff_buffer = shell_buffer(&state.runtime, diff_buffer_id)?;
    assert_ne!(diff_buffer_id, status_buffer_id);
    assert!(matches!(
        &diff_buffer.kind,
        BufferKind::Plugin(kind) if kind == GIT_DIFF_KIND
    ));
    assert_eq!(diff_buffer.language_id(), Some("diff"));
    assert!((0..diff_buffer.line_count()).any(|line_index| {
        diff_buffer
            .text
            .line(line_index)
            .unwrap_or_default()
            .contains("diff --git")
    }));
    assert!((0..diff_buffer.line_count()).any(|line_index| {
        diff_buffer
            .text
            .line(line_index)
            .unwrap_or_default()
            .contains("alpha.txt")
    }));

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_tab_on_header_still_toggles_section() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-tab-header")?;
    std::fs::write(repo.join("alpha.txt"), "before\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "add alpha"])?;
    std::fs::write(repo.join("alpha.txt"), "after\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let header_line = git_status_header_line(&state, buffer_id, GIT_SECTION_UNSTAGED)?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(header_line, 0));
    assert!(
        !shell_buffer(&state.runtime, buffer_id)?
            .section_state()
            .is_some_and(|state| state.collapsed.is_collapsed(GIT_SECTION_UNSTAGED))
    );

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);
    assert!(
        buffer
            .section_state()
            .is_some_and(|state| state.collapsed.is_collapsed(GIT_SECTION_UNSTAGED))
    );

    drop(state);
    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_push_upstream_streams_into_popup_buffer_and_refreshes_status() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-push-upstream-popup")?;
    let remote = unique_temp_dir("git-push-upstream-popup-remote");
    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            remote
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", remote.display()))?,
        ],
    )?;
    let branch = run_git_in_dir(&repo, &["symbolic-ref", "--short", "HEAD"])?
        .trim()
        .to_owned();
    run_git_in_dir(
        &repo,
        &["push", "-q", "--set-upstream", "origin", branch.as_str()],
    )?;
    install_git_hook(
        &repo,
        "pre-push",
        "#!/bin/sh\necho \"pre-push hook starting\"\nsleep 1\necho \"pre-push hook finishing\"\n",
    )?;
    std::fs::write(repo.join("feature.txt"), "feature\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "feature.txt"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "feature"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let snapshot = shell_buffer(&state.runtime, buffer_id)?
        .git_snapshot()
        .cloned()
        .ok_or_else(|| "git snapshot missing before push".to_owned())?;
    assert_eq!(snapshot.ahead(), 1);
    assert!(snapshot.upstream().is_some());

    push_git_to_upstream(&mut state.runtime, buffer_id)?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed popup was not opened for git push".to_owned())?;
    assert!(shell_ui(&state.runtime)?.popup_focus);
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert!(matches!(
        &shell_buffer(&state.runtime, popup.active_buffer)?.kind,
        BufferKind::Plugin(kind) if kind == INTERACTIVE_READONLY_KIND
    ));
    assert!(!terminal_buffer_state(&state.runtime)?.contains(popup.active_buffer));

    wait_for_streamed_command_output_line(
        &mut state,
        popup.active_buffer,
        "pre-push hook starting",
    )?;
    wait_for_streamed_command_buffer_close(&mut state, popup.active_buffer)?;
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.popup_focus);
    assert_eq!(ui.popup_buffer_id, None);
    assert!(ui.buffer(popup.active_buffer).is_none());
    assert!(!ui.streamed_command_worker.contains(popup.active_buffer));
    assert!(!terminal_buffer_state(&state.runtime)?.contains(popup.active_buffer));
    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);

    let refreshed = shell_buffer(&state.runtime, buffer_id)?
        .git_snapshot()
        .cloned()
        .ok_or_else(|| "git snapshot missing after push".to_owned())?;
    assert_eq!(refreshed.ahead(), 0);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&remote).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_pull_upstream_streams_into_popup_buffer() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-pull-upstream-popup")?;
    let remote = unique_temp_dir("git-pull-upstream-popup-remote");
    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            remote
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", remote.display()))?,
        ],
    )?;
    let branch = run_git_in_dir(&repo, &["symbolic-ref", "--short", "HEAD"])?
        .trim()
        .to_owned();
    run_git_in_dir(
        &repo,
        &["push", "-q", "--set-upstream", "origin", branch.as_str()],
    )?;
    install_git_hook(
        &repo,
        "pre-merge-commit",
        "#!/bin/sh\necho \"pre-merge hook starting\"\nsleep 1\necho \"pre-merge hook finishing\"\n",
    )?;

    // Create a second commit on remote via a clone so pull has work.
    let clone = unique_temp_dir("git-pull-upstream-clone");
    std::fs::remove_dir_all(&clone).ok();
    run_git_in_dir(
        repo.parent().unwrap_or(&repo),
        &[
            "clone",
            "-q",
            remote
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", remote.display()))?,
            clone
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", clone.display()))?,
        ],
    )?;
    run_git_in_dir(&clone, &["config", "user.email", "volt-tests@example.com"])?;
    run_git_in_dir(&clone, &["config", "user.name", "Volt Tests"])?;
    run_git_in_dir(&clone, &["config", "commit.gpgsign", "false"])?;
    std::fs::write(clone.join("remote.txt"), "from-remote\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&clone, &["add", "--", "remote.txt"])?;
    run_git_in_dir(&clone, &["commit", "-qm", "remote change"])?;
    run_git_in_dir(&clone, &["push", "-q", "origin", "HEAD"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    pull_git_upstream(&mut state.runtime, buffer_id)?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed popup was not opened for git pull".to_owned())?;
    assert!(
        shell_buffer(&state.runtime, popup.active_buffer)?
            .text
            .text()
            .contains("git pull")
    );
    wait_for_streamed_command_buffer_close(&mut state, popup.active_buffer)?;
    assert!(active_runtime_popup(&state.runtime)?.is_none());

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&remote).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&clone).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn fetch_git_prune_is_silent_command_without_popup() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-fetch-prune-silent")?;
    let remote = unique_temp_dir("git-fetch-prune-silent-remote");
    run_git_in_dir(&remote, &["init", "--bare", "-q"])?;
    run_git_in_dir(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            remote
                .to_str()
                .ok_or_else(|| format!("non-utf8 path `{}`", remote.display()))?,
        ],
    )?;
    let branch = run_git_in_dir(&repo, &["symbolic-ref", "--short", "HEAD"])?
        .trim()
        .to_owned();
    run_git_in_dir(
        &repo,
        &["push", "-q", "--set-upstream", "origin", branch.as_str()],
    )?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    fetch_git_prune(&mut state.runtime, &repo)?;
    assert!(
        active_runtime_popup(&state.runtime)?.is_none(),
        "Silent Command must not open a Command Stream popup"
    );

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&remote).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_editor_confirm_writes_file_and_signals_stub() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let mut env = Vec::new();
    inject_git_editor_env(&mut state.runtime, &mut env)?;
    let dir = env
        .iter()
        .find(|(key, _)| key == VOLT_GIT_EDITOR_DIR_ENV)
        .map(|(_, value)| PathBuf::from(value))
        .ok_or_else(|| "VOLT_GIT_EDITOR_DIR missing".to_owned())?;
    let edit_path = dir.join("todo.txt");
    std::fs::write(&edit_path, "pick abc hello\n").map_err(|error| error.to_string())?;
    let request_id = "test-confirm";
    std::fs::write(
        dir.join(format!("request-{request_id}")),
        format!("{}\n", edit_path.display()),
    )
    .map_err(|error| error.to_string())?;

    assert!(refresh_pending_git_editor(&mut state.runtime)?);
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert!(matches!(
        &shell_buffer(&state.runtime, buffer_id)?.kind,
        BufferKind::Plugin(kind) if kind == GIT_EDITOR_KIND
    ));
    shell_buffer_mut(&mut state.runtime, buffer_id)?
        .replace_with_lines(vec!["pick abc hello edited".to_owned()]);
    confirm_git_editor_buffer(&mut state.runtime, buffer_id)?;

    let written = std::fs::read_to_string(&edit_path).map_err(|error| error.to_string())?;
    assert!(written.contains("edited"));
    let result = std::fs::read_to_string(dir.join(format!("result-{request_id}")))
        .map_err(|error| error.to_string())?;
    assert_eq!(result.trim(), "0");
    Ok(())
}

#[test]
fn git_line_is_untracked_uses_section_metadata() {
    let meta = SectionLineMeta {
        section_id: GIT_SECTION_UNTRACKED.to_owned(),
        kind: SectionRenderLineKind::Item,
        action: None,
    };
    let staged_meta = SectionLineMeta {
        section_id: GIT_SECTION_UNSTAGED.to_owned(),
        kind: SectionRenderLineKind::Item,
        action: None,
    };

    assert!(git_line_is_untracked(Some(&meta)));
    assert!(!git_line_is_untracked(Some(&staged_meta)));
    assert!(!git_line_is_untracked(None));
}

#[test]
fn git_status_commit_message_spans_use_command_token_with_icon_prefix() {
    let line = SectionRenderLine {
        text: format!(
            "{} Press c to commit staged changes.",
            editor_icons::symbols::cod::COD_GIT_COMMIT
        ),
        depth: 1,
        section_id: GIT_SECTION_COMMIT.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![(
            TOKEN_GIT_STATUS_COMMAND.to_owned(),
            format!(
                "{} Press c to commit staged changes.",
                editor_icons::symbols::cod::COD_GIT_COMMIT
            ),
        )]
    );
}

#[test]
fn hover_registry_includes_signature_help_provider() {
    let user_library = editor_plugin_host::NullUserLibrary;
    let registry = HoverRegistry::from_user_config(&user_library);
    assert!(matches!(registry.providers[0].kind, HoverProviderKind::Lsp));
    assert!(matches!(
        registry.providers[1].kind,
        HoverProviderKind::SignatureHelp
    ));
    assert_eq!(registry.providers[1].label, "Signature");
    assert_eq!(
        registry.providers[1].icon,
        user_library.hover_signature_icon()
    );
    assert!(matches!(
        registry.providers[2].kind,
        HoverProviderKind::Diagnostics
    ));
}

#[test]
fn statusline_icon_segments_split_acp_and_lsp_icons() {
    let user_library = user::UserLibraryImpl;
    let acp_icon = editor_icons::symbols::fa::FA_CONNECTDEVELOP;
    let lsp_icon = user_library.statusline_lsp_connected_icon();
    let statusline = format!("NORMAL | {acp_icon} | Ln 3, Col 9 | {lsp_icon} rust-analyzer");
    assert_eq!(
        statusline_icon_segments(&statusline, &[acp_icon, lsp_icon]),
        vec![
            ("NORMAL | ", false),
            (acp_icon, true),
            (" | Ln 3, Col 9 | ", false),
            (lsp_icon, true),
            (" rust-analyzer", false),
        ]
    );
}

#[test]
fn statusline_icon_segments_split_diagnostic_icons() {
    let user_library = user::UserLibraryImpl;
    let lsp_icon = user_library.statusline_lsp_connected_icon();
    let error_icon = user_library.statusline_lsp_error_icon();
    let warning_icon = user_library.statusline_lsp_warning_icon();
    let prefix = format!("NORMAL | {lsp_icon} rust-analyzer ");
    let statusline = format!("NORMAL | {lsp_icon} rust-analyzer {error_icon} 2 {warning_icon} 4");
    assert_eq!(
        statusline_icon_segments(&statusline, &[error_icon, warning_icon]),
        vec![
            (prefix.as_str(), false),
            (error_icon, true),
            (" 2 ", false),
            (warning_icon, true),
            (" 4", false),
        ]
    );
}

#[test]
fn notification_center_updates_entries_and_expires_completed_toasts() {
    let now = Instant::now();
    let mut center = NotificationCenter::default();
    assert!(center.apply(
        test_notification_update(
            "progress",
            NotificationSeverity::Info,
            "LSP · rust-analyzer",
            &["Indexing", "Scanning workspace"],
            Some(24),
            true,
        ),
        now,
    ));
    assert_eq!(center.visible(now).len(), 1);
    assert!(center.visible(now)[0].active);

    assert!(center.apply(
        test_notification_update(
            "progress",
            NotificationSeverity::Success,
            "LSP · rust-analyzer",
            &["Indexed workspace"],
            Some(100),
            false,
        ),
        now + Duration::from_millis(25),
    ));
    let visible = center.visible(now + Duration::from_millis(25));
    assert_eq!(visible.len(), 1);
    assert!(!visible[0].active);
    assert_eq!(visible[0].severity, NotificationSeverity::Success);

    assert!(!center.prune_expired(now + NOTIFICATION_AUTO_DISMISS - Duration::from_millis(1)));
    assert!(center.prune_expired(now + NOTIFICATION_AUTO_DISMISS + Duration::from_millis(50)));
    assert!(
        center
            .visible(now + NOTIFICATION_AUTO_DISMISS + Duration::from_millis(50))
            .is_empty()
    );
}

#[test]
fn notification_center_prioritizes_active_toasts_with_visible_limit() {
    let now = Instant::now();
    let mut center = NotificationCenter::default();
    assert!(center.apply(
        test_notification_update(
            "old-complete",
            NotificationSeverity::Success,
            "Done",
            &["Completed task"],
            None,
            false,
        ),
        now,
    ));
    assert!(center.apply(
        test_notification_update(
            "active-a",
            NotificationSeverity::Info,
            "Active A",
            &["Working"],
            Some(10),
            true,
        ),
        now + Duration::from_millis(10),
    ));
    assert!(center.apply(
        test_notification_update(
            "active-b",
            NotificationSeverity::Info,
            "Active B",
            &["Working"],
            Some(40),
            true,
        ),
        now + Duration::from_millis(20),
    ));
    assert!(center.apply(
        test_notification_update(
            "active-c",
            NotificationSeverity::Warning,
            "Active C",
            &["Working"],
            None,
            true,
        ),
        now + Duration::from_millis(30),
    ));
    assert!(center.apply(
        test_notification_update(
            "new-complete",
            NotificationSeverity::Success,
            "Done",
            &["Completed task"],
            None,
            false,
        ),
        now + Duration::from_millis(40),
    ));

    let visible = center.visible(now + Duration::from_millis(40));
    assert_eq!(visible.len(), NOTIFICATION_VISIBLE_LIMIT);
    assert!(visible.iter().all(|notification| notification.active));
    assert_eq!(visible[0].key, "active-c");
    assert_eq!(visible[1].key, "active-b");
    assert_eq!(visible[2].key, "active-a");
}

#[test]
fn notification_action_at_point_returns_acp_permission_action() -> Result<(), String> {
    let now = Instant::now();
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "acp.permission.42".to_owned(),
            severity: NotificationSeverity::Warning,
            title: "project Read file is requesting permission".to_owned(),
            body_lines: vec!["Allow once".to_owned(), "Reject once".to_owned()],
            progress: None,
            active: true,
            action: Some(NotificationAction::OpenAcpPermissionPicker { request_id: 42 }),
            workspace_id: None,
        },
        now,
    );

    let ui = shell_ui(&state.runtime)?;
    let layouts = notification_overlay_layouts(
        &ui.visible_notifications(now),
        render_width,
        render_height,
        cell_width,
        line_height,
    );
    let rect = layouts
        .first()
        .map(|layout| layout.rect)
        .ok_or_else(|| "notification layout missing".to_owned())?;
    let action = notification_action_at_point(
        ui,
        render_width,
        render_height,
        cell_width,
        line_height,
        now,
        (rect.x() + 4, rect.y() + 4),
    );

    assert_eq!(
        action,
        Some(NotificationAction::OpenAcpPermissionPicker { request_id: 42 })
    );
    Ok(())
}

#[test]
fn notification_action_at_point_returns_copilot_sign_in_action() -> Result<(), String> {
    let now = Instant::now();
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "copilot.sign-in".to_owned(),
            severity: NotificationSeverity::Error,
            title: "Copilot authentication required".to_owned(),
            body_lines: vec!["Click notification to sign in.".to_owned()],
            progress: None,
            active: true,
            action: Some(NotificationAction::CopilotSignIn {
                root: Some(PathBuf::from(r"P:\volt")),
            }),
            workspace_id: None,
        },
        now,
    );

    let ui = shell_ui(&state.runtime)?;
    let layouts = notification_overlay_layouts(
        &ui.visible_notifications(now),
        render_width,
        render_height,
        cell_width,
        line_height,
    );
    let rect = layouts
        .first()
        .map(|layout| layout.rect)
        .ok_or_else(|| "notification layout missing".to_owned())?;
    let action = notification_action_at_point(
        ui,
        render_width,
        render_height,
        cell_width,
        line_height,
        now,
        (rect.x() + 4, rect.y() + 4),
    );

    assert_eq!(
        action,
        Some(NotificationAction::CopilotSignIn {
            root: Some(PathBuf::from(r"P:\volt")),
        })
    );
    Ok(())
}

#[test]
fn copilot_auth_notification_shows_device_code_and_stays_active() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let key = copilot_status_notification_key(Some(Path::new(r"P:\volt")));
    apply_copilot_auth_notification(
        &mut state.runtime,
        &key,
        NotificationSeverity::Info,
        "Copilot sign-in started",
        vec![
            "Device code: ABCD-EFGH".to_owned(),
            "Code copied to clipboard.".to_owned(),
            "Enter code in GitHub browser flow.".to_owned(),
        ],
        true,
    )?;

    let now = Instant::now();
    let ui = shell_ui(&state.runtime)?;
    let notification = ui
        .visible_notifications(now)
        .into_iter()
        .find(|notification| notification.key == key)
        .ok_or_else(|| "copilot auth notification missing".to_owned())?;

    assert_eq!(notification.body_lines[0], "Device code: ABCD-EFGH");
    assert!(notification.active);
    Ok(())
}

#[test]
fn acp_section_layout_orders_output_input_footer_and_statusline() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(
        &mut state,
        40,
        "",
        Some("chat · gpt-5.4 · shift+tab switch mode"),
    )?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 18, 8);
    let acp_layout = acp_buffer_layout(buffer, rect, layout, 8, 18)
        .ok_or_else(|| "ACP layout missing".to_owned())?;

    assert!(
        acp_layout.output.rect.y() + acp_layout.output.rect.height() as i32
            <= acp_layout.input.rect.y()
    );
    assert!(
        acp_layout.input.rect.y() + acp_layout.input.rect.height() as i32
            <= acp_layout.footer.rect.y()
    );
    assert!(
        acp_layout.footer.rect.y() + acp_layout.footer.rect.height() as i32 <= layout.pane_bottom
    );
    assert_eq!(
        acp_layout.input.rect.height() as i32,
        18 + input_panel_chrome_height()
    );
    Ok(())
}

#[test]
fn browser_input_layout_uses_symmetric_vertical_padding() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 18, 8);
    let browser_layout = browser_buffer_layout(buffer, rect, layout, 8, 18)
        .ok_or_else(|| "browser layout missing".to_owned())?;

    assert_eq!(
        browser_layout.input.rect.height() as i32,
        18 + input_panel_chrome_height()
    );
    Ok(())
}

#[test]
fn render_browser_input_cursor_uses_rounded_rect_in_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let cursor_color = Color::RGB(7, 77, 177);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?;
        input.set_text("volt");
        input.cursor = 2;
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let browser_layout = browser_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "browser layout missing".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_browser_buffer_body(
        &mut target,
        BrowserBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(55, 71, 99, 255),
            cursor: cursor_color,
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    let cursor_color = to_render_color(cursor_color);
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x >= browser_layout.input.rect.x()
                && rect.x < browser_layout.input.rect.x() + browser_layout.input.rect.width() as i32
                && rect.y >= browser_layout.input.rect.y()
                && rect.y < browser_layout.input.rect.y() + browser_layout.input.rect.height() as i32
                && rect.width == 8
                && rect.height == 16
                && *color == cursor_color
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x >= browser_layout.input.rect.x()
                && rect.x < browser_layout.input.rect.x() + browser_layout.input.rect.width() as i32
                && rect.y >= browser_layout.input.rect.y()
                && rect.y < browser_layout.input.rect.y() + browser_layout.input.rect.height() as i32
                && rect.width == 8
                && rect.height == 16
                && *color == cursor_color
    )));
    Ok(())
}

#[test]
fn command_line_footer_layout_reserves_row_below_statusline() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout_with_command_line(buffer, rect, 18, 8, true);
    let commandline_y = layout
        .commandline_y
        .ok_or_else(|| "command line row is missing".to_owned())?;

    assert!(layout.statusline_y < commandline_y);
    assert_eq!(commandline_y - layout.statusline_y, 26);
    Ok(())
}

#[test]
fn render_buffer_draws_command_line_row_without_active_overlay() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout_with_command_line(buffer, rect, 16, 8, true);
    let commandline_y = layout
        .commandline_y
        .ok_or_else(|| "command line row is missing".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: true,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == 8
                && rect.y == commandline_y - 6
                && rect.width == 304
                && rect.height == 1
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == 8
                && rect.y == commandline_y
                && rect.width == 304
                && rect.height == 16
    )));
    Ok(())
}

#[test]
fn render_buffer_draws_show_paren_match_highlight() -> Result<(), String> {
    let match_color = Color::RGBA(12, 34, 56, 128);
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme").with_token(
                TOKEN_SHOW_PAREN_MATCH,
                editor_theme::Color::rgba(12, 34, 56, 128),
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*show-paren*", vec!["call(foo)".to_owned()])?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 4));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "test-theme",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRoundedRect { color, .. }
                if *color == to_render_color(match_color)
        )),
        "expected show-paren match highlight, scene={scene:?}"
    );
    Ok(())
}

#[test]
fn render_buffer_draws_show_paren_html_tag_highlight() -> Result<(), String> {
    let match_color = Color::RGBA(9, 8, 7, 120);
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme").with_token(
                TOKEN_SHOW_PAREN_MATCH,
                editor_theme::Color::rgba(9, 8, 7, 120),
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*show-paren-html*",
        vec!["<div>hi</div>".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 1));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "test-theme",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRoundedRect { color, .. }
                if *color == to_render_color(match_color)
        )),
        "expected show-paren HTML tag highlight, scene={scene:?}"
    );
    Ok(())
}

#[test]
fn render_terminal_buffer_path_draws_command_line_separator_without_footer_fill()
-> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            1,
            12,
            vec![editor_terminal::TerminalRenderLine::new(vec![
                editor_terminal::TerminalRenderRun::new(
                    0,
                    11,
                    "echo hello",
                    editor_terminal::TerminalRgb {
                        r: 215,
                        g: 221,
                        b: 232,
                    },
                    None,
                    None,
                ),
            ])],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                0,
                0,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout_with_command_line(buffer, rect, 16, 8, true);
    let commandline_y = layout
        .commandline_y
        .ok_or_else(|| "command line row is missing".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: true,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == 8
                && rect.y == commandline_y - 6
                && rect.width == 304
                && rect.height == 1
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == 8
                && rect.y == commandline_y
                && rect.width == 304
                && rect.height == 16
    )));
    Ok(())
}

#[test]
fn render_buffer_uses_theme_commandline_background_token() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme").with_token(
                TOKEN_COMMANDLINE_BACKGROUND,
                editor_theme::Color::rgba(10, 20, 30, 144),
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout_with_command_line(buffer, rect, 16, 8, true);
    let commandline_y = layout
        .commandline_y
        .ok_or_else(|| "command line row is missing".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    let mut command_line = CommandLineOverlay::new();
    command_line.append_text("w");
    let command_line_input = command_line.input();
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: Some(command_line_input),
                row_visible: true,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "test-theme",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == 8
                && rect.y == commandline_y
                && rect.width == 304
                && rect.height == 16
                && *color == to_render_color(Color::RGBA(10, 20, 30, 144))
    )));
    Ok(())
}

#[test]
fn render_buffer_falls_back_to_statusline_theme_tokens_for_text() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    let active_text = Color::RGB(10, 20, 30);
    let inactive_text = Color::RGB(40, 50, 60);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(
                    TOKEN_STATUSLINE_ACTIVE,
                    editor_theme::Color::rgb(active_text.r, active_text.g, active_text.b),
                )
                .with_token(
                    TOKEN_STATUSLINE_INACTIVE,
                    editor_theme::Color::rgb(inactive_text.r, inactive_text.g, inactive_text.b),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let render_user_library = HeaderlineTestUserLibrary::default();
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);

    let mut active_scene = Vec::new();
    let mut active_target = DrawTarget::Scene(&mut active_scene);
    render_buffer(
        &mut active_target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(active_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, color, .. }
            if *y == layout.statusline_y && *color == to_render_color(active_text)
    )));

    let mut inactive_scene = Vec::new();
    let mut inactive_target = DrawTarget::Scene(&mut inactive_scene);
    render_buffer(
        &mut inactive_target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot {
                rect,
                active: false,
            },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(inactive_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, color, .. }
            if *y == layout.statusline_y && *color == to_render_color(inactive_text)
    )));
    Ok(())
}

#[test]
fn render_buffer_paints_modeline_mode_chip_and_right_aligned_segment() -> Result<(), String> {
    struct ModelineChipTestUserLibrary;

    impl UserLibrary for ModelineChipTestUserLibrary {
        fn modeline_segments(
            &self,
            context: &StatuslineContext<'_>,
        ) -> Vec<editor_plugin_api::ModelineSegment> {
            use editor_plugin_api::{ModelinePart, ModelineSegment};
            vec![
                ModelineSegment::left(vec![ModelinePart::new(
                    format!(" {} ", context.vim_mode),
                    "ui.modeline.mode.normal.foreground",
                    Some("ui.modeline.mode.normal.background".into()),
                )]),
                ModelineSegment::left(vec![ModelinePart::fg(
                    format!("{up} 2", up = editor_icons::symbols::cod::COD_ARROW_UP),
                    "ui.modeline.git.added",
                )]),
                ModelineSegment::right(vec![ModelinePart::fg("RHS", "ui.modeline.muted")]),
            ]
        }
    }

    let mut registry = ThemeRegistry::new();
    let mode_fg = Color::RGB(10, 10, 10);
    let mode_bg = Color::RGB(90, 160, 255);
    let git_added = Color::RGB(50, 200, 80);
    let muted = Color::RGB(120, 120, 130);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(TOKEN_STATUSLINE_ACTIVE, editor_theme::Color::rgb(1, 2, 3))
                .with_token(
                    TOKEN_STATUSLINE_FOREGROUND,
                    editor_theme::Color::rgb(200, 200, 200),
                )
                .with_token(
                    "ui.modeline.mode.normal.foreground",
                    editor_theme::Color::rgb(mode_fg.r, mode_fg.g, mode_fg.b),
                )
                .with_token(
                    "ui.modeline.mode.normal.background",
                    editor_theme::Color::rgb(mode_bg.r, mode_bg.g, mode_bg.b),
                )
                .with_token(
                    "ui.modeline.git.added",
                    editor_theme::Color::rgb(git_added.r, git_added.g, git_added.b),
                )
                .with_token(
                    "ui.modeline.muted",
                    editor_theme::Color::rgb(muted.r, muted.g, muted.b),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let render_user_library = ModelineChipTestUserLibrary;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let statusline_x = rect.x() + 12;
    let max_width = rect.width().saturating_sub(24);
    let rhs_width = monospace_text_width("RHS", 8);
    let expected_rhs_x = statusline_x + max_width.saturating_sub(rhs_width) as i32;

    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRoundedRect { rect, color, .. }
                if rect.y == layout.statusline_y
                    && *color == to_render_color(mode_bg)
        )),
        "expected mode chip background fill"
    );
    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { y, color, text, .. }
                if *y == layout.statusline_y
                    && *color == to_render_color(mode_fg)
                    && text.contains("NORMAL")
        )),
        "expected mode chip foreground text"
    );
    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { y, color, .. }
                if *y == layout.statusline_y && *color == to_render_color(git_added)
        )),
        "expected git added color"
    );
    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { x, y, color, text, .. }
                if *x == expected_rhs_x
                    && *y == layout.statusline_y
                    && *color == to_render_color(muted)
                    && text == "RHS"
        )),
        "expected right-aligned RHS segment"
    );
    Ok(())
}

#[test]
fn render_buffer_uses_statusline_foreground_tokens() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    let active_text = Color::RGB(212, 218, 226);
    let inactive_text = Color::RGB(148, 154, 164);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(
                    TOKEN_STATUSLINE_ACTIVE,
                    editor_theme::Color::rgb(10, 20, 30),
                )
                .with_token(
                    TOKEN_STATUSLINE_INACTIVE,
                    editor_theme::Color::rgb(40, 50, 60),
                )
                .with_token(
                    TOKEN_STATUSLINE_FOREGROUND,
                    editor_theme::Color::rgb(active_text.r, active_text.g, active_text.b),
                )
                .with_token(
                    TOKEN_STATUSLINE_INACTIVE_FOREGROUND,
                    editor_theme::Color::rgb(inactive_text.r, inactive_text.g, inactive_text.b),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let render_user_library = HeaderlineTestUserLibrary::default();
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);

    let mut active_scene = Vec::new();
    let mut active_target = DrawTarget::Scene(&mut active_scene);
    render_buffer(
        &mut active_target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(active_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, color, .. }
            if *y == layout.statusline_y && *color == to_render_color(active_text)
    )));

    let mut inactive_scene = Vec::new();
    let mut inactive_target = DrawTarget::Scene(&mut inactive_scene);
    render_buffer(
        &mut inactive_target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot {
                rect,
                active: false,
            },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(inactive_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, color, .. }
            if *y == layout.statusline_y && *color == to_render_color(inactive_text)
    )));
    Ok(())
}

#[test]
fn render_shell_state_uses_theme_background_for_active_pane() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let base_background = Color::RGB(15, 16, 20);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        &[],
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 320,
                height: 180,
            },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 0
                && rect.y == 0
                && rect.width == 320
                && rect.height == 180
                && *color == to_render_color(base_background)
    )));
    Ok(())
}

#[test]
fn render_shell_state_applies_window_opacity_only_to_backgrounds() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        &[],
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 320,
                height: 180,
            },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Clear { color } if color.a == 128
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 0
                && rect.y == 0
                && rect.width == 320
                && rect.height == 180
                && color.a == 128
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { color, .. } if color.a == 255
    )));
    Ok(())
}

#[test]
fn render_shell_state_draws_fps_overlay_when_enabled() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let fps_overlay = FpsOverlaySnapshot {
        latest_frame_time: Duration::from_nanos(8_100_000),
        average_frame_time: Duration::from_nanos(8_300_000),
        worst_frame_time: Duration::from_nanos(10_200_000),
    };

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        &[],
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 640,
                height: 360,
            },
            fps_overlay: Some(&fps_overlay),
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { text, .. } if text.contains("FPS")
    )));
    Ok(())
}

fn render_shell_state_scene_with_docked_runtime_popup(
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(Vec<DrawCommand>, Rect), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_popup_test_buffer(&mut state)?;
    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "runtime popup was not opened".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let width = 320;
    let height = 180;
    let cell_width = 8;
    let line_height = 16;
    let popup_height = popup_window_height(height, line_height);
    let popup_rect = PixelRectToRect::rect(
        0,
        height.saturating_sub(popup_height) as i32,
        width,
        popup_height,
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        Some(&popup),
        &[],
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize { width, height },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width,
                line_height,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    Ok((scene, popup_rect))
}

#[test]
fn render_shell_state_uses_theme_background_for_docked_runtime_popup_surface() -> Result<(), String>
{
    let base_background = Color::RGB(15, 16, 20);
    let (scene, popup_rect) = render_shell_state_scene_with_docked_runtime_popup(None)?;
    let popup_surface_fills = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::FillRoundedRect { rect, color, .. }
                if rect.x == popup_rect.x()
                    && rect.y == popup_rect.y()
                    && rect.width == popup_rect.width()
                    && rect.height == popup_rect.height() =>
            {
                Some(*color)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(popup_surface_fills, vec![to_render_color(base_background)]);
    Ok(())
}

#[test]
fn render_shell_state_uses_opaque_overlay_chrome_for_docked_runtime_popup_surface()
-> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let (scene, popup_rect) = render_shell_state_scene_with_docked_runtime_popup(Some(&registry))?;
    let popup_surface_fills = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::FillRoundedRect { rect, color, .. }
                if rect.x == popup_rect.x()
                    && rect.y == popup_rect.y()
                    && rect.width == popup_rect.width()
                    && rect.height == popup_rect.height() =>
            {
                Some(*color)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        popup_surface_fills,
        vec![to_render_color(Color::RGBA(15, 16, 20, 255))]
    );
    Ok(())
}

fn render_shell_state_scene_with_notification_overlay(
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(Vec<DrawCommand>, Rect), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let now = Instant::now();
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "toast".to_owned(),
            severity: NotificationSeverity::Info,
            title: "Overlay".to_owned(),
            body_lines: vec!["Readability check".to_owned()],
            progress: None,
            active: true,
            action: None,
            workspace_id: None,
        },
        now,
    );
    let ui = shell_ui(&state.runtime)?;
    let width = 320;
    let height = 180;
    let cell_width = 8;
    let line_height = 16;
    let rect = notification_overlay_layouts(
        &ui.visible_notifications(now),
        width,
        height,
        cell_width,
        line_height,
    )
    .first()
    .map(|layout| layout.rect)
    .ok_or_else(|| "notification overlay was not created".to_owned())?;

    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        &[],
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize { width, height },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width,
                line_height,
                ascent: 12,
            },
            pulse: FramePulse {
                now,
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    Ok((scene, rect))
}

#[test]
fn render_shell_state_uses_opaque_overlay_chrome_for_notification_surface() -> Result<(), String> {
    let base_background = Color::RGB(15, 16, 20);
    let expected_background = adjust_color(base_background, 18);
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let (scene, notification_rect) =
        render_shell_state_scene_with_notification_overlay(Some(&registry))?;
    let body_x = notification_rect.x + 1 + OVERLAY_ACCENT_BAR_WIDTH as i32;
    let notification_surface_fills = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::FillRoundedRect { rect, color, .. }
                if rect.x == body_x
                    && rect.y == notification_rect.y + 1
                    && rect.width
                        == notification_rect
                            .width()
                            .saturating_sub(2)
                            .saturating_sub(OVERLAY_ACCENT_BAR_WIDTH)
                    && rect.height == notification_rect.height().saturating_sub(2) =>
            {
                Some(*color)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        notification_surface_fills,
        vec![to_render_color(Color::RGBA(
            expected_background.r,
            expected_background.g,
            expected_background.b,
            255,
        ))]
    );
    Ok(())
}

#[test]
fn theme_runtime_settings_resolve_window_effects_from_theme_options() {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.65)
                .with_option(crate::window_effects::OPTION_WINDOW_BLUR, 18.0),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let settings = theme_runtime_settings(Some(&registry), &ShellConfig::default(), 1.0);

    assert_eq!(
        settings.window_effects,
        crate::window_effects::WindowEffects {
            opacity: 0.65,
            blur: 18.0,
        }
    );
}

#[test]
fn render_picker_overlay_uses_opaque_overlay_chrome() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let picker = PickerOverlay::from_entries(
        "Projects",
        vec![PickerEntry {
            item: PickerItem::new(
                ".config",
                ".config",
                "git",
                Some("C:\\Users\\sam\\.config".to_owned()),
            ),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    picker::render_picker_overlay(
        &mut target,
        &fonts,
        PickerOverlayDraw {
            picker: &picker,
            size: WindowSize {
                width: 320,
                height: 180,
            },
            line_height: 16,
            theme_registry: Some(&registry),
            picker_layout: editor_plugin_api::PickerLayout::default(),
            truncate_strategy: editor_plugin_api::PickerTruncateStrategy::Auto,
        },
    )
    .map_err(|error| error.to_string())?;

    let popup_rect = picker_card_rect(320, 180, editor_plugin_api::PickerLayout::default());
    let inner_x = popup_rect.x + 1;
    let inner_y = popup_rect.y + 1;
    let inner_height = popup_rect.height.saturating_sub(2);
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == inner_x + OVERLAY_ACCENT_BAR_WIDTH as i32
                && rect.y == inner_y
                && rect.width
                    == popup_rect
                        .width
                        .saturating_sub(2)
                        .saturating_sub(OVERLAY_ACCENT_BAR_WIDTH)
                && rect.height == inner_height
                && *color == to_render_color(Color::RGBA(15, 16, 20, 255))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { color, .. } if color.a == 255
    )));
    Ok(())
}

#[test]
fn render_autocomplete_overlay_uses_opaque_overlay_chrome() -> Result<(), String> {
    let _guard = crate::window_effects::force_surface_window_opacity_for_tests();
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*autocomplete-overlay*",
        vec!["alpha".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 5));

    let overlay = AutocompleteOverlay {
        buffer_id,
        buffer_revision: 0,
        query: AutocompleteQuery {
            prefix: String::new(),
            token: "alpha".to_owned(),
            replace_range: TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
        },
        entries: vec![AutocompleteEntry {
            provider_id: "manual".to_owned(),
            provider_label: "Manual".to_owned(),
            provider_icon: "M".to_owned(),
            item_icon: "•".to_owned(),
            label: "alpha".to_owned(),
            replacement: "alpha".to_owned(),
            replace_range: None,
            detail: Some("detail".to_owned()),
            documentation: Some("documentation".to_owned()),
        }],
        selected_index: 0,
        loading: false,
    };
    let base_background = theme_color(Some(&registry), "ui.background", Color::RGB(15, 16, 20));
    let is_dark = is_dark_color(base_background);
    let accent = theme_color(
        Some(&registry),
        "ui.selection",
        adjust_color(base_background, if is_dark { 48 } else { -48 }),
    );
    let panel_background = theme_color(
        Some(&registry),
        "ui.autocomplete.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let selected_background = theme_color(
        Some(&registry),
        "ui.autocomplete.selection",
        blend_color(accent, panel_background, 0.72),
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_autocomplete_overlay(
        &mut target,
        shell_ui(&state.runtime)?,
        &overlay,
        OverlayAnchorContext {
            pane_rect: PixelRectToRect::rect(0, 0, 640, 360),
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 16,
            },
            typing_active: false,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    panel_background.r,
                    panel_background.g,
                    panel_background.b,
                    255,
                ))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    selected_background.r,
                    selected_background.g,
                    selected_background.b,
                    255,
                ))
    )));
    Ok(())
}

#[test]
fn render_hover_overlay_uses_opaque_overlay_chrome() -> Result<(), String> {
    let _guard = crate::window_effects::force_surface_window_opacity_for_tests();
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_text_test_buffer(&mut state, "*hover-overlay*", vec!["hover".to_owned()])?;
    install_hover_test_overlay(&mut state, false)?;
    let hover = shell_ui(&state.runtime)?
        .hover()
        .cloned()
        .ok_or_else(|| "hover overlay missing".to_owned())?;

    let base_background = theme_color(Some(&registry), "ui.background", Color::RGB(15, 16, 20));
    let base_foreground = theme_color(
        Some(&registry),
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let accent = theme_color(
        Some(&registry),
        "ui.selection",
        adjust_color(base_background, if is_dark { 48 } else { -48 }),
    );
    let background = theme_color(
        Some(&registry),
        "ui.hover.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let header_background = theme_color(
        Some(&registry),
        "ui.hover.header.background",
        adjust_color(background, if is_dark { 6 } else { -6 }),
    );
    let selected_tab = theme_color(
        Some(&registry),
        "ui.hover.selection",
        blend_color(accent, header_background, 0.68),
    );
    let _muted = theme_color(
        Some(&registry),
        "ui.hover.muted",
        blend_color(base_foreground, background, 0.46),
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_hover_overlay(
        &mut target,
        shell_ui(&state.runtime)?,
        &hover,
        OverlayAnchorContext {
            pane_rect: PixelRectToRect::rect(0, 0, 640, 360),
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 16,
            },
            typing_active: false,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    background.r,
                    background.g,
                    background.b,
                    255,
                ))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    header_background.r,
                    header_background.g,
                    header_background.b,
                    255,
                ))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    selected_tab.r,
                    selected_tab.g,
                    selected_tab.b,
                    255,
                ))
    )));
    Ok(())
}

#[test]
fn render_picker_overlay_uses_picker_text_tokens() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    let picker_foreground = Color::RGB(220, 224, 230);
    let picker_muted = Color::RGB(176, 182, 191);
    let picker_subtle = Color::RGB(138, 144, 154);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(
                    TOKEN_PICKER_FOREGROUND,
                    editor_theme::Color::rgb(
                        picker_foreground.r,
                        picker_foreground.g,
                        picker_foreground.b,
                    ),
                )
                .with_token(
                    TOKEN_PICKER_MUTED,
                    editor_theme::Color::rgb(picker_muted.r, picker_muted.g, picker_muted.b),
                )
                .with_token(
                    TOKEN_PICKER_SUBTLE,
                    editor_theme::Color::rgb(picker_subtle.r, picker_subtle.g, picker_subtle.b),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let picker = PickerOverlay::from_entries(
        "Projects",
        vec![
            PickerEntry {
                item: PickerItem::new("alpha", "alpha", "one", None::<String>),
                action: PickerAction::NoOp,
                quickfix: None,
            },
            PickerEntry {
                item: PickerItem::new("beta", "beta", "two", None::<String>),
                action: PickerAction::NoOp,
                quickfix: None,
            },
        ],
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    picker::render_picker_overlay(
        &mut target,
        &fonts,
        PickerOverlayDraw {
            picker: &picker,
            size: WindowSize {
                width: 640,
                height: 360,
            },
            line_height: 16,
            theme_registry: Some(&registry),
            picker_layout: editor_plugin_api::PickerLayout::default(),
            truncate_strategy: editor_plugin_api::PickerTruncateStrategy::Auto,
        },
    )
    .map_err(|error| error.to_string())?;

    let expected_unselected_label = blend_color(picker_foreground, Color::RGB(15, 16, 20), 0.12);
    let text_commands = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, color, .. } => Some((text.clone(), *color)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(
        text_commands.iter().any(|(text, color)| {
            text == "Projects" && *color == to_render_color(picker_foreground)
        }),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands
            .iter()
            .any(|(text, color)| text == "alpha" && *color == to_render_color(picker_foreground)),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands.iter().any(|(text, color)| {
            text == "beta" && *color == to_render_color(expected_unselected_label)
        }),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands
            .iter()
            .any(|(text, color)| text == "filter" && *color == to_render_color(picker_muted)),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands
            .iter()
            .any(|(text, color)| text == "two" && *color == to_render_color(picker_muted)),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands.iter().any(|(text, color)| {
            text == "2 / 2 results" && *color == to_render_color(picker_subtle)
        }),
        "unexpected picker text colors: {text_commands:?}"
    );
    Ok(())
}

#[test]
fn preferred_primary_font_hinting_matches_transparent_window_policy() {
    if cfg!(target_os = "windows") {
        assert!(matches!(
            preferred_primary_font_hinting(),
            Some(Hinting::NONE)
        ));
    } else {
        assert!(preferred_primary_font_hinting().is_none());
    }
}

#[test]
fn rebuild_theme_registry_preserves_active_theme_when_still_present() {
    let registry = rebuild_theme_registry(
        vec![
            editor_theme::Theme::new("default", "Default"),
            editor_theme::Theme::new("night", "Night")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.55),
        ],
        Some("night"),
    )
    .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    assert_eq!(
        registry.active_theme().map(|theme| theme.id()),
        Some("night")
    );
    assert_eq!(
        registry.resolve_number(crate::window_effects::OPTION_WINDOW_OPACITY),
        Some(0.55)
    );
}

#[test]
fn theme_source_fingerprint_from_dir_changes_when_global_toml_changes() -> Result<(), String> {
    let temp = TempTestDir::new("theme-source-fingerprint");
    let themes_dir = temp.path().join("user").join("themes");
    fs::create_dir_all(&themes_dir).map_err(|error| error.to_string())?;
    let global = themes_dir.join("global.toml");
    fs::write(&global, "[options]\n\"window.opacity\" = 1.0\n")
        .map_err(|error| error.to_string())?;

    let before = theme_source_fingerprint_from_dir(&themes_dir)
        .ok_or_else(|| "missing initial theme fingerprint".to_owned())?;

    thread::sleep(Duration::from_millis(20));
    fs::write(
        &global,
        "[options]\n\"window.opacity\" = 0.35\n\"window.blur\" = 12.0\n",
    )
    .map_err(|error| error.to_string())?;

    let after = theme_source_fingerprint_from_dir(&themes_dir)
        .ok_or_else(|| "missing updated theme fingerprint".to_owned())?;

    assert_ne!(before, after);
    Ok(())
}

#[test]
fn user_config_source_fingerprint_changes_when_child_yaml_changes() -> Result<(), String> {
    let temp = TempTestDir::new("user-config-source-fingerprint");
    let user_dir = temp.path().join("user");
    let config_dir = user_dir.join("config");
    fs::create_dir_all(&config_dir).map_err(|error| error.to_string())?;
    fs::write(
        user_dir.join("config.yaml"),
        "workspace: config/workspace.yaml\nui: config/ui.yaml\n",
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        config_dir.join("workspace.yaml"),
        "project_search_roots:\n  - path: P:/\n    max_depth: 4\n",
    )
    .map_err(|error| error.to_string())?;
    let ui = config_dir.join("ui.yaml");
    fs::write(&ui, "ligatures_enabled: true\n").map_err(|error| error.to_string())?;

    let before = user_config_source_fingerprint_from_files(vec![
        user_dir.join("config.yaml"),
        config_dir.join("workspace.yaml"),
        config_dir.join("ui.yaml"),
    ])
    .ok_or_else(|| "missing initial user config fingerprint".to_owned())?;

    thread::sleep(Duration::from_millis(20));
    fs::write(&ui, "ligatures_enabled: false\n").map_err(|error| error.to_string())?;

    let after = user_config_source_fingerprint_from_files(vec![
        user_dir.join("config.yaml"),
        config_dir.join("workspace.yaml"),
        config_dir.join("ui.yaml"),
    ])
    .ok_or_else(|| "missing updated user config fingerprint".to_owned())?;

    assert_ne!(before, after);
    Ok(())
}

#[test]
fn hidden_window_startup_smoke_supports_window_effects() -> Result<(), String> {
    let _guard = crate::window_effects::lock_window_effects_for_tests();
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let video = sdl_context.video().map_err(|error| error.to_string())?;
    crate::window_effects::configure_window_opacity_driver(Some(video.current_video_driver()));
    let window_effects = crate::window_effects::WindowEffects {
        opacity: 0.35,
        blur: 0.0,
    };

    let mut window_builder = video.window("Volt Smoke", 320, 180);
    window_builder.hidden().high_pixel_density();
    window_builder.set_flags(
        window_builder.flags() | crate::window_effects::window_creation_flags(window_effects),
    );
    let mut window = window_builder.build().map_err(|error| error.to_string())?;
    assert!(WindowFlags::from(window.window_flags()).contains(WindowFlags::HIGH_PIXEL_DENSITY));
    apply_window_effects(&mut window, window_effects).map_err(|error| error.to_string())?;

    let mut canvas = window.into_canvas();
    canvas.set_draw_color(Color::RGBA(29, 32, 40, 128));
    canvas.clear();
    canvas.present();

    let size = canvas.output_size().map_err(|error| error.to_string())?;
    assert_eq!(size, (320, 180));
    Ok(())
}

#[test]
fn scaled_font_size_uses_window_display_scale() {
    assert_eq!(scaled_font_size(18, 2.0), 36.0);
    assert_eq!(scaled_font_size(18, 1.25), 22.5);
    assert_eq!(scaled_font_size(18, -1.0), 18.0);
}

#[test]
fn normalized_raster_pixel_size_matches_target_line_height() {
    let pixel_size = normalized_raster_pixel_size(
        18.0,
        24,
        Some(fontdue::LineMetrics {
            ascent: 15.0,
            descent: -5.0,
            line_gap: 0.0,
            new_line_size: 20.0,
        }),
    );

    assert!((pixel_size - 21.6).abs() < f32::EPSILON);
}

#[test]
fn load_font_set_normalizes_icon_raster_sizes_to_primary_line_height() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 18,
            emoji_font_size: 18,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let primary_line_height = fonts.primary().height().max(1) as f32;

    for icon_font in fonts.icon_fonts() {
        let line_metrics = icon_font
            .raster_font
            .horizontal_line_metrics(icon_font.pixel_size)
            .ok_or_else(|| format!("icon font `{}` is missing line metrics", icon_font.name))?;
        let icon_line_height = line_metrics.ascent - line_metrics.descent;
        assert!(
            (icon_line_height - primary_line_height).abs() <= 1.0,
            "expected icon font `{}` to target line height {primary_line_height}, got {icon_line_height}",
            icon_font.name,
        );
    }
    Ok(())
}

#[test]
fn plugin_sections_layout_keeps_output_pane_at_bottom_with_single_row_start() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_plugin_sections_test_buffer(
        &mut state,
        &["a = 1", "b = 2", "sqrt(a + b)"],
        &["(press Ctrl+c Ctrl+c to evaluate)"],
    )?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 18, 8);
    let panes = plugin_section_buffer_layout(buffer, rect, layout, 8, 18)
        .ok_or_else(|| "plugin section layout missing".to_owned())?;

    assert_eq!(panes.panes[1].visible_rows, 1);
    assert!(panes.panes[0].rect.y() >= layout.body_y);
    assert!(
        panes.panes[0].rect.y() + panes.panes[0].rect.height() as i32 <= panes.panes[1].rect.y()
    );
    assert!(panes.panes[1].rect.y() + panes.panes[1].rect.height() as i32 <= layout.pane_bottom);
    Ok(())
}

#[test]
fn plugin_sections_layout_reserves_extra_bottom_padding() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_plugin_sections_test_buffer(
        &mut state,
        &["a = 1", "b = 2", "sqrt(a + b)"],
        &["(press Ctrl+c Ctrl+c to evaluate)"],
    )?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 18, 8);
    let panes = plugin_section_buffer_layout(buffer, rect, layout, 8, 18)
        .ok_or_else(|| "plugin section layout missing".to_owned())?;

    assert_eq!(
        panes.panes[1].rect.height(),
        (plugin_section_panel_chrome_height("Output", 18) + panes.panes[1].visible_rows as i32 * 18)
            as u32
    );
    Ok(())
}

#[test]
fn plugin_sections_switching_output_pane_changes_focus_and_read_only_state() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_plugin_sections_test_buffer(&mut state, &["a = 1"], &["1"])?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;

    assert_eq!(buffer.plugin_active_section_index(), Some(0));
    assert!(!buffer.is_read_only());

    assert!(buffer.plugin_switch_pane());
    assert_eq!(buffer.plugin_active_section_index(), Some(1));
    assert!(buffer.is_read_only());

    assert!(buffer.plugin_switch_pane());
    assert_eq!(buffer.plugin_active_section_index(), Some(0));
    assert!(!buffer.is_read_only());
    Ok(())
}

#[test]
fn calculator_ctrl_tab_switches_sections_without_changing_workspace_pane() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(
        &mut state,
        user::calculator::BUFFER_NAME,
        user::calculator::CALCULATOR_KIND,
    )?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    let active_pane_id = shell_ui(&state.runtime)?
        .active_pane_id()
        .ok_or_else(|| "active pane is missing".to_owned())?;

    let handled = state
        .try_runtime_keybinding(Keycode::Tab, ctrl_mod())
        .map_err(|error| error.to_string())?;

    assert!(handled);
    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(active_pane_id)
    );
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.plugin_active_section_index(), Some(1));
    assert!(buffer.is_read_only());
    Ok(())
}

#[test]
fn calculator_switch_pane_command_targets_workspace_buffer_when_popup_has_focus()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(
        &mut state,
        user::calculator::BUFFER_NAME,
        user::calculator::CALCULATOR_KIND,
    )?;
    let _popup_buffer_id = install_terminal_popup_test_buffer(&mut state)?;

    state
        .runtime
        .execute_command("calculator.switch-pane")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.plugin_active_section_index(), Some(1));
    assert!(buffer.is_read_only());
    Ok(())
}

#[test]
fn plugin_sections_replace_output_lines_in_place() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_plugin_sections_test_buffer(&mut state, &["a = 1"], &["old", "lines"])?;

    shell_buffer_mut(&mut state.runtime, buffer_id)?
        .set_plugin_output_lines(vec!["2".to_owned(), "3".to_owned()]);

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let state = buffer
        .plugin_sections()
        .ok_or_else(|| "plugin section state missing".to_owned())?;
    let output = state
        .attached_section(1)
        .ok_or_else(|| "output section missing".to_owned())?;
    let lines = (0..output.line_count())
        .map(|index| output.text.line(index).unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(lines, vec!["2", "3"]);
    Ok(())
}

#[test]
fn plugin_sections_can_append_output_lines() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_plugin_sections_test_buffer_with_update(
        &mut state,
        &["a = 1"],
        &["old"],
        editor_plugin_api::PluginBufferSectionUpdate::Append,
    )?;

    shell_buffer_mut(&mut state.runtime, buffer_id)?
        .set_plugin_output_lines(vec!["new".to_owned()]);

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let state = buffer
        .plugin_sections()
        .ok_or_else(|| "plugin section state missing".to_owned())?;
    let output = state
        .attached_section(1)
        .ok_or_else(|| "output section missing".to_owned())?;
    let lines = (0..output.line_count())
        .map(|index| output.text.line(index).unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(lines, vec!["old", "new"]);
    Ok(())
}

#[test]
fn render_plugin_sections_active_header_keeps_neutral_background() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_plugin_sections_test_buffer(&mut state, &["alpha"], &["beta"])?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let pane_layout = plugin_section_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "plugin section layout missing".to_owned())?;
    let header_height = (16 + 10) as u32;
    let header_rect = PixelRectToRect::rect(
        pane_layout.panes[0].rect.x() + 1,
        pane_layout.panes[0].rect.y() + 1,
        pane_layout.panes[0].rect.width().saturating_sub(2),
        header_height,
    );
    let base_background = Color::RGB(15, 16, 20);
    let panel_background = theme_color(
        None,
        "ui.panel.background",
        adjust_color(base_background, 8),
    );
    let header_background = theme_color(
        None,
        "ui.panel.header.background",
        adjust_color(panel_background, 12),
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_plugin_section_buffer_body(
        &mut target,
        PluginSectionDraw {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            layout,
            visual_selection: None,
            yank_flash: None,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background,
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == header_rect.x()
                && rect.y == header_rect.y()
                && rect.width == header_rect.width()
                && rect.height == header_rect.height()
                && *color == to_render_color(header_background)
    )));
    Ok(())
}

#[test]
fn render_plugin_sections_keep_opaque_overlay_chrome() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_plugin_sections_test_buffer(&mut state, &["alpha"], &["beta"])?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let pane_layout = plugin_section_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "plugin section layout missing".to_owned())?;
    let header_height = (16 + 10) as u32;
    let header_rect = PixelRectToRect::rect(
        pane_layout.panes[0].rect.x() + 1,
        pane_layout.panes[0].rect.y() + 1,
        pane_layout.panes[0].rect.width().saturating_sub(2),
        header_height,
    );
    let base_background = Color::RGB(15, 16, 20);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_plugin_section_buffer_body(
        &mut target,
        PluginSectionDraw {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            layout,
            visual_selection: None,
            yank_flash: None,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: Some(&registry),
            base_background,
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == pane_layout.panes[0].rect.x()
                && rect.y == pane_layout.panes[0].rect.y()
                && rect.width == pane_layout.panes[0].rect.width()
                && rect.height == pane_layout.panes[0].rect.height()
                && color.a == 255
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == header_rect.x()
                && rect.y == header_rect.y()
                && rect.width == header_rect.width()
                && rect.height == header_rect.height()
                && color.a == 255
    )));
    Ok(())
}

#[test]
fn render_acp_sections_keep_opaque_overlay_chrome() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let _ = buffer.focus_acp_input();

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let acp_layout = acp_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "missing ACP layout".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        AcpBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            visual_selection: None,
            yank_flash: None,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: Some(&registry),
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == acp_layout.input.rect.x()
                && rect.y == acp_layout.input.rect.y()
                && rect.width == acp_layout.input.rect.width()
                && rect.height == acp_layout.input.rect.height()
                && color.a == 255
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == acp_layout.plan.rect.x() + 1
                && rect.y == acp_layout.plan.rect.y() + 1
                && rect.width == acp_layout.plan.rect.width().saturating_sub(2)
                && color.a == 255
    )));
    Ok(())
}

#[test]
fn render_browser_selected_section_border_stays_opaque() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?;
        input.set_text("volt");
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let browser_layout = browser_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "browser layout missing".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_browser_buffer_body(
        &mut target,
        BrowserBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: Some(&registry),
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(55, 71, 99, 255),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == browser_layout.input.rect.x()
                && rect.y == browser_layout.input.rect.y()
                && rect.width == browser_layout.input.rect.width()
                && rect.height == browser_layout.input.rect.height()
                && color.a == 255
    )));
    Ok(())
}

#[test]
fn render_plugin_sections_draw_visual_selection_highlight() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id =
        install_plugin_sections_test_buffer(&mut state, &["alpha beta"], &["gamma delta"])?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    assert!(buffer.plugin_switch_pane());
    buffer.set_cursor(TextPoint::new(0, 5));

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let selection_color = Color::RGBA(55, 71, 99, 255);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_plugin_section_buffer_body(
        &mut target,
        PluginSectionDraw {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            layout,
            visual_selection: Some(VisualSelection::Range(TextRange::new(
                TextPoint::new(0, 0),
                TextPoint::new(0, 5),
            ))),
            yank_flash: None,
            input_mode: InputMode::Visual,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: selection_color,
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. } if *color == to_render_color(selection_color)
    )));
    Ok(())
}

#[test]
fn render_image_buffer_body_draws_centered_clipped_image() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.kind = BufferKind::Image;
    buffer.image_state = Some(ImageBufferState {
        format: ImageBufferFormat::Raster,
        mode: ImageBufferMode::Rendered,
        decoded: DecodedImage {
            width: 200,
            height: 100,
            pixels: Arc::<[u8]>::from(vec![255; 200 * 100 * 4]),
        },
        zoom: 1.5,
    });

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let viewport = image_buffer_viewport_rect(rect, layout)
        .ok_or_else(|| "image viewport missing".to_owned())?;
    let expected = centered_image_draw_rect(viewport, 200, 100, 1.5)
        .ok_or_else(|| "image draw rect missing".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_image_buffer_body(
        &mut target,
        buffer,
        rect,
        layout,
        None,
        Color::RGB(15, 16, 20),
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Image {
            rect,
            clip_rect,
            image_width,
            image_height,
            ..
        } if *rect == to_pixel_rect(expected)
            && *clip_rect == Some(to_pixel_rect(viewport))
            && *image_width == 200
            && *image_height == 100
    )));
    Ok(())
}

#[test]
fn sync_active_viewport_matches_acp_footer_visible_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(
        &mut state,
        40,
        "first line\nsecond line",
        Some("chat · gpt-5.4 · shift+tab switch mode"),
    )?;

    state
        .sync_active_viewport(400, 18)
        .map_err(|error| error.to_string())?;

    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let layout = buffer_footer_layout(buffer, PixelRectToRect::rect(0, 0, 800, 400), 18, 8);
    assert_eq!(buffer.viewport_lines(), layout.visible_rows);

    buffer.scroll_output_to_end();
    buffer.append_output_lines(&["tail".to_owned()]);

    assert!(
        buffer.line_at_viewport_offset(buffer.viewport_lines().saturating_sub(1)) + 1
            >= buffer.line_count()
    );
    Ok(())
}

#[test]
fn acp_switch_pane_command_changes_internal_pane_without_changing_workspace_pane()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, "*acp*", user::acp::ACP_BUFFER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.init_acp_view("GitHub Copilot");
    }
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    let active_pane_id = shell_ui(&state.runtime)?
        .active_pane_id()
        .ok_or_else(|| "active pane is missing".to_owned())?;

    state
        .runtime
        .execute_command("acp.switch-pane")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(active_pane_id)
    );
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.acp_active_pane(), Some(AcpPane::Input));
    Ok(())
}

#[test]
fn acp_plan_entries_populate_static_plan_pane() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_set_plan(Plan::new(vec![
        PlanEntry::new(
            "Render the ACP plan pane",
            PlanEntryPriority::High,
            PlanEntryStatus::Pending,
        ),
        PlanEntry::new(
            "Stream tool output into cards",
            PlanEntryPriority::Medium,
            PlanEntryStatus::InProgress,
        ),
    ]));

    let acp = buffer.acp_state.as_ref().expect("ACP state missing");
    assert_eq!(acp.plan_entries.len(), 2);
    match &acp.plan_pane.render_lines[0] {
        AcpRenderedLine::Text(line) => {
            assert_eq!(line.text, "Render the ACP plan pane");
            assert_eq!(line.prefix[0].role, AcpColorRole::PriorityHigh);
        }
        other => panic!("expected text line, got {other:?}"),
    }
    match &acp.plan_pane.render_lines[1] {
        AcpRenderedLine::Text(line) => {
            assert_eq!(line.text, "Stream tool output into cards");
            assert!(line.prefix[0].animate);
        }
        other => panic!("expected text line, got {other:?}"),
    }
    Ok(())
}

#[test]
fn acp_plan_entries_normalize_completed_prefix_when_later_step_is_active() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_set_plan(Plan::new(vec![
        PlanEntry::new(
            "First step",
            PlanEntryPriority::High,
            PlanEntryStatus::Pending,
        ),
        PlanEntry::new(
            "Second step",
            PlanEntryPriority::High,
            PlanEntryStatus::InProgress,
        ),
        PlanEntry::new(
            "Third step",
            PlanEntryPriority::Medium,
            PlanEntryStatus::Pending,
        ),
    ]));

    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    assert_eq!(acp.plan_entries[0].status, PlanEntryStatus::Completed);
    assert_eq!(acp.plan_entries[1].status, PlanEntryStatus::InProgress);
    assert_eq!(acp.plan_entries[2].status, PlanEntryStatus::Pending);
    Ok(())
}

#[test]
fn acp_plan_entries_normalize_completed_prefix_without_active_step() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_set_plan(Plan::new(vec![
        PlanEntry::new(
            "First step",
            PlanEntryPriority::High,
            PlanEntryStatus::Pending,
        ),
        PlanEntry::new(
            "Second step",
            PlanEntryPriority::High,
            PlanEntryStatus::Completed,
        ),
        PlanEntry::new(
            "Third step",
            PlanEntryPriority::Medium,
            PlanEntryStatus::Pending,
        ),
    ]));

    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    assert_eq!(acp.plan_entries[0].status, PlanEntryStatus::Completed);
    assert_eq!(acp.plan_entries[1].status, PlanEntryStatus::Completed);
    assert_eq!(acp.plan_entries[2].status, PlanEntryStatus::Pending);
    Ok(())
}

#[test]
fn acp_tool_call_updates_replace_existing_output_item() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_upsert_tool_call(
        ToolCall::new("tool-1", "Read file")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::Pending),
    );
    buffer.acp_update_tool_call(ToolCallUpdate::new(
        "tool-1",
        ToolCallUpdateFields::new()
            .title("Read src\\main.rs")
            .status(ToolCallStatus::Completed)
            .content(vec![ToolCallContent::from("Loaded 42 lines")]),
    ));

    let acp = buffer.acp_state.as_ref().expect("ACP state missing");
    let tool_calls = acp
        .output_items
        .iter()
        .filter_map(|item| match item {
            AcpOutputItem::ToolCall(tool_call) => Some(tool_call),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].title, "Read src\\main.rs");
    assert_eq!(tool_calls[0].status, ToolCallStatus::Completed);
    assert_eq!(tool_calls[0].content.len(), 1);
    assert_eq!(acp.tool_item_indices.len(), 1);
    Ok(())
}

#[test]
fn acp_plan_height_caps_wrapped_content_at_ten_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_set_plan(Plan::new(
        (0..4)
            .map(|index| {
                PlanEntry::new(
                    format!(
                        "ACP plan item {index} should wrap several visual rows in a narrow pane so the plan height clamp is exercised"
                    ),
                    PlanEntryPriority::Medium,
                    PlanEntryStatus::Pending,
                )
            })
            .collect(),
    ));

    buffer.sync_acp_viewport_metrics(220, 420, 8, 16, true);

    let acp = buffer.acp_state.as_ref().expect("ACP state missing");
    assert_eq!(acp.plan_pane.visible_rows(), 10);
    assert!(acp.output_pane.visible_rows() >= 1);
    Ok(())
}

#[test]
fn acp_scroll_output_to_end_reaches_last_rendered_line() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_set_plan(Plan::new(vec![PlanEntry::new(
        "Keep the plan compact",
        PlanEntryPriority::Medium,
        PlanEntryStatus::InProgress,
    )]));
    for index in 0..48 {
        buffer.acp_push_system_message(format!("output line {index}"));
    }

    buffer.sync_acp_viewport_metrics(800, 400, 8, 16, true);
    buffer.scroll_output_to_end();

    assert!(
        buffer.line_at_viewport_offset(buffer.viewport_lines().saturating_sub(1)) + 1
            >= buffer.line_count()
    );
    Ok(())
}

#[test]
fn acp_output_scroll_reaches_wrapped_tail() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_push_system_message("word ".repeat(40));

    buffer.sync_acp_viewport_metrics(220, 420, 8, 16, true);
    {
        let acp = buffer
            .acp_state
            .as_mut()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        acp.active_pane = AcpPane::Output;
        acp.output_pane.scroll_visual_row = acp.output_pane.max_scroll_row();
    }

    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    assert_eq!(
        acp.output_pane.scroll_visual_row,
        acp.output_pane.max_scroll_row()
    );
    assert!(
        acp.output_pane.scroll_visual_row > 0,
        "wrapped output should require scrolling past the first visual row"
    );
    Ok(())
}

#[test]
fn acp_viewport_scroll_does_not_treat_visual_row_as_line_index() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    // Skip the Connected banner so the wrapped message is line 0.
    buffer.acp_prepare_session_replay("GitHub Copilot");
    buffer.acp_push_system_message("word ".repeat(40));
    for index in 0..20 {
        buffer.acp_push_system_message(format!("tail line {index}"));
    }

    // Narrow pane so line 0 wraps across several visual rows.
    buffer.sync_acp_viewport_metrics(220, 420, 8, 16, true);
    {
        let acp = buffer
            .acp_state
            .as_mut()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        acp.active_pane = AcpPane::Output;
        let wrap_cols = acp.output_pane.wrap_cols();
        let first_rows = acp_rendered_line_row_count(
            acp.output_pane
                .render_lines
                .first()
                .ok_or_else(|| "output render lines missing".to_owned())?,
            wrap_cols,
        );
        assert!(
            first_rows > 1,
            "line 0 must wrap; got {first_rows} visual rows at wrap_cols={wrap_cols}"
        );
        acp.output_pane.set_cursor(TextPoint::new(0, 0));
        acp.output_pane.scroll_visual_row = 0;
    }

    scroll_buffer_viewport_only(buffer, 1);

    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    assert_eq!(acp.output_pane.scroll_visual_row, 1);
    assert_eq!(
        buffer.cursor_point().line,
        0,
        "scrolling one visual row inside wrapped line 0 must not jump cursor to line index == scroll_visual_row"
    );
    Ok(())
}

#[test]
fn acp_visual_selection_uses_output_text_without_prefix() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_push_system_message("alpha beta");
    let line_index = buffer.line_count().saturating_sub(1);
    buffer.set_cursor(TextPoint::new(line_index, 4));

    let selection = visual_selection(
        buffer,
        TextPoint::new(line_index, 0),
        VisualSelectionKind::Character,
    )
    .ok_or_else(|| "visual selection should not be empty".to_owned())?;
    let VisualSelection::Range(range) = selection else {
        return Err("expected a range selection".to_owned());
    };

    assert_eq!(buffer.slice(range), "alpha");
    Ok(())
}

#[test]
fn render_acp_output_draws_visual_selection_highlight() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_push_system_message("alpha beta");
    buffer.sync_acp_viewport_metrics(640, 360, 8, 16, true);

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let selection_color = Color::RGBA(55, 71, 99, 255);
    let line_index = buffer.line_count().saturating_sub(1);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        AcpBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            visual_selection: Some(VisualSelection::Range(TextRange::new(
                TextPoint::new(line_index, 0),
                TextPoint::new(line_index, 5),
            ))),
            yank_flash: None,
            input_mode: InputMode::Visual,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: selection_color,
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. } if *color == to_render_color(selection_color)
    )));
    Ok(())
}

#[test]
fn render_acp_headers_use_rounded_caps() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let acp_layout = acp_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "missing ACP layout".to_owned())?;
    let header_height = (16 + 10) as u32;
    let inner_radius = shared_corner_radius(None).saturating_sub(1);
    let header_radius = inner_radius.min(header_height / 2);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        AcpBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            visual_selection: None,
            yank_flash: None,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    for pane in [acp_layout.plan, acp_layout.output] {
        assert!(scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRoundedRect { rect, radius, .. }
                if rect.x == pane.rect.x() + 1
                    && rect.y == pane.rect.y() + 1
                    && rect.width == pane.rect.width().saturating_sub(2)
                    && rect.height == header_height
                    && *radius == header_radius
        )));
    }
    Ok(())
}

#[test]
fn render_acp_output_header_shows_live_when_tool_in_progress() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_upsert_tool_call(
        ToolCall::new("tool-1", "Read file")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::InProgress),
    );

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        AcpBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            visual_selection: None,
            yank_flash: None,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { text, .. } if text.contains("Output · live")
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { text, .. } if text.contains("image continues")
    )));
    Ok(())
}

#[test]
fn render_acp_input_cursor_uses_rounded_rect_in_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "volt", None)?;
    let cursor_color = Color::RGB(17, 97, 197);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
        let _ = buffer.focus_acp_input();
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.cursor = 2;
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let acp_layout = acp_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "missing ACP layout".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        AcpBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            visual_selection: None,
            yank_flash: None,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: cursor_color,
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    let cursor_color = to_render_color(cursor_color);
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x >= acp_layout.input.rect.x()
                && rect.x < acp_layout.input.rect.x() + acp_layout.input.rect.width() as i32
                && rect.y >= acp_layout.input.rect.y()
                && rect.y < acp_layout.input.rect.y() + acp_layout.input.rect.height() as i32
                && rect.width == 8
                && rect.height == 16
                && *color == cursor_color
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x >= acp_layout.input.rect.x()
                && rect.x < acp_layout.input.rect.x() + acp_layout.input.rect.width() as i32
                && rect.y >= acp_layout.input.rect.y()
                && rect.y < acp_layout.input.rect.y() + acp_layout.input.rect.height() as i32
                && rect.width == 8
                && rect.height == 16
                && *color == cursor_color
    )));
    Ok(())
}

#[test]
fn render_acp_buffer_with_tall_multiline_input_keeps_footer_on_screen() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let pasted = "        AircraftEngineeringServicingEquipment, // ASE\n\
        AircraftTowBar, // ACTB\n\
        AircraftTug, // TUGS - 30\n\
        BaggageDollie, // BAGD\n\
        BaggagePOD, // POD\n\
        BaggageTug, // EBT\n\
        BeltLoader, // BELT\n\
        Van, // CAR\n\
        CateringVehicle, // CATV\n\
        Coach, // COAC\n\
        DeIcingVehicle, // DEIC\n\
        GroundPowerUnit, // GPU\n\
        HighLoader, // HILO - 40\n\
        LowLoader, // LOLO\n\
        Minibus, // MBUS\n\
        MotorisedStep, // MSTP\n\
        NonMotorisedStep, // STPN\n\
        PassengerBoardingRamp, // PBR\n\
        PassengerMobility, // LIFT - Ambulift\n"
        .repeat(6);
    let buffer_id = install_acp_test_buffer(&mut state, 0, &format!("/{pasted}"), None)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
        let _ = buffer.focus_acp_input();
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    assert!(layout.input_y >= layout.body_y);
    let acp_layout = acp_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "missing ACP layout".to_owned())?;
    let footer_bottom =
        acp_layout.footer.rect.y() + i32::try_from(acp_layout.footer.rect.height()).unwrap_or(0);
    assert!(footer_bottom <= layout.pane_bottom);
    assert!(acp_layout.input.rect.height() as i32 <= input_panel_chrome_height() + 16 * 10);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        AcpBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            visual_selection: None,
            yank_flash: None,
            input_mode: InputMode::Insert,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(acp_layout.input.rect.height() > 0);
    assert!(!scene.is_empty());
    Ok(())
}

#[test]
fn sync_active_viewport_uses_active_pane_height_for_horizontal_splits() -> Result<(), String> {
    let render_width = 640;
    let render_height = 320;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_text_test_buffer(
        &mut state,
        "*split-viewport*",
        (0..120).map(|index| format!("line {index}")).collect(),
    )?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Horizontal)?;

    state
        .sync_active_viewport_for_render_size(render_width, render_height, line_height)
        .map_err(|error| error.to_string())?;

    let command_line_visible = state.user_library.commandline_enabled();
    let pane_rect = horizontal_pane_rects(render_width, render_height, 2)
        .into_iter()
        .next()
        .ok_or_else(|| "horizontal split did not produce a pane rect".to_owned())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let pane_layout = buffer_footer_layout_with_command_line(
        buffer,
        PixelRectToRect::rect(pane_rect.x, pane_rect.y, pane_rect.width, pane_rect.height),
        line_height,
        8,
        command_line_visible,
    );
    let full_layout = buffer_footer_layout_with_command_line(
        buffer,
        PixelRectToRect::rect(0, 0, render_width, render_height),
        line_height,
        8,
        command_line_visible,
    );

    assert_eq!(buffer.viewport_lines(), pane_layout.visible_rows);
    assert!(pane_layout.visible_rows < full_layout.visible_rows);
    Ok(())
}

#[test]
fn sync_visible_buffer_layouts_use_split_width_for_vertical_splits() -> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let line = format!(
        "const wrapped_line = {};",
        "abcdefghijklmnopqrstuvwxyz".repeat(8)
    );
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*split-wrap*",
        (0..120).map(|_| line.clone()).collect(),
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(80, 80));

    state
        .sync_active_viewport_for_render_size(render_width, render_height, line_height)
        .map_err(|error| error.to_string())?;
    {
        let visible_rows = shell_buffer(&state.runtime, buffer_id)?.viewport_lines();
        let indent_size = theme_lang_indent(
            state.runtime.services().get::<ThemeRegistry>(),
            shell_buffer(&state.runtime, buffer_id)?.language_id(),
        );
        shell_buffer_mut(&mut state.runtime, buffer_id)?.ensure_visible(
            visible_rows,
            wrap_columns_for_width(render_width, cell_width),
            indent_size,
            0,
            0,
        );
    }
    shell_ui_mut(&mut state.runtime)?
        .workspace_view_mut()
        .ok_or_else(|| "workspace view is missing".to_owned())?
        .split_buffer_id = buffer_id;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    install_acp_test_buffer(
        &mut state,
        40,
        "",
        Some("chat · gpt-5.4 · shift+tab switch mode"),
    )?;

    let pane_rect = vertical_pane_rects(render_width, render_height, 2)
        .into_iter()
        .nth(1)
        .ok_or_else(|| "vertical split did not produce a right pane rect".to_owned())?;
    let before_sync = buffer_cursor_screen_anchor(
        shell_buffer(&state.runtime, buffer_id)?,
        PixelRectToRect::rect(pane_rect.x, pane_rect.y, pane_rect.width, pane_rect.height),
        &*shell_user_library(&state.runtime),
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
        false,
    );

    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let after_sync = buffer_cursor_screen_anchor(
        buffer,
        PixelRectToRect::rect(pane_rect.x, pane_rect.y, pane_rect.width, pane_rect.height),
        &*shell_user_library(&state.runtime),
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
        false,
    );
    assert!(before_sync.is_none());
    assert!(after_sync.is_some());
    Ok(())
}

#[test]
fn material_icons_rasterize_from_nfm_with_fontdue() -> Result<(), String> {
    let font = load_nfm_raster_font()?;
    let material_icon = editor_icons::symbols::md::MD_FORMAT_BOLD
        .chars()
        .next()
        .ok_or_else(|| "material icon glyph missing".to_owned())?;
    let (metrics, bitmap) = font.rasterize(material_icon, 48.0);
    let occupied_rows = bitmap
        .chunks(metrics.width)
        .map(|row| row.iter().filter(|alpha| **alpha > 32).count())
        .filter(|count| *count > 0)
        .collect::<Vec<_>>();
    let unique_row_widths = occupied_rows.iter().copied().collect::<BTreeSet<_>>();

    assert!(material_icon as u32 > 0xFFFF);
    assert!(metrics.width > 0);
    assert!(metrics.height > 0);
    assert!(!occupied_rows.is_empty());
    assert!(unique_row_widths.len() > 4);
    Ok(())
}

fn load_nfm_raster_font() -> Result<RasterFont, String> {
    let font_path = resolve_bundled_icon_font_dir()
        .map_err(|error| error.to_string())?
        .join("NFM.ttf");
    let bytes = fs::read(&font_path).map_err(|error| error.to_string())?;
    RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())
}

#[test]
fn icon_glyph_draw_offset_y_uses_icon_line_metrics_when_available() -> Result<(), String> {
    let font = load_nfm_raster_font()?;
    let codicon = editor_icons::symbols::cod::COD_DIFF_ADDED
        .chars()
        .next()
        .ok_or_else(|| "codicon glyph missing".to_owned())?;
    let requested_pixel_size = 18.0;
    let (raw_metrics, _) = font.rasterize(codicon, requested_pixel_size);
    let rasterized = rasterize_icon_glyph_for_cell(
        &font,
        codicon,
        requested_pixel_size,
        raw_metrics.width.max(1) as i32,
    );
    let line_metrics = font
        .horizontal_line_metrics(rasterized.pixel_size)
        .ok_or_else(|| "icon line metrics missing".to_owned())?;
    let primary_line_height = (line_metrics.ascent - line_metrics.descent).round() as i32;
    let synthetic_primary_ascent = line_metrics.ascent.round() as i32 - 2;
    let expected = (((primary_line_height as f32 - (line_metrics.ascent - line_metrics.descent))
        * 0.5)
        + line_metrics.ascent
        - rasterized.metrics.height as f32
        - rasterized.metrics.ymin as f32)
        .round() as i32;

    let draw_offset = icon_glyph_draw_offset_y(
        &rasterized.metrics,
        primary_line_height,
        synthetic_primary_ascent,
        font.horizontal_line_metrics(rasterized.pixel_size),
    );
    let fallback_offset = icon_glyph_draw_offset_y(
        &rasterized.metrics,
        primary_line_height,
        synthetic_primary_ascent,
        None,
    );

    assert!((rasterized.pixel_size - requested_pixel_size).abs() < f32::EPSILON);
    assert_eq!(draw_offset, expected);
    assert_eq!(
        fallback_offset,
        synthetic_primary_ascent - rasterized.metrics.height as i32 - rasterized.metrics.ymin
    );
    assert_ne!(draw_offset, fallback_offset);
    Ok(())
}

#[test]
fn icon_glyph_draw_offset_y_centers_width_fitted_icons_in_primary_line_height() -> Result<(), String>
{
    let font = load_nfm_raster_font()?;
    let codicon = editor_icons::symbols::cod::COD_DIFF_ADDED
        .chars()
        .next()
        .ok_or_else(|| "codicon glyph missing".to_owned())?;
    let requested_pixel_size = 18.0;
    let requested_line_metrics = font
        .horizontal_line_metrics(requested_pixel_size)
        .ok_or_else(|| "requested icon line metrics missing".to_owned())?;
    let primary_line_height =
        (requested_line_metrics.ascent - requested_line_metrics.descent).round() as i32;
    let primary_ascent = requested_line_metrics.ascent.round() as i32;
    let (raw_metrics, _) = font.rasterize(codicon, requested_pixel_size);
    let cell_width = (raw_metrics.width / 2).max(1) as i32;
    let rasterized =
        rasterize_icon_glyph_for_cell(&font, codicon, requested_pixel_size, cell_width);
    let fitted_line_metrics = font
        .horizontal_line_metrics(rasterized.pixel_size)
        .ok_or_else(|| "fitted icon line metrics missing".to_owned())?;
    let expected = (((primary_line_height as f32
        - (fitted_line_metrics.ascent - fitted_line_metrics.descent))
        * 0.5)
        + fitted_line_metrics.ascent
        - rasterized.metrics.height as f32
        - rasterized.metrics.ymin as f32)
        .round() as i32;

    let draw_offset = icon_glyph_draw_offset_y(
        &rasterized.metrics,
        primary_line_height,
        primary_ascent,
        font.horizontal_line_metrics(rasterized.pixel_size),
    );
    let fallback_offset = icon_glyph_draw_offset_y(
        &rasterized.metrics,
        primary_line_height,
        primary_ascent,
        None,
    );
    let draw_bottom_margin = primary_line_height - (draw_offset + rasterized.metrics.height as i32);
    let fallback_bottom_margin =
        primary_line_height - (fallback_offset + rasterized.metrics.height as i32);

    assert!(raw_metrics.width > cell_width as usize);
    assert!(rasterized.pixel_size < requested_pixel_size);
    assert_eq!(draw_offset, expected);
    assert!(draw_offset >= 0);
    assert!(draw_bottom_margin >= 0);
    assert!(
        (draw_offset - draw_bottom_margin).abs() < (fallback_offset - fallback_bottom_margin).abs()
    );
    Ok(())
}

#[test]
fn codicon_glyphs_fit_inside_one_editor_cell() -> Result<(), String> {
    let font = load_nfm_raster_font()?;
    let codicon = editor_icons::symbols::cod::COD_DIFF_ADDED
        .chars()
        .next()
        .ok_or_else(|| "codicon glyph missing".to_owned())?;
    let requested_pixel_size = 18.0;
    let (raw_metrics, _) = font.rasterize(codicon, requested_pixel_size);
    let cell_width = raw_metrics.width.saturating_sub(1).max(1) as i32;
    let rasterized =
        rasterize_icon_glyph_for_cell(&font, codicon, requested_pixel_size, cell_width);
    let layout = icon_glyph_cell_layout(&rasterized.metrics, cell_width);

    assert!(raw_metrics.width > cell_width as usize);
    assert!(rasterized.metrics.width as i32 <= cell_width);
    assert_eq!(layout.advance, cell_width);
    assert!(layout.draw_offset_x >= 0);
    assert!(layout.draw_offset_x + rasterized.metrics.width as i32 <= cell_width);
    Ok(())
}

#[test]
fn font_role_prefers_icon_font_for_private_use_glyphs_without_symbol_hint() -> Result<(), String> {
    let branch = editor_icons::symbols::ple::PL_BRANCH
        .chars()
        .next()
        .ok_or_else(|| "powerline branch glyph missing".to_owned())?;

    assert!(is_private_use_character(branch));
    assert_eq!(
        resolve_font_role_for_char(Some(0), true, false, false, branch),
        FontRole::Icon(0)
    );
    Ok(())
}

#[test]
fn font_role_prefers_icon_font_for_symbol_like_prompt_glyphs() -> Result<(), String> {
    let prompt = '\u{276F}';

    assert!(is_symbol_like_character(prompt));
    assert!(!is_private_use_character(prompt));
    assert_eq!(
        resolve_font_role_for_char(Some(0), true, false, false, prompt),
        FontRole::Icon(0)
    );
    Ok(())
}

#[test]
fn font_role_uses_emoji_when_emoji_font_has_glyph() {
    assert_eq!(
        resolve_font_role_for_char(None, false, false, true, '\u{1F642}'),
        FontRole::Emoji
    );
}

#[test]
fn zero_width_display_characters_include_joiners() {
    assert!(is_zero_width_display_character('\u{200D}'));
}

#[test]
fn strip_zero_width_display_characters_removes_variation_selectors() {
    assert_eq!(
        strip_zero_width_display_characters("- ⚛️ Built with Expo Router").as_ref(),
        "- ⚛ Built with Expo Router"
    );
}

#[test]
fn strip_zero_width_display_characters_removes_byte_order_marks() {
    assert_eq!(
        strip_zero_width_display_characters("\u{feff}<Project Sdk=\"Microsoft.NET.Sdk\">").as_ref(),
        "<Project Sdk=\"Microsoft.NET.Sdk\">"
    );
}

#[test]
fn emoji_raster_font_rasterizes_simple_emoji() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: Some("Segoe UI Emoji".to_owned()),
            font_size: 18,
            emoji_font_size: 18,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let raster_font = fonts
        .emoji_raster_font()
        .ok_or_else(|| "emoji raster font missing".to_owned())?;
    let (metrics, bitmap) =
        raster_font.rasterize('\u{1F642}', fonts.emoji_pixel_size().unwrap_or(18.0));
    assert!(metrics.width > 0, "emoji raster width should be non-zero");
    assert!(metrics.height > 0, "emoji raster height should be non-zero");
    assert!(
        bitmap.iter().any(|alpha| *alpha != 0),
        "emoji bitmap should contain visible coverage"
    );
    Ok(())
}

#[test]
fn compose_emoji_surface_rasterizes_simple_emoji() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: Some("Segoe UI Emoji".to_owned()),
            font_size: 18,
            emoji_font_size: 18,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let layout = cached_emoji_layout(&fonts, "\u{1F642}", fonts.primary().ascent())
        .ok_or_else(|| "emoji layout missing".to_owned())?;
    let surface = compose_emoji_surface(&fonts, &layout, RenderColor::rgb(255, 255, 255))
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "emoji surface missing".to_owned())?;
    assert!(
        surface.width() > 0,
        "emoji surface width should be non-zero"
    );
    assert!(
        surface.height() > 0,
        "emoji surface height should be non-zero"
    );

    let mut has_visible_alpha = false;
    surface.with_lock(|pixels| {
        has_visible_alpha = pixels.as_chunks::<4>().0.iter().any(|rgba| rgba[3] != 0);
    });
    assert!(
        has_visible_alpha,
        "emoji surface should contain visible pixels"
    );
    Ok(())
}

#[test]
fn autocomplete_or_group_uses_first_provider_with_results() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("alpha alphabet\nalp");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(1, 3));

    let (buffer_id, buffer_revision, text, cursor, query) = {
        let ui = state.ui().map_err(|error| error.to_string())?;
        let buffer_id = ui
            .active_buffer_id()
            .ok_or_else(|| "active buffer missing".to_owned())?;
        let buffer = ui
            .buffer(buffer_id)
            .ok_or_else(|| "shell buffer missing".to_owned())?;
        let text = buffer.text.snapshot();
        let query = autocomplete_query(&text, true)
            .ok_or_else(|| "autocomplete query missing".to_owned())?;
        (
            buffer_id,
            buffer.text.revision(),
            text,
            buffer.cursor_point(),
            query,
        )
    };
    let request = AutocompleteWorkerRequest {
        request_id: 1,
        buffer_id,
        buffer_revision,
        text,
        plugin_kind: None,
        db_candidates: Vec::new(),
        path: None,
        root: None,
        cursor,
        query,
        providers: vec![
            AutocompleteProviderSpec {
                id: "primary".to_owned(),
                label: "Primary".to_owned(),
                icon: "P".to_owned(),
                item_icon: "1".to_owned(),
                or_group: Some("source".to_owned()),
                buffer_kind: None,
                items: Vec::new(),
                kind: AutocompleteProviderKind::Buffer,
            },
            AutocompleteProviderSpec {
                id: "fallback".to_owned(),
                label: "Fallback".to_owned(),
                icon: "F".to_owned(),
                item_icon: "2".to_owned(),
                or_group: Some("source".to_owned()),
                buffer_kind: None,
                items: Vec::new(),
                kind: AutocompleteProviderKind::Buffer,
            },
        ],
        lsp_client: None,
        edits: None,
        token_edits_from: None,
        token_edits: None,
    };

    let entries = autocomplete_entries(&request, &mut AutocompleteTokenCache::default());
    assert!(!entries.is_empty());
    assert!(entries.iter().all(|entry| entry.provider_id == "primary"));
    Ok(())
}

#[test]
fn autocomplete_entries_are_not_limited_by_visible_result_limit() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("alpha alpine alphabet alchemy altar alto\nal");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(1, 2));

    let (buffer_id, buffer_revision, text, cursor, query) = {
        let ui = state.ui().map_err(|error| error.to_string())?;
        let buffer_id = ui
            .active_buffer_id()
            .ok_or_else(|| "active buffer missing".to_owned())?;
        let buffer = ui
            .buffer(buffer_id)
            .ok_or_else(|| "shell buffer missing".to_owned())?;
        let text = buffer.text.snapshot();
        let query = autocomplete_query(&text, true)
            .ok_or_else(|| "autocomplete query missing".to_owned())?;
        (
            buffer_id,
            buffer.text.revision(),
            text,
            buffer.cursor_point(),
            query,
        )
    };
    let request = AutocompleteWorkerRequest {
        request_id: 1,
        buffer_id,
        buffer_revision,
        text,
        plugin_kind: None,
        db_candidates: Vec::new(),
        path: None,
        root: None,
        cursor,
        query,
        providers: vec![AutocompleteProviderSpec {
            id: "buffer".to_owned(),
            label: "Buffer".to_owned(),
            icon: "B".to_owned(),
            item_icon: "T".to_owned(),
            or_group: None,
            buffer_kind: None,
            items: Vec::new(),
            kind: AutocompleteProviderKind::Buffer,
        }],
        lsp_client: None,
        edits: None,
        token_edits_from: None,
        token_edits: None,
    };

    let entries = autocomplete_entries(&request, &mut AutocompleteTokenCache::default());
    assert_eq!(entries.len(), 6);
    Ok(())
}

fn buffer_autocomplete_request(
    buffer_id: BufferId,
    buffer: &TextBuffer,
    query: AutocompleteQuery,
    token_edits_from: Option<u64>,
    token_edits: Option<Vec<TextEdit>>,
) -> AutocompleteWorkerRequest {
    AutocompleteWorkerRequest {
        request_id: 1,
        buffer_id,
        buffer_revision: buffer.revision(),
        text: buffer.snapshot(),
        plugin_kind: None,
        db_candidates: Vec::new(),
        path: None,
        root: None,
        cursor: buffer.cursor(),
        query,
        providers: vec![AutocompleteProviderSpec {
            id: "buffer".to_owned(),
            label: "Buffer".to_owned(),
            icon: "B".to_owned(),
            item_icon: "T".to_owned(),
            or_group: None,
            buffer_kind: None,
            items: Vec::new(),
            kind: AutocompleteProviderKind::Buffer,
        }],
        lsp_client: None,
        edits: None,
        token_edits_from,
        token_edits,
    }
}

#[test]
fn autocomplete_worker_reuses_token_map_for_same_revision() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("alpha alpine alphabet\nal");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(1, 2));

    let (buffer_id, request) = {
        let ui = state.ui().map_err(|error| error.to_string())?;
        let buffer_id = ui
            .active_buffer_id()
            .ok_or_else(|| "active buffer missing".to_owned())?;
        let buffer = ui
            .buffer(buffer_id)
            .ok_or_else(|| "shell buffer missing".to_owned())?;
        let query = autocomplete_query(&buffer.text.snapshot(), true)
            .ok_or_else(|| "autocomplete query missing".to_owned())?;
        (
            buffer_id,
            buffer_autocomplete_request(buffer_id, &buffer.text, query, None, None),
        )
    };
    let _ = buffer_id;
    let mut cache = AutocompleteTokenCache::default();
    let first = autocomplete_entries(&request, &mut cache);
    assert_eq!(
        cache.last_scan().map(|scan| scan.kind),
        Some(AutocompleteTokenScanKind::Rebuilt)
    );
    let second = autocomplete_entries(&request, &mut cache);
    assert_eq!(
        cache.last_scan().map(|scan| scan.kind),
        Some(AutocompleteTokenScanKind::Reused)
    );
    assert_eq!(first, second);
    Ok(())
}

#[test]
fn autocomplete_insert_identifier_appears_and_delete_drops_last_occurrence() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("alpha alpine\nal");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(1, 2));

    let (buffer_id, mut cache) = {
        let ui = state.ui().map_err(|error| error.to_string())?;
        let buffer_id = ui
            .active_buffer_id()
            .ok_or_else(|| "active buffer missing".to_owned())?;
        (buffer_id, AutocompleteTokenCache::default())
    };

    {
        let buffer = state
            .ui()
            .map_err(|error| error.to_string())?
            .buffer(buffer_id)
            .ok_or_else(|| "shell buffer missing".to_owned())?;
        let query = autocomplete_query(&buffer.text.snapshot(), true)
            .ok_or_else(|| "autocomplete query missing".to_owned())?;
        let request = buffer_autocomplete_request(buffer_id, &buffer.text, query, None, None);
        let entries = autocomplete_entries(&request, &mut cache);
        assert!(
            entries
                .iter()
                .any(|entry| entry.replacement == "alpha" || entry.replacement == "alpine")
        );
        assert!(!entries.iter().any(|entry| entry.replacement == "almond"));
    }

    let from_revision = {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        let from_revision = buffer.text.revision();
        buffer.text.set_cursor(TextPoint::new(0, 12));
        buffer.text.insert_text(" almond");
        buffer.text.set_cursor(TextPoint::new(1, 2));
        from_revision
    };
    {
        let buffer = state
            .ui()
            .map_err(|error| error.to_string())?
            .buffer(buffer_id)
            .ok_or_else(|| "shell buffer missing".to_owned())?;
        let query = autocomplete_query(&buffer.text.snapshot(), true)
            .ok_or_else(|| "autocomplete query missing".to_owned())?;
        let edits = buffer.text.edits_since(from_revision);
        let request =
            buffer_autocomplete_request(buffer_id, &buffer.text, query, Some(from_revision), edits);
        let entries = autocomplete_entries(&request, &mut cache);
        assert_eq!(
            cache.last_scan().map(|scan| scan.kind),
            Some(AutocompleteTokenScanKind::Incremental)
        );
        assert!(entries.iter().any(|entry| entry.replacement == "almond"));
    }

    let from_revision = {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        let from_revision = buffer.text.revision();
        buffer.text.replace(
            TextRange::new(TextPoint::new(0, 13), TextPoint::new(0, 19)),
            "",
        );
        buffer.text.set_cursor(TextPoint::new(1, 2));
        from_revision
    };
    {
        let buffer = state
            .ui()
            .map_err(|error| error.to_string())?
            .buffer(buffer_id)
            .ok_or_else(|| "shell buffer missing".to_owned())?;
        let query = autocomplete_query(&buffer.text.snapshot(), true)
            .ok_or_else(|| "autocomplete query missing".to_owned())?;
        let edits = buffer.text.edits_since(from_revision);
        let request =
            buffer_autocomplete_request(buffer_id, &buffer.text, query, Some(from_revision), edits);
        let entries = autocomplete_entries(&request, &mut cache);
        assert_eq!(
            cache.last_scan().map(|scan| scan.kind),
            Some(AutocompleteTokenScanKind::Incremental)
        );
        assert!(!entries.iter().any(|entry| entry.replacement == "almond"));
    }
    Ok(())
}

#[test]
fn autocomplete_query_allows_empty_member_access_after_dot_and_arrow() {
    let mut dot = TextBuffer::from_text("object.");
    dot.set_cursor(TextPoint::new(0, 7));
    let dot_query = autocomplete_query(&dot.snapshot(), false)
        .expect("dot member access should allow empty autocomplete query");
    assert_eq!(dot_query.prefix, "");
    assert_eq!(dot_query.replace_range.start(), TextPoint::new(0, 7));
    assert_eq!(dot_query.replace_range.end(), TextPoint::new(0, 7));

    let mut arrow = TextBuffer::from_text("object->");
    arrow.set_cursor(TextPoint::new(0, 8));
    let arrow_query = autocomplete_query(&arrow.snapshot(), false)
        .expect("arrow member access should allow empty autocomplete query");
    assert_eq!(arrow_query.prefix, "");
    assert_eq!(arrow_query.replace_range.start(), TextPoint::new(0, 8));
    assert_eq!(arrow_query.replace_range.end(), TextPoint::new(0, 8));
}

#[test]
fn normalize_completion_replacement_strips_duplicate_member_access_trigger() {
    let mut buffer = TextBuffer::from_text("foo.");
    buffer.set_cursor(TextPoint::new(0, 4));
    let snapshot = buffer.snapshot();
    let empty_after_dot = TextRange::new(TextPoint::new(0, 4), TextPoint::new(0, 4));
    assert_eq!(
        normalize_completion_replacement(&snapshot, empty_after_dot, ".bar()"),
        "bar()"
    );

    // textEdit that already covers the typed '.' must keep the leading '.' in newText.
    let cover_dot = TextRange::new(TextPoint::new(0, 3), TextPoint::new(0, 4));
    assert_eq!(
        normalize_completion_replacement(&snapshot, cover_dot, ".bar()"),
        ".bar()"
    );

    let mut arrow = TextBuffer::from_text("ptr->");
    arrow.set_cursor(TextPoint::new(0, 5));
    let arrow_snapshot = arrow.snapshot();
    let empty_after_arrow = TextRange::new(TextPoint::new(0, 5), TextPoint::new(0, 5));
    assert_eq!(
        normalize_completion_replacement(&arrow_snapshot, empty_after_arrow, "->method"),
        "method"
    );
}

#[test]
fn accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = active_shell_buffer_mut(&mut state.runtime)?;
        buffer.text = TextBuffer::from_text("foo.");
        buffer.set_cursor(TextPoint::new(0, 4));
    }
    let buffer_id = active_shell_buffer_id(&state.runtime)?;

    let overlay = AutocompleteOverlay {
        buffer_id,
        buffer_revision: 0,
        query: AutocompleteQuery {
            prefix: String::new(),
            token: String::new(),
            replace_range: TextRange::new(TextPoint::new(0, 4), TextPoint::new(0, 4)),
        },
        entries: vec![AutocompleteEntry {
            provider_id: "lsp".to_owned(),
            provider_label: "LSP".to_owned(),
            provider_icon: "L".to_owned(),
            item_icon: "ƒ".to_owned(),
            label: "bar".to_owned(),
            replacement: ".bar()".to_owned(),
            replace_range: None,
            detail: None,
            documentation: None,
        }],
        selected_index: 0,
        loading: false,
    };
    shell_ui_mut(&mut state.runtime)?.set_autocomplete(overlay);
    accept_autocomplete(&mut state.runtime)?;
    assert_eq!(
        active_shell_buffer_mut(&mut state.runtime)?.text.text(),
        "foo.bar()"
    );
    Ok(())
}

#[test]
fn accept_autocomplete_uses_lsp_text_edit_range_covering_trigger() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = active_shell_buffer_mut(&mut state.runtime)?;
        buffer.text = TextBuffer::from_text("foo.");
        buffer.set_cursor(TextPoint::new(0, 4));
    }
    let buffer_id = active_shell_buffer_id(&state.runtime)?;

    let overlay = AutocompleteOverlay {
        buffer_id,
        buffer_revision: 0,
        query: AutocompleteQuery {
            prefix: String::new(),
            token: String::new(),
            replace_range: TextRange::new(TextPoint::new(0, 4), TextPoint::new(0, 4)),
        },
        entries: vec![AutocompleteEntry {
            provider_id: "lsp".to_owned(),
            provider_label: "LSP".to_owned(),
            provider_icon: "L".to_owned(),
            item_icon: "ƒ".to_owned(),
            label: "bar".to_owned(),
            replacement: ".bar()".to_owned(),
            replace_range: Some(TextRange::new(TextPoint::new(0, 3), TextPoint::new(0, 4))),
            detail: None,
            documentation: None,
        }],
        selected_index: 0,
        loading: false,
    };
    shell_ui_mut(&mut state.runtime)?.set_autocomplete(overlay);
    accept_autocomplete(&mut state.runtime)?;
    assert_eq!(
        active_shell_buffer_mut(&mut state.runtime)?.text.text(),
        "foo.bar()"
    );
    Ok(())
}

#[test]
fn vim_search_entries_trim_whitespace_from_labels() {
    let buffer = TextBuffer::from_text("alpha\n   split here   \nbeta\n");
    let data = vim_search_entries(&buffer.snapshot(), VimSearchDirection::Forward, "split");

    assert_eq!(data.entries.len(), 1);
    assert_eq!(data.entries[0].item.label(), "split here");
}

#[test]
fn completion_token_at_cursor_supports_trailing_token_edge() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("alpha beta");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 5));

    let (range, token) = completion_token_at_cursor(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    )
    .ok_or_else(|| "completion token missing at cursor edge".to_owned())?;

    assert_eq!(token, "alpha");
    assert_eq!(range.start(), TextPoint::new(0, 0));
    assert_eq!(range.end(), TextPoint::new(0, 5));
    Ok(())
}

#[test]
fn hover_signature_request_point_prefers_callee_over_enclosing_macro() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let text = "let commands = vec![hook_command(\"alpha\", \"beta\", \"gamma\", \"delta\")];";
    let cursor_column = text
        .find("hook_command")
        .ok_or_else(|| "hook_command missing".to_owned())?
        + 4;
    let expected_column = text
        .find("(\"alpha\"")
        .ok_or_else(|| "hook_command call missing".to_owned())?
        + 1;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text(text);
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, cursor_column));

    let point = hover_signature_request_point(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    );

    assert_eq!(point, TextPoint::new(0, expected_column));
    Ok(())
}

#[test]
fn hover_signature_request_point_preserves_argument_cursor_context() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let text = "hook_command(name, description, hook_name, detail)";
    let cursor_column = text
        .find("description")
        .ok_or_else(|| "description missing".to_owned())?
        + 3;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text(text);
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, cursor_column));

    let point = hover_signature_request_point(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    );

    assert_eq!(point, TextPoint::new(0, cursor_column));
    Ok(())
}

#[test]
fn manual_autocomplete_entries_only_apply_to_matching_plugin_buffers() {
    let provider = AutocompleteProviderSpec {
        id: "calculator".to_owned(),
        label: "Calculator".to_owned(),
        icon: "C".to_owned(),
        item_icon: "ƒ".to_owned(),
        or_group: None,
        buffer_kind: Some("calculator".to_owned()),
        items: vec![editor_plugin_api::AutocompleteProviderItem {
            label: "sqrt(x)".to_owned(),
            replacement: "sqrt".to_owned(),
            detail: Some("Square root".to_owned()),
            documentation: Some("Returns the square root of x.".to_owned()),
        }],
        kind: AutocompleteProviderKind::Manual,
    };
    let query = AutocompleteQuery {
        prefix: "sq".to_owned(),
        token: "sq".to_owned(),
        replace_range: TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 2)),
    };

    let matching = manual_autocomplete_entries(&Some("calculator".to_owned()), &query, &provider);
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].0.replacement, "sqrt");

    let non_matching =
        manual_autocomplete_entries(&Some("git-status".to_owned()), &query, &provider);
    assert!(non_matching.is_empty());
}

#[test]
fn hover_manual_provider_lines_match_current_plugin_token() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.kind = BufferKind::Plugin("calculator".to_owned());
        buffer.text = TextBuffer::from_text("sqrt");
        buffer.set_cursor(TextPoint::new(0, 2));
    }
    let provider = HoverProviderSpec {
        label: "Calculator".to_owned(),
        icon: "C".to_owned(),
        buffer_kind: Some("calculator".to_owned()),
        topics: vec![editor_plugin_api::HoverProviderTopic {
            token: "sqrt".to_owned(),
            lines: vec!["sqrt(x)".to_owned(), "Square root".to_owned()],
        }],
        kind: HoverProviderKind::Manual,
    };

    let lines = hover_manual_provider_lines(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
        &provider,
    );
    assert_eq!(lines, vec!["sqrt(x)".to_owned(), "Square root".to_owned()]);
    Ok(())
}

#[test]
fn hover_test_provider_lines_include_theme_and_treesitter_tokens() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("alpha");
        buffer.set_cursor(TextPoint::new(0, 2));
        buffer.syntax_lines.insert(
            0,
            vec![LineSyntaxSpan {
                start: 0,
                end: 5,
                capture_name: Arc::from("function"),
                theme_token: Arc::from("syntax.function"),
            }],
        );
    }

    let lines = {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        let token_info = completion_token_at_cursor(buffer);
        hover_test_provider_lines(buffer, token_info.as_ref())
    };

    assert!(
        lines
            .iter()
            .any(|line| line == "Theme color: syntax.function")
    );
    assert!(
        lines
            .iter()
            .any(|line| line == "Tree-sitter token: @function")
    );
    Ok(())
}

#[test]
fn render_markdown_hover_content_highlights_registered_code_fences() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    register_rust_highlight_test_language(&mut state.runtime)?;

    let rendered = render_markdown_hover_content(
        &mut state.runtime,
        "Example:\n\n```rust\nfn example() {}\n```\n",
    );

    assert_eq!(
        rendered.lines,
        vec![
            "Example:".to_owned(),
            String::new(),
            "```rust".to_owned(),
            "fn example() {}".to_owned(),
            "```".to_owned(),
        ]
    );
    assert!(rendered.syntax_lines.get(&3).is_some_and(|spans| {
        spans
            .iter()
            .any(|span| span.theme_token.as_ref() == "syntax.keyword")
    }));
    Ok(())
}

#[test]
fn hover_diagnostic_provider_fragments_preserve_fenced_code_blocks() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("alpha");
        buffer.set_cursor(TextPoint::new(0, 2));
        buffer.set_lsp_diagnostics(vec![LspDiagnostic::new(
            "rust-analyzer",
            "Try this:\n```rust\nfn example() {}\n```",
            LspDiagnosticSeverity::Warning,
            TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
        )]);
    }

    let fragments = {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        hover_diagnostic_provider_fragments(buffer, &NullUserLibrary)
    };

    assert_eq!(
        fragments,
        vec![
            HoverProviderFragment::PlainLines(vec![format!(
                "{} rust-analyzer",
                NullUserLibrary.lsp_diagnostic_icon()
            )]),
            HoverProviderFragment::MarkdownText(
                "Try this:\n```rust\nfn example() {}\n```".to_owned()
            ),
        ]
    );
    Ok(())
}

#[test]
fn index_syntax_lines_preserves_capture_names() {
    let text = TextBuffer::from_text("alpha");
    let lines = index_syntax_lines(
        editor_syntax::SyntaxSnapshot {
            language_id: "rust".to_owned(),
            root_kind: "source_file".to_owned(),
            has_errors: false,
            highlight_spans: vec![editor_syntax::HighlightSpan {
                start_byte: 0,
                end_byte: 5,
                start_position: editor_syntax::SyntaxPoint::new(0, 0),
                end_position: editor_syntax::SyntaxPoint::new(0, 5),
                capture_name: Arc::from("function"),
                theme_token: Arc::from("syntax.function"),
            }],
        },
        &text,
    );

    let spans = lines.get(&0).expect("expected indexed syntax line");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].capture_name.as_ref(), "function");
    assert_eq!(spans[0].theme_token.as_ref(), "syntax.function");
}

#[test]
fn index_syntax_lines_converts_byte_columns_after_variation_selector() {
    let line = "- ⚛️ Built";
    let text = TextBuffer::from_text(line);
    let start_byte = line.find("Built").expect("Built should be present");
    let end_byte = start_byte + "Built".len();
    let lines = index_syntax_lines(
        editor_syntax::SyntaxSnapshot {
            language_id: "markdown".to_owned(),
            root_kind: "document".to_owned(),
            has_errors: false,
            highlight_spans: vec![editor_syntax::HighlightSpan {
                start_byte,
                end_byte,
                start_position: editor_syntax::SyntaxPoint::new(0, start_byte),
                end_position: editor_syntax::SyntaxPoint::new(0, end_byte),
                capture_name: Arc::from("text.literal"),
                theme_token: Arc::from("syntax.string"),
            }],
        },
        &text,
    );

    assert_eq!(
        syntax_span_segments(line, lines.get(&0).expect("expected line spans")),
        vec![("syntax.string".to_owned(), "Built".to_owned())]
    );
}

#[test]
fn line_color_segments_colors_opening_brace_from_rust_highlight_pipeline() {
    let line = "use crate::{";
    let text = editor_buffer::TextBuffer::from_text(line);
    let mut registry = editor_syntax::SyntaxRegistry::new();
    registry
        .register(
            editor_syntax::LanguageConfiguration::new(
                "rust-rainbow-render-test",
                ["__rainbow_render_test__"],
                rust_test_language,
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                [editor_syntax::CaptureThemeMapping::new(
                    "punctuation.bracket",
                    "syntax.punctuation.bracket",
                )],
            )
            .with_extra_highlight_query(
                r#"
[
  "(" ")" "[" "]" "{" "}"
] @punctuation.bracket
"#,
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));

    let mut snapshot = registry
        .highlight_buffer_for_extension("__rainbow_render_test__", &text)
        .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
    editor_syntax::apply_rainbow_delimiter_spans(&mut snapshot, line, true);
    let syntax_lines = index_syntax_lines(snapshot, &text);
    let spans = syntax_lines
        .get(&0)
        .unwrap_or_else(|| panic!("expected syntax spans for line 0: {syntax_lines:?}"));
    let brace_col = line
        .char_indices()
        .find_map(|(byte, character)| (character == '{').then_some(byte))
        .map(|byte| line[..byte].chars().count())
        .expect("opening brace column");
    let overlapping: Vec<_> = spans
        .iter()
        .filter(|span| brace_col >= span.start && brace_col < span.end)
        .collect();
    assert!(
        overlapping
            .iter()
            .any(|span| span.theme_token.starts_with("rainbow.paren.")),
        "expected rainbow span at opening brace column {brace_col}, spans={overlapping:?}, all={spans:?}"
    );
    assert!(
        overlapping
            .iter()
            .all(|span| span.theme_token.as_ref() == "rainbow.paren.depth.1"),
        "opening brace captures should all share depth 1, got {overlapping:?}"
    );

    let mut theme_registry = editor_theme::ThemeRegistry::new();
    theme_registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(
                    "syntax.punctuation.bracket",
                    editor_theme::Color::rgb(1, 2, 3),
                )
                .with_token("rainbow.paren.depth.1", editor_theme::Color::rgb(4, 5, 6)),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let char_map = LineCharMap::new(line);
    let byte_offsets = &char_map.bytes[..=char_map.len()];
    let colored = line_color_segments(
        line,
        Some(spans),
        Some(&theme_registry),
        Color::RGB(240, 240, 240),
        byte_offsets,
        0,
    );
    let brace_segment = colored
        .iter()
        .find(|(text, _, _)| text == "{")
        .unwrap_or_else(|| panic!("expected colored segment for '{{', got {colored:?}"));
    assert_eq!(
        brace_segment.1,
        Color::RGB(4, 5, 6),
        "opening brace should use rainbow token color, got {colored:?}"
    );
}

#[test]
fn line_color_segments_prefers_rainbow_paren_token_for_equal_width_spans() {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token("syntax.type", editor_theme::Color::rgb(1, 2, 3))
                .with_token("rainbow.paren.depth.2", editor_theme::Color::rgb(4, 5, 6)),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let spans = vec![
        LineSyntaxSpan {
            start: 0,
            end: 1,
            capture_name: Arc::from("type"),
            theme_token: Arc::from("syntax.type"),
        },
        LineSyntaxSpan {
            start: 0,
            end: 1,
            capture_name: Arc::from("rainbow.paren.open.2"),
            theme_token: Arc::from("rainbow.paren.depth.2"),
        },
    ];

    let segments = line_color_segments(
        "(",
        Some(&spans),
        Some(&registry),
        Color::RGB(0, 0, 0),
        &[0, 1],
        0,
    );

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].0, "(".to_owned());
    assert_eq!(segments[0].1, Color::RGB(4, 5, 6));
}

#[test]
fn browser_buffer_submit_tracks_requested_navigation() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?;
        input.set_text("example.com/docs");
    }

    submit_input_buffer(&mut state.runtime)?;

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let state = buffer
        .browser_state
        .as_ref()
        .ok_or_else(|| "browser state missing".to_owned())?;
    assert_eq!(state.current_url.as_deref(), None);
    assert_eq!(
        state.requested_url.as_deref(),
        Some("https://example.com/docs")
    );
    assert!(state.is_loading);
    assert_eq!(
        buffer.display_name(),
        "*browser* [loading] https://example.com/docs"
    );
    Ok(())
}

#[test]
fn browser_escape_from_insert_keeps_input_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_browser_input();
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?;
        input.set_text("https://example.com");
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .cursor_char(),
        "https://example.com".chars().count()
    );
    Ok(())
}

#[test]
fn acp_input_field_visual_yank_copies_selected_text() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "alpha beta", None)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.cursor = 0;
    }

    focus_input_normal_mode(&mut state, buffer_id)?;
    start_visual_mode_with_kind(&mut state.runtime, VisualSelectionKind::Character)?;
    apply_motion_command(&mut state.runtime, ShellMotion::Right)?;
    apply_visual_operator(&mut state.runtime, VimOperator::Yank)?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        ui.vim().yank,
        Some(YankRegister::Character("al".to_owned()))
    );
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .selection_anchor,
        None
    );
    Ok(())
}

#[test]
fn acp_input_field_dd_deletes_current_line() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta\ngamma")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char("alpha\n".chars().count());
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "alpha\ngamma");
    assert_eq!(ui.vim().yank, Some(YankRegister::Line("beta\n".to_owned())));
    Ok(())
}

#[test]
fn acp_input_field_dw_deletes_motion_range() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha beta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("w")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "beta");
    assert_eq!(
        ui.vim().yank,
        Some(YankRegister::Character("alpha ".to_owned()))
    );
    Ok(())
}

#[test]
fn acp_input_field_cw_enters_insert_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha beta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("w")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(input.text(), "beta");

    state
        .handle_text_input("zeta ")
        .map_err(|error| error.to_string())?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "zeta beta");
    Ok(())
}

#[test]
fn acp_input_field_visual_line_delete_removes_selected_lines() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta\ngamma")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "gamma");
    Ok(())
}

#[test]
fn acp_input_field_o_and_o_open_new_lines() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char("alpha\n".chars().count());
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    state
        .handle_text_input("middle")
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("O")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    state
        .handle_text_input("above")
        .map_err(|error| error.to_string())?;

    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "alpha\nbeta\nabove\nmiddle");
    Ok(())
}

#[test]
fn acp_input_field_yy_and_p_work_linewise() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("p")
        .map_err(|error| error.to_string())?;

    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "alpha\nalpha\nbeta");
    Ok(())
}

#[test]
fn acp_escape_from_insert_keeps_input_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "prompt", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .cursor_char(),
        "prompt".chars().count()
    );
    Ok(())
}

#[test]
fn acp_second_escape_returns_hjkl_and_visual_mode_to_output_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, "*acp*", user::acp::ACP_BUFFER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.init_acp_view("GitHub Copilot");
        let acp = buffer
            .acp_state
            .as_mut()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        acp.output_pane.replace_render_lines(
            vec![
                AcpRenderedLine::Text(AcpRenderedTextLine {
                    prefix: Vec::new(),
                    text: "alpha".to_owned(),
                    text_role: AcpColorRole::Default,
                    syntax_spans: Vec::new(),
                    row_fill: None,
                    gutter: false,
                    align: AcpChatAlign::Full,
                    bubble: false,
                    bubble_group: 0,
                }),
                AcpRenderedLine::Text(AcpRenderedTextLine {
                    prefix: Vec::new(),
                    text: "beta".to_owned(),
                    text_role: AcpColorRole::Default,
                    syntax_spans: Vec::new(),
                    row_fill: None,
                    gutter: false,
                    align: AcpChatAlign::Full,
                    bubble: false,
                    bubble_group: 0,
                }),
            ],
            false,
            4,
        );
        if let Some(input) = buffer.input_field_mut() {
            input.set_text("prompt");
            input.cursor = input.text().len();
        }
    }

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.acp_active_pane(),
        Some(AcpPane::Output)
    );

    assert!(
        state
            .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert!(shell_ui(&state.runtime)?.vim().target == VimTarget::Input);

    assert!(
        state
            .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert!(shell_ui(&state.runtime)?.vim().target == VimTarget::Buffer);

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    assert_eq!(acp.output_pane.cursor(), TextPoint::new(1, 0));
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .cursor_char(),
        "prompt".chars().count()
    );

    state
        .handle_text_input("v")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("h")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(TextPoint::new(1, 0)));
    assert_eq!(ui.vim().target, VimTarget::Buffer);
    Ok(())
}

#[test]
fn paste_text_into_active_input_buffer_updates_acp_input() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "alpha", None)?;

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        " beta"
    )?);

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "alpha beta"
    );
    Ok(())
}

#[test]
fn paste_text_into_active_input_buffer_closes_acp_picker_for_multiline_text() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/fix", None)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "ACP Slash Commands",
        Vec::new(),
    ));

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        "\nmore context"
    )?);

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "/fix\nmore context"
    );
    Ok(())
}

#[test]
fn acp_nonleading_double_slash_does_not_open_slash_picker() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "i have this code ", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .handle_text_input("//")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "i have this code //"
    );
    Ok(())
}

#[test]
fn acp_slash_picker_text_input_updates_acp_input() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries("ACP Slash Commands", Vec::new())
            .with_kind(PickerKind::AcpSlash { buffer_id }),
    );

    state
        .handle_text_input("fix")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "/fix"
    );
    Ok(())
}

#[test]
fn acp_slash_picker_backspace_can_delete_leading_slash() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries("ACP Slash Commands", Vec::new())
            .with_kind(PickerKind::AcpSlash { buffer_id }),
    );
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Backspace),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        ""
    );
    Ok(())
}

#[test]
fn acp_paste_code_with_inline_double_slash_comments_closes_slash_picker() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
        ui.set_picker(
            PickerOverlay::from_entries("ACP Slash Commands", Vec::new())
                .with_kind(PickerKind::AcpSlash { buffer_id }),
        );
    }

    let pasted = "        Unknown = 0,\n        Vehicle=1,\n        Other,\n        SmartPhone,\n        Person,\n        Trailer,\n        Train,\n        Aircraft,\n        Luggage,\n        Skip,\n        IoTDevice=10,\n        Building,\n        Robot,\n        Parcel,\n        Animal,\n        CommercialWasteBin,\n        Keg,\n        Crane,\n        Generator,\n        RetailCage,\n        GolfBuggy=20,\n        RoadSweeper,\n        BarCodeScanner,\n        Printer,\n        Computer,\n        Gritter,\n        AirStarterUnit,\n        AircraftEngineeringServicingEquipment, // ASE\n        AircraftTowBar, // ACTB\n        AircraftTug, // TUGS - 30\n        BaggageDollie, // BAGD\n        BaggagePOD, // POD\n        BaggageTug, // EBT\n        BeltLoader, // BELT\n        Car,\n        Van, // CAR\n        CateringVehicle, // CATV\n        Coach, // COAC\n        DeIcingVehicle, // DEIC\n        GroundPowerUnit, // GPU\n        HighLoader, // HILO - 40\n        Lorry,\n        LowLoader, // LOLO\n        Minibus, // MBUS\n        MotorisedStep, // MSTP\n        NonMotorisedStep, // STPN\n        PassengerBoardingRamp, // PBR\n        PassengerMobility, // LIFT - Ambulift\n        FuelBowser,\n        WaterBowser,\n";

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        pasted
    )?);

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        format!("/{pasted}")
    );
    Ok(())
}

#[test]
fn acp_at_symbol_opens_git_file_picker_and_return_inserts_mention() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = init_git_repo("acp-files")?;
    fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    fs::write(root.join("src").join("main.rs"), "fn main() {}\n")
        .map_err(|error| error.to_string())?;
    run_git_in_dir(&root, &["add", "src/main.rs"])?;
    open_workspace_from_project(&mut state.runtime, "acp-files", &root)
        .map_err(|error| error.to_string())?;

    let buffer_id = install_acp_test_buffer(&mut state, 0, "look at ", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .handle_text_input("@")
        .map_err(|error| error.to_string())?;

    {
        let ui = shell_ui(&state.runtime)?;
        let picker = ui
            .picker()
            .ok_or_else(|| "ACP file picker should open for @".to_owned())?;
        assert_eq!(picker.session().title(), "ACP Files");
        assert!(
            picker
                .session()
                .matches()
                .iter()
                .any(|matched| matched.item().label() == "src/main.rs"),
            "git file picker should list src/main.rs"
        );
        assert_eq!(ui.picker_kind(), Some(PickerKind::AcpFile { buffer_id }));
    }

    state
        .handle_text_input("main.rs")
        .map_err(|error| error.to_string())?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "look at @src/main.rs "
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn acp_paste_image_inserts_mention_token_and_stores_bytes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "see", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let image = normalize_clipboard_image(TINY_PNG.to_vec(), Some("image/png"), "Image")
        .ok_or_else(|| "png should normalize".to_owned())?;
    assert!(paste_image_into_active_input_buffer(
        &mut state.runtime,
        image
    )?);

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "see ![Image](acp-image:1) "
    );
    let images = buffer.acp_pasted_images();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].id, 1);
    assert_eq!(images[0].mime_type, "image/png");
    assert!(!images[0].data.is_empty());
    Ok(())
}

#[test]
fn paste_text_into_active_input_buffer_updates_browser_input() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        "example.com/docs"
    )?);

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .text(),
        "example.com/docs"
    );
    Ok(())
}

#[test]
fn browser_location_updates_rename_buffer_with_current_url() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    apply_browser_location_updates(
        &mut state.runtime,
        &[BrowserLocationUpdate {
            buffer_id,
            current_url: "https://docs.rs/volt".to_owned(),
        }],
    )?;

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    assert_eq!(buffer.display_name(), "*browser* https://docs.rs/volt");
    assert_eq!(
        buffer
            .browser_state
            .as_ref()
            .and_then(|browser| browser.current_url.as_deref()),
        Some("https://docs.rs/volt")
    );
    Ok(())
}

#[test]
fn browser_page_load_event_commits_current_url_and_clears_loading() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let user_library = shell_user_library(&state.runtime);
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        request_browser_buffer_navigation(
            buffer,
            "https://example.com/docs",
            false,
            &*user_library,
        );
    }

    state
        .apply_browser_host_events(&[BrowserHostEvent::PageLoadStateChanged {
            buffer_id,
            current_url: "https://example.com/docs".to_owned(),
            is_loading: false,
        }])
        .map_err(|error| error.to_string())?;

    let browser = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?
        .browser_state
        .as_ref()
        .ok_or_else(|| "browser state missing".to_owned())?;
    assert_eq!(
        browser.current_url.as_deref(),
        Some("https://example.com/docs")
    );
    assert_eq!(
        browser.requested_url.as_deref(),
        Some("https://example.com/docs")
    );
    assert!(!browser.is_loading);
    Ok(())
}

#[test]
fn browser_page_load_event_does_not_clobber_a_newer_requested_navigation() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let user_library = shell_user_library(&state.runtime);
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        request_browser_buffer_navigation(buffer, "https://example.com/old", false, &*user_library);
        request_browser_buffer_navigation(buffer, "https://example.com/new", false, &*user_library);
    }

    state
        .apply_browser_host_events(&[BrowserHostEvent::PageLoadStateChanged {
            buffer_id,
            current_url: "https://example.com/old".to_owned(),
            is_loading: false,
        }])
        .map_err(|error| error.to_string())?;

    let browser = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?
        .browser_state
        .as_ref()
        .ok_or_else(|| "browser state missing".to_owned())?;
    assert_eq!(browser.current_url.as_deref(), None);
    assert_eq!(
        browser.requested_url.as_deref(),
        Some("https://example.com/new")
    );
    assert!(browser.is_loading);
    Ok(())
}

#[test]
fn browser_page_load_event_accepts_redirect_after_location_sync() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let user_library = shell_user_library(&state.runtime);
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        request_browser_buffer_navigation(
            buffer,
            "https://example.com/start",
            false,
            &*user_library,
        );
    }

    apply_browser_location_updates(
        &mut state.runtime,
        &[BrowserLocationUpdate {
            buffer_id,
            current_url: "https://example.com/redirected#section".to_owned(),
        }],
    )?;

    state
        .apply_browser_host_events(&[BrowserHostEvent::PageLoadStateChanged {
            buffer_id,
            current_url: "https://example.com/redirected#section".to_owned(),
            is_loading: false,
        }])
        .map_err(|error| error.to_string())?;

    let browser = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?
        .browser_state
        .as_ref()
        .ok_or_else(|| "browser state missing".to_owned())?;
    assert_eq!(
        browser.current_url.as_deref(),
        Some("https://example.com/redirected#section")
    );
    assert_eq!(
        browser.requested_url.as_deref(),
        Some("https://example.com/redirected#section")
    );
    assert!(!browser.is_loading);
    Ok(())
}

#[test]
fn hover_next_command_cycles_open_overlay_without_focus() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_hover_test_overlay(&mut state, false)?;
    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Alpha".to_owned())
    );

    cycle_hover_provider(&mut state.runtime, true)?;

    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Beta".to_owned())
    );
    assert!(!state.hover_focused().map_err(|error| error.to_string())?);
    Ok(())
}

#[test]
fn hover_previous_command_wraps_open_overlay_without_focus() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_hover_test_overlay(&mut state, false)?;
    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Alpha".to_owned())
    );

    cycle_hover_provider(&mut state.runtime, false)?;

    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Gamma".to_owned())
    );
    Ok(())
}

#[cfg(target_os = "windows")]
#[test]
fn system_symbol_fallback_font_covers_starship_prompt_glyphs() -> Result<(), String> {
    let fallback = resolve_system_icon_font_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("seguisym.ttf"))
        })
        .ok_or_else(|| "Segoe UI Symbol fallback font was not found".to_owned())?;
    let bytes = fs::read(&fallback).map_err(|error| error.to_string())?;
    let font = RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())?;

    for glyph in ['◎', '⎪', '▴', '●', '◦', '◃', '◈', '⎥', '⎈', '◨', '⊃'] {
        let (metrics, _) = font.rasterize(glyph, 48.0);
        assert!(
            metrics.width > 0 && metrics.height > 0,
            "fallback font did not cover `{glyph}` (U+{:04X})",
            glyph as u32
        );
    }
    Ok(())
}

#[test]
fn hover_tab_shortcut_focuses_open_overlay() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_hover_test_overlay(&mut state, false)?;
    assert!(state.hover_visible().map_err(|error| error.to_string())?);
    assert!(!state.hover_focused().map_err(|error| error.to_string())?);

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::empty())
            .map_err(|error| error.to_string())?
    );

    assert!(state.hover_focused().map_err(|error| error.to_string())?);
    Ok(())
}

#[test]
fn hover_tab_shortcut_beats_markdown_table_navigation_and_allows_scroll() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*hover-markdown-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
    let cursor_before = shell_buffer(&state.runtime, buffer_id)?.cursor_point();
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, false)?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(state.hover_focused().map_err(|error| error.to_string())?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        cursor_before
    );

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 1);
    Ok(())
}

#[test]
fn hover_ctrl_n_shortcut_prefers_hover_overlay_over_popup_cycle() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_hover_test_overlay(&mut state, false)?;
    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Alpha".to_owned())
    );

    assert!(
        state
            .try_runtime_keybinding(Keycode::N, ctrl_mod())
            .map_err(|error| error.to_string())?
    );

    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Beta".to_owned())
    );
    Ok(())
}

#[test]
fn markdown_table_detection_requires_markdown_and_a_delimiter_row() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let markdown = install_markdown_test_buffer(
        &mut state,
        "*markdown-table*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    let malformed = install_markdown_test_buffer(
        &mut state,
        "*markdown-malformed*",
        "| Header 1 | Header 2 |\n| nope | nope |\n| Some text | Some more text |",
    )?;
    let scratch = install_scratch_test_buffer(&mut state, "*not-markdown*")?;
    shell_buffer_mut(&mut state.runtime, scratch)?.replace_with_lines(vec![
        "| Header 1 | Header 2 |".to_owned(),
        "| --- | --- |".to_owned(),
    ]);

    let table =
        detect_markdown_table(shell_buffer(&state.runtime, markdown)?).ok_or("table missing")?;
    assert_eq!(table.start_line, 0);
    assert_eq!(table.column_count, 2);
    assert_eq!(table.rows.len(), 3);
    assert!(table.rows[1].is_delimiter);
    assert!(detect_markdown_table(shell_buffer(&state.runtime, malformed)?).is_none());
    assert!(detect_markdown_table(shell_buffer(&state.runtime, scratch)?).is_none());
    Ok(())
}

#[test]
fn markdown_table_typing_auto_aligns_and_bootstraps_delimiter_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-align*",
        "| Header 1 | Header 2 |\n| -- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 3));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    state
        .handle_text_input("-")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(0).as_deref(),
        Some("| Header 1  | Header 2       |")
    );
    assert_eq!(
        buffer.text.line(1).as_deref(),
        Some("| --------- | -------------- |")
    );
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text | Some more text |")
    );
    Ok(())
}

#[test]
fn markdown_table_enter_inserts_a_new_row() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-enter*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(3).as_deref(),
        Some("|           |                |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 2));
    Ok(())
}

#[test]
fn insert_mode_closing_brace_does_not_reindent_inline_block() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*inline-closing-brace*",
        vec!["fn main() {".to_owned(), "    ".to_owned(), "}".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(1, 4));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .handle_text_input("if true {")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("}")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(1).as_deref(), Some("    if true {}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(1, 14));
    Ok(())
}

#[test]
fn insert_mode_enter_splits_brace_pair_into_indented_line() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*brace-pair-enter*",
        vec!["if true {}".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 9));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("if true {"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    "));
    assert_eq!(buffer.text.line(2).as_deref(), Some("}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(1, 4));
    Ok(())
}

#[test]
fn insert_mode_enter_splits_bracket_pair_into_indented_line() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*bracket-pair-enter*",
        vec!["let items = [];".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 13));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("let items = ["));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    "));
    assert_eq!(buffer.text.line(2).as_deref(), Some("];"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(1, 4));
    Ok(())
}

#[test]
fn insert_mode_enter_in_tsx_uses_two_space_indent_query() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("tsx")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id =
        install_text_test_buffer(&mut state, "*tsx-enter*", vec!["<div></div>".to_owned()])?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("tsx".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 5));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("<div>"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("  "));
    assert_eq!(buffer.text.line(2).as_deref(), Some("</div>"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(1, 2));
    Ok(())
}

#[test]
fn format_current_line_indent_uses_inherited_tsx_queries() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("tsx")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*tsx-indent-query*",
        vec!["<div>".to_owned(), String::new(), "</div>".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("tsx".to_owned()));
        buffer.set_cursor(TextPoint::new(1, 0));
    }

    format_current_line_indent(&mut state.runtime, buffer_id, 2, false)?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("  ")
    );
    Ok(())
}

#[test]
fn vim_open_line_below_in_tsx_uses_inherited_indent_queries() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("tsx")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*tsx-vim-open-line*",
        vec![
            "export default function Dashboard() {".to_owned(),
            "  return (".to_owned(),
            "    <div className=\"flex flex-1 flex-col gap-4 p-4\">".to_owned(),
            "      <div className=\"flex items-center justify-between\">".to_owned(),
            "      </div>".to_owned(),
            "    </div>".to_owned(),
            "  );".to_owned(),
            "}".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("tsx".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 4));
    }

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(3).as_deref(), Some("      "));
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 6));
    Ok(())
}

#[test]
fn vim_open_line_below_before_typescript_closing_object_dedents_to_sibling_indent()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("typescript")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typescript-vim-open-line*",
        vec![
            ";".to_owned(),
            "export const Endpoints = (builder: EndpointBuilder<any, any, any>) => ({"
                .to_owned(),
            "  getOutdoorTrackingHistoryByCustomer: builder.query<DashboardTrackingHistory[], TrackingHistoryAttributes, DashboardTrackingHistoryDto[]>({".to_owned(),
            "    query: (args: TrackingHistoryAttributes) => `outdoordashboard/trackingactivity/${args.customerId}?days=${args.days}`,".to_owned(),
            "    transformResponse: (response: DashboardTrackingHistoryDto[]) => toDashboardTrackingHistorySummaries(response),".to_owned(),
            "    transformErrorResponse: (response: { status: string | number }, _meta, _arg) => response.status,".to_owned(),
            "    providesTags: [{ type: HOURLY_TAG, id: 'LIST' }],".to_owned(),
            "    keepUnusedDataFor: 300".to_owned(),
            "  }),".to_owned(),
            "});".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("typescript".to_owned()));
        buffer.set_cursor(TextPoint::new(8, 2));
    }

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("open-line-below"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(9).as_deref(), Some("  "));
    assert_eq!(buffer.cursor_point(), TextPoint::new(9, 2));
    Ok(())
}

#[test]
fn vim_open_line_below_after_typescript_outer_object_opener_uses_sibling_indent()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("typescript")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typescript-vim-open-top-level-entry*",
        vec![
            ";".to_owned(),
            "export const Endpoints = (builder: EndpointBuilder<any, any, any>) => ({"
                .to_owned(),
            "  getOutdoorTrackingHistoryByCustomer: builder.query<DashboardTrackingHistory[], TrackingHistoryAttributes, DashboardTrackingHistoryDto[]>({".to_owned(),
            "    query: (args: TrackingHistoryAttributes) => `outdoordashboard/trackingactivity/${args.customerId}?days=${args.days}`,".to_owned(),
            "    transformResponse: (response: DashboardTrackingHistoryDto[]) => toDashboardTrackingHistorySummaries(response),".to_owned(),
            "    transformErrorResponse: (response: { status: string | number }, _meta, _arg) => response.status,".to_owned(),
            "    providesTags: [{ type: HOURLY_TAG, id: 'LIST' }],".to_owned(),
            "    keepUnusedDataFor: 300".to_owned(),
            "  }),".to_owned(),
            "});".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("typescript".to_owned()));
        buffer.set_cursor(TextPoint::new(1, 0));
    }

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(2).as_deref(), Some("  "));
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 2));
    Ok(())
}

#[test]
fn recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let install_root = TempTestDir::new("treesitter-recompile-empty");
    state
        .runtime
        .services_mut()
        .insert(editor_syntax::SyntaxRegistry::with_install_root(
            install_root.path(),
        ));

    recompile_installed_tree_sitter_languages(&mut state.runtime)?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let notifications = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
    let notification = notifications
        .into_iter()
        .find(|notification| notification.key == "treesitter.recompile-installed")
        .ok_or_else(|| "tree-sitter recompile notification was not shown".to_owned())?;
    assert_eq!(notification.title, "Tree-sitter recompile complete");
    assert_eq!(
        notification.body_lines,
        vec!["No installed Tree-sitter grammars found.".to_owned()]
    );
    Ok(())
}

#[test]
fn format_current_line_indent_uses_syntax_queries_for_blank_lines() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    syntax_registry_mut(&mut state.runtime)?
        .register(
            editor_syntax::LanguageConfiguration::new(
                "rust-test-indent",
                ["rs"],
                rust_test_language,
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                [editor_syntax::CaptureThemeMapping::new(
                    "keyword",
                    "syntax.keyword",
                )],
            )
            .with_extra_indent_query(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../volt/assets/grammars/queries/rust/indents.scm"
            ))),
        )
        .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*rust-indent*")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "fn main() {".to_owned(),
            "    if true {".to_owned(),
            String::new(),
            "    }".to_owned(),
            "}".to_owned(),
        ]);
        buffer.set_language_id(Some("rust-test-indent".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 0));
    }

    format_current_line_indent(&mut state.runtime, buffer_id, 4, false)?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(2)
            .as_deref(),
        Some("        ")
    );
    Ok(())
}

#[test]
fn format_current_line_indent_uses_syntax_queries_for_closing_braces() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    syntax_registry_mut(&mut state.runtime)?
        .register(
            editor_syntax::LanguageConfiguration::new(
                "rust-test-dedent",
                ["rs"],
                rust_test_language,
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                [editor_syntax::CaptureThemeMapping::new(
                    "keyword",
                    "syntax.keyword",
                )],
            )
            .with_extra_indent_query(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../volt/assets/grammars/queries/rust/indents.scm"
            ))),
        )
        .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*rust-dedent*")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "fn main() {".to_owned(),
            "    if true {".to_owned(),
            "        }".to_owned(),
            "}".to_owned(),
        ]);
        buffer.set_language_id(Some("rust-test-dedent".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 8));
    }

    format_current_line_indent(&mut state.runtime, buffer_id, 4, false)?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(2)
            .as_deref(),
        Some("    }")
    );
    Ok(())
}

#[test]
fn format_current_line_indent_skips_cold_syntax_parse_for_large_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    syntax_registry_mut(&mut state.runtime)?
        .register(
            editor_syntax::LanguageConfiguration::new(
                "rust-test-large-indent",
                ["rs"],
                rust_test_language,
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                [editor_syntax::CaptureThemeMapping::new(
                    "keyword",
                    "syntax.keyword",
                )],
            )
            .with_extra_indent_query(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../volt/assets/grammars/queries/rust/indents.scm"
            ))),
        )
        .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*rust-large-indent*")?;
    let mut lines = vec![String::new(); LARGE_BUFFER_SYNC_INDENT_LINE_THRESHOLD + 4];
    lines[0] = "fn main() {".to_owned();
    lines[1] = "    if true {".to_owned();
    lines[2] = String::new();
    lines[3] = "    }".to_owned();
    lines[4] = "}".to_owned();
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(lines);
        buffer.set_language_id(Some("rust-test-large-indent".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 0));
    }

    format_current_line_indent(&mut state.runtime, buffer_id, 4, false)?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(2)
            .as_deref(),
        Some("        ")
    );
    assert!(shell_ui(&state.runtime)?.indent_parse_sessions.is_empty());
    Ok(())
}

#[test]
fn markdown_table_preserves_insert_mode_spaces() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-space*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 11));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .handle_text_input(" ")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text  | Some more text |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 12));
    let _ = buffer;

    state
        .handle_text_input("m")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text m | Some more text |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 13));
    Ok(())
}

#[test]
fn insert_mode_tab_inserts_spaces_using_language_theme_options() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(&mut state, "*rust-insert-tab*", vec![String::new()])?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let theme_registry = state.runtime.services().get::<ThemeRegistry>();
    assert!(!theme_lang_use_tabs(theme_registry, Some("rust")));
    let expected = tab_insert_string(theme_registry, Some("rust"));
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some(expected.as_str()));
    assert_eq!(
        buffer.cursor_point(),
        TextPoint::new(0, expected.chars().count())
    );
    Ok(())
}

#[test]
fn replace_mode_tab_inserts_make_tabs_using_language_theme_options() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*make-replace-tab*", vec!["recipe".to_owned()])?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("make".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    shell_ui_mut(&mut state.runtime)?.enter_replace_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let theme_registry = state.runtime.services().get::<ThemeRegistry>();
    assert!(theme_lang_use_tabs(theme_registry, Some("make")));
    let expected = tab_insert_string(theme_registry, Some("make"));
    let expected_line = format!("{expected}ecipe");
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some(expected_line.as_str()));
    assert_eq!(
        buffer.cursor_point(),
        TextPoint::new(0, expected.chars().count())
    );
    Ok(())
}

#[test]
fn markdown_table_insert_tab_adds_a_column_across_the_table() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 14));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(0).as_deref(),
        Some("| Header 1  | Header 2       |   |")
    );
    assert_eq!(
        buffer.text.line(1).as_deref(),
        Some("| --------- | -------------- | --- |")
    );
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text | Some more text |   |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 31));
    Ok(())
}

#[test]
fn markdown_table_normal_tab_moves_between_columns() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-normal-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 14)
    );
    Ok(())
}

#[test]
fn non_table_normal_tab_still_cycles_panes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*pane-a*")?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    cycle_runtime_pane(&mut state.runtime)?;
    let buffer_b = install_scratch_test_buffer(&mut state, "*pane-b*")?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_ne!(ui.active_buffer_id(), Some(buffer_b));
    Ok(())
}

#[test]
fn focused_hover_text_motions_scroll_without_moving_buffer_cursor() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, true)?;
    let cursor_before = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();

    state
        .handle_text_input("3")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 3);

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 4);

    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 3);
    assert_eq!(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?
            .cursor_point(),
        cursor_before
    );
    Ok(())
}

#[test]
fn focused_hover_gg_and_g_scroll_to_expected_bounds() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, true)?;
    let cursor_before = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();

    state
        .handle_text_input("G")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 8);

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 0);

    state
        .handle_text_input("5")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 4);

    state
        .handle_text_input("2")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("0")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("G")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 8);
    assert_eq!(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?
            .cursor_point(),
        cursor_before
    );
    Ok(())
}

#[test]
fn vim_repeat_search_preserves_forward_and_backward_bindings() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*vim-search-repeat*",
        vec![
            "alpha".to_owned(),
            "beta".to_owned(),
            "alpha".to_owned(),
            "beta".to_owned(),
            "alpha".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));

    run_vim_search(&mut state.runtime, VimSearchDirection::Forward, "alpha")?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 0)
    );

    repeat_vim_search(&mut state.runtime, true)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(0, 0)
    );
    assert!(matches!(
        shell_ui(&state.runtime)?.vim().last_search,
        Some(LastSearch {
            direction: VimSearchDirection::Forward,
            ..
        })
    ));

    repeat_vim_search(&mut state.runtime, false)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 0)
    );
    assert!(matches!(
        shell_ui(&state.runtime)?.vim().last_search,
        Some(LastSearch {
            direction: VimSearchDirection::Forward,
            ..
        })
    ));
    Ok(())
}

#[test]
fn focused_hover_ctrl_scroll_motions_are_bounded() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, true)?;
    let cursor_before = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::D, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 2);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::F, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 6);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::E, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 7);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::Y, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 6);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::B, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 2);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::U, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 0);
    assert_eq!(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?
            .cursor_point(),
        cursor_before
    );
    Ok(())
}

#[test]
fn vim_g_prefix_executes_workspace_keybinding() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.g-prefix-exact",
            "Test exact g-prefix binding",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push("exact".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .register_key_binding_for_mode(
            "g z",
            "tests.g-prefix-exact",
            KeymapScope::Workspace,
            KeymapVimMode::Normal,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        state.ui().map_err(|error| error.to_string())?.vim().pending,
        Some(VimPending::GPrefix {
            operator: None,
            line_target: None,
        })
    );

    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec!["exact".to_owned()]
    );
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(ui.vim().pending, None);
    assert_eq!(ui.vim().pending_change_prefix, None);
    Ok(())
}

#[test]
fn vim_g_prefix_preserves_longer_workspace_sequence() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.g-prefix-sequence",
            "Test longer g-prefix binding",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push("sequence".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .register_key_binding_for_mode(
            "g z z",
            "tests.g-prefix-sequence",
            KeymapScope::Workspace,
            KeymapVimMode::Normal,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        Vec::<String>::new()
    );
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(
        ui.vim().pending,
        Some(VimPending::GPrefix {
            operator: None,
            line_target: None,
        })
    );
    assert_eq!(
        ui.vim().pending_change_prefix,
        Some(VimRecordedInput::Chord("g z".to_owned()))
    );

    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec!["sequence".to_owned()]
    );
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(ui.vim().pending, None);
    assert_eq!(ui.vim().pending_change_prefix, None);
    Ok(())
}

#[test]
fn vim_command_line_completion_includes_user_aliases() -> Result<(), String> {
    let state = state_with_user_library()?;

    let write_matches = vim_command_line_completion_matches(&state.runtime, "wr");
    assert!(write_matches.contains(&"write".to_owned()));

    let buffer_matches = vim_command_line_completion_matches(&state.runtime, "bd");
    assert!(buffer_matches.contains(&"bd".to_owned()));
    assert!(buffer_matches.contains(&"bdelete".to_owned()));
    Ok(())
}

#[test]
fn execute_vim_command_line_split_alias_splits_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    execute_vim_command_line(&mut state.runtime, "split")?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 2);
    Ok(())
}

#[test]
fn execute_vim_command_line_commands_alias_opens_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    execute_vim_command_line(&mut state.runtime, "commands")?;
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    Ok(())
}

#[test]
fn ctrl_enter_variants_match_manual_lsp_code_action_command() -> Result<(), String> {
    let root = unique_temp_dir("lsp-code-action-binding");
    let path = root.join("main.rs");
    fs::write(
        &path,
        "fn main() {\n    let value = 1;\n    let _ = value;\n}\n",
    )
    .map_err(|error| error.to_string())?;

    let manual_title = {
        let mut state = state_with_user_library()?;
        open_workspace_from_project(&mut state.runtime, "lsp-code-actions-manual", &root)?;
        open_workspace_file(&mut state.runtime, &path)?;
        shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
        state
            .runtime
            .execute_command("lsp.code-action")
            .map_err(|error| error.to_string())?;
        shell_ui(&state.runtime)?
            .picker()
            .map(|picker| picker.session.title().to_owned())
            .ok_or_else(|| "manual lsp code-action did not open a picker".to_owned())?
    };

    for (name, keycode) in [
        ("return", Keycode::Return),
        ("kp-enter", Keycode::KpEnter),
        ("return2", Keycode::Return2),
    ] {
        let mut state = state_with_user_library()?;
        open_workspace_from_project(
            &mut state.runtime,
            &format!("lsp-code-actions-binding-{name}"),
            &root,
        )?;
        open_workspace_file(&mut state.runtime, &path)?;
        shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

        let binding = state
            .runtime
            .keymaps()
            .get_for_mode(
                &editor_core::KeymapScope::Workspace,
                editor_core::KeymapVimMode::Normal,
                "Ctrl+Enter",
            )
            .ok_or_else(|| "Ctrl+Enter workspace binding is missing".to_owned())?;
        assert_eq!(binding.command_name(), "lsp.code-actions");

        let (render_width, render_height, cell_width, line_height) =
            markdown_table_event_dimensions();
        let handled = state
            .handle_event(
                Event::KeyDown {
                    timestamp: 0,
                    window_id: 0,
                    keycode: Some(keycode),
                    scancode: None,
                    keymod: ctrl_mod(),
                    repeat: false,
                    which: 0,
                    raw: 0,
                },
                render_width,
                render_height,
                cell_width,
                line_height,
            )
            .map_err(|error| error.to_string())?;

        assert!(!handled);
        let binding_title = shell_ui(&state.runtime)?
            .picker()
            .map(|picker| picker.session.title().to_owned())
            .ok_or_else(|| format!("Ctrl+Enter variant `{name}` did not open an LSP picker"))?;
        assert_eq!(binding_title, manual_title);
    }

    fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn f7_keydown_opens_keybinding_picker_from_user_binding() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let binding = state
        .runtime
        .keymaps()
        .get(&editor_core::KeymapScope::Global, "F7")
        .ok_or_else(|| "F7 global binding is missing".to_owned())?;
    assert_eq!(binding.command_name(), "picker.open-keybindings");

    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::F7),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    let picker_title = shell_ui(&state.runtime)?
        .picker()
        .map(|picker| picker.session.title().to_owned())
        .ok_or_else(|| "F7 binding did not open the keybinding picker".to_owned())?;
    assert_eq!(picker_title, "Keybindings");
    Ok(())
}

#[test]
fn browser_normal_mode_i_binding_focuses_input_without_inserting_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.enter_normal_mode();
        ui.set_active_vim_target(VimTarget::Buffer);
    }

    state
        .handle_text_input("I")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_id));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .text(),
        ""
    );
    Ok(())
}

#[test]
fn browser_insert_mode_enter_binding_submits_current_url() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_browser_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .set_text("example.com/docs");
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
    Ok(())
}

#[test]
fn browser_insert_mode_ctrl_enter_binding_submits_current_url() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_browser_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .set_text("example.com/docs");
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: ctrl_mod(),
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
    Ok(())
}

#[test]
fn leader_space_o_b_opens_browser_from_normal_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .handle_text_input(" ")
        .map_err(|error| error.to_string())?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);

    state
        .handle_text_input("b")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let browser_buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_ne!(browser_buffer_id, original_buffer_id);
    assert_eq!(ui.pane_count(), 2);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(matches!(
        shell_buffer(&state.runtime, browser_buffer_id)?.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
    Ok(())
}

#[test]
fn execute_vim_command_line_substitute_defaults_to_current_line() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*substitute-current-line*",
        vec!["alpha one".to_owned(), "alpha two".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));

    execute_vim_command_line(&mut state.runtime, "s/alpha/omega/")?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("omega one"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("alpha two"));
    Ok(())
}

#[test]
fn execute_vim_command_line_substitute_supports_numeric_ranges() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*substitute-range*",
        vec![
            "alpha one".to_owned(),
            "alpha two".to_owned(),
            "alpha three".to_owned(),
        ],
    )?;

    execute_vim_command_line(&mut state.runtime, "2,3s/alpha/beta/")?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha one"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("beta two"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("beta three"));
    Ok(())
}

#[test]
fn gcc_toggles_current_line_comments() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*comment-line*",
        vec![
            "fn main() {".to_owned(),
            "    println!(\"hi\");".to_owned(),
            "}".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_language_id(Some("rust".to_owned()));
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 4));

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.vim().pending,
        Some(VimPending::CommentToggle { count: 1 })
    );
    assert_eq!(
        shell_ui(&state.runtime)?.vim().pending_change_prefix,
        Some(VimRecordedInput::Chord("g c".to_owned()))
    );
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    // println!(\"hi\");")
    );

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(1).as_deref(),
        Some("    println!(\"hi\");")
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

fn run_gcc_comment_toggle(state: &mut ShellState) -> Result<(), String> {
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())
}

#[test]
fn comment_toggle_styles_cover_all_shipped_syntax_languages() {
    let missing = user::syntax_languages()
        .into_iter()
        .filter_map(|language| {
            comment_style_for_language_path(
                Some(language.id()),
                language.file_extensions().first().map(String::as_str),
                language.file_names().first().map(String::as_str),
            )
            .is_none()
            .then(|| language.id().to_owned())
        })
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "missing comment styles for: {}",
        missing.join(", ")
    );
}

#[test]
fn gcc_toggles_prefix_comment_styles() -> Result<(), String> {
    for (language_id, original, commented) in [
        ("clojure", "  (inc value)", "  ; (inc value)"),
        ("latex", "  \\section{Intro}", "  % \\section{Intro}"),
        ("vim", "  set number", "  \" set number"),
    ] {
        let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
        let mut state =
            ShellState::new_with_user_library(default_error_log_path(), false, user_library)
                .map_err(|error| error.to_string())?;
        let buffer_id = install_text_test_buffer(
            &mut state,
            &format!("*{language_id}-comment-line*"),
            vec![original.to_owned()],
        )?;
        shell_buffer_mut(&mut state.runtime, buffer_id)?
            .set_language_id(Some(language_id.to_owned()));
        shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(commented),
            "expected `{language_id}` to use `{commented}`",
        );

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(original),
            "expected `{language_id}` to restore the original line",
        );
    }

    Ok(())
}

#[test]
fn gcc_toggles_block_comment_styles() -> Result<(), String> {
    for (language_id, original, commented) in [
        ("css", "  color: red;", "  /* color: red; */"),
        ("html", "  <div>volt</div>", "  <!-- <div>volt</div> -->"),
        (
            "json",
            "  \"name\": \"volt\",",
            "  /* \"name\": \"volt\", */",
        ),
        ("markdown", "  - item", "  <!-- - item -->"),
        ("xml", "  <tag/>", "  <!-- <tag/> -->"),
    ] {
        let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
        let mut state =
            ShellState::new_with_user_library(default_error_log_path(), false, user_library)
                .map_err(|error| error.to_string())?;
        let buffer_id = install_text_test_buffer(
            &mut state,
            &format!("*{language_id}-block-comment-line*"),
            vec![original.to_owned()],
        )?;
        shell_buffer_mut(&mut state.runtime, buffer_id)?
            .set_language_id(Some(language_id.to_owned()));
        shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(commented),
            "expected `{language_id}` to use `{commented}`",
        );

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(original),
            "expected `{language_id}` to restore the original line",
        );
    }

    Ok(())
}

#[test]
fn visual_gc_toggles_region_comments() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*comment-region*",
        vec![
            "let alpha = 1;".to_owned(),
            "let beta = 2;".to_owned(),
            "let gamma = 3;".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_language_id(Some("rust".to_owned()));
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("// let alpha = 1;"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("// let beta = 2;"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("let gamma = 3;"));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("let alpha = 1;"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("let beta = 2;"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("let gamma = 3;"));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn visual_put_replaces_selection_and_updates_yank() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-put*",
        vec!["alpha beta gamma".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 6));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 6), VisualSelectionKind::Character);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 9));
    shell_ui_mut(&mut state.runtime)?.vim_mut().yank =
        Some(YankRegister::Character("delta".to_owned()));

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-put-after"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha delta gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 11));
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(
        ui.vim().yank,
        Some(YankRegister::Character("beta".to_owned()))
    );
    Ok(())
}

#[test]
fn visual_indent_shifts_selected_lines_right() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-indent*",
        vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));

    state
        .runtime
        .emit_hook(HOOK_VIM_EDIT, HookEvent::new().with_detail("visual-indent"))
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("    alpha"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    beta"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 4));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn visual_outdent_shifts_selected_lines_left() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-outdent*",
        vec![
            "    alpha".to_owned(),
            "        beta".to_owned(),
            "gamma".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-outdent"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    beta"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 0));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn visual_join_merges_selected_lines() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-join*",
        vec!["alpha".to_owned(), "  beta".to_owned(), "gamma".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));

    state
        .runtime
        .emit_hook(HOOK_VIM_EDIT, HookEvent::new().with_detail("visual-join"))
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.line_count(), 2);
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha beta"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 5));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn visual_move_down_reorders_selected_lines_and_keeps_visual_selection() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-move-down*",
        vec![
            "fn main() {".to_owned(),
            "    if ready {".to_owned(),
            "        alpha();".to_owned(),
            "    }".to_owned(),
            "}".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(2, 0), VisualSelectionKind::Line);

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-move-down"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("fn main() {"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    if ready {"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("    }"));
    assert_eq!(buffer.text.line(3).as_deref(), Some("    alpha();"));
    assert_eq!(buffer.text.line(4).as_deref(), Some("}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 0));
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(ui.vim().visual_anchor, Some(TextPoint::new(3, 0)));
    Ok(())
}

#[test]
fn visual_move_up_reorders_selected_lines_and_reindents() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-move-up*",
        vec![
            "fn main() {".to_owned(),
            "    if ready {".to_owned(),
            "    }".to_owned(),
            "    alpha();".to_owned(),
            "}".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(3, 0));
    }
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(3, 0), VisualSelectionKind::Line);

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-move-up"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("fn main() {"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    if ready {"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("        alpha();"));
    assert_eq!(buffer.text.line(3).as_deref(), Some("    }"));
    assert_eq!(buffer.text.line(4).as_deref(), Some("}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 0));
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(ui.vim().visual_anchor, Some(TextPoint::new(2, 0)));
    Ok(())
}

#[test]
fn visual_replace_char_replaces_selected_text() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-replace-char*",
        vec!["alpha".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 1));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 1), VisualSelectionKind::Character);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 3));

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-replace-char"),
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("axxxa"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 1));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn browser_viewport_rect_stays_above_prompt_footer() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        18,
        8,
        state.user_library.commandline_enabled(),
    );
    let viewport = browser_viewport_rect(
        buffer,
        rect,
        8,
        18,
        state.user_library.commandline_enabled(),
    )
    .ok_or_else(|| "browser viewport missing".to_owned())?;
    let viewport_bottom = viewport.y + viewport.height as i32;

    assert!(viewport.width > 0);
    assert!(viewport.height > 0);
    assert!(viewport.y >= layout.body_y - 2);
    assert!(viewport_bottom <= layout.input_y);
    Ok(())
}

#[test]
fn browser_surface_hit_testing_excludes_prompt_footer() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 480,
                height: 180,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
    )
    .map_err(|error| error.to_string())?;
    let surface = plan
        .visible_surfaces
        .iter()
        .find(|surface| surface.buffer_id == buffer_id)
        .ok_or_else(|| "browser surface missing".to_owned())?;

    assert_eq!(
        browser_surface_buffer_at_point(&plan, surface.rect.x + 4, surface.rect.y + 4),
        Some(buffer_id)
    );
    assert_eq!(
        browser_surface_buffer_at_point(
            &plan,
            surface.rect.x + 4,
            surface.rect.y + surface.rect.height as i32 + 4
        ),
        None
    );
    Ok(())
}

#[test]
fn browser_sync_plan_excludes_pdf_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("pdf-browser-plan");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["page one"])?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 400,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(
        plan.buffers
            .iter()
            .all(|buffer| buffer.buffer_id != buffer_id)
    );
    assert!(
        plan.visible_surfaces
            .iter()
            .all(|surface| surface.buffer_id != buffer_id)
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn browser_sync_plan_hides_surfaces_while_picker_is_visible() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .set_picker(PickerOverlay::from_entries("Buffers", Vec::new()));

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 400,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(plan.buffers.len(), 1);
    assert!(plan.visible_surfaces.is_empty());
    Ok(())
}

#[test]
fn browser_sync_plan_avoids_notification_overlays() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    let now = Instant::now();
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .apply_notification(
            test_notification_update(
                "progress",
                NotificationSeverity::Info,
                "LSP · rust-analyzer",
                &[
                    "Indexing workspace",
                    "Scanning project files",
                    "Resolving dependencies",
                    "Refreshing diagnostics",
                    "Updating symbol cache",
                    "Preparing semantic tokens",
                ],
                Some(32),
                true,
            ),
            now,
        );

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 260,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now,
        },
    )
    .map_err(|error| error.to_string())?;
    let notifications = state
        .ui()
        .map_err(|error| error.to_string())?
        .visible_notifications(now);
    let notification_rects = notification_overlay_layouts(&notifications, 800, 260, 8, 18)
        .into_iter()
        .map(|layout| layout.rect)
        .collect::<Vec<_>>();

    assert_eq!(plan.buffers.len(), 1);
    assert!(!notification_rects.is_empty());
    assert!(plan.visible_surfaces.iter().all(|surface| {
        notification_rects
            .iter()
            .all(|overlay| !rects_intersect(browser_viewport_rect_rect(surface.rect), *overlay))
    }));
    Ok(())
}

#[test]
fn detect_browser_url_uses_cursor_hit_or_single_line_url() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("See https://example.com/docs.");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 10));
    let cursor_hit = detect_browser_url(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    )
    .ok_or_else(|| "browser URL missing under cursor".to_owned())?;
    assert_eq!(cursor_hit, "https://example.com/docs");

    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 0));
    let single_url = detect_browser_url(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    )
    .ok_or_else(|| "browser URL missing from single-url line".to_owned())?;
    assert_eq!(single_url, "https://example.com/docs");
    Ok(())
}

#[test]
fn browser_url_command_opens_split_browser_with_detected_url() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("Docs: https://example.com/docs.");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 8));

    open_detected_browser_url(&mut state.runtime)?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "browser split buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    Ok(())
}

#[test]
fn browser_open_buffer_command_opens_split_with_file_url() -> Result<(), String> {
    let root = unique_temp_dir("browser-open-buffer");
    let html_path = root.join("page.html");
    std::fs::write(&html_path, "<html><body>preview</body></html>")
        .map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("<html><body>preview</body></html>");
        buffer.text.set_path(html_path.clone());
    }

    open_active_buffer_in_browser_split(&mut state.runtime)?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "browser split buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    let expected_url = path_to_file_url(&html_path);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some(expected_url.as_str())
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn browser_open_buffer_command_uses_existing_split_pane() -> Result<(), String> {
    let root = unique_temp_dir("browser-open-buffer-split");
    let html_path = root.join("preview.html");
    std::fs::write(&html_path, "<html></html>").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let source_buffer_id = active_shell_buffer_id(&state.runtime)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("<html></html>");
        buffer.text.set_path(html_path.clone());
    }
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 2);
    focus_test_buffer(&mut state, source_buffer_id)?;

    open_active_buffer_in_browser_split(&mut state.runtime)?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let browser_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(browser_buffer_id)
        .ok_or_else(|| "browser buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    assert!(
        ui.panes()
            .is_some_and(|panes| panes.iter().any(|pane| pane.buffer_id == source_buffer_id)),
        "source file buffer should remain open in the other pane"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn sync_active_browser_buffer_enters_insert_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            BROWSER_BUFFER_NAME,
            BufferKind::Plugin(BROWSER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;

    sync_active_buffer(&mut state.runtime)?;
    state
        .handle_text_input("example.com")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_id));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .text(),
        "example.com"
    );
    Ok(())
}

#[test]
fn browser_host_focus_parent_event_returns_to_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .enter_insert_mode();

    state
        .apply_browser_host_events(&[BrowserHostEvent::FocusParentRequested { buffer_id }])
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state.ui().map_err(|error| error.to_string())?.input_mode(),
        InputMode::Normal
    );
    Ok(())
}

#[test]
fn browser_host_new_window_event_routes_into_browser_popup() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    state
        .apply_browser_host_events(&[BrowserHostEvent::NewWindowRequested {
            buffer_id,
            url: "https://example.com/oauth/callback?code=test".to_owned(),
        }])
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "browser popup was not opened from new-window event".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    let popup_buffer = ui
        .buffer(popup.active_buffer)
        .ok_or_else(|| "popup browser buffer missing".to_owned())?;
    assert!(ui.popup_focus);
    assert!(matches!(
        popup_buffer.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
    assert_eq!(
        popup_buffer
            .browser_state
            .as_ref()
            .and_then(|browser| browser.requested_url.as_deref()),
        Some("https://example.com/oauth/callback?code=test")
    );
    Ok(())
}

#[test]
fn db_table_preview_buffer_exposes_hidden_sqls_path_without_file_open_hooks() -> Result<(), String>
{
    let state_dir = TempTestDir::new("db-preview-no-file-open-hooks");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("preview.sqlite3");
    let mut state = state_with_user_library()?;
    let connection_string = format!("sqlite://{}", db_path.display());
    let session = db_service_mut(&mut state.runtime)?
        .connect_raw(&connection_string, Some("preview"))
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .attach_query_buffer(99, Some(session.id), None)
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .execute_sql_for_buffer(
            99,
            "CREATE TABLE widgets (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
        )
        .map_err(|error| error.to_string())?;

    open_db_query_for_table_preview(
        &mut state.runtime,
        session.id,
        &QualifiedName {
            schema: None,
            name: "widgets".to_owned(),
        },
    )?;

    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert!(buffer_is_db_query(&buffer.kind));
    assert_eq!(buffer.language_id(), Some("sql"));
    assert!(
        buffer.lsp_enabled(),
        "DB scratch query buffers should opt into sqls syncs"
    );
    assert!(
        buffer.desired_syntax_window().is_some(),
        "DB scratch query buffers should be queued for tree-sitter highlighting"
    );
    assert!(
        buffer.path().is_none(),
        "DB scratch query buffers should not masquerade as file-backed workspace buffers",
    );
    assert!(
        buffer
            .lsp_path()
            .is_some_and(|path| path.extension().and_then(|value| value.to_str()) == Some("sql")),
        "DB scratch query buffers should expose a hidden .sql path for sqls",
    );
    assert!(
        formatter_registry(&state.runtime)?
            .formatter_for_language("sql")
            .is_none(),
        "DB scratch query buffers should not trigger generic file-open formatter hooks",
    );
    assert_eq!(
        syntax_indent_for_buffer(&mut state.runtime, buffer_id, 0, 2, false)?,
        None,
        "DB scratch query buffers should use text-only indentation"
    );
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.open_line_below();
    }
    format_current_line_indent(&mut state.runtime, buffer_id, 2, false)?;
    assert!(
        shell_user_library(&state.runtime)
            .plugin_buffer_key_bindings(DB_QUERY_KIND)
            .iter()
            .any(|binding| binding.chord() == "Ctrl+c Ctrl+c"
                && binding
                    .command_names()
                    .iter()
                    .any(|command| command.as_str() == "db.execute-sql")),
        "DB query buffers should expose the execute SQL chord"
    );
    Ok(())
}

#[test]
fn db_query_buffer_receives_sql_highlighting_without_blocking() -> Result<(), String> {
    let state_dir = TempTestDir::new("db-query-syntax-refresh");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("query.sqlite3");
    let mut state = state_with_user_library()?;
    let connection_string = format!("sqlite://{}", db_path.display());
    db_service_mut(&mut state.runtime)?
        .connect_raw(&connection_string, Some("query"))
        .map_err(|error| error.to_string())?;

    open_db_query_buffer(&mut state.runtime)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert!(buffer_is_db_query(&buffer.kind));
    assert_eq!(buffer.language_id(), Some("sql"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(3).is_some_and(|spans| {
            spans
                .iter()
                .any(|span| span.theme_token.starts_with("syntax.keyword"))
        }),
        "DB query starter SQL should receive keyword highlighting"
    );
    Ok(())
}

#[test]
fn opened_sql_file_survives_layout_and_syntax_refresh() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-sql-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("query.sql");
    fs::write(&path, "SELECT *\nFROM widgets\nWHERE id = 1;\n")
        .map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;
    sync_active_buffer_layout_for_test(&mut state)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.language_id(), Some("sql"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(0).is_some_and(|spans| {
            spans
                .iter()
                .any(|span| span.theme_token.starts_with("syntax.keyword"))
        }),
        "opened SQL file should receive keyword highlight spans"
    );
    Ok(())
}

#[test]
fn db_dashboard_layout_places_sidebar_left_and_editor_output_right() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    open_db_dashboard(&mut state.runtime)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let panes = plugin_section_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "dashboard section layout missing".to_owned())?;
    assert_eq!(panes.panes.len(), 4);
    let editor = panes.panes[0].rect;
    let connections = panes.panes[1].rect;
    let tables = panes.panes[2].rect;
    let output = panes.panes[3].rect;
    assert!(
        connections.x() < editor.x(),
        "Connections should sit left of Editor"
    );
    assert!(tables.x() < output.x(), "Tables should sit left of Output");
    assert!(
        tables.y() > connections.y(),
        "Tables should sit below Connections"
    );
    assert!(output.y() > editor.y(), "Output should sit below Editor");
    Ok(())
}

#[test]
fn db_dashboard_execute_replaces_output_and_concatenates_multiple_queries() -> Result<(), String> {
    let state_dir = TempTestDir::new("db-dashboard-execute");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("dashboard.sqlite3");
    let mut state = state_with_user_library()?;
    let connection_string = format!("sqlite://{}", db_path.display());
    db_service_mut(&mut state.runtime)?
        .connect_raw(&connection_string, Some("dashboard"))
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .attach_query_buffer(99, None, None)
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .execute_sql_for_buffer(
            99,
            "CREATE TABLE widgets (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
        )
        .map_err(|error| error.to_string())?;
    db_service_mut(&mut state.runtime)?
        .execute_sql_for_buffer(99, "INSERT INTO widgets(name) VALUES ('Ada'), ('Grace');")
        .map_err(|error| error.to_string())?;

    open_db_dashboard(&mut state.runtime)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec!["SELECT name FROM widgets WHERE id = 1;".to_owned()]);
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    }
    state
        .runtime
        .execute_command("db.execute-sql")
        .map_err(|error| error.to_string())?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let output = plugin_section_lines(buffer, DB_OUTPUT_SECTION)?;
        assert!(
            output.iter().any(|line| line.contains("Ada")),
            "first execute should write Ada into Output: {output:?}"
        );
        assert!(
            !output.iter().any(|line| line.contains("Grace")),
            "first execute should not include Grace: {output:?}"
        );
        assert_eq!(buffer.plugin_active_section_name(), Some(DB_EDITOR_SECTION));
    }

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec!["SELECT name FROM widgets WHERE id = 2;".to_owned()]);
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    }
    state
        .runtime
        .execute_command("db.execute-sql")
        .map_err(|error| error.to_string())?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let output = plugin_section_lines(buffer, DB_OUTPUT_SECTION)?;
        assert!(
            output.iter().any(|line| line.contains("Grace")),
            "second execute should replace Output with Grace: {output:?}"
        );
        assert!(
            !output.iter().any(|line| line.contains("Ada")),
            "second execute should overwrite Ada: {output:?}"
        );
    }

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "SELECT name FROM widgets WHERE id = 1;".to_owned(),
            "SELECT name FROM widgets WHERE id = 2;".to_owned(),
        ]);
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
    }
    state
        .runtime
        .execute_command("db.execute-sql")
        .map_err(|error| error.to_string())?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let output = plugin_section_lines(buffer, DB_OUTPUT_SECTION)?;
    assert!(
        output.iter().any(|line| line.contains("-- Query 1")),
        "batch execute should label first query: {output:?}"
    );
    assert!(
        output.iter().any(|line| line.contains("-- Query 2")),
        "batch execute should label second query: {output:?}"
    );
    assert!(output.iter().any(|line| line.contains("Ada")));
    assert!(output.iter().any(|line| line.contains("Grace")));
    Ok(())
}

#[test]
fn db_dashboard_opens_and_writes_files_through_editor_section() -> Result<(), String> {
    let root = TempTestDir::new("db-dashboard-file-open");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("query.sql");
    fs::write(&path, "SELECT 1;\n").map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;
    open_db_dashboard(&mut state.runtime)?;
    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        assert!(buffer_is_db_dashboard(&buffer.kind));
        assert_eq!(buffer.text.text().trim(), "SELECT 1;");
        assert_eq!(buffer.plugin_active_section_name(), Some(DB_EDITOR_SECTION));
        assert_eq!(buffer.path(), Some(path.as_path()));
    }
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec!["SELECT 2;".to_owned()]);
    }
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    save_buffer(&mut state.runtime, workspace_id, buffer_id)?;
    let saved = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    assert_eq!(saved.trim(), "SELECT 2;");
    Ok(())
}

#[test]
fn db_multiview_disables_golden_ratio_and_narrows_left_sidebar() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    open_db_multiview(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert_eq!(view.golden_ratio_override, Some(false));
    assert_eq!(
        view.pane_size_weights.as_deref(),
        Some([DB_MULTIVIEW_LEFT_WEIGHT, DB_MULTIVIEW_RIGHT_WEIGHT].as_slice())
    );
    assert_eq!(view.panes.len(), 2);
    let left = ui
        .buffer(view.panes[0].buffer_id)
        .ok_or_else(|| "left pane buffer missing".to_owned())?;
    let right = ui
        .buffer(view.panes[1].buffer_id)
        .ok_or_else(|| "right pane buffer missing".to_owned())?;
    assert!(buffer_is_db_sidebar(&left.kind));
    assert!(buffer_is_db_query(&right.kind));
    assert!(!buffer_is_db_dashboard(&right.kind));
    let rects = workspace_pane_rects(&*shell_user_library(&state.runtime), ui, 400, 200, 2);
    assert!(
        rects[0].width < rects[1].width,
        "multiview left split should be narrower: {rects:?}"
    );
    Ok(())
}

#[test]
fn db_multiview_toggle_restores_golden_ratio() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    open_db_multiview(&mut state.runtime)?;
    open_db_multiview(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert_eq!(view.golden_ratio_override, None);
    assert!(view.pane_size_weights.is_none());
    assert_eq!(view.panes.len(), 1);
    assert!(
        shell_user_library(&state.runtime)
            .pane_config()
            .golden_ratio,
        "default pane config should keep golden ratio enabled"
    );
    Ok(())
}

#[test]
fn db_connect_enter_submits_pasted_connection_string() -> Result<(), String> {
    let state_dir = TempTestDir::new("db-connect-enter");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("connect.sqlite3");
    let connection_string = format!("sqlite://{}", db_path.display());
    let mut state = state_with_user_library()?;

    state
        .runtime
        .execute_command("db.connect")
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui(&state.runtime)?;
        assert!(
            ui.popup_focus,
            "db.connect prompt should take popup focus so paste and Enter target the prompt"
        );
        assert_eq!(ui.input_mode(), InputMode::Insert);
        assert_eq!(ui.vim().target, VimTarget::Input);
    }
    assert!(
        paste_text_into_active_input_buffer(&mut state.runtime, &connection_string)
            .map_err(|error| error.to_string())?,
        "paste should land in the DB connect input"
    );
    let handled = state
        .try_runtime_keybinding(Keycode::Return, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    assert!(handled, "Enter should submit the DB connect prompt");
    let session = db_service(&state.runtime)?
        .active_session_summary()
        .ok_or_else(|| "Enter did not create a database session".to_owned())?;
    assert_eq!(session.engine.label(), "SQLite");
    Ok(())
}

#[test]
fn opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-toml-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("volt.toml");
    fs::write(&path, "title = \"Volt\"\n[editor]\nmode = \"vim\"\n")
        .map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;
    sync_active_buffer_layout_for_test(&mut state)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.language_id(), Some("toml"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(0).is_some(),
        "opened TOML file should receive syntax spans"
    );
    Ok(())
}

#[test]
fn opened_file_receives_tree_sitter_highlighting() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("main.rs");
    fs::write(&path, "fn main() {\n    let value = 1;\n}\n").map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;

    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .line_syntax_spans(0)
            .is_some_and(|spans| {
                spans
                    .iter()
                    .any(|span| span.theme_token.starts_with("syntax.keyword"))
            }),
        "opened file should receive syntax highlight spans"
    );
    Ok(())
}

#[test]
fn insert_mode_is_buffer_local_across_buffer_switches() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    let buffer_b = install_scratch_test_buffer(&mut state, "*vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);

    focus_test_buffer(&mut state, buffer_a)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn insert_mode_is_buffer_local_across_split_focus_changes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*split-vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    cycle_runtime_pane(&mut state.runtime)?;

    let buffer_b = install_scratch_test_buffer(&mut state, "*split-vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);

    cycle_runtime_pane(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Insert);

    cycle_runtime_pane(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn same_buffer_split_keeps_independent_cursor_and_scroll() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let lines = (0..64)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>();
    let buffer_id = install_text_test_buffer(&mut state, "*split-shared-buffer*", lines)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(buffer.id(), buffer_id);
        buffer.set_cursor(TextPoint::new(2, 3));
        buffer.scroll_row = 1;
    }

    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let pane_id = state
        .runtime
        .model_mut()
        .split_pane(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.split_pane(pane_id, buffer_id, PaneSplitDirection::Vertical);
    shell_ui_mut(&mut state.runtime)?.focus_pane(pane_id);
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(buffer.id(), buffer_id);
        buffer.set_cursor(TextPoint::new(20, 2));
        buffer.scroll_row = 18;
    }

    cycle_runtime_pane(&mut state.runtime)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(buffer.id(), buffer_id);
        assert_eq!(buffer.cursor_point(), TextPoint::new(2, 3));
        assert_eq!(buffer.scroll_row, 1);
    }

    cycle_runtime_pane(&mut state.runtime)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(buffer.id(), buffer_id);
        assert_eq!(buffer.cursor_point(), TextPoint::new(20, 2));
        assert_eq!(buffer.scroll_row, 18);
    }
    Ok(())
}

#[test]
fn inactive_split_render_reads_saved_buffer_input_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*render-vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    cycle_runtime_pane(&mut state.runtime)?;

    let buffer_b = install_scratch_test_buffer(&mut state, "*render-vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode_for_buffer(buffer_b, true), InputMode::Normal);
    assert_eq!(ui.input_mode_for_buffer(buffer_a, false), InputMode::Insert);
    Ok(())
}

#[test]
fn popup_terminal_focus_restores_its_own_vim_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let split_buffer = install_scratch_test_buffer(&mut state, "*popup-split*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let popup_buffer = install_terminal_popup_test_buffer(&mut state)?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(
        ui.input_mode_for_buffer(split_buffer, false),
        InputMode::Insert
    );

    let anchor = TextPoint::new(0, 0);
    shell_ui_mut(&mut state.runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(anchor));

    shell_ui_mut(&mut state.runtime)?.set_popup_focus(false);

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(
        ui.input_mode_for_buffer(popup_buffer, false),
        InputMode::Visual
    );

    shell_ui_mut(&mut state.runtime)?.set_popup_focus(true);

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(anchor));
    Ok(())
}

#[test]
fn visual_mode_is_buffer_local_across_buffer_switches() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*visual-a*")?;
    let anchor = TextPoint::new(0, 0);
    shell_ui_mut(&mut state.runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);

    let buffer_b = install_scratch_test_buffer(&mut state, "*visual-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(ui.vim().visual_anchor, None);

    focus_test_buffer(&mut state, buffer_a)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(anchor));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Character);
    Ok(())
}

#[test]
fn terminal_scroll_for_motion_maps_terminal_viewport_navigation() {
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::Down, None),
        Some(TerminalViewportScroll::LineDelta(-1))
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::Up, Some(3)),
        Some(TerminalViewportScroll::LineDelta(3))
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::FirstLine, Some(42)),
        Some(TerminalViewportScroll::Top)
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::LastLine, None),
        Some(TerminalViewportScroll::Bottom)
    );
    assert_eq!(terminal_scroll_for_motion(ShellMotion::Left, None), None);
}

#[test]
fn repeated_keydown_events_move_the_cursor() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(&mut state, "*repeat*", vec!["abcd".to_owned()])?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 3));

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Left),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: true,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(0, 2)
    );
    Ok(())
}

#[test]
fn undo_tree_root_cursor_tracks_last_root_cursor_across_undo_redo() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*undo-tree-root-redo*",
        vec!["alpha".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;

    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    buffer.record_undo_snapshot();

    assert!(buffer.undo_tree_undo());
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 5));

    buffer.set_cursor(TextPoint::new(0, 2));
    assert!(buffer.undo_tree_redo());
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha!"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 6));

    assert!(buffer.undo_tree_undo());
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 2));
    Ok(())
}

#[test]
fn undo_tree_select_restores_latest_root_cursor_without_changing_child_cursor() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*undo-tree-root-select*",
        vec!["alpha".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;

    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    buffer.record_undo_snapshot();

    assert!(buffer.undo_tree_undo());
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 5));

    buffer.set_cursor(TextPoint::new(0, 3));
    assert!(buffer.undo_tree_select(1));
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha!"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 6));

    assert!(buffer.undo_tree_select(0));
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 3));
    Ok(())
}

#[test]
fn undo_tree_picker_entries_use_fringe_indent_and_diff_preview() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*undo-tree-picker*", vec!["alpha".to_owned()])?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;

    buffer.set_cursor(TextPoint::new(0, 5));
    buffer.insert_text("!");
    buffer.record_undo_snapshot();

    let (entries, selected_index) = buffer.undo_tree_entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(selected_index, 1);
    assert!(!entries[0].label.starts_with(' '));
    assert!(!entries[1].label.starts_with(' '));
    assert!(entries[0].fringe.contains('*') || entries[0].fringe.contains('○'));
    assert!(entries[1].fringe.contains('├') || entries[1].fringe.contains('└'));
    let preview = entries[1]
        .preview
        .as_deref()
        .ok_or_else(|| "child preview missing".to_owned())?;
    assert!(
        preview.contains("-alpha") && preview.contains("+alpha!"),
        "preview should show parent→node diff, got {preview}"
    );
    Ok(())
}

#[test]
fn mouse_wheel_scrolls_the_buffer_under_the_pointer() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-scroll*",
        (0..20).map(|index| format!("line {index}")).collect(),
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;

    let handled = state
        .handle_event(
            Event::MouseWheel {
                timestamp: 0,
                window_id: 0,
                which: 0,
                x: 0.0,
                y: -1.0,
                direction: MouseWheelDirection::Normal,
                mouse_x: 24.0,
                mouse_y: 24.0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.scroll_row, MOUSE_WHEEL_SCROLL_LINES as usize);
    assert_eq!(buffer.cursor_row(), MOUSE_WHEEL_SCROLL_LINES as usize);
    Ok(())
}

#[test]
fn scroll_by_uses_wrapped_max_for_line_wrap() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*wrapped-scroll-max*",
        (0..30).map(|index| format!("line {index}")).collect(),
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.line_wrap = true;
        buffer.set_viewport_lines(8);
        buffer.set_scroll_layout(8, 40, 4);
        let expected = buffer.max_scroll_row_for_wrapped_rows(8, 40, 4);
        assert_eq!(buffer.max_scroll_row(), expected);
        assert!(buffer.max_scroll_row() < buffer.line_count().saturating_sub(1));
        buffer.scroll_row = buffer.max_scroll_row();
        assert_eq!(buffer.line_at_viewport_offset(7), 29);
    }
    Ok(())
}

#[test]
fn scroll_by_uses_content_viewport_rows_after_layout_sync() -> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*content-viewport-scroll*",
        (0..80).map(|index| format!("line {index}")).collect(),
    )?;
    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    assert!(buffer.content_viewport_lines < buffer.viewport_lines);
    assert_eq!(
        buffer.max_scroll_row(),
        buffer
            .line_count()
            .saturating_sub(buffer.content_viewport_lines)
    );
    buffer.scroll_row = buffer.max_scroll_row();
    assert_eq!(
        buffer.line_at_viewport_offset(buffer.content_viewport_lines.saturating_sub(1)),
        79
    );
    Ok(())
}

#[test]
fn mouse_drag_creates_a_character_visual_selection() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-drag*",
        vec!["alpha beta".to_owned(), "gamma delta".to_owned()],
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;
    let start = TextPoint::new(0, 1);
    let end = TextPoint::new(1, 3);
    let (start_x, start_y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        start,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;
    let (end_x, end_y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        end,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;

    state
        .handle_event(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: start_x,
                y: start_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseMotion {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mousestate: MouseState::from_sdl_state(0),
                x: end_x,
                y: end_y,
                xrel: end_x - start_x,
                yrel: end_y - start_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseButtonUp {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: end_x,
                y: end_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(start));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Character);
    assert_eq!(buffer.cursor_point(), end);
    assert_eq!(
        visual_selection(buffer, start, VisualSelectionKind::Character),
        Some(VisualSelection::Range(TextRange::new(
            start,
            buffer.point_after(end).unwrap_or(end)
        )))
    );
    assert!(state.mouse_drag.is_none());
    Ok(())
}

#[test]
fn mouse_double_click_selects_the_whole_line() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-double-click*",
        vec!["alpha beta".to_owned(), "gamma delta".to_owned()],
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;
    let point = TextPoint::new(1, 2);
    let (x, y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        point,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;

    state
        .handle_event(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 2,
                x,
                y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseButtonUp {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 2,
                x,
                y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(point));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(buffer.cursor_point(), point);
    assert_eq!(
        visual_selection(buffer, point, VisualSelectionKind::Line),
        buffer.line_span_range(1, 1).map(VisualSelection::Range)
    );
    assert!(state.mouse_drag.is_none());
    Ok(())
}

#[test]
fn terminal_mode_insert_hook_allows_reentering_insert_for_terminals() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_INSERT, HookEvent::new())
        .map_err(|error| error.to_string())?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn terminal_mode_normal_hook_uses_live_terminal_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.set_viewport_lines(2);
        buffer.replace_with_lines_follow_output(vec![
            "zero".to_owned(),
            "one".to_owned(),
            "two".to_owned(),
            "three456".to_owned(),
        ]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            8,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        3,
                        "two",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        8,
                        "three456",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                1,
                5,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 5));
    Ok(())
}

#[test]
fn terminal_popup_mode_normal_hook_uses_live_terminal_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_popup_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal popup test buffer missing".to_owned())?;
        buffer.set_viewport_lines(2);
        buffer.replace_with_lines_follow_output(vec![
            "zero".to_owned(),
            "one".to_owned(),
            "two".to_owned(),
            "three456".to_owned(),
        ]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            8,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        3,
                        "two",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        8,
                        "three456",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                1,
                4,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 4));
    Ok(())
}

#[test]
fn terminal_vim_edit_shortcuts_enter_insert_mode_instead_of_read_only_errors() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("substitute-char"),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn popup_terminal_event_context_prefers_popup_buffer_when_focused() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let pane_buffer = install_scratch_test_buffer(&mut state, "*popup-pane*")?;
    let popup_buffer = install_terminal_popup_test_buffer(&mut state)?;

    let context = active_buffer_event_context(&state.runtime)?;
    assert_eq!(context.buffer_id, popup_buffer);
    assert!(context.is_terminal);
    assert_ne!(context.buffer_id, pane_buffer);

    shell_ui_mut(&mut state.runtime)?.set_popup_focus(false);

    let context = active_buffer_event_context(&state.runtime)?;
    assert_eq!(context.buffer_id, pane_buffer);
    assert!(!context.is_terminal);
    Ok(())
}

#[test]
fn terminal_put_shortcuts_paste_yanks_in_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
    {
        let vim = shell_ui_mut(&mut state.runtime)?.vim_mut();
        vim.active_register = Some('a');
        vim.registers.insert(
            'a',
            YankRegister::Character("volt terminal paste".to_owned()),
        );
    }

    assert!(handle_terminal_vim_edit(
        &mut state.runtime,
        VimEditAction::PutAfter
    )?);
    assert!(terminal_buffer_state(&state.runtime)?.contains(buffer_id));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert_eq!(shell_ui(&state.runtime)?.vim().pending, None);
    Ok(())
}

#[test]
fn terminal_popup_bootstraps_session_and_enters_insert_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_popup_test_buffer(&mut state)?;

    let popup = state
        .runtime_popup()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;

    assert_eq!(popup.active_buffer, buffer_id);
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    assert!(terminal_buffer_state(&state.runtime)?.contains(buffer_id));
    Ok(())
}

#[test]
fn terminal_popup_command_focuses_the_popup_surface() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let pane_buffer = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("terminal.popup")
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.popup_buffer_id, Some(popup.active_buffer));
    assert_eq!(active_shell_buffer_id(&state.runtime)?, popup.active_buffer);
    assert_ne!(popup.active_buffer, pane_buffer);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(terminal_buffer_state(&state.runtime)?.contains(popup.active_buffer));
    Ok(())
}

#[test]
fn dismissed_popup_toggle_restores_terminal_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    state
        .runtime
        .execute_command("terminal.popup")
        .map_err(|error| error.to_string())?;

    let first_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;
    let terminal_buffer = first_popup.active_buffer;
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));

    state
        .runtime
        .execute_command("picker.toggle-popup-window")
        .map_err(|error| error.to_string())?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.popup_buffer_id, None);
    assert!(shell_buffer(&state.runtime, terminal_buffer).is_ok());
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));

    state
        .runtime
        .execute_command("picker.toggle-popup-window")
        .map_err(|error| error.to_string())?;

    let restored_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not restored".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(restored_popup.active_buffer, terminal_buffer);
    assert_eq!(ui.popup_buffer_id, Some(terminal_buffer));
    assert!(ui.popup_focus);
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));
    Ok(())
}

#[test]
fn browser_popup_command_focuses_the_popup_surface() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let pane_buffer = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("browser.open-popup")
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "browser popup was not opened".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.popup_buffer_id, Some(popup.active_buffer));
    assert_eq!(active_shell_buffer_id(&state.runtime)?, popup.active_buffer);
    assert_ne!(popup.active_buffer, pane_buffer);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(matches!(
        shell_buffer(&state.runtime, popup.active_buffer)?.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
    Ok(())
}

#[test]
fn workspace_dashboard_command_opens_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    state
        .runtime
        .execute_command("workspace.dashboard")
        .map_err(|error| error.to_string())?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "workspace dashboard picker did not open".to_owned())?;
    assert_eq!(picker.session.title(), "Worktrees");
    assert!(picker.session.item_count() > 0);
    Ok(())
}

#[test]
fn workspace_dashboard_command_opens_fallback_picker_outside_git() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-dashboard-non-git");
    open_workspace_from_project(&mut state.runtime, "non-git", &root)?;

    state
        .runtime
        .execute_command("workspace.dashboard")
        .map_err(|error| error.to_string())?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "workspace dashboard fallback picker did not open".to_owned())?;
    assert_eq!(picker.session.title(), "Worktrees");
    assert_eq!(picker.session.item_count(), 1);
    assert!(
        picker
            .session
            .selected()
            .is_some_and(|selected| selected.item().label() == "Workspace dashboard unavailable")
    );
    Ok(())
}

fn seed_worktree_remove_one_shot(runtime: &mut EditorRuntime, path: &Path) -> Result<(), String> {
    let path_text = path.display().to_string();
    shell_ui_mut(runtime)?.set_picker_one_shot(PickerOneShotContext::new(
        Some(PickerSelectedRow::new(
            path_text.clone(),
            "worktree",
            Some(path_text),
        )),
        Vec::new(),
    ));
    Ok(())
}

fn unique_sibling_path(anchor: &Path, label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    anchor.parent().unwrap_or(anchor).join(format!(
        "volt-shell-tests-{label}-{}-{unique}",
        std::process::id()
    ))
}

fn add_linked_worktree(main: &Path, label: &str, branch: &str) -> Result<PathBuf, String> {
    let worktree = unique_sibling_path(main, label);
    run_git_in_dir(main, &["branch", "-q", branch])?;
    let path_arg = worktree
        .to_str()
        .ok_or_else(|| format!("non-utf8 worktree path `{}`", worktree.display()))?;
    run_git_in_dir(main, &["worktree", "add", "-q", path_arg, branch])?;
    Ok(worktree)
}

fn wait_for_streamed_notification_title(
    state: &mut ShellState,
    needle: &str,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let visible = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
        if visible.iter().any(|entry| entry.title.contains(needle)) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "notification title containing `{needle}` never appeared"
    ))
}

#[test]
fn worktree_remove_missing_one_shot_is_silent_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let before = shell_ui(&state.runtime)?.notification_revision();

    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);
    Ok(())
}

#[test]
fn worktree_remove_create_affordance_one_shot_is_silent_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    shell_ui_mut(&mut state.runtime)?.set_picker_one_shot(PickerOneShotContext::new(
        Some(PickerSelectedRow::new(
            "git-worktree-dashboard:create",
            "+ new worktree",
            None::<&str>,
        )),
        Vec::new(),
    ));
    let before = shell_ui(&state.runtime)?.notification_revision();

    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);
    Ok(())
}

#[test]
fn worktree_remove_closes_matching_workspaces_streams_and_closes_on_success() -> Result<(), String>
{
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("worktree-remove-success-marks");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;

    let main = init_git_repo_with_commit("worktree-remove-success-main")?;
    let feature = add_linked_worktree(&main, "worktree-remove-success-feature", "feature-remove")?;
    let main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    let feature_ws = open_workspace_from_project(&mut state.runtime, "feature", &feature)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), feature_ws);

    state
        .runtime
        .execute_command("workspace.mark")
        .map_err(|error| error.to_string())?;
    let marks_before =
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?;
    assert!(!marks_before.trim().is_empty());

    seed_worktree_remove_one_shot(&mut state.runtime, &feature)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    assert!(
        find_workspace_by_root(&state.runtime, &feature)?.is_none(),
        "matching Project Workspace should close before git starts"
    );
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), main_ws);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed Worktree Remove popup was not opened".to_owned())?;
    let buffer_id = popup.active_buffer;
    assert!(shell_ui(&state.runtime)?.popup_focus);
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let contents = (0..buffer.line_count())
        .map(|line_index| buffer.text.line(line_index).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        contents.contains("git worktree remove") && contents.contains("--force"),
        "popup should show force remove command, got `{contents}`"
    );
    let feature_name = feature
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "feature worktree name".to_owned())?;
    assert!(
        contents.contains(feature_name),
        "popup should include worktree path, got `{contents}`"
    );

    wait_for_streamed_notification_title(&mut state, "Worktree Remove succeeded")?;
    wait_for_streamed_command_buffer_close(&mut state, buffer_id)?;
    assert!(
        !feature.exists(),
        "worktree path should be removed from disk"
    );
    assert_eq!(
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?,
        marks_before,
        "Mark List must stay untouched"
    );

    let branch_list = run_git_in_dir(&main, &["branch", "--list", "feature-remove"])?;
    assert!(
        branch_list.contains("feature-remove"),
        "Worktree Remove must not delete the branch"
    );

    let _ = std::fs::remove_dir_all(&main);
    let _ = std::fs::remove_dir_all(&state_dir);
    Ok(())
}

#[test]
fn worktree_remove_prunable_checkout_streams_prune_and_clears_registration() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("worktree-remove-prunable-main")?;
    let feature = add_linked_worktree(&main, "worktree-remove-prunable-feature", "feature-prune")?;
    let _main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    // Break the checkout so porcelain marks it prunable (matches stale `/w/...` trees).
    std::fs::remove_file(feature.join(".git")).map_err(|error| error.to_string())?;

    seed_worktree_remove_one_shot(&mut state.runtime, &feature)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed Worktree Remove popup was not opened".to_owned())?;
    let buffer_id = popup.active_buffer;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let contents = (0..buffer.line_count())
        .map(|line_index| buffer.text.line(line_index).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        contents.contains("git worktree prune"),
        "prunable worktree should prune, got `{contents}`"
    );

    wait_for_streamed_notification_title(&mut state, "Worktree Remove succeeded")?;
    wait_for_streamed_command_buffer_close(&mut state, buffer_id)?;
    assert!(
        !feature.exists(),
        "leftover prunable checkout path should be deleted"
    );
    let list = run_git_in_dir(&main, &["worktree", "list", "--porcelain"])?;
    assert!(
        !list.contains("feature-prune")
            && !list.contains(
                feature
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("feature-prune")
            ),
        "pruned worktree must not remain registered, got `{list}`"
    );

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn worktree_remove_failure_notifies_and_keeps_buffer_after_closing_workspaces() -> Result<(), String>
{
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("worktree-remove-fail-main")?;
    let main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    let default_workspace = shell_ui(&state.runtime)?.default_workspace();

    seed_worktree_remove_one_shot(&mut state.runtime, &main)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;

    assert!(
        find_workspace_by_root(&state.runtime, &main)?.is_none(),
        "Project Workspace should stay closed after git failure"
    );
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        default_workspace
    );
    assert_ne!(main_ws, default_workspace);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed Worktree Remove popup was not opened".to_owned())?;
    let buffer_id = popup.active_buffer;
    wait_for_streamed_notification_title(&mut state, "Worktree Remove failed")?;
    assert!(
        shell_ui(&state.runtime)?.buffer(buffer_id).is_some(),
        "failure must keep the streamed popup buffer"
    );
    assert!(
        !shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "worker should finish even when buffer is kept"
    );
    assert!(main.exists(), "main worktree should remain on disk");

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn worktree_remove_second_invocation_opens_distinct_streamed_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("worktree-remove-concurrent-main")?;
    let first = add_linked_worktree(&main, "worktree-remove-concurrent-a", "feature-a")?;
    let second = add_linked_worktree(&main, "worktree-remove-concurrent-b", "feature-b")?;
    open_workspace_from_project(&mut state.runtime, "main", &main)?;

    seed_worktree_remove_one_shot(&mut state.runtime, &first)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;
    let first_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "first Worktree Remove popup missing".to_owned())?;
    let first_buffer = first_popup.active_buffer;

    seed_worktree_remove_one_shot(&mut state.runtime, &second)?;
    state
        .runtime
        .execute_command("workspace.worktree-remove")
        .map_err(|error| error.to_string())?;
    let second_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "second Worktree Remove popup missing".to_owned())?;
    let second_buffer = second_popup.active_buffer;

    assert_ne!(first_buffer, second_buffer);
    assert!(
        shell_ui(&state.runtime)?.buffer(first_buffer).is_some()
            || shell_ui(&state.runtime)?
                .streamed_command_worker
                .contains(first_buffer),
        "first remove buffer should still exist or still be tracked"
    );
    assert!(
        shell_ui(&state.runtime)?.buffer(second_buffer).is_some()
            || shell_ui(&state.runtime)?
                .streamed_command_worker
                .contains(second_buffer),
        "second remove buffer should exist or be tracked"
    );

    wait_for_streamed_command_buffer_close(&mut state, first_buffer)?;
    wait_for_streamed_command_buffer_close(&mut state, second_buffer)?;

    let _ = std::fs::remove_dir_all(&main);
    let _ = std::fs::remove_dir_all(&first);
    let _ = std::fs::remove_dir_all(&second);
    Ok(())
}

fn open_workspace_dashboard(runtime: &mut EditorRuntime) -> Result<(), String> {
    runtime
        .execute_command("workspace.dashboard")
        .map_err(|error| error.to_string())?;
    shell_ui(runtime)?
        .picker()
        .ok_or_else(|| "workspace.dashboard did not open picker".to_owned())?;
    Ok(())
}

fn select_dashboard_row_matching_path(
    runtime: &mut EditorRuntime,
    path: &Path,
) -> Result<(), String> {
    let picker = shell_ui_mut(runtime)?
        .picker_mut()
        .ok_or_else(|| "dashboard picker missing".to_owned())?;
    let index = picker
        .session
        .matches()
        .iter()
        .position(|matched| project_roots_equal(Path::new(matched.item().id()), path))
        .ok_or_else(|| format!("dashboard missing worktree row for `{}`", path.display()))?;
    picker.session.set_selected_index(index);
    Ok(())
}

fn select_dashboard_create_row(runtime: &mut EditorRuntime) -> Result<(), String> {
    let picker = shell_ui_mut(runtime)?
        .picker_mut()
        .ok_or_else(|| "dashboard picker missing".to_owned())?;
    let index = picker
        .session
        .matches()
        .iter()
        .position(|matched| matched.item().id() == "git-worktree-dashboard:create")
        .ok_or_else(|| "dashboard missing `+ new worktree` row".to_owned())?;
    picker.session.set_selected_index(index);
    Ok(())
}

#[test]
fn workspace_dashboard_provider_extras_copy_ctrl_d_onto_instance() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-ctrl-d-extra-main")?;
    open_workspace_from_project(&mut state.runtime, "dashboard-ctrl-d-extra", &main)?;

    let overlay = picker::picker_overlay(&state.runtime, "workspace.dashboard")?;
    assert!(
        overlay.extra_keybinds().iter().any(|binding| {
            binding.chord() == "Ctrl+d" && binding.command_name() == "workspace.worktree-remove"
        }),
        "workspace.dashboard provider extras should land on the open picker instance"
    );

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn workspace_dashboard_ctrl_d_on_worktree_runs_remove_ux() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-ctrl-d-remove-main")?;
    let feature = add_linked_worktree(
        &main,
        "dashboard-ctrl-d-remove-feature",
        "feature-dash-remove",
    )?;
    let main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    let feature_ws = open_workspace_from_project(&mut state.runtime, "feature", &feature)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), feature_ws);

    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &feature)?;

    let handled = state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);
    assert!(
        shell_ui(&state.runtime)?.picker().is_none(),
        "Ctrl+d should close the Workspace Dashboard picker"
    );
    assert!(
        find_workspace_by_root(&state.runtime, &feature)?.is_none(),
        "matching Project Workspace should close"
    );
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), main_ws);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "streamed Worktree Remove popup was not opened".to_owned())?;
    let buffer_id = popup.active_buffer;
    assert!(shell_ui(&state.runtime)?.popup_focus);
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let contents = (0..buffer.line_count())
        .map(|line_index| buffer.text.line(line_index).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        contents.contains("git worktree remove") && contents.contains("--force"),
        "popup should show force remove command, got `{contents}`"
    );

    wait_for_streamed_notification_title(&mut state, "Worktree Remove succeeded")?;
    wait_for_streamed_command_buffer_close(&mut state, buffer_id)?;
    assert!(
        !feature.exists(),
        "worktree path should be removed from disk"
    );

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn workspace_dashboard_ctrl_d_on_create_row_is_silent_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-ctrl-d-create-noop-main")?;
    open_workspace_from_project(&mut state.runtime, "main", &main)?;

    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_create_row(&mut state.runtime)?;
    let before = shell_ui(&state.runtime)?.notification_revision();

    let handled = state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled, "Ctrl+d extra should still fire and close picker");
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);
    assert!(main.exists(), "primary worktree must stay on disk");

    let _ = std::fs::remove_dir_all(&main);
    Ok(())
}

#[test]
fn workspace_dashboard_ctrl_d_second_remove_while_first_runs() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-ctrl-d-concurrent-main")?;
    let first = add_linked_worktree(&main, "dashboard-ctrl-d-concurrent-a", "feature-a")?;
    let second = add_linked_worktree(&main, "dashboard-ctrl-d-concurrent-b", "feature-b")?;
    open_workspace_from_project(&mut state.runtime, "main", &main)?;

    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &first)?;
    state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    let first_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "first Worktree Remove popup missing".to_owned())?;
    let first_buffer = first_popup.active_buffer;

    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &second)?;
    state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    let second_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "second Worktree Remove popup missing".to_owned())?;
    let second_buffer = second_popup.active_buffer;

    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert_ne!(first_buffer, second_buffer);
    assert!(
        shell_ui(&state.runtime)?.buffer(first_buffer).is_some()
            || shell_ui(&state.runtime)?
                .streamed_command_worker
                .contains(first_buffer),
        "first remove buffer should still exist or still be tracked"
    );

    wait_for_streamed_command_buffer_close(&mut state, first_buffer)?;
    wait_for_streamed_command_buffer_close(&mut state, second_buffer)?;

    let _ = std::fs::remove_dir_all(&main);
    let _ = std::fs::remove_dir_all(&first);
    let _ = std::fs::remove_dir_all(&second);
    Ok(())
}

#[test]
fn workspace_dashboard_enter_still_switches_and_creates() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let main = init_git_repo_with_commit("dashboard-enter-unchanged-main")?;
    let feature = add_linked_worktree(&main, "dashboard-enter-unchanged-feature", "feature-enter")?;
    let main_ws = open_workspace_from_project(&mut state.runtime, "main", &main)?;
    let feature_ws = open_workspace_from_project(&mut state.runtime, "feature", &feature)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), feature_ws);

    // Switch: Enter on an already-open Worktree row.
    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &main)?;
    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), main_ws);

    // Open/create: Enter on a Worktree that is not yet a Project Workspace.
    let closed = add_linked_worktree(&main, "dashboard-enter-unchanged-closed", "feature-closed")?;
    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_row_matching_path(&mut state.runtime, &closed)?;
    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    let opened = find_workspace_by_root(&state.runtime, &closed)?
        .ok_or_else(|| "Enter should open Project Workspace for worktree".to_owned())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), opened);

    // Create affordance: Enter on `+ new worktree` still starts create flow.
    open_workspace_dashboard(&mut state.runtime)?;
    select_dashboard_create_row(&mut state.runtime)?;
    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .directory_state()
            .is_some(),
        "`+ new worktree` Enter should open oil directory"
    );
    assert_eq!(
        shell_ui(&state.runtime)?
            .picker()
            .map(|picker| picker.session.title().to_owned()),
        Some("Git Worktree Branch".to_owned()),
        "`+ new worktree` Enter should open the branch picker"
    );

    let _ = std::fs::remove_dir_all(&main);
    let _ = std::fs::remove_dir_all(&feature);
    let _ = std::fs::remove_dir_all(&closed);
    Ok(())
}

#[test]
fn split_runtime_pane_switches_focus_to_the_new_pane() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let original_pane_id = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .active_pane_id()
        .ok_or_else(|| "initial pane is missing".to_owned())?;

    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;

    let runtime_workspace = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let active_pane_id = runtime_workspace
        .active_pane_id()
        .ok_or_else(|| "split pane is missing".to_owned())?;
    assert_ne!(active_pane_id, original_pane_id);
    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(active_pane_id)
    );
    Ok(())
}

#[test]
fn pane_close_hook_closes_the_focused_split() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let initial_pane_id = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .active_pane_id()
        .ok_or_else(|| "initial pane is missing".to_owned())?;

    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Horizontal)?;
    cycle_runtime_pane(&mut state.runtime)?;
    state
        .runtime
        .emit_hook(HOOK_PANE_CLOSE, HookEvent::new())
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?
            .pane_count(),
        1
    );
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    assert_eq!(
        state
            .runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?
            .active_pane_id(),
        Some(initial_pane_id)
    );
    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(initial_pane_id)
    );
    Ok(())
}

#[test]
fn switch_split_hook_reverses_pane_order_and_preserves_the_active_pane() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;

    let (active_pane_id, before) = {
        let ui = shell_ui(&state.runtime)?;
        assert_eq!(ui.active_pane_index(), 0);
        let active_pane_id = ui
            .active_pane_id()
            .ok_or_else(|| "active pane is missing".to_owned())?;
        let before = ui
            .panes()
            .ok_or_else(|| "pane list is missing".to_owned())?
            .iter()
            .map(|pane| pane.buffer_id)
            .collect::<Vec<_>>();
        (active_pane_id, before)
    };

    state
        .runtime
        .emit_hook(HOOK_PANE_SWITCH_SPLIT, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let after = ui
        .panes()
        .ok_or_else(|| "pane list is missing after switch".to_owned())?
        .iter()
        .map(|pane| pane.buffer_id)
        .collect::<Vec<_>>();

    assert_eq!(after, before.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(ui.active_pane_id(), Some(active_pane_id));
    assert_eq!(ui.active_pane_index(), 1);
    Ok(())
}

#[test]
fn render_terminal_buffer_prefers_terminal_render_snapshot() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            12,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        11,
                        "echo hello",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                0,
                0,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Normal,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    let rendered_text = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(rendered_text.contains(&"echo hello"));
    assert!(
        !rendered_text
            .iter()
            .any(|text| text.contains("launching the configured shell"))
    );
    assert!(
        scene
            .iter()
            .any(|command| matches!(command, DrawCommand::FillRoundedRect { .. }))
    );
    Ok(())
}

#[test]
fn terminal_box_drawing_chars_render_as_strokes() -> Result<(), String> {
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let color = Color::RGB(200, 205, 210);

    draw_terminal_text_run(&mut target, 10, 20, "a│b", color, 8, 16)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        scene
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        vec!["a", "b"]
    );
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 21
                && rect.y == 20
                && rect.height == 16
                && *color == to_render_color(Color::RGB(200, 205, 210))
    )));
    Ok(())
}

#[test]
fn render_terminal_buffer_draws_visual_selection_highlight() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let selection_color = Color::RGBA(55, 71, 99, 255);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec!["echo hello".to_owned(), String::new()]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            12,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        10,
                        "echo hello",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![]),
            ],
            None,
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Visual,
            visual_selection: Some(VisualSelection::Range(TextRange::new(
                TextPoint::new(0, 0),
                TextPoint::new(0, 4),
            ))),
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: selection_color,
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. } if *color == to_render_color(selection_color)
    )));
    Ok(())
}

#[test]
fn render_terminal_buffer_keeps_terminal_content_opaque_with_window_opacity() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let terminal_background = editor_terminal::TerminalRgb {
        r: 24,
        g: 36,
        b: 48,
    };
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec!["echo hello".to_owned()]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            1,
            12,
            vec![editor_terminal::TerminalRenderLine::new(vec![
                editor_terminal::TerminalRenderRun::new(
                    0,
                    10,
                    "echo hello",
                    editor_terminal::TerminalRgb {
                        r: 215,
                        g: 221,
                        b: 232,
                    },
                    Some(terminal_background),
                    None,
                ),
            ])],
            None,
            None,
        ));
    }

    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Insert,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: Some(&registry),
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 12
                && rect.y == layout.body_y
                && rect.width == 80
                && rect.height == 16
                && *color == to_render_color(Color::RGB(24, 36, 48))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.y == layout.statusline_y - 6
                && rect.height == 1
                && color.a == 128
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { text, color, .. }
            if text == "echo hello" && color.a == 255
    )));
    Ok(())
}

#[test]
fn render_buffer_multicursor_draws_one_cursor_per_range() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*multicursor-render*",
        vec!["alpha alpha alpha".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.input = Some(InputField::new(">"));
        buffer.set_cursor(TextPoint::new(0, 6));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let cursor_color = to_render_color(Color::RGB(110, 170, 255));
    let multicursor = MulticursorState {
        match_text: "alpha".to_owned(),
        ranges: vec![
            TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
            TextRange::new(TextPoint::new(0, 6), TextPoint::new(0, 11)),
            TextRange::new(TextPoint::new(0, 12), TextPoint::new(0, 17)),
        ],
        primary: 1,
        cursor_offset: 0,
        visual_anchor_offset: None,
    };
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Insert,
                multicursor: Some(&multicursor),
                vim_targets_input: true,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: NullUserLibrary.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    let cursor_positions = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::FillRoundedRect { rect, color, .. }
                if *color == cursor_color && rect.y == layout.body_y =>
            {
                Some(rect.x)
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();

    let text_x = rect.x() + 12 + 8 + (5 * 8);
    assert_eq!(
        cursor_positions,
        [text_x, text_x + 6 * 8, text_x + 12 * 8]
            .into_iter()
            .collect()
    );
    Ok(())
}

#[test]
fn render_terminal_buffer_uses_buffer_cursor_in_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let cursor_color = Color::RGB(110, 170, 255);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec![
            "echo hello".to_owned(),
            "second line".to_owned(),
        ]);
        buffer.set_cursor(TextPoint::new(1, 2));
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            12,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        10,
                        "echo hello",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        11,
                        "second line",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                0,
                0,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let text_x = rect.x() + 12;
    let expected_x = text_x + 2 * 8;
    let expected_y = layout.body_y + 16;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Normal,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: cursor_color,
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == expected_x
                && rect.y == expected_y
                && *color == to_render_color(cursor_color)
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == text_x
                && rect.y == layout.body_y
                && *color == to_render_color(cursor_color)
    )));
    Ok(())
}

#[test]
fn render_terminal_buffer_uses_editor_insert_cursor_style() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let cursor_color = Color::RGB(110, 170, 255);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec!["echo hello".to_owned()]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            1,
            12,
            vec![editor_terminal::TerminalRenderLine::new(vec![
                editor_terminal::TerminalRenderRun::new(
                    0,
                    10,
                    "echo hello",
                    editor_terminal::TerminalRgb {
                        r: 215,
                        g: 221,
                        b: 232,
                    },
                    None,
                    None,
                ),
            ])],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                0,
                3,
                1,
                editor_terminal::TerminalCursorShape::Block,
                "o",
            )),
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let text_x = rect.x() + 12;
    let expected_x = text_x + 3 * 8;
    let expected_y = layout.body_y;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Insert,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: cursor_color,
            cursor_roundness: 4,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, radius }
            if rect.x == expected_x
                && rect.y == expected_y
                && rect.width == 2
                && rect.height == 16
                && *radius == 4
                && *color == to_render_color(cursor_color)
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == expected_x
                && rect.y == expected_y
                && rect.width == 2
                && rect.height == 16
                && *color == to_render_color(cursor_color)
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == expected_x
                && rect.y == expected_y
                && rect.width == 8
                && rect.height == 16
                && *color == to_render_color(cursor_color)
    )));
    Ok(())
}

#[test]
fn shell_start_does_not_construct_browser_web_context() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    assert!(
        !state.browser_host.has_live_web_context(),
        "shell start without a browser buffer must not construct WebContext"
    );
    Ok(())
}

#[test]
fn browser_host_open_devtools_event_is_ignored_without_a_live_webview() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    state
        .apply_browser_host_events(&[BrowserHostEvent::OpenDevtoolsRequested { buffer_id }])
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[test]
fn browser_devtools_shortcut_requested_recognizes_f12_and_ctrl_shift_i() {
    assert!(browser_devtools_shortcut_requested(
        Keycode::F12,
        Mod::NOMOD
    ));
    assert!(browser_devtools_shortcut_requested(
        Keycode::F12,
        shift_mod()
    ));
    assert!(browser_devtools_shortcut_requested(
        Keycode::I,
        ctrl_mod() | shift_mod()
    ));
}

#[test]
fn input_field_paste_shortcut_requested_recognizes_ctrl_shift_v_only() {
    assert!(input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod() | shift_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        shift_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod() | shift_mod() | Mod::LALTMOD
    ));
}

#[test]
fn browser_devtools_shortcut_requested_rejects_other_modifiers() {
    assert!(!browser_devtools_shortcut_requested(Keycode::I, ctrl_mod()));
    assert!(!browser_devtools_shortcut_requested(
        Keycode::I,
        ctrl_mod() | shift_mod() | Mod::LALTMOD
    ));
    assert!(!browser_devtools_shortcut_requested(
        Keycode::F11,
        Mod::NOMOD
    ));
}

#[test]
fn workspace_search_provider_extras_copy_ctrl_q_onto_instance() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-search-ctrl-q-extra");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "workspace-search-ctrl-q-extra", &root)?;

    let overlay = picker::picker_overlay(&state.runtime, "workspace.search")?;
    assert!(
        overlay.extra_keybinds().iter().any(|binding| {
            binding.chord() == "Ctrl+q" && binding.command_name() == "quickfix.open"
        }),
        "workspace.search provider extras should land on the open picker instance"
    );
    Ok(())
}

fn prepare_quickfix_workspace_search_picker(
    test_name: &str,
) -> Result<(ShellState, PathBuf, PathBuf, PathBuf), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir(test_name);
    let first = root.join("src").join("main.rs");
    let second = root.join("src").join("lib.rs");
    std::fs::create_dir_all(first.parent().ok_or_else(|| "missing src dir".to_owned())?)
        .map_err(|error| error.to_string())?;
    std::fs::write(&first, "fn alpha() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "fn beta() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, test_name, &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Workspace Search",
            vec![
                workspace_search::workspace_search_match_entry(
                    &root,
                    "src/main.rs",
                    1,
                    4,
                    "fn alpha() {}",
                ),
                workspace_search::workspace_search_match_entry(
                    &root,
                    "src/lib.rs",
                    1,
                    4,
                    "fn beta() {}",
                ),
            ],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new("Ctrl+q", "quickfix.open")]),
    );

    Ok((state, root, first, second))
}

#[test]
fn ctrl_q_with_workspace_search_picker_exports_quickfix_instead_of_quitting() -> Result<(), String>
{
    let (mut state, _root, _first, _second) =
        prepare_quickfix_workspace_search_picker("quickfix-ctrl-q-export")?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "Ctrl+Q did not open quickfix popup".to_owned())?;
    assert_eq!(
        shell_ui(&state.runtime)?.popup_buffer_id,
        Some(popup.active_buffer)
    );
    assert!(shell_ui(&state.runtime)?.popup_focus);
    let buffer = shell_buffer(&state.runtime, popup.active_buffer)?;
    assert!(buffer_is_quickfix(&buffer.kind));
    let first_line = buffer
        .text
        .line(0)
        .ok_or_else(|| "quickfix first line missing".to_owned())?;
    assert!(first_line.contains("main.rs:1:4 | fn alpha() {}"));
    assert!(first_line.contains("[ ] "));
    let second_line = buffer
        .text
        .line(1)
        .ok_or_else(|| "quickfix second line missing".to_owned())?;
    assert!(second_line.contains("lib.rs:1:4 | fn beta() {}"));
    assert!(second_line.contains("[ ] "));
    Ok(())
}

#[test]
fn ctrl_q_with_non_quickfix_picker_does_not_quit() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Buffers",
        vec![PickerEntry {
            item: PickerItem::new("buffer:alpha", "alpha", "scratch", None::<&str>),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    ));
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    Ok(())
}

#[test]
fn ctrl_q_without_quickfix_extra_does_not_export_popup_global() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("quickfix-ctrl-q-no-extra");
    let first = root.join("src").join("main.rs");
    std::fs::create_dir_all(first.parent().ok_or_else(|| "missing src dir".to_owned())?)
        .map_err(|error| error.to_string())?;
    std::fs::write(&first, "fn alpha() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "quickfix-ctrl-q-no-extra", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Search",
        vec![workspace_search::workspace_search_match_entry(
            &root,
            "src/main.rs",
            1,
            4,
            "fn alpha() {}",
        )],
    ));
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    Ok(())
}

#[test]
fn picker_extra_keybind_snapshots_context_closes_and_runs_command() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.picker-extra",
            "Consumes picker one-shot context",
            CommandSource::Core,
            |runtime| {
                let context = shell_ui_mut(runtime)?
                    .take_picker_one_shot()
                    .ok_or_else(|| "picker one-shot missing".to_owned())?;
                let selected = context
                    .selected()
                    .ok_or_else(|| "selected row missing".to_owned())?;
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push(format!(
                    "{}|{}|{}",
                    selected.id(),
                    selected.label(),
                    selected.path().unwrap_or("")
                ));
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;

    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Worktrees",
            vec![PickerEntry {
                item: PickerItem::new(
                    r"P:\repo\feature",
                    "feature",
                    "branch",
                    Some(r"P:\repo\feature"),
                ),
                action: PickerAction::NoOp,
                quickfix: None,
            }],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new(
            "Ctrl+d",
            "tests.picker-extra",
        )]),
    );

    let handled = state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(
        shell_ui_mut(&mut state.runtime)?
            .take_picker_one_shot()
            .is_none()
    );
    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec![r"P:\repo\feature|feature|P:\repo\feature".to_string()]
    );
    Ok(())
}

#[test]
fn popup_focus_ctrl_n_cycles_popup_buffers_instead_of_marked_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let first = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup-a*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    let second = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup-b*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![first, second], first)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(&state.runtime);
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.ensure_popup_buffer(first, "*popup-a*", BufferKind::Scratch, &*user_library);
        ui.ensure_popup_buffer(second, "*popup-b*", BufferKind::Scratch, &*user_library);
        ui.set_popup_buffer(first);
        ui.set_popup_focus(true);
        ui.enter_normal_mode();
    }

    let handled = state
        .try_runtime_keybinding(Keycode::N, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "popup missing after Ctrl+n".to_owned())?;
    assert_eq!(popup.active_buffer, second);
    assert_eq!(shell_ui(&state.runtime)?.popup_buffer_id, Some(second));
    Ok(())
}

#[test]
fn popup_focus_j_k_do_not_cycle_workspace_dock() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("popup-jk-dock-a");
    let second_root = unique_temp_dir("popup-jk-dock-b");
    let first = open_workspace_from_project(&mut state.runtime, "popup-jk-a", &first_root)?;
    let _second = open_workspace_from_project(&mut state.runtime, "popup-jk-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;

    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let popup_buffer = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![popup_buffer], popup_buffer)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(&state.runtime);
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.ensure_popup_buffer(popup_buffer, "*popup*", BufferKind::Scratch, &*user_library);
        ui.set_popup_buffer(popup_buffer);
        ui.set_popup_focus(true);
        ui.enter_normal_mode();
    }

    let modes = state
        .overlay_minor_modes()
        .map_err(|error| error.to_string())?;
    assert!(
        modes.contains(&KeymapScope::Popup),
        "popup focus must activate Popup Minor Mode: {modes:?}"
    );
    assert!(
        !modes.contains(&KeymapScope::WorkspaceDock),
        "popup focus must not activate Workspace Dock Minor Mode: {modes:?}"
    );
    for chord in ["j", "k"] {
        let overlay = state
            .runtime
            .keymaps()
            .find_in_scopes(&modes, KeymapVimMode::Normal, chord)
            .map(|binding| binding.command_name().to_owned());
        assert_ne!(
            overlay.as_deref(),
            Some("workspace.dock.next"),
            "popup {chord} must not fire workspace dock cycle"
        );
        assert_ne!(
            overlay.as_deref(),
            Some("workspace.dock.previous"),
            "popup {chord} must not fire workspace dock cycle"
        );
    }

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        first,
        "popup j must not cycle the workspace dock"
    );
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        first,
        "popup k must not cycle the workspace dock"
    );
    Ok(())
}

#[test]
fn picker_extra_keybind_falls_through_for_shared_popup_navigation() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Buffers",
            vec![
                PickerEntry {
                    item: PickerItem::new("a", "alpha", "scratch", None::<&str>),
                    action: PickerAction::NoOp,
                    quickfix: None,
                },
                PickerEntry {
                    item: PickerItem::new("b", "beta", "scratch", None::<&str>),
                    action: PickerAction::NoOp,
                    quickfix: None,
                },
            ],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new(
            "Ctrl+d",
            "tests.picker-extra",
        )]),
    );

    let handled = state
        .try_runtime_keybinding(Keycode::N, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    let selected = shell_ui(&state.runtime)?
        .picker()
        .and_then(|picker| picker.session().selected())
        .map(|matched| matched.item().id().to_owned());
    assert_eq!(selected.as_deref(), Some("b"));
    Ok(())
}

#[test]
#[ignore = "enable once quickfix picker export command lands"]
fn quickfix_picker_export_opens_popup_and_renders_workspace_search_results() -> Result<(), String> {
    let (state, root, first, second) =
        prepare_quickfix_workspace_search_picker("quickfix-export-popup")?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "picker missing before quickfix export".to_owned())?;
    assert_eq!(picker.session().matches().len(), 2);
    let _ = (first, second);
    let _ = active_runtime_popup(&state.runtime)?;
    let _ = shell_ui(&state.runtime)?.popup_buffer_id;
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = root;
    unimplemented!("invoke quickfix export and assert popup buffer renders exported rows");
}

#[test]
#[ignore = "enable once quickfix enter handler lands"]
fn quickfix_enter_opens_target_and_moves_focus_back_to_workspace() -> Result<(), String> {
    let (state, root, first, _) = prepare_quickfix_workspace_search_picker("quickfix-enter-focus")?;

    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let _ = (root, first);
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = original_buffer_id;
    unimplemented!(
        "export picker to quickfix, press Enter on quickfix row, assert workspace focus"
    );
}

#[test]
#[ignore = "enable once quickfix next and previous commands land"]
fn quickfix_next_previous_wraparound_tracks_current_list() -> Result<(), String> {
    let (state, root, first, second) =
        prepare_quickfix_workspace_search_picker("quickfix-wraparound")?;

    let _ = active_shell_buffer_id(&state.runtime)?;
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = (root, first, second);
    unimplemented!("export picker, drive quickfix.next/previous, assert wraparound navigation");
}

#[test]
#[ignore = "enable once quickfix export command lands"]
fn quickfix_export_from_unsupported_picker_is_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Buffers",
        vec![PickerEntry {
            item: PickerItem::new("buffer:alpha", "alpha", "scratch", None::<&str>),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    ));

    let _ = original_buffer_id;
    let _ = active_runtime_popup(&state.runtime)?;
    unimplemented!("invoke quickfix export and assert picker closes or no-ops without popup");
}

// ---------------------------------------------------------------------------
// LeaveOpen exit action
// ---------------------------------------------------------------------------

fn wait_for_streamed_command_worker_done(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        if !tracked {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "streamed command worker for buffer `{buffer_id}` did not finish in time"
    ))
}

#[test]
fn leave_open_keeps_popup_buffer_after_process_exits() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = open_streamed_command_popup(
        &mut state.runtime,
        StreamedCommandSpec {
            popup_title: "Leave Open Test".to_owned(),
            buffer_name: "*leave-open-test*".to_owned(),
            command_label: "true".to_owned(),
            #[cfg(unix)]
            program: "true".to_owned(),
            #[cfg(windows)]
            program: "cmd".to_owned(),
            #[cfg(unix)]
            args: vec![],
            #[cfg(windows)]
            args: vec!["/C".to_owned(), "exit 0".to_owned()],
            env: Vec::new(),
            cwd: std::env::temp_dir(),
            on_exit: StreamedCommandExitAction::LeaveOpen,
            notify_on_success: false,
            notify_on_failure: false,
        },
    )?;

    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;

    let ui = shell_ui(&state.runtime)?;
    assert!(
        ui.buffer(buffer_id).is_some(),
        "LeaveOpen: popup buffer must remain open after process exits"
    );
    assert!(
        !ui.streamed_command_worker.contains(buffer_id),
        "LeaveOpen: worker should be done"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Cancel flag: closing a popup buffer kills its worker
// ---------------------------------------------------------------------------

#[test]
fn closing_streamed_command_popup_kills_worker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = open_streamed_command_popup(
        &mut state.runtime,
        StreamedCommandSpec {
            popup_title: "Cancel Test".to_owned(),
            buffer_name: "*cancel-test*".to_owned(),
            command_label: "sleep".to_owned(),
            #[cfg(unix)]
            program: "sleep".to_owned(),
            #[cfg(windows)]
            program: "cmd".to_owned(),
            #[cfg(unix)]
            args: vec!["60".to_owned()],
            #[cfg(windows)]
            args: vec!["/C".to_owned(), "timeout /T 60 /NOBREAK".to_owned()],
            env: Vec::new(),
            cwd: std::env::temp_dir(),
            on_exit: StreamedCommandExitAction::LeaveOpen,
            notify_on_success: false,
            notify_on_failure: false,
        },
    )?;

    assert!(
        shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "worker should be active before close"
    );

    close_buffer_immediate(&mut state.runtime, buffer_id).map_err(|error| error.to_string())?;

    assert!(
        !shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "worker should be removed immediately after buffer close"
    );

    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// InputPromptOverlay
// ---------------------------------------------------------------------------

fn shell_echo_command(marker: &str) -> String {
    if cfg!(windows) {
        format!("Write-Output {marker}")
    } else {
        format!("printf '{marker}\\n'")
    }
}

fn shell_sleep_then_echo_command(seconds: u64, marker: &str) -> String {
    if cfg!(windows) {
        format!("Start-Sleep -Seconds {seconds}; Write-Output {marker}")
    } else {
        format!("sleep {seconds}; printf '{marker}\\n'")
    }
}

fn execute_shell_command(state: &mut ShellState, command: &str) -> Result<(), String> {
    state
        .runtime
        .execute_command(command)
        .map_err(|error| error.to_string())
}

fn active_input_prompt_text(state: &ShellState) -> Result<Option<String>, String> {
    Ok(shell_ui(&state.runtime)?
        .input_prompt()
        .map(|prompt| prompt.text().to_owned()))
}

fn confirm_input_prompt(state: &mut ShellState, text: &str) -> Result<(), String> {
    if !text.is_empty() {
        state
            .handle_text_input(text)
            .map_err(|error| error.to_string())?;
    }
    state
        .try_runtime_keybinding(Keycode::Return, Mod::empty())
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn start_workspace_compile(state: &mut ShellState, command: &str) -> Result<BufferId, String> {
    execute_shell_command(state, "workspace.compile")?;
    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "workspace.compile should open InputPromptOverlay"
    );
    confirm_input_prompt(state, command)?;
    active_runtime_popup(&state.runtime)?
        .map(|popup| popup.active_buffer)
        .ok_or_else(|| "compile confirmation did not open streamed popup".to_owned())
}

fn prompt_prefill_for_marker(
    tag: &str,
    marker_name: &str,
    marker_contents: &str,
) -> Result<String, String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir(tag);
    if !marker_name.is_empty() {
        std::fs::write(root.join(marker_name), marker_contents)
            .map_err(|error| error.to_string())?;
    }
    open_workspace_from_project(&mut state.runtime, tag, &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;
    let text = active_input_prompt_text(&state)?.unwrap_or_default();
    std::fs::remove_dir_all(&root).ok();
    Ok(text)
}

#[test]
fn input_prompt_overlay_confirm_delivers_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-confirm");
    open_workspace_from_project(&mut state.runtime, "input-prompt-confirm", &root)?;
    let marker = "volt-input-prompt-confirm";
    let popup_buffer = start_workspace_compile(&mut state, &shell_echo_command(marker))?;
    wait_for_streamed_command_output_line(&mut state, popup_buffer, marker)?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt should close after Enter with text"
    );
    assert!(
        shell_ui(&state.runtime)?
            .buffer(popup_buffer)
            .is_some_and(|buffer| buffer.text.text().contains(marker)),
        "confirmed prompt text should reach streamed compile command"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_escape_cancels() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-escape");
    open_workspace_from_project(&mut state.runtime, "input-prompt-escape", &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;

    state
        .try_runtime_keybinding(Keycode::Escape, Mod::empty())
        .map_err(|e| e.to_string())?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt should close on Escape"
    );
    assert!(
        active_runtime_popup(&state.runtime)?.is_none(),
        "Escape should discard the compile prompt without opening a popup"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_enter_with_empty_text_is_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-empty");
    open_workspace_from_project(&mut state.runtime, "input-prompt-empty", &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;

    state
        .try_runtime_keybinding(Keycode::Return, Mod::empty())
        .map_err(|e| e.to_string())?;

    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt must stay open when Enter pressed with empty text"
    );
    assert!(
        active_runtime_popup(&state.runtime)?.is_none(),
        "empty Enter should not open the compile popup"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_prefill_appears_in_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let overlay = InputPromptOverlay::new("test.prompt", "Build: ", "cargo build");
    shell_ui_mut(&mut state.runtime)?.open_input_prompt(overlay);
    assert_eq!(
        shell_ui(&state.runtime)?.input_prompt().map(|p| p.text()),
        Some("cargo build")
    );
    Ok(())
}

#[test]
fn render_shell_state_draws_input_prompt_overlay_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let overlay = InputPromptOverlay::new(COMPILE_PROMPT_ID, "Build command: ", "cargo build");
    shell_ui_mut(&mut state.runtime)?.open_input_prompt(overlay);

    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        &[],
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 640,
                height: 360,
            },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { text, .. } if text.contains("Build command: cargo build")
        )),
        "InputPromptOverlay must draw into the command-line footer row"
    );
    Ok(())
}

// ─── workspace.compile prompt tests ──────────────────────────────────────────

#[test]
fn workspace_compile_opens_input_prompt_overlay() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-prompt-opens");
    open_workspace_from_project(&mut state.runtime, "compile-prompt-opens", &root)?;

    execute_shell_command(&mut state, "workspace.compile")?;

    let prompt = shell_ui(&state.runtime)?.input_prompt();
    assert!(prompt.is_some(), "InputPromptOverlay should be open");
    assert_eq!(
        prompt.map(|p| p.id.as_str()),
        Some(COMPILE_PROMPT_ID),
        "overlay id must be COMPILE_PROMPT_ID"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_cargo_toml() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker(
            "compile-prompt-cargo",
            "Cargo.toml",
            "[package]\nname = \"test\"\n",
        )?,
        "cargo build"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_sln() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-sln", "App.sln", "")?,
        "dotnet build"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_csproj() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-csproj", "App.csproj", "")?,
        "dotnet build"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_package_json() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-package-json", "package.json", "{}")?,
        "npm run build"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_makefile() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-makefile", "Makefile", "")?,
        "make"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_empty_command_for_empty_directory() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-empty", "", "")?,
        ""
    );
    Ok(())
}

#[test]
fn workspace_compile_escape_does_not_store_command() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-escape");
    open_workspace_from_project(&mut state.runtime, "compile-escape", &root)?;

    execute_shell_command(&mut state, "workspace.compile")?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::empty())
        .map_err(|e| e.to_string())?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt should close on Escape"
    );
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    let stored = shell_ui(&state.runtime)?
        .compile_commands
        .get(&workspace_id)
        .cloned();
    assert!(stored.is_none(), "Escape must not store a command");
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_stored_command_over_detected() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-stored");
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n")
        .map_err(|e| e.to_string())?;
    open_workspace_from_project(&mut state.runtime, "compile-stored", &root)?;

    // Pre-store a custom command.
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    shell_ui_mut(&mut state.runtime)?
        .compile_commands
        .insert(workspace_id, "cargo build --release".to_owned());

    execute_shell_command(&mut state, "workspace.compile")?;

    let text = shell_ui(&state.runtime)?
        .input_prompt()
        .map(|p| p.text().to_owned())
        .unwrap_or_default();
    assert_eq!(
        text, "cargo build --release",
        "stored command should take priority over auto-detection"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

// ─── workspace.recompile tests ────────────────────────────────────────────────

#[test]
fn workspace_recompile_with_stored_command_does_not_open_input_prompt() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("recompile-stored");
    open_workspace_from_project(&mut state.runtime, "recompile-stored", &root)?;

    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    shell_ui_mut(&mut state.runtime)?
        .compile_commands
        .insert(workspace_id, shell_echo_command("recompile-ok"));

    execute_shell_command(&mut state, "workspace.recompile")?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "recompile with stored command must not open InputPromptOverlay"
    );
    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "recompile with stored command should open streamed popup".to_owned())?;
    wait_for_streamed_command_output_line(&mut state, popup.active_buffer, "recompile-ok")?;
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_recompile_without_stored_command_falls_back_to_compile_prompt() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("recompile-fallback");
    open_workspace_from_project(&mut state.runtime, "recompile-fallback", &root)?;

    execute_shell_command(&mut state, "workspace.recompile")?;

    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "recompile without stored command must open InputPromptOverlay"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_recompile_uses_workspace_scoped_stored_command() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root_a = unique_temp_dir("recompile-scope-a");
    let root_b = unique_temp_dir("recompile-scope-b");

    open_workspace_from_project(&mut state.runtime, "recompile-scope-a", &root_a)?;
    let workspace_a = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    shell_ui_mut(&mut state.runtime)?
        .compile_commands
        .insert(workspace_a, shell_echo_command("workspace-a"));

    open_workspace_from_project(&mut state.runtime, "recompile-scope-b", &root_b)?;
    let workspace_b = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    assert_ne!(workspace_a, workspace_b);

    execute_shell_command(&mut state, "workspace.recompile")?;

    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "recompile in workspace B must open prompt when only workspace A has a stored command"
    );
    std::fs::remove_dir_all(&root_a).ok();
    std::fs::remove_dir_all(&root_b).ok();
    Ok(())
}

#[test]
fn workspace_compile_confirm_reuses_existing_streamed_popup() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-reuse-popup");
    open_workspace_from_project(&mut state.runtime, "compile-reuse-popup", &root)?;

    let first_buffer = start_workspace_compile(&mut state, &shell_echo_command("compile-one"))?;
    wait_for_streamed_command_output_line(&mut state, first_buffer, "compile-one")?;
    wait_for_streamed_command_worker_done(&mut state, first_buffer)?;

    let second_buffer = start_workspace_compile(&mut state, &shell_echo_command("compile-two"))?;
    assert_eq!(
        first_buffer, second_buffer,
        "workspace.compile should reuse the existing streamed popup buffer"
    );
    wait_for_streamed_command_output_line(&mut state, second_buffer, "compile-two")?;

    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_compile_closing_popup_mid_build_stops_tracking_worker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-close-popup");
    open_workspace_from_project(&mut state.runtime, "compile-close-popup", &root)?;

    let buffer_id = start_workspace_compile(
        &mut state,
        &shell_sleep_then_echo_command(60, "compile-stop"),
    )?;
    assert!(
        shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "compile worker should be tracked before popup close"
    );

    close_buffer_immediate(&mut state.runtime, buffer_id).map_err(|error| error.to_string())?;
    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;
    assert!(
        !shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "closing compile popup should stop tracking the worker within the poll timeout"
    );

    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn lsp_stop_with_no_live_sessions_returns_error_without_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("lsp-stop-empty");
    open_workspace_from_project(&mut state.runtime, "lsp-stop-empty", &root)?;
    let path = root.join("main.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_file(&mut state.runtime, &path)?;

    let error = state
        .runtime
        .execute_command("lsp.stop")
        .expect_err("lsp.stop should fail when no Sessions are live");
    assert!(
        error
            .to_string()
            .contains("no running Language Server Sessions"),
        "unexpected error: {error}"
    );
    assert!(shell_ui(&state.runtime)?.picker().is_none());

    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn lsp_restart_with_no_live_sessions_returns_error_without_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let error = state
        .runtime
        .execute_command("lsp.restart")
        .expect_err("lsp.restart should fail when no Sessions are live");
    assert!(
        error
            .to_string()
            .contains("no running Language Server Sessions"),
        "unexpected error: {error}"
    );
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    Ok(())
}

#[test]
fn lsp_install_server_opens_recipe_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state
        .runtime
        .execute_command("lsp.install-server")
        .map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let picker = ui
        .picker()
        .ok_or_else(|| "install picker missing".to_owned())?;
    assert_eq!(picker.session().title(), "Install Language Server");
    assert!(picker.session().item_count() > 0);
    let selected = picker.session().selected().expect("one row");
    assert!(
        selected
            .item()
            .label()
            .contains("typescript-language-server")
            || selected.item().label().contains("rust-analyzer")
            || !selected.item().label().is_empty()
    );
    Ok(())
}

#[test]
fn dap_install_server_opens_recipe_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state
        .runtime
        .execute_command("dap.install-server")
        .map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let picker = ui
        .picker()
        .ok_or_else(|| "install picker missing".to_owned())?;
    assert_eq!(picker.session().title(), "Install Debug Adapter");
    assert!(picker.session().item_count() > 0);
    Ok(())
}

#[test]
fn install_picker_label_prefixes_status_icon() {
    let plus = tool_install::install_picker_label(false, "rust-analyzer");
    let check = tool_install::install_picker_label(true, "rust-analyzer");
    assert!(plus.ends_with(" rust-analyzer"));
    assert!(check.ends_with(" rust-analyzer"));
    assert_ne!(plus, check);
}

#[test]
fn lsp_install_unknown_id_returns_error() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let error = tool_install::install_language_server_by_id(&mut state.runtime, "not-a-server")
        .expect_err("unknown spec must fail");
    assert!(
        error.contains("not registered"),
        "unexpected error: {error}"
    );
    Ok(())
}

#[test]
fn lsp_session_lifecycle_picker_labels_sessions_and_wires_stop_action() {
    let root = {
        #[cfg(windows)]
        {
            PathBuf::from(r"p:\volt")
        }
        #[cfg(not(windows))]
        {
            PathBuf::from("/volt")
        }
    };
    let session = LspLiveSession::new("rust-analyzer", Some(root.clone()));
    let picker = lsp_session_lifecycle_picker_overlay(LspSessionPickerAction::Stop, &[session]);
    assert_eq!(picker.session().title(), "Stop Language Server Session");
    assert_eq!(picker.session().item_count(), 1);
    let selected = picker.session().selected().expect("one row");
    assert_eq!(
        selected.item().label(),
        format!("rust-analyzer — {}", root.display())
    );
    let action = picker
        .actions
        .get(selected.item().id())
        .expect("stop action");
    assert!(matches!(
        action,
        PickerAction::StopLspSession {
            server_id,
            root: action_root
        } if server_id == "rust-analyzer" && action_root.as_deref() == Some(root.as_path())
    ));
}

#[derive(Debug, Clone, Copy)]
struct WorkspaceDockTestUserLibrary {
    config: WorkspaceDockConfig,
}

impl UserLibrary for WorkspaceDockTestUserLibrary {
    fn workspace_dock_config(&self) -> WorkspaceDockConfig {
        self.config
    }
}

fn state_with_workspace_dock_config(config: WorkspaceDockConfig) -> Result<ShellState, String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(WorkspaceDockTestUserLibrary { config });
    ShellState::new_with_user_library(default_error_log_path(), false, user_library)
        .map_err(|error| error.to_string())
}

#[test]
fn workspace_dock_config_defaults_left_undocked() {
    let config = WorkspaceDockConfig::default();
    assert_eq!(config.side, WorkspaceDockSide::Left);
    assert!(!config.docked);
}

#[test]
fn workspace_dock_toggle_shows_and_hides_when_not_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: false,
    })?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_open());
    assert!(!workspace_dock_visible(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?
    ));

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(shell_ui(&state.runtime)?.workspace_dock_open());
    assert!(workspace_dock_visible(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?
    ));

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_open());
    Ok(())
}

#[test]
fn workspace_dock_docked_stays_visible_across_toggle() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    assert!(workspace_dock_visible(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?
    ));
    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(workspace_dock_visible(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?
    ));
    assert!(shell_ui(&state.runtime)?.workspace_dock_open());
    Ok(())
}

#[test]
fn workspace_dock_entries_include_default_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-dock-default");
    let project = open_workspace_from_project(&mut state.runtime, "dock-project", &root)?;
    let default_workspace = shell_ui(&state.runtime)?.default_workspace();
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    assert!(
        entries
            .iter()
            .any(|entry| entry.workspace_id == default_workspace)
    );
    assert!(entries.iter().any(|entry| entry.workspace_id == project));
    assert_eq!(entries[0].workspace_id, default_workspace);
    Ok(())
}

#[test]
fn workspace_dock_unread_badge_tracks_other_workspace_notifications() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-unread-a");
    let second_root = unique_temp_dir("workspace-dock-unread-b");
    let first = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    let now = Instant::now();
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "other-ws".to_owned(),
            severity: NotificationSeverity::Info,
            title: "Agent finished".to_owned(),
            body_lines: vec!["done".to_owned()],
            progress: None,
            active: true,
            action: None,
            workspace_id: Some(second),
        },
        now,
    );
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let second_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == second)
        .ok_or_else(|| "second workspace missing from dock".to_owned())?;
    assert!(second_entry.unread >= 1);
    let first_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == first)
        .ok_or_else(|| "first workspace missing from dock".to_owned())?;
    assert_eq!(first_entry.unread, 0);
    switch_runtime_workspace(&mut state.runtime, second)?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let second_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == second)
        .ok_or_else(|| "second workspace missing after switch".to_owned())?;
    assert_eq!(second_entry.unread, 0);
    Ok(())
}

#[test]
fn workspace_dock_highlight_tracks_active_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-highlight-a");
    let second_root = unique_temp_dir("workspace-dock-highlight-b");
    let first = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == first)
            .is_some_and(|entry| entry.active)
    );
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == second)
            .is_some_and(|entry| !entry.active)
    );

    state
        .runtime
        .execute_command("workspace.next")
        .map_err(|error| error.to_string())?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == second)
            .is_some_and(|entry| entry.active)
    );
    Ok(())
}

#[test]
fn workspace_dock_click_switches_workspace() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    let first_root = unique_temp_dir("workspace-dock-click-a");
    let second_root = unique_temp_dir("workspace-dock-click-b");
    let first = open_workspace_from_project(&mut state.runtime, "click-a", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "click-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;

    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let second_index = entries
        .iter()
        .position(|entry| entry.workspace_id == second)
        .ok_or_else(|| "missing second workspace in dock".to_owned())?;
    let cell_width = 8;
    let line_height = 16;
    let render_width = 800;
    let render_height = 600;
    let layout = workspace_dock_layout(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?,
        render_width,
        render_height,
        cell_width,
    );
    assert!(layout.visible);
    let card_height = workspace_dock_card_height(line_height) as i32;
    let click_x = layout.dock_rect.x + 8;
    let click_y = layout.dock_rect.y + second_index as i32 * card_height + 4;

    state
        .handle_event(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: click_x as f32,
                y: click_y as f32,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    Ok(())
}

#[test]
fn workspace_dock_layout_shrinks_content_for_left_dock() -> Result<(), String> {
    let state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    let layout = workspace_dock_layout(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?,
        800,
        600,
        8,
    );
    assert!(layout.visible);
    assert!(layout.dock_width > 0);
    assert_eq!(layout.content_x, layout.dock_width as i32);
    assert_eq!(layout.content_width, 800 - layout.dock_width);
    assert_eq!(layout.dock_rect.x, 0);
    Ok(())
}

#[test]
fn workspace_dock_render_marks_active_row() -> Result<(), String> {
    let state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &*shell_user_library(&state.runtime),
    )
    .map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        &entries,
        ShellChrome {
            user_library: &*shell_user_library(&state.runtime),
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 640,
                height: 360,
            },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;
    let layout = workspace_dock_layout(&*shell_user_library(&state.runtime), ui, 640, 360, 8);
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == layout.dock_rect.x && rect.width == layout.dock_rect.width
    )));
    assert!(entries.iter().any(|entry| entry.active));
    Ok(())
}

#[test]
fn workspace_dock_ctrl_h_enters_focus_from_panes_when_left_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_LEFT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(
        shell_ui(&state.runtime)?.workspace_dock_focus_active(&*shell_user_library(&state.runtime))
    );
    Ok(())
}

#[test]
fn workspace_dock_ctrl_l_exits_focus_back_to_panes_when_left_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    shell_ui_mut(&mut state.runtime)?.set_workspace_dock_focus(true);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_RIGHT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());
    Ok(())
}

#[test]
fn workspace_dock_h_j_cycles_workspaces_when_focused() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    let first_root = unique_temp_dir("workspace-dock-keys-a");
    let second_root = unique_temp_dir("workspace-dock-keys-b");
    let first = open_workspace_from_project(&mut state.runtime, "keys-a", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "keys-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    shell_ui_mut(&mut state.runtime)?.set_workspace_dock_focus(true);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_NEXT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_PREVIOUS, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), first);
    Ok(())
}

#[test]
fn workspace_dock_focus_j_k_cycle_workspaces() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-jk-a");
    let second_root = unique_temp_dir("workspace-dock-jk-b");
    let first = open_workspace_from_project(&mut state.runtime, "dock-jk-a", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "dock-jk-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_workspace_dock_focus(true);
        ui.enter_normal_mode();
    }

    let modes = state
        .overlay_minor_modes()
        .map_err(|error| error.to_string())?;
    assert!(
        modes.contains(&KeymapScope::WorkspaceDock),
        "dock focus must activate Workspace Dock Minor Mode: {modes:?}"
    );
    assert!(
        !modes.contains(&KeymapScope::Popup),
        "dock focus must not activate Popup Minor Mode: {modes:?}"
    );

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), first);
    Ok(())
}

#[test]
fn workspace_dock_ctrl_l_enters_focus_when_right_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Right,
        docked: true,
    })?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_RIGHT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(
        shell_ui(&state.runtime)?.workspace_dock_focus_active(&*shell_user_library(&state.runtime))
    );
    Ok(())
}

#[test]
fn debug_fringe_is_one_cell_when_idle_and_two_when_live() {
    assert_eq!(debug_fringe_cell_count(false), 1);
    assert_eq!(debug_fringe_cell_count(true), 2);
    assert_eq!(editor_fringe_width_px(8, false), 8);
    assert_eq!(editor_fringe_width_px(8, true), 16);
}

#[test]
fn toggle_breakpoint_without_session_shows_idle_fringe_marker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-idle-fringe");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("Program.cs");
    fs::write(&program, "class Program { static void Main() {} }\n").map_err(|e| e.to_string())?;
    let buffer_id = open_workspace_file(&mut state.runtime, &program)?;

    toggle_dap_breakpoint_at_cursor(&mut state.runtime)?;
    sync_active_buffer(&mut state.runtime)?;

    let focused = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        focused, buffer_id,
        "toggling a Breakpoint must not switch away from the editor buffer"
    );
    let focused_name = shell_ui(&state.runtime)?
        .buffer(focused)
        .ok_or_else(|| "focused buffer missing".to_owned())?
        .display_name()
        .to_owned();
    assert_ne!(
        focused_name, DAP_BREAKPOINTS_BUFFER_NAME,
        "toggle must not open `{DAP_BREAKPOINTS_BUFFER_NAME}`"
    );

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "buffer missing".to_owned())?;
    assert!(
        !buffer.dap_fringe_live(),
        "idle Workspace must keep one-cell fringe (no live Session)"
    );
    assert_eq!(
        buffer.dap_fringe_marker(0),
        Some(BreakpointState::Pending),
        "Breakpoint must appear in Debug Fringe before a Session starts"
    );

    let workspace_id = workspace.get();
    let listed = state
        .runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .ok_or_else(|| "dap manager missing".to_owned())?
        .list_breakpoints(workspace_id)
        .map_err(|e| e.to_string())?;
    assert_eq!(listed.len(), 1);
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn wrap_columns_shrink_when_debug_fringe_widens() {
    let idle = wrap_columns_for_width_with_fringe(320, 8, 1);
    let live = wrap_columns_for_width_with_fringe(320, 8, 2);
    assert!(live < idle);
}

fn install_fake_tcp_dap_manager(
    runtime: &mut EditorRuntime,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    use editor_dap::{DebugAdapterRegistry, DebugAdapterSpec, DebugAdapterTransport};
    use std::io::{BufRead, Read, Write};
    use std::net::TcpListener;

    fn write_raw(writer: &mut impl Write, body: &str) {
        write!(writer, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("write");
        writer.flush().expect("flush");
    }

    fn read_body(reader: &mut impl BufRead) -> Result<String, String> {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            let read = reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if read == 0 {
                return Err("adapter closed".to_owned());
            }
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                break;
            }
            let Some((key, value)) = trimmed.split_once(':') else {
                continue;
            };
            if key.eq_ignore_ascii_case("Content-Length") {
                content_length = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?);
            }
        }
        let len = content_length.ok_or_else(|| "missing Content-Length".to_owned())?;
        let mut buf = vec![0_u8; len];
        reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
        String::from_utf8(buf).map_err(|e| e.to_string())
    }

    fn extract_field(body: &str, key: &str) -> Option<String> {
        let needle = format!("\"{key}\"");
        let start = body.find(&needle)?;
        let after = &body[start + needle.len()..];
        let after = after.trim_start_matches([' ', ':', '\t']);
        if let Some(rest) = after.strip_prefix('"') {
            let end = rest.find('"')?;
            return Some(rest[..end].to_owned());
        }
        let end = after
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after.len());
        Some(after[..end].to_owned())
    }

    fn fake_adapter_loop(reader: impl Read, mut writer: impl Write) {
        let mut reader = std::io::BufReader::new(reader);
        let mut seq = 1_u64;
        let mut stopped_line = 1_u64;
        let mut program_path = "main.rs".to_owned();
        while let Ok(body) = read_body(&mut reader) {
            let command = extract_field(&body, "command").unwrap_or_default();
            let request_seq = extract_field(&body, "seq").unwrap_or_else(|| "0".to_owned());
            match command.as_str() {
                "initialize" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"initialize","body":{{"supportsConfigurationDoneRequest":true,"supportTerminateDebuggee":true,"supportsRestartRequest":true}}}}"#
                        ),
                    );
                    seq += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"initialized","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "configurationDone" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"configurationDone","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "launch" | "attach" => {
                    if let Some(program) = extract_field(&body, "program") {
                        program_path = program;
                    }
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"{command}","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line = 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"entry","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "setBreakpoints" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"setBreakpoints","body":{{"breakpoints":[]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "continue" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"continue"}}"#
                        ),
                    );
                    seq += 1;
                    if program_path.contains("exit-on-continue") {
                        write_raw(
                            &mut writer,
                            &format!(
                                r#"{{"seq":{seq},"type":"event","event":"exited","body":{{"exitCode":0}}}}"#
                            ),
                        );
                        seq += 1;
                        write_raw(
                            &mut writer,
                            &format!(
                                r#"{{"seq":{seq},"type":"event","event":"terminated","body":{{}}}}"#
                            ),
                        );
                        break;
                    }
                }
                "next" | "stepIn" | "stepOut" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"{command}","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"step","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "pause" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"pause","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"pause","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "restart" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"restart","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line = 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"entry","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "stackTrace" => {
                    let path_json = program_path.replace('\\', "\\\\");
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"stackTrace","body":{{"stackFrames":[{{"id":1,"name":"main","source":{{"path":"{path_json}"}},"line":{stopped_line},"column":1}},{{"id":2,"name":"caller","source":{{"path":"{path_json}"}},"line":{},"column":1}}],"totalFrames":2}}}}"#,
                            stopped_line + 10
                        ),
                    );
                    seq += 1;
                }
                "threads" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"threads","body":{{"threads":[{{"id":1,"name":"main"}},{{"id":2,"name":"worker"}}]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "evaluate" => {
                    let expression = extract_field(&body, "expression").unwrap_or_default();
                    let frame_id =
                        extract_field(&body, "frameId").unwrap_or_else(|| "1".to_owned());
                    let eval_body = if expression == "person" {
                        r#"{"result":"Person { ... }","type":"Person","variablesReference":2}"#
                            .to_owned()
                    } else {
                        format!(
                            r#"{{"result":"{expression}@{frame_id}={stopped_line}","variablesReference":0}}"#
                        )
                    };
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"evaluate","body":{eval_body}}}"#
                        ),
                    );
                    seq += 1;
                }
                "scopes" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"scopes","body":{{"scopes":[{{"name":"Locals","variablesReference":1,"expensive":false}}]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "variables" => {
                    let reference = extract_field(&body, "variablesReference")
                        .unwrap_or_else(|| "1".to_owned());
                    let variables = match reference.as_str() {
                        "2" => {
                            r#"[{"name":"Name","value":"\"Ada\"","type":"string","variablesReference":0},{"name":"Address","value":"Address { ... }","type":"Address","variablesReference":3}]"#
                        }
                        "3" => {
                            r#"[{"name":"City","value":"\"London\"","type":"string","variablesReference":0}]"#
                        }
                        _ => {
                            r#"[{"name":"x","value":"42","type":"i32","variablesReference":0},{"name":"person","value":"Person { ... }","type":"Person","variablesReference":2}]"#
                        }
                    };
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"variables","body":{{"variables":{variables}}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "disconnect" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"disconnect","body":{{}}}}"#
                        ),
                    );
                    break;
                }
                _ => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":false,"command":"{command}","message":"unsupported"}}"#
                        ),
                    );
                    seq += 1;
                }
            }
        }
    }

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut sessions = 0_u8;
        while sessions < 4 && Instant::now() < deadline {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let reader = stream.try_clone().expect("clone");
                    fake_adapter_loop(reader, stream);
                    sessions += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(_) => break,
            }
        }
    });

    let mut registry = DebugAdapterRegistry::new();
    registry
        .register(
            DebugAdapterSpec::new("fake-dap", "rust", ["rs"], "", [] as [&str; 0])
                .with_transport(DebugAdapterTransport::Tcp {
                    host: "127.0.0.1".to_owned(),
                    port,
                })
                .with_preference(10),
        )
        .map_err(|e| e.to_string())?;
    runtime
        .services_mut()
        .insert(Arc::new(DapClientManager::new(registry)));
    Ok((port, handle))
}

#[test]
fn debug_layout_installs_three_panes_and_disables_golden_ratio() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    install_debug_layout(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert!(ui.is_debug_layout_active());
    assert_eq!(view.golden_ratio_override, Some(false));
    assert_eq!(
        view.pane_size_weights.as_deref(),
        Some(
            [
                DEBUG_LAYOUT_BREAKPOINTS_WEIGHT,
                DEBUG_LAYOUT_EDITOR_WEIGHT,
                DEBUG_LAYOUT_LOCALS_WEIGHT
            ]
            .as_slice()
        )
    );
    assert_eq!(view.panes.len(), 3);
    assert_eq!(view.split_direction, Some(PaneSplitDirection::Vertical));
    let left = ui
        .buffer(view.panes[0].buffer_id)
        .ok_or_else(|| "breakpoints pane missing".to_owned())?;
    let right = ui
        .buffer(view.panes[2].buffer_id)
        .ok_or_else(|| "locals pane missing".to_owned())?;
    assert!(matches!(
        &left.kind,
        BufferKind::Plugin(kind) if kind == DAP_BREAKPOINTS_KIND
    ));
    assert!(matches!(
        &right.kind,
        BufferKind::Plugin(kind) if kind == DAP_LOCALS_KIND
    ));
    assert!(
        right.plugin_section_state.is_some(),
        "locals pane should expose Locals/Expressions sections"
    );
    let rects = workspace_pane_rects(&*shell_user_library(&state.runtime), ui, 600, 200, 3);
    assert_eq!(rects.len(), 3);
    assert!(
        rects[0].width < rects[1].width && rects[2].width < rects[1].width,
        "editor pane should be widest: {rects:?}"
    );
    Ok(())
}

#[test]
fn debug_layout_blocks_user_splits() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    install_debug_layout(&mut state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 3);
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    assert_eq!(
        shell_ui(&state.runtime)?.pane_count(),
        3,
        "user splits must be blocked while Debug Layout is active"
    );
    Ok(())
}

#[test]
fn debug_layout_teardown_restores_golden_ratio() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    assert!(
        shell_user_library(&state.runtime)
            .pane_config()
            .golden_ratio
    );
    install_debug_layout(&mut state.runtime)?;
    teardown_debug_layout(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert!(!ui.is_debug_layout_active());
    assert_eq!(view.golden_ratio_override, None);
    assert!(view.pane_size_weights.is_none());
    assert_eq!(view.panes.len(), 1);
    Ok(())
}

#[test]
fn dap_start_installs_debug_layout_and_stop_restores() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-layout-start-stop");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 3);

    stop_dap_for_active_workspace(&mut state.runtime)?;
    assert!(!shell_ui(&state.runtime)?.is_debug_layout_active());
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_start_saves_dirty_workspace_files_before_launch() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-start-saves-dirty");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let buffer_id = open_workspace_file(&mut state.runtime, &program)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// dirty\n");
        assert!(buffer.is_dirty());
    }
    assert_eq!(
        fs::read_to_string(&program).map_err(|e| e.to_string())?,
        "fn main() {}\n",
        "disk must stay stale until dap.start"
    );

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;

    assert_eq!(
        fs::read_to_string(&program).map_err(|e| e.to_string())?,
        "// dirty\nfn main() {}\n",
        "dap.start must save dirty Workspace files before compile-before-debug / launch"
    );
    assert!(
        !shell_buffer(&state.runtime, buffer_id)?.is_dirty(),
        "saved buffer must clear dirty"
    );
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_stopped_jumps_to_source_refreshes_locals_and_steps() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-step-jump-locals");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(
        &program,
        "fn main() {\n    let x = 1;\n    let y = 2;\n    let z = 3;\n}\n",
    )
    .map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    toggle_dap_breakpoint_at_cursor(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    {
        let dap = state
            .runtime
            .services()
            .get::<Arc<DapClientManager>>()
            .ok_or_else(|| "dap manager missing".to_owned())?;
        assert!(
            dap.session_info(workspace_id.get())
                .map_err(|e| e.to_string())?
                .is_some(),
            "session must be live after start"
        );
    }

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DAP stopped UI".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }

    let ui = shell_ui(&state.runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?;
    assert_eq!(view.active_pane, 1, "center editor pane should be focused");
    let editor = ui
        .buffer(view.panes[1].buffer_id)
        .ok_or_else(|| "center editor missing".to_owned())?;
    assert_eq!(editor.cursor_row(), 0, "should jump to stop line 1");
    assert!(
        editor.dap_fringe_live(),
        "live Session must widen Debug Fringe in the center pane"
    );
    assert_eq!(editor.dap_execution_line(), Some(0));
    assert!(
        editor.dap_fringe_marker(0).is_some(),
        "Breakpoint on the stopped line must stay in the Debug Fringe"
    );
    let locals = ui
        .buffer(view.panes[2].buffer_id)
        .ok_or_else(|| "locals pane missing".to_owned())?;
    let locals_text = locals.text.text();
    assert!(
        locals_text.contains("x: 42"),
        "locals should refresh on stop: {locals_text}"
    );
    assert!(
        locals_text.contains(&format!("{DAP_VAR_COLLAPSED_GLYPH} person:")),
        "structured Locals must show a collapsed chevron: {locals_text}"
    );
    let breakpoints = ui
        .buffer(view.panes[0].buffer_id)
        .ok_or_else(|| "breakpoints pane missing".to_owned())?;
    let bp_text = breakpoints.text.text();
    assert!(
        bp_text.contains("main.rs:1"),
        "Breakpoints pane should list the source line: {bp_text}"
    );
    assert!(
        !bp_text.contains("Breakpoints:"),
        "Breakpoints pane should not repeat the title as a header: {bp_text}"
    );

    dap_control_for_active_workspace(&mut state.runtime, DapControl::StepOver)?;
    {
        let ui = shell_ui(&state.runtime)?;
        let editor = ui
            .buffer(
                ui.workspace_view()
                    .ok_or_else(|| "view missing".to_owned())?
                    .panes[1]
                    .buffer_id,
            )
            .ok_or_else(|| "center editor missing".to_owned())?;
        assert_eq!(
            editor.dap_execution_line(),
            None,
            "step must clear the execution highlight until the next stop"
        );
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for step stop".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }
    let ui = shell_ui(&state.runtime)?;
    let editor = ui
        .buffer(
            ui.workspace_view()
                .ok_or_else(|| "view missing".to_owned())?
                .panes[1]
                .buffer_id,
        )
        .ok_or_else(|| "center editor missing".to_owned())?;
    assert_eq!(editor.cursor_row(), 1);
    assert_eq!(editor.dap_execution_line(), Some(1));

    restart_dap_for_active_workspace(&mut state.runtime)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for restart stop".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }
    let ui = shell_ui(&state.runtime)?;
    let editor = ui
        .buffer(
            ui.workspace_view()
                .ok_or_else(|| "view missing".to_owned())?
                .panes[1]
                .buffer_id,
        )
        .ok_or_else(|| "center editor missing".to_owned())?;
    assert_eq!(editor.cursor_row(), 0);
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_locals_and_watches_expand_structured_variables() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-expand-vars");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {\n    let person = Person;\n}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DAP stopped UI".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }

    focus_debug_layout_pane(&mut state.runtime, 2)?;
    {
        let buffer = active_shell_buffer_mut(&mut state.runtime)?;
        buffer.plugin_focus_section_named(DAP_LOCALS_SECTION);
        let line = (0..buffer.line_count())
            .find(|&index| {
                buffer
                    .text
                    .line(index)
                    .is_some_and(|line| line.contains("person:"))
            })
            .ok_or_else(|| "person Locals row missing".to_owned())?;
        buffer.set_cursor(TextPoint::new(line, 0));
    }
    state
        .runtime
        .execute_command("dap.toggle-variable")
        .map_err(|error| error.to_string())?;

    let locals_id = shell_ui(&state.runtime)?
        .workspace_view()
        .ok_or_else(|| "workspace view missing".to_owned())?
        .panes[2]
        .buffer_id;
    let locals_text = {
        let locals = shell_buffer(&state.runtime, locals_id)?;
        plugin_section_lines(locals, DAP_LOCALS_SECTION)?.join("\n")
    };
    assert!(
        locals_text.contains(DAP_WATCHES_HEADER),
        "Locals section must keep Watch Expressions header: {locals_text}"
    );
    assert!(
        locals_text.contains(&format!("{DAP_VAR_EXPANDED_GLYPH} person:")),
        "person should expand: {locals_text}"
    );
    assert!(
        locals_text.contains("Name:") && locals_text.contains("Address:"),
        "expanded person must show members: {locals_text}"
    );

    add_dap_expression(&mut state.runtime, "person")?;
    {
        let buffer = active_shell_buffer_mut(&mut state.runtime)?;
        buffer.plugin_focus_section_named(DAP_EXPRESSIONS_SECTION);
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    state
        .runtime
        .execute_command("dap.toggle-variable")
        .map_err(|error| error.to_string())?;
    let watch_text = {
        let locals = shell_buffer(&state.runtime, locals_id)?;
        plugin_section_lines(locals, DAP_EXPRESSIONS_SECTION)?.join("\n")
    };
    assert!(
        watch_text.contains(&format!("{DAP_VAR_EXPANDED_GLYPH} person:")),
        "Watch Expression should expand: {watch_text}"
    );
    assert!(
        watch_text.contains("Name:"),
        "expanded watch must show members: {watch_text}"
    );

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_locals_insert_watch_expression_evaluates_while_stopped() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-insert-watch");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {\n    let x = 1;\n}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DAP stopped UI".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }

    focus_debug_layout_pane(&mut state.runtime, 2)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    {
        let buffer = active_shell_buffer_mut(&mut state.runtime)?;
        buffer.plugin_focus_section_named(DAP_LOCALS_SECTION);
        let mut lines: Vec<String> = (0..buffer.line_count())
            .filter_map(|index| buffer.text.line(index))
            .collect();
        let header = lines
            .iter()
            .position(|line| line == DAP_WATCHES_HEADER)
            .ok_or_else(|| format!("Watch Expressions header missing: {lines:?}"))?;
        lines.insert(header + 1, "x".to_owned());
        buffer.replace_with_lines_preserve_view(lines);
    }
    apply_dap_locals_edits(&mut state.runtime, buffer_id)?;

    let locals_text = {
        let locals = shell_buffer(&state.runtime, buffer_id)?;
        plugin_section_lines(locals, DAP_LOCALS_SECTION)?.join("\n")
    };
    assert!(
        locals_text.contains(DAP_WATCHES_HEADER),
        "header must remain: {locals_text}"
    );
    assert!(
        locals_text.lines().any(|line| line.contains("x@")),
        "inserted Watch Expression must evaluate while stopped: {locals_text}"
    );
    let watch_text = {
        let locals = shell_buffer(&state.runtime, buffer_id)?;
        plugin_section_lines(locals, DAP_EXPRESSIONS_SECTION)?.join("\n")
    };
    assert!(
        watch_text.contains("x@"),
        "Expressions section must mirror the new watch: {watch_text}"
    );

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_continue_to_exit_tears_down_debug_layout() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-continue-exit");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("exit-on-continue.rs");
    fs::write(&program, "fn main() {\n    let x = 1;\n}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DAP stopped UI".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());

    dap_control_for_active_workspace(&mut state.runtime, DapControl::Continue)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let _ = refresh_pending_dap(&mut state.runtime)?;
        if !shell_ui(&state.runtime)?.is_debug_layout_active() {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for Debug Stop cleanup after continue".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    {
        let dap = state
            .runtime
            .services()
            .get::<Arc<DapClientManager>>()
            .ok_or_else(|| "dap manager missing".to_owned())?;
        assert!(
            dap.session_info(workspace_id.get())
                .map_err(|e| e.to_string())?
                .is_none(),
            "Session must end after process exit"
        );
    }
    let ui = shell_ui(&state.runtime)?;
    let editor = ui
        .buffer(
            ui.workspace_view()
                .ok_or_else(|| "view missing".to_owned())?
                .panes[0]
                .buffer_id,
        )
        .ok_or_else(|| "editor missing".to_owned())?;
    assert!(
        !editor.dap_fringe_live(),
        "Debug Fringe must not stay live after Debug Stop cleanup"
    );

    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_mode_function_keys_continue_and_step() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let idle_modes = state.overlay_minor_modes().map_err(|e| e.to_string())?;
    assert!(
        !idle_modes.contains(&KeymapScope::Dap),
        "DAP Mode must stay off without a Session"
    );
    let start = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&idle_modes, KeymapVimMode::Any, "F5")
        .ok_or_else(|| "expected Global F5".to_owned())?;
    assert_eq!(start.command_name(), "dap.start");

    let root = unique_temp_dir("dap-mode-keys");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;
    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;

    let live_modes = state.overlay_minor_modes().map_err(|e| e.to_string())?;
    assert!(
        live_modes.contains(&KeymapScope::Dap),
        "live Session must activate DAP Mode: {live_modes:?}"
    );
    let continue_binding = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&live_modes, KeymapVimMode::Any, "F5")
        .ok_or_else(|| "expected DAP F5".to_owned())?;
    assert_eq!(continue_binding.command_name(), "dap.continue");
    let step = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&live_modes, KeymapVimMode::Any, "F10")
        .ok_or_else(|| "expected DAP F10".to_owned())?;
    assert_eq!(step.command_name(), "dap.step");
    let into = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&live_modes, KeymapVimMode::Any, "F11")
        .ok_or_else(|| "expected DAP F11".to_owned())?;
    assert_eq!(into.command_name(), "dap.step-into");

    let toggle = state
        .runtime
        .keymaps()
        .resolve_with_minor_modes(&[KeymapScope::Workspace], KeymapVimMode::Any, "Space d a")
        .ok_or_else(|| "expected <leader> da".to_owned())?;
    assert_eq!(toggle.command_name(), "dap.toggle-breakpoint");

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_watches_eval_repl_switch_and_breakpoint_extras() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-polish");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() { let x = 1; }\n").map_err(|e| e.to_string())?;
    let buffer_id = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changed = refresh_pending_dap(&mut state.runtime)?;
        if changed {
            break;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for stop".to_owned());
        }
        thread::sleep(Duration::from_millis(5));
    }

    add_dap_expression(&mut state.runtime, "x")?;
    let workspace_id = workspace.get();
    let (locals, expressions) = dap_locals_and_expression_lines(&state.runtime, workspace_id)?;
    assert!(
        locals.iter().any(|line| line == DAP_WATCHES_HEADER),
        "locals must keep Watch Expressions header: {locals:?}"
    );
    assert!(
        locals.iter().any(|line| line.contains("x")),
        "locals rows: {locals:?}"
    );
    assert!(
        expressions.iter().any(|line| line.contains("x:")),
        "expression rows: {expressions:?}"
    );

    show_dap_eval_result(&mut state.runtime, "y", DapEvaluateContext::Repl)?;
    open_dap_repl(&mut state.runtime)?;
    submit_dap_repl_expression(&mut state.runtime, "z")?;
    assert!(
        shell_ui(&state.runtime)?
            .input_prompt()
            .is_some_and(|prompt| prompt.id == DAP_REPL_PROMPT_ID),
        "REPL should reopen prompt"
    );

    switch_dap_thread(&mut state.runtime, 2)?;
    switch_dap_stack_frame(&mut state.runtime, 2)?;

    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.focus_buffer_in_active_pane(buffer_id);
        if let Some(buffer) = ui.buffer_mut(buffer_id) {
            buffer.text.set_cursor(TextPoint::new(0, 0));
        }
    }
    apply_dap_breakpoint_extra(
        &mut state.runtime,
        DapBreakpointExtraKind::Condition,
        "x > 0",
    )?;
    apply_dap_breakpoint_extra(
        &mut state.runtime,
        DapBreakpointExtraKind::HitCondition,
        "2",
    )?;
    apply_dap_breakpoint_extra(
        &mut state.runtime,
        DapBreakpointExtraKind::LogMessage,
        "hit",
    )?;
    let bps = dap_client_manager(&state.runtime)?
        .list_breakpoints(workspace_id)
        .map_err(|e| e.to_string())?;
    let bp = bps
        .iter()
        .find(|bp| bp.line() == 1)
        .ok_or_else(|| "breakpoint missing".to_owned())?;
    assert_eq!(bp.condition(), Some("x > 0"));
    assert_eq!(bp.hit_condition(), Some("2"));
    assert_eq!(bp.log_message(), Some("hit"));

    open_dap_log_buffer(&mut state.runtime)?;
    remove_dap_expression(&mut state.runtime, "x")?;

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = workspace;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn debug_layout_hides_on_workspace_switch_and_rebuilds_on_return() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("dap-layout-ws-a");
    let second_root = unique_temp_dir("dap-layout-ws-b");
    let first = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let program = first_root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());

    let second = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    assert_ne!(first, second);
    assert!(
        !shell_ui(&state.runtime)?.is_debug_layout_active(),
        "leaving Workspace must tear down Debug Layout"
    );
    let dap = state
        .runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .ok_or_else(|| "dap manager missing".to_owned())?;
    assert!(
        dap.session_info(first.get())
            .map_err(|e| e.to_string())?
            .is_some(),
        "Debug Session must survive Workspace switch"
    );

    switch_runtime_workspace(&mut state.runtime, first)?;
    assert!(
        shell_ui(&state.runtime)?.is_debug_layout_active(),
        "returning to Workspace with live Session must rebuild Debug Layout"
    );
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 3);

    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = fs::remove_dir_all(&first_root);
    let _ = fs::remove_dir_all(&second_root);
    Ok(())
}

#[test]
fn dap_start_opens_adapter_picker_ordered_by_preference() -> Result<(), String> {
    use editor_dap::{DebugAdapterRegistry, DebugAdapterSpec, DebugAdapterTransport};

    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-adapter-picker");
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let mut registry = DebugAdapterRegistry::new();
    registry
        .register(
            DebugAdapterSpec::new("gdb-fake", "rust", ["rs"], "", [] as [&str; 0])
                .with_transport(DebugAdapterTransport::Tcp {
                    host: "127.0.0.1".to_owned(),
                    port: 1,
                })
                .with_preference(50),
        )
        .map_err(|e| e.to_string())?;
    registry
        .register(
            DebugAdapterSpec::new("codelldb-fake", "rust", ["rs"], "", [] as [&str; 0])
                .with_transport(DebugAdapterTransport::Tcp {
                    host: "127.0.0.1".to_owned(),
                    port: 2,
                })
                .with_preference(100),
        )
        .map_err(|e| e.to_string())?;
    state
        .runtime
        .services_mut()
        .insert(Arc::new(DapClientManager::new(registry)));

    start_dap_for_active_workspace(&mut state.runtime, None)?;
    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "expected Debug Adapter picker".to_owned())?;
    assert_eq!(picker.session().title(), "Choose Debug Adapter");
    let ids: Vec<_> = picker
        .session()
        .matches()
        .iter()
        .map(|entry| entry.item().label().to_owned())
        .collect();
    assert_eq!(ids, ["codelldb-fake", "gdb-fake"]);
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_start_lists_project_configurations_in_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-project-configs");
    let volt_dir = root.join(".volt");
    fs::create_dir_all(&volt_dir).map_err(|e| e.to_string())?;
    fs::write(
        volt_dir.join("debug.json"),
        r#"{
          "configurations": [
            {
              "name": "Project Launch",
              "adapter": "fake-dap",
              "request": "launch",
              "program": "main.rs"
            }
          ]
        }"#,
    )
    .map_err(|e| e.to_string())?;
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "expected Debug Configuration picker".to_owned())?;
    assert_eq!(picker.session().title(), "Choose Debug Configuration");
    let labels: Vec<_> = picker
        .session()
        .matches()
        .iter()
        .map(|entry| entry.item().label().to_owned())
        .collect();
    assert!(
        labels.iter().any(|label| label.contains("Project Launch")),
        "project config missing from {labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label.contains("Debug (current file)")),
        "inferred/compiled default missing from {labels:?}"
    );
    let _ = fake.join();
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_start_last_replays_prior_configuration() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-start-last");
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;
    start_dap_for_active_workspace(&mut state.runtime, Some("fake-dap"))?;
    stop_dap_for_active_workspace(&mut state.runtime)?;

    start_dap_last(&mut state.runtime)?;
    assert!(shell_ui(&state.runtime)?.is_debug_layout_active());
    let dap = state
        .runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .ok_or_else(|| "dap manager missing".to_owned())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    assert!(
        dap.session_info(workspace_id.get())
            .map_err(|e| e.to_string())?
            .is_some()
    );
    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_heuristic_compile_opens_confirm_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-compile-confirm");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .map_err(|e| e.to_string())?;
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;
    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;

    let configuration = DebugConfiguration::new("Debug", DebugRequestKind::Launch)
        .with_target_program(program)
        .with_cwd(root.clone());
    continue_dap_start(&mut state.runtime, "fake-dap", configuration, true)?;
    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "expected compile confirm picker".to_owned())?;
    assert_eq!(picker.session().title(), "Compile before debug?");
    let _ = fake.join();
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_default_workspace_skips_deep_inference() -> Result<(), String> {
    let state = state_with_user_library()?;
    let ctx = dap_start_context(&state.runtime)?;
    assert!(
        !ctx.allow_deep_inference,
        "Default Workspace must not deep-infer Debug Configurations"
    );
    Ok(())
}
