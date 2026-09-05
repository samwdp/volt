fn draw_modeline(
    target: &mut DrawTarget<'_>,
    segments: &[ModelineSegment],
    draw: ModelineDraw<'_>,
) -> Result<(), ShellError> {
    let ModelineDraw {
        x,
        y,
        max_width,
        default_color,
        apply_tokens,
        theme_registry,
        user_library,
        acp_connected,
        lsp_server_visible,
        lsp_workspace_loaded,
        connected_color,
        cell_width,
        line_height,
    } = draw;
    let cell_width = cell_width.max(1);
    let gap_width = monospace_text_width(" ", cell_width);
    let chip_height = line_height.unwrap_or(cell_width).max(1) as u32;

    let left: Vec<&ModelineSegment> = segments
        .iter()
        .filter(|segment| segment.alignment == ModelineAlignment::Left)
        .collect();
    let right: Vec<&ModelineSegment> = segments
        .iter()
        .filter(|segment| segment.alignment == ModelineAlignment::Right)
        .collect();

    let right_width = modeline_side_width(&right, gap_width, cell_width);
    let left_budget = max_width.saturating_sub(right_width.saturating_add(gap_width));

    let joined = flatten_modeline_text(segments);
    let icon_colors = statusline_icon_colors(
        &joined,
        user_library,
        acp_connected,
        lsp_server_visible,
        lsp_workspace_loaded,
        theme_registry,
        connected_color,
    );
    let highlighted_icons = icon_colors
        .iter()
        .map(|(icon, _)| *icon)
        .collect::<Vec<_>>();

    draw_modeline_side(
        target,
        &left,
        ModelineSideDraw {
            x,
            y,
            max_width: left_budget,
            default_color,
            apply_tokens,
            theme_registry,
            icon_colors: &icon_colors,
            highlighted_icons: &highlighted_icons,
            cell_width,
            gap_width,
            chip_height,
            preserve_end: false,
        },
    )?;

    if right_width > 0 && right_width <= max_width {
        let right_x = x + max_width.saturating_sub(right_width) as i32;
        draw_modeline_side(
            target,
            &right,
            ModelineSideDraw {
                x: right_x,
                y,
                max_width: right_width,
                default_color,
                apply_tokens,
                theme_registry,
                icon_colors: &icon_colors,
                highlighted_icons: &highlighted_icons,
                cell_width,
                gap_width,
                chip_height,
                preserve_end: true,
            },
        )?;
    }
    Ok(())
}

fn modeline_side_width(segments: &[&ModelineSegment], gap_width: u32, cell_width: i32) -> u32 {
    let mut width = 0u32;
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            width = width.saturating_add(gap_width);
        }
        for (part_index, part) in segment.parts.iter().enumerate() {
            if part_index > 0 {
                width = width.saturating_add(gap_width);
            }
            width = width.saturating_add(monospace_text_width(&part.text, cell_width));
        }
    }
    width
}

fn draw_modeline_side(
    target: &mut DrawTarget<'_>,
    segments: &[&ModelineSegment],
    draw: ModelineSideDraw<'_>,
) -> Result<(), ShellError> {
    let ModelineSideDraw {
        x,
        y,
        max_width,
        default_color,
        apply_tokens,
        theme_registry,
        icon_colors,
        highlighted_icons,
        cell_width,
        gap_width,
        chip_height,
        preserve_end,
    } = draw;
    let mut draw_x = x;
    let mut remaining_width = max_width;
    for (index, segment) in segments.iter().enumerate() {
        if remaining_width == 0 {
            break;
        }
        if index > 0 {
            if remaining_width < gap_width {
                break;
            }
            draw_x += gap_width as i32;
            remaining_width = remaining_width.saturating_sub(gap_width);
        }
        for (part_index, part) in segment.parts.iter().enumerate() {
            if remaining_width == 0 {
                break;
            }
            if part_index > 0 {
                if remaining_width < gap_width {
                    break;
                }
                draw_x += gap_width as i32;
                remaining_width = remaining_width.saturating_sub(gap_width);
            }
            let text = if preserve_end {
                truncate_text_to_width_preserving_end(&part.text, remaining_width, cell_width)
            } else {
                truncate_text_to_width(&part.text, remaining_width, cell_width)
            };
            if text.is_empty() {
                break;
            }
            let painted_width = monospace_text_width(&text, cell_width);
            if let Some(background) = part.background.as_deref().filter(|token| !token.is_empty()) {
                let bg = theme_color(theme_registry, background, default_color);
                fill_rounded_rect(
                    target,
                    PixelRectToRect::rect(draw_x, y, painted_width, chip_height),
                    chip_height / 4,
                    bg,
                )?;
            }
            let token_color = (!part.foreground.is_empty() && apply_tokens)
                .then(|| theme_color(theme_registry, &part.foreground, default_color));
            for (segment_text, highlighted) in statusline_icon_segments(&text, highlighted_icons) {
                if remaining_width == 0 {
                    break;
                }
                let piece = if preserve_end {
                    truncate_text_to_width_preserving_end(segment_text, remaining_width, cell_width)
                } else {
                    truncate_text_to_width(segment_text, remaining_width, cell_width)
                };
                if piece.is_empty() {
                    break;
                }
                let color = if highlighted {
                    icon_colors
                        .iter()
                        .find_map(|(icon, color)| (*icon == segment_text).then_some(*color))
                        .unwrap_or(default_color)
                } else {
                    token_color.unwrap_or(default_color)
                };
                draw_text(target, draw_x, y, &piece, color)?;
                let piece_width = monospace_text_width(&piece, cell_width);
                draw_x += piece_width as i32;
                remaining_width = remaining_width.saturating_sub(piece_width);
            }
        }
    }
    Ok(())
}

pub(super) fn buffer_cursor_screen_anchor(
    buffer: &ShellBuffer,
    rect: Rect,
    user_library: &dyn UserLibrary,
    theme_registry: Option<&ThemeRegistry>,
    cell_width: i32,
    line_height: i32,
    typing_active: bool,
) -> Option<CursorScreenAnchor> {
    let cell_width = cell_width.max(1);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let headerline_rows = buffer_visible_headerline_row_count(
        buffer,
        user_library,
        layout.visible_rows,
        typing_active,
    );
    let body_y = layout.body_y + headerline_rows as i32 * line_height;
    let visible_rows = layout.visible_rows.saturating_sub(headerline_rows).max(1);
    let fringe_width = editor_fringe_width_px(cell_width, buffer.dap_fringe_live());
    let line_number_width = cell_width * 5;
    let wrap_cols = wrap_columns_for_width_with_fringe(
        rect.width(),
        cell_width,
        debug_fringe_cell_count(buffer.dap_fringe_live()),
    );
    let indent_size = theme_lang_indent(theme_registry, buffer.language_id());
    let cursor_row = buffer.cursor_row();
    let cursor_col = buffer.cursor_col();
    let wrapped_lines = collect_wrapped_lines(
        buffer,
        WrapCollect {
            start_line: buffer.scroll_row,
            max_rows: visible_rows,
            wrap_cols,
            indent_size,
            scroll_col: buffer.scroll_col,
            line_wrap: buffer.line_wrap(),
        },
    );
    let mut cursor_row_on_screen = None;
    let mut cursor_col_on_screen = None;
    let mut cursor_indent_cols = 0usize;
    let mut visual_row = 0usize;
    for wrapped in &wrapped_lines {
        if wrapped.line_index == cursor_row {
            let display_cursor_col = wrapped.char_map.cursor_anchor_col(cursor_col);
            let segment_index = segment_index_for_column(&wrapped.segments, display_cursor_col);
            if let Some(segment) = wrapped.segments.get(segment_index) {
                cursor_row_on_screen = Some(visual_row + segment_index);
                cursor_col_on_screen = Some(
                    wrapped
                        .char_map
                        .display_cols_between(segment.start_col, display_cursor_col),
                );
                cursor_indent_cols = if segment_index == 0 {
                    0
                } else {
                    wrapped.continuation_indent_cols
                };
            }
        }
        visual_row = visual_row.saturating_add(wrapped.visual_row_count());
        if visual_row >= visible_rows {
            break;
        }
    }
    let cursor_row_on_screen = cursor_row_on_screen?;
    let cursor_col_on_screen = cursor_col_on_screen?;
    Some(CursorScreenAnchor {
        x: rect.x()
            + 12
            + fringe_width
            + line_number_width
            + ((cursor_indent_cols + cursor_col_on_screen) as i32 * cell_width),
        y: body_y + cursor_row_on_screen as i32 * line_height,
        pane_bottom: layout.pane_bottom,
    })
}

#[cfg(test)]
pub(super) fn pretty_cursor_body_row(
    buffer: &ShellBuffer,
    rect: Rect,
    user_library: &dyn UserLibrary,
    theme_registry: Option<&ThemeRegistry>,
    cell_width: i32,
    line_height: i32,
) -> Option<usize> {
    let cell_width = cell_width.max(1);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let headerline_rows =
        buffer_visible_headerline_row_count(buffer, user_library, layout.visible_rows, false);
    let visible_rows = layout.visible_rows.saturating_sub(headerline_rows).max(1);
    let wrap_cols = wrap_columns_for_width(rect.width(), cell_width);
    let indent_size = theme_lang_indent(theme_registry, buffer.language_id());
    let text_width_px = (wrap_cols as i32 * cell_width).max(1) as u32;
    let pretty_paint = markdown_pretty_paint_plan(
        buffer,
        user_library,
        MarkdownPrettyPaintArgs {
            visible_start: buffer.scroll_row,
            visible_end: buffer
                .scroll_row
                .saturating_add(visible_rows.saturating_add(8)),
            visual_selection: None,
            input_mode: InputMode::Normal,
            pane_width_px: text_width_px,
            line_height,
        },
    );
    let wrapped_lines = collect_wrapped_lines_with_display(
        buffer,
        WrapCollect {
            start_line: buffer.scroll_row,
            max_rows: visible_rows,
            wrap_cols,
            indent_size,
            scroll_col: buffer.scroll_col,
            line_wrap: buffer.line_wrap(),
        },
        Some(&pretty_paint.text_overrides),
        Some(&pretty_paint.images),
    );
    let cursor_row = buffer.cursor_row();
    let mut visual_row = 0usize;
    for wrapped in &wrapped_lines {
        if wrapped.line_index == cursor_row {
            return Some(visual_row);
        }
        visual_row = visual_row.saturating_add(wrapped.visual_row_count());
        if visual_row >= visible_rows {
            break;
        }
    }
    None
}

pub(super) fn buffer_point_at_screen(
    buffer: &ShellBuffer,
    rect: Rect,
    user_library: &dyn UserLibrary,
    theme_registry: Option<&ThemeRegistry>,
    hit: ScreenHit,
    metrics: CellMetrics,
) -> Option<TextPoint> {
    let ScreenHit {
        x,
        y,
        clamp_body,
        typing_active,
    } = hit;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let line_height = line_height.max(1);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let headerline_rows = buffer_visible_headerline_row_count(
        buffer,
        user_library,
        layout.visible_rows,
        typing_active,
    );
    let visible_rows = layout.visible_rows.saturating_sub(headerline_rows).max(1);
    let body_top = layout.body_y + headerline_rows as i32 * line_height;
    let body_height = visible_rows as i32 * line_height;
    let body_bottom = body_top + body_height;
    if body_height <= 0 {
        return None;
    }
    if !clamp_body && (y < body_top || y >= body_bottom) {
        return None;
    }
    let y = y.clamp(body_top, body_bottom.saturating_sub(1));
    let visual_row_target = ((y - body_top) / line_height) as usize;
    let cell_width = cell_width.max(1);
    let fringe_width = editor_fringe_width_px(cell_width, buffer.dap_fringe_live());
    let line_number_width = cell_width * 5;
    let text_x = rect.x() + 12 + fringe_width + line_number_width;
    let wrap_cols = wrap_columns_for_width_with_fringe(
        rect.width(),
        cell_width,
        debug_fringe_cell_count(buffer.dap_fringe_live()),
    );
    let indent_size = theme_lang_indent(theme_registry, buffer.language_id());
    let wrapped_lines = collect_wrapped_lines(
        buffer,
        WrapCollect {
            start_line: buffer.scroll_row,
            max_rows: visible_rows,
            wrap_cols,
            indent_size,
            scroll_col: buffer.scroll_col,
            line_wrap: buffer.line_wrap(),
        },
    );
    let mut visual_row = 0usize;
    for wrapped in wrapped_lines {
        let line_len = buffer.line_len_chars(wrapped.line_index);
        let row_count = wrapped.visual_row_count();
        if row_count > wrapped.segments.len() {
            if visual_row_target >= visual_row
                && visual_row_target < visual_row.saturating_add(row_count)
            {
                return Some(TextPoint::new(wrapped.line_index, 0));
            }
            visual_row = visual_row.saturating_add(row_count);
            continue;
        }
        for (segment_index, segment) in wrapped.segments.iter().enumerate() {
            if visual_row == visual_row_target {
                let segment_indent_cols = if segment_index == 0 {
                    0
                } else {
                    wrapped.continuation_indent_cols
                };
                let segment_x = text_x + (segment_indent_cols as i32 * cell_width);
                let display_offset = (x.saturating_sub(segment_x) / cell_width).max(0) as usize;
                let max_col = if line_len == 0 {
                    0
                } else {
                    segment.end_col.saturating_sub(1)
                };
                let display_col = wrapped
                    .char_map
                    .display_col_at(segment.start_col)
                    .saturating_add(display_offset);
                let column = wrapped
                    .char_map
                    .char_col_for_display_col(display_col)
                    .min(max_col);
                return Some(TextPoint::new(wrapped.line_index, column));
            }
            visual_row = visual_row.saturating_add(1);
        }
    }
    None
}

pub(super) fn fallback_overlay_anchor(
    buffer: &ShellBuffer,
    rect: Rect,
    line_height: i32,
    command_line_visible: bool,
) -> CursorScreenAnchor {
    let layout =
        buffer_footer_layout_with_command_line(buffer, rect, line_height, 0, command_line_visible);
    CursorScreenAnchor {
        x: rect.x() + 24,
        y: layout.body_y + 6,
        pane_bottom: layout.pane_bottom,
    }
}

#[derive(Debug)]
pub(super) struct WrappedLine {
    pub(super) line_index: usize,
    pub(super) line: String,
    pub(super) char_map: LineCharMap,
    pub(super) segments: Vec<LineWrapSegment>,
    pub(super) continuation_indent_cols: usize,
    pub(super) inline_image: Option<MarkdownInlineImageDraw>,
    visual_rows: usize,
}
