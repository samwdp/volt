use super::*;

#[test]
fn results_table_row_colors_header_and_numeric_cells() {
    let lines = vec![
        "SQLite result".to_owned(),
        String::new(),
        "┌────┬───────┐".to_owned(),
        "│ id │ name  │".to_owned(),
        "├────┼───────┤".to_owned(),
        "│  1 │ Ada   │".to_owned(),
        "│ 12 │ NULL  │".to_owned(),
        "└────┴───────┘".to_owned(),
        String::new(),
        "2 rows  ·  2 columns".to_owned(),
    ];
    let syntax = db_results_syntax_lines(&lines, false);
    assert!(syntax.get(&0).is_some_and(|spans| {
        spans
            .iter()
            .any(|span| span.theme_token.as_ref() == TOKEN_DB_RESULTS_TITLE)
    }));
    assert!(syntax.get(&3).is_some_and(|spans| {
        spans
            .iter()
            .any(|span| span.theme_token.as_ref() == TOKEN_DB_RESULTS_HEADER)
    }));
    assert!(syntax.get(&5).is_some_and(|spans| {
        spans
            .iter()
            .any(|span| span.theme_token.as_ref() == TOKEN_DB_RESULTS_NUMBER)
    }));
    assert!(syntax.get(&6).is_some_and(|spans| {
        spans
            .iter()
            .any(|span| span.theme_token.as_ref() == TOKEN_DB_RESULTS_NULL)
    }));
    assert!(syntax.get(&9).is_some_and(|spans| {
        spans
            .iter()
            .any(|span| span.theme_token.as_ref() == TOKEN_DB_RESULTS_FOOTER)
    }));
}

#[test]
fn browser_header_and_table_lines_use_distinct_tokens() {
    let header = db_browser_line_spans("  Tables (2)", DbBrowserItemKind::Header);
    let table = db_browser_line_spans("    users", DbBrowserItemKind::Table);
    assert!(
        header
            .iter()
            .any(|span| span.theme_token.as_ref() == TOKEN_DB_BROWSER_HEADER)
    );
    assert!(
        table
            .iter()
            .any(|span| span.theme_token.as_ref() == TOKEN_DB_BROWSER_TABLE)
    );
}
