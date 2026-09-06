use super::{TOKEN_MISMATCHED, TOKEN_UNMATCHED, apply_rainbow_delimiter_spans, depth_theme_token};
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
        registry.highlight_buffer_for_extension("__rainbow_test__", &TextBuffer::from_text(source)),
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
    let mut from_text = must(registry.highlight_buffer_for_extension("__rainbow_test__", &text));
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
        registry.highlight_buffer_for_extension("__rainbow_test__", &TextBuffer::from_text(source)),
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
