use super::*;

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
fn acp_rendered_text_segments_break_long_tokens_before_future_whitespace() {
    let line = acp_text_line(
        Vec::new(),
        "src\\AssetFusion.Shared.EntityFrameworkCore\\AssetFusion.Shared.EntityFrameworkCore.csproj --no-restore",
        AcpColorRole::Default,
    );
    let map = LineCharMap::new(&line.text);
    let widths = acp_rendered_text_segments(&line, 13)
        .into_iter()
        .map(|segment| map.display_cols_between(segment.start_col, segment.end_col))
        .collect::<Vec<_>>();

    assert!(
        widths.iter().all(|width| *width <= 13),
        "segment widths must stay within the wrap width: {widths:?}"
    );
}

#[test]
fn acp_rendered_text_segments_skip_whitespace_only_rows_after_hard_break() {
    let line = acp_text_line(
        Vec::new(),
        "abcdefghijklm              --flag more",
        AcpColorRole::Default,
    );
    let map = LineCharMap::new(&line.text);
    let texts = acp_rendered_text_segments(&line, 13)
        .iter()
        .map(|segment| map.slice(&line.text, segment.start_col, segment.end_col))
        .collect::<Vec<_>>();

    assert!(
        texts.iter().all(|text| !text.trim().is_empty()),
        "segments must not collapse to whitespace-only rows: {texts:?}"
    );
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
fn acp_dock_toggle_shows_and_hides() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_open());
    assert!(!acp_dock_visible(shell_ui(&state.runtime)?));

    toggle_acp_dock(&mut state.runtime)?;
    assert!(shell_ui(&state.runtime)?.acp_dock_open());
    assert!(acp_dock_visible(shell_ui(&state.runtime)?));

    toggle_acp_dock(&mut state.runtime)?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_open());
    assert!(!acp_dock_visible(shell_ui(&state.runtime)?));
    Ok(())
}

#[test]
fn acp_dock_focus_j_k_cycle_buffers() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first = install_user_plugin_buffer(&mut state, "*acp Claude*", user::acp::ACP_BUFFER_KIND)?;
    let second = install_user_plugin_buffer(&mut state, "*acp Codex*", user::acp::ACP_BUFFER_KIND)?;
    shell_buffer_mut(&mut state.runtime, first)?.init_acp_view("Claude");
    shell_buffer_mut(&mut state.runtime, second)?.init_acp_view("Codex");
    acp::focus_acp_buffer(&mut state.runtime, first)?;
    toggle_acp_dock(&mut state.runtime)?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_acp_dock_focus(true);
        ui.enter_normal_mode();
    }

    let modes = state
        .overlay_minor_modes()
        .map_err(|error| error.to_string())?;
    assert!(
        modes.contains(&KeymapScope::AcpDock),
        "dock focus must activate ACP Dock Minor Mode: {modes:?}"
    );
    assert!(
        !modes.contains(&KeymapScope::WorkspaceDock),
        "ACP dock focus must not activate Workspace Dock Minor Mode: {modes:?}"
    );

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_buffer_id(), Some(second));
    assert!(shell_ui(&state.runtime)?.acp_dock_focus_active());
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_buffer_id(), Some(first));
    assert!(shell_ui(&state.runtime)?.acp_dock_focus_active());
    Ok(())
}

#[test]
fn acp_dock_entries_list_active_workspace_buffers() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first = install_user_plugin_buffer(&mut state, "*acp Claude*", user::acp::ACP_BUFFER_KIND)?;
    let second = install_user_plugin_buffer(&mut state, "*acp Codex*", user::acp::ACP_BUFFER_KIND)?;
    shell_buffer_mut(&mut state.runtime, first)?.init_acp_view("Claude");
    shell_buffer_mut(&mut state.runtime, second)?.init_acp_view("Codex");
    shell_buffer_mut(&mut state.runtime, first)?
        .acp_set_session_title(Some("Refactor dock".to_owned()));

    let entries = collect_acp_dock_entries(&state.runtime)?;
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].buffer_id, first);
    assert_eq!(entries[0].name, "Claude");
    assert_eq!(entries[0].session, "Refactor dock");
    assert_eq!(entries[1].buffer_id, second);
    assert_eq!(entries[1].name, "Codex");
    assert_eq!(entries[1].session, "New session");
    assert!(entries.iter().any(|entry| entry.active));
    Ok(())
}

#[test]
fn acp_dock_layout_shrinks_content_on_the_right() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_open());
    toggle_acp_dock(&mut state.runtime)?;
    let docks = shell_docks_layout(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?,
        640,
        360,
        8,
    );
    assert!(docks.acp.visible);
    assert_eq!(docks.acp.side, WorkspaceDockSide::Right);
    assert!(docks.acp.dock_width > 0);
    assert_eq!(docks.content_width + docks.acp.dock_width, 640);
    assert_eq!(docks.content_x, 0);
    Ok(())
}
