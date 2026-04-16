//! Corpus inventory regression tests.
//!
//! These tests scan every bundled `.scm` query file under
//! `crates/volt/assets/grammars/queries` and assert the exact set of predicate
//! operators used in active (non-commented) patterns.  Keeping this list locked
//! means any grammar update that silently introduces a new operator—especially a
//! general predicate the runtime must handle—will cause this test to fail and
//! force an explicit review.
//!
//! To update the expected set after an intentional corpus change, update the
//! constant at the top of each test and leave a note in the commit message.
#![allow(unused_crate_dependencies)]
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

fn query_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("volt")
        .join("assets")
        .join("grammars")
        .join("queries")
}

/// Returns every predicate operator (`#name?` or `#name!`) found in
/// non-commented lines across all `.scm` files under `root`.
fn collect_operators(root: &Path) -> BTreeSet<String> {
    let mut operators = BTreeSet::new();
    visit_scm_files(root, &mut |_, content| {
        for line in content.lines() {
            let trimmed = line.trim_start();
            // Skip full-line comments (`;`) and empty lines.
            if trimmed.starts_with(';') || trimmed.is_empty() {
                continue;
            }
            // Find every `(#name` occurrence on this line.
            let mut search = trimmed;
            while let Some(pos) = search.find("(#") {
                let after = &search[pos + 2..];
                // Operator ends at the first whitespace or `)`.
                let end = after
                    .find(|c: char| c.is_whitespace() || c == ')')
                    .unwrap_or(after.len());
                let candidate = &after[..end];
                if !candidate.is_empty()
                    && (candidate.ends_with('?') || candidate.ends_with('!'))
                    && candidate.chars().all(|c| {
                        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '!'
                    })
                {
                    operators.insert(format!("#{candidate}"));
                }
                search = &search[pos + 2 + end..];
            }
        }
    });
    operators
}

/// Walk `root` recursively, calling `visitor` with the text of each `.scm` file.
fn visit_scm_files(root: &Path, visitor: &mut impl FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            visit_scm_files(&path, visitor);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("scm")
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            visitor(&path, &content);
        }
    }
}

fn skip_query_string(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            b'"' => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn skip_query_comment(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    while index < bytes.len() && bytes[index] != b'\n' {
        index += 1;
    }
    index
}

fn find_matching_paren(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            b';' => index = skip_query_comment(bytes, index),
            b'"' => index = skip_query_string(bytes, index),
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                depth = depth.checked_sub(1)?;
                index += 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => index += 1,
        }
    }
    None
}

fn query_atoms(query: &str) -> Vec<String> {
    let bytes = query.as_bytes();
    let mut atoms = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'(' | b')' => index += 1,
            b';' => index = skip_query_comment(bytes, index),
            b'"' => {
                let end = skip_query_string(bytes, index);
                atoms.push(query[index..end].to_string());
                index = end;
            }
            byte if byte.is_ascii_whitespace() => index += 1,
            _ => {
                let start = index;
                while index < bytes.len()
                    && !bytes[index].is_ascii_whitespace()
                    && !matches!(bytes[index], b'(' | b')' | b';')
                {
                    index += 1;
                }
                atoms.push(query[start..index].to_string());
            }
        }
    }
    atoms
}

fn collect_invalid_capture_valued_set_directives(root: &Path) -> Vec<String> {
    let mut offenders = Vec::new();
    visit_scm_files(root, &mut |path, content| {
        let bytes = content.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            match bytes[index] {
                b';' => index = skip_query_comment(bytes, index),
                b'"' => index = skip_query_string(bytes, index),
                b'(' if bytes[index..].starts_with(b"(#set!") => {
                    let end = find_matching_paren(bytes, index).unwrap_or(bytes.len());
                    let directive = &content[index..end];
                    let atoms = query_atoms(directive);
                    let value_index = if atoms.get(1).is_some_and(|atom| atom.starts_with('@')) {
                        3
                    } else {
                        2
                    };
                    if atoms
                        .get(value_index)
                        .is_some_and(|value| value.starts_with('@'))
                    {
                        let line = content[..index]
                            .bytes()
                            .filter(|byte| *byte == b'\n')
                            .count()
                            + 1;
                        offenders.push(format!("{}:{line}: {}", path.display(), directive.trim()));
                    }
                    index = end;
                }
                _ => index += 1,
            }
        }
    });
    offenders
}

/// The full set of predicate operators active (non-commented) in the bundled
/// query corpus.  Update this when the corpus intentionally gains new operators.
///
/// NOTE: this inventory is intentionally broader than Volt's custom predicate
/// layer. Some operators are handled directly by tree-sitter, while Volt
/// additionally evaluates corpus-specific general predicates in
/// `evaluate_general_predicate` inside `editor-syntax`. Keep this list in sync
/// with both the bundled corpus and the runtime support table.
const EXPECTED_OPERATORS: &[&str] = &[
    "#any-of?",
    "#contains?",
    "#eq?",
    "#gsub!",
    "#has-ancestor?",
    "#has-parent?",
    "#lua-match?",
    "#match?",
    "#not-any-of?",
    "#not-eq?",
    "#not-has-ancestor?",
    "#not-has-parent?",
    "#not-kind-eq?",
    "#not-lua-match?",
    "#not-match?",
    "#offset!",
    "#set!",
    "#trim!",
];

#[test]
fn corpus_predicate_operator_inventory_is_stable() {
    let root = query_asset_root();
    assert!(
        root.is_dir(),
        "bundled query asset root not found: {}",
        root.display()
    );

    let found = collect_operators(&root);
    let expected: BTreeSet<String> = EXPECTED_OPERATORS.iter().map(|s| s.to_string()).collect();

    let new_operators: Vec<_> = found.difference(&expected).collect();
    let removed_operators: Vec<_> = expected.difference(&found).collect();

    assert!(
        new_operators.is_empty(),
        "corpus uses operators not in EXPECTED_OPERATORS — review whether the \
         runtime handles them and update the constant:\n  added: {new_operators:?}"
    );
    assert!(
        removed_operators.is_empty(),
        "operators no longer appear in the corpus — remove them from \
         EXPECTED_OPERATORS:\n  removed: {removed_operators:?}"
    );
}

/// The `#set!` property keys actively used in the corpus.  Tracked so that new
/// property keys (especially any that affect runtime behaviour) are visible.
///
/// The keys include both simple-form `(#set! key value)` and capture-prefixed
/// `(#set! @capture key value)` patterns.
const EXPECTED_SET_KEYS: &[&str] = &[
    // Neovim/editor comment string (JSX, Vue)
    "bo.commentstring",
    // concealment
    "conceal",
    "conceal_lines",
    // node priority
    "priority",
    // indentation properties (see `desired_indent_for_language`)
    "indent.avoid_last_matching_next",
    "indent.close_delimiter",
    "indent.immediate",
    "indent.open_delimiter",
    "indent.start_at_same_line",
    // injection properties
    "injection.combined",
    "injection.include-children",
    "injection.language",
    "injection.parent",
    "injection.self",
    // locals / symbol-lookup properties
    "definition.constant.scope",
    "definition.function.scope",
    "definition.import.scope",
    "definition.macro.scope",
    "definition.method.scope",
    "definition.namespace.scope",
    "definition.type.scope",
    "definition.var.scope",
    "reference.kind",
];

#[test]
fn corpus_set_key_inventory_is_stable() {
    let root = query_asset_root();
    assert!(
        root.is_dir(),
        "bundled query asset root not found: {}",
        root.display()
    );

    let mut found_keys: BTreeSet<String> = BTreeSet::new();
    visit_scm_files(&root, &mut |_, content| {
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with(';') || trimmed.is_empty() {
                continue;
            }
            // Extract the key from `(#set! key ...)` and `(#set! @capture key ...)`.
            // The key is the first non-`@` argument that looks like an identifier.
            let mut search = trimmed;
            while let Some(pos) = search.find("(#set!") {
                let mut after = search[pos + 6..].trim_start();
                // Skip an optional leading `@capture` argument.
                if after.starts_with('@') {
                    let end = after
                        .find(|c: char| c.is_whitespace())
                        .unwrap_or(after.len());
                    after = after[end..].trim_start();
                }
                // Strip optional surrounding quotes.
                let unquoted = after.trim_start_matches('"');
                let end = unquoted
                    .find(|c: char| c.is_whitespace() || c == ')' || c == '"')
                    .unwrap_or(unquoted.len());
                let key = &unquoted[..end];
                // Only record structured keys (containing `.`) or named single-word keys
                // to avoid false positives from injection language name values.
                if !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
                    && (key.contains('.')
                        || matches!(key, "conceal" | "conceal_lines" | "priority" | "url"))
                {
                    found_keys.insert(key.to_string());
                }
                search = &search[pos + 6..];
            }
        }
    });

    let expected: BTreeSet<String> = EXPECTED_SET_KEYS.iter().map(|s| s.to_string()).collect();
    let new_keys: Vec<_> = found_keys.difference(&expected).collect();
    let removed_keys: Vec<_> = expected.difference(&found_keys).collect();

    assert!(
        new_keys.is_empty(),
        "corpus uses #set! keys not in EXPECTED_SET_KEYS — review whether the \
         runtime handles them and update the constant:\n  added: {new_keys:?}"
    );
    assert!(
        removed_keys.is_empty(),
        "some expected #set! keys no longer appear in the corpus — remove them \
         from EXPECTED_SET_KEYS:\n  removed: {removed_keys:?}"
    );
}

/// Tree-sitter only accepts string values for `#set!`; a capture in the value
/// position (for example `(#set! @_hyperlink url @markup.link.url)`) is rejected
/// when the query is compiled. Keep this as a static corpus scan so it still
/// protects grammars that are not installed on the current machine.
#[test]
fn corpus_set_directives_do_not_use_capture_values() {
    let root = query_asset_root();
    assert!(
        root.is_dir(),
        "bundled query asset root not found: {}",
        root.display()
    );

    let offenders = collect_invalid_capture_valued_set_directives(&root);
    assert!(
        offenders.is_empty(),
        "bundled query assets contain capture-valued #set! directives, which \
         tree-sitter rejects at query-parse time:\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn corpus_query_asset_root_contains_expected_languages() {
    let root = query_asset_root();
    assert!(
        root.is_dir(),
        "bundled query asset root not found: {}",
        root.display()
    );

    // A subset of language directories that must always be present in the corpus.
    const REQUIRED_LANGUAGES: &[&str] = &[
        "rust",
        "markdown",
        "markdown-inline",
        "javascript",
        "typescript",
        "python",
        "json",
        "yaml",
        "toml",
    ];

    for language in REQUIRED_LANGUAGES {
        let lang_dir = root.join(language);
        assert!(
            lang_dir.is_dir(),
            "expected language directory missing from corpus: {}",
            lang_dir.display()
        );
        let highlights = lang_dir.join("highlights.scm");
        assert!(
            highlights.exists(),
            "highlights.scm missing for language `{language}`: {}",
            highlights.display()
        );
    }
}
