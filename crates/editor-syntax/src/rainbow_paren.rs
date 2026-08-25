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
mod tests {
    use super::{
        TOKEN_MISMATCHED, TOKEN_UNMATCHED, apply_rainbow_delimiter_spans, depth_theme_token,
    };
    use crate::{
        CaptureThemeMapping, HighlightWindow, LanguageConfiguration, SyntaxRegistry, SyntaxSnapshot,
    };
    use editor_buffer::TextBuffer;

    fn rust_language() -> tree_sitter::Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn rainbow_test_configuration() -> LanguageConfiguration {
        LanguageConfiguration::new(
            "rust-rainbow-test",
            ["__rainbow_test__"],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [CaptureThemeMapping::new(
                "punctuation.bracket",
                "syntax.punctuation.bracket",
            )],
        )
        .with_extra_highlight_query(
            r#"
[
  "(" ")" "[" "]" "{" "}"
] @punctuation.bracket
"#,
        )
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    fn highlight_nested_parens(source: &str) -> SyntaxSnapshot {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rainbow_test_configuration()));
        let mut snapshot = must(
            registry
                .highlight_buffer_for_extension("__rainbow_test__", &TextBuffer::from_text(source)),
        );
        apply_rainbow_delimiter_spans(&mut snapshot, source, true);
        snapshot
    }

    #[test]
    fn buffer_apply_matches_contiguous_text_apply() {
        let source = "fn main() { if true { () } }";
        let text = TextBuffer::from_text(source);
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rainbow_test_configuration()));
        let mut from_text =
            must(registry.highlight_buffer_for_extension("__rainbow_test__", &text));
        let mut from_buffer = from_text.clone();
        apply_rainbow_delimiter_spans(&mut from_text, source, true);
        super::apply_rainbow_delimiter_spans_for_buffer(&mut from_buffer, &text, true);
        assert_eq!(bracket_tokens(&from_text), bracket_tokens(&from_buffer));
    }

    fn bracket_tokens(snapshot: &SyntaxSnapshot) -> Vec<&str> {
        snapshot
            .highlight_spans
            .iter()
            .filter(|span| span.capture_name.starts_with("rainbow.paren."))
            .map(|span| span.theme_token.as_ref())
            .collect()
    }

    #[test]
    fn depth_tokens_cycle_at_max_depth() {
        assert_eq!(depth_theme_token(1), "rainbow.paren.depth.1");
        assert_eq!(depth_theme_token(9), "rainbow.paren.depth.9");
        assert_eq!(depth_theme_token(10), "rainbow.paren.depth.1");
    }

    #[test]
    fn nested_parens_receive_increasing_depth_tokens() {
        let snapshot = highlight_nested_parens("fn main() { if true { () } }");
        let tokens = bracket_tokens(&snapshot);
        assert!(tokens.contains(&"rainbow.paren.depth.1"));
        assert!(tokens.contains(&"rainbow.paren.depth.2"));
        assert!(tokens.contains(&"rainbow.paren.depth.3"));
    }

    #[test]
    fn unmatched_closing_paren_uses_unmatched_token() {
        let snapshot = highlight_nested_parens("fn main() )");
        assert!(bracket_tokens(&snapshot).contains(&TOKEN_UNMATCHED));
    }

    #[test]
    fn mismatched_closing_delimiter_uses_mismatched_token() {
        let snapshot = highlight_nested_parens("fn main() { ]");
        assert!(bracket_tokens(&snapshot).contains(&TOKEN_MISMATCHED));
    }

    #[test]
    fn duplicate_bracket_captures_at_same_byte_share_depth() {
        let source = "use crate::{";
        let snapshot = highlight_nested_parens(source);
        let brace_byte = source.find('{').expect("opening brace byte");
        let tokens = snapshot
            .highlight_spans
            .iter()
            .filter(|span| {
                span.start_byte == brace_byte && span.capture_name.starts_with("rainbow.paren.")
            })
            .map(|span| span.theme_token.as_ref())
            .collect::<Vec<_>>();
        assert!(
            tokens.len() >= 2,
            "expected duplicate bracket captures in test setup, got {tokens:?}"
        );
        assert!(
            tokens.iter().all(|token| *token == "rainbow.paren.depth.1"),
            "duplicate captures at the same delimiter should share depth, got {tokens:?}"
        );
    }

    #[test]
    fn disabled_rainbow_leaves_bracket_tokens_unchanged() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rainbow_test_configuration()));
        let source = "fn main() {}";
        let mut snapshot = must(
            registry
                .highlight_buffer_for_extension("__rainbow_test__", &TextBuffer::from_text(source)),
        );
        apply_rainbow_delimiter_spans(&mut snapshot, source, false);
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .filter(|span| span.capture_name.as_ref() == "punctuation.bracket")
                .all(|span| span.theme_token.as_ref() == "syntax.punctuation.bracket")
        );
    }

    #[test]
    fn rainbow_apply_on_large_buffer_stays_within_scroll_budget() {
        use std::hint::black_box;
        use std::time::{Duration, Instant};

        let line = "fn f() { if true { let x = vec![(1, 2, { 3 })]; } }\n";
        let source = line.repeat(30_000);
        let text = TextBuffer::from_text(&source);
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rainbow_test_configuration()));
        let snapshot = must(registry.highlight_buffer_for_extension_window(
            "__rainbow_test__",
            &text,
            HighlightWindow::new(0, 256),
        ));
        let bracket_spans = snapshot
            .highlight_spans
            .iter()
            .filter(|span| span.capture_name.as_ref() == "punctuation.bracket")
            .count();

        const ITERATIONS: u32 = 20;
        let clone_started = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(text.text());
        }
        let clone_elapsed = clone_started.elapsed();

        let mut apply_snapshots = (0..ITERATIONS)
            .map(|_| snapshot.clone())
            .collect::<Vec<_>>();
        let apply_started = Instant::now();
        for apply_snapshot in &mut apply_snapshots {
            apply_rainbow_delimiter_spans(apply_snapshot, source.as_str(), true);
            black_box(apply_snapshot.highlight_spans.len());
        }
        let apply_elapsed = apply_started.elapsed();

        let mut buffer_snapshots = (0..ITERATIONS)
            .map(|_| snapshot.clone())
            .collect::<Vec<_>>();
        let buffer_started = Instant::now();
        for apply_snapshot in &mut buffer_snapshots {
            super::apply_rainbow_delimiter_spans_for_buffer(apply_snapshot, &text, true);
            black_box(apply_snapshot.highlight_spans.len());
        }
        let buffer_elapsed = buffer_started.elapsed();

        let per_clone = clone_elapsed / ITERATIONS;
        let per_apply = apply_elapsed / ITERATIONS;
        let per_buffer_apply = buffer_elapsed / ITERATIONS;
        eprintln!(
            "rainbow large-buffer cost: bytes={} spans={} brackets={} clone={clone_elapsed:?} apply={apply_elapsed:?} buffer_apply={buffer_elapsed:?} per_clone={per_clone:?} per_apply={per_apply:?} per_buffer_apply={per_buffer_apply:?}",
            source.len(),
            snapshot.highlight_spans.len(),
            bracket_spans,
        );

        assert!(
            per_apply < Duration::from_millis(8),
            "rainbow apply per syntax refresh too slow for j/k scroll: {per_apply:?} (clone {per_clone:?}, buffer-apply {per_buffer_apply:?}, spans {}, brackets {bracket_spans})",
            snapshot.highlight_spans.len(),
        );
        assert!(
            per_buffer_apply < Duration::from_millis(8),
            "rainbow buffer apply per syntax refresh too slow for j/k scroll: {per_buffer_apply:?}"
        );
    }
}
