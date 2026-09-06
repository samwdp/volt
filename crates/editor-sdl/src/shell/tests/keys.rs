#![allow(unused_imports)]
use super::*;

#[test]
fn keydown_chord_maps_alt_x() {
    assert_eq!(
        keydown_chord(Keycode::X, Mod::LALTMOD).as_deref(),
        Some("Alt+x")
    );
}

#[test]
fn keydown_chord_maps_enter_variants() {
    for keycode in [Keycode::Return, Keycode::KpEnter, Keycode::Return2] {
        assert_eq!(
            keydown_chord(keycode, ctrl_mod()).as_deref(),
            Some("Ctrl+Enter")
        );
        assert_eq!(keydown_chord(keycode, Mod::NOMOD).as_deref(), Some("Enter"));
    }
}

#[test]
fn keydown_chord_maps_image_zoom_controls() {
    assert_eq!(
        keydown_chord(Keycode::Equals, ctrl_mod()).as_deref(),
        Some("Ctrl+=")
    );
    assert_eq!(
        keydown_chord(Keycode::Minus, ctrl_mod()).as_deref(),
        Some("Ctrl+-")
    );
    assert_eq!(
        keydown_chord(Keycode::_0, ctrl_mod()).as_deref(),
        Some("Ctrl+0")
    );
}

#[test]
fn keydown_chord_maps_shifted_letter_and_function_key_modifiers() {
    assert_eq!(
        keydown_chord(Keycode::F7, Mod::NOMOD).as_deref(),
        Some("F7")
    );
    assert_eq!(
        keydown_chord(
            Keycode::F7,
            ctrl_mod() | alt_mod() | shift_mod() | gui_mod()
        )
        .as_deref(),
        Some("Ctrl+Alt+Shift+Gui+F7")
    );
    assert_eq!(
        keydown_chord(Keycode::H, ctrl_mod() | shift_mod()).as_deref(),
        Some("Ctrl+Shift+h")
    );
}

#[test]
fn keydown_chord_maps_shifted_printable_aliases() {
    assert_eq!(
        keydown_chord(Keycode::Backslash, ctrl_mod() | shift_mod()).as_deref(),
        Some("Ctrl+|")
    );
    assert_eq!(
        keydown_chord(Keycode::Pipe, ctrl_mod() | shift_mod()).as_deref(),
        Some("Ctrl+|")
    );
    assert_eq!(
        keydown_chord(Keycode::M, ctrl_mod()).as_deref(),
        Some("Ctrl+m")
    );
    assert_eq!(
        keydown_chord(Keycode::PageDown, Mod::NOMOD).as_deref(),
        Some("PageDown")
    );
}

#[test]
fn repeated_keydown_events_move_the_cursor() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(&mut state, "*repeat*", vec!["abcd".to_owned()])?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 3));

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Left),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: true,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(0, 2)
    );
    Ok(())
}
