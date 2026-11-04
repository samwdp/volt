//! Rule-based highlighting for Snippet Definition (`.vsnip`) files.
//!
//! No tree-sitter grammar: keywords, field labels, and insert-point markers are
//! matched with line scans so authoring works without a grammar install.

use std::sync::Arc;

use editor_buffer::SyntaxText;

use crate::language::{HighlightSpan, SyntaxPoint, SyntaxSnapshot};

const LANGUAGE_ID: &str = "vsnip";

const HEADER_KEYS: &[&str] = &[
    "name", "trigger", "target", "each", "maybe", "body", "path", "line",
];
const TARGET_RELATIONS: &[&str] = &["next", "enclosing"];

/// Builds a [`SyntaxSnapshot`] for a `.vsnip` buffer using keyword rules.
pub fn highlight_vsnip(buffer: &impl SyntaxText) -> SyntaxSnapshot {
    let text = buffer_text(buffer);
    let mut spans = Vec::new();
    let mut byte_offset = 0usize;

    for (line_index, line) in text.split_inclusive('\n').enumerate() {
        let content = line.strip_suffix('\n').unwrap_or(line);
        let content = content.strip_suffix('\r').unwrap_or(content);
        highlight_line(content, byte_offset, line_index, &mut spans);
        byte_offset += line.len();
    }

    SyntaxSnapshot {
        language_id: LANGUAGE_ID.to_owned(),
        root_kind: "vsnip_file".to_owned(),
        has_errors: false,
        highlight_spans: spans,
    }
}

/// Returns true when `path` should use rule-based vsnip highlighting.
pub fn path_is_vsnip(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("vsnip"))
}

fn buffer_text(buffer: &impl SyntaxText) -> String {
    let mut text = String::with_capacity(buffer.byte_count());
    for chunk in buffer.byte_slice_chunks(0..buffer.byte_count()) {
        if let Ok(chunk) = std::str::from_utf8(chunk) {
            text.push_str(chunk);
        }
    }
    text
}

fn highlight_line(line: &str, line_byte: usize, line_index: usize, spans: &mut Vec<HighlightSpan>) {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let indent_byte = line[..indent].len();

    if let Some(rest) = match_keyword_prefix(trimmed, HEADER_KEYS) {
        let key_len = trimmed.len() - rest.len();
        push_span(
            spans,
            line_byte + indent_byte,
            line_byte + indent_byte + key_len,
            line_index,
            indent,
            indent + key_len,
            "keyword",
            "syntax.keyword",
        );
        highlight_value_and_markers(
            rest,
            line_byte + indent_byte + key_len,
            line_index,
            indent + key_len,
            spans,
        );
        return;
    }

    highlight_value_and_markers(trimmed, line_byte + indent_byte, line_index, indent, spans);
}

fn match_keyword_prefix<'a>(line: &'a str, keys: &[&str]) -> Option<&'a str> {
    for key in keys {
        if let Some(rest) = line.strip_prefix(key)
            && (rest.is_empty() || rest.starts_with(':') || rest.starts_with(char::is_whitespace))
        {
            return Some(rest);
        }
    }
    None
}

fn highlight_value_and_markers(
    text: &str,
    start_byte: usize,
    line_index: usize,
    start_column: usize,
    spans: &mut Vec<HighlightSpan>,
) {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if text[index..].starts_with("${{") {
            // skip invalid
            index += 1;
            continue;
        }
        if text[index..].starts_with("$${") {
            let end = find_closing_brace(&text[index..]).unwrap_or(3);
            push_span(
                spans,
                start_byte + index,
                start_byte + index + end,
                line_index,
                start_column + char_count(&text[..index]),
                start_column + char_count(&text[..index + end]),
                "insert_point",
                "syntax.constant",
            );
            index += end;
            continue;
        }
        if text[index..].starts_with("${") {
            let end = find_closing_brace(&text[index..]).unwrap_or(2);
            push_span(
                spans,
                start_byte + index,
                start_byte + index + end,
                line_index,
                start_column + char_count(&text[..index]),
                start_column + char_count(&text[..index + end]),
                "hole",
                "syntax.variable",
            );
            index += end;
            continue;
        }
        if bytes[index] == b'$' && index + 1 < bytes.len() && bytes[index + 1].is_ascii_digit() {
            let mut end = index + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            push_span(
                spans,
                start_byte + index,
                start_byte + end,
                line_index,
                start_column + char_count(&text[..index]),
                start_column + char_count(&text[..end]),
                "insert_point",
                "syntax.constant",
            );
            index = end;
            continue;
        }

        // Highlight target relations when they appear as whole words.
        if let Some(len) = match_word_at(&text[index..], TARGET_RELATIONS) {
            push_span(
                spans,
                start_byte + index,
                start_byte + index + len,
                line_index,
                start_column + char_count(&text[..index]),
                start_column + char_count(&text[..index + len]),
                "keyword",
                "syntax.keyword",
            );
            index += len;
            continue;
        }

        index += 1;
    }
}

fn match_word_at(text: &str, words: &[&str]) -> Option<usize> {
    for word in words {
        if let Some(rest) = text.strip_prefix(word) {
            let boundary_ok = rest
                .chars()
                .next()
                .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_');
            if boundary_ok {
                return Some(word.len());
            }
        }
    }
    None
}

fn find_closing_brace(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'}' {
            return Some(index + 1);
        }
        index += 1;
    }
    None
}

fn char_count(text: &str) -> usize {
    text.chars().count()
}

#[allow(clippy::too_many_arguments)]
fn push_span(
    spans: &mut Vec<HighlightSpan>,
    start_byte: usize,
    end_byte: usize,
    line: usize,
    start_column: usize,
    end_column: usize,
    capture_name: &str,
    theme_token: &str,
) {
    if start_byte >= end_byte {
        return;
    }
    spans.push(HighlightSpan {
        start_byte,
        end_byte,
        start_position: SyntaxPoint::new(line, start_column),
        end_position: SyntaxPoint::new(line, end_column),
        capture_name: Arc::<str>::from(capture_name),
        theme_token: Arc::<str>::from(theme_token),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_buffer::TextBuffer;

    #[test]
    fn highlights_header_keys_and_insert_points() {
        let buffer = TextBuffer::from_text(
            "name: demo\ntrigger: ///\ntarget: next method_declaration\nbody:\n  /// $0\n  ${params}\n",
        );
        let snapshot = highlight_vsnip(&buffer);
        assert_eq!(snapshot.language_id, "vsnip");
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token.as_ref() == "syntax.keyword")
        );
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.capture_name.as_ref() == "insert_point")
        );
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.capture_name.as_ref() == "hole")
        );
    }
}
