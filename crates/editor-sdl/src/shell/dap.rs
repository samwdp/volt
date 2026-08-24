use std::path::Path;

use editor_dap::{BreakpointState, DapVariableRow, StoredBreakpoint};

use super::*;

pub(super) const DAP_WATCHES_HEADER: &str = "Watch Expressions";
pub(super) const DAP_VAR_EXPANDED_GLYPH: &str = "▾";
pub(super) const DAP_VAR_COLLAPSED_GLYPH: &str = "▸";

pub(super) const TOKEN_DEBUG_BREAKPOINT_PATH: &str = "debug.breakpoint.path";
pub(super) const TOKEN_DEBUG_BREAKPOINT_LINE: &str = "debug.breakpoint.line";
pub(super) const TOKEN_DEBUG_BREAKPOINT_EXTRA: &str = "debug.breakpoint.extra";
pub(super) const TOKEN_DEBUG_BREAKPOINT_EMPTY: &str = "debug.breakpoint.empty";
pub(super) const TOKEN_DEBUG_VARIABLE_HEADER: &str = "debug.variable.header";
pub(super) const TOKEN_DEBUG_VARIABLE_CHEVRON: &str = "debug.variable.chevron";
pub(super) const TOKEN_DEBUG_VARIABLE_NAME: &str = "debug.variable.name";
pub(super) const TOKEN_DEBUG_VARIABLE_VALUE: &str = "debug.variable.value";
pub(super) const TOKEN_DEBUG_VARIABLE_TYPE: &str = "debug.variable.type";
pub(super) const TOKEN_DEBUG_VARIABLE_ERROR: &str = "debug.variable.error";
pub(super) const TOKEN_DEBUG_VARIABLE_WATCH: &str = "debug.variable.watch";
pub(super) const TOKEN_DEBUG_VARIABLE_EMPTY: &str = "debug.variable.empty";

const EMPTY_BREAKPOINTS: &str = "No Breakpoints in this Workspace.";
const EMPTY_LOCALS: &str = "No locals";
const EMPTY_WATCHES: &str = "No watches";
const NOT_STOPPED_SUFFIX: &str = "(not stopped)";

pub(super) fn format_dap_breakpoint_line(bp: &StoredBreakpoint, root: Option<&Path>) -> String {
    let glyph = breakpoint_glyph(bp.state());
    let path = dap_breakpoint_display_path(bp.path(), root);
    let extras = breakpoint_extras(bp);
    if extras.is_empty() {
        format!("{glyph} {path}:{}", bp.line())
    } else {
        format!("{glyph} {path}:{}  {extras}", bp.line())
    }
}

pub(super) fn dap_breakpoint_display_path(path: &Path, root: Option<&Path>) -> String {
    if path.is_absolute() {
        if let Some(root) = root
            && let Ok(relative) = path.strip_prefix(root)
        {
            return relative.display().to_string();
        }
        return path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
    }
    path.display().to_string()
}

pub(super) fn dap_breakpoint_lines(
    breakpoints: &[StoredBreakpoint],
    root: Option<&Path>,
) -> Vec<String> {
    if breakpoints.is_empty() {
        return vec![EMPTY_BREAKPOINTS.to_owned()];
    }
    breakpoints
        .iter()
        .map(|bp| format_dap_breakpoint_line(bp, root))
        .collect()
}

pub(super) fn dap_breakpoint_syntax_lines(
    breakpoints: &[StoredBreakpoint],
    root: Option<&Path>,
) -> IndexedSyntaxLines {
    if breakpoints.is_empty() {
        let mut syntax_lines = IndexedSyntaxLines::new();
        let mut spans = Vec::new();
        push_span_bytes(
            &mut spans,
            EMPTY_BREAKPOINTS,
            0,
            EMPTY_BREAKPOINTS.len(),
            TOKEN_DEBUG_BREAKPOINT_EMPTY,
        );
        syntax_lines.insert(0, spans);
        return syntax_lines;
    }
    breakpoints
        .iter()
        .enumerate()
        .filter_map(|(index, bp)| {
            let line = format_dap_breakpoint_line(bp, root);
            let spans = dap_breakpoint_line_spans(&line, bp.state());
            (!spans.is_empty()).then_some((index, spans))
        })
        .collect()
}

pub(super) fn format_dap_variable_row(row: &DapVariableRow) -> String {
    let indent = "  ".repeat(row.depth());
    let marker = if row.expandable() {
        if row.expanded() {
            format!("{DAP_VAR_EXPANDED_GLYPH} ")
        } else {
            format!("{DAP_VAR_COLLAPSED_GLYPH} ")
        }
    } else {
        "  ".to_owned()
    };
    let name = row.name();
    if !row.ok() {
        return format!("{indent}{marker}{name}: ! {}", row.value());
    }
    match row.type_name() {
        Some(type_name) => format!("{indent}{marker}{name}: {}  {type_name}", row.value()),
        None => format!("{indent}{marker}{name}: {}", row.value()),
    }
}

pub(super) fn dap_locals_section_lines(
    local_rows: &[DapVariableRow],
    watch_rows: &[DapVariableRow],
    idle_watches: &[String],
    stopped: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    if local_rows.is_empty() {
        lines.push(EMPTY_LOCALS.to_owned());
    } else {
        lines.extend(local_rows.iter().map(format_dap_variable_row));
    }
    lines.push(DAP_WATCHES_HEADER.to_owned());
    if stopped {
        if watch_rows.is_empty() {
            return lines;
        }
        lines.extend(watch_rows.iter().map(format_dap_variable_row));
    } else if idle_watches.is_empty() {
        return lines;
    } else {
        lines.extend(
            idle_watches
                .iter()
                .map(|expression| format!("{expression}: {NOT_STOPPED_SUFFIX}")),
        );
    }
    lines
}

pub(super) fn dap_expression_section_lines(
    watch_rows: &[DapVariableRow],
    idle_watches: &[String],
    stopped: bool,
) -> Vec<String> {
    if stopped {
        if watch_rows.is_empty() {
            vec![EMPTY_WATCHES.to_owned()]
        } else {
            watch_rows.iter().map(format_dap_variable_row).collect()
        }
    } else if idle_watches.is_empty() {
        vec![EMPTY_WATCHES.to_owned()]
    } else {
        idle_watches
            .iter()
            .map(|expression| format!("{expression}: {NOT_STOPPED_SUFFIX}"))
            .collect()
    }
}

pub(super) fn dap_variable_syntax_lines(lines: &[String], force_watch: bool) -> IndexedSyntaxLines {
    let header_index = lines.iter().position(|line| line == DAP_WATCHES_HEADER);
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let watches = force_watch || header_index.is_some_and(|header| index >= header);
            let spans = dap_variable_line_spans(line, watches);
            (!spans.is_empty()).then_some((index, spans))
        })
        .collect()
}

pub(super) fn extract_watch_expressions(lines: &[String]) -> Vec<String> {
    let Some(header_index) = lines.iter().position(|line| line == DAP_WATCHES_HEADER) else {
        return Vec::new();
    };
    lines
        .iter()
        .skip(header_index + 1)
        .filter_map(|line| parse_dap_watch_line(line))
        .collect()
}

pub(super) fn locals_line_variable_kind(
    lines: &[String],
    line_index: usize,
    local_row_count: usize,
    watch_row_count: usize,
) -> Option<DapLocalsLineTarget> {
    let Some(header_index) = lines.iter().position(|line| line == DAP_WATCHES_HEADER) else {
        if local_row_count == 0 {
            return None;
        }
        return (line_index < local_row_count).then_some(DapLocalsLineTarget::Local(line_index));
    };
    if line_index < header_index {
        if local_row_count == 0 {
            return None;
        }
        return (line_index < local_row_count).then_some(DapLocalsLineTarget::Local(line_index));
    }
    if line_index == header_index {
        return None;
    }
    let watch_index = line_index.saturating_sub(header_index + 1);
    (watch_index < watch_row_count).then_some(DapLocalsLineTarget::Watch(watch_index))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DapLocalsLineTarget {
    Local(usize),
    Watch(usize),
}

fn parse_dap_watch_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed == DAP_WATCHES_HEADER || trimmed == EMPTY_WATCHES {
        return None;
    }
    let stripped = trimmed
        .trim_start_matches(DAP_VAR_EXPANDED_GLYPH)
        .trim_start_matches(DAP_VAR_COLLAPSED_GLYPH)
        .trim();
    let expression = stripped
        .split_once(':')
        .map(|(name, _)| name.trim())
        .unwrap_or(stripped);
    if expression.is_empty() || expression == NOT_STOPPED_SUFFIX {
        None
    } else {
        Some(expression.to_owned())
    }
}

fn breakpoint_glyph(state: BreakpointState) -> &'static str {
    match state {
        BreakpointState::Verified => DEBUG_FRINGE_VERIFIED_GLYPH,
        BreakpointState::Pending => DEBUG_FRINGE_PENDING_GLYPH,
        BreakpointState::Unverified => DEBUG_FRINGE_UNVERIFIED_GLYPH,
    }
}

fn breakpoint_glyph_token(state: BreakpointState) -> &'static str {
    match state {
        BreakpointState::Verified => TOKEN_DEBUG_FRINGE_BREAKPOINT,
        BreakpointState::Pending => TOKEN_DEBUG_FRINGE_PENDING,
        BreakpointState::Unverified => TOKEN_DEBUG_FRINGE_BREAKPOINT,
    }
}

fn breakpoint_extras(bp: &StoredBreakpoint) -> String {
    let mut extras = Vec::new();
    if let Some(condition) = bp.condition() {
        extras.push(format!("when {condition}"));
    }
    if let Some(hit) = bp.hit_condition() {
        extras.push(format!("hit {hit}"));
    }
    if let Some(log) = bp.log_message() {
        extras.push(format!("log {log}"));
    }
    extras.join(" · ")
}

fn dap_breakpoint_line_spans(line: &str, state: BreakpointState) -> Vec<LineSyntaxSpan> {
    let mut spans = Vec::new();
    let glyph = breakpoint_glyph(state);
    let glyph_end = glyph.len();
    push_span_bytes(
        &mut spans,
        line,
        0,
        glyph_end,
        breakpoint_glyph_token(state),
    );
    let path_start = glyph_end.saturating_add(1);
    if path_start >= line.len() {
        return spans;
    }
    let extras_at = line[path_start..]
        .find("  ")
        .map(|offset| path_start + offset);
    let path_line_end = extras_at.unwrap_or(line.len());
    if let Some(colon) = line[path_start..path_line_end].rfind(':') {
        let colon_at = path_start + colon;
        push_span_bytes(
            &mut spans,
            line,
            path_start,
            colon_at,
            TOKEN_DEBUG_BREAKPOINT_PATH,
        );
        push_span_bytes(
            &mut spans,
            line,
            colon_at,
            path_line_end,
            TOKEN_DEBUG_BREAKPOINT_LINE,
        );
    } else {
        push_span_bytes(
            &mut spans,
            line,
            path_start,
            path_line_end,
            TOKEN_DEBUG_BREAKPOINT_PATH,
        );
    }
    if let Some(extras_at) = extras_at {
        push_span_bytes(
            &mut spans,
            line,
            extras_at,
            line.len(),
            TOKEN_DEBUG_BREAKPOINT_EXTRA,
        );
    }
    spans
}

fn dap_variable_line_spans(line: &str, watch_section: bool) -> Vec<LineSyntaxSpan> {
    let mut spans = Vec::new();
    if line == DAP_WATCHES_HEADER {
        push_span_bytes(&mut spans, line, 0, line.len(), TOKEN_DEBUG_VARIABLE_HEADER);
        return spans;
    }
    if line == EMPTY_LOCALS || line == EMPTY_WATCHES {
        push_span_bytes(&mut spans, line, 0, line.len(), TOKEN_DEBUG_VARIABLE_EMPTY);
        return spans;
    }
    if line.is_empty() {
        return spans;
    }

    let indent = leading_indent_bytes(line);
    let rest = &line[indent..];
    let mut cursor = indent;
    if rest.starts_with(DAP_VAR_EXPANDED_GLYPH) || rest.starts_with(DAP_VAR_COLLAPSED_GLYPH) {
        let glyph_len = if rest.starts_with(DAP_VAR_EXPANDED_GLYPH) {
            DAP_VAR_EXPANDED_GLYPH.len()
        } else {
            DAP_VAR_COLLAPSED_GLYPH.len()
        };
        push_span_bytes(
            &mut spans,
            line,
            cursor,
            cursor + glyph_len,
            TOKEN_DEBUG_VARIABLE_CHEVRON,
        );
        cursor += glyph_len;
        if line[cursor..].starts_with(' ') {
            cursor += 1;
        }
    } else if rest.starts_with("  ") {
        cursor += 2;
    }

    let name_token = if watch_section {
        TOKEN_DEBUG_VARIABLE_WATCH
    } else {
        TOKEN_DEBUG_VARIABLE_NAME
    };
    let remainder = &line[cursor..];
    if let Some(colon) = remainder.find(':') {
        push_span_bytes(&mut spans, line, cursor, cursor + colon, name_token);
        let after_colon = cursor + colon + 1;
        let after = line.get(after_colon..).unwrap_or_default();
        let value_start = after_colon
            + after
                .find(|character: char| !character.is_whitespace())
                .unwrap_or(after.len());
        if after.trim_start().starts_with('!') {
            push_span_bytes(
                &mut spans,
                line,
                value_start,
                line.len(),
                TOKEN_DEBUG_VARIABLE_ERROR,
            );
            return spans;
        }
        if after.contains(NOT_STOPPED_SUFFIX) {
            push_span_bytes(
                &mut spans,
                line,
                value_start,
                line.len(),
                TOKEN_DEBUG_VARIABLE_EMPTY,
            );
            return spans;
        }
        if let Some(type_at) = line[value_start..].rfind("  ") {
            let type_start = value_start + type_at;
            push_span_bytes(
                &mut spans,
                line,
                value_start,
                type_start,
                TOKEN_DEBUG_VARIABLE_VALUE,
            );
            push_span_bytes(
                &mut spans,
                line,
                type_start,
                line.len(),
                TOKEN_DEBUG_VARIABLE_TYPE,
            );
        } else {
            push_span_bytes(
                &mut spans,
                line,
                value_start,
                line.len(),
                TOKEN_DEBUG_VARIABLE_VALUE,
            );
        }
    } else {
        push_span_bytes(&mut spans, line, cursor, line.len(), name_token);
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locals_section_always_keeps_watch_header() {
        let lines = dap_locals_section_lines(&[], &[], &[], true);
        assert_eq!(lines, ["No locals", DAP_WATCHES_HEADER]);
    }

    #[test]
    fn extract_watch_expressions_reads_lines_after_header() {
        let lines = vec![
            "  x: 42  i32".to_owned(),
            DAP_WATCHES_HEADER.to_owned(),
            "  person: Person { ... }  Person".to_owned(),
            "len".to_owned(),
        ];
        assert_eq!(extract_watch_expressions(&lines), ["person", "len"]);
    }

    #[test]
    fn locals_line_kind_skips_header_and_placeholder() {
        let lines = vec![
            "  x: 42  i32".to_owned(),
            DAP_WATCHES_HEADER.to_owned(),
            "  person: Person { ... }  Person".to_owned(),
        ];
        assert_eq!(
            locals_line_variable_kind(&lines, 0, 1, 1),
            Some(DapLocalsLineTarget::Local(0))
        );
        assert_eq!(locals_line_variable_kind(&lines, 1, 1, 1), None);
        assert_eq!(
            locals_line_variable_kind(&lines, 2, 1, 1),
            Some(DapLocalsLineTarget::Watch(0))
        );
        assert_eq!(
            locals_line_variable_kind(
                &["No locals".to_owned(), DAP_WATCHES_HEADER.to_owned()],
                0,
                0,
                0
            ),
            None
        );
    }

    #[test]
    fn watch_header_and_draft_use_watch_tokens() {
        let lines = vec![
            "No locals".to_owned(),
            DAP_WATCHES_HEADER.to_owned(),
            "len".to_owned(),
        ];
        let syntax = dap_variable_syntax_lines(&lines, false);
        assert_eq!(
            syntax.get(&0).map(|spans| spans[0].theme_token.as_str()),
            Some(TOKEN_DEBUG_VARIABLE_EMPTY)
        );
        assert_eq!(
            syntax.get(&1).map(|spans| spans[0].theme_token.as_str()),
            Some(TOKEN_DEBUG_VARIABLE_HEADER)
        );
        assert_eq!(
            syntax.get(&2).map(|spans| spans[0].theme_token.as_str()),
            Some(TOKEN_DEBUG_VARIABLE_WATCH)
        );
    }
}
