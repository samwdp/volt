//! Regression tests that compile the bundled highlight and indent queries for
//! registered languages against their actual tree-sitter grammars.
//!
//! The Rust tests use the statically-linked `tree-sitter-rust` dev-dependency
//! so they run without any installed grammars.  The markdown-inline test loads
//! the pre-built grammar from `user/lang/grammars/` (committed for development)
//! and uses the bundled `markdown-inline/highlights.scm` query; this is the
//! explicit regression guard requested for that language pair.
#![allow(unused_crate_dependencies)]
use std::{env, path::PathBuf};

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

fn default_grammars_root() -> PathBuf {
    if let Some(path) = env::var_os("VOLT_GRAMMAR_DIR").map(PathBuf::from) {
        return path;
    }

    let base = if cfg!(target_os = "windows") {
        env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from)
    } else {
        env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| {
            env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
    };

    base.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("volt")
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

fn razor_config() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "razor",
        ["cshtml", "razor"],
        GrammarSource::new(
            "https://github.com/tris203/tree-sitter-razor",
            ".",
            "src",
            "tree-sitter-razor",
            "tree_sitter_razor",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )
}

fn csharp_config() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "csharp",
        ["cs"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-c-sharp.git",
            ".",
            "src",
            "tree-sitter-c-sharp",
            "tree_sitter_c_sharp",
        ),
        [
            CaptureThemeMapping::new("attribute", "syntax.attribute"),
            CaptureThemeMapping::new("comment", "syntax.comment"),
            CaptureThemeMapping::new("constant.builtin", "syntax.constant.builtin"),
            CaptureThemeMapping::new("constructor", "syntax.constructor"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("module", "syntax.module"),
            CaptureThemeMapping::new("number", "syntax.number"),
            CaptureThemeMapping::new("operator", "syntax.operator"),
            CaptureThemeMapping::new("property.definition", "syntax.property"),
            CaptureThemeMapping::new("punctuation.bracket", "syntax.punctuation.bracket"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("string.escape", "syntax.string.escape"),
            CaptureThemeMapping::new("type", "syntax.type"),
            CaptureThemeMapping::new("type.builtin", "syntax.type.builtin"),
            CaptureThemeMapping::new("variable", "syntax.variable"),
            CaptureThemeMapping::new("variable.parameter", "syntax.variable.parameter"),
        ],
    )
}

fn csharp_grammar_available() -> bool {
    let install_root = default_grammars_root();
    install_root
        .join(if cfg!(target_os = "windows") {
            "libtree-sitter-c-sharp.dll"
        } else if cfg!(target_os = "macos") {
            "libtree-sitter-c-sharp.dylib"
        } else {
            "libtree-sitter-c-sharp.so"
        })
        .exists()
}

fn typescript_config() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "typescript",
        ["ts"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-typescript.git",
            ".",
            "typescript/src",
            "tree-sitter-typescript",
            "tree_sitter_typescript",
        ),
        [
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("variable", "syntax.variable"),
            CaptureThemeMapping::new("type", "syntax.type"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("number", "syntax.number"),
            CaptureThemeMapping::new("operator", "syntax.operator"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
        ],
    )
}

fn typescript_grammar_available() -> bool {
    let install_root = default_grammars_root();
    install_root
        .join(if cfg!(target_os = "windows") {
            "libtree-sitter-typescript.dll"
        } else if cfg!(target_os = "macos") {
            "libtree-sitter-typescript.dylib"
        } else {
            "libtree-sitter-typescript.so"
        })
        .exists()
}

fn razor_grammar_available() -> bool {
    let install_root = default_grammars_root();
    install_root
        .join("tree-sitter-razor")
        .join(if cfg!(target_os = "windows") {
            "libtree-sitter-razor.dll"
        } else if cfg!(target_os = "macos") {
            "libtree-sitter-razor.dylib"
        } else {
            "libtree-sitter-razor.so"
        })
        .exists()
}

fn zig_config() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "zig",
        ["zig"],
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-zig.git",
            ".",
            "src",
            "tree-sitter-zig",
            "tree_sitter_zig",
        ),
        [
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("variable", "syntax.variable"),
            CaptureThemeMapping::new("type", "syntax.type"),
            CaptureThemeMapping::new("constant", "syntax.constant"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("number", "syntax.number"),
            CaptureThemeMapping::new("operator", "syntax.operator"),
            CaptureThemeMapping::new("punctuation.delimiter", "syntax.punctuation.delimiter"),
        ],
    )
}

fn zig_grammar_available() -> bool {
    let install_root = default_grammars_root();
    install_root
        .join(if cfg!(target_os = "windows") {
            "libtree-sitter-zig.dll"
        } else if cfg!(target_os = "macos") {
            "libtree-sitter-zig.dylib"
        } else {
            "libtree-sitter-zig.so"
        })
        .exists()
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
fn razor_bundled_highlights_query_compiles() {
    let install_root = default_grammars_root();
    if !razor_grammar_available() {
        eprintln!(
            "SKIP: razor grammar not found at {}",
            install_root.display()
        );
        return;
    }

    let mut registry = SyntaxRegistry::with_install_root(&install_root);
    registry.set_query_asset_root(Some(query_asset_root()));
    must(registry.register(razor_config()));

    let buffer = TextBuffer::from_text("<div>@await WorkAsync()</div>\n");
    let snapshot = must(registry.highlight_buffer_for_language("razor", &buffer));
    assert_eq!(snapshot.language_id, "razor");
    assert!(!snapshot.has_errors);
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|span| span.capture_name == "keyword.coroutine"),
        "expected @await to produce a keyword.coroutine span, got {:?}",
        snapshot.highlight_spans
    );
}

#[test]
fn csharp_flat_grammar_uses_bundled_queries() {
    let install_root = default_grammars_root();
    if !csharp_grammar_available() {
        eprintln!(
            "SKIP: csharp grammar not found at {}",
            install_root.display()
        );
        return;
    }

    let mut registry = SyntaxRegistry::with_install_root(&install_root);
    registry.set_query_asset_root(Some(query_asset_root()));
    must(registry.register(csharp_config()));

    let buffer = TextBuffer::from_text(
        "class Demo {\n    void Run() {\n        var xs = numbers is [.. var rest];\n    }\n}\n",
    );
    let snapshot = must(registry.highlight_buffer_for_language("csharp", &buffer));
    assert_eq!(snapshot.language_id, "csharp");
    assert!(!snapshot.has_errors);
    assert!(
        !snapshot.highlight_spans.is_empty(),
        "csharp bundled highlights produced no spans"
    );
}

#[test]
fn typescript_blank_line_before_closing_object_dedents_to_sibling_indent() {
    let install_root = default_grammars_root();
    if !typescript_grammar_available() {
        eprintln!(
            "SKIP: typescript grammar not found at {}",
            install_root.display()
        );
        return;
    }

    let mut registry = SyntaxRegistry::with_install_root(&install_root);
    registry.set_query_asset_root(Some(query_asset_root()));
    must(registry.register(typescript_config()));

    let buffer = TextBuffer::from_text(
        ";\nexport const Endpoints = (builder: EndpointBuilder<any, any, any>) => ({\n  getOutdoorTrackingHistoryByCustomer: builder.query<DashboardTrackingHistory[], TrackingHistoryAttributes, DashboardTrackingHistoryDto[]>({\n    query: (args: TrackingHistoryAttributes) => `outdoordashboard/trackingactivity/${args.customerId}?days=${args.days}`,\n    transformResponse: (response: DashboardTrackingHistoryDto[]) => toDashboardTrackingHistorySummaries(response),\n    transformErrorResponse: (response: { status: string | number }, _meta, _arg) => response.status,\n    providesTags: [{ type: HOURLY_TAG, id: 'LIST' }],\n    keepUnusedDataFor: 300\n  }),\n\n});\n",
    );

    assert_eq!(
        must(registry.desired_indent_for_language("typescript", &buffer, 9, 2)),
        Some(2)
    );
}

#[test]
fn typescript_blank_line_after_outer_object_opener_uses_sibling_indent() {
    let install_root = default_grammars_root();
    if !typescript_grammar_available() {
        eprintln!(
            "SKIP: typescript grammar not found at {}",
            install_root.display()
        );
        return;
    }

    let mut registry = SyntaxRegistry::with_install_root(&install_root);
    registry.set_query_asset_root(Some(query_asset_root()));
    must(registry.register(typescript_config()));

    let buffer = TextBuffer::from_text(
        ";\nexport const Endpoints = (builder: EndpointBuilder<any, any, any>) => ({\n\n  getOutdoorTrackingHistoryByCustomer: builder.query<DashboardTrackingHistory[], TrackingHistoryAttributes, DashboardTrackingHistoryDto[]>({\n    query: (args: TrackingHistoryAttributes) => `outdoordashboard/trackingactivity/${args.customerId}?days=${args.days}`,\n    transformResponse: (response: DashboardTrackingHistoryDto[]) => toDashboardTrackingHistorySummaries(response),\n    transformErrorResponse: (response: { status: string | number }, _meta, _arg) => response.status,\n    providesTags: [{ type: HOURLY_TAG, id: 'LIST' }],\n    keepUnusedDataFor: 300\n  }),\n});\n",
    );

    assert_eq!(
        must(registry.desired_indent_for_language("typescript", &buffer, 2, 2)),
        Some(2)
    );
}

#[test]
fn zig_flat_grammar_uses_bundled_queries() {
    std::thread::Builder::new()
        .name("zig-query-regression".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(zig_flat_grammar_uses_bundled_queries_on_app_sized_stack)
        .expect("spawn zig query regression thread")
        .join()
        .expect("zig query regression thread panicked");
}

fn zig_flat_grammar_uses_bundled_queries_on_app_sized_stack() {
    let install_root = default_grammars_root();
    if !zig_grammar_available() {
        eprintln!("SKIP: zig grammar not found at {}", install_root.display());
        return;
    }

    let mut registry = SyntaxRegistry::with_install_root(&install_root);
    registry.set_query_asset_root(Some(query_asset_root()));
    must(registry.register(zig_config()));

    let buffer = TextBuffer::from_text(
        "const std = @import(\"std\");\npub fn main() void {\n    std.debug.print(\"hi\\n\", .{});\n}\n",
    );
    let snapshot = must(registry.highlight_buffer_for_language("zig", &buffer));
    assert_eq!(snapshot.language_id, "zig");
    assert!(!snapshot.has_errors);
    assert!(
        !snapshot.highlight_spans.is_empty(),
        "zig bundled highlights produced no spans"
    );
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
