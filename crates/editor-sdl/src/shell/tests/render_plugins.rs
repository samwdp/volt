use super::*;

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
    let panel_background = buffer_section_panel_background(base_background);
    let header_background = buffer_section_header_background(None, panel_background);
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
        DrawCommand::FillTopRoundedRect { rect, color, .. }
            if rect.x == header_rect.x()
                && rect.y == header_rect.y()
                && rect.width == header_rect.width()
                && rect.height == header_rect.height()
                && *color == to_render_color(header_background)
    )));
    Ok(())
}

#[test]
fn render_plugin_sections_apply_window_opacity_to_panel_chrome() -> Result<(), String> {
    let _guard = crate::window_effects::force_surface_window_opacity_for_tests();
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
                && color.a == 128
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillTopRoundedRect { rect, color, .. }
            if rect.x == header_rect.x()
                && rect.y == header_rect.y()
                && rect.width == header_rect.width()
                && rect.height == header_rect.height()
                && color.a == 128
    )));
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
