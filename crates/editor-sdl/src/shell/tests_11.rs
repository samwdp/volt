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
