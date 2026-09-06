use super::*;

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
fn vim_search_entries_trim_whitespace_from_labels() {
    let buffer = TextBuffer::from_text("alpha\n   split here   \nbeta\n");
    let data = vim_search_entries(&buffer.snapshot(), VimSearchDirection::Forward, "split");

    assert_eq!(data.entries.len(), 1);
    assert_eq!(data.entries[0].item.label(), "split here");
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

    focus_input_normal_mode(&mut state, buffer_id)?;
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
fn acp_input_field_visual_line_delete_removes_selected_lines() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta\ngamma")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "gamma");
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
                    syntax_spans: Vec::new(),
                    row_fill: None,
                    gutter: false,
                    align: AcpChatAlign::Full,
                    bubble: false,
                    bubble_group: 0,
                }),
                AcpRenderedLine::Text(AcpRenderedTextLine {
                    prefix: Vec::new(),
                    text: "beta".to_owned(),
                    text_role: AcpColorRole::Default,
                    syntax_spans: Vec::new(),
                    row_fill: None,
                    gutter: false,
                    align: AcpChatAlign::Full,
                    bubble: false,
                    bubble_group: 0,
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
        "prompt".chars().count()
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
fn vim_open_line_below_before_typescript_closing_object_dedents_to_sibling_indent()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("typescript")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typescript-vim-open-line*",
        vec![
            ";".to_owned(),
            "export const Endpoints = (builder: EndpointBuilder<any, any, any>) => ({"
                .to_owned(),
            "  getOutdoorTrackingHistoryByCustomer: builder.query<DashboardTrackingHistory[], TrackingHistoryAttributes, DashboardTrackingHistoryDto[]>({".to_owned(),
            "    query: (args: TrackingHistoryAttributes) => `outdoordashboard/trackingactivity/${args.customerId}?days=${args.days}`,".to_owned(),
            "    transformResponse: (response: DashboardTrackingHistoryDto[]) => toDashboardTrackingHistorySummaries(response),".to_owned(),
            "    transformErrorResponse: (response: { status: string | number }, _meta, _arg) => response.status,".to_owned(),
            "    providesTags: [{ type: HOURLY_TAG, id: 'LIST' }],".to_owned(),
            "    keepUnusedDataFor: 300".to_owned(),
            "  }),".to_owned(),
            "});".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("typescript".to_owned()));
        buffer.set_cursor(TextPoint::new(8, 2));
    }

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("open-line-below"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(9).as_deref(), Some("  "));
    assert_eq!(buffer.cursor_point(), TextPoint::new(9, 2));
    Ok(())
}

#[test]
fn vim_open_line_below_after_typescript_outer_object_opener_uses_sibling_indent()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("typescript")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typescript-vim-open-top-level-entry*",
        vec![
            ";".to_owned(),
            "export const Endpoints = (builder: EndpointBuilder<any, any, any>) => ({"
                .to_owned(),
            "  getOutdoorTrackingHistoryByCustomer: builder.query<DashboardTrackingHistory[], TrackingHistoryAttributes, DashboardTrackingHistoryDto[]>({".to_owned(),
            "    query: (args: TrackingHistoryAttributes) => `outdoordashboard/trackingactivity/${args.customerId}?days=${args.days}`,".to_owned(),
            "    transformResponse: (response: DashboardTrackingHistoryDto[]) => toDashboardTrackingHistorySummaries(response),".to_owned(),
            "    transformErrorResponse: (response: { status: string | number }, _meta, _arg) => response.status,".to_owned(),
            "    providesTags: [{ type: HOURLY_TAG, id: 'LIST' }],".to_owned(),
            "    keepUnusedDataFor: 300".to_owned(),
            "  }),".to_owned(),
            "});".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("typescript".to_owned()));
        buffer.set_cursor(TextPoint::new(1, 0));
    }

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(2).as_deref(), Some("  "));
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 2));
    Ok(())
}

#[test]
fn vim_repeat_search_preserves_forward_and_backward_bindings() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*vim-search-repeat*",
        vec![
            "alpha".to_owned(),
            "beta".to_owned(),
            "alpha".to_owned(),
            "beta".to_owned(),
            "alpha".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));

    run_vim_search(&mut state.runtime, VimSearchDirection::Forward, "alpha")?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 0)
    );

    repeat_vim_search(&mut state.runtime, true)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(0, 0)
    );
    assert!(matches!(
        shell_ui(&state.runtime)?.vim().last_search,
        Some(LastSearch {
            direction: VimSearchDirection::Forward,
            ..
        })
    ));

    repeat_vim_search(&mut state.runtime, false)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 0)
    );
    assert!(matches!(
        shell_ui(&state.runtime)?.vim().last_search,
        Some(LastSearch {
            direction: VimSearchDirection::Forward,
            ..
        })
    ));
    Ok(())
}

#[test]
fn vim_g_prefix_executes_workspace_keybinding() -> Result<(), String> {
    let mut state = state_with_user_library()?;
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
    // `g` is an exact binding and a prefix of longer sequences, so it waits in
    // the key-sequence resolver without starting the Vim g-prefix yet.
    assert_eq!(
        state.ui().map_err(|error| error.to_string())?.vim().pending,
        None
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
    let mut state = state_with_user_library()?;
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
    // `g z` is a proper prefix of `g z z`, so the resolver keeps waiting
    // without firing anything or starting the Vim g-prefix.
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(ui.vim().pending, None);
    assert_eq!(ui.vim().pending_change_prefix, None);

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
    let state = state_with_user_library()?;

    let write_matches = vim_command_line_completion_matches(&state.runtime, "wr");
    assert!(write_matches.contains(&"write".to_owned()));

    let buffer_matches = vim_command_line_completion_matches(&state.runtime, "bd");
    assert!(buffer_matches.contains(&"bd".to_owned()));
    assert!(buffer_matches.contains(&"bdelete".to_owned()));
    Ok(())
}

#[test]
fn execute_vim_command_line_split_alias_splits_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    execute_vim_command_line(&mut state.runtime, "split")?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 2);
    Ok(())
}

#[test]
fn execute_vim_command_line_commands_alias_opens_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    execute_vim_command_line(&mut state.runtime, "commands")?;
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    Ok(())
}

#[test]
fn leader_space_o_b_opens_browser_from_normal_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .handle_text_input(" ")
        .map_err(|error| error.to_string())?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);

    state
        .handle_text_input("b")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let browser_buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_ne!(browser_buffer_id, original_buffer_id);
    assert_eq!(ui.pane_count(), 2);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(matches!(
        shell_buffer(&state.runtime, browser_buffer_id)?.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
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
fn gcc_toggles_prefix_comment_styles() -> Result<(), String> {
    for (language_id, original, commented) in [
        ("clojure", "  (inc value)", "  ; (inc value)"),
        ("latex", "  \\section{Intro}", "  % \\section{Intro}"),
        ("vim", "  set number", "  \" set number"),
    ] {
        let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
        let mut state =
            ShellState::new_with_user_library(default_error_log_path(), false, user_library)
                .map_err(|error| error.to_string())?;
        let buffer_id = install_text_test_buffer(
            &mut state,
            &format!("*{language_id}-comment-line*"),
            vec![original.to_owned()],
        )?;
        shell_buffer_mut(&mut state.runtime, buffer_id)?
            .set_language_id(Some(language_id.to_owned()));
        shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(commented),
            "expected `{language_id}` to use `{commented}`",
        );

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(original),
            "expected `{language_id}` to restore the original line",
        );
    }

    Ok(())
}

#[test]
fn gcc_toggles_block_comment_styles() -> Result<(), String> {
    for (language_id, original, commented) in [
        ("css", "  color: red;", "  /* color: red; */"),
        ("html", "  <div>volt</div>", "  <!-- <div>volt</div> -->"),
        (
            "json",
            "  \"name\": \"volt\",",
            "  /* \"name\": \"volt\", */",
        ),
        ("markdown", "  - item", "  <!-- - item -->"),
        ("xml", "  <tag/>", "  <!-- <tag/> -->"),
    ] {
        let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
        let mut state =
            ShellState::new_with_user_library(default_error_log_path(), false, user_library)
                .map_err(|error| error.to_string())?;
        let buffer_id = install_text_test_buffer(
            &mut state,
            &format!("*{language_id}-block-comment-line*"),
            vec![original.to_owned()],
        )?;
        shell_buffer_mut(&mut state.runtime, buffer_id)?
            .set_language_id(Some(language_id.to_owned()));
        shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(commented),
            "expected `{language_id}` to use `{commented}`",
        );

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(original),
            "expected `{language_id}` to restore the original line",
        );
    }

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
fn visual_move_down_reorders_selected_lines_and_keeps_visual_selection() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-move-down*",
        vec![
            "fn main() {".to_owned(),
            "    if ready {".to_owned(),
            "        alpha();".to_owned(),
            "    }".to_owned(),
            "}".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(2, 0), VisualSelectionKind::Line);

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-move-down"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("fn main() {"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    if ready {"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("    }"));
    assert_eq!(buffer.text.line(3).as_deref(), Some("    alpha();"));
    assert_eq!(buffer.text.line(4).as_deref(), Some("}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 0));
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(ui.vim().visual_anchor, Some(TextPoint::new(3, 0)));
    Ok(())
}

#[test]
fn visual_move_up_reorders_selected_lines_and_reindents() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-move-up*",
        vec![
            "fn main() {".to_owned(),
            "    if ready {".to_owned(),
            "    }".to_owned(),
            "    alpha();".to_owned(),
            "}".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(3, 0));
    }
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(3, 0), VisualSelectionKind::Line);

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-move-up"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("fn main() {"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    if ready {"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("        alpha();"));
    assert_eq!(buffer.text.line(3).as_deref(), Some("    }"));
    assert_eq!(buffer.text.line(4).as_deref(), Some("}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 0));
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(ui.vim().visual_anchor, Some(TextPoint::new(2, 0)));
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
