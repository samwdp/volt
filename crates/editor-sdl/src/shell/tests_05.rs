#[test]
fn insert_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*newline-wrap-edit*",
        vec![
            "abcde".to_owned(),
            "    wrappedtail".to_owned(),
            "end".to_owned(),
        ],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 3));

    buffer.insert_text("\n");

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)?;
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was cleared after newline insert".to_owned())?;
    assert_eq!(cache.prefix_rows.len(), 5);
    Ok(())
}

#[test]
fn join_lines_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*join-wrap-edit*",
        vec!["abcde".to_owned(), "fghij".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(1, 0));

    buffer.backspace();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)?;
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was cleared after join".to_owned())?;
    assert_eq!(cache.prefix_rows.len(), 3);
    Ok(())
}

#[test]
fn delete_forward_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*delete-newline-wrap-edit*",
        vec!["abcde".to_owned(), "fghij".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 5));

    buffer.delete_forward();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn newline_insert_does_not_create_wrap_cache() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*newline-no-wrap-cache*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = None;
    buffer.set_cursor(TextPoint::new(0, 2));

    buffer.insert_text("\n");

    assert!(
        buffer.wrap_cache.is_none(),
        "newline must not create a wrap cache by itself"
    );
    Ok(())
}

#[test]
fn replace_mode_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*replace-newline-wrap-edit*",
        vec!["hello".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 2));

    buffer.replace_mode_text("\n");

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn open_line_below_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*open-line-wrap-edit*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 1));

    buffer.open_line_below();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn same_line_replace_keeps_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*replace-range-wrap-edit*",
        vec!["hello".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));

    buffer.replace_range(
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 0)),
        "    ",
    );

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn undo_newline_wrap_cache_matches_cold_rebuild() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*undo-newline-wrap-edit*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 3));
    buffer.insert_text("\n");
    buffer.record_undo_snapshot();
    buffer.undo();

    assert_eq!(buffer.line_count(), 2);
    match buffer.wrap_cache.as_ref() {
        None => {}
        Some(_) => assert_wrap_cache_matches_cold_build(buffer, 8, 4)?,
    }
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
fn sync_visible_buffer_layouts_counts_markdown_pretty_image_rows_for_scrolloff()
-> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 3.0,
        headerline_lines: Vec::new(),
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
    let mut text = format!("![red](data:image/png;base64,{png})\n");
    for index in 1..80 {
        text.push_str(&format!("line {index}\n"));
    }
    let buffer_id = install_markdown_test_buffer(&mut state, "*pretty-image-scrolloff*", &text)?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(4, 0));

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
    let wrap_cols = wrap_columns_for_width(render_width, cell_width);
    let text_width_px = (wrap_cols as i32 * cell_width).max(1) as u32;
    let pretty_paint = markdown_pretty_paint_plan(
        buffer,
        &*user_library,
        MarkdownPrettyPaintArgs {
            visible_start: 0,
            visible_end: buffer.line_count().max(1),
            visual_selection: None,
            input_mode: InputMode::Normal,
            pane_width_px: text_width_px,
            line_height,
        },
    );
    let image_rows = pretty_paint
        .images
        .get(&0)
        .map(|image| image.rows())
        .ok_or_else(|| "pretty image did not decode for scroll fixture".to_owned())?;
    assert!(
        image_rows > 1,
        "fixture image should occupy multiple visual rows, got {image_rows}"
    );
    let expected_scrolloff = 3usize.min(layout.visible_rows.saturating_sub(1) / 2);
    assert!(expected_scrolloff > 1);
    let cursor_body_row = pretty_cursor_body_row(
        buffer,
        rect,
        &*user_library,
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
    )
    .ok_or_else(|| "cursor went off screen before scrolloff".to_owned())?;
    assert!(
        cursor_body_row >= expected_scrolloff,
        "cursor visual row {cursor_body_row} is above scrolloff {expected_scrolloff}"
    );
    assert!(
        cursor_body_row
            <= layout
                .visible_rows
                .saturating_sub(1)
                .saturating_sub(expected_scrolloff),
        "cursor visual row {cursor_body_row} is below scrolloff in {} visible rows",
        layout.visible_rows
    );
    Ok(())
}

#[test]
fn sync_visible_buffer_layouts_reuses_headerline_snapshot_while_typing() -> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typing-headerline-cache*",
        vec!["alpha".to_owned()],
    )?;

    let before = user_library.headerline_call_count();
    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;
    let after_first = user_library.headerline_call_count();
    assert!(after_first > before);

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text("!");
    }
    state.last_text_input_at = Some(Instant::now());
    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;
    assert_eq!(user_library.headerline_call_count(), after_first);
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
fn input_field_wrap_keeps_words_intact() {
    let mut input = InputField::new("> ");
    input.set_text("prefix text Please see the screenshot of this input");
    let rows = input.wrapped_visual_rows(28);

    assert!(
        !rows.iter().any(|row| row == "Pl" || row == "ease"),
        "rows: {rows:?}"
    );
    assert!(
        rows.windows(2)
            .all(|pair| { !(pair[0].ends_with("Pl") && pair[1].starts_with("ease")) }),
        "rows: {rows:?}"
    );
}

#[test]
fn block_cursor_text_overlay_positions_multibyte_cursor_text() {
    let line = "aéz";
    let char_map = LineCharMap::new(line);
    let overlay = block_cursor_text_overlay(CursorOverlayQuery {
        x: 24,
        line,
        char_map: &char_map,
        segment: LineWrapSegment {
            start_col: 0,
            end_col: 3,
        },
        line_index: 0,
        cursor: TextPoint::new(0, 1),
        color: Some(Color::RGB(1, 2, 3)),
        cell_width: 8,
    })
    .expect("cursor on a multibyte character should produce an overlay");

    assert_eq!(overlay.draw_x, 32);
    assert_eq!(overlay.text, "é");
    assert_eq!(overlay.color, Color::RGB(1, 2, 3));
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
            capture_name: Arc::from("source_file"),
            theme_token: Arc::from("syntax.source"),
        },
        LineSyntaxSpan {
            start: 0,
            end: 3,
            capture_name: Arc::from("keyword"),
            theme_token: Arc::from("syntax.keyword"),
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

fn focus_input_normal_mode(state: &mut ShellState, buffer_id: BufferId) -> Result<(), String> {
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
    Ok(())
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

fn plugin_section_lines(buffer: &ShellBuffer, name: &str) -> Result<Vec<String>, String> {
    let state = buffer
        .plugin_sections()
        .ok_or_else(|| "plugin section state missing".to_owned())?;
    let index = state
        .section_index_by_name(name)
        .ok_or_else(|| format!("section `{name}` missing"))?;
    if index == 0 {
        return Ok((0..buffer.text.line_count())
            .filter_map(|line_index| buffer.text.line(line_index))
            .collect());
    }
    let pane = state
        .attached_section(index)
        .ok_or_else(|| format!("attached section `{name}` missing"))?;
    Ok((0..pane.line_count())
        .map(|line_index| pane.text.line(line_index).unwrap_or_default())
        .collect())
}

fn install_user_acp_test_buffer(
    state: &mut ShellState,
    input_text: &str,
) -> Result<BufferId, String> {
    let buffer_id = install_user_plugin_buffer(state, "*acp*", user::acp::ACP_BUFFER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.init_acp_view("Test ACP");
        let _ = buffer.focus_acp_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .set_text(input_text);
    }
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

const PRETTY_CACHE_FIXTURE: &str = "# Title\n- item\nplain\n";

fn markdown_pretty_paint_args(buffer: &ShellBuffer) -> MarkdownPrettyPaintArgs {
    MarkdownPrettyPaintArgs {
        visible_start: 0,
        visible_end: buffer.line_count().max(1),
        visual_selection: None,
        input_mode: InputMode::Normal,
        pane_width_px: 640,
        line_height: 16,
    }
}

fn park_cursor_on_plain_pretty_line(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 0));
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_reuses_plan_for_same_revision() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-cache-revision*", PRETTY_CACHE_FIXTURE)?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let args = markdown_pretty_paint_args(buffer);
    let first = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let first_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing cached plan after first paint")?;
    let second = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let second_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing cached plan after second paint")?;
    assert!(
        std::sync::Arc::ptr_eq(&first_plan, &second_plan),
        "same revision should reuse MarkdownPrettyPlan"
    );
    assert_eq!(first.text_overrides, second.text_overrides);
    let heading = first
        .text_overrides
        .get(&0)
        .ok_or("heading Pretty override missing")?;
    assert!(
        heading.contains("Title") && !heading.starts_with("# "),
        "heading should conceal markers: {heading:?}"
    );
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_rebuilds_after_edit() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-cache-edit*", PRETTY_CACHE_FIXTURE)?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let before_plan = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let args = markdown_pretty_paint_args(buffer);
        let paint = markdown_pretty_paint_plan(buffer, &*user_library, args);
        let plan = markdown_pretty::last_cached_pretty_plan(buffer)
            .ok_or("missing cached plan before edit")?;
        (paint.text_overrides, plan)
    };
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 7));
        buffer.insert_text("!");
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let args = markdown_pretty_paint_args(buffer);
    let after = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let after_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing cached plan after edit")?;
    assert!(!std::sync::Arc::ptr_eq(&before_plan.1, &after_plan));
    assert_ne!(before_plan.0, after.text_overrides);
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_cursor_anti_conceal_uses_source() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*pretty-anti-conceal-cursor*",
        "# Title\n- item\n",
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    assert!(
        !paint.text_overrides.contains_key(&0),
        "cursor line should paint Markdown Raw: {:?}",
        paint.text_overrides
    );
    assert!(
        paint.text_overrides.contains_key(&1),
        "non-cursor Pretty lines should still override: {:?}",
        paint.text_overrides
    );
    let plan = markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing cached plan")?;
    let reused =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let reused_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing reused plan")?;
    assert!(std::sync::Arc::ptr_eq(&plan, &reused_plan));
    assert_eq!(paint.text_overrides, reused.text_overrides);
    Ok(())
}
