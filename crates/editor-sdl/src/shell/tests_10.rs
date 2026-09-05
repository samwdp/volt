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
fn render_acp_sections_apply_window_opacity_to_panel_chrome() -> Result<(), String> {
    let _guard = crate::window_effects::force_surface_window_opacity_for_tests();
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
                && color.a == 128
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == acp_layout.plan.rect.x()
                && rect.y == acp_layout.plan.rect.y()
                && rect.width == acp_layout.plan.rect.width()
                && rect.height == acp_layout.plan.rect.height()
                && color.a == 128
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == acp_layout.output.rect.x()
                && rect.y == acp_layout.output.rect.y()
                && rect.width == acp_layout.output.rect.width()
                && rect.height == acp_layout.output.rect.height()
                && color.a == 128
    )));
    Ok(())
}

#[test]
fn render_browser_selected_section_applies_window_opacity() -> Result<(), String> {
    let _guard = crate::window_effects::force_surface_window_opacity_for_tests();
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
                && color.a == 128
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
fn acp_output_wraps_long_tool_tokens_within_bubble_width() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    let title = "dotnet build src\\AssetFusion.Shared.EntityFrameworkCore\\AssetFusion.Shared.EntityFrameworkCore.csproj --no-restore 2>&1";
    buffer.acp_upsert_tool_call(
        ToolCall::new("tool-1", title)
            .kind(ToolKind::Execute)
            .status(ToolCallStatus::InProgress),
    );

    buffer.sync_acp_viewport_metrics(220, 420, 8, 16, true);

    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    let wrap_cols = acp.output_pane.wrap_cols();
    let rendered_line = acp
        .output_pane
        .render_lines
        .iter()
        .find_map(|line| match line {
            AcpRenderedLine::Text(text) if text.text == title => Some(text),
            _ => None,
        })
        .ok_or_else(|| "tool title line missing".to_owned())?;
    let text_wrap_cols = acp_rendered_text_wrap_cols(rendered_line, wrap_cols);
    let map = LineCharMap::new(&rendered_line.text);
    let segment_widths = acp_rendered_text_segments(rendered_line, wrap_cols)
        .into_iter()
        .map(|segment| map.display_cols_between(segment.start_col, segment.end_col))
        .collect::<Vec<_>>();

    assert!(
        segment_widths.iter().all(|width| *width <= text_wrap_cols),
        "segment widths {segment_widths:?} exceeded bubble width {text_wrap_cols}"
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
fn acp_screen_top_motion_targets_wrapped_visual_row() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.acp_prepare_session_replay("GitHub Copilot");
    buffer.acp_push_system_message("word ".repeat(40));
    for index in 0..12 {
        buffer.acp_push_system_message(format!("tail line {index}"));
    }
    buffer.sync_acp_viewport_metrics(220, 420, 8, 16, true);
    {
        let acp = buffer
            .acp_state
            .as_mut()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        acp.active_pane = AcpPane::Output;
        acp.output_pane.scroll_visual_row = 1;
    }

    assert!(buffer.move_to_viewport_offset(0));
    assert_eq!(buffer.cursor_point().line, 0);
    assert!(
        buffer.cursor_point().column > 0,
        "screen-top motion should target the wrapped visual row, not the logical line start"
    );
    Ok(())
}

#[test]
fn acp_page_scroll_preserves_wrapped_visual_row_alignment() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.acp_prepare_session_replay("GitHub Copilot");
    buffer.acp_push_system_message("word ".repeat(40));
    for index in 0..20 {
        buffer.acp_push_system_message(format!("tail line {index}"));
    }
    buffer.sync_acp_viewport_metrics(220, 420, 8, 16, true);
    {
        let acp = buffer
            .acp_state
            .as_mut()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        acp.active_pane = AcpPane::Output;
        acp.output_pane.set_cursor(TextPoint::new(0, 0));
    }

    apply_motion_command(&mut state.runtime, ShellMotion::Down)?;
    let before = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    scroll_buffer_with_cursor(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
        1,
    );
    let after = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();

    assert_eq!(before.line, 0);
    assert!(before.column > 0);
    assert_eq!(after.line, 0);
    assert!(
        after.column >= before.column,
        "page scroll should keep the cursor aligned to the wrapped ACP visual row"
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
fn acp_output_visual_row_motion_aligns_with_yank() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.acp_prepare_session_replay("GitHub Copilot");
    buffer.acp_push_system_message("word ".repeat(40));
    buffer.sync_acp_viewport_metrics(220, 420, 8, 16, true);
    {
        let acp = buffer
            .acp_state
            .as_mut()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        acp.active_pane = AcpPane::Output;
        acp.output_pane.set_cursor(TextPoint::new(0, 0));
    }

    apply_motion_command(&mut state.runtime, ShellMotion::Down)?;

    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        assert_eq!(
            buffer.cursor_point().line,
            0,
            "wrapped visual-row motion must stay on the same logical line"
        );
        assert!(
            buffer.cursor_point().column > 0,
            "wrapped visual-row motion should advance within the line"
        );
    }

    let anchor = TextPoint::new(0, 0);
    let expected = {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer
            .slice(
                match visual_selection(buffer, anchor, VisualSelectionKind::Character)
                    .ok_or_else(|| "visual selection missing".to_owned())?
                {
                    VisualSelection::Range(range) => range,
                    VisualSelection::Block(_) => {
                        return Err("expected range visual selection".to_owned());
                    }
                },
            )
            .trim_end_matches('\n')
            .to_owned()
    };
    shell_ui_mut(&mut state.runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);
    apply_visual_operator(&mut state.runtime, VimOperator::Yank)?;

    let ui = shell_ui(&state.runtime)?;
    let yanked = ui
        .vim()
        .yank
        .as_ref()
        .and_then(|register| match register {
            YankRegister::Character(text) => Some(text.as_str()),
            _ => None,
        })
        .ok_or_else(|| "expected character yank".to_owned())?;
    assert_eq!(yanked, expected.as_str());
    Ok(())
}

#[test]
fn acp_output_visual_anchor_survives_streaming_rebuild() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.acp_prepare_session_replay("GitHub Copilot");
    buffer.acp_push_system_message("stable prefix");
    buffer.sync_acp_viewport_metrics(640, 360, 8, 16, true);
    let line_index = buffer.line_count().saturating_sub(1);
    {
        let acp = buffer
            .acp_state
            .as_mut()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        acp.active_pane = AcpPane::Output;
        acp.output_pane.set_cursor(TextPoint::new(line_index, 0));
        acp.output_pane.scroll_visual_row = 0;
    }
    let anchor = TextPoint::new(line_index, 0);
    shell_ui_mut(&mut state.runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);
    apply_motion_command(&mut state.runtime, ShellMotion::Right)?;
    apply_motion_command(&mut state.runtime, ShellMotion::Right)?;

    let anchor_offset = shell_ui(&state.runtime)?
        .vim()
        .visual_anchor_char_offset
        .ok_or_else(|| "visual anchor offset missing".to_owned())?;

    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.acp_append_agent_chunk(ContentBlock::Text(TextContent::new(" streaming tail")));

    remap_acp_output_visual_anchors(&mut state.runtime)?;

    let remapped_anchor = shell_ui(&state.runtime)?
        .vim()
        .visual_anchor
        .ok_or_else(|| "visual anchor missing after rebuild".to_owned())?;
    let (anchor_char, selected) = {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        let pane = buffer
            .acp_state
            .as_ref()
            .ok_or_else(|| "ACP output pane is missing".to_owned())?
            .output_pane
            .clone();
        let anchor = pane
            .text
            .point_from_char_index(anchor_offset.min(pane.text.char_count()));
        assert_eq!(anchor, remapped_anchor);
        let anchor_char = pane.text.char_at_point(anchor);
        let selected = buffer
            .slice(
                match visual_selection(buffer, anchor, VisualSelectionKind::Character)
                    .ok_or_else(|| "visual selection missing".to_owned())?
                {
                    VisualSelection::Range(range) => range,
                    VisualSelection::Block(_) => {
                        return Err("expected range visual selection".to_owned());
                    }
                },
            )
            .trim_end_matches('\n')
            .to_owned();
        (anchor_char, selected)
    };
    assert_eq!(anchor_char, Some('s'));
    assert_eq!(selected, "sta");
    apply_visual_operator(&mut state.runtime, VimOperator::Yank)?;
    assert_eq!(
        shell_ui(&state.runtime)?.vim().yank,
        Some(YankRegister::Character("sta".to_owned()))
    );
    Ok(())
}

#[test]
fn terminal_visual_line_yank_includes_multiple_lines() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec![
            "line alpha".to_owned(),
            "line beta".to_owned(),
            "line gamma".to_owned(),
        ]);
        buffer.set_viewport_lines(10);
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 0), VisualSelectionKind::Line);
    apply_motion_command(&mut state.runtime, ShellMotion::Down)?;
    apply_motion_command(&mut state.runtime, ShellMotion::Down)?;
    apply_visual_operator(&mut state.runtime, VimOperator::Yank)?;

    assert_eq!(
        shell_ui(&state.runtime)?.vim().yank,
        Some(YankRegister::Line(
            "line alpha\nline beta\nline gamma".to_owned()
        ))
    );
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
    let header_radius = inner_radius.min(header_height);
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
            DrawCommand::FillTopRoundedRect { rect, radius, .. }
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
