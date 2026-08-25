use criterion::{Criterion, black_box, criterion_group, criterion_main};
use editor_buffer::TextBuffer;
use editor_syntax::{CaptureThemeMapping, HighlightWindow, LanguageConfiguration, SyntaxRegistry};

fn rust_language() -> editor_syntax::Language {
    tree_sitter_rust::LANGUAGE.into()
}

fn rust_registry() -> SyntaxRegistry {
    let mut registry = SyntaxRegistry::new();
    registry
        .register(LanguageConfiguration::new(
            "rust",
            ["rs"],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [
                CaptureThemeMapping::new("keyword", "syntax.keyword"),
                CaptureThemeMapping::new("function", "syntax.function"),
                CaptureThemeMapping::new("string", "syntax.string"),
            ],
        ))
        .expect("register rust language");
    registry
}

fn rust_fixture() -> String {
    let mut source = String::from("pub fn demo(values: &[i32]) -> i32 {\n    let mut total = 0;\n");
    for index in 0..80 {
        source.push_str("    total += values.get(");
        source.push_str(&index.to_string());
        source.push_str(").copied().unwrap_or(0);\n");
    }
    source.push_str("    total\n}\n");
    source
}

fn bench_highlight_rust(c: &mut Criterion) {
    let mut registry = rust_registry();
    let buffer = TextBuffer::from_text(rust_fixture());
    c.bench_function("highlight_rust_full", |b| {
        b.iter(|| {
            let snapshot = registry
                .highlight_buffer_for_language("rust", black_box(&buffer))
                .expect("highlight rust buffer");
            black_box(snapshot.highlight_count());
        });
    });
}

fn bench_highlight_rust_window(c: &mut Criterion) {
    let mut registry = rust_registry();
    let buffer = TextBuffer::from_text(rust_fixture());
    let window = HighlightWindow::new(0, 24);
    c.bench_function("highlight_rust_window", |b| {
        b.iter(|| {
            let snapshot = registry
                .highlight_buffer_for_language_window("rust", black_box(&buffer), window)
                .expect("highlight rust window");
            black_box(snapshot.highlight_count());
        });
    });
}

criterion_group!(benches, bench_highlight_rust, bench_highlight_rust_window);
criterion_main!(benches);
