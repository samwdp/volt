use super::*;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DiagnosticUnderlineSpan {
    pub(super) start_col: usize,
    pub(super) end_col: usize,
    pub(super) severity: LspDiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DiagnosticLineSpan {
    pub(super) start_col: Option<usize>,
    pub(super) end_col: Option<usize>,
    pub(super) severity: LspDiagnosticSeverity,
}

pub(super) const fn diagnostic_severity_rank(severity: LspDiagnosticSeverity) -> u8 {
    match severity {
        LspDiagnosticSeverity::Error => 0,
        LspDiagnosticSeverity::Warning => 1,
        LspDiagnosticSeverity::Information => 2,
    }
}

pub(super) const fn diagnostic_color(severity: LspDiagnosticSeverity) -> Color {
    match severity {
        LspDiagnosticSeverity::Error => Color::RGB(224, 107, 117),
        LspDiagnosticSeverity::Warning => Color::RGB(209, 154, 102),
        LspDiagnosticSeverity::Information => Color::RGB(110, 170, 255),
    }
}

pub(super) fn statusline_lsp_diagnostics(diagnostics: &[LspDiagnostic]) -> Option<PluginLspDiagnosticsInfo> {
    let mut errors = 0usize;
    let mut warnings = 0usize;
    for diagnostic in diagnostics {
        match diagnostic.severity() {
            LspDiagnosticSeverity::Error => errors += 1,
            LspDiagnosticSeverity::Warning => warnings += 1,
            LspDiagnosticSeverity::Information => {}
        }
    }
    (errors > 0 || warnings > 0).then_some(PluginLspDiagnosticsInfo { errors, warnings })
}

pub(super) fn diagnostic_line_spans_for_diagnostics(
    diagnostics: &[LspDiagnostic],
) -> BTreeMap<usize, Box<[DiagnosticLineSpan]>> {
    let mut spans: BTreeMap<usize, Vec<DiagnosticLineSpan>> = BTreeMap::new();
    for diagnostic in diagnostics {
        let range = diagnostic.range().normalized();
        for line_index in range.start().line..=range.end().line {
            spans
                .entry(line_index)
                .or_default()
                .push(DiagnosticLineSpan {
                    start_col: (line_index == range.start().line).then_some(range.start().column),
                    end_col: (line_index == range.end().line).then_some(range.end().column),
                    severity: diagnostic.severity(),
                });
        }
    }
    spans
        .into_iter()
        .map(|(line_index, line_spans)| (line_index, line_spans.into_boxed_slice()))
        .collect()
}

pub(super) fn covering_syntax_span_for_range(
    syntax_spans: &[LineSyntaxSpan],
    start: usize,
    end: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    syntax_spans
        .iter()
        .filter_map(|span| {
            let span_start = span.start.min(line_len);
            let span_end = span.end.min(line_len);
            (span_start < span_end && span_start <= start && span_end >= end)
                .then_some((span_start, span_end))
        })
        .min_by_key(|(span_start, span_end)| span_end.saturating_sub(*span_start))
}

pub(super) fn diagnostic_columns_for_line(
    diagnostic: DiagnosticLineSpan,
    line_len: usize,
    syntax_spans: Option<&[LineSyntaxSpan]>,
) -> Option<(usize, usize)> {
    let start = diagnostic.start_col.unwrap_or(0).min(line_len);
    let end = diagnostic.end_col.unwrap_or(line_len).min(line_len);
    let columns = if start < end {
        (start, end)
    } else if line_len == 0 {
        return None;
    } else if start >= line_len {
        (line_len.saturating_sub(1), line_len)
    } else {
        (start, (start + 1).min(line_len))
    };
    Some(
        syntax_spans
            .and_then(|spans| covering_syntax_span_for_range(spans, columns.0, columns.1, line_len))
            .unwrap_or(columns),
    )
}

#[cfg(test)]
pub(super) fn diagnostic_underlines_for_segment(
    diagnostics: &[DiagnosticLineSpan],
    syntax_spans: Option<&[LineSyntaxSpan]>,
    line_len: usize,
    segment: LineWrapSegment,
) -> Vec<DiagnosticUnderlineSpan> {
    let mut spans = Vec::with_capacity(diagnostics.len());
    for severity in [
        LspDiagnosticSeverity::Information,
        LspDiagnosticSeverity::Warning,
        LspDiagnosticSeverity::Error,
    ] {
        for diagnostic in diagnostics {
            if diagnostic.severity != severity {
                continue;
            }
            let Some((start, end)) =
                diagnostic_columns_for_line(*diagnostic, line_len, syntax_spans)
            else {
                continue;
            };
            let clipped_start = start.max(segment.start_col);
            let clipped_end = end.min(segment.end_col);
            if clipped_start < clipped_end {
                spans.push(DiagnosticUnderlineSpan {
                    start_col: clipped_start,
                    end_col: clipped_end,
                    severity,
                });
            }
        }
    }
    spans
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_diagnostic_underlines_for_segment(
    target: &mut DrawTarget<'_>,
    diagnostics: &[DiagnosticLineSpan],
    syntax_spans: Option<&[LineSyntaxSpan]>,
    segment_x: i32,
    y: i32,
    line_len: usize,
    segment: LineWrapSegment,
    cell_width: i32,
    line_height: i32,
) -> Result<(), ShellError> {
    for severity in [
        LspDiagnosticSeverity::Information,
        LspDiagnosticSeverity::Warning,
        LspDiagnosticSeverity::Error,
    ] {
        for diagnostic in diagnostics {
            if diagnostic.severity != severity {
                continue;
            }
            let Some((start, end)) =
                diagnostic_columns_for_line(*diagnostic, line_len, syntax_spans)
            else {
                continue;
            };
            let clipped_start = start.max(segment.start_col);
            let clipped_end = end.min(segment.end_col);
            if clipped_start >= clipped_end {
                continue;
            }
            draw_diagnostic_undercurl(
                target,
                segment_x + (clipped_start.saturating_sub(segment.start_col) as i32 * cell_width),
                y,
                (clipped_end.saturating_sub(clipped_start) as i32 * cell_width).max(1),
                line_height,
                diagnostic_color(severity),
            )?;
        }
    }
    Ok(())
}
