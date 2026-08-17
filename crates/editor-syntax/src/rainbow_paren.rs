use std::collections::BTreeMap;

use crate::{HighlightSpan, SyntaxSnapshot};

/// Maximum rainbow depth with a dedicated theme token (cycles after this).
pub const MAX_DEPTH: usize = 9;

const BRACKET_CAPTURE: &str = "punctuation.bracket";

/// Theme token for unmatched closing delimiters.
pub const TOKEN_UNMATCHED: &str = "rainbow.paren.unmatched";
/// Theme token for mismatched closing delimiters.
pub const TOKEN_MISMATCHED: &str = "rainbow.paren.mismatched";

/// Returns the theme token for a nesting depth (1-based, cycles at [`MAX_DEPTH`]).
pub fn depth_theme_token(depth: usize) -> String {
    let face = depth_face_index(depth);
    format!("rainbow.paren.depth.{face}")
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
    let mut bracket_spans = snapshot
        .highlight_spans
        .iter()
        .enumerate()
        .filter_map(|(index, span)| {
            if span.capture_name != BRACKET_CAPTURE {
                return None;
            }
            let text = delimiter_text(buffer_bytes, span)?;
            let family = delimiter_family(&text)?;
            Some(BracketSpan {
                index,
                start_byte: span.start_byte,
                is_open: family.open == text,
                family,
            })
        })
        .collect::<Vec<_>>();

    if bracket_spans.is_empty() {
        return;
    }

    bracket_spans.sort_by_key(|span| (span.start_byte, !span.is_open));

    let mut groups = BTreeMap::<usize, Vec<BracketSpan>>::new();
    for bracket in bracket_spans {
        groups.entry(bracket.start_byte).or_default().push(bracket);
    }

    let mut stack = Vec::<DelimiterFamily>::new();
    let mut replacements = Vec::new();

    for group in groups.into_values() {
        let Some(representative) = group.first().copied() else {
            continue;
        };
        let indices = group.iter().map(|span| span.index).collect::<Vec<_>>();

        if representative.is_open {
            stack.push(representative.family);
            let depth = stack.len();
            let theme_token = depth_theme_token(depth);
            let capture_name = format!("rainbow.paren.open.{depth}");
            for index in indices {
                replacements.push((index, theme_token.clone(), capture_name.clone()));
            }
            continue;
        }

        let depth = stack.len();
        let (theme_token, capture_name) = if depth == 0 {
            (
                TOKEN_UNMATCHED.to_owned(),
                "rainbow.paren.unmatched".to_owned(),
            )
        } else if stack.last().copied() != Some(representative.family) {
            let _ = stack.pop();
            (
                TOKEN_MISMATCHED.to_owned(),
                "rainbow.paren.mismatched".to_owned(),
            )
        } else {
            let _ = stack.pop();
            (
                depth_theme_token(depth),
                format!("rainbow.paren.close.{depth}"),
            )
        };
        for index in indices {
            replacements.push((index, theme_token.clone(), capture_name.clone()));
        }
    }

    for (index, theme_token, capture_name) in replacements {
        let Some(span) = snapshot.highlight_spans.get_mut(index) else {
            continue;
        };
        span.theme_token = theme_token;
        span.capture_name = capture_name;
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
    index: usize,
    open: &'static str,
    close: &'static str,
}

const DELIMITER_FAMILIES: [DelimiterFamily; 3] = [
    DelimiterFamily {
        index: 0,
        open: "(",
        close: ")",
    },
    DelimiterFamily {
        index: 1,
        open: "[",
        close: "]",
    },
    DelimiterFamily {
        index: 2,
        open: "{",
        close: "}",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BracketSpan {
    index: usize,
    start_byte: usize,
    is_open: bool,
    family: DelimiterFamily,
}

fn delimiter_family(text: &str) -> Option<DelimiterFamily> {
    DELIMITER_FAMILIES
        .iter()
        .copied()
        .find(|family| family.open == text || family.close == text)
}

fn delimiter_text(buffer_bytes: &[u8], span: &HighlightSpan) -> Option<String> {
    let bytes = buffer_bytes.get(span.start_byte..span.end_byte)?;
    std::str::from_utf8(bytes).ok().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{
        TOKEN_MISMATCHED, TOKEN_UNMATCHED, apply_rainbow_delimiter_spans, depth_theme_token,
    };
    use crate::{CaptureThemeMapping, LanguageConfiguration, SyntaxRegistry, SyntaxSnapshot};
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

    fn bracket_tokens(snapshot: &SyntaxSnapshot) -> Vec<String> {
        snapshot
            .highlight_spans
            .iter()
            .filter(|span| span.capture_name.starts_with("rainbow.paren."))
            .map(|span| span.theme_token.clone())
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
        assert!(tokens.iter().any(|token| token == "rainbow.paren.depth.1"));
        assert!(tokens.iter().any(|token| token == "rainbow.paren.depth.2"));
        assert!(tokens.iter().any(|token| token == "rainbow.paren.depth.3"));
    }

    #[test]
    fn unmatched_closing_paren_uses_unmatched_token() {
        let snapshot = highlight_nested_parens("fn main() )");
        assert!(
            bracket_tokens(&snapshot)
                .iter()
                .any(|token| token == TOKEN_UNMATCHED)
        );
    }

    #[test]
    fn mismatched_closing_delimiter_uses_mismatched_token() {
        let snapshot = highlight_nested_parens("fn main() { ]");
        assert!(
            bracket_tokens(&snapshot)
                .iter()
                .any(|token| token == TOKEN_MISMATCHED)
        );
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
            .map(|span| span.theme_token.clone())
            .collect::<Vec<_>>();
        assert!(
            tokens.len() >= 2,
            "expected duplicate bracket captures in test setup, got {tokens:?}"
        );
        assert!(
            tokens.iter().all(|token| token == "rainbow.paren.depth.1"),
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
                .filter(|span| span.capture_name == "punctuation.bracket")
                .all(|span| span.theme_token == "syntax.punctuation.bracket")
        );
    }
}
