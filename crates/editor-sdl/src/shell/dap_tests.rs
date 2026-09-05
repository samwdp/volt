
use super::*;

#[test]
fn locals_section_always_keeps_watch_header() {
    let lines = dap_locals_section_lines(&[], &[], &[], true);
    assert_eq!(lines, ["No locals", DAP_WATCHES_HEADER]);
}

#[test]
fn extract_watch_expressions_reads_lines_after_header() {
    let lines = vec![
        "  x: 42  i32".to_owned(),
        DAP_WATCHES_HEADER.to_owned(),
        "  person: Person { ... }  Person".to_owned(),
        "len".to_owned(),
    ];
    assert_eq!(extract_watch_expressions(&lines), ["person", "len"]);
}

#[test]
fn locals_line_kind_skips_header_and_placeholder() {
    let lines = vec![
        "  x: 42  i32".to_owned(),
        DAP_WATCHES_HEADER.to_owned(),
        "  person: Person { ... }  Person".to_owned(),
    ];
    assert_eq!(
        locals_line_variable_kind(&lines, 0, 1, 1),
        Some(DapLocalsLineTarget::Local(0))
    );
    assert_eq!(locals_line_variable_kind(&lines, 1, 1, 1), None);
    assert_eq!(
        locals_line_variable_kind(&lines, 2, 1, 1),
        Some(DapLocalsLineTarget::Watch(0))
    );
    assert_eq!(
        locals_line_variable_kind(
            &["No locals".to_owned(), DAP_WATCHES_HEADER.to_owned()],
            0,
            0,
            0
        ),
        None
    );
}

#[test]
fn watch_header_and_draft_use_watch_tokens() {
    let lines = vec![
        "No locals".to_owned(),
        DAP_WATCHES_HEADER.to_owned(),
        "len".to_owned(),
    ];
    let syntax = dap_variable_syntax_lines(&lines, false);
    assert_eq!(
        syntax.get(&0).map(|spans| spans[0].theme_token.as_ref()),
        Some(TOKEN_DEBUG_VARIABLE_EMPTY)
    );
    assert_eq!(
        syntax.get(&1).map(|spans| spans[0].theme_token.as_ref()),
        Some(TOKEN_DEBUG_VARIABLE_HEADER)
    );
    assert_eq!(
        syntax.get(&2).map(|spans| spans[0].theme_token.as_ref()),
        Some(TOKEN_DEBUG_VARIABLE_WATCH)
    );
}
