#![allow(unused_imports)]
use crate::{HighlightSpan, SyntaxSnapshot};
use editor_buffer::SyntaxText;
use std::sync::Arc;

/// Maximum rainbow depth with a dedicated theme token (cycles after this).
pub const MAX_DEPTH: usize = 9;

const BRACKET_CAPTURE: &str = "punctuation.bracket";

/// Theme token for unmatched closing delimiters.
pub const TOKEN_UNMATCHED: &str = "rainbow.paren.unmatched";
/// Theme token for mismatched closing delimiters.
pub const TOKEN_MISMATCHED: &str = "rainbow.paren.mismatched";

const DEPTH_THEME_TOKENS: [&str; MAX_DEPTH] = [
    "rainbow.paren.depth.1",
    "rainbow.paren.depth.2",
    "rainbow.paren.depth.3",
    "rainbow.paren.depth.4",
    "rainbow.paren.depth.5",
    "rainbow.paren.depth.6",
    "rainbow.paren.depth.7",
    "rainbow.paren.depth.8",
    "rainbow.paren.depth.9",
];
const DEPTH_OPEN_CAPTURES: [&str; MAX_DEPTH] = [
    "rainbow.paren.open.1",
    "rainbow.paren.open.2",
    "rainbow.paren.open.3",
    "rainbow.paren.open.4",
    "rainbow.paren.open.5",
    "rainbow.paren.open.6",
    "rainbow.paren.open.7",
    "rainbow.paren.open.8",
    "rainbow.paren.open.9",
];
const DEPTH_CLOSE_CAPTURES: [&str; MAX_DEPTH] = [
    "rainbow.paren.close.1",
    "rainbow.paren.close.2",
    "rainbow.paren.close.3",
    "rainbow.paren.close.4",
    "rainbow.paren.close.5",
    "rainbow.paren.close.6",
    "rainbow.paren.close.7",
    "rainbow.paren.close.8",
    "rainbow.paren.close.9",
];

/// Returns the theme token for a nesting depth (1-based, cycles at [`MAX_DEPTH`]).
pub fn depth_theme_token(depth: usize) -> String {
    depth_theme_token_str(depth).to_owned()
}

fn depth_theme_token_str(depth: usize) -> &'static str {
    DEPTH_THEME_TOKENS[depth_face_index(depth) - 1]
}

fn depth_open_capture(depth: usize) -> &'static str {
    DEPTH_OPEN_CAPTURES[depth_face_index(depth) - 1]
}

fn depth_close_capture(depth: usize) -> &'static str {
    DEPTH_CLOSE_CAPTURES[depth_face_index(depth) - 1]
}

/// Applies rainbow delimiter coloring to bracket highlight spans in `snapshot`.
pub fn apply_rainbow_delimiter_spans(
    snapshot: &mut SyntaxSnapshot,
    buffer_text: &str,
    enabled: bool,
) {
    if !enabled {
        return;
    }
    let buffer_bytes = buffer_text.as_bytes();
    apply_rainbow_delimiter_spans_inner(snapshot, |span| {
        delimiter_kind(buffer_bytes.get(span.start_byte..span.end_byte)?)
    });
}

/// Applies rainbow delimiter coloring without flattening the rope into a `String`.
pub fn apply_rainbow_delimiter_spans_for_buffer(
    snapshot: &mut SyntaxSnapshot,
    buffer: &impl SyntaxText,
    enabled: bool,
) {
    if !enabled {
        return;
    }
    let mut current_chunk: Option<(&str, usize)> = None;
    apply_rainbow_delimiter_spans_inner(snapshot, |span| {
        let mut current = current_chunk;
        let span_end = span.end_byte;
        let needs_refresh = match current {
            Some((chunk, chunk_start)) => {
                span.start_byte < chunk_start || span_end > chunk_start.saturating_add(chunk.len())
            }
            None => true,
        };
        if needs_refresh {
            current = buffer.chunk_at_byte(span.start_byte);
            current_chunk = current;
        }
        let (chunk, chunk_start) = current?;
        let offset = span.start_byte.saturating_sub(chunk_start);
        let len = span_end.saturating_sub(span.start_byte);
        delimiter_kind(chunk.as_bytes().get(offset..offset.saturating_add(len))?)
    });
}

fn apply_rainbow_delimiter_spans_inner(
    snapshot: &mut SyntaxSnapshot,
    mut delimiter_at: impl FnMut(&HighlightSpan) -> Option<(bool, DelimiterFamily)>,
) {
    let mut bracket_spans = snapshot
        .highlight_spans
        .iter()
        .enumerate()
        .filter(|(_, span)| span.capture_name.as_ref() == BRACKET_CAPTURE)
        .map(|(index, span)| (index, span.start_byte))
        .collect::<Vec<_>>();

    if bracket_spans.is_empty() {
        return;
    }

    bracket_spans.sort_by_key(|(_, start_byte)| *start_byte);

    let mut resolved = Vec::with_capacity(bracket_spans.len());
    for (index, start_byte) in bracket_spans {
        let Some(span) = snapshot.highlight_spans.get(index) else {
            continue;
        };
        let Some((is_open, family)) = delimiter_at(span) else {
            continue;
        };
        resolved.push(BracketSpan {
            index,
            start_byte,
            is_open,
            family,
        });
    }

    if resolved.is_empty() {
        return;
    }

    resolved.sort_by_key(|span| (span.start_byte, !span.is_open));

    let mut stack = Vec::<DelimiterFamily>::new();
    let mut group_start = 0;
    while group_start < resolved.len() {
        let start_byte = resolved[group_start].start_byte;
        let mut group_end = group_start + 1;
        while group_end < resolved.len() && resolved[group_end].start_byte == start_byte {
            group_end += 1;
        }
        let representative = resolved[group_start];
        let (theme_token, capture_name) = if representative.is_open {
            stack.push(representative.family);
            let depth = stack.len();
            (depth_theme_token_str(depth), depth_open_capture(depth))
        } else {
            let depth = stack.len();
            if depth == 0 {
                (TOKEN_UNMATCHED, TOKEN_UNMATCHED)
            } else if stack.last().copied() != Some(representative.family) {
                let _ = stack.pop();
                (TOKEN_MISMATCHED, TOKEN_MISMATCHED)
            } else {
                let _ = stack.pop();
                (depth_theme_token_str(depth), depth_close_capture(depth))
            }
        };
        for bracket in &resolved[group_start..group_end] {
            let Some(span) = snapshot.highlight_spans.get_mut(bracket.index) else {
                continue;
            };
            span.theme_token = Arc::from(theme_token);
            span.capture_name = Arc::from(capture_name);
        }
        group_start = group_end;
    }
}

fn depth_face_index(depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }
    if depth <= MAX_DEPTH {
        return depth;
    }
    ((depth - 1) % MAX_DEPTH) + 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DelimiterFamily {
    index: u8,
}

const FAMILY_PAREN: DelimiterFamily = DelimiterFamily { index: 0 };
const FAMILY_BRACKET: DelimiterFamily = DelimiterFamily { index: 1 };
const FAMILY_BRACE: DelimiterFamily = DelimiterFamily { index: 2 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BracketSpan {
    index: usize,
    start_byte: usize,
    is_open: bool,
    family: DelimiterFamily,
}

fn delimiter_kind(bytes: &[u8]) -> Option<(bool, DelimiterFamily)> {
    match bytes {
        b"(" => Some((true, FAMILY_PAREN)),
        b")" => Some((false, FAMILY_PAREN)),
        b"[" => Some((true, FAMILY_BRACKET)),
        b"]" => Some((false, FAMILY_BRACKET)),
        b"{" => Some((true, FAMILY_BRACE)),
        b"}" => Some((false, FAMILY_BRACE)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "rainbow_paren_tests.rs"]
mod tests;
