use super::*;
use agent_client_protocol::{
    Plan, PlanEntry, PlanEntryPriority, PlanEntryStatus, ToolCall, ToolCallContent, ToolCallStatus,
    ToolCallUpdate, ToolCallUpdateFields, ToolKind,
};
use editor_lsp::{LanguageServerRegistry, LspClientManager, LspLogDirection};
use editor_plugin_api::{
    AcpClient, AutocompleteProvider, DebugAdapterSpec, GhostTextContext, HoverProvider,
    LanguageConfiguration, LanguageServerSpec, LigatureConfig, OilDefaults, OilKeyAction,
    OilKeybindings, PluginBuffer, PluginBufferSections, TerminalConfig, Theme, WorkspaceRoot,
};
use editor_plugin_host::StatuslineContext;
use editor_render::horizontal_pane_rects;
use sdl3::mouse::MouseState;
use sdl3::video::WindowFlags;
use std::{
    collections::BTreeMap,
    env, fs,
    sync::Arc,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
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

struct HeaderlineTestUserLibrary {
    scrolloff: f64,
    headerline_lines: Vec<String>,
    headerline_requires_scrolled_viewport: bool,
}

impl Default for HeaderlineTestUserLibrary {
    fn default() -> Self {
        Self {
            scrolloff: 1.0,
            headerline_lines: vec!["fn render(value: usize)".to_owned()],
            headerline_requires_scrolled_viewport: false,
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

    fn ligature_config(&self) -> LigatureConfig {
        LigatureConfig { enabled: false }
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

    fn git_commit_template(&self) -> Vec<String> {
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

    fn headerline_lines(&self, context: &GhostTextContext<'_>) -> Vec<String> {
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
                span.theme_token.clone(),
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
    let lines = pdf_buffer_lines("sample.pdf", Some(&path), &state);
    let body = lines.join("\n");
    assert!(body.contains("Page 2/2"));
    assert!(body.contains("rotation 90°"));
    assert!(body.contains("page two"));
    assert!(body.contains("Modified: yes"));

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
fn contextual_ligature_raster_size_expands_changed_glyphs() {
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
                    ) > 18.0
            })
    );
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
fn normalize_premultiplied_rgba_surface_restores_rgb_for_partially_transparent_pixels() {
    let mut surface = Surface::new(1, 1, PixelFormat::RGBA32)
        .unwrap_or_else(|error| panic!("failed to create surface: {error}"));
    surface.with_lock_mut(|pixels| {
        pixels[..4].copy_from_slice(&[64, 32, 16, 64]);
    });

    let surface = normalize_premultiplied_rgba_surface(surface)
        .unwrap_or_else(|error| panic!("failed to normalize surface: {error}"));
    surface.with_lock(|pixels| {
        assert_eq!(&pixels[..4], &[255, 128, 64, 64]);
    });
}

#[test]
fn normalize_premultiplied_rgba_surface_clears_rgb_for_fully_transparent_pixels() {
    let mut surface = Surface::new(1, 1, PixelFormat::RGBA32)
        .unwrap_or_else(|error| panic!("failed to create surface: {error}"));
    surface.with_lock_mut(|pixels| {
        pixels[..4].copy_from_slice(&[15, 25, 35, 0]);
    });

    let surface = normalize_premultiplied_rgba_surface(surface)
        .unwrap_or_else(|error| panic!("failed to normalize surface: {error}"));
    surface.with_lock(|pixels| {
        assert_eq!(&pixels[..4], &[0, 0, 0, 0]);
    });
}

#[test]
fn collapse_subpixel_bitmap_to_alpha_averages_channels() {
    assert_eq!(
        collapse_subpixel_bitmap_to_alpha(2, &[255, 0, 0, 0, 255, 255]),
        vec![85, 170]
    );
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
fn terminal_key_for_event_maps_special_keys() {
    assert_eq!(
        terminal_key_for_event(Keycode::Tab, Mod::LSHIFTMOD),
        Some(TerminalKey::BackTab)
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
    let defaults = editor_plugin_host::NullUserLibrary.oil_defaults();
    let state = DirectoryViewState::new(std::path::PathBuf::from("."), Vec::new(), defaults);

    assert_eq!(state.show_hidden, defaults.show_hidden);
    assert_eq!(state.sort_mode, defaults.sort_mode);
    assert_eq!(state.trash_enabled, defaults.trash_enabled);
}

#[test]
fn oil_normal_mode_dd_applies_delete_immediately() -> Result<(), String> {
    let root = unique_temp_dir("oil-normal-delete");
    let file_path = root.join("alpha.txt");
    std::fs::write(&file_path, "alpha\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    assert!(!file_path.exists());
    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.line_count(), 1);

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
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
    assert!(buffer.text.text().contains("hello from page one"));

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
        0,
        0,
        line,
        LineWrapSegment {
            start_col: 0,
            end_col: 3,
        },
        &char_map,
        None,
        None,
        default_color,
        8,
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
        0,
        0,
        line,
        LineWrapSegment {
            start_col: 0,
            end_col: line.chars().count(),
        },
        &char_map,
        None,
        None,
        default_color,
        8,
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
        0,
        0,
        &formatted,
        LineWrapSegment {
            start_col: 0,
            end_col: formatted.chars().count(),
        },
        &char_map,
        Some(&spans),
        None,
        Color::RGB(240, 240, 240),
        8,
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
    assert_eq!(
        visible_headerline_lines(
            vec![
                "module app".to_owned(),
                "impl Demo".to_owned(),
                "render(value: usize)".to_owned(),
            ],
            3,
        ),
        vec!["impl Demo".to_owned(), "render(value: usize)".to_owned()]
    );
}

#[test]
fn visible_headerline_lines_reserves_at_least_one_buffer_row() {
    assert!(visible_headerline_lines(vec!["render()".to_owned()], 1).is_empty());
}

#[test]
fn render_buffer_headerline_overlays_without_shifting_buffer_rows() -> Result<(), String> {
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
        buffer,
        rect,
        true,
        None,
        None,
        None,
        InputMode::Normal,
        false,
        None,
        None,
        render_user_library.commandline_enabled(),
        &render_user_library,
        "default",
        None,
        false,
        false,
        None,
        state.runtime.services().get::<ThemeRegistry>(),
        false,
        8,
        16,
        12,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. } if *y == layout.body_y && text == "beta"
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. } if *y == layout.body_y + 16 && text == "beta"
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_does_not_shift_buffer_rows_or_cursor() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary::default();
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(&mut state, "*headerline*", vec!["alpha".to_owned()])?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let cursor_color = to_render_color(Color::RGB(110, 170, 255));
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        buffer,
        rect,
        true,
        None,
        None,
        None,
        InputMode::Normal,
        false,
        None,
        None,
        render_user_library.commandline_enabled(),
        &render_user_library,
        "default",
        None,
        false,
        false,
        None,
        None,
        false,
        8,
        16,
        12,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y && text == "alpha"
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y && text == "fn render(value: usize)"
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.y == layout.body_y && *color == cursor_color
    )));
    assert!(!scene.iter().any(|command| matches!(
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
    };
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec![
            "module app".to_owned(),
            "fn render(value: usize)".to_owned(),
        ],
        headerline_requires_scrolled_viewport: false,
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
        buffer,
        rect,
        true,
        None,
        None,
        None,
        InputMode::Normal,
        false,
        None,
        None,
        render_user_library.commandline_enabled(),
        &render_user_library,
        "default",
        None,
        false,
        false,
        None,
        None,
        false,
        8,
        16,
        12,
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
    };
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 3.0,
        headerline_lines: vec!["STICKY HEADER".to_owned()],
        headerline_requires_scrolled_viewport: true,
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
        shell_buffer(&state.runtime, buffer_id)?,
        rect,
        true,
        None,
        None,
        None,
        InputMode::Normal,
        false,
        None,
        None,
        render_user_library.commandline_enabled(),
        &render_user_library,
        "default",
        None,
        false,
        false,
        None,
        None,
        false,
        8,
        16,
        12,
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
        buffer,
        rect,
        true,
        None,
        None,
        None,
        InputMode::Normal,
        false,
        None,
        None,
        render_user_library.commandline_enabled(),
        &render_user_library,
        "default",
        None,
        false,
        false,
        None,
        None,
        false,
        8,
        16,
        12,
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
    let layout = buffer_footer_layout(buffer, rect, line_height, cell_width);
    let expected_scrolloff = 3usize.min(layout.visible_rows.saturating_sub(1) / 2);
    assert!(expected_scrolloff > 1);
    let anchor = buffer_cursor_screen_anchor(
        buffer,
        rect,
        &*user_library,
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
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
fn acp_wrapped_text_uses_full_width_on_continuation_rows() {
    let line = AcpRenderedTextLine {
        prefix: vec![
            acp_icon_segment(editor_icons::symbols::cod::COD_COMMENT, AcpColorRole::Accent),
            acp_text_segment(" ", AcpColorRole::Default),
        ],
        text: "Excellent! Now let me gather more context about the project to inform the documentation content:".to_owned(),
        text_role: AcpColorRole::Default,
    };

    let segments = acp_rendered_text_segments(&line, 32);

    assert!(segments.len() > 1);
    assert!(segments[1].end_col.saturating_sub(segments[1].start_col) > 8);
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
fn block_cursor_text_overlay_positions_multibyte_cursor_text() {
    let line = "aéz";
    let char_map = LineCharMap::new(line);
    let overlay = block_cursor_text_overlay(
        24,
        line,
        &char_map,
        LineWrapSegment {
            start_col: 0,
            end_col: 3,
        },
        0,
        0,
        1,
        Some(Color::RGB(1, 2, 3)),
        8,
    )
    .expect("cursor on a multibyte character should produce an overlay");

    assert_eq!(overlay.draw_x, 32);
    assert_eq!(overlay.text, "é");
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
            capture_name: "source_file".to_owned(),
            theme_token: "syntax.source".to_owned(),
        },
        LineSyntaxSpan {
            start: 0,
            end: 3,
            capture_name: "keyword".to_owned(),
            theme_token: "syntax.keyword".to_owned(),
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

fn open_repo_git_status_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "git-test", root)?;
    let buffer_id = install_git_status_test_buffer(state)?;
    refresh_git_status_buffer(&mut state.runtime, buffer_id)?;
    Ok(buffer_id)
}

fn open_oil_test_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "oil-test", root)?;
    open_oil_directory(&mut state.runtime, root.to_path_buf())?;
    active_shell_buffer_id(&state.runtime)
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
fn context_overlay_cache_reuses_stale_snapshot_while_typing() {
    let cached = BufferContextOverlaySnapshot {
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
    };
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

    assert_eq!(snapshot.key.buffer_revision, 41);
    assert_eq!(snapshot.headerline_lines, vec!["fn demo".to_owned()]);
}

#[test]
fn context_overlay_cache_requires_matching_buffer_identity() {
    let cached = BufferContextOverlaySnapshot {
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
    };
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
    assert_eq!(
        git_status_command_name(&NullUserLibrary, None, "S"),
        Some("git.status.stage-all")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Pull), "u"),
        Some("git.status.pull-upstream")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Branch), "b"),
        Some("git.status.branches")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Diff), "w"),
        Some("git.diff")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Log), "l"),
        Some("git.log")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Stash), "l"),
        Some("git.stash-list")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Rebase), "f"),
        Some("git.status.rebase-autosquash")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Reset), "f"),
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
            .is_some_and(|action| action.id() == user::git::ACTION_COMMIT_OPEN)
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
    let user_library = editor_plugin_host::NullUserLibrary;
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
    let user_library = editor_plugin_host::NullUserLibrary;
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
    assert_eq!(commandline_y - layout.statusline_y, 18);
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
        buffer,
        rect,
        true,
        None,
        None,
        None,
        InputMode::Normal,
        false,
        None,
        None,
        true,
        &NullUserLibrary,
        "default",
        None,
        false,
        false,
        None,
        None,
        false,
        8,
        16,
        12,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
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
            font_size: 16,
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
        &NullUserLibrary,
        "default",
        None,
        false,
        false,
        None,
        320,
        180,
        8,
        16,
        12,
        Instant::now(),
        false,
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
            font_size: 16,
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
        &NullUserLibrary,
        "default",
        None,
        false,
        false,
        Some(&registry),
        320,
        180,
        8,
        16,
        12,
        Instant::now(),
        false,
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
fn theme_runtime_settings_resolve_window_effects_from_theme_options() {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.65)
                .with_option(crate::window_effects::OPTION_WINDOW_BLUR, 18.0),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let settings = theme_runtime_settings(Some(&registry), &ShellConfig::default());

    assert_eq!(
        settings.window_effects,
        crate::window_effects::WindowEffects {
            opacity: 0.65,
            blur: 18.0,
        }
    );
}

#[test]
fn render_picker_overlay_keeps_picker_background_opaque_with_window_opacity() -> Result<(), String>
{
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            font_size: 16,
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
        }],
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    picker::render_picker_overlay(&mut target, &fonts, &picker, 320, 180, 16, Some(&registry))
        .map_err(|error| error.to_string())?;

    let popup_rect = centered_rect(320, 180, 320 * 2 / 3, 180 * 3 / 5);
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == popup_rect.x + 2
                && rect.y == popup_rect.y + 2
                && rect.width == popup_rect.width.saturating_sub(4)
                && rect.height == popup_rect.height.saturating_sub(4)
                && color.a == 255
    )));
    Ok(())
}

#[test]
fn render_picker_overlay_uses_higher_contrast_muted_text() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            font_size: 16,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let picker = PickerOverlay::from_entries(
        "Projects",
        vec![
            PickerEntry {
                item: PickerItem::new(
                    ".config",
                    ".config",
                    "git",
                    Some("C:\\Users\\sam\\.config".to_owned()),
                ),
                action: PickerAction::NoOp,
            },
            PickerEntry {
                item: PickerItem::new("4coder", "4coder", "git", Some("P:\\4ed".to_owned())),
                action: PickerAction::NoOp,
            },
        ],
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    picker::render_picker_overlay(&mut target, &fonts, &picker, 320, 180, 16, None)
        .map_err(|error| error.to_string())?;

    let base_background = Color::RGB(29, 32, 40);
    let popup_background = adjust_color(base_background, 8);
    let foreground = Color::RGBA(215, 221, 232, 255);
    let expected_muted = blend_color(foreground, popup_background, 0.25);

    let text_commands = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, color, .. } => Some((text.clone(), *color)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(
        text_commands.iter().any(|(text, color)| {
            text == "Query > " && *color == to_render_color(expected_muted)
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
            Some(Hinting::NORMAL)
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
fn hidden_window_startup_smoke_supports_window_effects() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let video = sdl_context.video().map_err(|error| error.to_string())?;
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
        buffer,
        rect,
        layout,
        true,
        None,
        None,
        InputMode::Normal,
        None,
        base_background,
        Color::RGB(215, 221, 232),
        Color::RGB(140, 144, 152),
        Color::RGB(40, 44, 52),
        Color::RGBA(55, 71, 99, 255),
        Color::RGBA(112, 196, 255, 120),
        Color::RGB(110, 170, 255),
        2,
        8,
        16,
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
fn render_plugin_sections_header_applies_window_opacity() -> Result<(), String> {
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
    let panel_background = theme_color(
        Some(&registry),
        "ui.panel.background",
        adjust_color(base_background, 8),
    );
    let header_background = theme_color(
        Some(&registry),
        "ui.panel.header.background",
        adjust_color(panel_background, 12),
    );
    let expected = to_render_color(window_surface_color(
        header_background,
        current_window_effect_settings(Some(&registry)),
    ));
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_plugin_section_buffer_body(
        &mut target,
        buffer,
        rect,
        layout,
        true,
        None,
        None,
        InputMode::Normal,
        Some(&registry),
        base_background,
        Color::RGB(215, 221, 232),
        Color::RGB(140, 144, 152),
        Color::RGB(40, 44, 52),
        Color::RGBA(55, 71, 99, 255),
        Color::RGBA(112, 196, 255, 120),
        Color::RGB(110, 170, 255),
        2,
        8,
        16,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == header_rect.x()
                && rect.y == header_rect.y()
                && rect.width == header_rect.width()
                && rect.height == header_rect.height()
                && *color == expected
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
        buffer,
        rect,
        layout,
        true,
        Some(VisualSelection::Range(TextRange::new(
            TextPoint::new(0, 0),
            TextPoint::new(0, 5),
        ))),
        None,
        InputMode::Visual,
        None,
        Color::RGB(15, 16, 20),
        Color::RGB(215, 221, 232),
        Color::RGB(140, 144, 152),
        Color::RGB(40, 44, 52),
        selection_color,
        Color::RGBA(112, 196, 255, 120),
        Color::RGB(110, 170, 255),
        2,
        8,
        16,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { color, .. } if *color == to_render_color(selection_color)
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

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let selection_color = Color::RGBA(55, 71, 99, 255);
    let line_index = buffer.line_count().saturating_sub(1);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        buffer,
        rect,
        layout,
        true,
        Some(VisualSelection::Range(TextRange::new(
            TextPoint::new(line_index, 0),
            TextPoint::new(line_index, 5),
        ))),
        None,
        InputMode::Visual,
        None,
        Color::RGB(15, 16, 20),
        Color::RGB(215, 221, 232),
        Color::RGB(140, 144, 152),
        Color::RGB(40, 44, 52),
        selection_color,
        Color::RGBA(112, 196, 255, 120),
        Color::RGB(110, 170, 255),
        2,
        8,
        16,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { color, .. } if *color == to_render_color(selection_color)
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
    let header_radius = 9.min(header_height / 2);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        buffer,
        rect,
        layout,
        true,
        None,
        None,
        InputMode::Normal,
        None,
        Color::RGB(15, 16, 20),
        Color::RGB(215, 221, 232),
        Color::RGB(140, 144, 152),
        Color::RGB(40, 44, 52),
        Color::RGBA(55, 71, 99, 255),
        Color::RGBA(112, 196, 255, 120),
        Color::RGB(110, 170, 255),
        2,
        8,
        16,
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
    );
    assert!(before_sync.is_none());
    assert!(after_sync.is_some());
    Ok(())
}

#[test]
fn material_icons_rasterize_from_nfm_with_fontdue() -> Result<(), String> {
    let font_path = resolve_bundled_icon_font_dir()
        .map_err(|error| error.to_string())?
        .join("NFM.ttf");
    let bytes = fs::read(&font_path).map_err(|error| error.to_string())?;
    let font = RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())?;
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

#[test]
fn codicon_glyphs_fit_inside_one_editor_cell() -> Result<(), String> {
    let font_path = resolve_bundled_icon_font_dir()
        .map_err(|error| error.to_string())?
        .join("NFM.ttf");
    let bytes = fs::read(&font_path).map_err(|error| error.to_string())?;
    let font = RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())?;
    let codicon = editor_icons::symbols::cod::COD_DIFF_ADDED
        .chars()
        .next()
        .ok_or_else(|| "codicon glyph missing".to_owned())?;
    let requested_pixel_size = 18.0;
    let (raw_metrics, _) = font.rasterize(codicon, requested_pixel_size);
    let cell_width = raw_metrics.width.saturating_sub(1).max(1) as i32;
    let (fitted_metrics, _) =
        rasterize_icon_glyph_for_cell(&font, codicon, requested_pixel_size, cell_width);
    let layout = icon_glyph_cell_layout(&fitted_metrics, cell_width);

    assert!(raw_metrics.width > cell_width as usize);
    assert!(fitted_metrics.width as i32 <= cell_width);
    assert_eq!(layout.advance, cell_width);
    assert!(layout.draw_offset_x >= 0);
    assert!(layout.draw_offset_x + fitted_metrics.width as i32 <= cell_width);
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
        resolve_font_role_for_char(Some(0), true, false, branch),
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
        resolve_font_role_for_char(Some(0), true, false, prompt),
        FontRole::Icon(0)
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
        result_limit: 8,
        lsp_client: None,
    };

    let entries = autocomplete_entries(&request);
    assert!(!entries.is_empty());
    assert!(entries.iter().all(|entry| entry.provider_id == "primary"));
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
                capture_name: "function".to_owned(),
                theme_token: "syntax.function".to_owned(),
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
            .any(|span| span.theme_token == "syntax.keyword")
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
    let lines = index_syntax_lines(editor_syntax::SyntaxSnapshot {
        language_id: "rust".to_owned(),
        root_kind: "source_file".to_owned(),
        has_errors: false,
        highlight_spans: vec![editor_syntax::HighlightSpan {
            start_byte: 0,
            end_byte: 5,
            start_position: editor_syntax::SyntaxPoint::new(0, 0),
            end_position: editor_syntax::SyntaxPoint::new(0, 5),
            capture_name: "function".to_owned(),
            theme_token: "syntax.function".to_owned(),
        }],
    });

    let spans = lines.get(&0).expect("expected indexed syntax line");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].capture_name, "function");
    assert_eq!(spans[0].theme_token, "syntax.function");
}

#[test]
fn browser_buffer_submit_tracks_current_url() -> Result<(), String> {
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
    assert_eq!(
        state.current_url.as_deref(),
        Some("https://example.com/docs")
    );
    assert_eq!(buffer.display_name(), "*browser* https://example.com/docs");
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

    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
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
                }),
                AcpRenderedLine::Text(AcpRenderedTextLine {
                    prefix: Vec::new(),
                    text: "beta".to_owned(),
                    text_role: AcpColorRole::Default,
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
        "prompt".chars().count().saturating_sub(1)
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
    assert!(buffer.text.text().contains("https://docs.rs/volt"));
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
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
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
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
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
    let state = ShellState::new().map_err(|error| error.to_string())?;

    let write_matches = vim_command_line_completion_matches(&state.runtime, "wr");
    assert!(write_matches.contains(&"write".to_owned()));

    let buffer_matches = vim_command_line_completion_matches(&state.runtime, "bd");
    assert!(buffer_matches.contains(&"bd".to_owned()));
    assert!(buffer_matches.contains(&"bdelete".to_owned()));
    Ok(())
}

#[test]
fn execute_vim_command_line_split_alias_splits_workspace() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;

    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    execute_vim_command_line(&mut state.runtime, "split")?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 2);
    Ok(())
}

#[test]
fn execute_vim_command_line_commands_alias_opens_picker() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;

    execute_vim_command_line(&mut state.runtime, "commands")?;
    assert!(shell_ui(&state.runtime)?.picker().is_some());
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
        None,
        &*state.user_library,
        800,
        400,
        8,
        18,
        Instant::now(),
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
fn browser_sync_plan_hides_surfaces_while_picker_is_visible() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .set_picker(PickerOverlay::from_entries("Buffers", Vec::new()));

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        None,
        &*state.user_library,
        800,
        400,
        8,
        18,
        Instant::now(),
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(plan.buffers.len(), 1);
    assert!(plan.visible_surfaces.is_empty());
    Ok(())
}

#[test]
fn browser_sync_plan_hides_surfaces_when_notifications_overlap() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .apply_notification(
            test_notification_update(
                "progress",
                NotificationSeverity::Info,
                "LSP · rust-analyzer",
                &["Indexing workspace", "Scanning project"],
                Some(32),
                true,
            ),
            Instant::now(),
        );

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        None,
        &*state.user_library,
        800,
        400,
        8,
        18,
        Instant::now(),
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(plan.buffers.len(), 1);
    assert!(plan.visible_surfaces.is_empty());
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
fn browser_url_command_opens_popup_browser_with_detected_url() -> Result<(), String> {
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

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "browser popup was not opened".to_owned())?;
    let buffer = shell_ui(&state.runtime)?
        .buffer(popup.active_buffer)
        .ok_or_else(|| "browser popup buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    assert!(buffer.text.text().contains("https://example.com/docs"));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
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

    assert!(handle_terminal_vim_edit(&mut state.runtime, "put-after")?);
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
        buffer,
        buffer
            .terminal_render()
            .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
        rect,
        layout,
        true,
        InputMode::Normal,
        None,
        None,
        None,
        Color::RGB(15, 16, 20),
        Color::RGB(110, 170, 255),
        Color::RGB(215, 221, 232),
        Color::RGB(40, 44, 52),
        "status".to_owned(),
        Color::RGB(110, 170, 255),
        Color::RGB(140, 144, 152),
        Color::RGBA(55, 71, 99, 255),
        Color::RGBA(112, 196, 255, 120),
        8,
        16,
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
        buffer,
        buffer
            .terminal_render()
            .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
        rect,
        layout,
        true,
        InputMode::Visual,
        Some(VisualSelection::Range(TextRange::new(
            TextPoint::new(0, 0),
            TextPoint::new(0, 4),
        ))),
        None,
        None,
        Color::RGB(15, 16, 20),
        Color::RGB(110, 170, 255),
        Color::RGB(215, 221, 232),
        Color::RGB(40, 44, 52),
        "status".to_owned(),
        Color::RGB(110, 170, 255),
        Color::RGB(140, 144, 152),
        selection_color,
        Color::RGBA(112, 196, 255, 120),
        8,
        16,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { color, .. } if *color == to_render_color(selection_color)
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
        buffer,
        buffer
            .terminal_render()
            .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
        rect,
        layout,
        true,
        InputMode::Insert,
        None,
        None,
        Some(&registry),
        Color::RGB(15, 16, 20),
        Color::RGB(110, 170, 255),
        Color::RGB(215, 221, 232),
        Color::RGB(40, 44, 52),
        "status".to_owned(),
        Color::RGB(110, 170, 255),
        Color::RGB(140, 144, 152),
        Color::RGBA(55, 71, 99, 255),
        Color::RGBA(112, 196, 255, 120),
        8,
        16,
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
        buffer,
        rect,
        true,
        None,
        Some(&multicursor),
        None,
        InputMode::Insert,
        true,
        None,
        None,
        NullUserLibrary.commandline_enabled(),
        &NullUserLibrary,
        "default",
        None,
        false,
        false,
        None,
        None,
        false,
        8,
        16,
        12,
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
        buffer,
        buffer
            .terminal_render()
            .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
        rect,
        layout,
        true,
        InputMode::Normal,
        None,
        None,
        None,
        Color::RGB(15, 16, 20),
        cursor_color,
        Color::RGB(215, 221, 232),
        Color::RGB(40, 44, 52),
        "status".to_owned(),
        Color::RGB(110, 170, 255),
        Color::RGB(140, 144, 152),
        Color::RGBA(55, 71, 99, 255),
        Color::RGBA(112, 196, 255, 120),
        8,
        16,
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
