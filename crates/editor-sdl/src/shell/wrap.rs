fn wrap_columns_for_width(width: u32, cell_width: i32) -> usize {
    wrap_columns_for_width_with_fringe(width, cell_width, 1)
}

/// Editor gutter fringe cell count: two cells while a Debug Session is live, else one.
/// Idle Breakpoints still draw in the single cell (may replace git on that line).
pub(super) const fn debug_fringe_cell_count(session_live: bool) -> u32 {
    if session_live { 2 } else { 1 }
}

pub(super) fn editor_fringe_width_px(cell_width: i32, session_live: bool) -> i32 {
    cell_width.max(1) * debug_fringe_cell_count(session_live) as i32
}

fn wrap_columns_for_width_with_fringe(width: u32, cell_width: i32, fringe_cells: u32) -> usize {
    let cell_width = cell_width.max(1) as u32;
    let line_number_width = cell_width.saturating_mul(5);
    let fringe_width = cell_width.saturating_mul(fringe_cells.max(1));
    let right_padding = cell_width;
    let padding = 12u32 + line_number_width + fringe_width + right_padding;
    let available = width.saturating_sub(padding).max(cell_width);
    (available / cell_width).max(1) as usize
}

fn wrap_line_segments(
    map: &LineCharMap,
    first_cols: usize,
    continuation_cols: usize,
) -> Vec<LineWrapSegment> {
    let first_cols = first_cols.max(1);
    let continuation_cols = continuation_cols.max(1);
    let len = map.len();
    if len == 0 {
        return vec![LineWrapSegment {
            start_col: 0,
            end_col: 0,
        }];
    }

    let mut segments = Vec::new();
    let mut start = 0;
    let mut max_cols = first_cols;
    while start < len {
        let remaining = map.display_cols_between(start, len);
        if remaining <= max_cols {
            segments.push(LineWrapSegment {
                start_col: start,
                end_col: len,
            });
            break;
        }

        let mut wrap_limit = start;
        while wrap_limit < len && map.display_cols_between(start, wrap_limit + 1) <= max_cols {
            wrap_limit += 1;
        }
        if wrap_limit == start {
            wrap_limit = (start + 1).min(len);
        }
        let mut break_at = None;
        for idx in (start..wrap_limit).rev() {
            if map.whitespace.get(idx).copied().unwrap_or(false) {
                break_at = Some(idx + 1);
                break;
            }
        }
        if break_at.is_none() {
            for idx in wrap_limit..len {
                if map.whitespace.get(idx).copied().unwrap_or(false) {
                    break_at = Some(idx + 1);
                    break;
                }
            }
        }

        let end = break_at.unwrap_or(wrap_limit);
        segments.push(LineWrapSegment {
            start_col: start,
            end_col: end,
        });
        start = end;
        max_cols = continuation_cols;
    }

    if segments.is_empty() {
        segments.push(LineWrapSegment {
            start_col: 0,
            end_col: 0,
        });
    }

    segments
}

fn wrap_line_segments_for_line(
    line: &str,
    wrap_cols: usize,
    indent_size: usize,
) -> Vec<LineWrapSegment> {
    let tab_width = resolved_tab_width(indent_size);
    let char_map = LineCharMap::with_tab_width(line, tab_width);
    let (leading_indent_cols, _) = leading_whitespace_info(line, tab_width);
    let continuation_indent_cols = leading_indent_cols.saturating_add(indent_size);
    let continuation_cols = wrap_cols.saturating_sub(continuation_indent_cols).max(1);
    wrap_line_segments(&char_map, wrap_cols, continuation_cols)
}

fn line_wrap_row_count(line: &str, wrap_cols: usize, indent_size: usize) -> usize {
    wrap_line_segments_for_line(line, wrap_cols, indent_size)
        .len()
        .max(1)
}

fn segment_index_for_column(segments: &[LineWrapSegment], column: usize) -> usize {
    if segments.is_empty() {
        return 0;
    }
    for (index, segment) in segments.iter().enumerate() {
        if column < segment.end_col || index == segments.len().saturating_sub(1) {
            return index;
        }
    }
    segments.len().saturating_sub(1)
}
