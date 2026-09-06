impl WrappedLine {
    fn visual_row_count(&self) -> usize {
        self.visual_rows.max(1)
    }
}

pub(super) fn collect_wrapped_lines(buffer: &ShellBuffer, wrap: WrapCollect) -> Vec<WrappedLine> {
    collect_wrapped_lines_with_display(buffer, wrap, None, None)
}

pub(super) fn collect_wrapped_lines_with_display(
    buffer: &ShellBuffer,
    wrap: WrapCollect,
    display_overrides: Option<&BTreeMap<usize, String>>,
    inline_images: Option<&BTreeMap<usize, MarkdownInlineImageDraw>>,
) -> Vec<WrappedLine> {
    let WrapCollect {
        start_line,
        max_rows,
        wrap_cols,
        indent_size,
        scroll_col,
        line_wrap,
    } = wrap;
    if max_rows == 0 {
        return Vec::new();
    }

    let wrap_cols = wrap_cols.max(1);
    let mut lines = Vec::new();
    let mut visual_rows = 0usize;
    let mut line_index = start_line;
    let line_count = buffer.line_count();
    while line_index < line_count && visual_rows < max_rows {
        let inline_image = inline_images.and_then(|images| images.get(&line_index).cloned());
        let pretty_rows = buffer.pretty_display_rows.get(&line_index).copied();
        let is_pretty_image = inline_image.is_some() || pretty_rows.is_some();
        let line = if is_pretty_image {
            String::new()
        } else {
            display_overrides
                .and_then(|overrides| overrides.get(&line_index).cloned())
                .unwrap_or_else(|| buffer.text.line(line_index).unwrap_or_default())
        };
        let tab_width = resolved_tab_width(indent_size);
        let char_map = LineCharMap::with_tab_width(&line, tab_width);
        let (leading_indent_cols, _) = leading_whitespace_info(&line, tab_width);
        let continuation_indent_cols = leading_indent_cols.saturating_add(indent_size);
        let (continuation_indent_cols, segments) = if is_pretty_image {
            (
                0,
                vec![LineWrapSegment {
                    start_col: 0,
                    end_col: 0,
                }],
            )
        } else if line_wrap {
            let continuation_cols = wrap_cols.saturating_sub(continuation_indent_cols).max(1);
            (
                continuation_indent_cols,
                wrap_line_segments(&char_map, wrap_cols, continuation_cols),
            )
        } else {
            let start_col = char_map.char_col_for_display_col(scroll_col);
            let end_display_col = scroll_col.saturating_add(wrap_cols);
            let end_col = if end_display_col >= char_map.display_col_at(char_map.len()) {
                char_map.len()
            } else {
                char_map.char_col_for_display_col(end_display_col)
            };
            (
                0,
                vec![LineWrapSegment {
                    start_col: start_col.min(end_col),
                    end_col,
                }],
            )
        };
        let row_count = inline_image
            .as_ref()
            .map(|image| image.rows.max(1))
            .or(pretty_rows.map(|rows| rows.max(1)))
            .unwrap_or_else(|| segments.len().max(1));
        visual_rows = visual_rows.saturating_add(row_count);
        lines.push(WrappedLine {
            line_index,
            line,
            char_map,
            segments,
            continuation_indent_cols,
            inline_image,
            visual_rows: row_count,
        });
        line_index = line_index.saturating_add(1);
    }

    lines
}

pub(super) fn render_buffer(
    target: &mut DrawTarget<'_>,
    request: BufferDrawRequest<'_>,
    chrome: BufferChrome<'_>,
    metrics: TextMetrics,
) -> Result<(), ShellError> {
    let BufferDrawRequest {
        buffer,
        view_state,
        pane: PaneSlot { rect, active },
        decorations:
            BufferDecorations {
                visual_selection,
                yank_flash,
                input_mode,
                multicursor,
                vim_targets_input,
                recording_macro,
                typing_active,
            },
        command_line:
            CommandLineSlot {
                input: command_line_input,
                row_visible: command_line_row_visible,
            },
    } = request;
    let BufferChrome {
        user_library,
        theme_registry,
        workspace_name,
        lsp_server,
        lsp_workspace_loaded,
        acp_connected,
        git_summary,
    } = chrome;
    let TextMetrics {
        cell_width,
        line_height,
        ascent,
    } = metrics;
    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let muted = blend_color(foreground, base_background, 0.5);
    let border_color = adjust_color(base_background, if is_dark { 24 } else { -24 });
    let commandline_background = theme_color(
        theme_registry,
        TOKEN_COMMANDLINE_BACKGROUND,
        Color::RGBA(base_background.r, base_background.g, base_background.b, 0),
    );
    let statusline_active = theme_color(
        theme_registry,
        TOKEN_STATUSLINE_ACTIVE,
        Color::RGBA(110, 170, 255, 255),
    );
    let statusline_inactive = theme_color(theme_registry, TOKEN_STATUSLINE_INACTIVE, muted);
    let statusline_text_color = if active {
        theme_color(
            theme_registry,
            TOKEN_STATUSLINE_FOREGROUND,
            statusline_active,
        )
    } else {
        theme_color(
            theme_registry,
            TOKEN_STATUSLINE_INACTIVE_FOREGROUND,
            theme_color(
                theme_registry,
                TOKEN_STATUSLINE_FOREGROUND,
                statusline_inactive,
            ),
        )
    };
    let text_color = foreground;
    let line_number_color = theme_color(theme_registry, TOKEN_LINE_NUMBER, muted);
    let line_number_current_color =
        theme_color(theme_registry, TOKEN_LINE_NUMBER_CURRENT, foreground);
    let ghost_text_color = theme_color(theme_registry, TOKEN_GHOST_TEXT, muted);
    let headerline_color = theme_color(theme_registry, TOKEN_HEADERLINE, statusline_active);
    let headerline_background =
        theme_color(theme_registry, TOKEN_HEADERLINE_BACKGROUND, base_background);
    let cursor = theme_color(theme_registry, "ui.cursor", Color::RGB(110, 170, 255));
    let selection = theme_color(theme_registry, "ui.selection", Color::RGBA(55, 71, 99, 255));
    let current_line_wash = theme_color(
        theme_registry,
        TOKEN_CURRENT_LINE,
        adjust_color(base_background, if is_dark { 12 } else { -12 }),
    );
    let relative_line_numbers = theme_registry
        .and_then(|registry| registry.resolve_bool(OPTION_LINE_NUMBER_RELATIVE))
        .unwrap_or(false);
    let cursor_roundness = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_CURSOR_ROUNDNESS))
        .map(|value| value.clamp(0.0, 16.0).round() as u32)
        .unwrap_or(2);
    let yank_flash_color = theme_registry
        .and_then(|registry| registry.resolve("ui.yank-flash"))
        .map(to_sdl_color)
        .unwrap_or(Color::RGBA(112, 196, 255, 120));
    let show_paren_match_color = theme_color(
        theme_registry,
        TOKEN_SHOW_PAREN_MATCH,
        Color::RGBA(110, 170, 255, 110),
    );
    let show_paren_mismatch_color = theme_color(
        theme_registry,
        TOKEN_SHOW_PAREN_MISMATCH,
        Color::RGBA(220, 80, 80, 160),
    );
    let git_added_fallback = theme_color(
        theme_registry,
        "git.status.entry.added",
        Color::RGB(108, 193, 118),
    );
    let git_modified_fallback = theme_color(
        theme_registry,
        "git.status.entry.modified",
        Color::RGB(209, 154, 102),
    );
    let git_removed_fallback = theme_color(
        theme_registry,
        "git.status.entry.deleted",
        Color::RGB(224, 107, 117),
    );
    let git_fringe_added = theme_color(
        theme_registry,
        user_library.gitfringe_token_added(),
        git_added_fallback,
    );
    let git_fringe_modified = theme_color(
        theme_registry,
        user_library.gitfringe_token_modified(),
        git_modified_fallback,
    );
    let git_fringe_removed = theme_color(
        theme_registry,
        user_library.gitfringe_token_removed(),
        git_removed_fallback,
    );
    let debug_fringe_verified = theme_color(
        theme_registry,
        TOKEN_DEBUG_FRINGE_BREAKPOINT,
        Color::RGB(224, 107, 117),
    );
    let debug_fringe_pending = theme_color(
        theme_registry,
        TOKEN_DEBUG_FRINGE_PENDING,
        Color::RGB(209, 154, 102),
    );
    let debug_fringe_execution = theme_color(
        theme_registry,
        TOKEN_DEBUG_FRINGE_EXECUTION,
        Color::RGB(86, 182, 194),
    );
    let debug_line_execution = theme_color(
        theme_registry,
        TOKEN_DEBUG_LINE_EXECUTION,
        Color::RGBA(86, 182, 194, 48),
    );
    let cell_width = cell_width.max(1);
    let (git_branch, git_added, git_removed) = git_summary
        .map(|summary| (summary.branch.as_deref(), summary.added, summary.removed))
        .unwrap_or((None, 0, 0));
    let lsp_diagnostics = statusline_lsp_diagnostics(buffer.lsp_diagnostics());
    let terminal_cursor = (buffer_is_terminal(&buffer.kind)
        && active
        && matches!(input_mode, InputMode::Insert | InputMode::Replace))
    .then(|| {
        buffer
            .terminal_render()
            .and_then(TerminalRenderSnapshot::cursor)
    })
    .flatten();
    let cursor_row = view_state.cursor.line;
    let cursor_col = view_state.cursor.column;
    let scroll_row = view_state.scroll_row;
    let statusline_line = terminal_cursor
        .map(|cursor| cursor.row() as usize + 1)
        .unwrap_or(cursor_row + 1);
    let statusline_column = terminal_cursor
        .map(|cursor| cursor.col() as usize + 1)
        .unwrap_or(cursor_col + 1);
    let statusline_context = HostStatuslineContext {
        vim_mode: statusline_mode_label(input_mode, multicursor.is_some()),
        recording_macro,
        workspace_name,
        buffer_name: buffer.display_name(),
        buffer_modified: buffer.is_dirty(),
        language_id: buffer.language_id(),
        line: statusline_line,
        column: statusline_column,
        lsp_server,
        lsp_diagnostics,
        acp_connected,
        git_branch,
        git_added,
        git_removed,
    };
    let modeline_segments = user_library.modeline_segments(&statusline_context);
    let statusline = truncate_text_to_width(
        &flatten_modeline_text(&modeline_segments),
        rect.width().saturating_sub(24),
        cell_width,
    );
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        command_line_row_visible,
    );
    if buffer_is_terminal(&buffer.kind)
        && let Some(terminal_render) = buffer.terminal_render()
    {
        render_terminal_buffer(
            target,
            TerminalBufferDraw {
                buffer,
                terminal_render,
                rect,
                layout,
                active,
                input_mode,
                visual_selection,
                yank_flash,
            },
            BufferBodyPalette {
                theme_registry,
                base_background,
                foreground: text_color,
                muted: text_color,
                border_color,
                selection,
                yank_flash_color,
                cursor,
                cursor_roundness,
            },
            TerminalStatusline {
                text: statusline,
                active: statusline_active,
                inactive: statusline_inactive,
            },
            CellMetrics {
                cell_width,
                line_height,
            },
        )?;
        if let Some(commandline_y) = layout.commandline_y {
            render_footer_separator(
                target,
                rect,
                commandline_y - BUFFER_FOOTER_SEPARATOR_OFFSET,
                border_color,
                window_effects,
            )?;
        }
        render_command_line_overlay(
            target,
            CommandLineOverlayDraw {
                input: command_line_input,
                rect,
                layout,
                active,
                input_mode,
                paint: CommandLinePaint {
                    window_effects,
                    background: commandline_background,
                    foreground: text_color,
                    muted,
                    cursor,
                    cursor_roundness,
                    chip_radius: overlay_radius(theme_registry).min(8),
                },
                metrics: CellMetrics {
                    cell_width,
                    line_height,
                },
            },
        )?;
        return Ok(());
    }
    let text_x = rect.x() + 12 + cell_width + cell_width * 5;
    if buffer_is_browser(&buffer.kind) {
        render_browser_buffer_body(
            target,
            BrowserBufferDraw {
                buffer,
                rect,
                layout,
                active,
                input_mode,
            },
            BufferBodyPalette {
                theme_registry,
                base_background,
                foreground,
                muted,
                border_color,
                selection,
                yank_flash_color: selection,
                cursor,
                cursor_roundness,
            },
            CellMetrics {
                cell_width,
                line_height,
            },
        )?;
    } else if buffer.has_pdf_preview_surface() {
        render_pdf_buffer_body(target, rect, layout, theme_registry, base_background)?;
    } else if buffer.is_acp_buffer() {
        render_acp_buffer_body(
            target,
            AcpBufferDraw {
                buffer,
                rect,
                layout,
                active,
                visual_selection,
                yank_flash,
                input_mode,
            },
            BufferBodyPalette {
                theme_registry,
                base_background,
                foreground,
                muted,
                border_color,
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
    } else if buffer.is_rendered_image_buffer() {
        render_image_buffer_body(
            target,
            buffer,
            rect,
            layout,
            theme_registry,
            base_background,
        )?;
    } else if buffer.has_plugin_sections() {
        render_plugin_section_buffer_body(
            target,
            PluginSectionDraw {
                buffer,
                view_state,
                pane: PaneSlot { rect, active },
                layout,
                visual_selection,
                yank_flash,
                input_mode,
            },
            BufferBodyPalette {
                theme_registry,
                base_background,
                foreground,
                muted,
                border_color,
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
    } else {
        let debug_fringe_live = buffer.dap_fringe_live();
        let fringe_width = editor_fringe_width_px(cell_width, debug_fringe_live);
        let line_number_width = cell_width * 5;
        let wrap_cols = wrap_columns_for_width_with_fringe(
            rect.width(),
            cell_width,
            debug_fringe_cell_count(debug_fringe_live),
        );
        let indent_size = theme_lang_indent(theme_registry, buffer.language_id());
        let context_overlay =
            buffer_context_overlay_snapshot(buffer, active, typing_active, user_library);
        let headerline_lines = context_overlay
            .as_ref()
            .map(|snapshot| {
                visible_headerline_lines(&snapshot.headerline_lines, layout.visible_rows)
            })
            .unwrap_or_default();
        let headerline_rows = headerline_lines.len();
        let body_y = layout.body_y + headerline_rows as i32 * line_height;
        let visible_rows = layout.visible_rows.saturating_sub(headerline_rows).max(1);
        let text_width_px = (wrap_cols as i32 * cell_width).max(1) as u32;
        let pretty_paint = markdown_pretty_paint_plan(
            buffer,
            user_library,
            MarkdownPrettyPaintArgs {
                visible_start: view_state.scroll_row,
                visible_end: view_state
                    .scroll_row
                    .saturating_add(visible_rows.saturating_add(8)),
                visual_selection,
                input_mode,
                pane_width_px: text_width_px,
                line_height,
            },
        );
        let wrapped_lines = collect_wrapped_lines_with_display(
            buffer,
            WrapCollect {
                start_line: scroll_row,
                max_rows: visible_rows,
                wrap_cols,
                indent_size,
                scroll_col: view_state.scroll_col,
                line_wrap: buffer.line_wrap(),
            },
            Some(&pretty_paint.text_overrides),
            Some(&pretty_paint.images),
        );
        let mut cursor_row_on_screen = None;
        let mut cursor_col_on_screen = None;
        let mut cursor_indent_cols = 0usize;
        let multicursor_points = if active {
            multicursor
                .map(multicursor_cursor_points)
                .unwrap_or_default()
        } else {
            Vec::new()
        };
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

        let gutter_x = rect.x() + 12;
        let fringe_x = gutter_x;
        let line_number_x = gutter_x + fringe_width;
        let text_x = line_number_x + line_number_width;
        let headerline_width = rect
            .width()
            .saturating_sub((text_x - rect.x()).max(0) as u32 + 12);
        let show_text_cursor = !buffer.has_input_field()
            || !active
            || !vim_targets_input
            || !matches!(input_mode, InputMode::Insert | InputMode::Replace);
        let cursor_width = match input_mode {
            InputMode::Normal | InputMode::Visual => cell_width.max(2) as u32,
            InputMode::Insert | InputMode::Replace => (cell_width / 4).max(2) as u32,
        };
        let primary_cursor_rect = if show_text_cursor
            && let (Some(cursor_row_on_screen), Some(cursor_col_on_screen)) =
                (cursor_row_on_screen, cursor_col_on_screen)
            && cursor_row_on_screen < visible_rows
        {
            Some(PixelRectToRect::rect(
                text_x + ((cursor_indent_cols + cursor_col_on_screen) as i32 * cell_width),
                body_y + cursor_row_on_screen as i32 * line_height,
                cursor_width,
                line_height.max(2) as u32,
            ))
        } else {
            None
        };
        let mut primary_cursor_text_overlay: Option<(i32, CursorTextOverlay)> = None;
        let mut multicursor_rects = Vec::new();
        let mut visual_row = 0usize;
        let show_paren = buffer
            .show_paren_enabled(user_library.show_paren_config().enabled)
            .then(|| {
                buffer
                    .text
                    .show_paren_at(view_state.cursor, buffer.language_id())
            })
            .flatten();
        for wrapped in wrapped_lines {
            let line_index = wrapped.line_index;
            let line_len = buffer.line_len_chars(line_index);
            let selection_range = visual_selection.and_then(|selection_state| {
                selection_columns_for_visual(selection_state, line_index, line_len)
            });
            let multicursor_ranges = multicursor
                .map(|state| multicursor_ranges_for_line(state, input_mode, line_index, line_len))
                .unwrap_or_default();
            let yank_range = yank_flash.and_then(|selection_state| {
                selection_columns_for_visual(selection_state, line_index, line_len)
            });
            let show_paren_color = show_paren.as_ref().map(|pair| {
                if pair.matched {
                    show_paren_match_color
                } else {
                    show_paren_mismatch_color
                }
            });
            if let Some(image) = wrapped.inline_image.as_ref() {
                if visual_row >= visible_rows {
                    break;
                }
                let remaining_rows = visible_rows.saturating_sub(visual_row);
                let image_rows_draw = image.rows.min(remaining_rows).max(1);
                let y = body_y + visual_row as i32 * line_height;
                let text_area_width = rect
                    .width()
                    .saturating_sub((text_x - rect.x()).max(0) as u32)
                    .saturating_sub(12)
                    .max(1);
                let image_rect = PixelRectToRect::rect(
                    text_x,
                    y,
                    text_area_width,
                    (image_rows_draw as i32 * line_height).max(line_height) as u32,
                );
                let line_number = if relative_line_numbers {
                    if line_index == cursor_row {
                        0
                    } else {
                        cursor_row.abs_diff(line_index)
                    }
                } else {
                    line_index + 1
                };
                draw_text(
                    target,
                    line_number_x,
                    y,
                    &format!("{:>4}", line_number),
                    if line_index == cursor_row {
                        line_number_current_color
                    } else {
                        line_number_color
                    },
                )?;
                draw_image(
                    target,
                    image_rect,
                    image.width,
                    image.height,
                    Arc::clone(&image.pixels),
                    Some(image_rect),
                )?;
                let _ = &image.alt;
                visual_row = visual_row.saturating_add(image_rows_draw);
                if visual_row >= visible_rows {
                    break;
                }
                continue;
            }
            for (segment_index, segment) in wrapped.segments.iter().enumerate() {
                if visual_row >= visible_rows {
                    break;
                }
                let y = body_y + visual_row as i32 * line_height;
                let is_execution_line = buffer.dap_execution_line() == Some(line_index);
                if is_execution_line {
                    fill_window_surface_rect(
                        target,
                        PixelRectToRect::rect(
                            gutter_x,
                            y,
                            rect.width().saturating_sub(24),
                            line_height.max(1) as u32,
                        ),
                        debug_line_execution,
                        window_effects,
                    )?;
                } else if active
                    && line_index == cursor_row
                    && !matches!(input_mode, InputMode::Visual)
                {
                    fill_window_surface_rect(
                        target,
                        PixelRectToRect::rect(
                            gutter_x,
                            y,
                            rect.width().saturating_sub(24),
                            line_height.max(1) as u32,
                        ),
                        current_line_wash,
                        window_effects,
                    )?;
                }
                let segment_indent_cols = if segment_index == 0 {
                    0
                } else {
                    wrapped.continuation_indent_cols
                };
                let segment_x = text_x + (segment_indent_cols as i32 * cell_width);
                if let Some((selection_start, selection_end)) = selection_range {
                    let start = selection_start.max(segment.start_col);
                    let end = selection_end.min(segment.end_col);
                    if start < end {
                        let start_display = wrapped
                            .char_map
                            .display_cols_between(segment.start_col, start);
                        let width_display = wrapped.char_map.display_cols_between(start, end);
                        fill_selection_highlight(
                            target,
                            segment_x + (start_display as i32 * cell_width),
                            y,
                            (width_display as i32 * cell_width) as u32,
                            line_height.max(1) as u32,
                            cursor_roundness,
                            selection,
                        )?;
                    }
                }
                for (selection_start, selection_end) in multicursor_ranges.iter().copied() {
                    let start = selection_start.max(segment.start_col);
                    let end = selection_end.min(segment.end_col);
                    if start < end {
                        let start_display = wrapped
                            .char_map
                            .display_cols_between(segment.start_col, start);
                        let width_display = wrapped.char_map.display_cols_between(start, end);
                        fill_selection_highlight(
                            target,
                            segment_x + (start_display as i32 * cell_width),
                            y,
                            (width_display as i32 * cell_width) as u32,
                            line_height.max(1) as u32,
                            cursor_roundness,
                            blend_color(selection, cursor, 0.25),
                        )?;
                    }
                }
                if let Some((selection_start, selection_end)) = yank_range {
                    let start = selection_start.max(segment.start_col);
                    let end = selection_end.min(segment.end_col);
                    if start < end {
                        let start_display = wrapped
                            .char_map
                            .display_cols_between(segment.start_col, start);
                        let width_display = wrapped.char_map.display_cols_between(start, end);
                        fill_selection_highlight(
                            target,
                            segment_x + (start_display as i32 * cell_width),
                            y,
                            (width_display as i32 * cell_width) as u32,
                            line_height.max(1) as u32,
                            cursor_roundness,
                            yank_flash_color,
                        )?;
                    }
                }
                if let (Some(pair), Some(color)) = (show_paren.as_ref(), show_paren_color) {
                    for range in [Some(pair.origin), pair.counterpart].into_iter().flatten() {
                        if let Some((selection_start, selection_end)) =
                            selection_columns_for_line(range, line_index, line_len)
                        {
                            let start = selection_start.max(segment.start_col);
                            let end = selection_end.min(segment.end_col);
                            if start < end {
                                let start_display = wrapped
                                    .char_map
                                    .display_cols_between(segment.start_col, start);
                                let width_display =
                                    wrapped.char_map.display_cols_between(start, end);
                                fill_selection_highlight(
                                    target,
                                    segment_x + (start_display as i32 * cell_width),
                                    y,
                                    (width_display as i32 * cell_width) as u32,
                                    line_height.max(1) as u32,
                                    cursor_roundness,
                                    color,
                                )?;
                            }
                        }
                    }
                }
                if segment_index == 0 {
                    let diagnostic_severity = user_library
                        .lsp_show_buffer_diagnostics()
                        .then(|| buffer.lsp_diagnostic_severity(line_index))
                        .flatten();
                    let dap_marker = buffer.dap_fringe_marker(line_index);
                    let dap_execution = buffer.dap_execution_line() == Some(line_index);
                    let shows_dap_glyph = dap_execution || dap_marker.is_some();
                    let git_fringe_x = if debug_fringe_live {
                        fringe_x + cell_width
                    } else {
                        fringe_x
                    };
                    if dap_execution {
                        draw_text(
                            target,
                            fringe_x,
                            y,
                            DEBUG_FRINGE_EXECUTION_GLYPH,
                            debug_fringe_execution,
                        )?;
                    } else if let Some(state) = dap_marker {
                        let (glyph, color) = match state {
                            BreakpointState::Verified => {
                                (DEBUG_FRINGE_VERIFIED_GLYPH, debug_fringe_verified)
                            }
                            BreakpointState::Pending | BreakpointState::Unverified => {
                                (DEBUG_FRINGE_PENDING_GLYPH, debug_fringe_pending)
                            }
                        };
                        draw_text(target, fringe_x, y, glyph, color)?;
                    }
                    // Idle: Breakpoint glyph replaces git on that line. Live Session:
                    // DAP markers keep the left cell; git uses the widened right cell.
                    let draw_git_or_diagnostic = debug_fringe_live || !shows_dap_glyph;
                    if draw_git_or_diagnostic {
                        if let Some(severity) = diagnostic_severity {
                            let color = diagnostic_color(severity, theme_registry);
                            draw_text(
                                target,
                                git_fringe_x,
                                y,
                                user_library.lsp_diagnostic_icon(),
                                color,
                            )?;
                        } else if let Some(kind) = buffer.git_fringe_kind(line_index) {
                            let color = match kind {
                                GitFringeKind::Added => git_fringe_added,
                                GitFringeKind::Modified => git_fringe_modified,
                                GitFringeKind::Removed => git_fringe_removed,
                            };
                            fill_window_surface_rect(
                                target,
                                PixelRectToRect::rect(
                                    git_fringe_x - 4,
                                    y,
                                    GIT_FRINGE_BAR_WIDTH,
                                    line_height.max(1) as u32,
                                ),
                                color,
                                window_effects,
                            )?;
                        }
                    }
                    let line_number = if relative_line_numbers {
                        if line_index == cursor_row {
                            0
                        } else {
                            cursor_row.abs_diff(line_index)
                        }
                    } else {
                        line_index + 1
                    };
                    let line_number_color = diagnostic_severity
                        .map(|severity| diagnostic_color(severity, theme_registry))
                        .unwrap_or(if line_index == cursor_row {
                            line_number_current_color
                        } else {
                            line_number_color
                        });
                    draw_text(
                        target,
                        line_number_x,
                        y,
                        &format!("{:>4}", line_number),
                        line_number_color,
                    )?;
                }
                draw_buffer_text(
                    target,
                    BufferTextRun {
                        x: segment_x,
                        y,
                        line: &wrapped.line,
                        segment: *segment,
                        char_map: &wrapped.char_map,
                        line_syntax_spans: buffer.line_syntax_spans(line_index),
                        default_color: text_color,
                        cell_width,
                    },
                    theme_registry,
                )?;
                if primary_cursor_text_overlay.is_none()
                    && let Some(overlay) = block_cursor_text_overlay(CursorOverlayQuery {
                        x: segment_x,
                        line: &wrapped.line,
                        char_map: &wrapped.char_map,
                        segment: *segment,
                        line_index,
                        cursor: TextPoint::new(cursor_row, cursor_col),
                        color: (matches!(input_mode, InputMode::Normal | InputMode::Visual)
                            && !vim_targets_input)
                            .then_some(base_background),
                        cell_width,
                    })
                {
                    primary_cursor_text_overlay = Some((y, overlay));
                }
                if user_library.lsp_show_buffer_diagnostics() && buffer.lsp_enabled() {
                    draw_diagnostic_underlines_for_segment(
                        target,
                        DiagnosticUnderlineDraw {
                            diagnostics: buffer.lsp_diagnostic_line_spans(line_index),
                            syntax_spans: buffer.line_syntax_spans(line_index),
                            char_map: &wrapped.char_map,
                            segment_x,
                            y,
                            line_len,
                            segment: *segment,
                            metrics: CellMetrics {
                                cell_width,
                                line_height,
                            },
                            theme_registry,
                        },
                    )?;
                }
                draw_line_ghost_text_for_segment(
                    target,
                    GhostTextSegmentDraw {
                        x: segment_x,
                        y,
                        segment: *segment,
                        char_map: &wrapped.char_map,
                        line_len,
                        ghost_text: context_overlay
                            .as_ref()
                            .and_then(|snapshot| snapshot.ghost_text_by_line.get(&line_index))
                            .map(String::as_str),
                        color: ghost_text_color,
                        cell_width,
                    },
                )?;
                for point in multicursor_points.iter().copied().filter(|point| {
                    point.line == line_index
                        && point.column >= segment.start_col
                        && point.column <= segment.end_col
                }) {
                    multicursor_rects.push(PixelRectToRect::rect(
                        segment_x
                            + (wrapped
                                .char_map
                                .display_cols_between(segment.start_col, point.column)
                                as i32
                                * cell_width),
                        y,
                        cursor_width,
                        line_height.max(2) as u32,
                    ));
                }
                visual_row = visual_row.saturating_add(1);
            }
            if visual_row >= visible_rows {
                break;
            }
        }
        if headerline_rows > 0 {
            for (index, headerline) in headerline_lines.iter().enumerate() {
                let y = layout.body_y + index as i32 * line_height;
                fill_window_surface_rounded_rect(
                    target,
                    PixelRectToRect::rect(
                        rect.x() + 8,
                        y + 1,
                        rect.width().saturating_sub(16),
                        line_height.saturating_sub(2).max(1) as u32,
                    ),
                    overlay_radius(theme_registry).min(8),
                    headerline_background,
                    window_effects,
                )?;
                draw_text(
                    target,
                    text_x,
                    y,
                    &truncate_text_to_width_preserving_end(
                        headerline,
                        headerline_width,
                        cell_width,
                    ),
                    headerline_color,
                )?;
            }
            fill_window_surface_rect(
                target,
                PixelRectToRect::rect(
                    rect.x() + 8,
                    body_y.saturating_sub(1),
                    rect.width().saturating_sub(16),
                    1,
                ),
                border_color,
                window_effects,
            )?;
        }
        for rect in multicursor_rects {
            fill_rounded_rect(target, rect, cursor_roundness, cursor)?;
        }
        if let Some(rect) = primary_cursor_rect {
            fill_rounded_rect(target, rect, cursor_roundness, cursor)?;
        }
        if let Some((y, overlay)) = primary_cursor_text_overlay {
            draw_text(target, overlay.draw_x, y, &overlay.text, overlay.color)?;
        }
        paint_buffer_scrollbar(
            target,
            ScrollbarPaint {
                pane_rect: rect,
                body_y,
                visible_rows,
                line_height,
                scroll_row: view_state.scroll_row,
                max_scroll: buffer.max_scroll_row_for_wrapped_rows(
                    visible_rows,
                    wrap_cols,
                    indent_size,
                ),
                color: muted,
                window_effects,
            },
        )?;
    }

    if let Some(input) = buffer.standalone_input_field() {
        let input_background = theme_color(
            theme_registry,
            "ui.input.background",
            adjust_color(base_background, if is_dark { 8 } else { -8 }),
        );
        let input_foreground = theme_color(theme_registry, "ui.input.foreground", foreground);
        let placeholder_color = theme_color(theme_registry, "ui.input.placeholder", muted);
        fill_window_surface_rect(
            target,
            PixelRectToRect::rect(
                rect.x() + 8,
                layout.input_y - 4,
                rect.width().saturating_sub(16),
                layout.input_box_height as u32,
            ),
            input_background,
            window_effects,
        )?;
        if buffer_is_acp(&buffer.kind) {
            let border = if acp_connected {
                git_added_fallback
            } else {
                border_color
            };
            fill_window_surface_rect(
                target,
                PixelRectToRect::rect(
                    rect.x() + 8,
                    layout.input_y - 4,
                    rect.width().saturating_sub(16),
                    1,
                ),
                border,
                window_effects,
            )?;
            fill_window_surface_rect(
                target,
                PixelRectToRect::rect(
                    rect.x() + 8,
                    layout.input_y - 4 + layout.input_box_height,
                    rect.width().saturating_sub(16),
                    1,
                ),
                border,
                window_effects,
            )?;
        }
        let input_x = text_x;
        let prompt = input.prompt();
        let prompt_len = prompt.chars().count();
        let prompt_padding = " ".repeat(prompt_len);
        let text_width = rect.width() as i32 - (text_x - rect.x()) - 12;
        let available_input_cols = (text_width / cell_width.max(1)).max(1) as usize;
        if active && vim_targets_input && matches!(input_mode, InputMode::Visual) {
            for (row, start_col, end_col) in
                input.selection_visual_ranges(VisualSelectionKind::Character, available_input_cols)
            {
                fill_selection_highlight(
                    target,
                    input_x + ((prompt_len + start_col) as i32 * cell_width),
                    layout.input_y + row as i32 * line_height,
                    ((end_col.saturating_sub(start_col)) as i32 * cell_width.max(1)) as u32,
                    line_height.max(1) as u32,
                    cursor_roundness,
                    selection,
                )?;
            }
        }
        let max_visible_rows = visible_input_text_rows(layout.input_box_height, line_height);
        let (visible_rows, first_visible_row) =
            input.visible_wrapped_visual_rows(available_input_cols, max_visible_rows);
        if input.text().is_empty() {
            if let Some(placeholder) = input.placeholder() {
                let line = format!("{prompt}{placeholder}");
                draw_text(target, input_x, layout.input_y, &line, placeholder_color)?;
            } else {
                draw_text(target, input_x, layout.input_y, prompt, input_foreground)?;
            }
        } else {
            for (index, line) in visible_rows.into_iter().enumerate() {
                let prefix = if index == 0 { prompt } else { &prompt_padding };
                let rendered = format!("{prefix}{line}");
                draw_text(
                    target,
                    input_x,
                    layout.input_y + index as i32 * line_height,
                    &rendered,
                    input_foreground,
                )?;
            }
        }
        if let Some(hint) = input.hint() {
            let hint_y = layout.input_y + layout.input_box_height + layout.input_hint_gap;
            if let Some((mode_label, rest)) = hint.split_once(" · ") {
                let prefix = format!("{prompt_padding}{mode_label}");
                draw_text(target, input_x, hint_y, &prefix, git_added_fallback)?;
                let prefix_width = monospace_text_width(&prefix, cell_width) as i32;
                let suffix = format!(" · {rest}");
                draw_text(
                    target,
                    input_x + prefix_width,
                    hint_y,
                    &suffix,
                    placeholder_color,
                )?;
            } else {
                let hint_line = format!("{prompt_padding}{hint}");
                draw_text(target, input_x, hint_y, &hint_line, placeholder_color)?;
            }
        }
        if active
            && vim_targets_input
            && matches!(input_mode, InputMode::Insert | InputMode::Replace)
        {
            let (input_row, col_in_visual_row) = input.cursor_visual_row_col(available_input_cols);
            let input_row = input_row.saturating_sub(first_visible_row);
            let input_col = prompt_len + col_in_visual_row;
            let cursor_width = (cell_width / 4).max(2) as u32;
            fill_rounded_rect(
                target,
                PixelRectToRect::rect(
                    input_x + (input_col as i32 * cell_width),
                    layout.input_y + input_row as i32 * line_height,
                    cursor_width,
                    line_height.max(2) as u32,
                ),
                cursor_roundness,
                cursor,
            )?;
        } else if active
            && vim_targets_input
            && matches!(input_mode, InputMode::Normal | InputMode::Visual)
        {
            let cursor_char = input.cursor_char();
            let char_count = input.char_count();
            if char_count > 0 {
                let cursor_index = cursor_char.min(char_count.saturating_sub(1));
                let mut cursor_input = input.clone();
                cursor_input.cursor = cursor_index;
                cursor_input.clear_selection();
                let (input_row, col_in_visual_row) =
                    cursor_input.cursor_visual_row_col(available_input_cols);
                let input_row = input_row.saturating_sub(first_visible_row);
                fill_rect(
                    target,
                    PixelRectToRect::rect(
                        input_x + ((prompt_len + col_in_visual_row) as i32 * cell_width),
                        layout.input_y + input_row as i32 * line_height,
                        cell_width.max(1) as u32,
                        line_height.max(1) as u32,
                    ),
                    cursor,
                )?;
            }
        }
    }

    render_footer_separator(
        target,
        rect,
        layout.statusline_y - BUFFER_FOOTER_SEPARATOR_OFFSET,
        border_color,
        window_effects,
    )?;
    // CONTEXT: modeline stays fully opaque even when window.opacity < 1 so the
    // status row remains readable over acrylic/mica. Pane fills still use
    // window opacity; only this band forces alpha 255.
    fill_rect(
        target,
        PixelRectToRect::rect(
            rect.x(),
            layout.statusline_y,
            rect.width(),
            line_height.max(1) as u32,
        ),
        Color::RGBA(base_background.r, base_background.g, base_background.b, 255),
    )?;
    let statusline_x = rect.x() + 12;
    draw_modeline(
        target,
        &modeline_segments,
        ModelineDraw {
            x: statusline_x,
            y: layout.statusline_y,
            max_width: rect.width().saturating_sub(24),
            default_color: statusline_text_color,
            apply_tokens: active,
            theme_registry,
            user_library,
            acp_connected,
            lsp_server_visible: lsp_server.is_some(),
            lsp_workspace_loaded,
            connected_color: git_added_fallback,
            cell_width,
            line_height: Some(line_height),
        },
    )?;
    if let Some(commandline_y) = layout.commandline_y {
        render_footer_separator(
            target,
            rect,
            commandline_y - BUFFER_FOOTER_SEPARATOR_OFFSET,
            border_color,
            window_effects,
        )?;
    }
    render_command_line_overlay(
        target,
        CommandLineOverlayDraw {
            input: command_line_input,
            rect,
            layout,
            active,
            input_mode,
            paint: CommandLinePaint {
                window_effects,
                background: commandline_background,
                foreground,
                muted,
                cursor,
                cursor_roundness,
                chip_radius: overlay_radius(theme_registry).min(8),
            },
            metrics: CellMetrics {
                cell_width,
                line_height,
            },
        },
    )?;

    let _ = ascent;
    fill_window_surface_rect(
        target,
        PixelRectToRect::rect(
            rect.x(),
            rect.y() + rect.height() as i32 - 2,
            rect.width(),
            1,
        ),
        border_color,
        window_effects,
    )?;
    Ok(())
}

pub(super) fn render_command_line_overlay(
    target: &mut DrawTarget<'_>,
    draw: CommandLineOverlayDraw<'_>,
) -> Result<(), ShellError> {
    let CommandLineOverlayDraw {
        input: command_line_input,
        rect,
        layout,
        active,
        input_mode,
        paint:
            CommandLinePaint {
                window_effects,
                background,
                foreground,
                muted,
                cursor,
                cursor_roundness,
                chip_radius,
            },
        metrics: CellMetrics {
            cell_width,
            line_height,
        },
    } = draw;
    let Some(commandline_y) = layout.commandline_y else {
        return Ok(());
    };
    let Some(input) = command_line_input else {
        return Ok(());
    };
    let chip_background = if background.a == 0 {
        Color::RGBA(foreground.r, foreground.g, foreground.b, 18)
    } else {
        background
    };
    fill_window_surface_rounded_rect(
        target,
        PixelRectToRect::rect(
            rect.x() + 8,
            commandline_y,
            rect.width().saturating_sub(16),
            line_height.max(1) as u32,
        ),
        chip_radius,
        chip_background,
        window_effects,
    )?;
    let text_x = rect.x() + 12;
    let prompt = input.prompt();
    let rendered = if input.text().is_empty() {
        input.placeholder().map_or_else(
            || prompt.to_owned(),
            |placeholder| format!("{prompt}{placeholder}"),
        )
    } else {
        format!("{prompt}{}", input.text())
    };
    let color = if input.text().is_empty() {
        muted
    } else {
        foreground
    };
    draw_text(target, text_x, commandline_y, &rendered, color)?;
    if active {
        let cursor_color = if matches!(input_mode, InputMode::Replace) {
            adjust_color(cursor, -24)
        } else {
            cursor
        };
        let cursor_col = prompt.chars().count() + input.cursor;
        let cursor_width = (cell_width / 4).max(2) as u32;
        fill_rounded_rect(
            target,
            PixelRectToRect::rect(
                text_x + cursor_col as i32 * cell_width.max(1),
                commandline_y,
                cursor_width,
                line_height.max(2) as u32,
            ),
            cursor_roundness,
            cursor_color,
        )?;
    }
    Ok(())
}
