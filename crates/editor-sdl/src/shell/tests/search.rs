#![allow(unused_imports)]
use super::*;

#[test]
fn parse_rg_workspace_search_line_extracts_location() {
    let parsed = parse_rg_workspace_search_line(r"src\main.rs:12:7:let answer = compute();")
        .expect("rg output should parse into a workspace search match");
    assert_eq!(parsed.0, r"src\main.rs");
    assert_eq!(parsed.1, 12);
    assert_eq!(parsed.2, 7);
    assert_eq!(parsed.3, "let answer = compute();");
}

#[test]
fn parse_grep_workspace_search_line_finds_case_insensitive_column() {
    let parsed = parse_grep_workspace_search_line(r"src\lib.rs:3:Hello Workspace", "workspace")
        .expect("grep output should parse into a workspace search match");
    assert_eq!(parsed.0, r"src\lib.rs");
    assert_eq!(parsed.1, 3);
    assert_eq!(parsed.2, 7);
    assert_eq!(parsed.3, "Hello Workspace");
}

#[test]
fn workspace_search_char_column_handles_utf8_offsets() {
    assert_eq!(workspace_search_char_column("aébc", 0), 0);
    assert_eq!(workspace_search_char_column("aébc", 1), 1);
    assert_eq!(workspace_search_char_column("aébc", 3), 2);
}

#[test]
fn collect_search_output_stops_after_limit() {
    let (output, reached_limit) =
        collect_search_output(std::io::Cursor::new("one\ntwo\nthree\n"), 2)
            .expect("search output should be collected");
    assert_eq!(output, "one\ntwo\n");
    assert!(reached_limit);
}

#[test]
fn workspace_search_provider_extras_copy_ctrl_q_onto_instance() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-search-ctrl-q-extra");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "workspace-search-ctrl-q-extra", &root)?;

    let overlay = picker::picker_overlay(&state.runtime, "workspace.search")?;
    assert!(
        overlay.extra_keybinds().iter().any(|binding| {
            binding.chord() == "Ctrl+q" && binding.command_name() == "quickfix.open"
        }),
        "workspace.search provider extras should land on the open picker instance"
    );
    Ok(())
}
