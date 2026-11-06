use super::*;

#[test]
fn locals_section_is_adapter_locals_only() {
    let lines = dap_locals_section_lines(&[]);
    assert_eq!(lines, ["No locals."]);
}

#[test]
fn extract_watch_expressions_reads_expression_section_lines() {
    let lines = vec![
        "  person: Person { ... }  Person".to_owned(),
        "len".to_owned(),
        EMPTY_WATCHES.to_owned(),
    ];
    assert_eq!(extract_watch_expressions(&lines), ["person", "len"]);
}

#[test]
fn locals_line_kind_maps_local_rows_only() {
    assert_eq!(locals_line_variable_kind(0, 1), Some(0));
    assert_eq!(locals_line_variable_kind(1, 1), None);
    assert_eq!(locals_line_variable_kind(0, 0), None);
}

#[test]
fn expression_draft_uses_watch_tokens() {
    let lines = vec!["len".to_owned(), EMPTY_WATCHES.to_owned()];
    let syntax = dap_variable_syntax_lines(&lines, true);
    assert_eq!(
        syntax.get(&0).map(|spans| spans[0].theme_token.as_ref()),
        Some(TOKEN_DEBUG_VARIABLE_WATCH)
    );
    assert_eq!(
        syntax.get(&1).map(|spans| spans[0].theme_token.as_ref()),
        Some(TOKEN_DEBUG_VARIABLE_EMPTY)
    );
}
