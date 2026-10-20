//! Compile every bundled query kind against each installed registered grammar.
//!
//! Skips languages whose DLL is missing from the default grammar install root.
//! Failures print as `language/query_kind: message` so scm fixes stay easy to find.

use std::path::PathBuf;

use editor_buffer::TextBuffer;
use editor_syntax::{SyntaxError, SyntaxRegistry};

fn query_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("grammars")
        .join("queries")
}

fn is_missing_grammar(error: &SyntaxError) -> bool {
    matches!(error, SyntaxError::GrammarNotInstalled { .. })
}

#[test]
fn all_installed_registered_language_queries_compile() {
    let mut registry = SyntaxRegistry::new();
    registry.set_query_asset_root(Some(query_asset_root()));
    registry
        .register_all(user::syntax_languages())
        .expect("register user syntax languages");

    let mut failures = Vec::new();
    let mut checked = 0usize;
    let mut skipped = 0usize;
    let stub = TextBuffer::from_text("\n");

    let language_ids: Vec<String> = user::syntax_languages()
        .into_iter()
        .map(|language| language.id().to_owned())
        .collect();

    for language_id in language_ids {
        match registry.preload_language(&language_id) {
            Ok(()) => {}
            Err(error) if is_missing_grammar(&error) => {
                if std::env::var_os("VOLT_INSTALL_MISSING_GRAMMARS").is_some() {
                    eprintln!("INSTALL {language_id}…");
                    if let Err(install_error) = registry.install_language(&language_id) {
                        failures.push(format!("{language_id}/install: {install_error}"));
                        continue;
                    }
                    if let Err(error) = registry.preload_language(&language_id) {
                        failures.push(format!("{language_id}/highlight: {error}"));
                        continue;
                    }
                } else {
                    skipped += 1;
                    eprintln!("SKIP {language_id}: grammar not installed");
                    continue;
                }
            }
            Err(error) => {
                failures.push(format!("{language_id}/highlight: {error}"));
                continue;
            }
        }
        checked += 1;

        if let Err(error) = registry.highlight_buffer_for_language(&language_id, &stub) {
            failures.push(format!("{language_id}/highlight-run: {error}"));
        }
        if let Err(error) = registry.desired_indent_for_language(&language_id, &stub, 0, 4) {
            failures.push(format!("{language_id}/indents: {error}"));
        }
        if let Err(error) = registry.locals_query_for_language(&language_id) {
            failures.push(format!("{language_id}/locals: {error}"));
        }
        if let Err(error) = registry.folds_query_for_language(&language_id) {
            failures.push(format!("{language_id}/folds: {error}"));
        }
        if let Err(error) = registry.injections_query_for_language(&language_id) {
            failures.push(format!("{language_id}/injections: {error}"));
        }
    }

    eprintln!(
        "checked={checked} skipped={skipped} failures={}",
        failures.len()
    );
    if !failures.is_empty() {
        panic!(
            "query compile failures ({}):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
    assert!(
        checked > 0,
        "expected at least one installed grammar under {:?}",
        registry.install_root()
    );
}
