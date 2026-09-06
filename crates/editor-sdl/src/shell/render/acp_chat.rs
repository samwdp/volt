const ACP_CHAT_ROUNDED_OPTION: &str = "acp.chat.rounded";

pub(super) fn acp_chat_rounded(theme_registry: Option<&ThemeRegistry>) -> bool {
    theme_registry
        .and_then(|registry| registry.resolve_bool(ACP_CHAT_ROUNDED_OPTION))
        .unwrap_or(true)
}

pub(super) fn acp_chat_corner_radius(theme_registry: Option<&ThemeRegistry>) -> u32 {
    if acp_chat_rounded(theme_registry) {
        shared_corner_radius(theme_registry)
    } else {
        0
    }
}

fn acp_chat_origin_x(
    line: &AcpRenderedTextLine,
    body_x: i32,
    wrap_cols: usize,
    cell_width: i32,
) -> i32 {
    match line.align {
        AcpChatAlign::End => {
            let bubble_cols = acp_chat_bubble_cols(wrap_cols);
            let shift = wrap_cols.saturating_sub(bubble_cols) as i32 * cell_width.max(1);
            body_x + shift
        }
        AcpChatAlign::Start | AcpChatAlign::Full => body_x,
    }
}

fn acp_chat_bubble_width_px(
    line: &AcpRenderedTextLine,
    wrap_cols: usize,
    cell_width: i32,
    body_width: u32,
) -> u32 {
    match line.align {
        AcpChatAlign::Full => body_width,
        AcpChatAlign::Start | AcpChatAlign::End => {
            (acp_chat_bubble_cols(wrap_cols) as i32 * cell_width.max(1)).max(1) as u32
        }
    }
}

fn acp_bubble_remaining_rows(
    pane: &AcpPaneState,
    line_index: usize,
    segment_index: usize,
    bubble_group: u32,
    wrap_cols: usize,
) -> usize {
    let mut rows = 0usize;
    for (index, rendered_line) in pane.render_lines.iter().enumerate().skip(line_index) {
        let AcpRenderedLine::Text(line) = rendered_line else {
            break;
        };
        if !line.bubble || line.bubble_group != bubble_group {
            break;
        }
        let segments = acp_rendered_text_segments(line, wrap_cols);
        if index == line_index {
            rows = rows.saturating_add(segments.len().saturating_sub(segment_index).max(1));
        } else {
            rows = rows.saturating_add(segments.len().max(1));
        }
    }
    rows.max(1)
}

pub(super) fn acp_draw_prefix_segments(
    target: &mut DrawTarget<'_>,
    draw: AcpPrefixDraw<'_>,
) -> Result<(), ShellError> {
    let AcpPrefixDraw {
        x,
        y,
        segments,
        spinner_frame,
        theme_registry,
        foreground,
        muted,
        accent,
        cell_width,
    } = draw;
    let mut draw_x = x;
    for segment in segments {
        let text = if segment.animate {
            spinner_frame
        } else {
            segment.text.as_str()
        };
        let color = acp_color(segment.role, theme_registry, foreground, muted, accent);
        draw_text(target, draw_x, y, text, color)?;
        draw_x += monospace_text_width(text, cell_width) as i32;
    }
    Ok(())
}

pub(super) fn acp_prefix_columns(segments: &[AcpRenderedSegment], spinner_frame: &str) -> usize {
    segments
        .iter()
        .map(|segment| {
            if segment.animate {
                spinner_frame.chars().count()
            } else {
                segment.text.chars().count()
            }
        })
        .sum()
}

pub(super) fn acp_color(
    role: AcpColorRole,
    theme_registry: Option<&ThemeRegistry>,
    foreground: Color,
    muted: Color,
    accent: Color,
) -> Color {
    match role {
        AcpColorRole::Default => foreground,
        AcpColorRole::Muted => muted,
        AcpColorRole::Accent => accent,
        AcpColorRole::Success => theme_color(
            theme_registry,
            "git.status.entry.added",
            Color::RGB(108, 193, 118),
        ),
        AcpColorRole::Warning => theme_color(
            theme_registry,
            "ui.notification.warning",
            Color::RGB(209, 154, 102),
        ),
        AcpColorRole::Error => theme_color(
            theme_registry,
            "ui.notification.error",
            Color::RGB(224, 107, 117),
        ),
        AcpColorRole::PriorityHigh => theme_color(
            theme_registry,
            "ui.notification.error",
            Color::RGB(224, 107, 117),
        ),
        AcpColorRole::PriorityMedium => theme_color(
            theme_registry,
            "ui.notification.warning",
            Color::RGB(209, 154, 102),
        ),
        AcpColorRole::PriorityLow => theme_color(
            theme_registry,
            "ui.notification.info",
            Color::RGB(110, 170, 255),
        ),
    }
}

pub(super) fn acp_spinner_frame() -> &'static str {
    const FRAMES: [&str; 6] = ["◜", "◠", "◝", "◞", "◡", "◟"];
    let frame = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| ((duration.as_millis() / 120) % FRAMES.len() as u128) as usize)
        .unwrap_or(0);
    FRAMES[frame]
}

pub(super) fn acp_slice_chars(text: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    let mut start_byte = text.len();
    let mut end_byte = text.len();
    let mut seen = 0usize;
    for (index, character) in text.char_indices() {
        if seen == start {
            start_byte = index;
        }
        if seen == end {
            end_byte = index;
            break;
        }
        seen = seen.saturating_add(1);
        if seen == text.chars().count() {
            end_byte = text.len();
        }
        let _ = character;
    }
    if start == 0 {
        start_byte = 0;
    }
    if end >= text.chars().count() {
        end_byte = text.len();
    }
    text.get(start_byte..end_byte)
        .unwrap_or_default()
        .to_owned()
}

pub(super) fn block_cursor_text_overlay(
    query: CursorOverlayQuery<'_>,
) -> Option<CursorTextOverlay> {
    let CursorOverlayQuery {
        x,
        line,
        char_map,
        segment,
        line_index,
        cursor,
        color,
        cell_width,
    } = query;
    let cursor_col = char_map.cursor_anchor_col(cursor.column);
    let color = color?;
    if line_index != cursor.line || cursor_col < segment.start_col || cursor_col >= segment.end_col
    {
        return None;
    }
    if cursor_col >= char_map.len() {
        return None;
    }
    let text = char_map.display_text_for_range(line, cursor_col, cursor_col.saturating_add(1));
    (!text.is_empty()).then_some(CursorTextOverlay {
        draw_x: x
            + (char_map.display_cols_between(segment.start_col, cursor_col) as i32 * cell_width),
        text,
        color,
    })
}

pub(super) fn draw_buffer_text(
    target: &mut DrawTarget<'_>,
    run: BufferTextRun<'_>,
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(), ShellError> {
    let BufferTextRun {
        x,
        y,
        line,
        segment,
        char_map,
        line_syntax_spans,
        default_color,
        cell_width,
    } = run;
    let segment_end_col = segment.end_col.min(char_map.len());
    let segment_start_col = segment.start_col.min(segment_end_col);
    let segment_text = char_map.slice(line, segment_start_col, segment_end_col);
    let segment_base_byte = char_map
        .bytes
        .get(segment_start_col)
        .copied()
        .unwrap_or_default();
    let segment_byte_offsets = &char_map.bytes[segment_start_col..=segment_end_col];
    let mut clipped_spans = Vec::new();
    if let Some(line_syntax_spans) = line_syntax_spans {
        for span in line_syntax_spans {
            let start = span.start.max(segment.start_col);
            let end = span.end.min(segment.end_col);
            if start < end {
                clipped_spans.push(LineSyntaxSpan {
                    start: start - segment.start_col,
                    end: end - segment.start_col,
                    capture_name: span.capture_name.clone(),
                    theme_token: span.theme_token.clone(),
                });
            }
        }
    }
    let clipped_spans = if clipped_spans.is_empty() {
        None
    } else {
        Some(clipped_spans.as_slice())
    };

    let mut draw_x = x;
    let mut segment_char_offset = 0usize;
    for (colored_segment, color, style) in line_color_segments(
        segment_text,
        clipped_spans,
        theme_registry,
        default_color,
        segment_byte_offsets,
        segment_base_byte,
    ) {
        let colored_segment_chars = colored_segment.chars().count();
        if colored_segment.is_empty() {
            segment_char_offset = segment_char_offset.saturating_add(colored_segment_chars);
            continue;
        }
        let rendered_segment = char_map.display_text_for_range(
            line,
            segment_start_col + segment_char_offset,
            segment_start_col + segment_char_offset + colored_segment_chars,
        );
        if style == TextStyle::plain() {
            draw_text(target, draw_x, y, &rendered_segment, color)?;
        } else {
            let mut character_x = draw_x;
            for character in rendered_segment.chars() {
                let character_text = character.to_string();
                draw_styled_text(target, character_x, y, &character_text, color, style)?;
                character_x += monospace_text_width(&character_text, cell_width) as i32;
            }
        }
        draw_x += monospace_text_width(&rendered_segment, cell_width) as i32;
        segment_char_offset = segment_char_offset.saturating_add(colored_segment_chars);
    }
    Ok(())
}

pub(super) struct GhostTextSegmentDraw<'a> {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) segment: LineWrapSegment,
    pub(super) char_map: &'a LineCharMap,
    pub(super) line_len: usize,
    pub(super) ghost_text: Option<&'a str>,
    pub(super) color: Color,
    pub(super) cell_width: i32,
}

pub(super) fn draw_line_ghost_text_for_segment(
    target: &mut DrawTarget<'_>,
    draw: GhostTextSegmentDraw<'_>,
) -> Result<(), ShellError> {
    let Some(ghost_text) = draw.ghost_text.filter(|text| !text.is_empty()) else {
        return Ok(());
    };
    let visible_end = draw.segment.end_col.min(draw.line_len);
    if visible_end < draw.line_len {
        return Ok(());
    }
    let visible_cols = draw
        .char_map
        .display_cols_between(draw.segment.start_col, visible_end);
    // Leave one monospace cell between the closing delimiter and the ghost text.
    let draw_x = draw.x + visible_cols as i32 * draw.cell_width + draw.cell_width;
    draw_text(target, draw_x, draw.y, ghost_text, draw.color)
}

fn headerline_max_rows(visible_rows: usize) -> usize {
    visible_rows.saturating_sub(1)
}

pub(super) fn visible_headerline_row_count(lines: &[String], visible_rows: usize) -> usize {
    let max_rows = headerline_max_rows(visible_rows);
    if max_rows == 0 {
        return 0;
    }
    lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .count()
        .min(max_rows)
}

pub(super) fn visible_headerline_lines(lines: &[String], visible_rows: usize) -> Vec<&str> {
    let max_rows = headerline_max_rows(visible_rows);
    if max_rows == 0 {
        return Vec::new();
    }
    let mut kept: Vec<&str> = lines
        .iter()
        .map(String::as_str)
        .filter(|line| !line.trim().is_empty())
        .collect();
    let extra = kept.len().saturating_sub(max_rows);
    if extra > 0 {
        kept.drain(..extra);
    }
    kept
}

pub(super) fn line_color_segments(
    line: &str,
    line_syntax_spans: Option<&[LineSyntaxSpan]>,
    theme_registry: Option<&ThemeRegistry>,
    default_color: Color,
    column_byte_offsets: &[usize],
    base_byte: usize,
) -> Vec<(String, Color, TextStyle)> {
    let Some(line_syntax_spans) = line_syntax_spans else {
        return vec![(line.to_owned(), default_color, TextStyle::plain())];
    };

    let relevant_spans = line_syntax_spans
        .iter()
        .filter_map(|span| {
            let start = column_to_relative_byte_offset(
                line.len(),
                column_byte_offsets,
                base_byte,
                span.start,
            );
            let end = column_to_relative_byte_offset(
                line.len(),
                column_byte_offsets,
                base_byte,
                span.end,
            );
            if start >= end {
                return None;
            }

            Some((start, end, span.theme_token.as_ref()))
        })
        .collect::<Vec<_>>();
    if relevant_spans.is_empty() {
        return vec![(line.to_owned(), default_color, TextStyle::plain())];
    }

    let mut breakpoints = vec![0, line.len()];
    for (start, end, _) in &relevant_spans {
        breakpoints.push(*start);
        breakpoints.push(*end);
    }
    breakpoints.sort_unstable();
    breakpoints.dedup();

    let mut segments = Vec::new();
    for window in breakpoints.windows(2) {
        let start = window[0];
        let end = window[1];
        if start >= end {
            continue;
        }
        let Some(text) = line.get(start..end) else {
            continue;
        };
        let token_style = relevant_spans
            .iter()
            .filter(|(span_start, span_end, token)| {
                start >= *span_start
                    && end <= *span_end
                    && *token != super::HOVER_SIGNATURE_ACTIVE_PARAMETER_TOKEN
            })
            .min_by_key(|(span_start, span_end, token)| {
                (
                    span_end.saturating_sub(*span_start),
                    usize::from(!theme_token_has_render_priority(token)),
                )
            })
            .and_then(|(_, _, token)| {
                theme_registry.and_then(|registry| registry.resolve_style(token))
            });
        let color = token_style
            .map(|token_style| to_sdl_color(token_style.color))
            .unwrap_or(default_color);
        let mut style = token_style
            .map(|token_style| text_style_from_theme_style(token_style.style))
            .unwrap_or_else(TextStyle::plain);
        let has_active_parameter_emphasis =
            relevant_spans.iter().any(|(span_start, span_end, token)| {
                start >= *span_start
                    && end <= *span_end
                    && *token == super::HOVER_SIGNATURE_ACTIVE_PARAMETER_TOKEN
            });
        if has_active_parameter_emphasis {
            let emphasis_bold = theme_registry
                .and_then(|registry| {
                    registry.resolve_style(super::HOVER_SIGNATURE_ACTIVE_PARAMETER_TOKEN)
                })
                .map(|token_style| token_style.style.bold)
                .unwrap_or(true);
            if emphasis_bold {
                style = TextStyle::new(true, style.italic);
            }
        }
        segments.push((text.to_owned(), color, style));
    }

    if segments.is_empty() {
        vec![(line.to_owned(), default_color, TextStyle::plain())]
    } else {
        segments
    }
}

fn theme_token_has_render_priority(token: &str) -> bool {
    token.starts_with("rainbow.paren.")
}

pub(super) fn column_to_relative_byte_offset(
    line_len: usize,
    column_byte_offsets: &[usize],
    base_byte: usize,
    column: usize,
) -> usize {
    column_byte_offsets
        .get(column)
        .copied()
        .unwrap_or_else(|| base_byte.saturating_add(line_len))
        .saturating_sub(base_byte)
        .min(line_len)
}

pub(super) fn draw_diagnostic_undercurl(
    target: &mut DrawTarget<'_>,
    x: i32,
    y: i32,
    width: i32,
    line_height: i32,
    color: Color,
) -> Result<(), ShellError> {
    if width <= 0 || line_height <= 0 {
        return Ok(());
    }
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::Undercurl {
            x,
            y,
            width: width as u32,
            line_height: line_height as u32,
            color: to_render_color(color),
        }),
    }
    Ok(())
}

pub(super) fn selection_columns_for_line(
    range: TextRange,
    line_index: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    let range = range.normalized();
    if line_index < range.start().line || line_index > range.end().line {
        return None;
    }

    let start = if line_index == range.start().line {
        range.start().column
    } else {
        0
    };
    let end = if line_index == range.end().line {
        range.end().column
    } else {
        line_len
    };
    let start = start.min(line_len);
    let end = end.min(line_len);
    (start < end).then_some((start, end))
}

pub(super) fn selection_columns_for_visual(
    selection: VisualSelection,
    line_index: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    match selection {
        VisualSelection::Range(range) => selection_columns_for_line(range, line_index, line_len),
        VisualSelection::Block(block) => {
            if line_index < block.start_line || line_index > block.end_line {
                return None;
            }
            let start = block.start_col.min(line_len);
            let end = block.end_col.min(line_len);
            (start < end).then_some((start, end))
        }
    }
}

pub(super) fn multicursor_ranges_for_line(
    state: &MulticursorState,
    input_mode: InputMode,
    line_index: usize,
    line_len: usize,
) -> Vec<(usize, usize)> {
    let Some((start_offset, end_offset)) = multicursor_selection_offsets(state, input_mode) else {
        return Vec::new();
    };
    let start_text = state
        .match_text
        .chars()
        .take(start_offset)
        .collect::<String>();
    let end_text = state
        .match_text
        .chars()
        .take(end_offset)
        .collect::<String>();
    state
        .ranges
        .iter()
        .filter_map(|range| {
            selection_columns_for_visual(
                VisualSelection::Range(TextRange::new(
                    advance_point_by_text(range.start(), &start_text),
                    advance_point_by_text(range.start(), &end_text),
                )),
                line_index,
                line_len,
            )
        })
        .collect()
}

pub(super) fn multicursor_cursor_points(state: &MulticursorState) -> Vec<TextPoint> {
    let prefix = state
        .match_text
        .chars()
        .take(state.cursor_offset)
        .collect::<String>();
    state
        .ranges
        .iter()
        .map(|range| advance_point_by_text(range.start(), &prefix))
        .collect()
}

fn relative_byte_column_to_char_column(line: &str, byte_column: usize) -> usize {
    let mut bytes = 0usize;
    let mut chars = 0usize;
    for character in line.chars() {
        if bytes >= byte_column {
            break;
        }
        bytes = bytes.saturating_add(character.len_utf8());
        chars = chars.saturating_add(1);
    }
    chars
}

pub(super) fn index_syntax_lines(
    snapshot: SyntaxSnapshot,
    text: &TextBuffer,
) -> IndexedSyntaxLines {
    let mut syntax_lines = BTreeMap::new();
    for span in snapshot.highlight_spans {
        let start_line = span.start_position.line;
        let end_line = span.end_position.line;
        let mut capture_name = span.capture_name;
        let mut theme_token = span.theme_token;
        for line_index in start_line..=end_line {
            let Some(line_text) = text.line(line_index) else {
                continue;
            };
            let Some(line_start_byte) = text.line_start_byte(line_index) else {
                continue;
            };
            let start_byte = if line_index == start_line {
                span.start_byte.saturating_sub(line_start_byte)
            } else {
                0
            };
            let end_byte = if line_index == end_line {
                span.end_byte.saturating_sub(line_start_byte)
            } else {
                line_text.len()
            };
            let start =
                relative_byte_column_to_char_column(&line_text, start_byte.min(line_text.len()));
            let end =
                relative_byte_column_to_char_column(&line_text, end_byte.min(line_text.len()));
            if start >= end {
                continue;
            }
            syntax_lines
                .entry(line_index)
                .or_insert_with(Vec::new)
                .push(LineSyntaxSpan {
                    start,
                    end,
                    capture_name: if line_index == end_line {
                        std::mem::take(&mut capture_name)
                    } else {
                        capture_name.clone()
                    },
                    theme_token: if line_index == end_line {
                        std::mem::take(&mut theme_token)
                    } else {
                        theme_token.clone()
                    },
                });
        }
    }

    syntax_lines
}

pub(super) fn clamp_to_char_boundary(text: &str, index: usize) -> usize {
    let mut clamped = index.min(text.len());
    while clamped > 0 && !text.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FontRole {
    Primary,
    Icon(usize),
    Emoji,
}

#[derive(Debug, Clone)]
pub(super) struct FontRun {
    role: FontRole,
    text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PrimaryTextRenderMode {
    Normal,
    Ligature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PrimaryTextRun {
    pub(super) render_mode: PrimaryTextRenderMode,
    pub(super) text: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ShapedGlyph {
    pub(super) cluster: usize,
    pub(super) glyph_id: u16,
    pub(super) x_advance: f32,
    pub(super) x_offset: f32,
    pub(super) y_offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ShapedRun {
    pub(super) glyphs: Vec<ShapedGlyph>,
    pub(super) total_advance: f32,
}

pub(super) fn shaped_run_uses_cell_grid(text: &str, shaped: &ShapedRun) -> bool {
    shaped.glyphs.len() == text.chars().count()
}

pub(super) fn shaped_run_preserves_monospace_layout(
    text: &str,
    shaped: &ShapedRun,
    cell_width: i32,
) -> bool {
    shaped_run_uses_cell_grid(text, shaped)
        || (shaped.total_advance - monospace_text_width(text, cell_width) as f32).abs() <= 1.0
}

pub(super) fn is_private_use_character(character: char) -> bool {
    matches!(
        character as u32,
        0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD
    )
}

pub(super) fn is_symbol_like_character(character: char) -> bool {
    matches!(
        character as u32,
        0x2190..=0x21FF
            | 0x2300..=0x23FF
            | 0x2500..=0x257F
            | 0x2580..=0x259F
            | 0x25A0..=0x25FF
            | 0x2600..=0x27BF
            | 0x2B00..=0x2BFF
    )
}

pub(super) fn is_emoji_character(character: char) -> bool {
    matches!(
        character as u32,
        0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0xFE00..=0xFE0F
    )
}

pub(super) fn resolve_font_role_for_char(
    icon_font_index: Option<usize>,
    primary_has_glyph: bool,
    prefers_icon_font: bool,
    emoji_has_glyph: bool,
    character: char,
) -> FontRole {
    // Emoji characters get their own font role when available
    if emoji_has_glyph && is_emoji_character(character) {
        return FontRole::Emoji;
    }
    if let Some(index) = icon_font_index
        && (prefers_icon_font
            || is_private_use_character(character)
            || is_symbol_like_character(character))
    {
        return FontRole::Icon(index);
    }
    if primary_has_glyph {
        return FontRole::Primary;
    }
    icon_font_index
        .map(FontRole::Icon)
        .unwrap_or(FontRole::Primary)
}

pub(super) fn font_role_for_char(fonts: &FontSet<'_>, character: char) -> FontRole {
    resolve_font_role_for_char(
        fonts.icon_font_index_for_char(character),
        fonts.primary().find_glyph(character).is_some(),
        fonts.prefers_icon_font(character),
        fonts.emoji_font_has_char(character),
        character,
    )
}

pub(super) fn font_runs(text: &str, fonts: &FontSet<'_>) -> Vec<FontRun> {
    if text.is_empty() {
        return Vec::new();
    }
    if fonts.icon_fonts().is_empty() || text.is_ascii() {
        return vec![FontRun {
            role: FontRole::Primary,
            text: text.to_owned(),
        }];
    }
    let mut runs = Vec::new();
    let mut current_role = FontRole::Primary;
    let mut current_text = String::new();
    for character in text.chars() {
        if is_zero_width_display_character(character) {
            if current_role == FontRole::Emoji && !current_text.is_empty() {
                current_text.push(character);
            }
            continue;
        }
        let next_role = font_role_for_char(fonts, character);
        if next_role != current_role && !current_text.is_empty() {
            runs.push(FontRun {
                role: current_role,
                text: std::mem::take(&mut current_text),
            });
        }
        current_role = next_role;
        current_text.push(character);
    }
    if !current_text.is_empty() {
        runs.push(FontRun {
            role: current_role,
            text: current_text,
        });
    }
    runs
}
