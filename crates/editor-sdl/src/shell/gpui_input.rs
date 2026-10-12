fn sdl_keymod(modifiers: &gpui::Modifiers) -> Mod {
    let mut keymod = Mod::NOMOD;
    if modifiers.control {
        keymod |= ctrl_mod();
    }
    if modifiers.alt {
        keymod |= alt_mod();
    }
    if modifiers.shift {
        keymod |= shift_mod();
    }
    if modifiers.platform {
        keymod |= gui_mod();
    }
    keymod
}

fn letter_keycode(letter: char) -> Option<Keycode> {
    Some(match letter {
        'A' => Keycode::A,
        'B' => Keycode::B,
        'C' => Keycode::C,
        'D' => Keycode::D,
        'E' => Keycode::E,
        'F' => Keycode::F,
        'G' => Keycode::G,
        'H' => Keycode::H,
        'I' => Keycode::I,
        'J' => Keycode::J,
        'K' => Keycode::K,
        'L' => Keycode::L,
        'M' => Keycode::M,
        'N' => Keycode::N,
        'O' => Keycode::O,
        'P' => Keycode::P,
        'Q' => Keycode::Q,
        'R' => Keycode::R,
        'S' => Keycode::S,
        'T' => Keycode::T,
        'U' => Keycode::U,
        'V' => Keycode::V,
        'W' => Keycode::W,
        'X' => Keycode::X,
        'Y' => Keycode::Y,
        'Z' => Keycode::Z,
        _ => return None,
    })
}

fn digit_keycode(digit: char) -> Option<Keycode> {
    Some(match digit {
        '0' => Keycode::_0,
        '1' => Keycode::_1,
        '2' => Keycode::_2,
        '3' => Keycode::_3,
        '4' => Keycode::_4,
        '5' => Keycode::_5,
        '6' => Keycode::_6,
        '7' => Keycode::_7,
        '8' => Keycode::_8,
        '9' => Keycode::_9,
        _ => return None,
    })
}

fn function_keycode(key: &str) -> Option<Keycode> {
    Some(match key {
        "f1" => Keycode::F1,
        "f2" => Keycode::F2,
        "f3" => Keycode::F3,
        "f4" => Keycode::F4,
        "f5" => Keycode::F5,
        "f6" => Keycode::F6,
        "f7" => Keycode::F7,
        "f8" => Keycode::F8,
        "f9" => Keycode::F9,
        "f10" => Keycode::F10,
        "f11" => Keycode::F11,
        "f12" => Keycode::F12,
        _ => return None,
    })
}

fn keycode_from_gpui_key(key: &str) -> Option<Keycode> {
    let mut characters = key.chars();
    if let Some(character) = characters.next()
        && characters.next().is_none()
    {
        if character.is_ascii_alphabetic() {
            return letter_keycode(character.to_ascii_uppercase());
        }
        if character.is_ascii_digit() {
            return digit_keycode(character);
        }
    }
    if let Some(keycode) = function_keycode(key) {
        return Some(keycode);
    }
    Some(match key {
        "enter" => Keycode::Return,
        "escape" => Keycode::Escape,
        "backspace" => Keycode::Backspace,
        "tab" => Keycode::Tab,
        "space" => Keycode::Space,
        "left" => Keycode::Left,
        "right" => Keycode::Right,
        "up" => Keycode::Up,
        "down" => Keycode::Down,
        "pageup" => Keycode::PageUp,
        "pagedown" => Keycode::PageDown,
        "home" => Keycode::Home,
        "end" => Keycode::End,
        "delete" => Keycode::Delete,
        "insert" => Keycode::Insert,
        "-" => Keycode::Minus,
        "=" => Keycode::Equals,
        "[" => Keycode::LeftBracket,
        "]" => Keycode::RightBracket,
        "\\" => Keycode::Backslash,
        ";" => Keycode::Semicolon,
        "'" => Keycode::Apostrophe,
        "," => Keycode::Comma,
        "." => Keycode::Period,
        "/" => Keycode::Slash,
        "`" => Keycode::Grave,
        _ => return None,
    })
}

fn text_from_keystroke(keystroke: &gpui::Keystroke) -> Option<String> {
    if keystroke.modifiers.control || keystroke.modifiers.alt || keystroke.modifiers.platform {
        return None;
    }
    if keystroke.key == "space" {
        return Some(" ".to_owned());
    }
    if let Some(text) = keystroke.key_char.as_ref() {
        if text.is_empty() || text.chars().any(char::is_control) {
            return None;
        }
        return Some(text.clone());
    }
    let mut characters = keystroke.key.chars();
    let character = characters.next()?;
    if characters.next().is_some() || character.is_control() {
        return None;
    }
    if character.is_ascii_alphabetic() && keystroke.modifiers.shift {
        return Some(character.to_ascii_uppercase().to_string());
    }
    if character.is_ascii_graphic() {
        return Some(character.to_string());
    }
    None
}

fn sdl_mouse_button(button: gpui::MouseButton) -> MouseButton {
    match button {
        gpui::MouseButton::Left => MouseButton::Left,
        gpui::MouseButton::Right => MouseButton::Right,
        gpui::MouseButton::Middle => MouseButton::Middle,
        gpui::MouseButton::Navigate(gpui::NavigationDirection::Back) => MouseButton::X1,
        gpui::MouseButton::Navigate(gpui::NavigationDirection::Forward) => MouseButton::X2,
    }
}

fn key_down_event(event: &gpui::KeyDownEvent) -> Event {
    Event::KeyDown {
        timestamp: 0,
        window_id: 0,
        keycode: keycode_from_gpui_key(&event.keystroke.key),
        scancode: None,
        keymod: sdl_keymod(&event.keystroke.modifiers),
        repeat: event.is_held,
        which: 0,
        raw: 0,
    }
}

fn text_input_event(text: String) -> Event {
    Event::TextInput {
        timestamp: 0,
        window_id: 0,
        text,
    }
}
