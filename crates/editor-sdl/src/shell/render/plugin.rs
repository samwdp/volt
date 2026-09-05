fn plugin_section_tree_layout(
    state: &PluginSectionBufferState,
    tree: &PluginBufferLayout,
    bounds: PixelRect,
    cell_width: i32,
    line_height: i32,
    gap: i32,
) -> Option<PluginSectionLayout> {
    let line_height = line_height.max(1);
    let split = plugin_layout_to_split_node(state, tree, line_height)?;
    let leaves = layout_split_tree(bounds, &split, gap.max(0) as u32);
    let mut panes = vec![
        TextPaneLayout {
            rect: Rect::new(bounds.x, bounds.y, 0, 0),
            visible_rows: 1,
            wrap_cols: 1,
        };
        state.section_count()
    ];
    for (index, leaf) in leaves {
        let Some(pane) = panes.get_mut(index) else {
            continue;
        };
        let chrome = plugin_section_panel_chrome_height(state.section_title(index), line_height);
        let inner_height = (leaf.height as i32).saturating_sub(chrome).max(line_height);
        let wrap_cols = overlay_text_columns(leaf.width.saturating_sub(20), 0, cell_width);
        *pane = TextPaneLayout {
            rect: Rect::new(leaf.x, leaf.y, leaf.width, leaf.height),
            visible_rows: (inner_height / line_height).max(1) as usize,
            wrap_cols,
        };
    }
    Some(PluginSectionLayout { panes })
}

fn plugin_layout_to_split_node(
    state: &PluginSectionBufferState,
    layout: &PluginBufferLayout,
    line_height: i32,
) -> Option<SplitNode> {
    let children = layout
        .children()
        .iter()
        .filter_map(|child| plugin_layout_node_to_child(state, child, line_height))
        .collect::<Vec<_>>();
    (!children.is_empty()).then_some(SplitNode::new(plugin_layout_axis(layout.axis()), children))
}

fn plugin_layout_node_to_child(
    state: &PluginSectionBufferState,
    node: &PluginBufferLayoutNode,
    line_height: i32,
) -> Option<SplitChild> {
    match node {
        PluginBufferLayoutNode::Section { name, weight } => {
            let index = state.section_index_by_name(name.as_str())?;
            let chrome =
                plugin_section_panel_chrome_height(state.section_title(index), line_height);
            let min_rows = state.section_min_rows(index).unwrap_or(1).max(1) as i32;
            Some(SplitChild::leaf(
                index,
                (*weight).max(1),
                (chrome + min_rows * line_height).max(1) as u32,
            ))
        }
        PluginBufferLayoutNode::Split {
            axis,
            weight,
            children,
        } => {
            let mapped = children
                .iter()
                .filter_map(|child| plugin_layout_node_to_child(state, child, line_height))
                .collect::<Vec<_>>();
            if mapped.is_empty() {
                return None;
            }
            let min_px = mapped.iter().map(|child| child.min_px).sum::<u32>();
            Some(SplitChild::node(
                SplitNode::new(plugin_layout_axis(*axis), mapped),
                (*weight).max(1),
                min_px.max(1),
            ))
        }
    }
}

fn plugin_layout_axis(axis: PluginBufferLayoutAxis) -> SplitAxis {
    match axis {
        PluginBufferLayoutAxis::Rows => SplitAxis::Rows,
        PluginBufferLayoutAxis::Columns => SplitAxis::Columns,
    }
}

pub(super) fn render_plugin_section_buffer_body(
    target: &mut DrawTarget<'_>,
    draw: PluginSectionDraw<'_>,
    palette: BufferBodyPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let PluginSectionDraw {
        buffer,
        view_state: base_view_state,
        pane: PaneSlot { rect, active },
        layout,
        visual_selection,
        yank_flash,
        input_mode,
    } = draw;
    let BufferBodyPalette {
        theme_registry,
        base_background,
        foreground,
        muted,
        border_color,
        selection,
        yank_flash_color,
        cursor,
        cursor_roundness,
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let Some(state) = buffer.plugin_sections() else {
        return Ok(());
    };
    let Some(section_layout) =
        plugin_section_buffer_layout(buffer, rect, layout, cell_width, line_height)
    else {
        return Ok(());
    };
    let panel_background = buffer_section_panel_background(base_background);
    let header_background = buffer_section_header_background(theme_registry, panel_background);
    let active_border = theme_color(theme_registry, TOKEN_STATUSLINE_ACTIVE, cursor);
    for (index, pane_layout) in section_layout.panes.iter().copied().enumerate() {
        let pane_active = active && state.active_section == index;
        let pane_visual_selection = if state.active_section == index {
            visual_selection
        } else {
            None
        };
        let pane_yank_flash = if state.active_section == index {
            yank_flash
        } else {
            None
        };
        let (text, scroll_row, cursor_point, title, pane_mode, pane_syntax) = if index == 0 {
            (
                &buffer.text,
                base_view_state.scroll_row,
                (state.active_section == 0).then_some(base_view_state.cursor),
                state.base_title.as_str(),
                if state.base_writable {
                    input_mode
                } else {
                    InputMode::Normal
                },
                Some(&buffer.syntax_lines),
            )
        } else {
            let Some(pane) = state.attached_section(index) else {
                continue;
            };
            (
                &pane.text,
                pane.scroll_row,
                (state.active_section == index).then_some(pane.cursor()),
                pane.title.as_str(),
                if pane.writable {
                    input_mode
                } else {
                    InputMode::Normal
                },
                Some(&pane.syntax_lines),
            )
        };
        render_text_panel(
            target,
            TextPanelDraw {
                text,
                syntax_lines: pane_syntax,
                scroll_row,
                cursor_point,
                pane_active,
                pane_layout,
                title,
                visual_selection: pane_visual_selection,
                yank_flash: pane_yank_flash,
                input_mode: pane_mode,
            },
            PanelPalette {
                theme_registry,
                panel_background,
                header_background,
                foreground,
                muted,
                border_color,
                active_border,
                selection,
                yank_flash_color,
                cursor,
                cursor_roundness,
            },
            CellMetrics {
                cell_width,
                line_height,
            },
        )?;
    }
    Ok(())
}

pub(super) fn render_text_panel(
    target: &mut DrawTarget<'_>,
    draw: TextPanelDraw<'_>,
    palette: PanelPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let TextPanelDraw {
        text,
        syntax_lines,
        scroll_row,
        cursor_point,
        pane_active,
        pane_layout,
        title,
        visual_selection,
        yank_flash,
        input_mode,
    } = draw;
    let PanelPalette {
        theme_registry,
        panel_background,
        header_background,
        foreground,
        muted,
        border_color,
        active_border,
        selection,
        yank_flash_color,
        cursor,
        cursor_roundness,
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let window_effects = current_window_effect_settings(theme_registry);
    let corner_radius = shared_corner_radius(theme_registry);
    let rect = pane_layout.rect;
    let border = if pane_active {
        active_border
    } else {
        border_color
    };
    fill_window_panel_frame(
        target,
        rect,
        corner_radius,
        border,
        panel_background,
        window_effects,
    )?;
    let header_height = text_panel_header_height(title, line_height.max(1));
    if header_height > 0 {
        let header_rect = PixelRectToRect::rect(
            rect.x() + 1,
            rect.y() + 1,
            rect.width().saturating_sub(2),
            header_height as u32,
        );
        let inner_radius = corner_radius.saturating_sub(1);
        fill_window_top_header_band(
            target,
            header_rect,
            inner_radius,
            header_background,
            window_effects,
        )?;
        draw_text(target, rect.x() + 10, rect.y() + 6, title, foreground)?;
    }
    let body_x = rect.x() + 10;
    let body_y = if header_height > 0 {
        rect.y() + header_height + 6
    } else {
        rect.y() + 10
    };
    let mut visual_row = 0usize;
    let line_count = text.line_count();
    let mut cursor_screen: Option<(usize, usize)> = None;
    for line_index in scroll_row.min(line_count.saturating_sub(1))..line_count {
        let line = text.line(line_index).unwrap_or_default();
        let line_len = text.line_len_chars(line_index).unwrap_or(0);
        let char_map = LineCharMap::new(&line);
        let selection_range = visual_selection.and_then(|selection_state| {
            selection_columns_for_visual(selection_state, line_index, line_len)
        });
        let yank_range = yank_flash.and_then(|selection_state| {
            selection_columns_for_visual(selection_state, line_index, line_len)
        });
        let segments = wrap_line_segments(&char_map, pane_layout.wrap_cols, pane_layout.wrap_cols);
        for segment in &segments {
            if visual_row >= pane_layout.visible_rows {
                break;
            }
            let y = body_y + visual_row as i32 * line_height;
            if let Some((selection_start, selection_end)) = selection_range {
                let start = selection_start.max(segment.start_col);
                let end = selection_end.min(segment.end_col);
                if start < end {
                    let start_display = char_map.display_cols_between(segment.start_col, start);
                    let width_display = char_map.display_cols_between(start, end);
                    fill_selection_highlight(
                        target,
                        body_x + (start_display as i32 * cell_width),
                        y,
                        (width_display as i32 * cell_width) as u32,
                        line_height.max(1) as u32,
                        cursor_roundness,
                        selection,
                    )?;
                }
            }
            if let Some((selection_start, selection_end)) = yank_range {
                let start = selection_start.max(segment.start_col);
                let end = selection_end.min(segment.end_col);
                if start < end {
                    let start_display = char_map.display_cols_between(segment.start_col, start);
                    let width_display = char_map.display_cols_between(start, end);
                    fill_selection_highlight(
                        target,
                        body_x + (start_display as i32 * cell_width),
                        y,
                        (width_display as i32 * cell_width) as u32,
                        line_height.max(1) as u32,
                        cursor_roundness,
                        yank_flash_color,
                    )?;
                }
            }
            if cursor_screen.is_none()
                && let Some(cursor_point) = cursor_point
                && cursor_point.line == line_index
                && char_map.cursor_anchor_col(cursor_point.column) >= segment.start_col
                && char_map.cursor_anchor_col(cursor_point.column) <= segment.end_col
            {
                let cursor_col = char_map.cursor_anchor_col(cursor_point.column);
                cursor_screen = Some((
                    visual_row,
                    char_map.display_cols_between(segment.start_col, cursor_col),
                ));
            }
            draw_buffer_text(
                target,
                BufferTextRun {
                    x: body_x,
                    y,
                    line: &line,
                    segment: *segment,
                    char_map: &char_map,
                    line_syntax_spans: syntax_lines
                        .and_then(|lines| lines.get(&line_index))
                        .map(Vec::as_slice),
                    default_color: foreground,
                    cell_width,
                },
                theme_registry,
            )?;
            visual_row = visual_row.saturating_add(1);
        }
        if visual_row >= pane_layout.visible_rows {
            break;
        }
    }
    if let Some((cursor_row, cursor_col)) = cursor_screen
        && pane_active
        && cursor_row < pane_layout.visible_rows
    {
        let cursor_width = match input_mode {
            InputMode::Normal | InputMode::Visual => cell_width.max(2) as u32,
            InputMode::Insert | InputMode::Replace => (cell_width / 4).max(2) as u32,
        };
        fill_rounded_rect(
            target,
            PixelRectToRect::rect(
                body_x + (cursor_col as i32 * cell_width),
                body_y + cursor_row as i32 * line_height,
                cursor_width,
                line_height.max(2) as u32,
            ),
            cursor_roundness,
            cursor,
        )?;
    } else if line_count == 0 {
        draw_text(target, body_x, body_y, "", muted)?;
    }
    Ok(())
}

pub(super) fn render_input_panel(
    target: &mut DrawTarget<'_>,
    draw: InputPanelDraw<'_>,
    palette: PanelPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let InputPanelDraw {
        input,
        pane_active,
        pane_layout,
        input_mode,
        window_effects,
        corner_radius,
    } = draw;
    let PanelPalette {
        panel_background,
        foreground,
        muted,
        border_color,
        active_border,
        selection,
        cursor,
        cursor_roundness,
        ..
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let rect = pane_layout.rect;
    let border = if pane_active {
        active_border
    } else {
        border_color
    };
    fill_window_panel_frame(
        target,
        rect,
        corner_radius,
        border,
        panel_background,
        window_effects,
    )?;
    let input_x = rect.x() + INPUT_PANEL_VERTICAL_PADDING;
    let input_y = rect.y() + INPUT_PANEL_VERTICAL_PADDING;
    let prompt = input.prompt();
    let prompt_len = prompt.chars().count();
    let prompt_padding = " ".repeat(prompt_len);
    let available_input_cols = pane_layout.wrap_cols.max(prompt_len.saturating_add(1));
    if pane_active && matches!(input_mode, InputMode::Visual) {
        for (row, start_col, end_col) in
            input.selection_visual_ranges(VisualSelectionKind::Character, available_input_cols)
        {
            fill_selection_highlight(
                target,
                input_x + ((prompt_len + start_col) as i32 * cell_width),
                input_y + row as i32 * line_height,
                ((end_col.saturating_sub(start_col)) as i32 * cell_width.max(1)) as u32,
                line_height.max(1) as u32,
                cursor_roundness,
                selection,
            )?;
        }
    }
    let max_visible_rows = visible_input_text_rows(pane_layout.rect.height() as i32, line_height);
    let (visible_rows, first_visible_row) =
        input.visible_wrapped_visual_rows(available_input_cols, max_visible_rows);
    if input.text().is_empty() {
        if let Some(placeholder) = input.placeholder() {
            let line = format!("{prompt}{placeholder}");
            draw_text(target, input_x, input_y, &line, muted)?;
        } else {
            draw_text(target, input_x, input_y, prompt, foreground)?;
        }
    } else {
        for (index, line) in visible_rows.into_iter().enumerate() {
            let prefix = if index == 0 { prompt } else { &prompt_padding };
            let rendered = format!("{prefix}{line}");
            draw_text(
                target,
                input_x,
                input_y + index as i32 * line_height,
                &rendered,
                foreground,
            )?;
        }
    }
    if pane_active && matches!(input_mode, InputMode::Insert | InputMode::Replace) {
        let (input_row, col_in_visual_row) = input.cursor_visual_row_col(available_input_cols);
        let input_row = input_row.saturating_sub(first_visible_row);
        let input_col = prompt_len + col_in_visual_row;
        let cursor_width = (cell_width / 4).max(2) as u32;
        fill_rounded_rect(
            target,
            PixelRectToRect::rect(
                input_x + (input_col as i32 * cell_width),
                input_y + input_row as i32 * line_height,
                cursor_width,
                line_height.max(2) as u32,
            ),
            cursor_roundness,
            cursor,
        )?;
    } else if pane_active && matches!(input_mode, InputMode::Normal | InputMode::Visual) {
        let cursor_char = input.cursor_char();
        let char_count = input.char_count();
        if char_count > 0 {
            let cursor_index = cursor_char.min(char_count.saturating_sub(1));
            let (input_row, col_in_visual_row) =
                input.visual_row_col_for_cursor(cursor_index, available_input_cols);
            let input_row = input_row.saturating_sub(first_visible_row);
            fill_rounded_rect(
                target,
                PixelRectToRect::rect(
                    input_x + ((prompt_len + col_in_visual_row) as i32 * cell_width),
                    input_y + input_row as i32 * line_height,
                    cell_width.max(1) as u32,
                    line_height.max(2) as u32,
                ),
                cursor_roundness,
                cursor,
            )?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AcpPaneLayout {
    pub(super) rect: Rect,
    pub(super) visible_rows: usize,
    pub(super) wrap_cols: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AcpBufferLayout {
    pub(super) plan: AcpPaneLayout,
    pub(super) output: AcpPaneLayout,
    pub(super) input: TextPaneLayout,
    pub(super) footer: TextPaneLayout,
}

pub(super) fn acp_pane_body_visible_rows(rect_height: u32, line_height: i32) -> usize {
    let line_height = line_height.max(1);
    let header_height = (line_height + 10).max(line_height);
    let body_height = rect_height as i32 - header_height - 4;
    (body_height.max(line_height) / line_height).max(1) as usize
}

pub(super) fn acp_buffer_layout(
    buffer: &ShellBuffer,
    rect: Rect,
    layout: BufferFooterLayout,
    cell_width: i32,
    line_height: i32,
) -> Option<AcpBufferLayout> {
    let state = buffer.acp_state.as_ref()?;
    let line_height = line_height.max(1);
    let panel_x = rect.x() + 8;
    let panel_width = rect.width().saturating_sub(16);
    let gap = 8i32;
    let body_width = panel_width.saturating_sub(20);
    let wrap_cols = overlay_text_columns(body_width, 0, cell_width);
    let desired_input_rows = state
        .input
        .visual_line_count(wrap_cols)
        .clamp(1, MAX_VISIBLE_INPUT_TEXT_ROWS);
    let footer_line_count = state.footer_pane.line_count().max(1);
    let footer_rows = state
        .footer_pane
        .min_rows
        .unwrap_or(footer_line_count)
        .max(footer_line_count);
    let input_chrome = input_panel_chrome_height();
    let footer_chrome = text_panel_chrome_height("", line_height);
    let plan_chrome = text_panel_chrome_height("Plan", line_height);
    let output_chrome = text_panel_chrome_height("Output", line_height);
    let total_height = layout.pane_bottom.saturating_sub(layout.body_y).max(
        plan_chrome + output_chrome + input_chrome + footer_chrome + gap * 3 + line_height * 4,
    );
    let min_top_rows = 2i32;
    let max_input_rows = total_height
        .saturating_sub(
            plan_chrome
                + output_chrome
                + footer_chrome
                + footer_rows as i32 * line_height
                + input_chrome
                + gap * 3
                + min_top_rows * line_height,
        )
        .saturating_div(line_height.max(1))
        .max(1) as usize;
    let input_rows = desired_input_rows.min(max_input_rows);
    let bottom_reserved = input_chrome
        + input_rows as i32 * line_height
        + footer_chrome
        + footer_rows as i32 * line_height;
    let top_height = total_height.saturating_sub(bottom_reserved + gap * 3);
    let total_row_budget = ((top_height - plan_chrome - output_chrome).max(line_height * 2)
        / line_height)
        .max(2) as usize;
    let plan_target_rows = acp_pane_content_rows(&state.plan_pane, wrap_cols).clamp(1, 10);
    let plan_rows = plan_target_rows.min(total_row_budget.saturating_sub(1).max(1));
    let output_rows = total_row_budget.saturating_sub(plan_rows).max(1);
    let used_top_height = plan_chrome
        + output_chrome
        + gap
        + ((plan_rows.saturating_add(output_rows)) as i32 * line_height);
    let output_extra = top_height.saturating_sub(used_top_height);
    let plan_height = plan_chrome + plan_rows as i32 * line_height;
    let output_height = output_chrome + output_rows as i32 * line_height + output_extra;
    let input_height = input_chrome + input_rows as i32 * line_height;
    let footer_height = footer_chrome + footer_rows as i32 * line_height;
    let output_y = layout.body_y + plan_height + gap;
    let input_y = output_y + output_height + gap;
    let footer_y = input_y + input_height + gap;
    Some(AcpBufferLayout {
        plan: AcpPaneLayout {
            rect: Rect::new(panel_x, layout.body_y, panel_width, plan_height as u32),
            visible_rows: acp_pane_body_visible_rows(plan_height as u32, line_height),
            wrap_cols,
        },
        output: AcpPaneLayout {
            rect: Rect::new(panel_x, output_y, panel_width, output_height as u32),
            visible_rows: acp_pane_body_visible_rows(output_height as u32, line_height),
            wrap_cols,
        },
        input: TextPaneLayout {
            rect: Rect::new(panel_x, input_y, panel_width, input_height as u32),
            visible_rows: input_rows,
            wrap_cols,
        },
        footer: TextPaneLayout {
            rect: Rect::new(panel_x, footer_y, panel_width, footer_height as u32),
            visible_rows: footer_rows,
            wrap_cols,
        },
    })
}

pub(super) fn render_acp_buffer_body(
    target: &mut DrawTarget<'_>,
    draw: AcpBufferDraw<'_>,
    palette: BufferBodyPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let AcpBufferDraw {
        buffer,
        rect,
        layout,
        active,
        visual_selection,
        yank_flash,
        input_mode,
    } = draw;
    let BufferBodyPalette {
        theme_registry,
        base_background,
        foreground,
        muted,
        border_color,
        selection,
        yank_flash_color,
        cursor,
        cursor_roundness,
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let Some(state) = buffer.acp_state.as_ref() else {
        return Ok(());
    };
    let window_effects = current_window_effect_settings(theme_registry);
    let Some(acp_layout) = acp_buffer_layout(buffer, rect, layout, cell_width, line_height) else {
        return Ok(());
    };
    let panel_background = buffer_section_panel_background(base_background);
    let header_background = buffer_section_header_background(theme_registry, panel_background);
    let active_border = theme_color(theme_registry, TOKEN_STATUSLINE_ACTIVE, cursor);
    let active_pane = state.active_pane;
    let corner_radius = shared_corner_radius(theme_registry);

    render_acp_pane(
        target,
        AcpPaneDraw {
            pane: &state.plan_pane,
            pane_active: active_pane == AcpPane::Plan,
            pane_layout: acp_layout.plan,
            title: "Plan",
            shell_active: active,
            visual_selection: if active_pane == AcpPane::Plan {
                visual_selection
            } else {
                None
            },
            yank_flash: if active_pane == AcpPane::Plan {
                yank_flash
            } else {
                None
            },
            input_mode,
        },
        PanelPalette {
            theme_registry,
            panel_background,
            header_background,
            foreground,
            muted,
            border_color,
            active_border,
            selection,
            yank_flash_color,
            cursor,
            cursor_roundness,
        },
        CellMetrics {
            cell_width,
            line_height,
        },
    )?;
    render_acp_pane(
        target,
        AcpPaneDraw {
            pane: &state.output_pane,
            pane_active: active_pane == AcpPane::Output,
            pane_layout: acp_layout.output,
            title: &acp_output_header_title(state),
            shell_active: active,
            visual_selection: if active_pane == AcpPane::Output {
                visual_selection
            } else {
                None
            },
            yank_flash: if active_pane == AcpPane::Output {
                yank_flash
            } else {
                None
            },
            input_mode,
        },
        PanelPalette {
            theme_registry,
            panel_background,
            header_background,
            foreground,
            muted,
            border_color,
            active_border,
            selection,
            yank_flash_color,
            cursor,
            cursor_roundness,
        },
        CellMetrics {
            cell_width,
            line_height,
        },
    )?;
    render_input_panel(
        target,
        InputPanelDraw {
            input: &state.input,
            pane_active: active && active_pane == AcpPane::Input,
            pane_layout: acp_layout.input,
            input_mode,
            window_effects,
            corner_radius,
        },
        PanelPalette {
            theme_registry,
            panel_background,
            header_background,
            foreground,
            muted,
            border_color,
            active_border,
            selection,
            yank_flash_color,
            cursor,
            cursor_roundness,
        },
        CellMetrics {
            cell_width,
            line_height,
        },
    )?;
    render_text_panel(
        target,
        TextPanelDraw {
            text: &state.footer_pane.text,
            syntax_lines: Some(&state.footer_pane.syntax_lines),
            scroll_row: state.footer_pane.scroll_row,
            cursor_point: (active && active_pane == AcpPane::Footer)
                .then_some(state.footer_pane.cursor()),
            pane_active: active && active_pane == AcpPane::Footer,
            pane_layout: acp_layout.footer,
            title: "",
            visual_selection: if active_pane == AcpPane::Footer {
                visual_selection
            } else {
                None
            },
            yank_flash: if active_pane == AcpPane::Footer {
                yank_flash
            } else {
                None
            },
            input_mode: InputMode::Normal,
        },
        PanelPalette {
            theme_registry,
            panel_background,
            header_background,
            foreground,
            muted,
            border_color,
            active_border,
            selection,
            yank_flash_color,
            cursor,
            cursor_roundness,
        },
        CellMetrics {
            cell_width,
            line_height,
        },
    )?;
    Ok(())
}

pub(super) fn render_acp_pane(
    target: &mut DrawTarget<'_>,
    draw: AcpPaneDraw<'_>,
    palette: PanelPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let AcpPaneDraw {
        pane,
        pane_active,
        pane_layout,
        title,
        shell_active,
        visual_selection,
        yank_flash,
        input_mode,
    } = draw;
    let PanelPalette {
        theme_registry,
        panel_background,
        header_background,
        foreground,
        muted,
        border_color,
        active_border,
        selection,
        yank_flash_color,
        cursor,
        cursor_roundness,
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let window_effects = current_window_effect_settings(theme_registry);
    let corner_radius = acp_chat_corner_radius(theme_registry);
    let rect = pane_layout.rect;
    let border = if pane_active {
        active_border
    } else {
        border_color
    };
    fill_window_panel_frame(
        target,
        rect,
        corner_radius,
        border,
        panel_background,
        window_effects,
    )?;
    let header_height = (line_height + 10).max(line_height);
    let header_rect = PixelRectToRect::rect(
        rect.x() + 1,
        rect.y() + 1,
        rect.width().saturating_sub(2),
        header_height as u32,
    );
    // CONTEXT: keep ACP headers neutral like plugin sections. Blending selection
    // into the active header produced a washed-out top bar once opacity scaled
    // the alpha, and made the focused section look uniquely translucent.
    fill_window_top_header_band(
        target,
        header_rect,
        corner_radius.saturating_sub(1),
        header_background,
        window_effects,
    )?;
    draw_text(target, rect.x() + 12, rect.y() + 6, title, foreground)?;
    let body_x = rect.x() + 10;
    let body_y = rect.y() + header_height + 4;
    let body_width = rect.width().saturating_sub(20);
    let max_draw_rows = acp_pane_body_visible_rows(pane_layout.rect.height(), line_height)
        .min(pane_layout.visible_rows);
    let spinner_frame = acp_spinner_frame();
    let cursor_point = pane.cursor();
    let show_text_cursor = pane_active
        && shell_active
        && !matches!(input_mode, InputMode::Insert | InputMode::Replace);
    let mut global_visual_row = 0usize;
    let mut drawn_rows = 0usize;
    let scroll_top = pane.viewport_scroll_top();
    let mut painted_bubble_groups: Vec<u32> = Vec::new();
    for (line_index, rendered_line) in pane.render_lines.iter().enumerate() {
        if drawn_rows >= max_draw_rows {
            break;
        }
        match rendered_line {
            AcpRenderedLine::Text(line) => {
                let prefix_cols = acp_prefix_columns(&line.prefix, spinner_frame);
                let line_len = pane.line_len_chars(line_index);
                let selection_range = visual_selection.and_then(|selection_state| {
                    selection_columns_for_visual(selection_state, line_index, line_len)
                });
                let yank_range = yank_flash.and_then(|selection_state| {
                    selection_columns_for_visual(selection_state, line_index, line_len)
                });
                let segments = acp_rendered_text_segments(line, pane_layout.wrap_cols);
                let cursor_segment = acp_segment_index_for_column(&segments, cursor_point.column);
                let origin_x = acp_chat_origin_x(line, body_x, pane_layout.wrap_cols, cell_width);
                let bubble_width =
                    acp_chat_bubble_width_px(line, pane_layout.wrap_cols, cell_width, body_width);
                for (segment_index, segment) in segments.iter().enumerate() {
                    if global_visual_row < scroll_top {
                        global_visual_row = global_visual_row.saturating_add(1);
                        continue;
                    }
                    if drawn_rows >= max_draw_rows {
                        break;
                    }
                    let y = body_y + drawn_rows as i32 * line_height;
                    if line.bubble
                        && line.bubble_group != 0
                        && !painted_bubble_groups.contains(&line.bubble_group)
                    {
                        painted_bubble_groups.push(line.bubble_group);
                        let remaining = acp_bubble_remaining_rows(
                            pane,
                            line_index,
                            segment_index,
                            line.bubble_group,
                            pane_layout.wrap_cols,
                        );
                        let rows = remaining
                            .min(max_draw_rows.saturating_sub(drawn_rows))
                            .max(1);
                        let fill_role = line.row_fill.unwrap_or(match line.align {
                            AcpChatAlign::End => AcpColorRole::Accent,
                            _ => AcpColorRole::Muted,
                        });
                        let fill = blend_color(
                            panel_background,
                            acp_color(fill_role, theme_registry, foreground, muted, cursor),
                            match line.align {
                                AcpChatAlign::End => 0.24,
                                _ => 0.16,
                            },
                        );
                        fill_overlay_surface_rounded_rect(
                            target,
                            PixelRectToRect::rect(
                                origin_x.saturating_sub(6),
                                y,
                                bubble_width.saturating_add(12),
                                (rows as i32 * line_height).max(line_height) as u32,
                            ),
                            corner_radius.min(18),
                            fill,
                            window_effects,
                        )?;
                    }
                    let segment_x = origin_x + (prefix_cols as i32 * cell_width);
                    if let Some(fill_role) = line.row_fill.filter(|_| !line.bubble) {
                        let fill = blend_color(
                            panel_background,
                            acp_color(fill_role, theme_registry, foreground, muted, cursor),
                            0.22,
                        );
                        fill_overlay_surface_rounded_rect(
                            target,
                            PixelRectToRect::rect(body_x, y, body_width, line_height.max(1) as u32),
                            4,
                            fill,
                            window_effects,
                        )?;
                    }
                    if line.gutter {
                        fill_overlay_surface_rect(
                            target,
                            PixelRectToRect::rect(origin_x + 2, y, 2, line_height.max(1) as u32),
                            muted,
                            window_effects,
                        )?;
                    }
                    if let Some((selection_start, selection_end)) = selection_range {
                        let start = selection_start.max(segment.start_col);
                        let end = selection_end.min(segment.end_col);
                        if start < end {
                            fill_selection_highlight(
                                target,
                                segment_x
                                    + (start.saturating_sub(segment.start_col) as i32 * cell_width),
                                y,
                                (end.saturating_sub(start) as i32 * cell_width) as u32,
                                line_height.max(1) as u32,
                                cursor_roundness,
                                selection,
                            )?;
                        }
                    }
                    if let Some((selection_start, selection_end)) = yank_range {
                        let start = selection_start.max(segment.start_col);
                        let end = selection_end.min(segment.end_col);
                        if start < end {
                            fill_selection_highlight(
                                target,
                                segment_x
                                    + (start.saturating_sub(segment.start_col) as i32 * cell_width),
                                y,
                                (end.saturating_sub(start) as i32 * cell_width) as u32,
                                line_height.max(1) as u32,
                                cursor_roundness,
                                yank_flash_color,
                            )?;
                        }
                    }
                    if show_text_cursor
                        && cursor_point.line == line_index
                        && cursor_segment == segment_index
                    {
                        let cursor_x = origin_x
                            + (prefix_cols as i32 * cell_width)
                            + (cursor_point.column.saturating_sub(segment.start_col) as i32
                                * cell_width);
                        let cursor_width = match input_mode {
                            InputMode::Normal | InputMode::Visual => cell_width.max(2) as u32,
                            InputMode::Insert | InputMode::Replace => {
                                (cell_width / 4).max(2) as u32
                            }
                        };
                        fill_rounded_rect(
                            target,
                            PixelRectToRect::rect(
                                cursor_x,
                                y,
                                cursor_width,
                                line_height.max(2) as u32,
                            ),
                            cursor_roundness,
                            cursor,
                        )?;
                    }
                    if segment_index == 0 {
                        acp_draw_prefix_segments(
                            target,
                            AcpPrefixDraw {
                                x: origin_x,
                                y,
                                segments: &line.prefix,
                                spinner_frame,
                                theme_registry,
                                foreground,
                                muted,
                                accent: cursor,
                                cell_width,
                            },
                        )?;
                    }
                    let segment_text =
                        acp_slice_chars(&line.text, segment.start_col, segment.end_col);
                    let default_color =
                        acp_color(line.text_role, theme_registry, foreground, muted, cursor);
                    if line.syntax_spans.is_empty() {
                        draw_text(target, segment_x, y, &segment_text, default_color)?;
                    } else {
                        let char_map = LineCharMap::new(&line.text);
                        draw_buffer_text(
                            target,
                            BufferTextRun {
                                x: segment_x,
                                y,
                                line: &line.text,
                                segment: LineWrapSegment {
                                    start_col: segment.start_col,
                                    end_col: segment.end_col,
                                },
                                char_map: &char_map,
                                line_syntax_spans: Some(line.syntax_spans.as_slice()),
                                default_color,
                                cell_width,
                            },
                            theme_registry,
                        )?;
                    }
                    drawn_rows = drawn_rows.saturating_add(1);
                    global_visual_row = global_visual_row.saturating_add(1);
                }
            }
            AcpRenderedLine::Image(image) => {
                let image_rows = image.rows.max(1);
                if global_visual_row.saturating_add(image_rows) <= scroll_top {
                    global_visual_row = global_visual_row.saturating_add(image_rows);
                    continue;
                }
                if drawn_rows >= max_draw_rows {
                    break;
                }
                let remaining_rows = max_draw_rows.saturating_sub(drawn_rows);
                let image_rows_draw = image.rows.min(remaining_rows).max(1);
                let y = body_y + drawn_rows as i32 * line_height;
                let image_rect = PixelRectToRect::rect(
                    body_x,
                    y,
                    body_width,
                    (image_rows_draw as i32 * line_height).max(line_height) as u32,
                );
                fill_window_surface_rounded_rect(
                    target,
                    image_rect,
                    8,
                    adjust_color(
                        panel_background,
                        if is_dark_color(panel_background) {
                            10
                        } else {
                            -10
                        },
                    ),
                    window_effects,
                )?;
                draw_text(target, body_x + 8, y + 6, &image.label, muted)?;
                if let Some(decoded) = image.image.as_ref() {
                    let top = y + line_height;
                    let height = image_rect
                        .height()
                        .saturating_sub(line_height.max(1) as u32)
                        .saturating_sub(8);
                    if height > 0 {
                        let width = body_width.saturating_sub(12);
                        let draw_height = height;
                        draw_image(
                            target,
                            PixelRectToRect::rect(body_x + 6, top + 2, width, draw_height),
                            decoded.width,
                            decoded.height,
                            Arc::clone(&decoded.pixels),
                            Some(image_rect),
                        )?;
                    }
                }
                drawn_rows = drawn_rows.saturating_add(image_rows_draw);
                global_visual_row = global_visual_row.saturating_add(image_rows);
            }
            AcpRenderedLine::Spacer => {
                if global_visual_row < scroll_top {
                    global_visual_row = global_visual_row.saturating_add(1);
                    continue;
                }
                if drawn_rows >= max_draw_rows {
                    break;
                }
                drawn_rows = drawn_rows.saturating_add(1);
                global_visual_row = global_visual_row.saturating_add(1);
            }
        }
    }
    Ok(())
}
