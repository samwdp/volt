#![allow(unused_imports)]
use super::*;

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
fn monospace_text_width_ignores_variation_selectors() {
    assert_eq!(monospace_text_width("⚛️", 8), 8);
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
        ShellDockEntries {
            workspace: &[],
            acp: &[],
        },
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
        ShellDockEntries {
            workspace: &[],
            acp: &[],
        },
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
        DrawCommand::Clear { color } if color.a == 0
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

    let settings = theme_runtime_settings(Some(&registry), &ShellConfig::default(), 1.0);

    assert_eq!(
        settings.window_effects,
        crate::window_effects::WindowEffects {
            opacity: 0.65,
            blur: 18.0,
            transparency: crate::window_effects::WindowTransparency::Blur,
        }
    );
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
        transparency: crate::window_effects::WindowTransparency::None,
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
        ShellDockEntries {
            workspace: &entries,
            acp: &[],
        },
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
