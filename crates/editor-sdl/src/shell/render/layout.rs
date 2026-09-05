const BUFFER_BODY_TOP_PADDING: i32 = 10;
const BUFFER_BODY_BOTTOM_PADDING: i32 = 10;
const BUFFER_STATUSLINE_BOTTOM_PADDING: i32 = 8;
const BUFFER_STATUSLINE_COMMANDLINE_GAP: i32 = 8;
const BUFFER_FOOTER_SEPARATOR_OFFSET: i32 = 6;
const BUFFER_INPUT_BOX_EXTRA_HEIGHT: i32 = 8;
const BUFFER_INPUT_HINT_GAP: i32 = 4;
const BUFFER_INPUT_FOOTER_GAP: i32 = 10;
const BUFFER_OVERLAY_BOTTOM_GAP: i32 = 8;
const INPUT_PANEL_VERTICAL_PADDING: i32 = 10;
const MAX_VISIBLE_INPUT_TEXT_ROWS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BufferFooterLayout {
    pub(super) body_y: i32,
    pub(super) statusline_y: i32,
    pub(super) commandline_y: Option<i32>,
    pub(super) input_y: i32,
    pub(super) input_box_height: i32,
    pub(super) input_hint_gap: i32,
    pub(super) visible_rows: usize,
    pub(super) pane_bottom: i32,
}

#[cfg(test)]
pub(super) fn buffer_footer_layout(
    buffer: &ShellBuffer,
    rect: Rect,
    line_height: i32,
    cell_width: i32,
) -> BufferFooterLayout {
    buffer_footer_layout_with_command_line(buffer, rect, line_height, cell_width, false)
}

pub(super) fn buffer_footer_layout_with_command_line(
    buffer: &ShellBuffer,
    rect: Rect,
    line_height: i32,
    cell_width: i32,
    command_line_visible: bool,
) -> BufferFooterLayout {
    let line_height = line_height.max(1);
    let body_y = rect.y() + BUFFER_BODY_TOP_PADDING;
    let command_line_reserved = if command_line_visible {
        line_height + BUFFER_STATUSLINE_COMMANDLINE_GAP
    } else {
        0
    };
    let statusline_y = rect.y() + rect.height() as i32
        - line_height
        - command_line_reserved
        - BUFFER_STATUSLINE_BOTTOM_PADDING;
    let commandline_y = command_line_visible
        .then_some(statusline_y + line_height + BUFFER_STATUSLINE_COMMANDLINE_GAP);
    let available_input_cols = if cell_width > 0 {
        ((rect.width() as i32 - 16) / cell_width).max(1) as usize
    } else {
        0
    };
    let (input_text_lines, has_hint) = buffer
        .standalone_input_field()
        .map(|input| {
            let line_count = if available_input_cols > 0 {
                input.visual_line_count(available_input_cols)
            } else {
                input.text_line_count()
            };
            (line_count, input.hint().is_some())
        })
        .unwrap_or((0, false));
    let input_hint_gap = if has_hint { BUFFER_INPUT_HINT_GAP } else { 0 };
    let input_footer_gap = if has_hint { BUFFER_INPUT_FOOTER_GAP } else { 0 };
    let fixed_input_reserved = if input_text_lines > 0 {
        input_hint_gap + i32::from(has_hint) * line_height + input_footer_gap
    } else {
        0
    };
    let min_input_y = body_y + BUFFER_BODY_BOTTOM_PADDING + line_height;
    let max_input_reserved = statusline_y.saturating_sub(min_input_y).max(0);
    let capped_input_text_lines = input_text_lines.min(MAX_VISIBLE_INPUT_TEXT_ROWS);
    let desired_input_box_height = if capped_input_text_lines > 0 {
        (line_height * capped_input_text_lines as i32 + BUFFER_INPUT_BOX_EXTRA_HEIGHT)
            .max(line_height)
    } else {
        0
    };
    let max_input_box_height = max_input_reserved.saturating_sub(fixed_input_reserved);
    let input_box_height = if input_text_lines > 0 && max_input_box_height > 0 {
        desired_input_box_height.clamp(line_height.min(max_input_box_height), max_input_box_height)
    } else {
        0
    };
    let input_reserved = if input_text_lines > 0 {
        input_box_height + fixed_input_reserved
    } else {
        0
    };
    let input_y = statusline_y - input_reserved;
    let visible_body_height = (input_y - body_y - BUFFER_BODY_BOTTOM_PADDING).max(line_height);
    let visible_rows = (visible_body_height / line_height).max(1) as usize;
    BufferFooterLayout {
        body_y,
        statusline_y,
        commandline_y,
        input_y,
        input_box_height,
        input_hint_gap,
        visible_rows,
        pane_bottom: input_y - BUFFER_OVERLAY_BOTTOM_GAP,
    }
}

fn visible_input_text_rows(input_box_height: i32, line_height: i32) -> usize {
    if input_box_height <= 0 {
        return 0;
    }
    let rows = input_box_height
        .saturating_sub(BUFFER_INPUT_BOX_EXTRA_HEIGHT)
        .max(line_height)
        .saturating_div(line_height.max(1)) as usize;
    rows.min(MAX_VISIBLE_INPUT_TEXT_ROWS)
}

pub(super) fn buffer_visible_rows_for_height(
    buffer: &ShellBuffer,
    height: u32,
    line_height: i32,
    command_line_visible: bool,
) -> usize {
    buffer_footer_layout_with_command_line(
        buffer,
        PixelRectToRect::rect(0, 0, 1, height),
        line_height,
        0,
        command_line_visible,
    )
    .visible_rows
}

fn render_footer_separator(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    y: i32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_window_surface_rect(
        target,
        PixelRectToRect::rect(rect.x() + 8, y, rect.width().saturating_sub(16), 1),
        color,
        window_effects,
    )
}

fn buffer_visible_headerline_row_count(
    buffer: &ShellBuffer,
    user_library: &dyn UserLibrary,
    visible_rows: usize,
    typing_active: bool,
) -> usize {
    buffer_context_overlay_snapshot(buffer, true, typing_active, user_library)
        .map(|snapshot| visible_headerline_row_count(&snapshot.headerline_lines, visible_rows))
        .unwrap_or(0)
}

pub(super) fn image_buffer_viewport_rect(rect: Rect, layout: BufferFooterLayout) -> Option<Rect> {
    let x = rect.x().saturating_add(8);
    let y = layout.body_y;
    let width = rect.width().saturating_sub(16);
    let height = layout.pane_bottom.saturating_sub(y);
    (width > 0 && height > 0).then(|| Rect::new(x, y, width, height as u32))
}

pub(super) fn centered_image_draw_rect(
    viewport: Rect,
    image_width: u32,
    image_height: u32,
    zoom: f32,
) -> Option<Rect> {
    if image_width == 0 || image_height == 0 || viewport.width() == 0 || viewport.height() == 0 {
        return None;
    }
    let fit_scale = (viewport.width() as f32 / image_width as f32)
        .min(viewport.height() as f32 / image_height as f32);
    let scale = (fit_scale * zoom).max(0.000_1);
    let draw_width = ((image_width as f32 * scale).round() as u32).max(1);
    let draw_height = ((image_height as f32 * scale).round() as u32).max(1);
    let x = viewport.x() + (viewport.width() as i32 - draw_width as i32) / 2;
    let y = viewport.y() + (viewport.height() as i32 - draw_height as i32) / 2;
    Some(Rect::new(x, y, draw_width, draw_height))
}

pub(super) fn render_image_buffer_body(
    target: &mut DrawTarget<'_>,
    buffer: &ShellBuffer,
    rect: Rect,
    layout: BufferFooterLayout,
    theme_registry: Option<&ThemeRegistry>,
    base_background: Color,
) -> Result<(), ShellError> {
    let window_effects = current_window_effect_settings(theme_registry);
    let Some(state) = buffer.image_state() else {
        return Ok(());
    };
    if state.mode != ImageBufferMode::Rendered {
        return Ok(());
    }
    let Some(viewport) = image_buffer_viewport_rect(rect, layout) else {
        return Ok(());
    };
    let viewport_background = theme_color(
        theme_registry,
        "ui.panel.background",
        adjust_color(
            base_background,
            if is_dark_color(base_background) {
                4
            } else {
                -4
            },
        ),
    );
    fill_window_surface_rect(target, viewport, viewport_background, window_effects)?;
    let Some(draw_rect) = centered_image_draw_rect(
        viewport,
        state.decoded.width,
        state.decoded.height,
        state.zoom,
    ) else {
        return Ok(());
    };
    draw_image(
        target,
        draw_rect,
        state.decoded.width,
        state.decoded.height,
        Arc::clone(&state.decoded.pixels),
        Some(viewport),
    )?;
    Ok(())
}

pub(super) fn render_pdf_buffer_body(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    layout: BufferFooterLayout,
    theme_registry: Option<&ThemeRegistry>,
    base_background: Color,
) -> Result<(), ShellError> {
    let window_effects = current_window_effect_settings(theme_registry);
    let Some(viewport) = image_buffer_viewport_rect(rect, layout) else {
        return Ok(());
    };
    let viewport_background = theme_color(
        theme_registry,
        "ui.panel.background",
        adjust_color(
            base_background,
            if is_dark_color(base_background) {
                4
            } else {
                -4
            },
        ),
    );
    fill_window_surface_rect(target, viewport, viewport_background, window_effects)
}

pub(super) fn autocomplete_preview_lines(
    entry: Option<&AutocompleteEntry>,
    token: &str,
    max_columns: usize,
    max_lines: usize,
    token_icon: &str,
) -> Vec<String> {
    let max_lines = max_lines.max(1);
    let Some(entry) = entry else {
        return wrap_overlay_text(
            &format!("{token_icon} {token}\n\nSelect a completion to preview details."),
            max_columns,
            max_lines,
        );
    };
    let mut lines = Vec::new();
    lines.extend(wrap_overlay_text(
        &format!("{} {}", entry.item_icon, entry.label),
        max_columns,
        max_lines,
    ));
    if lines.len() < max_lines {
        let meta = entry
            .detail
            .as_deref()
            .filter(|detail| !detail.is_empty())
            .map(|detail| {
                format!(
                    "{} {} · {detail}",
                    entry.provider_icon, entry.provider_label
                )
            })
            .unwrap_or_else(|| format!("{} {}", entry.provider_icon, entry.provider_label));
        lines.extend(wrap_overlay_text(
            &meta,
            max_columns,
            max_lines - lines.len(),
        ));
    }
    if lines.len() < max_lines {
        lines.push(String::new());
    }
    if lines.len() < max_lines {
        let body = entry
            .documentation
            .as_deref()
            .filter(|documentation| !documentation.trim().is_empty())
            .unwrap_or("No documentation available for this completion.");
        lines.extend(wrap_overlay_text(
            body,
            max_columns,
            max_lines - lines.len(),
        ));
    }
    lines.truncate(max_lines);
    lines
}

pub(super) fn wrap_overlay_lines(
    lines: &[String],
    max_columns: usize,
    max_lines: usize,
) -> Vec<String> {
    if max_lines == 0 {
        return Vec::new();
    }
    let mut wrapped = Vec::new();
    for line in lines {
        if wrapped.len() >= max_lines {
            break;
        }
        wrapped.extend(wrap_overlay_text(
            line,
            max_columns,
            max_lines.saturating_sub(wrapped.len()),
        ));
    }
    wrapped.truncate(max_lines);
    wrapped
}

pub(super) fn wrap_overlay_text(text: &str, max_columns: usize, max_lines: usize) -> Vec<String> {
    if max_lines == 0 {
        return Vec::new();
    }
    let max_columns = max_columns.max(1);
    let mut wrapped = Vec::new();
    for raw_line in text.lines() {
        if wrapped.len() >= max_lines {
            break;
        }
        if raw_line.is_empty() {
            wrapped.push(String::new());
            continue;
        }
        let mut remaining = raw_line;
        while !remaining.is_empty() && wrapped.len() < max_lines {
            if remaining.chars().count() <= max_columns {
                wrapped.push(remaining.to_owned());
                break;
            }
            let mut split_at = 0usize;
            let mut last_whitespace = None;
            let mut columns = 0usize;
            for (byte_index, character) in remaining.char_indices() {
                columns += 1;
                if character.is_whitespace() {
                    last_whitespace = Some(byte_index);
                }
                split_at = byte_index + character.len_utf8();
                if columns >= max_columns {
                    break;
                }
            }
            let split_at = last_whitespace
                .filter(|index| *index > 0)
                .unwrap_or(split_at);
            let (head, tail) = remaining.split_at(split_at);
            wrapped.push(head.trim_end().to_owned());
            remaining = tail.trim_start();
        }
    }
    if wrapped.is_empty() {
        wrapped.push(String::new());
    }
    wrapped.truncate(max_lines);
    wrapped
}

pub(super) fn statusline_icon_segments<'a>(
    text: &'a str,
    icons: &[&'a str],
) -> Vec<(&'a str, bool)> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut remaining = text;
    let mut segments = Vec::new();
    while !remaining.is_empty() {
        let next_icon = icons
            .iter()
            .filter_map(|icon| remaining.find(icon).map(|index| (index, *icon)))
            .min_by_key(|(index, _)| *index);
        let Some((index, icon)) = next_icon else {
            segments.push((remaining, false));
            break;
        };
        if index > 0 {
            let (before, after) = remaining.split_at(index);
            segments.push((before, false));
            remaining = after;
            continue;
        }
        let after = &remaining[icon.len()..];
        segments.push((icon, true));
        remaining = after;
    }
    segments
}

pub(super) fn statusline_icon_colors(
    statusline: &str,
    user_library: &dyn UserLibrary,
    acp_connected: bool,
    lsp_server_visible: bool,
    lsp_workspace_loaded: bool,
    theme_registry: Option<&ThemeRegistry>,
    connected_color: Color,
) -> Vec<(&'static str, Color)> {
    let acp_icon = editor_icons::symbols::fa::FA_CONNECTDEVELOP;
    let lsp_icon = user_library.statusline_lsp_connected_icon();
    let error_icon = user_library.statusline_lsp_error_icon();
    let warning_icon = user_library.statusline_lsp_warning_icon();
    let mut icon_colors = Vec::new();
    if acp_connected && statusline.contains(acp_icon) {
        icon_colors.push((acp_icon, connected_color));
    }
    if lsp_server_visible && lsp_workspace_loaded && statusline.contains(lsp_icon) {
        icon_colors.push((lsp_icon, connected_color));
    }
    if statusline.contains(error_icon) {
        icon_colors.push((
            error_icon,
            diagnostic_color(LspDiagnosticSeverity::Error, theme_registry),
        ));
    }
    if statusline.contains(warning_icon) {
        icon_colors.push((
            warning_icon,
            diagnostic_color(LspDiagnosticSeverity::Warning, theme_registry),
        ));
    }
    icon_colors
}
