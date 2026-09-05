use super::*;

#[test]
fn browser_buffer_display_name_prefers_title() {
    assert_eq!(
        browser_buffer_display_name(Some("Volt Docs"), Some("https://example.com"), false),
        "*browser* Volt Docs"
    );
}

#[test]
fn browser_buffer_display_name_marks_loading_state() {
    assert_eq!(
        browser_buffer_display_name(None, Some("https://example.com"), true),
        "*browser* [loading] https://example.com"
    );
}

#[test]
fn browser_display_url_prefers_requested_navigation() {
    let state = BrowserBufferState {
        current_url: Some("https://volt.test/current".to_owned()),
        requested_url: Some("https://volt.test/requested".to_owned()),
        ..BrowserBufferState::default()
    };
    assert_eq!(
        browser_display_url(&state),
        Some("https://volt.test/requested")
    );
}

#[test]
fn path_to_file_url_encodes_spaces() {
    let path = std::path::Path::new(r"C:\volt docs\page.html");
    assert_eq!(path_to_file_url(path), "file:///C:/volt%20docs/page.html");
}
