use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    ShellConfig,
    shell::{
        ShellState, delete_runtime_workspace, open_workspace_from_project,
        switch_runtime_workspace, workspace_delete_picker_overlay, workspace_switch_picker_overlay,
    },
    state::{BlockSelection, InputMode, VisualSelection, VisualSelectionKind},
};
use editor_buffer::{TextBuffer, TextPoint};
use editor_core::{BufferKind, HookEvent};
use editor_render::RenderBackend;
use sdl3::keyboard::{Keycode, Mod};

fn temp_workspace_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("volt-workspace-{name}-{unique}"))
}

fn git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

fn run_git(root: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("git").args(args).current_dir(root).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {:?} failed with status {status}", args).into())
    }
}

fn set_active_buffer_text(
    state: &mut ShellState,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    state.replace_active_buffer_text_for_test(text)?;
    Ok(())
}

fn flush_picker_searches(state: &mut ShellState) -> Result<(), Box<dyn std::error::Error>> {
    state.flush_picker_searches_for_test()?;
    Ok(())
}

fn user_shell_state() -> Result<ShellState, Box<dyn std::error::Error>> {
    Ok(ShellState::new_with_user_library(
        temp_workspace_root("user-shell").join("shell.log"),
        false,
        Arc::new(user::UserLibraryImpl),
    )?)
}

#[test]
fn vim_bindings_switch_modes_and_move_words() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    set_active_buffer_text(&mut state, "alpha beta")?;

    state.handle_text_input("w")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 6);

    state.handle_text_input("i")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);

    state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?;
    assert_eq!(state.input_mode()?, InputMode::Normal);
    Ok(())
}

#[test]
fn vim_extended_motions_and_edit_commands_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    set_active_buffer_text(&mut state, "alpha beta")?;

    state.handle_text_input("w")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 6);

    state.handle_text_input("b")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    state.handle_text_input("e")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 4);

    state.handle_text_input("$")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 9);

    state.handle_text_input("0")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    state.handle_text_input("A")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);
    state.handle_text_input("!")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    assert_eq!(state.input_mode()?, InputMode::Normal);
    assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta!");

    state.handle_text_input("u")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta");

    assert!(state.try_runtime_keybinding(Keycode::R, Mod::LCTRLMOD)?);
    assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta!");

    Ok(())
}

#[test]
fn vim_command_line_opens_and_tab_completes_commands() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = user_shell_state()?;

    state.runtime.execute_command("vim.command-line")?;
    assert!(state.command_line_visible()?);

    state.handle_text_input("picker.open-bu")?;
    assert_eq!(
        state.command_line_text()?,
        Some("picker.open-bu".to_owned())
    );

    assert!(state.try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)?);
    assert_eq!(
        state.command_line_text()?,
        Some("picker.open-buffers".to_owned())
    );
    Ok(())
}

#[test]
fn vim_command_line_runs_shell_commands_and_substitutions() -> Result<(), Box<dyn std::error::Error>>
{
    let mut state = user_shell_state()?;
    set_active_buffer_text(&mut state, "alpha beta\nalpha")?;

    state.runtime.execute_command("vim.command-line")?;
    state.handle_text_input("%s/alpha/omega/g")?;
    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    assert_eq!(state.active_buffer_mut()?.text.text(), "omega beta\nomega");

    state.runtime.execute_command("vim.command-line")?;
    state.handle_text_input("!echo volt-command-line")?;
    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    let active = state.active_buffer_mut()?;
    assert!(active.display_name().contains("*command "));
    assert!(active.text.text().contains("volt-command-line"));
    Ok(())
}

#[test]
fn vim_counts_operators_and_text_objects_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    set_active_buffer_text(&mut state, "alpha beta gamma")?;

    state.handle_text_input("d")?;
    state.handle_text_input("w")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "beta gamma");

    state.handle_text_input("u")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta gamma");

    state.handle_text_input("c")?;
    state.handle_text_input("i")?;
    state.handle_text_input("w")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);
    state.handle_text_input("delta")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    assert_eq!(state.active_buffer_mut()?.text.text(), "delta beta gamma");

    state.handle_text_input("y")?;
    state.handle_text_input("y")?;
    state.handle_text_input("p")?;
    assert_eq!(
        state.active_buffer_mut()?.text.text(),
        "delta beta gamma\ndelta beta gamma\n"
    );

    Ok(())
}

#[test]
fn vim_visual_mode_and_find_repeats_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("bananas");

    state.handle_text_input("f")?;
    state.handle_text_input("a")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 1);

    state.handle_text_input(";")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 3);

    state.handle_text_input(",")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 1);

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
    state.handle_text_input("v")?;
    assert_eq!(state.input_mode()?, InputMode::Visual);
    state.handle_text_input("e")?;
    state.handle_text_input("d")?;
    assert_eq!(state.input_mode()?, InputMode::Normal);
    assert_eq!(state.active_buffer_mut()?.text.text(), " beta");

    Ok(())
}

#[test]
fn vim_linewise_visual_mode_and_anchor_swap_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree\n");

    state.handle_text_input("V")?;
    assert_eq!(state.input_mode()?, InputMode::Visual);
    assert_eq!(state.ui()?.vim().visual_kind, VisualSelectionKind::Line);

    state.handle_text_input("j")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 1);

    state.handle_text_input("o")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);
    assert_eq!(state.ui()?.vim().visual_anchor, Some(TextPoint::new(1, 0)));

    state.handle_text_input("d")?;
    assert_eq!(state.input_mode()?, InputMode::Normal);
    assert_eq!(state.active_buffer_mut()?.text.text(), "three\n");

    Ok(())
}

#[test]
fn vim_visual_block_insert_applies_to_all_lines() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));

    state.handle_text_input("v")?;
    assert_eq!(state.input_mode()?, InputMode::Visual);
    assert!(state.try_runtime_keybinding(Keycode::V, Mod::LCTRLMOD)?);
    assert_eq!(state.ui()?.vim().visual_kind, VisualSelectionKind::Block);

    state.handle_text_input("j")?;
    state.handle_text_input("j")?;
    state.handle_text_input("I")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);
    state.handle_text_input("x")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    assert_eq!(state.input_mode()?, InputMode::Normal);
    assert_eq!(state.active_buffer_mut()?.text.text(), "xone\nxtwo\nxthree");
    Ok(())
}

#[test]
fn vim_visual_text_objects_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");

    state.handle_text_input("v")?;
    state.handle_text_input("i")?;
    state.handle_text_input("w")?;
    state.handle_text_input("d")?;
    assert_eq!(state.input_mode()?, InputMode::Normal);
    assert_eq!(state.active_buffer_mut()?.text.text(), " beta");

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));

    state.handle_text_input("v")?;
    state.handle_text_input("a")?;
    state.handle_text_input("w")?;
    state.handle_text_input("d")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "beta");

    Ok(())
}

#[test]
fn vim_multicursor_ctrl_b_adds_next_exact_match() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = user_shell_state()?;
    set_active_buffer_text(
        &mut state,
        "Volt is a new editor. Volt does things like Emacs + vim",
    )?;
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));

    assert!(state.try_runtime_keybinding(Keycode::B, Mod::LCTRLMOD)?);
    let mc = state
        .ui()?
        .vim()
        .multicursor
        .clone()
        .ok_or("missing multicursor state")?;
    assert_eq!(mc.match_text, "Volt");
    assert_eq!(mc.ranges.len(), 1);
    assert_eq!(state.input_mode()?, InputMode::Normal);

    assert!(state.try_runtime_keybinding(Keycode::B, Mod::LCTRLMOD)?);
    let mc = state
        .ui()?
        .vim()
        .multicursor
        .clone()
        .ok_or("missing multicursor state")?;
    assert_eq!(mc.ranges.len(), 2);
    assert_eq!(mc.cursor_offset, 0);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 22);
    Ok(())
}

#[test]
fn vim_multicursor_caw_changes_all_matches() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = user_shell_state()?;
    set_active_buffer_text(
        &mut state,
        "Volt is a new editor. Volt does things like Emacs + vim",
    )?;
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));

    assert!(state.try_runtime_keybinding(Keycode::B, Mod::LCTRLMOD)?);
    assert!(state.try_runtime_keybinding(Keycode::B, Mod::LCTRLMOD)?);

    state.handle_text_input("c")?;
    state.handle_text_input("a")?;
    state.handle_text_input("w")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);

    state.handle_text_input("Volt2")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);

    assert_eq!(state.input_mode()?, InputMode::Normal);
    assert!(state.ui()?.vim().multicursor.is_none());
    assert_eq!(
        state.active_buffer_mut()?.text.text(),
        "Volt2 is a new editor. Volt2 does things like Emacs + vim"
    );
    Ok(())
}

#[test]
fn vim_multicursor_motions_move_linked_cursor_offsets() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = user_shell_state()?;
    set_active_buffer_text(
        &mut state,
        "Volt is a new editor. Volt does things like Emacs + vim",
    )?;
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));

    assert!(state.try_runtime_keybinding(Keycode::B, Mod::LCTRLMOD)?);
    assert!(state.try_runtime_keybinding(Keycode::B, Mod::LCTRLMOD)?);

    state.handle_text_input("l")?;
    let mc = state
        .ui()?
        .vim()
        .multicursor
        .clone()
        .ok_or("missing multicursor state")?;
    assert_eq!(mc.cursor_offset, 1);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 23);
    Ok(())
}

#[test]
fn vim_counted_line_end_and_aliases_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree");

    state.handle_text_input("2")?;
    state.handle_text_input("$")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 1);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 2);

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta gamma");
    state.handle_text_input("w")?;
    state.handle_text_input("D")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "alpha ");

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta gamma");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("w")?;
    state.handle_text_input("C")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);
    state.handle_text_input("delta")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    assert_eq!(state.active_buffer_mut()?.text.text(), "alpha delta");

    state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree\n");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("2")?;
    state.handle_text_input("Y")?;
    state.handle_text_input("p")?;
    assert_eq!(
        state.active_buffer_mut()?.text.text(),
        "one\none\ntwo\ntwo\nthree\n"
    );

    Ok(())
}

#[test]
fn vim_substitute_delete_counts_and_visual_toggles_work() -> Result<(), Box<dyn std::error::Error>>
{
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");

    state.handle_text_input("2")?;
    state.handle_text_input("x")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "pha beta");

    state.handle_text_input("2")?;
    state.handle_text_input("s")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);
    state.handle_text_input("Z")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    assert_eq!(state.active_buffer_mut()?.text.text(), "Za beta");

    state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\nthree\n");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 1));
    state.handle_text_input("V")?;
    assert_eq!(state.ui()?.vim().visual_kind, VisualSelectionKind::Line);
    state.handle_text_input("v")?;
    assert_eq!(state.input_mode()?, InputMode::Visual);
    assert_eq!(
        state.ui()?.vim().visual_kind,
        VisualSelectionKind::Character
    );
    state.handle_text_input("v")?;
    assert_eq!(state.input_mode()?, InputMode::Normal);

    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("2")?;
    state.handle_text_input("S")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);
    state.handle_text_input("hello")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    assert_eq!(state.active_buffer_mut()?.text.text(), "hello\nthree\n");

    Ok(())
}

#[test]
fn vim_search_prompt_and_repeats_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("one two one two");

    state.handle_text_input("/")?;
    assert!(state.picker_visible()?);
    state.handle_text_input("one")?;
    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    assert!(!state.picker_visible()?);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 8);

    state.handle_text_input("n")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    state.handle_text_input("N")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 8);

    state.handle_text_input("?")?;
    state.handle_text_input("two")?;
    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 4);

    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("*")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 8);

    state.handle_text_input("#")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    Ok(())
}

#[test]
fn vim_search_prompt_supports_fuzzy_matches() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("one two one two");

    state.handle_text_input("/")?;
    state.handle_text_input("otw")?;
    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 8);

    Ok(())
}

#[test]
fn autocomplete_trigger_updates_and_accepts_buffer_tokens() -> Result<(), Box<dyn std::error::Error>>
{
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpine alphabet alpha\nalp");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 3));

    state.handle_text_input("i")?;
    assert!(state.try_runtime_keybinding(Keycode::Space, Mod::LCTRLMOD)?);
    assert!(!state.autocomplete_visible()?);
    state.wait_for_autocomplete_results()?;
    assert!(state.autocomplete_visible()?);
    assert_eq!(
        state.autocomplete_entries()?,
        vec![
            "alpha".to_owned(),
            "alpine".to_owned(),
            "alphabet".to_owned()
        ]
    );
    assert_eq!(state.autocomplete_selected()?, Some("alpha".to_owned()));

    assert!(state.try_runtime_keybinding(Keycode::N, Mod::LCTRLMOD)?);
    assert_eq!(state.autocomplete_selected()?, Some("alpine".to_owned()));
    assert!(state.try_runtime_keybinding(Keycode::P, Mod::LCTRLMOD)?);
    assert_eq!(state.autocomplete_selected()?, Some("alpha".to_owned()));

    state.handle_text_input("h")?;
    assert!(!state.autocomplete_visible()?);
    state.wait_for_autocomplete_results()?;
    assert!(state.autocomplete_visible()?);
    assert_eq!(
        state.autocomplete_entries()?,
        vec!["alpha".to_owned(), "alphabet".to_owned()]
    );

    assert!(state.try_runtime_keybinding(Keycode::Y, Mod::LCTRLMOD)?);
    assert_eq!(
        state.active_buffer_mut()?.text.text(),
        "alpine alphabet alpha\nalpha"
    );
    assert!(!state.autocomplete_visible()?);

    Ok(())
}

#[test]
fn autocomplete_opens_while_typing_buffer_tokens() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpine alphabet alpha\nal");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 2));

    state.handle_text_input("i")?;
    state.handle_text_input("p")?;
    assert!(!state.autocomplete_visible()?);
    state.wait_for_autocomplete_results()?;
    assert!(state.autocomplete_visible()?);
    assert_eq!(
        state.autocomplete_entries()?,
        vec![
            "alpha".to_owned(),
            "alpine".to_owned(),
            "alphabet".to_owned()
        ]
    );

    state.handle_text_input(" ")?;
    assert!(!state.autocomplete_visible()?);

    Ok(())
}

#[test]
fn ctrl_space_triggers_autocomplete_without_inserting_space()
-> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpine alphabet alpha\nalp");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 3));

    state.handle_text_input("i")?;
    assert!(state.try_runtime_keybinding(Keycode::Space, Mod::LCTRLMOD)?);
    state.handle_text_input(" ")?;
    assert!(!state.autocomplete_visible()?);
    state.wait_for_autocomplete_results()?;

    assert!(state.autocomplete_visible()?);
    assert_eq!(
        state.autocomplete_entries()?,
        vec![
            "alpha".to_owned(),
            "alpine".to_owned(),
            "alphabet".to_owned()
        ]
    );
    assert_eq!(
        state.active_buffer_mut()?.text.text(),
        "alpine alphabet alpha\nalp"
    );

    Ok(())
}

#[test]
fn ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text()
-> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpine alphabet alpha\nalp");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 3));

    state.handle_text_input("i")?;
    assert!(state.try_runtime_keybinding(Keycode::Space, Mod::LCTRLMOD)?);
    state.handle_text_input(" ")?;
    state.wait_for_autocomplete_results()?;
    assert_eq!(state.autocomplete_selected()?, Some("alpha".to_owned()));

    assert!(state.try_runtime_keybinding(Keycode::N, Mod::LCTRLMOD)?);
    state.handle_text_input("n")?;
    assert_eq!(state.autocomplete_selected()?, Some("alpine".to_owned()));

    assert!(state.try_runtime_keybinding(Keycode::P, Mod::LCTRLMOD)?);
    state.handle_text_input("p")?;
    assert_eq!(state.autocomplete_selected()?, Some("alpha".to_owned()));
    assert_eq!(
        state.active_buffer_mut()?.text.text(),
        "alpine alphabet alpha\nalp"
    );

    Ok(())
}

#[test]
fn autocomplete_closes_when_no_results_remain() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta\ngh");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 2));

    state.handle_text_input("z")?;
    state.wait_for_autocomplete_results()?;

    assert!(!state.autocomplete_visible()?);
    Ok(())
}

#[test]
fn hover_toggle_previews_and_focus_enters_panel() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 1));

    state.handle_text_input("K")?;
    assert!(state.hover_visible()?);
    assert!(!state.hover_focused()?);
    assert_eq!(state.hover_provider_label()?, Some("Token".to_owned()));

    state
        .runtime
        .emit_hook(editor_plugin_api::hover_hooks::FOCUS, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(state.hover_focused()?);

    state
        .runtime
        .emit_hook(editor_plugin_api::hover_hooks::FOCUS, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(state.hover_visible()?);
    assert!(state.hover_focused()?);

    state.handle_text_input("K")?;
    assert!(!state.hover_visible()?);

    Ok(())
}

#[test]
fn hover_toggle_works_at_token_end() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 5));

    state.handle_text_input("K")?;

    assert!(state.hover_visible()?);
    assert_eq!(state.hover_provider_label()?, Some("Token".to_owned()));
    Ok(())
}

#[test]
fn hover_toggle_command_shows_feedback_without_symbol() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text(" alpha");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));

    state
        .runtime
        .execute_command("hover.toggle")
        .map_err(|error| error.to_string())?;

    assert!(state.hover_visible()?);
    assert_eq!(state.hover_provider_label()?, Some("Token".to_owned()));
    Ok(())
}

#[test]
fn hover_closes_after_cursor_motion() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 1));

    state.handle_text_input("K")?;
    assert!(state.hover_visible()?);

    state.handle_text_input("l")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 2);
    assert!(!state.hover_visible()?);

    Ok(())
}

#[test]
fn vim_search_prompt_matches_case_insensitive() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    set_active_buffer_text(&mut state, "Users user")?;

    state.handle_text_input("/")?;
    state.handle_text_input("user")?;
    flush_picker_searches(&mut state)?;
    let picker = state.ui()?.picker().ok_or("missing search picker")?;
    assert_eq!(picker.session().match_count(), 2);

    Ok(())
}

#[test]
fn vim_search_picker_selects_match_entries() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    set_active_buffer_text(&mut state, "alpha\nsplit here\nbeta\nsplit again\n")?;

    state.handle_text_input("/")?;
    state.handle_text_input("split")?;
    flush_picker_searches(&mut state)?;
    let picker = state.ui()?.picker().ok_or("missing search picker")?;
    assert!(picker.session().match_count() > 0);
    let selected = picker
        .session()
        .selected()
        .ok_or("missing search selection")?;
    let selected_id = selected.item().id();
    let (line, column) = selected_id
        .split_once(':')
        .ok_or("missing search id delimiter")?;
    let line: usize = line.parse()?;
    let column: usize = column.parse()?;

    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    assert_eq!(state.active_buffer_mut()?.cursor_row(), line);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), column);
    Ok(())
}

#[test]
fn vim_quickref_word_motions_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha-beta gamma");
    state.handle_text_input("W")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 11);

    state.handle_text_input("B")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    state.handle_text_input("E")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 9);

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta gamma");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 11));
    state.handle_text_input("g")?;
    state.handle_text_input("e")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 9);

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha-beta gamma");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 11));
    state.handle_text_input("g")?;
    state.handle_text_input("E")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 9);

    state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar])");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 4));
    state.handle_text_input("%")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 13);

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha-beta gamma");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("d")?;
    state.handle_text_input("W")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "gamma");

    state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar])");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 4));
    state.handle_text_input("d")?;
    state.handle_text_input("%")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "call");

    Ok(())
}

#[test]
fn vim_word_motions_treat_punctuation_as_words() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text =
        TextBuffer::from_text("PluginKeymapScope::Workspace,\n),\nnormal_binding");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 19));

    state.handle_text_input("w")?;
    assert_eq!(
        state.active_buffer_mut()?.cursor_point(),
        TextPoint::new(0, 28)
    );

    state.handle_text_input("w")?;
    assert_eq!(
        state.active_buffer_mut()?.cursor_point(),
        TextPoint::new(1, 0)
    );

    state.handle_text_input("w")?;
    assert_eq!(
        state.active_buffer_mut()?.cursor_point(),
        TextPoint::new(2, 0)
    );

    state.handle_text_input("b")?;
    assert_eq!(
        state.active_buffer_mut()?.cursor_point(),
        TextPoint::new(1, 0)
    );

    state.handle_text_input("b")?;
    assert_eq!(
        state.active_buffer_mut()?.cursor_point(),
        TextPoint::new(0, 28)
    );

    state.handle_text_input("b")?;
    assert_eq!(
        state.active_buffer_mut()?.cursor_point(),
        TextPoint::new(0, 19)
    );

    Ok(())
}

#[test]
fn vim_quickref_structure_motions_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;

    state.active_buffer_mut()?.text = TextBuffer::from_text("Alpha. Bravo! Charlie?");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input(")")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 7);

    state.handle_text_input("(")?;
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\n\nthree\nfour\n\nfive");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(3, 1));
    state.handle_text_input("{")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 2);

    state.handle_text_input("{")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    state.active_buffer_mut()?.set_cursor(TextPoint::new(3, 1));
    state.handle_text_input("{")?;
    state.handle_text_input("}")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 5);

    state.handle_text_input("}")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 6);

    state.active_buffer_mut()?.set_viewport_lines(4);
    state.active_buffer_mut()?.scroll_row = 3;
    state.handle_text_input("H")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 3);

    state.handle_text_input("M")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 4);

    state.handle_text_input("L")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 6);

    state.active_buffer_mut()?.scroll_row = 0;
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    assert!(state.try_runtime_keybinding(Keycode::D, Mod::LCTRLMOD)?);
    assert_eq!(state.active_buffer_mut()?.scroll_row, 2);
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 2);

    assert!(state.try_runtime_keybinding(Keycode::U, Mod::LCTRLMOD)?);
    assert_eq!(state.active_buffer_mut()?.scroll_row, 0);
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);

    assert!(state.try_runtime_keybinding(Keycode::F, Mod::LCTRLMOD)?);
    assert_eq!(state.active_buffer_mut()?.scroll_row, 4);
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 4);

    assert!(state.try_runtime_keybinding(Keycode::Y, Mod::LCTRLMOD)?);
    assert_eq!(state.active_buffer_mut()?.scroll_row, 3);
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 4);

    assert!(state.try_runtime_keybinding(Keycode::E, Mod::LCTRLMOD)?);
    assert_eq!(state.active_buffer_mut()?.scroll_row, 4);
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 4);

    Ok(())
}

#[test]
fn vim_quickref_change_ops_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha");
    state.handle_text_input("r")?;
    state.handle_text_input("Z")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "Zlpha");
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha");
    state.handle_text_input("2")?;
    state.handle_text_input("r")?;
    state.handle_text_input("x")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "xxpha");

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha");
    state.handle_text_input("R")?;
    assert_eq!(state.input_mode()?, InputMode::Replace);
    state.handle_text_input("XYZ")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    assert_eq!(state.input_mode()?, InputMode::Normal);
    assert_eq!(state.active_buffer_mut()?.text.text(), "XYZha");

    state.active_buffer_mut()?.text = TextBuffer::from_text("aBcD");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("~")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "ABcD");
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 1);

    state.active_buffer_mut()?.text = TextBuffer::from_text("aBcD");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("3")?;
    state.handle_text_input("~")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "AbCD");
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 3);

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("g")?;
    state.handle_text_input("U")?;
    state.handle_text_input("w")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "ALPHA beta");

    state.active_buffer_mut()?.text = TextBuffer::from_text("ALPHA beta");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("g")?;
    state.handle_text_input("u")?;
    state.handle_text_input("w")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "alpha beta");

    state.active_buffer_mut()?.text = TextBuffer::from_text("ABcd");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("v")?;
    state.handle_text_input("l")?;
    state.handle_text_input("u")?;
    assert_eq!(state.input_mode()?, InputMode::Normal);
    assert_eq!(state.active_buffer_mut()?.text.text(), "abcd");

    Ok(())
}

#[test]
fn vim_quickref_repeat_registers_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("d")?;
    state.handle_text_input("w")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "beta");

    state.handle_text_input(".")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "");

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha");
    state.handle_text_input("q")?;
    state.handle_text_input("a")?;
    state.handle_text_input("A")?;
    state.handle_text_input("!")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    state.handle_text_input("q")?;
    assert!(state.ui()?.vim().macros.contains_key(&'a'));

    state.active_buffer_mut()?.text = TextBuffer::from_text("beta");
    state.handle_text_input("@")?;
    state.handle_text_input("a")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "beta!");

    state.active_buffer_mut()?.text = TextBuffer::from_text("one\ntwo\n");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 1));
    state.handle_text_input("m")?;
    state.handle_text_input("a")?;
    state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 0));
    state.handle_text_input("'")?;
    state.handle_text_input("a")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 0);

    state.active_buffer_mut()?.set_cursor(TextPoint::new(1, 0));
    state.handle_text_input("`")?;
    state.handle_text_input("a")?;
    assert_eq!(state.active_buffer_mut()?.cursor_row(), 0);
    assert_eq!(state.active_buffer_mut()?.cursor_col(), 1);

    Ok(())
}

#[test]
fn vim_yanks_set_flash_selections() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha beta\ngamma delta");
    let buffer_id = state.active_buffer_mut()?.id();

    state.handle_text_input("y")?;
    state.handle_text_input("w")?;
    let line_flash = state.ui()?.yank_flash(buffer_id, Instant::now());
    assert_eq!(
        line_flash,
        Some(VisualSelection::Range(
            state
                .active_buffer_mut()?
                .line_range(0)
                .ok_or("missing line range")?
        ))
    );

    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    state.handle_text_input("V")?;
    state.handle_text_input("j")?;
    state.handle_text_input("y")?;
    let line_flash = state.ui()?.yank_flash(buffer_id, Instant::now());
    assert_eq!(
        line_flash,
        Some(VisualSelection::Range(
            state
                .active_buffer_mut()?
                .line_span_range(0, 2)
                .ok_or("missing span")?
        ))
    );

    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 0));
    assert!(state.try_runtime_keybinding(Keycode::V, Mod::LCTRLMOD)?);
    assert_eq!(state.ui()?.vim().visual_kind, VisualSelectionKind::Block);
    state.handle_text_input("l")?;
    state.handle_text_input("j")?;
    state.handle_text_input("y")?;
    assert_eq!(
        state.ui()?.yank_flash(buffer_id, Instant::now()),
        Some(VisualSelection::Block(BlockSelection {
            start_line: 0,
            end_line: 1,
            start_col: 0,
            end_col: 2,
        }))
    );

    Ok(())
}

#[test]
fn vim_delimited_text_objects_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar], \"baz\")");

    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 17));
    state.handle_text_input("c")?;
    state.handle_text_input("i")?;
    state.handle_text_input("\"")?;
    assert_eq!(state.input_mode()?, InputMode::Insert);
    state.handle_text_input("qux")?;
    assert!(state.try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)?);
    assert_eq!(
        state.active_buffer_mut()?.text.text(),
        "call(foo[bar], \"qux\")"
    );

    state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar], \"baz\")");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 9));
    state.handle_text_input("v")?;
    state.handle_text_input("i")?;
    state.handle_text_input("[")?;
    state.handle_text_input("d")?;
    assert_eq!(
        state.active_buffer_mut()?.text.text(),
        "call(foo[], \"baz\")"
    );

    state.active_buffer_mut()?.text = TextBuffer::from_text("call(foo[bar], \"baz\")");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 6));
    state.handle_text_input("d")?;
    state.handle_text_input("a")?;
    state.handle_text_input("(")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "call");

    state.active_buffer_mut()?.text = TextBuffer::from_text("outer{alpha(beta)}");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 12));
    state.handle_text_input("d")?;
    state.handle_text_input("i")?;
    state.handle_text_input("b")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "outer{alpha()}");

    state.active_buffer_mut()?.text = TextBuffer::from_text("outer{alpha(beta)}");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 12));
    state.handle_text_input("d")?;
    state.handle_text_input("a")?;
    state.handle_text_input("B")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "outer");

    Ok(())
}

#[test]
fn vim_quickref_text_objects_work() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;

    state.active_buffer_mut()?.text = TextBuffer::from_text("alpha-beta gamma");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 7));
    state.handle_text_input("d")?;
    state.handle_text_input("i")?;
    state.handle_text_input("W")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), " gamma");

    state.active_buffer_mut()?.text = TextBuffer::from_text("Alpha. Bravo! Charlie?");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 9));
    state.handle_text_input("d")?;
    state.handle_text_input("i")?;
    state.handle_text_input("s")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "Alpha.  Charlie?");

    state.active_buffer_mut()?.text = TextBuffer::from_text("one\n\nalpha\nbeta\n\ntwo\n");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(2, 1));
    state.handle_text_input("d")?;
    state.handle_text_input("i")?;
    state.handle_text_input("p")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "one\n\n\ntwo\n");

    state.active_buffer_mut()?.text = TextBuffer::from_text("foo <bar> baz");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 5));
    state.handle_text_input("d")?;
    state.handle_text_input("i")?;
    state.handle_text_input(">")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "foo <> baz");

    state.active_buffer_mut()?.text = TextBuffer::from_text("<div>hello</div>");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 6));
    state.handle_text_input("d")?;
    state.handle_text_input("i")?;
    state.handle_text_input("t")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "<div></div>");

    state.active_buffer_mut()?.text = TextBuffer::from_text("<div>hello</div>");
    state.active_buffer_mut()?.set_cursor(TextPoint::new(0, 6));
    state.handle_text_input("v")?;
    state.handle_text_input("i")?;
    state.handle_text_input("t")?;
    state.handle_text_input("d")?;
    assert_eq!(state.active_buffer_mut()?.text.text(), "<div></div>");

    Ok(())
}

#[test]
fn picker_bindings_open_and_navigate_results() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;

    assert!(state.try_runtime_keybinding(Keycode::F4, Mod::NOMOD)?);
    let initial = state
        .ui()?
        .picker()
        .and_then(|picker| picker.session().selected())
        .map(|selected| selected.item().id().to_owned())
        .ok_or("missing initial picker selection")?;

    assert!(state.try_runtime_keybinding(Keycode::N, Mod::LCTRLMOD)?);
    let next = state
        .ui()?
        .picker()
        .and_then(|picker| picker.session().selected())
        .map(|selected| selected.item().id().to_owned())
        .ok_or("missing next picker selection")?;

    assert_ne!(initial, next);

    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    assert!(!state.picker_visible()?);
    assert_eq!(state.active_buffer_mut()?.id().to_string(), next);
    Ok(())
}

#[test]
fn keybinding_picker_lists_runtime_bindings() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;

    assert!(state.try_runtime_keybinding(Keycode::F6, Mod::NOMOD)?);
    let picker = state.ui()?.picker().ok_or("missing keybinding picker")?;
    assert_eq!(picker.session().title(), "Keybindings");
    assert!(picker.session().matches().iter().any(|matched| {
        matched.item().label() == "F3" && matched.item().detail().contains("picker.open-commands")
    }));

    state.handle_text_input("h")?;
    let picker = state.ui()?.picker().ok_or("missing keybinding picker")?;
    assert!(picker.session().matches().iter().any(|matched| {
        matched.item().label() == "h" && matched.item().detail().contains("[normal]")
    }));

    Ok(())
}

#[test]
fn shell_defaults_to_sdl_canvas_backend() {
    assert_eq!(
        ShellConfig::default().render_backend,
        RenderBackend::SdlCanvas
    );
}

#[test]
fn lsp_hook_updates_statusline_state() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    let workspace_id = state.runtime.model().active_workspace_id()?;

    state
        .runtime
        .emit_hook(
            "lsp.server-start",
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_detail("rust-analyzer"),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(state.ui()?.attached_lsp_server(), Some("rust-analyzer"));
    assert_eq!(state.runtime.model().active_workspace()?.name(), "default");
    Ok(())
}

#[test]
fn workspace_helpers_open_switch_and_delete_workspaces() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    let default_workspace = state.runtime.model().active_workspace_id()?;
    let root = temp_workspace_root("demo");
    fs::create_dir_all(&root)?;

    let demo_workspace = open_workspace_from_project(&mut state.runtime, "demo", &root)?;
    assert_eq!(state.runtime.model().active_workspace()?.name(), "demo");
    assert_eq!(state.ui()?.active_workspace(), demo_workspace);

    switch_runtime_workspace(&mut state.runtime, default_workspace)?;
    assert_eq!(
        state.runtime.model().active_workspace_id()?,
        default_workspace
    );
    assert_eq!(state.ui()?.active_workspace(), default_workspace);

    delete_runtime_workspace(&mut state.runtime, demo_workspace)?;
    assert_eq!(
        state.runtime.model().active_workspace_id()?,
        default_workspace
    );
    assert_eq!(state.ui()?.active_workspace(), default_workspace);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn workspace_delete_picker_hides_default_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    let root = temp_workspace_root("picker");
    fs::create_dir_all(&root)?;
    let _workspace_id = open_workspace_from_project(&mut state.runtime, "picker", &root)?;

    let switch_picker = workspace_switch_picker_overlay(&state.runtime)?;
    assert!(
        switch_picker
            .session()
            .matches()
            .iter()
            .any(|matched| matched.item().label() == "default")
    );
    assert!(
        switch_picker
            .session()
            .matches()
            .iter()
            .any(|matched| matched.item().label() == "picker")
    );

    let delete_picker = workspace_delete_picker_overlay(&state.runtime)?;
    assert!(
        delete_picker
            .session()
            .matches()
            .iter()
            .all(|matched| matched.item().label() != "default")
    );
    assert!(
        delete_picker
            .session()
            .matches()
            .iter()
            .any(|matched| matched.item().label() == "picker")
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn workspace_file_picker_lists_visible_files_and_opens_selection()
-> Result<(), Box<dyn std::error::Error>> {
    if !git_available() {
        return Ok(());
    }

    let mut state = ShellState::new()?;
    let root = temp_workspace_root("files");
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join(".gitignore"), "ignored.txt\n")?;
    fs::write(
        root.join("src").join("main.rs"),
        "fn main() {\n    println!(\"hi\");\n}\n",
    )?;
    fs::write(root.join("ignored.txt"), "ignored\n")?;

    run_git(&root, &["init", "-q"])?;
    run_git(&root, &["add", ".gitignore", "src/main.rs"])?;
    open_workspace_from_project(&mut state.runtime, "files", &root)?;

    state
        .runtime
        .execute_command("workspace.list-files")
        .map_err(|error| error.to_string())?;
    let picker = state
        .ui()?
        .picker()
        .ok_or("missing workspace file picker")?;
    assert_eq!(picker.session().title(), "Workspace Files");
    assert!(
        picker
            .session()
            .matches()
            .iter()
            .any(|matched| matched.item().label().contains("main.rs"))
    );
    assert!(
        picker
            .session()
            .matches()
            .iter()
            .all(|matched| !matched.item().label().contains("ignored.txt"))
    );

    state.handle_text_input("main.rs")?;
    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    assert!(!state.picker_visible()?);
    let active = state.active_buffer_mut()?;
    assert_eq!(active.kind, BufferKind::File);
    assert!(active.display_name().contains("main.rs"));
    assert_eq!(active.text.line(0).as_deref(), Some("fn main() {"));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn workspace_file_picker_creates_missing_file() -> Result<(), Box<dyn std::error::Error>> {
    if !git_available() {
        return Ok(());
    }

    let mut state = ShellState::new()?;
    let root = temp_workspace_root("files-create");
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join(".gitignore"), "ignored.txt\n")?;
    fs::write(root.join("src").join("main.rs"), "fn main() {}\n")?;

    run_git(&root, &["init", "-q"])?;
    run_git(&root, &["add", ".gitignore", "src/main.rs"])?;
    open_workspace_from_project(&mut state.runtime, "files-create", &root)?;

    state
        .runtime
        .execute_command("workspace.list-files")
        .map_err(|error| error.to_string())?;

    state.handle_text_input("dir1/dir2/test.md")?;
    let picker = state
        .ui()?
        .picker()
        .ok_or("missing workspace file picker")?;
    assert!(picker.session().matches().is_empty());
    assert!(state.try_runtime_keybinding(Keycode::Return, Mod::NOMOD)?);
    assert!(!state.picker_visible()?);

    let new_path = root.join("dir1").join("dir2").join("test.md");
    assert!(new_path.exists());
    let active = state.active_buffer_mut()?;
    assert_eq!(active.kind, BufferKind::File);
    assert!(active.display_name().contains("dir1/dir2/test.md"));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn file_buffer_reload_refreshes_clean_open_buffers_after_disk_change()
-> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    let root = temp_workspace_root("file-reload");
    let path = root.join("src").join("main.rs");
    fs::create_dir_all(root.join("src"))?;
    fs::write(&path, "fn main() {}\n")?;

    {
        let buffer = state.active_buffer_mut()?;
        buffer.kind = BufferKind::File;
        buffer.name = path.display().to_string();
        buffer.text = TextBuffer::load_from_path(&path)?;
    }
    assert!(!state.refresh_pending_file_reloads_for_test()?);

    fs::write(&path, "fn main() {\n    println!(\"disk\");\n}\n")?;

    assert!(state.refresh_pending_file_reloads_for_test()?);
    assert_eq!(
        state.active_buffer_mut()?.text.line(1).as_deref(),
        Some("    println!(\"disk\");")
    );
    assert!(!state.active_buffer_mut()?.text.is_dirty());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn file_buffer_reload_waits_for_dirty_buffers_to_become_clean()
-> Result<(), Box<dyn std::error::Error>> {
    let mut state = ShellState::new()?;
    let root = temp_workspace_root("file-reload-dirty");
    let path = root.join("src").join("main.rs");
    fs::create_dir_all(root.join("src"))?;
    fs::write(&path, "fn main() {}\n")?;

    {
        let buffer = state.active_buffer_mut()?;
        buffer.kind = BufferKind::File;
        buffer.name = path.display().to_string();
        buffer.text = TextBuffer::load_from_path(&path)?;
    }
    assert!(!state.refresh_pending_file_reloads_for_test()?);

    {
        let buffer = state.active_buffer_mut()?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
    }
    fs::write(&path, "fn main() {\n    println!(\"disk\");\n}\n")?;

    assert!(!state.refresh_pending_file_reloads_for_test()?);
    assert_eq!(
        state.active_buffer_mut()?.text.line(0).as_deref(),
        Some("// local")
    );
    assert!(state.active_buffer_mut()?.text.undo());
    assert!(!state.active_buffer_mut()?.text.is_dirty());

    assert!(state.refresh_pending_file_reloads_for_test()?);
    assert_eq!(
        state.active_buffer_mut()?.text.line(1).as_deref(),
        Some("    println!(\"disk\");")
    );

    fs::remove_dir_all(root)?;
    Ok(())
}
