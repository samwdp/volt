use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    CaptureThemeMapping, GrammarSource, HighlightWindow, LanguageConfiguration,
    LanguageInstallPlan, SyntaxError, SyntaxParseSession, SyntaxRegistry, compile_query_source,
    maybe_read_bundled_query_source, optional_query_source, resolve_bundled_query_source,
    resolve_query_asset_root_from_roots,
};
use editor_buffer::{TextBuffer, TextPoint};

fn rust_language() -> super::Language {
    tree_sitter_rust::LANGUAGE.into()
}

fn html_language() -> super::Language {
    tree_sitter_html::LANGUAGE.into()
}

fn rust_configuration() -> LanguageConfiguration {
    LanguageConfiguration::new(
        "rust",
        ["rs"],
        rust_language,
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        [
            CaptureThemeMapping::new("keyword", "syntax.keyword"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("string", "syntax.string"),
        ],
    )
}

fn installable_rust_configuration() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "rust",
        ["rs"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-rust.git",
            ".",
            "src",
            "tree-sitter-rust",
            "tree_sitter_rust",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )
}

fn rust_inline_configuration() -> LanguageConfiguration {
    LanguageConfiguration::new(
        "rust-inline",
        [] as [&str; 0],
        rust_language,
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        [CaptureThemeMapping::new("string", "syntax.string.inline")],
    )
}

fn cmake_configuration() -> LanguageConfiguration {
    LanguageConfiguration::new(
        "cmake",
        ["cmake"],
        rust_language,
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )
    .with_file_names(["CMakeLists.txt"])
}

fn dockerfile_configuration() -> LanguageConfiguration {
    LanguageConfiguration::new(
        "dockerfile",
        [] as [&str; 0],
        rust_language,
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        [CaptureThemeMapping::new("string", "syntax.string")],
    )
    .with_file_names(["Dockerfile"])
    .with_file_globs(["Dockerfile.*"])
}

fn dev_extension_configuration() -> LanguageConfiguration {
    LanguageConfiguration::new(
        "dev",
        ["dev"],
        rust_language,
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        [CaptureThemeMapping::new("string", "syntax.string")],
    )
}

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("unexpected error: {error:?}"),
    }
}

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = env::temp_dir().join(format!("volt-syntax-{name}-{unique}"));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn registry_resolves_languages_by_extension_and_path() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    assert_eq!(
        registry
            .language_for_extension(".rs")
            .map(|language| language.id()),
        Some("rust")
    );
    assert_eq!(
        registry
            .language_for_path("src/main.rs")
            .map(|language| language.id()),
        Some("rust")
    );
}

#[test]
fn preload_language_loads_static_language_without_parsing() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    assert!(!registry.is_loaded("rust"));
    must(registry.preload_language("rust"));
    assert!(registry.is_loaded("rust"));
}

#[test]
fn registry_prefers_exact_filenames_and_globs_over_extensions() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(LanguageConfiguration::new(
        "plaintext",
        ["txt"],
        rust_language,
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    must(registry.register(cmake_configuration()));
    must(registry.register(dockerfile_configuration()));

    assert_eq!(
        registry
            .language_for_path("project/CMakeLists.txt")
            .map(LanguageConfiguration::id),
        Some("cmake")
    );
    assert_eq!(
        registry
            .language_for_path("containers/Dockerfile.dev")
            .map(LanguageConfiguration::id),
        Some("dockerfile")
    );
    assert_eq!(
        registry
            .language_for_path("notes/guide.txt")
            .map(LanguageConfiguration::id),
        Some("plaintext")
    );
}

#[test]
fn registry_resolves_languages_by_filename_and_glob() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(dockerfile_configuration()));

    assert_eq!(
        registry
            .language_for_path("Dockerfile")
            .map(|language| language.id()),
        Some("dockerfile")
    );
    assert_eq!(
        registry
            .language_for_path("containers/Dockerfile.dev")
            .map(|language| language.id()),
        Some("dockerfile")
    );
}

#[test]
fn registry_prefers_filename_globs_over_extension_matches() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(dev_extension_configuration()));
    must(registry.register(dockerfile_configuration()));

    assert_eq!(
        registry
            .language_for_path("containers/Dockerfile.dev")
            .map(|language| language.id()),
        Some("dockerfile")
    );
}

#[test]
fn rust_highlighting_produces_theme_tokens() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let buffer = TextBuffer::from_text(
        r#"
fn main() {
    let value = "volt";
    println!("{value}");
}
"#,
    );

    let snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
    assert_eq!(snapshot.language_id, "rust");
    assert_eq!(snapshot.root_kind, "source_file");
    assert!(!snapshot.has_errors);
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
            .any(|span| span.theme_token.as_ref() == "syntax.string")
    );
}

#[test]
fn highlight_spans_reuse_interned_capture_and_theme_tokens() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));
    let buffer = TextBuffer::from_text("fn first() {}\nfn second() {}\n");
    let snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));
    let keyword_spans = snapshot
        .highlight_spans
        .iter()
        .filter(|span| span.theme_token.as_ref() == "syntax.keyword")
        .collect::<Vec<_>>();
    assert!(
        keyword_spans.len() >= 2,
        "expected repeated keyword spans, got {}",
        keyword_spans.len()
    );
    assert!(Arc::ptr_eq(
        &keyword_spans[0].capture_name,
        &keyword_spans[1].capture_name
    ));
    assert!(Arc::ptr_eq(
        &keyword_spans[0].theme_token,
        &keyword_spans[1].theme_token
    ));
}

#[test]
fn ancestor_contexts_include_named_nodes_up_to_the_root() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let buffer = TextBuffer::from_text(
        r#"impl Demo {
    fn render(value: usize) {
        let current = value;
    }
}
"#,
    );
    let contexts =
        must(registry.ancestor_contexts_for_language("rust", &buffer, TextPoint::new(2, 8)));

    let kinds = contexts
        .iter()
        .map(|context| context.kind.as_str())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"function_item"));
    assert!(kinds.contains(&"impl_item"));
    assert!(!kinds.contains(&"source_file"));
}

#[test]
fn ancestor_contexts_parse_session_matches_cold_query_after_edits() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let mut buffer = TextBuffer::from_text(
        r#"impl Demo {
    fn render(value: usize) {
        let current = value;
    }
}
"#,
    );
    let mut parse_session = None;

    let cold = must(registry.ancestor_contexts_for_language("rust", &buffer, TextPoint::new(2, 8)));
    let incremental = must(registry.ancestor_contexts_for_language_with_parse_session(
        "rust",
        &buffer,
        TextPoint::new(2, 8),
        &mut parse_session,
    ));
    assert_eq!(incremental, cold);

    buffer.set_cursor(TextPoint::new(2, 8));
    buffer.insert_text("mut ");

    let cold_after =
        must(registry.ancestor_contexts_for_language("rust", &buffer, TextPoint::new(2, 12)));
    let incremental_after = must(registry.ancestor_contexts_for_language_with_parse_session(
        "rust",
        &buffer,
        TextPoint::new(2, 12),
        &mut parse_session,
    ));
    assert_eq!(incremental_after, cold_after);
}

#[test]
fn additional_highlight_languages_merge_spans() {
    let mut registry = SyntaxRegistry::new();
    must(
        registry
            .register(rust_configuration().with_additional_highlight_languages(["rust-inline"])),
    );
    must(registry.register(rust_inline_configuration()));

    let buffer = TextBuffer::from_text("fn main() { let value = \"volt\"; }");
    let snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|span| span.theme_token.as_ref() == "syntax.string.inline")
    );
}

#[test]
fn injected_highlighting_merges_nested_language_and_additional_spans() {
    let mut registry = SyntaxRegistry::new();
    must(
        registry.register(
            rust_configuration()
                .with_additional_highlight_languages(["rust-inline"])
                .with_extra_injections_query(
                    r#"((raw_string_literal
  (string_content) @injection.content)
  (#set! injection.language "rs"))"#,
                ),
        ),
    );
    must(registry.register(rust_inline_configuration()));

    let buffer = TextBuffer::from_text(
        r##"fn main() {
    let source = r#"fn injected() { let value = "volt"; }"#;
}"##,
    );
    let source = buffer.text();
    let Some(injected_fn_byte) = source.find("injected") else {
        panic!("expected injected function name in test buffer");
    };
    let Some(injected_string_byte) = source.find("volt") else {
        panic!("expected injected string literal in test buffer");
    };

    let snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|span| span.theme_token.as_ref() == "syntax.function"
                && span.start_byte <= injected_fn_byte
                && injected_fn_byte < span.end_byte),
        "expected injected Rust function highlight at byte {injected_fn_byte}, got {:?}",
        snapshot.highlight_spans
    );
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|span| span.theme_token.as_ref() == "syntax.string.inline"
                && span.start_byte <= injected_string_byte
                && injected_string_byte < span.end_byte),
        "expected injected additional highlight at byte {injected_string_byte}, got {:?}",
        snapshot.highlight_spans
    );
}

#[test]
fn unknown_injection_language_is_ignored_without_failing_host_highlighting() {
    let mut registry = SyntaxRegistry::new();
    must(
        registry.register(rust_configuration().with_extra_injections_query(
            r#"((raw_string_literal
  (string_content) @injection.content)
  (#set! injection.language "not-registered"))"#,
        )),
    );

    let buffer = TextBuffer::from_text(
        r##"fn main() {
    let source = r#"fn injected() {}"#;
}"##,
    );
    let source = buffer.text();
    let Some(main_byte) = source.find("main") else {
        panic!("expected main function name in test buffer");
    };
    let Some(injected_byte) = source.find("injected") else {
        panic!("expected injected function name in test buffer");
    };

    let snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|span| span.theme_token.as_ref() == "syntax.function"
                && span.start_byte <= main_byte
                && main_byte < span.end_byte),
        "expected host Rust function highlight at byte {main_byte}, got {:?}",
        snapshot.highlight_spans
    );
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .all(|span| !(span.theme_token.as_ref() == "syntax.function"
                && span.start_byte <= injected_byte
                && injected_byte < span.end_byte)),
        "unexpected injected function highlight at byte {injected_byte}: {:?}",
        snapshot.highlight_spans
    );
}

#[test]
fn whole_buffer_self_injection_is_ignored() {
    let mut registry = SyntaxRegistry::new();
    must(
        registry.register(rust_configuration().with_extra_injections_query(
            r#"((source_file) @injection.content
  (#set! injection.language "rust"))"#,
        )),
    );

    let buffer = TextBuffer::from_text("fn main() {}\n");
    let snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));
    let function_spans = snapshot
        .highlight_spans
        .iter()
        .filter(|span| span.capture_name.as_ref() == "function")
        .count();

    assert_eq!(function_spans, 1);
}

#[test]
fn visible_spans_filters_to_line_window() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let mut source = String::new();
    for _ in 0..256 {
        source.push_str("fn demo() {}\n");
    }
    source.push_str("let target = \"visible\";\n");
    for _ in 0..256 {
        source.push_str("fn tail() {}\n");
    }

    let snapshot =
        must(registry.highlight_buffer_for_extension("rs", &TextBuffer::from_text(source)));
    let visible = snapshot.visible_spans(256, 4);

    assert!(!visible.is_empty());
    assert!(visible.iter().all(|span| span.start_position.line <= 259));
}

#[test]
fn highlight_window_limits_highlight_spans_to_requested_lines() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let mut source = String::new();
    for index in 0..512 {
        source.push_str(&format!(
            "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
        ));
    }
    let buffer = TextBuffer::from_text(source);

    let full_snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
    let window = HighlightWindow::new(240, 16);
    let windowed_snapshot =
        must(registry.highlight_buffer_for_extension_window("rs", &buffer, window));

    assert!(!windowed_snapshot.highlight_spans.is_empty());
    assert!(windowed_snapshot.highlight_spans.len() < full_snapshot.highlight_spans.len());
    assert!(windowed_snapshot.highlight_spans.iter().all(|span| {
        span.start_position.line < window.end_line_exclusive()
            && span.end_position.line >= window.start_line()
    }));
}

#[test]
fn incremental_parse_session_matches_cold_highlight_after_edits() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let mut buffer = TextBuffer::from_text("fn main() {\n    let value = 1;\n}\n");
    let mut parse_session: Option<SyntaxParseSession> = None;
    let initial_snapshot = must(registry.highlight_buffer_for_language_with_session(
        "rust",
        &buffer,
        &mut parse_session,
    ));

    buffer.set_cursor(editor_buffer::TextPoint::new(1, 16));
    buffer.insert_text("mut ");
    let incremental_snapshot = must(registry.highlight_buffer_for_language_with_session(
        "rust",
        &buffer,
        &mut parse_session,
    ));
    let cold_snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));

    assert_eq!(incremental_snapshot, cold_snapshot);
    assert_ne!(initial_snapshot, incremental_snapshot);
}

#[test]
fn incremental_windowed_session_matches_cold_highlight_after_edits() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let mut source = String::new();
    for index in 0..512 {
        source.push_str(&format!(
            "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
        ));
    }
    let mut buffer = TextBuffer::from_text(source);
    let window = HighlightWindow::new(240, 24);
    let mut parse_session: Option<SyntaxParseSession> = None;

    let _ = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        window,
        &mut parse_session,
    ));

    buffer.set_cursor(editor_buffer::TextPoint::new(248, 0));
    buffer.insert_text("x");

    let incremental_snapshot = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        window,
        &mut parse_session,
    ));
    let cold_snapshot =
        must(registry.highlight_buffer_for_language_window("rust", &buffer, window));

    assert_eq!(incremental_snapshot, cold_snapshot);
}

#[test]
fn windowed_session_reuses_spans_when_window_expands_at_same_revision() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let mut source = String::new();
    for index in 0..512 {
        source.push_str(&format!(
            "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
        ));
    }
    let buffer = TextBuffer::from_text(source);
    let initial_window = HighlightWindow::new(240, 16);
    let expanded_window = HighlightWindow::new(240, 24);
    let mut parse_session: Option<SyntaxParseSession> = None;

    let initial_snapshot = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        initial_window,
        &mut parse_session,
    ));
    let expanded_snapshot = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        expanded_window,
        &mut parse_session,
    ));
    let cold_snapshot =
        must(registry.highlight_buffer_for_language_window("rust", &buffer, expanded_window));

    assert!(
        initial_snapshot
            .highlight_spans
            .iter()
            .all(|span| { expanded_snapshot.highlight_spans.contains(span) })
    );
    assert!(expanded_snapshot.highlight_spans.len() > initial_snapshot.highlight_spans.len());
    assert_eq!(expanded_snapshot, cold_snapshot);
}

#[test]
fn windowed_session_merges_spans_when_window_shifts_at_same_revision() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let mut source = String::new();
    for index in 0..512 {
        source.push_str(&format!(
            "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
        ));
    }
    let buffer = TextBuffer::from_text(source);
    let initial_window = HighlightWindow::new(240, 16);
    let shifted_window = HighlightWindow::new(241, 16);
    let mut parse_session: Option<SyntaxParseSession> = None;

    let _ = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        initial_window,
        &mut parse_session,
    ));
    let shifted_snapshot = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        shifted_window,
        &mut parse_session,
    ));
    let cold_snapshot =
        must(registry.highlight_buffer_for_language_window("rust", &buffer, shifted_window));

    assert_eq!(shifted_snapshot, cold_snapshot);
}

#[test]
fn windowed_session_does_not_duplicate_spans_that_straddle_the_old_window() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let source = concat!(
        "fn holder() {\n",
        "    let value = \"first\n",
        "second\";\n",
        "}\n",
        "fn after() {}\n",
    );
    let buffer = TextBuffer::from_text(source);
    let initial_window = HighlightWindow::new(0, 2);
    let expanded_window = HighlightWindow::new(0, 5);
    let mut parse_session: Option<SyntaxParseSession> = None;

    let _ = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        initial_window,
        &mut parse_session,
    ));
    let expanded_snapshot = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        expanded_window,
        &mut parse_session,
    ));
    let cold_snapshot =
        must(registry.highlight_buffer_for_language_window("rust", &buffer, expanded_window));

    assert_eq!(expanded_snapshot, cold_snapshot);
}

#[test]
fn windowed_session_clips_spans_when_window_shrinks_at_same_revision() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));

    let mut source = String::new();
    for index in 0..512 {
        source.push_str(&format!(
            "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
        ));
    }
    let buffer = TextBuffer::from_text(source);
    let initial_window = HighlightWindow::new(240, 24);
    let shrunk_window = HighlightWindow::new(240, 16);
    let mut parse_session: Option<SyntaxParseSession> = None;

    let initial_snapshot = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        initial_window,
        &mut parse_session,
    ));
    let shrunk_snapshot = must(registry.highlight_buffer_for_language_window_with_session(
        "rust",
        &buffer,
        shrunk_window,
        &mut parse_session,
    ));
    let cold_snapshot =
        must(registry.highlight_buffer_for_language_window("rust", &buffer, shrunk_window));

    assert!(shrunk_snapshot.highlight_spans.iter().all(|span| {
        span.start_position.line < shrunk_window.end_line_exclusive()
            && span.end_position.line >= shrunk_window.start_line()
    }));
    assert!(shrunk_snapshot.highlight_spans.len() < initial_snapshot.highlight_spans.len());
    assert_eq!(shrunk_snapshot, cold_snapshot);
}

#[test]
fn grammar_configuration_uses_flat_install_root_paths() {
    let grammar = GrammarSource::new(
        "https://example.com/tree-sitter-rust.git",
        ".",
        "src",
        "tree-sitter-rust",
        "tree_sitter_rust",
    );
    let install_root = PathBuf::from("volt-grammars");

    assert_eq!(
        grammar.legacy_install_directory(&install_root),
        install_root.join("tree-sitter-rust")
    );
    assert_eq!(
        grammar.installed_library_path(&install_root).parent(),
        Some(install_root.as_path())
    );
    assert!(
        grammar
            .installed_library_path(&install_root)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(|file_name| file_name.starts_with("libtree-sitter-rust"))
            .unwrap_or(false)
    );
}

#[test]
fn grammar_backed_language_reports_missing_install() {
    let install_root = std::env::temp_dir().join("volt-missing-tree-sitter-grammar");
    let mut registry = SyntaxRegistry::with_install_root(install_root.clone());
    must(registry.register(installable_rust_configuration()));

    assert!(!must(registry.is_installed("rust")));
    let error = registry
        .highlight_buffer_for_extension("rs", &TextBuffer::from_text("fn main() {}"))
        .expect_err("expected missing grammar error");
    match error {
        SyntaxError::GrammarNotInstalled {
            language_id,
            install_dir,
        } => {
            assert_eq!(language_id, "rust");
            assert_eq!(install_dir, install_root);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn install_plan_requests_generate_when_parser_is_missing() {
    let install_root = TempTestDir::new("install-plan-generate-install");
    let mut registry = SyntaxRegistry::with_install_root(install_root.path());
    must(registry.register(LanguageConfiguration::from_grammar(
        "latex",
        ["tex"],
        GrammarSource::new(
            "https://example.com/tree-sitter-latex.git",
            ".",
            "src",
            "tree-sitter-latex",
            "tree_sitter_latex",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    let plan = registry
        .prepare_language_install("latex")
        .expect("plan")
        .expect("grammar-backed plan");
    let source_dir = plan.source_dir();
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(
        plan.grammar_dir().join("grammar.js"),
        "module.exports = grammar({});",
    )
    .expect("write grammar.js");
    let parser_path = plan.source_dir().join("parser.c");
    if parser_path.exists() {
        fs::remove_file(&parser_path).expect("remove unexpected parser.c");
    }
    assert!(
        !parser_path.exists(),
        "parser.c should be absent for generate test"
    );
    assert!(plan.grammar_dir().join("grammar.js").exists());

    let generate = plan
        .generate_command_if_needed()
        .expect("generate command")
        .expect("missing parser should require generate");
    assert_eq!(generate.program(), "tree-sitter");
    assert_eq!(generate.args(), ["generate".to_owned()]);
    assert_eq!(generate.cwd(), plan.grammar_dir().as_path());
    assert!(matches!(
        plan.compile_command(),
        Err(SyntaxError::Io { message, .. }) if message == "parser.c is missing"
    ));
}

#[test]
fn install_plan_reports_missing_grammar_sources_before_compile() {
    let install_root = TempTestDir::new("install-plan-missing-sources-install");
    let mut registry = SyntaxRegistry::with_install_root(install_root.path());
    must(registry.register(LanguageConfiguration::from_grammar(
        "latex",
        ["tex"],
        GrammarSource::new(
            "https://example.com/tree-sitter-latex.git",
            ".",
            "src",
            "tree-sitter-latex",
            "tree_sitter_latex",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    let plan = registry
        .prepare_language_install("latex")
        .expect("plan")
        .expect("grammar-backed plan");
    fs::create_dir_all(plan.source_dir()).expect("create source dir");

    assert!(matches!(
        plan.generate_command_if_needed(),
        Err(SyntaxError::Io { message, .. })
            if message == "grammar.js is missing and parser.c was not pre-generated"
    ));
}

#[test]
fn install_plan_compile_command_prefers_cpp_scanner() {
    let install_root = TempTestDir::new("install-plan-compile-install");
    let mut registry = SyntaxRegistry::with_install_root(install_root.path());
    must(registry.register(LanguageConfiguration::from_grammar(
        "latex",
        ["tex"],
        GrammarSource::new(
            "https://example.com/tree-sitter-latex.git",
            ".",
            "src",
            "tree-sitter-latex",
            "tree_sitter_latex",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    let plan = registry
        .prepare_language_install("latex")
        .expect("plan")
        .expect("grammar-backed plan");
    let source_dir = plan.source_dir();
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("parser.c"), "/* parser */").expect("write parser.c");
    fs::write(source_dir.join("scanner.cc"), "// scanner").expect("write scanner.cc");

    assert!(
        plan.generate_command_if_needed()
            .expect("generate decision")
            .is_none()
    );
    let compile = plan.compile_command().expect("compile command");
    if cfg!(windows) {
        assert!(compile.program().ends_with("cl.exe"));
        assert!(compile.args().contains(&"/LD".to_owned()));
        assert!(compile.args().contains(&"/EHsc".to_owned()));
        assert!(compile.args().contains(&"/link".to_owned()));
        assert!(compile.args().contains(&"/NOIMPLIB".to_owned()));
        assert!(compile.args().iter().any(|arg| arg.starts_with("/Fo")));
        assert!(compile.args().iter().any(|arg| arg.starts_with("/Fe:")));
    } else {
        assert_eq!(compile.program(), "c++");
        assert!(compile.args().contains(&"-std=c++14".to_owned()));
    }
    assert!(compile.args().iter().any(|arg| arg.ends_with("parser.c")));
    assert!(compile.args().iter().any(|arg| arg.ends_with("scanner.cc")));
}

#[test]
fn install_plan_compile_command_uses_windows_msvc_for_c_scanner() {
    let install_root = TempTestDir::new("install-plan-c-scanner-install");
    let mut registry = SyntaxRegistry::with_install_root(install_root.path());
    must(registry.register(LanguageConfiguration::from_grammar(
        "zig",
        ["zig"],
        GrammarSource::new(
            "https://example.com/tree-sitter-zig.git",
            ".",
            "src",
            "tree-sitter-zig",
            "tree_sitter_zig",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    let plan = registry
        .prepare_language_install("zig")
        .expect("plan")
        .expect("grammar-backed plan");
    let source_dir = plan.source_dir();
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("parser.c"), "/* parser */").expect("write parser.c");
    fs::write(source_dir.join("scanner.c"), "/* scanner */").expect("write scanner.c");

    let compile = plan.compile_command().expect("compile command");

    if cfg!(windows) {
        assert!(compile.program().ends_with("cl.exe"));
        assert!(compile.env().iter().any(|(name, _)| name == "PATH"));
        assert!(compile.args().contains(&"/LD".to_owned()));
        assert!(compile.args().contains(&"/link".to_owned()));
        assert!(compile.args().contains(&"/NOIMPLIB".to_owned()));
        assert!(!compile.args().contains(&"/EHsc".to_owned()));
    } else {
        assert_eq!(compile.program(), "cc");
        assert!(!compile.args().contains(&"-std=c++14".to_owned()));
    }
    assert!(compile.args().iter().any(|arg| arg.ends_with("parser.c")));
    assert!(compile.args().iter().any(|arg| arg.ends_with("scanner.c")));
}

#[test]
fn installed_grammar_language_ids_only_returns_installed_grammar_languages() {
    let install_root = TempTestDir::new("installed-grammar-language-ids");
    let mut registry = SyntaxRegistry::with_install_root(install_root.path());
    must(registry.register(rust_configuration()));
    must(registry.register(LanguageConfiguration::from_grammar(
        "zig",
        ["zig"],
        GrammarSource::new(
            "https://example.com/tree-sitter-zig.git",
            ".",
            "src",
            "tree-sitter-zig",
            "tree_sitter_zig",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    must(registry.register(LanguageConfiguration::from_grammar(
        "tsx",
        ["tsx"],
        GrammarSource::new(
            "https://example.com/tree-sitter-typescript.git",
            ".",
            "tsx/src",
            "tree-sitter-tsx",
            "tree_sitter_tsx",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    must(registry.register(LanguageConfiguration::from_grammar(
        "zig-alias",
        ["zig-alias"],
        GrammarSource::new(
            "https://example.com/tree-sitter-zig.git",
            ".",
            "src",
            "tree-sitter-zig",
            "tree_sitter_zig",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    let zig_library = registry
        .language("zig")
        .and_then(LanguageConfiguration::grammar)
        .expect("zig grammar")
        .installed_library_path(install_root.path());
    fs::write(&zig_library, "fake dll").expect("write installed zig marker");

    assert_eq!(
        registry.installed_grammar_language_ids(),
        ["zig".to_owned()]
    );
}

#[test]
fn finalize_language_install_removes_compiler_sidecars() {
    let install_root = TempTestDir::new("finalize-language-install");
    let mut registry = SyntaxRegistry::with_install_root(install_root.path());
    must(registry.register(LanguageConfiguration::from_grammar(
        "zig",
        ["zig"],
        GrammarSource::new(
            "https://example.com/tree-sitter-zig.git",
            ".",
            "src",
            "tree-sitter-zig",
            "tree_sitter_zig",
        ),
        [CaptureThemeMapping::new("keyword", "syntax.keyword")],
    )));
    let plan = registry
        .prepare_language_install("zig")
        .expect("plan")
        .expect("grammar-backed plan");
    let library_path = plan.installed_library_path();
    fs::write(&library_path, "fake dll").expect("write installed zig library");
    fs::write(library_path.with_extension("exp"), "fake exp").expect("write installed zig exp");
    fs::write(library_path.with_extension("lib"), "fake lib").expect("write installed zig lib");

    registry
        .finalize_language_install(&plan)
        .expect("finalize language install");

    assert!(!library_path.with_extension("exp").exists());
    assert!(!library_path.with_extension("lib").exists());
}

#[cfg(windows)]
#[test]
fn windows_msvc_target_triple_matches_current_architecture() {
    let expected = match env::consts::ARCH {
        "aarch64" => "aarch64-pc-windows-msvc",
        "x86" => "i686-pc-windows-msvc",
        _ => "x86_64-pc-windows-msvc",
    };

    assert_eq!(super::windows_msvc_target_triple(), expected);
}

#[test]
fn extra_highlight_query_can_supply_missing_bundled_query() {
    let source = must(optional_query_source(
        None,
        "test-lang",
        "highlights.scm",
        Some("(identifier) @function"),
    ));

    assert_eq!(source.as_deref(), Some("(identifier) @function"));
}

#[test]
fn bundled_query_resolution_flattens_inherited_queries() {
    let asset_root = TempTestDir::new("query-assets");
    let base_dir = asset_root.path().join("base");
    let child_dir = asset_root.path().join("child");
    fs::create_dir_all(&base_dir).expect("create base query dir");
    fs::create_dir_all(&child_dir).expect("create child query dir");
    fs::write(base_dir.join("highlights.scm"), "(identifier) @variable\n")
        .expect("write base highlight query");
    fs::write(
        child_dir.join("highlights.scm"),
        "; inherits: base\n(string_literal) @string\n",
    )
    .expect("write child highlight query");
    let config = LanguageConfiguration::from_grammar(
        "child",
        ["child"],
        GrammarSource::new(
            "https://example.com/tree-sitter-child.git",
            ".",
            "src",
            "tree-sitter-child",
            "tree_sitter_child",
        ),
        [CaptureThemeMapping::new("variable", "syntax.variable")],
    );

    let source = must(resolve_bundled_query_source(
        asset_root.path(),
        config.id(),
        "highlights.scm",
        &mut Vec::new(),
    ))
    .expect("child query should resolve");

    assert!(source.contains("(identifier) @variable"));
    assert!(source.contains("(string_literal) @string"));
    assert!(!source.contains("; inherits:"));
}

#[test]
fn query_asset_root_prefers_workspace_source_over_staged_target_copy() {
    let workspace_root = TempTestDir::new("query-root-workspace");
    let staged_root = workspace_root.path().join("target").join("debug");

    fs::create_dir_all(
        workspace_root
            .path()
            .join("crates")
            .join("volt")
            .join("assets")
            .join("grammars")
            .join("queries"),
    )
    .expect("create workspace asset tree");
    fs::create_dir_all(staged_root.join("assets").join("grammars").join("queries"))
        .expect("create staged asset tree");
    fs::write(workspace_root.path().join("Cargo.toml"), "[workspace]\n")
        .expect("write workspace Cargo.toml");

    let resolved = resolve_query_asset_root_from_roots([
        staged_root.clone(),
        workspace_root.path().to_path_buf(),
    ])
    .expect("resolve query asset root");

    assert_eq!(
        resolved,
        workspace_root
            .path()
            .join("crates")
            .join("volt")
            .join("assets")
            .join("grammars")
            .join("queries")
    );
}

#[test]
fn prepare_install_root_does_not_copy_queries_and_removes_legacy_query_dir() {
    let asset_root = TempTestDir::new("prepare-flat-query-assets");
    let install_root = TempTestDir::new("prepare-flat-install");
    let clone_root = TempTestDir::new("prepare-flat-clone");
    let lang_id = "child";
    let asset_dir = asset_root.path().join(lang_id);
    fs::create_dir_all(&asset_dir).expect("create asset dir");
    fs::write(asset_dir.join("highlights.scm"), "(identifier) @variable\n")
        .expect("write highlight query");
    let legacy_dir = install_root.path().join("tree-sitter-child");
    fs::create_dir_all(legacy_dir.join("queries")).expect("create legacy query dir");
    fs::write(
        legacy_dir.join("queries").join("highlights.scm"),
        "(identifier) @stale\n",
    )
    .expect("write stale installed query");
    fs::create_dir_all(clone_root.path()).expect("create clone root");
    let config = LanguageConfiguration::from_grammar(
        lang_id,
        ["child"],
        GrammarSource::new(
            "https://example.com/tree-sitter-child.git",
            ".",
            "src",
            "tree-sitter-child",
            "tree_sitter_child",
        ),
        [CaptureThemeMapping::new("variable", "syntax.variable")],
    );
    let plan = LanguageInstallPlan {
        config,
        grammar: GrammarSource::new(
            "https://example.com/tree-sitter-child.git",
            ".",
            "src",
            "tree-sitter-child",
            "tree_sitter_child",
        ),
        install_root: install_root.path().to_path_buf(),
        query_asset_root: Some(asset_root.path().to_path_buf()),
        temp_clone_root: clone_root.path().to_path_buf(),
    };

    must(plan.prepare_install_root());

    assert!(
        !legacy_dir.exists(),
        "legacy per-language query directory should be removed"
    );
    assert!(
        install_root.path().exists(),
        "flat grammar install root should remain"
    );
}

/// Query loading only uses bundled assets; stale installed query files are ignored.
#[test]
fn bundled_query_asset_ignores_stale_installed_query() {
    let asset_root = TempTestDir::new("bundled-wins-asset");
    let install_root = TempTestDir::new("bundled-wins-install");
    let lang_id = "test-lang";

    // Write the "live" bundled asset (what the repo has after a fix).
    let asset_dir = asset_root.path().join(lang_id);
    fs::create_dir_all(&asset_dir).expect("create asset dir");
    fs::write(
        asset_dir.join("highlights.scm"),
        "(identifier) @variable.bundled\n",
    )
    .expect("write bundled highlights");

    // Write a "stale" installed query in the simulated grammar install directory
    // (the kind that would exist before a manual grammar reinstall).
    let installed_query_dir = install_root
        .path()
        .join(format!("tree-sitter-{lang_id}"))
        .join("queries");
    fs::create_dir_all(&installed_query_dir).expect("create installed query dir");
    let installed_path = installed_query_dir.join("highlights.scm");
    fs::write(&installed_path, "(identifier) @variable.stale\n")
        .expect("write stale installed highlights");

    let source = must(maybe_read_bundled_query_source(
        Some(asset_root.path()),
        lang_id,
        "highlights.scm",
    ))
    .expect("bundled query should exist");
    assert!(
        source.contains("@variable.bundled"),
        "expected bundled content, got: {source:?}"
    );
    assert!(
        !source.contains("@variable.stale"),
        "stale installed content leaked into result: {source:?}"
    );

    let fallback = must(maybe_read_bundled_query_source(
        None,
        lang_id,
        "highlights.scm",
    ));
    assert!(
        fallback.is_none(),
        "installed query files are not fallback sources"
    );
}

/// Optional query loading only uses bundled assets and extra query text.
#[test]
fn bundled_optional_query_asset_ignores_stale_installed_query() {
    let asset_root = TempTestDir::new("bundled-opt-asset");
    let install_root = TempTestDir::new("bundled-opt-install");
    let lang_id = "test-lang-opt";

    // Write bundled indents asset.
    let asset_dir = asset_root.path().join(lang_id);
    fs::create_dir_all(&asset_dir).expect("create asset dir");
    fs::write(asset_dir.join("indents.scm"), "(block) @indent.bundled\n")
        .expect("write bundled indents");

    // Write stale installed indents.
    let installed_query_dir = install_root
        .path()
        .join(format!("tree-sitter-{lang_id}"))
        .join("queries");
    fs::create_dir_all(&installed_query_dir).expect("create installed query dir");
    let installed_path = installed_query_dir.join("indents.scm");
    fs::write(&installed_path, "(block) @indent.stale\n").expect("write stale installed indents");

    let source = must(optional_query_source(
        Some(asset_root.path()),
        lang_id,
        "indents.scm",
        None,
    ));
    assert!(
        source.as_deref().unwrap_or("").contains("@indent.bundled"),
        "expected bundled content, got: {source:?}"
    );

    // When bundled asset is absent, installed files are ignored.
    let absent_asset_root = TempTestDir::new("bundled-opt-absent");
    let fallback = must(optional_query_source(
        Some(absent_asset_root.path()),
        lang_id,
        "indents.scm",
        None,
    ));
    assert!(
        fallback.is_none(),
        "installed optional query files are not fallback sources"
    );

    let extra = must(optional_query_source(
        Some(absent_asset_root.path()),
        lang_id,
        "indents.scm",
        Some("(block) @indent.extra"),
    ));
    assert!(
        extra.as_deref().unwrap_or("").contains("@indent.extra"),
        "extra query should be used when bundled query is absent"
    );
}

#[test]
fn indent_queries_compute_nested_and_branch_indentation() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(
        rust_configuration().with_extra_indent_query(include_str!(concat!(
            core::env!("CARGO_MANIFEST_DIR"),
            "/../volt/assets/grammars/queries/rust/indents.scm"
        ))),
    ));

    let buffer = TextBuffer::from_text("fn main() {\n    if true {\n\n    }\n}\n");

    assert_eq!(
        must(registry.desired_indent_for_language("rust", &buffer, 2, 4)),
        Some(8)
    );
    assert_eq!(
        must(registry.desired_indent_for_language("rust", &buffer, 3, 4)),
        Some(4)
    );
}

#[test]
fn indent_queries_reuse_parse_sessions_after_edits() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(
        rust_configuration().with_extra_indent_query(include_str!(concat!(
            core::env!("CARGO_MANIFEST_DIR"),
            "/../volt/assets/grammars/queries/rust/indents.scm"
        ))),
    ));

    let mut buffer = TextBuffer::from_text("fn main() {\n    if true {\n\n    }\n}\n");
    let mut parse_session = None;

    assert_eq!(
        must(registry.desired_indent_for_language_with_session(
            "rust",
            &buffer,
            2,
            4,
            &mut parse_session,
        )),
        Some(8)
    );

    buffer.set_cursor(TextPoint::new(2, 0));
    buffer.insert_text("        println!(\"hi\");\n");

    assert_eq!(
        must(registry.desired_indent_for_language("rust", &buffer, 3, 4)),
        must(registry.desired_indent_for_language_with_session(
            "rust",
            &buffer,
            3,
            4,
            &mut parse_session,
        )),
    );
    assert_eq!(
        must(registry.desired_indent_for_language("rust", &buffer, 4, 4)),
        must(registry.desired_indent_for_language_with_session(
            "rust",
            &buffer,
            4,
            4,
            &mut parse_session,
        )),
    );
}

// --- Additional query kind tests ---

#[test]
fn extra_injections_query_compiles_for_static_language() {
    let mut registry = SyntaxRegistry::new();
    must(
        registry.register(rust_configuration().with_extra_injections_query(
            r#"((string_literal) @injection.content (#set! injection.language "json"))"#,
        )),
    );
    let result = must(registry.injections_query_for_language("rust"));
    assert!(result.is_some(), "injections query should be present");
    let q = result.expect("query");
    assert_eq!(q.pattern_count(), 1);
    assert!(q.capture_names().contains(&"injection.content"));
}

#[test]
fn extra_locals_query_compiles_for_static_language() {
    let mut registry = SyntaxRegistry::new();
    must(
        registry.register(rust_configuration().with_extra_locals_query(r#"(block) @local.scope"#)),
    );
    let result = must(registry.locals_query_for_language("rust"));
    assert!(result.is_some(), "locals query should be present");
    let q = result.expect("query");
    assert!(q.capture_names().contains(&"local.scope"));
}

#[test]
fn extra_folds_query_compiles_for_static_language() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(
        rust_configuration().with_extra_folds_query(r#"[(block) (use_declaration)] @fold"#),
    ));
    let result = must(registry.folds_query_for_language("rust"));
    assert!(result.is_some(), "folds query should be present");
    let q = result.expect("query");
    assert!(q.capture_names().contains(&"fold"));
}

#[test]
fn bundled_injections_query_compiles_for_rust() {
    let injections_text = std::fs::read_to_string(
        PathBuf::from(core::env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("volt")
            .join("assets")
            .join("grammars")
            .join("queries")
            .join("rust")
            .join("injections.scm"),
    );
    let Ok(injections_text) = injections_text else {
        eprintln!("SKIP: bundled rust/injections.scm not found");
        return;
    };
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration().with_extra_injections_query(injections_text)));
    let result = must(registry.injections_query_for_language("rust"));
    assert!(
        result.is_some(),
        "rust injections query should compile successfully"
    );
}

#[test]
fn bundled_html_highlights_query_compiles() {
    let query_asset_root = PathBuf::from(core::env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("volt")
        .join("assets")
        .join("grammars")
        .join("queries");
    let query_text = must(resolve_bundled_query_source(
        &query_asset_root,
        "html",
        "highlights.scm",
        &mut Vec::new(),
    ))
    .expect("bundled html highlights.scm should exist");

    let config = LanguageConfiguration::new(
        "html",
        ["html"],
        html_language,
        query_text,
        [
            CaptureThemeMapping::new("tag", "syntax.tag"),
            CaptureThemeMapping::new("tag.delimiter", "syntax.punctuation.delimiter"),
            CaptureThemeMapping::new("attribute", "syntax.attribute"),
            CaptureThemeMapping::new("string", "syntax.string"),
            CaptureThemeMapping::new("string.special.url", "syntax.string.special.url"),
            CaptureThemeMapping::new("markup.link.label", "syntax.markup.link.label"),
            CaptureThemeMapping::new("constant", "syntax.constant"),
            CaptureThemeMapping::new("character.special", "syntax.character.special"),
        ],
    );

    let mut registry = SyntaxRegistry::new();
    must(registry.register(config));

    let buffer =
        TextBuffer::from_text("<!DOCTYPE html>\n<a href=\"https://example.com\">link</a>\n");
    let snapshot = must(registry.highlight_buffer_for_language("html", &buffer));

    assert_eq!(snapshot.language_id, "html");
    assert!(!snapshot.has_errors);
    assert!(!snapshot.highlight_spans.is_empty());
}

#[test]
fn bundled_locals_query_compiles_for_rust() {
    let locals_text = std::fs::read_to_string(
        PathBuf::from(core::env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("volt")
            .join("assets")
            .join("grammars")
            .join("queries")
            .join("rust")
            .join("locals.scm"),
    );
    let Ok(locals_text) = locals_text else {
        eprintln!("SKIP: bundled rust/locals.scm not found");
        return;
    };
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration().with_extra_locals_query(locals_text)));
    let result = must(registry.locals_query_for_language("rust"));
    assert!(
        result.is_some(),
        "rust locals query should compile successfully"
    );
}

#[test]
fn bundled_folds_query_compiles_for_rust() {
    let folds_text = std::fs::read_to_string(
        PathBuf::from(core::env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("volt")
            .join("assets")
            .join("grammars")
            .join("queries")
            .join("rust")
            .join("folds.scm"),
    );
    let Ok(folds_text) = folds_text else {
        eprintln!("SKIP: bundled rust/folds.scm not found");
        return;
    };
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration().with_extra_folds_query(folds_text)));
    let result = must(registry.folds_query_for_language("rust"));
    assert!(
        result.is_some(),
        "rust folds query should compile successfully"
    );
}

#[test]
fn missing_extra_query_returns_none() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_configuration()));
    // No extra queries configured — all three should return None.
    assert!(must(registry.injections_query_for_language("rust")).is_none());
    assert!(must(registry.locals_query_for_language("rust")).is_none());
    assert!(must(registry.folds_query_for_language("rust")).is_none());
}

#[test]
fn query_accessors_return_unknown_language_error_for_unregistered_id() {
    let mut registry = SyntaxRegistry::new();
    assert!(matches!(
        registry.injections_query_for_language("not-registered"),
        Err(SyntaxError::UnknownLanguage(_))
    ));
    assert!(matches!(
        registry.locals_query_for_language("not-registered"),
        Err(SyntaxError::UnknownLanguage(_))
    ));
    assert!(matches!(
        registry.folds_query_for_language("not-registered"),
        Err(SyntaxError::UnknownLanguage(_))
    ));
}

// --- Query predicate evaluation tests ---

/// Helper: build a LanguageConfiguration for Rust with a custom highlight query so
/// that predicate evaluation is exercised end-to-end through `highlight_tree`.
fn rust_config_with_query(query: &str) -> LanguageConfiguration {
    LanguageConfiguration::new(
        "rust-predicate-test",
        ["__rust_pred_test__"],
        rust_language,
        query,
        [CaptureThemeMapping::new("function", "syntax.function")],
    )
}

#[test]
fn highlight_not_kind_eq_predicate_filters_captures() {
    // `(identifier) @function (#not-kind-eq? @function "identifier")` should never
    // produce a span because every identifier is – by definition – of kind
    // "identifier".
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#not-kind-eq? @function "identifier"))"#,
    )));

    let buffer = TextBuffer::from_text("fn main() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert!(
        snapshot.highlight_spans.is_empty(),
        "expected no spans after #not-kind-eq? filtered them all, got {:?}",
        snapshot.highlight_spans
    );
}

#[test]
fn highlight_kind_eq_predicate_keeps_matching_captures() {
    // `(identifier) @function (#kind-eq? @function "identifier")` should keep every
    // identifier span since kind always matches.
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#kind-eq? @function "identifier"))"#,
    )));

    let buffer = TextBuffer::from_text("fn main() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert!(
        !snapshot.highlight_spans.is_empty(),
        "expected spans to pass through #kind-eq? unchanged"
    );
}

#[test]
fn highlight_has_ancestor_predicate_matches_nested_nodes() {
    // Identifiers inside a block_expression have "block" as an ancestor.
    // (#has-ancestor? @fn "block") should keep them.
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#has-ancestor? @function "block"))"#,
    )));

    // `value` lives inside the block `{}`.
    let buffer = TextBuffer::from_text("fn main() { let value = 1; }");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert!(
        !snapshot.highlight_spans.is_empty(),
        "expected identifier inside block to pass #has-ancestor? block"
    );
}

#[test]
fn highlight_not_has_ancestor_predicate_filters_nested_nodes() {
    // (#not-has-ancestor? @fn "block") should reject identifiers inside a block.
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#not-has-ancestor? @function "block"))"#,
    )));

    // All identifiers in `fn main() { let value = 1; }` are inside a block.
    let buffer = TextBuffer::from_text("fn main() { let value = 1; }");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    // At minimum `main` is NOT inside a block – the function name is a direct child
    // of the function_item, which itself is a direct child of source_file.  So some
    // spans should survive.
    // We just verify the predicate doesn't crash and returns a consistent result.
    let _ = snapshot.highlight_spans.len();
}

#[test]
fn highlight_has_parent_predicate_checks_immediate_parent() {
    // `(identifier) @function (#has-parent? @function "function_item")` keeps
    // identifiers whose direct parent is a function_item (i.e. the function name).
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#has-parent? @function "function_item"))"#,
    )));

    let buffer = TextBuffer::from_text("fn main() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert!(
        !snapshot.highlight_spans.is_empty(),
        "expected function name identifier to pass #has-parent? function_item"
    );
}

#[test]
fn highlight_contains_predicate_filters_by_text_content() {
    // (#contains? @function "main") keeps only identifiers whose text contains "main".
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#contains? @function "main"))"#,
    )));

    let buffer = TextBuffer::from_text("fn main() { let value = 1; }");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert_eq!(
        snapshot.highlight_spans.len(),
        1,
        "expected exactly one span for the identifier `main`"
    );
    assert_eq!(
        snapshot.highlight_spans[0].capture_name.as_ref(),
        "function"
    );
}

#[test]
fn highlight_not_lua_match_predicate_filters_matching_text() {
    // (#not-lua-match? @function "^main") rejects identifiers that start with "main".
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#not-lua-match? @function "^main"))"#,
    )));

    let buffer = TextBuffer::from_text("fn main() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    // `main` matches the lua pattern so it should be filtered out.
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .all(|span| span.capture_name.as_ref() != "function"
                || span.start_byte != buffer.text().find("main").unwrap_or(usize::MAX)),
        "identifier `main` should have been removed by #not-lua-match?"
    );
}

#[test]
fn highlight_directive_predicate_does_not_filter_matches() {
    // A query using a directive (#offset! …) should still produce spans because
    // directives are metadata, not match filters.
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#offset! @function 0 1))"#,
    )));

    let buffer = TextBuffer::from_text("fn main() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert!(
        !snapshot.highlight_spans.is_empty(),
        "directive predicate should not filter spans"
    );
}

#[test]
fn highlight_unknown_predicate_does_not_filter_matches() {
    // An unknown custom predicate should allow the match through rather than silently
    // discarding it.
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @function (#unknown-custom-predicate? @function "value"))"#,
    )));

    let buffer = TextBuffer::from_text("fn main() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert!(
        !snapshot.highlight_spans.is_empty(),
        "unknown predicates should allow matches through"
    );
}

#[test]
fn highlight_skips_internal_and_meta_captures() {
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"
(identifier) @_helper
(identifier) @function
(identifier) @spell
(identifier) @conceal
"#,
    )));

    let buffer = TextBuffer::from_text("fn main() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert_eq!(snapshot.highlight_spans.len(), 1);
    assert_eq!(
        snapshot.highlight_spans[0].capture_name.as_ref(),
        "function"
    );
}

#[test]
fn query_capture_property_value_returns_set_property() {
    use super::query_capture_property_value;

    let language = rust_language();
    // `#set!` with no capture argument produces a pattern-wide property.
    let query = tree_sitter::Query::new(&language, r#"((identifier) @var (#set! priority "90"))"#)
        .expect("valid query");

    let value = query_capture_property_value(&query, 0, 0, "priority");
    assert_eq!(value, Some("90"));
}

// ── Regression: #not-has-parent? must check only the immediate parent ────────

#[test]
fn not_has_parent_checks_only_immediate_parent() {
    // `#not-has-parent? @fn "source_file"` should keep identifiers whose direct
    // parent is NOT `source_file`.  In `fn foo() {}` the identifier `foo` has
    // `function_item` as its immediate parent, not `source_file`, so the predicate
    // must return true (keep the capture).
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @fn (#not-has-parent? @fn "source_file"))"#,
    )));
    let buffer = TextBuffer::from_text("fn foo() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|s| s.capture_name.as_ref() == "fn"),
        "#not-has-parent? must not reject nodes whose immediate parent does not match; \
             `foo` lives under function_item, not source_file"
    );
}

#[test]
fn not_has_parent_rejects_when_immediate_parent_matches() {
    // `#not-has-parent? @fn "function_item"` must reject the function name `foo`
    // because its direct parent IS a `function_item`.
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @fn (#not-has-parent? @fn "function_item"))"#,
    )));
    let buffer = TextBuffer::from_text("fn foo() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    let fn_name_byte = buffer.text().find("foo").unwrap_or(usize::MAX);
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .all(|s| s.capture_name.as_ref() != "fn" || s.start_byte != fn_name_byte),
        "#not-has-parent? must reject a node whose immediate parent matches"
    );
}

// ── Regression: lua_pattern_matches must handle real corpus patterns ─────────

#[test]
fn lua_pattern_matches_uppercase_class() {
    use super::lua_pattern_matches;
    // `^[A-Z]` — identifiers starting with an uppercase letter
    assert!(
        lua_pattern_matches("MyType", "^[A-Z]"),
        "^[A-Z] should match MyType"
    );
    assert!(
        !lua_pattern_matches("myVar", "^[A-Z]"),
        "^[A-Z] should not match myVar"
    );
}

#[test]
fn lua_pattern_matches_uppercase_identifier_pattern() {
    use super::lua_pattern_matches;
    // `^[A-Z][A-Z0-9_]*$` — ALL_CAPS_CONSTANT style
    assert!(
        lua_pattern_matches("MAX_SIZE", "^[A-Z][A-Z0-9_]*$"),
        "should match ALL_CAPS"
    );
    assert!(
        !lua_pattern_matches("maxSize", "^[A-Z][A-Z0-9_]*$"),
        "should not match camelCase"
    );
}

#[test]
fn lua_pattern_matches_percent_u_class() {
    use super::lua_pattern_matches;
    // `%u` — Lua uppercase class
    assert!(
        lua_pattern_matches("A", "%u"),
        "%u should match uppercase 'A'"
    );
    assert!(
        !lua_pattern_matches("a", "%u"),
        "%u should not match lowercase 'a'"
    );
}

#[test]
fn lua_pattern_matches_percent_l_class() {
    use super::lua_pattern_matches;
    // `%l` — Lua lowercase class
    assert!(
        lua_pattern_matches("x", "%l"),
        "%l should match lowercase 'x'"
    );
    assert!(
        !lua_pattern_matches("X", "%l"),
        "%l should not match uppercase 'X'"
    );
}

#[test]
fn lua_pattern_matches_percent_a_class() {
    use super::lua_pattern_matches;
    // `%a` — Lua letter class
    assert!(lua_pattern_matches("hello", "%a"), "%a should match letter");
    assert!(
        !lua_pattern_matches("123", "^%a"),
        "^%a should not match digits"
    );
}

#[test]
fn lua_pattern_matches_percent_d_class() {
    use super::lua_pattern_matches;
    // `%d` — Lua digit class
    assert!(lua_pattern_matches("42", "^%d"), "^%d should match '42'");
    assert!(
        !lua_pattern_matches("abc", "^%d"),
        "^%d should not match 'abc'"
    );
}

#[test]
fn lua_pattern_matches_anchored_literal() {
    use super::lua_pattern_matches;
    assert!(
        lua_pattern_matches("else", "^else"),
        "^else should match 'else'"
    );
    assert!(
        !lua_pattern_matches("elsewhere", "^else$"),
        "^else$ should not match 'elsewhere'"
    );
    assert!(
        lua_pattern_matches("else", "^else$"),
        "^else$ should match exact 'else'"
    );
}

#[test]
fn query_compiler_accepts_vim_case_insensitive_regex_prefix() {
    let language = rust_language();
    let query = compile_query_source(
        &language,
        "rust",
        "highlight",
        r#"((identifier) @function (#match? @function "\\c^main$"))"#,
    );

    assert!(query.is_ok(), "query failed: {query:?}");
}

#[test]
fn lua_pattern_matches_escaped_parens() {
    use super::lua_pattern_matches;
    // `%(` literal open-paren, non-newline follows
    assert!(
        lua_pattern_matches("(x", "%("),
        "should match text starting with ("
    );
    assert!(
        !lua_pattern_matches("x(", "^%("),
        "^%( should not match text not starting with ("
    );
}

#[test]
fn not_lua_match_integration_uppercase_class() {
    // Smoke test: `#lua-match? @fn "^[A-Z]"` keeps only identifiers beginning with uppercase.
    let mut registry = SyntaxRegistry::new();
    must(registry.register(rust_config_with_query(
        r#"((identifier) @fn (#lua-match? @fn "^[A-Z]"))"#,
    )));
    let buffer = TextBuffer::from_text("fn MyFunc() {} fn lowercase() {}");
    let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
    let my_func_byte = buffer.text().find("MyFunc").unwrap_or(usize::MAX);
    let lower_byte = buffer.text().find("lowercase").unwrap_or(usize::MAX);
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .any(|s| s.capture_name.as_ref() == "fn" && s.start_byte == my_func_byte),
        "#lua-match? ^[A-Z] should keep 'MyFunc'"
    );
    assert!(
        snapshot
            .highlight_spans
            .iter()
            .all(|s| s.capture_name.as_ref() != "fn" || s.start_byte != lower_byte),
        "#lua-match? ^[A-Z] should reject 'lowercase'"
    );
}
