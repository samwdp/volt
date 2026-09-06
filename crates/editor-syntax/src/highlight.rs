#![allow(unused_imports)]
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fmt, fs,
    mem::ManuallyDrop,
    ops::ControlFlow,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use editor_buffer::{SyntaxText, TextBuffer, TextByteChunks, TextEdit, TextPoint};
use editor_path::PathMatcher;
use tree_sitter::Language;
use tree_sitter::{
    InputEdit, Node, Parser, Point, Query, QueryCursor, QueryCursorOptions, QueryPredicateArg,
    QueryProperty, Range, StreamingIterator, TextProvider, Tree,
};
use tree_sitter_language::LanguageFn;

#[allow(unused_imports)]
use crate::install::*;
#[allow(unused_imports)]
use crate::language::*;
#[allow(unused_imports)]
use crate::query::*;
#[allow(unused_imports)]
use crate::registry::*;

impl SyntaxSnapshot {
    /// Returns the number of highlight spans.
    pub fn highlight_count(&self) -> usize {
        self.highlight_spans.len()
    }

    /// Returns highlight spans intersecting a visible line window.
    pub fn visible_spans(&self, start_line: usize, line_count: usize) -> Vec<&HighlightSpan> {
        if line_count == 0 {
            return Vec::new();
        }

        let end_line = start_line + line_count.saturating_sub(1);
        self.highlight_spans
            .iter()
            .filter(|span| {
                span.start_position.line <= end_line && span.end_position.line >= start_line
            })
            .collect()
    }
}

pub(crate) fn create_parser(
    language_id: &str,
    loaded: &LoadedLanguage,
) -> Result<Parser, SyntaxError> {
    let mut parser = Parser::new();
    parser
        .set_language(&loaded.language)
        .map_err(|error| SyntaxError::ParserConfiguration {
            language_id: language_id.to_owned(),
            message: error.to_string(),
        })?;
    Ok(parser)
}

pub(crate) fn text_edit_to_input_edit(edit: TextEdit) -> InputEdit {
    InputEdit {
        start_byte: edit.start_byte,
        old_end_byte: edit.old_end_byte,
        new_end_byte: edit.new_end_byte,
        start_position: Point {
            row: edit.start_position.line,
            column: edit.start_position.column,
        },
        old_end_position: Point {
            row: edit.old_end_position.line,
            column: edit.old_end_position.column,
        },
        new_end_position: Point {
            row: edit.new_end_position.line,
            column: edit.new_end_position.column,
        },
    }
}

pub(crate) fn parse_with_parser(
    language_id: &str,
    parser: &mut Parser,
    buffer: &impl SyntaxText,
    old_tree: Option<&Tree>,
) -> Result<Tree, SyntaxError> {
    let byte_count = buffer.byte_count();
    parser
        .parse_with_options(
            &mut |byte_offset, _| {
                if byte_offset >= byte_count {
                    return &[][..];
                }
                let Some((chunk, chunk_start_byte)) = buffer.chunk_at_byte(byte_offset) else {
                    return &[][..];
                };
                &chunk.as_bytes()[byte_offset.saturating_sub(chunk_start_byte)..]
            },
            old_tree,
            None,
        )
        .ok_or_else(|| SyntaxError::ParseCancelled(language_id.to_owned()))
}

pub(crate) fn parse_session_ref(
    parse_session: Option<&Option<SyntaxParseSession>>,
) -> Option<&SyntaxParseSession> {
    parse_session.and_then(Option::as_ref)
}

pub(crate) fn require_tree<'a>(
    language_id: &str,
    parse_result: &'a ParseTreeResult,
    session: Option<&'a SyntaxParseSession>,
) -> Result<&'a Tree, SyntaxError> {
    parse_result
        .tree(session)
        .ok_or_else(|| SyntaxError::ParseCancelled(language_id.to_owned()))
}

pub(crate) fn parse_tree(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &impl SyntaxText,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<ParseTreeResult, SyntaxError> {
    let Some(parse_session) = parse_session else {
        let mut parser = create_parser(language_id, loaded)?;
        return Ok(ParseTreeResult {
            owned_tree: Some(parse_with_parser(language_id, &mut parser, buffer, None)?),
            changed_ranges: None,
            applied_edits: None,
        });
    };

    if let Some(session) = parse_session.as_mut()
        && session.language_id == language_id
    {
        if session.revision == buffer.revision() {
            return Ok(ParseTreeResult {
                owned_tree: None,
                changed_ranges: Some(Vec::new()),
                applied_edits: Some(Vec::new()),
            });
        }

        let applied_edits = if session.revision < buffer.revision() {
            buffer.edits_since(session.revision)
        } else {
            None
        };
        if let Some(edits) = applied_edits.as_ref() {
            for edit in edits {
                session.tree.edit(&text_edit_to_input_edit(*edit));
            }
            let new_tree = parse_with_parser(
                language_id,
                &mut session.parser,
                buffer,
                Some(&session.tree),
            )?;
            let changed_ranges = session.tree.changed_ranges(&new_tree).collect::<Vec<_>>();
            session.revision = buffer.revision();
            session.tree = new_tree;
            return Ok(ParseTreeResult {
                owned_tree: None,
                changed_ranges: Some(changed_ranges),
                applied_edits,
            });
        }

        let new_tree = parse_with_parser(language_id, &mut session.parser, buffer, None)?;
        session.revision = buffer.revision();
        session.tree = new_tree;
        session.last_highlight_window = None;
        session.last_snapshot = None;
        return Ok(ParseTreeResult {
            owned_tree: None,
            changed_ranges: None,
            applied_edits: None,
        });
    }

    let mut parser = create_parser(language_id, loaded)?;
    let tree = parse_with_parser(language_id, &mut parser, buffer, None)?;
    *parse_session = Some(SyntaxParseSession {
        language_id: language_id.to_owned(),
        revision: buffer.revision(),
        parser,
        tree,
        last_highlight_window: None,
        last_snapshot: None,
    });
    Ok(ParseTreeResult {
        owned_tree: None,
        changed_ranges: None,
        applied_edits: None,
    })
}

pub(crate) fn highlight_tree(
    loaded: &LoadedLanguage,
    tree: &Tree,
    buffer: &impl SyntaxText,
    highlight_window: Option<HighlightWindow>,
) -> Vec<HighlightSpan> {
    let mut query_cursor = QueryCursor::new();
    if let Some(highlight_window) = highlight_window.filter(|window| !window.is_empty()) {
        query_cursor.set_point_range(
            Point {
                row: highlight_window.start_line(),
                column: 0,
            }..Point {
                row: highlight_window.end_line_exclusive(),
                column: 0,
            },
        );
    }
    let mut highlight_spans = Vec::new();
    let mut matches = query_cursor.matches(
        &loaded.query,
        tree.root_node(),
        SyntaxTextProvider { buffer },
    );
    loop {
        matches.advance();
        let Some(query_match) = matches.get() else {
            break;
        };

        // Apply custom predicates that tree-sitter does not evaluate automatically.
        if !general_predicates_match(
            &loaded.query,
            query_match.pattern_index,
            query_match.captures,
            buffer,
        ) {
            continue;
        }

        for capture in query_match.captures {
            let node = capture.node;
            let start = node.start_position();
            let end = node.end_position();
            let Some((capture_name, theme_token)) = loaded.interned_capture(capture.index as usize)
            else {
                continue;
            };
            if capture_name.is_empty() || !capture_requires_theme_token(capture_name) {
                continue;
            }

            highlight_spans.push(HighlightSpan {
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                start_position: SyntaxPoint::new(start.row, start.column),
                end_position: SyntaxPoint::new(end.row, end.column),
                theme_token: Arc::clone(theme_token),
                capture_name: Arc::clone(capture_name),
            });
        }
    }
    highlight_spans
}

pub(crate) fn collect_injection_regions(
    injections_query: &Query,
    tree: &Tree,
    buffer: &impl SyntaxText,
    highlight_window: Option<HighlightWindow>,
) -> Vec<InjectionRegion> {
    let mut query_cursor = QueryCursor::new();
    if let Some(highlight_window) = highlight_window.filter(|window| !window.is_empty()) {
        query_cursor.set_point_range(
            Point {
                row: highlight_window.start_line(),
                column: 0,
            }..Point {
                row: highlight_window.end_line_exclusive(),
                column: 0,
            },
        );
    }
    let capture_names = injections_query.capture_names();
    let mut regions = Vec::new();
    let mut matches = query_cursor.matches(
        injections_query,
        tree.root_node(),
        SyntaxTextProvider { buffer },
    );
    loop {
        matches.advance();
        let Some(query_match) = matches.get() else {
            break;
        };
        if !general_predicates_match(
            injections_query,
            query_match.pattern_index,
            query_match.captures,
            buffer,
        ) {
            continue;
        }

        let language_capture = query_match.captures.iter().find_map(|capture| {
            let capture_name = capture_names.get(capture.index as usize)?;
            (*capture_name == "injection.language").then(|| {
                buffer_text_for_byte_range(
                    buffer,
                    capture.node.start_byte(),
                    capture.node.end_byte(),
                )
            })?
        });
        for capture in query_match.captures {
            let Some(capture_name) = capture_names.get(capture.index as usize) else {
                continue;
            };
            if *capture_name != "injection.content" {
                continue;
            }

            let language_name = query_capture_property_value(
                injections_query,
                query_match.pattern_index,
                capture.index,
                "injection.language",
            )
            .map(str::to_owned)
            .or_else(|| language_capture.clone())
            .map(|language| language.trim().to_owned())
            .filter(|language| !language.is_empty());
            let Some(language_name) = language_name else {
                continue;
            };

            let node = capture.node;
            let start = node.start_position();
            let end = node.end_position();
            regions.push(InjectionRegion {
                language_name,
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                start_position: SyntaxPoint::new(start.row, start.column),
                end_position: SyntaxPoint::new(end.row, end.column),
            });
        }
    }
    regions
}

pub(crate) fn sort_highlight_spans(highlight_spans: &mut [HighlightSpan]) {
    highlight_spans.sort_by(|left, right| {
        (
            left.start_byte,
            left.end_byte,
            left.start_position.line,
            left.start_position.column,
            &left.capture_name,
            &left.theme_token,
        )
            .cmp(&(
                right.start_byte,
                right.end_byte,
                right.start_position.line,
                right.start_position.column,
                &right.capture_name,
                &right.theme_token,
            ))
    });
}

pub(crate) fn span_intersects_window(span: &HighlightSpan, window: HighlightWindow) -> bool {
    span.start_position.line < window.end_line_exclusive()
        && span.end_position.line >= window.start_line()
}

pub(crate) fn injection_region_intersects_window(
    region: &InjectionRegion,
    window: HighlightWindow,
) -> bool {
    region.start_position.line < window.end_line_exclusive()
        && region.end_position.line >= window.start_line()
}

pub(crate) fn apply_text_edits_to_span(
    mut span: HighlightSpan,
    edits: &[TextEdit],
) -> HighlightSpan {
    for edit in edits {
        let input_edit = text_edit_to_input_edit(*edit);
        let mut range = Range {
            start_byte: span.start_byte,
            end_byte: span.end_byte,
            start_point: Point {
                row: span.start_position.line,
                column: span.start_position.column,
            },
            end_point: Point {
                row: span.end_position.line,
                column: span.end_position.column,
            },
        };
        input_edit.edit_range(&mut range);

        span.start_byte = range.start_byte;
        span.end_byte = range.end_byte;
        span.start_position = SyntaxPoint::new(range.start_point.row, range.start_point.column);
        span.end_position = SyntaxPoint::new(range.end_point.row, range.end_point.column);
    }

    span
}

pub(crate) fn changed_range_windows(
    changed_ranges: &[Range],
    highlight_window: HighlightWindow,
) -> Vec<HighlightWindow> {
    const CONTEXT_LINES: usize = 1;

    if highlight_window.is_empty() {
        return Vec::new();
    }

    let mut ranges = changed_ranges
        .iter()
        .filter_map(|range| {
            let start_line = range
                .start_point
                .row
                .saturating_sub(CONTEXT_LINES)
                .max(highlight_window.start_line());
            let end_line_exclusive = range
                .end_point
                .row
                .saturating_add(1)
                .saturating_add(CONTEXT_LINES)
                .min(highlight_window.end_line_exclusive());
            (start_line < end_line_exclusive)
                .then(|| HighlightWindow::new(start_line, end_line_exclusive - start_line))
        })
        .collect::<Vec<_>>();
    ranges.sort_by_key(HighlightWindow::start_line);

    let mut merged: Vec<HighlightWindow> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && last.end_line_exclusive() >= range.start_line()
        {
            let end_line_exclusive = last.end_line_exclusive().max(range.end_line_exclusive());
            *last = HighlightWindow::new(last.start_line(), end_line_exclusive - last.start_line());
            continue;
        }
        merged.push(range);
    }
    merged
}

pub(crate) fn highlight_windows_missing_from(
    old_window: HighlightWindow,
    new_window: HighlightWindow,
) -> Vec<HighlightWindow> {
    if new_window.is_empty() {
        return Vec::new();
    }
    if old_window.is_empty()
        || new_window.end_line_exclusive() <= old_window.start_line()
        || new_window.start_line() >= old_window.end_line_exclusive()
    {
        return vec![new_window];
    }

    let mut missing = Vec::new();
    if new_window.start_line() < old_window.start_line() {
        missing.push(HighlightWindow::new(
            new_window.start_line(),
            old_window.start_line() - new_window.start_line(),
        ));
    }
    if new_window.end_line_exclusive() > old_window.end_line_exclusive() {
        missing.push(HighlightWindow::new(
            old_window.end_line_exclusive(),
            new_window.end_line_exclusive() - old_window.end_line_exclusive(),
        ));
    }
    missing
}

pub(crate) fn reuse_highlight_spans_for_window(
    previous_highlight_spans: Vec<HighlightSpan>,
    last_window: Option<HighlightWindow>,
    highlight_window: Option<HighlightWindow>,
    loaded: &LoadedLanguage,
    tree: &Tree,
    buffer: &impl SyntaxText,
) -> Option<Vec<HighlightSpan>> {
    match (last_window, highlight_window) {
        (None, None) => Some(previous_highlight_spans),
        (Some(old_window), Some(new_window)) => {
            let missing = highlight_windows_missing_from(old_window, new_window);
            let mut spans = previous_highlight_spans
                .into_iter()
                .filter(|span| span_intersects_window(span, new_window))
                .filter(|span| {
                    !missing
                        .iter()
                        .any(|window| span_intersects_window(span, *window))
                })
                .collect::<Vec<_>>();
            for window in missing {
                spans.extend(
                    highlight_tree(loaded, tree, buffer, Some(window))
                        .into_iter()
                        .filter(|span| span_intersects_window(span, window))
                        .filter(|span| span_intersects_window(span, new_window)),
                );
            }
            Some(spans)
        }
        _ => None,
    }
}

pub(crate) fn highlight_spans_for_tree(
    session: Option<&SyntaxParseSession>,
    loaded: &LoadedLanguage,
    tree: &Tree,
    buffer: &impl SyntaxText,
    highlight_window: Option<HighlightWindow>,
    parse_result: &ParseTreeResult,
) -> Vec<HighlightSpan> {
    session
        .and_then(|session| {
            let previous_snapshot = session.last_snapshot.as_ref()?;
            let changed_ranges = parse_result.changed_ranges.as_ref()?;
            let applied_edits = parse_result.applied_edits.as_deref().unwrap_or(&[]);
            let previous_highlight_spans = previous_snapshot
                .highlight_spans
                .iter()
                .cloned()
                .map(|span| apply_text_edits_to_span(span, applied_edits))
                .collect::<Vec<_>>();
            if changed_ranges.is_empty() {
                return reuse_highlight_spans_for_window(
                    previous_highlight_spans,
                    session.last_highlight_window,
                    highlight_window,
                    loaded,
                    tree,
                    buffer,
                );
            }
            if session.last_highlight_window != highlight_window {
                return None;
            }
            let highlight_window = highlight_window?;
            let changed_windows = changed_range_windows(changed_ranges, highlight_window);
            if changed_windows.is_empty() {
                return Some(previous_highlight_spans);
            }

            let mut highlight_spans = previous_highlight_spans
                .iter()
                .filter(|span| {
                    !changed_windows
                        .iter()
                        .any(|window| span_intersects_window(span, *window))
                })
                .cloned()
                .collect::<Vec<_>>();
            for changed_window in changed_windows {
                highlight_spans.extend(highlight_tree(loaded, tree, buffer, Some(changed_window)));
            }
            Some(highlight_spans)
        })
        .unwrap_or_else(|| highlight_tree(loaded, tree, buffer, highlight_window))
}

pub(crate) fn highlight_loaded_language_with_tree(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &impl SyntaxText,
    highlight_window: Option<HighlightWindow>,
    mut parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<ParsedHighlight, SyntaxError> {
    let parse_result = parse_tree(language_id, loaded, buffer, parse_session.as_deref_mut())?;
    let mut highlight_spans = {
        let session = parse_session_ref(parse_session.as_deref());
        let tree = require_tree(language_id, &parse_result, session)?;
        highlight_spans_for_tree(
            session,
            loaded,
            tree,
            buffer,
            highlight_window,
            &parse_result,
        )
    };
    sort_highlight_spans(&mut highlight_spans);
    let snapshot = {
        let session = parse_session_ref(parse_session.as_deref());
        let tree = require_tree(language_id, &parse_result, session)?;
        SyntaxSnapshot {
            language_id: language_id.to_owned(),
            root_kind: tree.root_node().kind().to_owned(),
            has_errors: tree.root_node().has_error(),
            highlight_spans,
        }
    };
    if let Some(session) = parse_session.and_then(|inner| inner.as_mut()) {
        session.last_highlight_window = highlight_window;
        session.last_snapshot = Some(snapshot.clone());
    }
    Ok(ParsedHighlight {
        snapshot,
        tree: parse_result.owned_tree,
    })
}

pub(crate) fn highlight_loaded_language(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &impl SyntaxText,
    highlight_window: Option<HighlightWindow>,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<SyntaxSnapshot, SyntaxError> {
    Ok(highlight_loaded_language_with_tree(
        language_id,
        loaded,
        buffer,
        highlight_window,
        parse_session,
    )?
    .snapshot)
}

pub(crate) fn markdown_inline_line_indices(tree: &Tree) -> Vec<usize> {
    pub(crate) fn collect_lines(node: tree_sitter::Node<'_>, lines: &mut BTreeSet<usize>) {
        if node.kind() == "inline" {
            for line in node.start_position().row..=node.end_position().row {
                lines.insert(line);
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_lines(child, lines);
        }
    }

    let mut lines = BTreeSet::new();
    collect_lines(tree.root_node(), &mut lines);
    lines.into_iter().collect()
}

pub(crate) fn collect_structure_nodes(root: tree_sitter::Node<'_>) -> Vec<SyntaxStructureNode> {
    pub(crate) fn walk(node: tree_sitter::Node<'_>, out: &mut Vec<SyntaxStructureNode>) {
        if node.is_named() {
            let start = node.start_position();
            let end = node.end_position();
            out.push(SyntaxStructureNode {
                kind: node.kind().to_owned(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                start_position: SyntaxPoint::new(start.row, start.column),
                end_position: SyntaxPoint::new(end.row, end.column),
            });
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk(child, out);
        }
    }

    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

pub(crate) fn buffer_text_for_byte_range(
    buffer: &impl SyntaxText,
    start_byte: usize,
    end_byte: usize,
) -> Option<String> {
    if start_byte >= end_byte || end_byte > buffer.byte_count() {
        return None;
    }

    let mut text = String::new();
    for chunk in buffer.byte_slice_chunks(start_byte..end_byte) {
        let Ok(chunk) = std::str::from_utf8(chunk) else {
            return None;
        };
        text.push_str(chunk);
    }
    Some(text)
}

pub(crate) fn translate_injected_highlight_span(
    span: HighlightSpan,
    region: &InjectionRegion,
) -> HighlightSpan {
    let start_line = region.start_position.line + span.start_position.line;
    let end_line = region.start_position.line + span.end_position.line;
    let start_column = if span.start_position.line == 0 {
        region.start_position.column + span.start_position.column
    } else {
        span.start_position.column
    };
    let end_column = if span.end_position.line == 0 {
        region.start_position.column + span.end_position.column
    } else {
        span.end_position.column
    };

    HighlightSpan {
        start_byte: region.start_byte.saturating_add(span.start_byte),
        end_byte: region.start_byte.saturating_add(span.end_byte),
        start_position: SyntaxPoint::new(start_line, start_column),
        end_position: SyntaxPoint::new(end_line, end_column),
        capture_name: span.capture_name,
        theme_token: span.theme_token,
    }
}

pub(crate) fn highlight_inline_language_per_line(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &impl SyntaxText,
    line_indices: &[usize],
) -> Result<SyntaxSnapshot, SyntaxError> {
    let mut highlight_spans = Vec::new();
    let mut has_errors = false;

    for &line_index in line_indices {
        let Some(line_text) = buffer.line(line_index) else {
            continue;
        };
        if line_text.is_empty() {
            continue;
        }
        let Some(start_byte) = buffer.line_start_byte(line_index) else {
            continue;
        };
        let line_buffer = TextBuffer::from_text(&line_text);
        let mut parser = create_parser(language_id, loaded)?;
        let tree = parse_with_parser(language_id, &mut parser, &line_buffer, None)?;
        has_errors |= tree.root_node().has_error();

        let spans = highlight_tree(loaded, &tree, &line_buffer, None);
        let line_len = line_text.chars().count();
        for span in spans {
            let start_col = span.start_position.column.min(line_len);
            let end_col = span.end_position.column.min(line_len);
            if start_col >= end_col {
                continue;
            }
            highlight_spans.push(HighlightSpan {
                start_byte: start_byte.saturating_add(span.start_byte),
                end_byte: start_byte.saturating_add(span.end_byte),
                start_position: SyntaxPoint::new(line_index, start_col),
                end_position: SyntaxPoint::new(line_index, end_col),
                capture_name: span.capture_name,
                theme_token: span.theme_token,
            });
        }
    }

    Ok(SyntaxSnapshot {
        language_id: language_id.to_owned(),
        root_kind: "inline".to_owned(),
        has_errors,
        highlight_spans,
    })
}
