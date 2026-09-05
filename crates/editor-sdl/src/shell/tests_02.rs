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
fn oil_insert_patches_listing_without_rereading_siblings() -> Result<(), String> {
    let root = unique_temp_dir("oil-insert-patch");
    std::fs::write(root.join("existing.txt"), "keep\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    std::fs::write(root.join("sneaky.txt"), "external\n").map_err(|error| error.to_string())?;
    oil_type_new_entry_and_leave_insert(&mut state, "abc.txt")?;

    assert!(root.join("abc.txt").is_file());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "abc.txt").is_ok(),
        "created file should appear in the patched listing"
    );
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "sneaky.txt").is_err(),
        "insert must not reread siblings created on disk after open"
    );

    state
        .runtime
        .execute_command("oil.refresh")
        .map_err(|error| error.to_string())?;
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "sneaky.txt").is_ok(),
        "explicit refresh should reread the directory"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_toggle_hidden_filters_cached_entries_without_reread() -> Result<(), String> {
    let root = unique_temp_dir("oil-hidden-cache");
    std::fs::write(root.join(".hidden"), "hidden\n").map_err(|error| error.to_string())?;
    std::fs::write(root.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    std::fs::write(root.join("sneaky.txt"), "external\n").map_err(|error| error.to_string())?;
    state
        .runtime
        .execute_command("oil.toggle-hidden")
        .map_err(|error| error.to_string())?;

    assert!(oil_line_index_containing(&state.runtime, buffer_id, ".hidden").is_ok());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "sneaky.txt").is_err(),
        "hidden toggle must filter the cached listing"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_cycle_sort_does_not_reread_listing() -> Result<(), String> {
    let root = unique_temp_dir("oil-sort-cache");
    std::fs::write(root.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(root.join("zeta.txt"), "zeta\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    std::fs::write(root.join("sneaky.txt"), "external\n").map_err(|error| error.to_string())?;
    state
        .runtime
        .execute_command("oil.cycle-sort")
        .map_err(|error| error.to_string())?;

    assert!(oil_line_index_containing(&state.runtime, buffer_id, "alpha.txt").is_ok());
    assert!(oil_line_index_containing(&state.runtime, buffer_id, "zeta.txt").is_ok());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "sneaky.txt").is_err(),
        "sort must reorder the cached listing without a disk walk"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_rename_moves_cursor_to_new_entry_not_substring_sibling() -> Result<(), String> {
    let root = unique_temp_dir("oil-rename-cursor");
    std::fs::write(root.join("old.txt"), "old\n").map_err(|error| error.to_string())?;
    std::fs::write(root.join("a_foo.txt"), "sibling\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    let old_line = oil_line_index_containing(&state.runtime, buffer_id, "old.txt")?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(old_line, 0));

    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("foo.txt")
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;

    assert!(root.join("foo.txt").is_file());
    assert!(!root.join("old.txt").exists());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "a_foo.txt").is_ok(),
        "substring sibling must stay in the patched listing"
    );

    let cursor_line = shell_buffer(&state.runtime, buffer_id)?.cursor_point().line;
    let renamed_line =
        oil_line_index_for_entry_path(&state.runtime, buffer_id, &root.join("foo.txt"))?;
    assert_eq!(
        cursor_line, renamed_line,
        "rename cursor follow must match the new entry path, not a substring sibling"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_root_change_rereads_the_new_directory() -> Result<(), String> {
    let root = unique_temp_dir("oil-root-reread");
    let child = root.join("child");
    std::fs::create_dir(&child).map_err(|error| error.to_string())?;
    std::fs::write(child.join("inside.txt"), "inside\n").map_err(|error| error.to_string())?;
    std::fs::write(root.join("outside.txt"), "outside\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    let child_line = oil_line_index_containing(&state.runtime, buffer_id, "child")?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(child_line, 0));
    state
        .runtime
        .execute_command("oil.set-root")
        .map_err(|error| error.to_string())?;

    assert!(oil_line_index_containing(&state.runtime, buffer_id, "inside.txt").is_ok());
    assert!(oil_line_index_containing(&state.runtime, buffer_id, "outside.txt").is_err());

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
