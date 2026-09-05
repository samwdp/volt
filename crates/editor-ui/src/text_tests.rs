use super::*;

#[test]
fn truncate_start_appends_ellipsis() {
    assert_eq!(truncate_text_to_width("abcdef", 5, 1), "ab...");
}

#[test]
fn truncate_end_keeps_tail() {
    assert_eq!(
        truncate_text_to_width_preserving_end("abcdef", 5, 1),
        "...ef"
    );
}

#[test]
fn truncate_is_identity_when_text_fits() {
    assert_eq!(truncate_text_to_width("abc", 10, 1), "abc");
    assert_eq!(truncate_text_to_width_preserving_end("abc", 10, 1), "abc");
}
