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
    core::env!("CARGO_MANIFEST_DIR"),
    "/../volt/assets/font/material-design-icons.ttf"
));

fn berkeley_mono_font() -> Option<&'static [u8]> {
    static BERKELEY_MONO_FONT: std::sync::OnceLock<Option<Box<[u8]>>> = std::sync::OnceLock::new();
    BERKELEY_MONO_FONT
        .get_or_init(|| {
            let path = std::path::Path::new(core::env!("CARGO_MANIFEST_DIR"))
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
