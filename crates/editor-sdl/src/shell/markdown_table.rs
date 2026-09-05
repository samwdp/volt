#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkdownTableAlignment {
    None,
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownTableLine {
    prefix: String,
    cells: Vec<String>,
    is_delimiter: bool,
    alignments: Vec<MarkdownTableAlignment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownTable {
    start_line: usize,
    rows: Vec<MarkdownTableLine>,
    column_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MarkdownTableCursorTarget {
    row_index: usize,
    cell_index: usize,
    content_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownTableRender {
    lines: Vec<String>,
    widths: Vec<usize>,
}

fn markdown_pretty_active_for_buffer(buffer: &ShellBuffer) -> bool {
    buffer.markdown_pretty_enabled().unwrap_or(true)
}

fn detect_markdown_table(buffer: &ShellBuffer) -> Option<MarkdownTable> {
    if !markdown_pretty_active_for_buffer(buffer) {
        return None;
    }
    if buffer.language_id() != Some("markdown") || buffer.line_count() < 2 {
        return None;
    }

    let cursor_line = buffer
        .cursor_row()
        .min(buffer.line_count().saturating_sub(1));
    parse_markdown_table_line(&buffer.text.line(cursor_line)?)
        .as_ref()
        .map(|_| ())?;

    let mut start_line = cursor_line;
    while start_line > 0 {
        let previous = start_line.saturating_sub(1);
        let Some(line) = buffer.text.line(previous) else {
            break;
        };
        if parse_markdown_table_line(&line).is_none() {
            break;
        }
        start_line = previous;
    }

    let mut end_line = cursor_line;
    while end_line + 1 < buffer.line_count() {
        let next = end_line + 1;
        let Some(line) = buffer.text.line(next) else {
            break;
        };
        if parse_markdown_table_line(&line).is_none() {
            break;
        }
        end_line = next;
    }

    let mut rows = Vec::new();
    let mut column_count = 0usize;
    for line_index in start_line..=end_line {
        let (prefix, cells) = parse_markdown_table_line(&buffer.text.line(line_index)?)?;
        column_count = column_count.max(cells.len());
        rows.push(MarkdownTableLine {
            prefix,
            cells,
            is_delimiter: false,
            alignments: Vec::new(),
        });
    }

    if rows.len() < 2 || column_count < 2 {
        return None;
    }

    let delimiter_cells = rows[1].cells.clone();
    if !is_markdown_table_delimiter_row_candidate(&delimiter_cells) {
        return None;
    }

    rows[1].is_delimiter = true;
    rows[1].alignments = (0..column_count)
        .map(|index| markdown_table_alignment(delimiter_cells.get(index).map(String::as_str)))
        .collect();

    Some(MarkdownTable {
        start_line,
        rows,
        column_count,
    })
}

fn parse_markdown_table_line(line: &str) -> Option<(String, Vec<String>)> {
    let trimmed_end = line.trim_end();
    let body = trimmed_end.trim_start();
    let prefix = trimmed_end[..trimmed_end.len().saturating_sub(body.len())].to_owned();
    if !body.starts_with('|') || !body.ends_with('|') || body.chars().count() < 3 {
        return None;
    }
    let inner = body.get(1..body.len().saturating_sub(1))?;
    let cells = inner.split('|').map(str::to_owned).collect::<Vec<_>>();
    (!cells.is_empty()).then_some((prefix, cells))
}

fn markdown_table_alignment(cell: Option<&str>) -> MarkdownTableAlignment {
    let trimmed = cell.unwrap_or_default().trim();
    if trimmed.starts_with(':') && trimmed.ends_with(':') && trimmed.chars().count() >= 2 {
        MarkdownTableAlignment::Center
    } else if trimmed.starts_with(':') {
        MarkdownTableAlignment::Left
    } else if trimmed.ends_with(':') {
        MarkdownTableAlignment::Right
    } else {
        MarkdownTableAlignment::None
    }
}

fn is_markdown_table_delimiter_row_candidate(cells: &[String]) -> bool {
    let mut has_bootstrap_cell = false;
    let mut has_any_content = false;
    for cell in cells {
        let trimmed = cell.trim();
        if trimmed.is_empty() {
            continue;
        }
        has_any_content = true;
        let without_prefix = trimmed.strip_prefix(':').unwrap_or(trimmed);
        let stripped = without_prefix.strip_suffix(':').unwrap_or(without_prefix);
        if stripped.is_empty() || !stripped.chars().all(|character| character == '-') {
            return false;
        }
        if stripped.chars().count() >= 2 {
            has_bootstrap_cell = true;
        }
    }
    has_any_content && has_bootstrap_cell
}

fn render_markdown_table(table: &MarkdownTable) -> MarkdownTableRender {
    let mut widths = vec![1usize; table.column_count];
    for row in table.rows.iter().filter(|row| !row.is_delimiter) {
        for (index, width) in widths.iter_mut().enumerate() {
            let content_width = row
                .cells
                .get(index)
                .map(|cell| cell.trim().chars().count())
                .unwrap_or(0)
                .max(1);
            *width = (*width).max(content_width);
        }
    }

    let lines = table
        .rows
        .iter()
        .map(|row| {
            let mut line = row.prefix.clone();
            for (column_index, base_width) in widths.iter().copied().enumerate() {
                line.push('|');
                line.push(' ');
                let width = if row.is_delimiter {
                    base_width.max(3)
                } else {
                    base_width
                };
                if row.is_delimiter {
                    line.push_str(&render_markdown_table_delimiter(
                        width,
                        row.alignments
                            .get(column_index)
                            .copied()
                            .unwrap_or(MarkdownTableAlignment::None),
                    ));
                } else {
                    let content = row
                        .cells
                        .get(column_index)
                        .map(|cell| cell.trim())
                        .unwrap_or_default();
                    line.push_str(content);
                    line.push_str(&" ".repeat(width.saturating_sub(content.chars().count())));
                }
                line.push(' ');
            }
            line.push('|');
            line
        })
        .collect();

    MarkdownTableRender { lines, widths }
}

fn render_markdown_table_delimiter(width: usize, alignment: MarkdownTableAlignment) -> String {
    let width = width.max(3);
    match alignment {
        MarkdownTableAlignment::None => "-".repeat(width),
        MarkdownTableAlignment::Left => format!(":{}", "-".repeat(width.saturating_sub(1))),
        MarkdownTableAlignment::Right => format!("{}:", "-".repeat(width.saturating_sub(1))),
        MarkdownTableAlignment::Center => {
            format!(":{}:", "-".repeat(width.saturating_sub(2)))
        }
    }
}

fn markdown_table_cursor_target(
    buffer: &ShellBuffer,
    table: &MarkdownTable,
    point: TextPoint,
) -> Option<MarkdownTableCursorTarget> {
    if point.line < table.start_line || point.line >= table.start_line + table.rows.len() {
        return None;
    }
    let row_index = point.line.saturating_sub(table.start_line);
    let line = buffer.text.line(point.line)?;
    let (prefix, cells) = parse_markdown_table_line(&line)?;
    let prefix_width = prefix.chars().count();
    let mut column = prefix_width;

    for cell_index in 0..table.column_count {
        column = column.saturating_add(1);
        let cell = cells
            .get(cell_index)
            .map(String::as_str)
            .unwrap_or_default();
        let raw_len = cell.chars().count();
        let cell_start = column;
        let cell_end = cell_start + raw_len;
        let (editable_start, editable_len) =
            markdown_table_editable_cell_bounds(cell, cell_start, cell_end);

        if point.column <= cell_start {
            return Some(MarkdownTableCursorTarget {
                row_index,
                cell_index,
                content_offset: 0,
            });
        }
        if point.column <= cell_end {
            return Some(MarkdownTableCursorTarget {
                row_index,
                cell_index,
                content_offset: point
                    .column
                    .saturating_sub(editable_start)
                    .min(editable_len),
            });
        }

        column = cell_end;
        if point.column <= column.saturating_add(1) {
            return Some(MarkdownTableCursorTarget {
                row_index,
                cell_index: (cell_index + 1).min(table.column_count.saturating_sub(1)),
                content_offset: 0,
            });
        }
    }

    Some(MarkdownTableCursorTarget {
        row_index,
        cell_index: table.column_count.saturating_sub(1),
        content_offset: 0,
    })
}

fn markdown_table_editable_cell_bounds(
    cell: &str,
    cell_start: usize,
    cell_end: usize,
) -> (usize, usize) {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        let start = if cell_start < cell_end {
            cell_start.saturating_add(1).min(cell_end)
        } else {
            cell_start
        };
        return (start, 0);
    }
    let leading = cell
        .chars()
        .take_while(|character| character.is_whitespace())
        .count();
    (cell_start + leading, trimmed.chars().count())
}

fn markdown_table_point_for_target(
    table: &MarkdownTable,
    render: &MarkdownTableRender,
    target: MarkdownTableCursorTarget,
) -> TextPoint {
    let row_index = target.row_index.min(table.rows.len().saturating_sub(1));
    let cell_index = target.cell_index.min(table.column_count.saturating_sub(1));
    let row = &table.rows[row_index];
    let mut column = row.prefix.chars().count();
    for current_cell in 0..table.column_count {
        column = column.saturating_add(1);
        let editable_start = column.saturating_add(1);
        let editable_len = if row.is_delimiter {
            render.widths[current_cell].max(3)
        } else {
            row.cells
                .get(current_cell)
                .map(|cell| cell.trim().chars().count())
                .unwrap_or(0)
        };
        if current_cell == cell_index {
            return TextPoint::new(
                table.start_line + row_index,
                editable_start + target.content_offset.min(editable_len),
            );
        }
        let display_width = if row.is_delimiter {
            render.widths[current_cell].max(3)
        } else {
            render.widths[current_cell]
        };
        column = editable_start + display_width + 1;
    }
    TextPoint::new(table.start_line + row_index, row.prefix.chars().count())
}

fn apply_markdown_table_update(
    buffer: &mut ShellBuffer,
    original: &MarkdownTable,
    updated: &MarkdownTable,
    target: MarkdownTableCursorTarget,
) -> bool {
    let Some(range) = buffer.line_span_range(original.start_line, original.rows.len()) else {
        return false;
    };
    let original_text = buffer.slice(range);
    let render = render_markdown_table(updated);
    let mut replacement = render.lines.join("\n");
    if original_text.ends_with('\n') {
        replacement.push('\n');
    }
    let changed = original_text != replacement;
    if changed {
        buffer.replace_range(range, &replacement);
    }
    buffer.set_cursor(markdown_table_point_for_target(updated, &render, target));
    changed
}

fn format_markdown_table_at_cursor(buffer: &mut ShellBuffer) -> Option<bool> {
    let table = detect_markdown_table(buffer)?;
    let target = markdown_table_cursor_target(buffer, &table, buffer.cursor_point())?;
    Some(apply_markdown_table_update(buffer, &table, &table, target))
}

fn should_defer_table_format(buffer: &ShellBuffer, text: &str) -> bool {
    if text.is_empty() || !text.chars().all(|character| character == ' ') {
        return false;
    }
    let Some(table) = detect_markdown_table(buffer) else {
        return false;
    };
    markdown_table_cursor_target(buffer, &table, buffer.cursor_point())
        .and_then(|target| table.rows.get(target.row_index))
        .is_some_and(|row| !row.is_delimiter)
}

fn insert_markdown_table_row_at_cursor(buffer: &mut ShellBuffer) -> Option<bool> {
    let table = detect_markdown_table(buffer)?;
    let current = markdown_table_cursor_target(buffer, &table, buffer.cursor_point())?;
    let insert_after = if current.row_index <= 1 {
        1
    } else {
        current.row_index
    };
    let mut updated = table.clone();
    updated.rows.insert(
        insert_after + 1,
        MarkdownTableLine {
            prefix: updated
                .rows
                .get(insert_after)
                .map(|row| row.prefix.clone())
                .unwrap_or_default(),
            cells: vec![String::new(); updated.column_count],
            is_delimiter: false,
            alignments: Vec::new(),
        },
    );
    Some(apply_markdown_table_update(
        buffer,
        &table,
        &updated,
        MarkdownTableCursorTarget {
            row_index: insert_after + 1,
            cell_index: 0,
            content_offset: 0,
        },
    ))
}

fn advance_markdown_table_insert_tab(buffer: &mut ShellBuffer) -> Option<bool> {
    let table = detect_markdown_table(buffer)?;
    let current = markdown_table_cursor_target(buffer, &table, buffer.cursor_point())?;
    let mut updated = table.clone();
    let target = if current.cell_index + 1 < updated.column_count {
        MarkdownTableCursorTarget {
            row_index: current.row_index,
            cell_index: current.cell_index + 1,
            content_offset: 0,
        }
    } else {
        updated.column_count = updated.column_count.saturating_add(1);
        for row in &mut updated.rows {
            row.cells.resize(updated.column_count, String::new());
            if row.is_delimiter {
                row.alignments
                    .resize(updated.column_count, MarkdownTableAlignment::None);
            }
        }
        MarkdownTableCursorTarget {
            row_index: current.row_index,
            cell_index: updated.column_count.saturating_sub(1),
            content_offset: 0,
        }
    };
    Some(apply_markdown_table_update(
        buffer, &table, &updated, target,
    ))
}

fn advance_markdown_table_normal_tab(buffer: &mut ShellBuffer) -> Option<bool> {
    let table = detect_markdown_table(buffer)?;
    let current = markdown_table_cursor_target(buffer, &table, buffer.cursor_point())?;
    let mut targets = Vec::new();
    for row_index in 0..table.rows.len() {
        if table.rows[row_index].is_delimiter {
            continue;
        }
        for cell_index in 0..table.column_count {
            targets.push((row_index, cell_index));
        }
    }
    let target = if let Some(position) = targets.iter().position(|&(row_index, cell_index)| {
        row_index == current.row_index && cell_index == current.cell_index
    }) {
        let (row_index, cell_index) = targets[(position + 1) % targets.len()];
        MarkdownTableCursorTarget {
            row_index,
            cell_index,
            content_offset: 0,
        }
    } else {
        let (row_index, cell_index) = targets
            .into_iter()
            .find(|&(row_index, cell_index)| {
                (row_index, cell_index) > (current.row_index, current.cell_index)
            })
            .unwrap_or((0, 0));
        MarkdownTableCursorTarget {
            row_index,
            cell_index,
            content_offset: 0,
        }
    };
    Some(apply_markdown_table_update(buffer, &table, &table, target))
}
