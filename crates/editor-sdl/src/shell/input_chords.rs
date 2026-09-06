#[derive(Clone, Copy)]
struct ChordModifiers {
    ctrl: bool,
    alt: bool,
    shift: bool,
    gui: bool,
}

impl ChordModifiers {
    fn from_keymod(keymod: Mod) -> Self {
        Self {
            ctrl: keymod.intersects(ctrl_mod()),
            alt: keymod.intersects(alt_mod()),
            shift: keymod.intersects(shift_mod()),
            gui: keymod.intersects(gui_mod()),
        }
    }

    fn has_non_shift_modifier(self) -> bool {
        self.ctrl || self.alt || self.gui
    }
}

struct KeydownChordToken {
    key: String,
    printable: bool,
    include_shift: bool,
}

impl KeydownChordToken {
    fn printable(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            printable: true,
            include_shift: false,
        }
    }

    fn alphabetic(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            printable: true,
            include_shift: true,
        }
    }

    fn special(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            printable: false,
            include_shift: true,
        }
    }
}

fn keydown_chord(keycode: Keycode, keymod: Mod) -> Option<String> {
    let modifiers = ChordModifiers::from_keymod(keymod);
    let token = keydown_chord_token(keycode, modifiers)?;
    if token.printable && !modifiers.has_non_shift_modifier() {
        return None;
    }

    Some(build_keydown_chord(
        &token.key,
        ChordModifiers {
            shift: modifiers.shift && token.include_shift,
            ..modifiers
        },
    ))
}

fn keydown_chord_token(keycode: Keycode, modifiers: ChordModifiers) -> Option<KeydownChordToken> {
    let key_name = keycode_name_token(keycode)?;
    if key_name == "Space" {
        return Some(KeydownChordToken::printable("Space"));
    }

    let mut characters = key_name.chars();
    let character = characters.next()?;
    if characters.next().is_none() {
        if character.is_ascii_alphabetic() {
            return Some(KeydownChordToken::alphabetic(
                character.to_ascii_lowercase().to_string(),
            ));
        }
        let character = if modifiers.shift {
            shifted_printable_character(character).unwrap_or(character)
        } else {
            character
        };
        if modifiers.ctrl
            && matches!(
                keycode,
                Keycode::Equals | Keycode::Plus | Keycode::KpEquals | Keycode::KpPlus
            )
        {
            return Some(KeydownChordToken::printable("="));
        }
        return Some(KeydownChordToken::printable(character.to_string()));
    }

    Some(KeydownChordToken::special(normalize_named_key_token(
        &key_name,
    )))
}

fn build_keydown_chord(key: &str, modifiers: ChordModifiers) -> String {
    let mut chord = String::new();
    if modifiers.ctrl {
        chord.push_str("Ctrl+");
    }
    if modifiers.alt {
        chord.push_str("Alt+");
    }
    if modifiers.shift {
        chord.push_str("Shift+");
    }
    if modifiers.gui {
        chord.push_str("Gui+");
    }
    chord.push_str(key);
    chord
}

fn keycode_name_token(keycode: Keycode) -> Option<String> {
    if matches!(
        keycode,
        Keycode::ScancodeMask
            | Keycode::Unknown
            | Keycode::LCtrl
            | Keycode::RCtrl
            | Keycode::LShift
            | Keycode::RShift
            | Keycode::LAlt
            | Keycode::RAlt
            | Keycode::LGui
            | Keycode::RGui
            | Keycode::Mode
    ) {
        return None;
    }

    if matches!(
        keycode,
        Keycode::Return | Keycode::KpEnter | Keycode::Return2
    ) {
        return Some("Enter".to_owned());
    }

    let mut name = keycode.name();
    if let Some(stripped) = name.strip_prefix("Keypad ") {
        name = stripped.to_owned();
    }
    let normalized = name.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_owned())
    }
}

fn normalize_named_key_token(name: &str) -> String {
    name.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn shifted_printable_character(character: char) -> Option<char> {
    match character {
        '`' => Some('~'),
        '1' => Some('!'),
        '2' => Some('@'),
        '3' => Some('#'),
        '4' => Some('$'),
        '5' => Some('%'),
        '6' => Some('^'),
        '7' => Some('&'),
        '8' => Some('*'),
        '9' => Some('('),
        '0' => Some(')'),
        '-' => Some('_'),
        '=' => Some('+'),
        '[' => Some('{'),
        ']' => Some('}'),
        '\\' => Some('|'),
        ';' => Some(':'),
        '\'' => Some('"'),
        ',' => Some('<'),
        '.' => Some('>'),
        '/' => Some('?'),
        _ => None,
    }
}

fn text_chord(text: &str) -> Option<String> {
    let mut characters = text.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }
    Some(character.to_string())
}

fn key_sequence_options(user_library: &dyn UserLibrary) -> KeySequenceOptions {
    KeySequenceOptions {
        ambiguous_prefix_timeout_ms: user_library.keymap_config().ambiguous_prefix_timeout_ms,
        ..KeySequenceOptions::default()
    }
}

fn normalize_text_token(chord: &str) -> String {
    if chord == " " {
        "Space".to_owned()
    } else {
        chord.to_owned()
    }
}

fn suppressed_text_input_for_chord(chord: &str) -> Option<String> {
    let mut remaining = chord;
    let mut modifiers = ChordModifiers {
        ctrl: false,
        alt: false,
        shift: false,
        gui: false,
    };

    if let Some(stripped) = remaining.strip_prefix("Ctrl+") {
        modifiers.ctrl = true;
        remaining = stripped;
    }
    if let Some(stripped) = remaining.strip_prefix("Alt+") {
        modifiers.alt = true;
        remaining = stripped;
    }
    if let Some(stripped) = remaining.strip_prefix("Shift+") {
        modifiers.shift = true;
        remaining = stripped;
    }
    if let Some(stripped) = remaining.strip_prefix("Gui+") {
        modifiers.gui = true;
        remaining = stripped;
    }

    if !modifiers.has_non_shift_modifier() {
        return None;
    }

    if remaining == "Space" {
        return Some(" ".to_owned());
    }

    let mut characters = remaining.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }

    let text = if modifiers.shift && character.is_ascii_lowercase() {
        character.to_ascii_uppercase().to_string()
    } else {
        character.to_string()
    };
    Some(text)
}

fn ctrl_mod() -> Mod {
    Mod::LCTRLMOD | Mod::RCTRLMOD
}

fn shift_mod() -> Mod {
    Mod::LSHIFTMOD | Mod::RSHIFTMOD
}

fn alt_mod() -> Mod {
    Mod::LALTMOD | Mod::RALTMOD
}

fn gui_mod() -> Mod {
    Mod::LGUIMOD | Mod::RGUIMOD
}

fn browser_devtools_shortcut_requested(keycode: Keycode, keymod: Mod) -> bool {
    if !keymod.intersects(alt_mod() | gui_mod()) && keycode == Keycode::F12 {
        return true;
    }
    keycode == Keycode::I
        && keymod.intersects(ctrl_mod())
        && keymod.intersects(shift_mod())
        && !keymod.intersects(alt_mod() | gui_mod())
}

fn keymap_vim_mode(input_mode: InputMode) -> KeymapVimMode {
    match input_mode {
        InputMode::Normal => KeymapVimMode::Normal,
        InputMode::Insert | InputMode::Replace => KeymapVimMode::Insert,
        InputMode::Visual => KeymapVimMode::Visual,
    }
}

fn active_workspace_has_debug_session(runtime: &EditorRuntime) -> bool {
    let Ok(workspace_id) = runtime.model().active_workspace_id() else {
        return false;
    };
    let Some(dap_client) = runtime.services().get::<Arc<DapClientManager>>() else {
        return false;
    };
    dap_client
        .session_info(workspace_id.get())
        .ok()
        .flatten()
        .is_some()
}

fn plugin_buffer_binding_scope_active(
    scope: editor_plugin_api::PluginKeymapScope,
    popup_focused: bool,
) -> bool {
    match scope {
        editor_plugin_api::PluginKeymapScope::Global
        | editor_plugin_api::PluginKeymapScope::Workspace => true,
        editor_plugin_api::PluginKeymapScope::Popup => popup_focused,
        editor_plugin_api::PluginKeymapScope::Autocomplete
        | editor_plugin_api::PluginKeymapScope::Hover
        | editor_plugin_api::PluginKeymapScope::Dap
        | editor_plugin_api::PluginKeymapScope::WorkspaceDock
        | editor_plugin_api::PluginKeymapScope::AcpDock
        | editor_plugin_api::PluginKeymapScope::Multicursor => false,
    }
}

fn plugin_vim_mode_matches(
    binding_mode: editor_plugin_api::PluginVimMode,
    active_mode: KeymapVimMode,
) -> bool {
    match binding_mode {
        editor_plugin_api::PluginVimMode::Any => true,
        editor_plugin_api::PluginVimMode::Normal => active_mode == KeymapVimMode::Normal,
        editor_plugin_api::PluginVimMode::Insert => active_mode == KeymapVimMode::Insert,
        editor_plugin_api::PluginVimMode::Visual => active_mode == KeymapVimMode::Visual,
    }
}
