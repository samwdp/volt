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
    let mut state = BrowserBufferState::default();
    {
        let tab = state.active_tab_mut();
        tab.current_url = Some("https://volt.test/current".to_owned());
        tab.requested_url = Some("https://volt.test/requested".to_owned());
    }
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

#[test]
fn add_tab_switches_active_tab_and_keeps_prior() {
    let mut state = BrowserBufferState::default();
    {
        let tab = state.active_tab_mut();
        tab.current_url = Some("https://first.example".to_owned());
        tab.requested_url = Some("https://first.example".to_owned());
        tab.page_title = Some("First".to_owned());
    }
    let second = state.add_tab(Some("https://second.example"));
    assert_eq!(state.active_tab_id, second);
    assert_eq!(state.tabs.len(), 2);
    assert_eq!(
        state.active_tab().requested_url.as_deref(),
        Some("https://second.example")
    );
    assert!(state.activate_tab(BrowserTabId(1)));
    assert_eq!(state.active_tab().page_title.as_deref(), Some("First"));
}

#[test]
fn close_tab_keeps_sibling_and_rejects_last() {
    let mut state = BrowserBufferState::default();
    let second = state.add_tab(Some("https://second.example"));
    assert!(state.close_tab(second));
    assert_eq!(state.tabs.len(), 1);
    assert_eq!(state.active_tab_id, BrowserTabId(1));
    assert!(!state.close_tab(BrowserTabId(1)));
    assert_eq!(state.tabs.len(), 1);
}
