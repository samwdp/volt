use std::collections::BTreeMap;

use super::git::{leading_indent_bytes, push_span_bytes, split_icon_prefixed_content};
use super::*;
use editor_plugin_api::DbBrowserItemKind;

pub(super) const TOKEN_DB_RESULTS_TITLE: &str = "db.results.title";
pub(super) const TOKEN_DB_RESULTS_GRID: &str = "db.results.grid";
pub(super) const TOKEN_DB_RESULTS_HEADER: &str = "db.results.header";
pub(super) const TOKEN_DB_RESULTS_NULL: &str = "db.results.null";
pub(super) const TOKEN_DB_RESULTS_NUMBER: &str = "db.results.number";
pub(super) const TOKEN_DB_RESULTS_STRING: &str = "db.results.string";
pub(super) const TOKEN_DB_RESULTS_FOOTER: &str = "db.results.footer";
pub(super) const TOKEN_DB_RESULTS_ERROR: &str = "db.results.error";
pub(super) const TOKEN_DB_RESULTS_MESSAGE: &str = "db.results.message";
pub(super) const TOKEN_DB_BROWSER_HEADER: &str = "db.browser.header";
pub(super) const TOKEN_DB_BROWSER_EMPTY: &str = "db.browser.empty";
pub(super) const TOKEN_DB_BROWSER_CONNECTION: &str = "db.browser.connection";
pub(super) const TOKEN_DB_BROWSER_ACTIVE: &str = "db.browser.active";
pub(super) const TOKEN_DB_BROWSER_TABLE: &str = "db.browser.table";
pub(super) const TOKEN_DB_BROWSER_VIEW: &str = "db.browser.view";
pub(super) const TOKEN_DB_BROWSER_INDEX: &str = "db.browser.index";
pub(super) const TOKEN_DB_BROWSER_HISTORY: &str = "db.browser.history";
pub(super) const TOKEN_DB_BROWSER_SNIPPET: &str = "db.browser.snippet";
pub(super) const TOKEN_DB_BROWSER_COLUMN: &str = "db.schema.column";
pub(super) const TOKEN_DB_BROWSER_MUTED: &str = "db.browser.muted";

pub(super) fn db_results_syntax_lines(
    lines: &[String],
    is_error: bool,
) -> BTreeMap<usize, Vec<LineSyntaxSpan>> {
    let mut syntax_lines = BTreeMap::new();
    let mut seen_header_row = false;
    for (index, line) in lines.iter().enumerate() {
        let spans = if is_error {
            db_results_error_spans(line, index)
        } else {
            let spans = db_results_line_spans(line, seen_header_row);
            if line_is_table_row(line) && !seen_header_row {
                seen_header_row = true;
            }
            spans
        };
        if !spans.is_empty() {
            syntax_lines.insert(index, spans);
        }
    }
    syntax_lines
}

fn db_results_error_spans(line: &str, index: usize) -> Vec<LineSyntaxSpan> {
    let token = if index == 0 {
        TOKEN_DB_RESULTS_ERROR
    } else if line.is_empty() {
        return Vec::new();
    } else {
        TOKEN_DB_RESULTS_MESSAGE
    };
    let mut spans = Vec::new();
    push_span_bytes(&mut spans, line, 0, line.len(), token);
    spans
}

fn db_results_line_spans(line: &str, seen_header_row: bool) -> Vec<LineSyntaxSpan> {
    if line.is_empty() {
        return Vec::new();
    }
    if line_is_table_rule(line) {
        let mut spans = Vec::new();
        push_span_bytes(&mut spans, line, 0, line.len(), TOKEN_DB_RESULTS_GRID);
        return spans;
    }
    if line_is_table_row(line) {
        return db_results_table_row_spans(line, !seen_header_row);
    }
    let token = if line.contains(" · ") || line.ends_with("affected.") {
        TOKEN_DB_RESULTS_FOOTER
    } else {
        TOKEN_DB_RESULTS_TITLE
    };
    let mut spans = Vec::new();
    push_span_bytes(&mut spans, line, 0, line.len(), token);
    spans
}

fn line_is_table_rule(line: &str) -> bool {
    line.starts_with('┌') || line.starts_with('├') || line.starts_with('└')
}

fn line_is_table_row(line: &str) -> bool {
    line.starts_with('│')
}

fn db_results_table_row_spans(line: &str, is_header: bool) -> Vec<LineSyntaxSpan> {
    let mut spans = Vec::new();
    let mut byte_index = 0usize;
    for character in line.chars() {
        let len = character.len_utf8();
        if character == '│' || character == '─' {
            push_span_bytes(
                &mut spans,
                line,
                byte_index,
                byte_index + len,
                TOKEN_DB_RESULTS_GRID,
            );
        }
        byte_index += len;
    }
    let mut search_from = 0usize;
    while let Some(relative) = line[search_from..].find('│') {
        let start = search_from + relative + '│'.len_utf8();
        let Some(end_relative) = line[start..].find('│') else {
            break;
        };
        let end = start + end_relative;
        let cell = line[start..end].trim();
        if !cell.is_empty() {
            let token = if is_header {
                TOKEN_DB_RESULTS_HEADER
            } else {
                cell_theme_token(cell)
            };
            let trim_start = start + leading_whitespace_bytes(&line[start..end]);
            push_span_bytes(&mut spans, line, trim_start, trim_start + cell.len(), token);
        }
        search_from = start;
    }
    spans
}

fn leading_whitespace_bytes(text: &str) -> usize {
    text.len() - text.trim_start().len()
}

fn cell_theme_token(cell: &str) -> &'static str {
    if cell == "NULL" {
        TOKEN_DB_RESULTS_NULL
    } else if is_numeric_result_cell(cell) {
        TOKEN_DB_RESULTS_NUMBER
    } else {
        TOKEN_DB_RESULTS_STRING
    }
}

fn is_numeric_result_cell(value: &str) -> bool {
    let mut seen_digit = false;
    let mut seen_dot = false;
    for (index, character) in value.chars().enumerate() {
        match character {
            '0'..='9' => seen_digit = true,
            '+' | '-' if index == 0 => {}
            '.' if !seen_dot => seen_dot = true,
            _ => return false,
        }
    }
    seen_digit
}

pub(super) fn db_browser_line_spans(line: &str, kind: DbBrowserItemKind) -> Vec<LineSyntaxSpan> {
    if line.is_empty() {
        return Vec::new();
    }
    match kind {
        DbBrowserItemKind::Header if line.starts_with(' ') => db_schema_column_spans(line),
        DbBrowserItemKind::Header => icon_line_spans(line, TOKEN_DB_BROWSER_HEADER, None),
        DbBrowserItemKind::Empty => whole_line_spans(line, TOKEN_DB_BROWSER_EMPTY),
        DbBrowserItemKind::ActiveConnection => connection_line_spans(line, true),
        DbBrowserItemKind::RememberedConnection => connection_line_spans(line, false),
        DbBrowserItemKind::Table => icon_line_spans(line, TOKEN_DB_BROWSER_TABLE, None),
        DbBrowserItemKind::View => icon_line_spans(line, TOKEN_DB_BROWSER_VIEW, None),
        DbBrowserItemKind::Index => icon_line_spans(line, TOKEN_DB_BROWSER_INDEX, None),
        DbBrowserItemKind::HistoryEntry => {
            icon_line_spans(line, TOKEN_DB_BROWSER_HISTORY, Some(TOKEN_DB_BROWSER_MUTED))
        }
        DbBrowserItemKind::Snippet => {
            icon_line_spans(line, TOKEN_DB_BROWSER_SNIPPET, Some(TOKEN_DB_BROWSER_MUTED))
        }
    }
}

fn whole_line_spans(line: &str, token: &str) -> Vec<LineSyntaxSpan> {
    let mut spans = Vec::new();
    push_span_bytes(&mut spans, line, 0, line.len(), token);
    spans
}

fn icon_line_spans(line: &str, token: &str, detail_token: Option<&str>) -> Vec<LineSyntaxSpan> {
    let mut spans = Vec::new();
    let indent_bytes = leading_indent_bytes(line);
    let trimmed = &line[indent_bytes..];
    let (icon_bounds, content_start, content) =
        split_icon_prefixed_content(trimmed).unwrap_or((None, 0, trimmed));
    if let Some((icon_start, icon_end)) = icon_bounds {
        push_span_bytes(
            &mut spans,
            line,
            indent_bytes + icon_start,
            indent_bytes + icon_end,
            token,
        );
    }
    let content_offset = indent_bytes + content_start;
    if let Some(detail_token) = detail_token
        && let Some(separator) = content.find("  ·  ")
    {
        push_span_bytes(
            &mut spans,
            line,
            content_offset,
            content_offset + separator,
            token,
        );
        push_span_bytes(
            &mut spans,
            line,
            content_offset + separator,
            content_offset + content.len(),
            detail_token,
        );
        return spans;
    }
    push_span_bytes(
        &mut spans,
        line,
        content_offset,
        content_offset + content.len(),
        token,
    );
    spans
}

fn connection_line_spans(line: &str, active: bool) -> Vec<LineSyntaxSpan> {
    let mut spans = icon_line_spans(
        line,
        TOKEN_DB_BROWSER_CONNECTION,
        Some(TOKEN_DB_BROWSER_MUTED),
    );
    if active && let Some(index) = line.rfind("active") {
        push_span_bytes(
            &mut spans,
            line,
            index,
            index + "active".len(),
            TOKEN_DB_BROWSER_ACTIVE,
        );
    }
    spans
}

fn db_schema_column_spans(line: &str) -> Vec<LineSyntaxSpan> {
    let mut spans = Vec::new();
    let indent_bytes = leading_indent_bytes(line);
    let trimmed = &line[indent_bytes..];
    let name_end = trimmed
        .find("  ")
        .unwrap_or_else(|| trimmed.trim_end().len());
    push_span_bytes(
        &mut spans,
        line,
        indent_bytes,
        indent_bytes + name_end,
        TOKEN_DB_BROWSER_COLUMN,
    );
    if name_end < trimmed.len() {
        push_span_bytes(
            &mut spans,
            line,
            indent_bytes + name_end,
            indent_bytes + trimmed.len(),
            TOKEN_DB_BROWSER_MUTED,
        );
    }
    spans
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;
