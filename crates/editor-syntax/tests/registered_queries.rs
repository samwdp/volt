//! Regression tests that compile the bundled highlight and indent queries for
//! registered languages against their actual tree-sitter grammars.
//!
//! The Rust tests use the statically-linked `tree-sitter-rust` dev-dependency
//! so they run without any installed grammars.  The markdown-inline test loads
//! the pre-built grammar from `user/lang/grammars/` (committed for development)
//! and uses the bundled `markdown-inline/highlights.scm` query; this is the
//! explicit regression guard requested for that language pair.
#![allow(unused_crate_dependencies)]
use std::path::PathBuf;

use editor_buffer::TextBuffer;
use editor_syntax::{
    CaptureThemeMapping, GrammarSource, HighlightWindow, LanguageConfiguration, SyntaxRegistry,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("unexpected error: {error:?}"),
    }
}

/// Path to the bundled query asset root (the `queries/` directory under
/// `crates/volt/assets/grammars/`).  Resolved at compile time relative to
/// this crate's manifest directory.
fn query_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("volt")
        .join("assets")
        .join("grammars")
        .join("queries")
}

/// Path to the pre-built grammar install root (`user/lang/grammars/`).
fn user_grammars_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("user")
        .join("lang")
        .join("grammars")
}

fn rust_language() -> editor_syntax::Language {
    tree_sitter_rust::LANGUAGE.into()
}

// ---------------------------------------------------------------------------
// Static (tree-sitter-rust) compilation tests
// ---------------------------------------------------------------------------

/// The bundled `rust/highlights.scm` must compile against the tree-sitter-rust
/// grammar without errors.
#[test]
fn rust_bundled_highlights_query_compiles() {
    let query_text =
        std::fs::read_to_string(query_asset_root().join("rust").join("highlights.scm"))
            .expect("failed to read bundled rust highlights.scm");

    let config = LanguageConfiguration::new(
        "rust",
        ["rs"],
        rust_language,
        query_text,
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    );

    let mut registry = SyntaxRegistry::new();
    must(registry.register(config));

    let buffer = TextBuffer::from_text("fn main() {}");
    let snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));
    assert_eq!(snapshot.language_id, "rust");
    assert!(!snapshot.has_errors);
}

/// The bundled `rust/indents.scm` must compile against the tree-sitter-rust
/// grammar without errors and produce a non-None indent for a nested block.
#[test]
fn rust_bundled_indents_query_compiles_and_produces_indent() {
    let highlights_text =
        std::fs::read_to_string(query_asset_root().join("rust").join("highlights.scm"))
            .expect("failed to read bundled rust highlights.scm");
    let indents_text = std::fs::read_to_string(query_asset_root().join("rust").join("indents.scm"))
        .expect("failed to read bundled rust indents.scm");

    let config = LanguageConfiguration::new(
        "rust",
        ["rs"],
        rust_language,
        highlights_text,
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )
    .with_extra_indent_query(indents_text);

    let mut registry = SyntaxRegistry::new();
    must(registry.register(config));

    // Line 2 (0-indexed) is the empty line inside `if true { ... }`.
    let buffer = TextBuffer::from_text("fn main() {\n    if true {\n\n    }\n}\n");
    let indent = must(registry.desired_indent_for_language("rust", &buffer, 2, 4));
    assert_eq!(
        indent,
        Some(8),
        "expected 8 columns of indent for nested block"
    );
}

/// Windowed highlighting against the bundled rust query must return spans
/// within the requested line range.
#[test]
fn rust_bundled_highlights_windowed_returns_bounded_spans() {
    let query_text =
        std::fs::read_to_string(query_asset_root().join("rust").join("highlights.scm"))
            .expect("failed to read bundled rust highlights.scm");

    let config = LanguageConfiguration::new(
        "rust",
        ["rs"],
        rust_language,
        query_text,
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    );

    let mut registry = SyntaxRegistry::new();
    must(registry.register(config));

    let mut source = String::new();
    for i in 0..64u32 {
        source.push_str(&format!("fn f{i}() {{}}\n"));
    }
    let buffer = TextBuffer::from_text(source);
    let window = HighlightWindow::new(10, 8);
    let snapshot = must(registry.highlight_buffer_for_language_window("rust", &buffer, window));

    assert!(!snapshot.highlight_spans.is_empty());
    assert!(snapshot.highlight_spans.iter().all(|span| {
        span.start_position.line < window.start_line() + window.line_count()
            && span.end_position.line >= window.start_line()
    }));
}

// ---------------------------------------------------------------------------
// Grammar-backed (markdown-inline) compilation tests
// ---------------------------------------------------------------------------

/// Builds the `markdown-inline` `LanguageConfiguration` exactly as the user
/// library registers it, using the pre-built grammar DLL from
/// `user/lang/grammars/`.
fn markdown_inline_config() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "markdown-inline",
        [] as [&str; 0],
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
            ".",
            "tree-sitter-markdown-inline/src",
            "tree-sitter-markdown-inline",
            "tree_sitter_markdown_inline",
        ),
        [
            CaptureThemeMapping::new("text.literal", "syntax.text.literal"),
            CaptureThemeMapping::new("text.emphasis", "syntax.text.emphasis"),
            CaptureThemeMapping::new("text.strong", "syntax.text.strong"),
            CaptureThemeMapping::new("text.uri", "syntax.text.uri"),
            CaptureThemeMapping::new("text.reference", "syntax.text.reference"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("string.escape", "syntax.string.escape"),
        ],
    )
}

/// Returns `true` when the markdown-inline grammar DLL is present in the
/// development pre-built grammar directory.
fn markdown_inline_grammar_available() -> bool {
    let install_root = user_grammars_root();
    let dll =
        install_root
            .join("tree-sitter-markdown-inline")
            .join(if cfg!(target_os = "windows") {
                "libtree-sitter-markdown-inline.dll"
            } else if cfg!(target_os = "macos") {
                "libtree-sitter-markdown-inline.dylib"
            } else {
                "libtree-sitter-markdown-inline.so"
            });
    let query = install_root
        .join("tree-sitter-markdown-inline")
        .join("queries")
        .join("highlights.scm");
    dll.exists() && query.exists()
}

fn markdown_grammar_available() -> bool {
    let install_root = user_grammars_root();
    install_root
        .join("tree-sitter-markdown")
        .join(if cfg!(target_os = "windows") {
            "libtree-sitter-markdown.dll"
        } else if cfg!(target_os = "macos") {
            "libtree-sitter-markdown.dylib"
        } else {
            "libtree-sitter-markdown.so"
        })
        .exists()
        && install_root
            .join("tree-sitter-markdown")
            .join("queries")
            .join("highlights.scm")
            .exists()
}

fn markdown_config() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "markdown",
        ["md", "markdown"],
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
            ".",
            "tree-sitter-markdown/src",
            "tree-sitter-markdown",
            "tree_sitter_markdown",
        ),
        [
            CaptureThemeMapping::new("text.title", "syntax.text.title"),
            CaptureThemeMapping::new("text.literal", "syntax.text.literal"),
        ],
    )
    .with_additional_highlight_languages(["markdown-inline"])
}

/// The bundled `markdown-inline/highlights.scm` must compile against the
/// pre-built grammar DLL and not return a query compilation error.
///
/// This is the explicit markdown-inline regression: the bundled query still
/// exercises markdown-inline-specific captures such as `@nospell`,
/// `@markup.link.url`, and the conceal/entity rules that previously triggered
/// the reported query-parse failure.
///
/// Skipped automatically when the pre-built grammar DLL is absent (e.g. on CI
/// runners that have not run `treesitter.install`).
#[test]
fn markdown_inline_bundled_highlights_query_compiles() {
    if !markdown_inline_grammar_available() {
        eprintln!(
            "SKIP: markdown-inline grammar not found at {}",
            user_grammars_root().display()
        );
        return;
    }

    let mut registry = SyntaxRegistry::with_install_root(user_grammars_root());
    must(registry.register(markdown_inline_config()));

    // A line of markdown inline content.
    let buffer =
        TextBuffer::from_text("Hello **world** with `code` and [a link](http://example.com).");
    let result = registry.highlight_buffer_for_language("markdown-inline", &buffer);
    let snapshot = must(result);
    assert_eq!(snapshot.language_id, "markdown-inline");
    // The inline grammar must produce at least one highlight span.
    assert!(
        !snapshot.highlight_spans.is_empty(),
        "markdown-inline highlights produced no spans for inline markdown content"
    );
}

/// Markdown and markdown-inline languages both register successfully and the
/// merged highlight path does not panic.
#[test]
fn markdown_and_inline_merged_highlight_compiles() {
    if !markdown_inline_grammar_available() {
        eprintln!(
            "SKIP: markdown-inline grammar not found at {}",
            user_grammars_root().display()
        );
        return;
    }

    // Both grammars live in the same install root.
    let install_root = user_grammars_root();
    if !markdown_grammar_available() {
        eprintln!(
            "SKIP: markdown grammar not found at {}",
            install_root.display()
        );
        return;
    }

    let mut registry = SyntaxRegistry::with_install_root(install_root);
    must(registry.register(markdown_config()));
    must(registry.register(markdown_inline_config()));

    let buffer = TextBuffer::from_text("# Heading\n\nParagraph with **bold** and `code`.\n");
    let snapshot = must(registry.highlight_buffer_for_language("markdown", &buffer));
    assert_eq!(snapshot.language_id, "markdown");
    assert!(!snapshot.has_errors);
}

#[test]
fn markdown_fenced_code_blocks_use_injected_language_highlighting() {
    if !markdown_inline_grammar_available() {
        eprintln!(
            "SKIP: markdown-inline grammar not found at {}",
            user_grammars_root().display()
        );
        return;
    }

    let install_root = user_grammars_root();
    if !markdown_grammar_available() {
        eprintln!(
            "SKIP: markdown grammar not found at {}",
            install_root.display()
        );
        return;
    }

    let rust_highlights =
        std::fs::read_to_string(query_asset_root().join("rust").join("highlights.scm"))
            .expect("failed to read bundled rust highlights.scm");
    let rust_config = LanguageConfiguration::new(
        "rust",
        ["rs"],
        rust_language,
        rust_highlights,
        [
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("string", "syntax.string"),
        ],
    )
    .with_additional_highlight_languages(["rust-inline"]);
    let rust_inline_config = LanguageConfiguration::new(
        "rust-inline",
        [] as [&str; 0],
        rust_language,
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        [CaptureThemeMapping::new("string", "syntax.string.inline")],
    );

    let mut registry = SyntaxRegistry::with_install_root(install_root);
    must(registry.register_all([
        markdown_config(),
        markdown_inline_config(),
        rust_config,
        rust_inline_config,
    ]));

    let buffer = TextBuffer::from_text(
        "Paragraph with **bold**.\n\n```rs\nfn injected() { let value = \"volt\"; }\n```\n",
    );
    let source = buffer.text();
    let Some(bold_byte) = source.find("bold") else {
        panic!("expected bold text in markdown fixture");
    };
    let Some(injected_fn_byte) = source.find("injected") else {
        panic!("expected injected Rust function name in markdown fixture");
    };
    let Some(injected_string_byte) = source.find("volt") else {
        panic!("expected injected Rust string literal in markdown fixture");
    };

    let snapshot = must(registry.highlight_buffer_for_language("markdown", &buffer));
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|span| span.capture_name == "markup.strong"
                && span.start_byte <= bold_byte
                && bold_byte < span.end_byte),
        "expected markdown-inline strong emphasis span covering byte {bold_byte}, got {:?}",
        snapshot.highlight_spans
    );
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|span| span.theme_token == "syntax.function"
                && span.start_byte <= injected_fn_byte
                && injected_fn_byte < span.end_byte),
        "expected injected Rust function span covering byte {injected_fn_byte}, got {:?}",
        snapshot.highlight_spans
    );
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|span| span.theme_token == "syntax.string.inline"
                && span.start_byte <= injected_string_byte
                && injected_string_byte < span.end_byte),
        "expected injected Rust inline string span covering byte {injected_string_byte}, got {:?}",
        snapshot.highlight_spans
    );
}
