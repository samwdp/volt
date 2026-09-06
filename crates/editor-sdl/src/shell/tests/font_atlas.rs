#![allow(unused_imports)]
use super::*;

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
fn scaled_font_size_uses_window_display_scale() {
    assert_eq!(scaled_font_size(18, 2.0), 36.0);
    assert_eq!(scaled_font_size(18, 1.25), 22.5);
    assert_eq!(scaled_font_size(18, -1.0), 18.0);
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
