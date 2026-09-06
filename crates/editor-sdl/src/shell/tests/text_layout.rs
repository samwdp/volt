#![allow(unused_imports)]
use super::*;

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
