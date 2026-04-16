use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use crate::CommandSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ChordModifier {
    Ctrl,
    Alt,
    Shift,
    Gui,
}

impl ChordModifier {
    const fn label(self) -> &'static str {
        match self {
            Self::Ctrl => "Ctrl",
            Self::Alt => "Alt",
            Self::Shift => "Shift",
            Self::Gui => "Gui",
        }
    }
}

fn normalize_chord(chord: &str) -> String {
    chord
        .split_whitespace()
        .map(normalize_chord_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_chord_token(token: &str) -> String {
    normalize_delimited_token(token, '+')
        .or_else(|| normalize_delimited_token(token, '-'))
        .unwrap_or_else(|| normalize_key_name(token))
}

fn normalize_delimited_token(token: &str, delimiter: char) -> Option<String> {
    let parts = token.split(delimiter).collect::<Vec<_>>();
    if parts.len() < 2 || parts.iter().any(|part| part.is_empty()) {
        return None;
    }

    let mut modifiers = BTreeSet::new();
    for part in &parts[..parts.len() - 1] {
        modifiers.insert(parse_modifier(part)?);
    }

    if modifiers.is_empty() {
        return None;
    }

    Some(render_normalized_token(
        &modifiers,
        &normalize_key_name(parts[parts.len() - 1]),
    ))
}

fn render_normalized_token(modifiers: &BTreeSet<ChordModifier>, key: &str) -> String {
    let mut parts = modifiers
        .iter()
        .map(|modifier| modifier.label())
        .collect::<Vec<_>>();
    parts.push(key);
    parts.join("+")
}

fn parse_modifier(token: &str) -> Option<ChordModifier> {
    if token.eq_ignore_ascii_case("c")
        || token.eq_ignore_ascii_case("ctrl")
        || token.eq_ignore_ascii_case("control")
    {
        Some(ChordModifier::Ctrl)
    } else if token.eq_ignore_ascii_case("a")
        || token.eq_ignore_ascii_case("alt")
        || token.eq_ignore_ascii_case("option")
        || token.eq_ignore_ascii_case("opt")
        || token.eq_ignore_ascii_case("m")
    {
        Some(ChordModifier::Alt)
    } else if token.eq_ignore_ascii_case("s") || token.eq_ignore_ascii_case("shift") {
        Some(ChordModifier::Shift)
    } else if token.eq_ignore_ascii_case("g")
        || token.eq_ignore_ascii_case("gui")
        || token.eq_ignore_ascii_case("cmd")
        || token.eq_ignore_ascii_case("command")
        || token.eq_ignore_ascii_case("meta")
        || token.eq_ignore_ascii_case("super")
        || token.eq_ignore_ascii_case("win")
        || token.eq_ignore_ascii_case("windows")
    {
        Some(ChordModifier::Gui)
    } else {
        None
    }
}

fn normalize_key_name(token: &str) -> String {
    if token.eq_ignore_ascii_case("tab") {
        "Tab".to_owned()
    } else if token.eq_ignore_ascii_case("enter") || token.eq_ignore_ascii_case("return") {
        "Enter".to_owned()
    } else if token.eq_ignore_ascii_case("escape") || token.eq_ignore_ascii_case("esc") {
        "Escape".to_owned()
    } else if token.eq_ignore_ascii_case("space") {
        "Space".to_owned()
    } else if token.eq_ignore_ascii_case("pageup") || token.eq_ignore_ascii_case("page-up") {
        "PageUp".to_owned()
    } else if token.eq_ignore_ascii_case("pagedown") || token.eq_ignore_ascii_case("page-down") {
        "PageDown".to_owned()
    } else if token.eq_ignore_ascii_case("backspace") {
        "Backspace".to_owned()
    } else if token.eq_ignore_ascii_case("delete") || token.eq_ignore_ascii_case("del") {
        "Delete".to_owned()
    } else if token.eq_ignore_ascii_case("insert") || token.eq_ignore_ascii_case("ins") {
        "Insert".to_owned()
    } else if token.eq_ignore_ascii_case("home") {
        "Home".to_owned()
    } else if token.eq_ignore_ascii_case("end") {
        "End".to_owned()
    } else if token.eq_ignore_ascii_case("up") {
        "Up".to_owned()
    } else if token.eq_ignore_ascii_case("down") {
        "Down".to_owned()
    } else if token.eq_ignore_ascii_case("left") {
        "Left".to_owned()
    } else if token.eq_ignore_ascii_case("right") {
        "Right".to_owned()
    } else {
        normalize_function_key(token).unwrap_or_else(|| token.to_owned())
    }
}

fn normalize_function_key(token: &str) -> Option<String> {
    let mut characters = token.chars();
    let prefix = characters.next()?;
    if prefix != 'f' && prefix != 'F' {
        return None;
    }

    let digits = characters.as_str();
    if digits.is_empty() || !digits.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    Some(format!("F{digits}"))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    scope: KeymapScope,
    vim_mode: KeymapVimMode,
    chord: String,
}

/// Scope in which a keybinding is active.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeymapScope {
    /// Binding is active globally.
    Global,
    /// Binding is active while a workspace is focused.
    Workspace,
    /// Binding is active while a popup is focused.
    Popup,
}

impl fmt::Display for KeymapScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global => formatter.write_str("global"),
            Self::Workspace => formatter.write_str("workspace"),
            Self::Popup => formatter.write_str("popup"),
        }
    }
}

/// Modal Vim state that can activate a keybinding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeymapVimMode {
    /// Binding is active regardless of the current Vim mode.
    Any,
    /// Binding is active while Vim normal mode is focused.
    Normal,
    /// Binding is active while Vim insert mode is focused.
    Insert,
    /// Binding is active while Vim visual mode is focused.
    Visual,
}

impl fmt::Display for KeymapVimMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => formatter.write_str("any"),
            Self::Normal => formatter.write_str("normal"),
            Self::Insert => formatter.write_str("insert"),
            Self::Visual => formatter.write_str("visual"),
        }
    }
}

/// Metadata describing a registered keybinding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    chord: String,
    command_name: String,
    scope: KeymapScope,
    vim_mode: KeymapVimMode,
    source: CommandSource,
}

impl KeyBinding {
    fn new(
        chord: String,
        command_name: String,
        scope: KeymapScope,
        vim_mode: KeymapVimMode,
        source: CommandSource,
    ) -> Self {
        Self {
            chord,
            command_name,
            scope,
            vim_mode,
            source,
        }
    }

    /// Returns the key chord.
    pub fn chord(&self) -> &str {
        &self.chord
    }

    /// Returns the command invoked by the binding.
    pub fn command_name(&self) -> &str {
        &self.command_name
    }

    /// Returns the scope in which the binding applies.
    pub const fn scope(&self) -> &KeymapScope {
        &self.scope
    }

    /// Returns the Vim mode in which the binding applies.
    pub const fn vim_mode(&self) -> KeymapVimMode {
        self.vim_mode
    }

    /// Returns the source that registered the binding.
    pub const fn source(&self) -> &CommandSource {
        &self.source
    }
}

/// Registry of keybindings layered on top of the command registry.
#[derive(Default)]
pub struct KeymapRegistry {
    bindings: BTreeMap<BindingKey, KeyBinding>,
}

impl KeymapRegistry {
    /// Creates an empty keymap registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new keybinding.
    pub fn register(
        &mut self,
        chord: impl Into<String>,
        command_name: impl Into<String>,
        scope: KeymapScope,
        source: CommandSource,
    ) -> Result<(), KeymapError> {
        self.register_for_mode(chord, command_name, scope, KeymapVimMode::Any, source)
    }

    /// Registers a new keybinding for a specific Vim mode.
    pub fn register_for_mode(
        &mut self,
        chord: impl Into<String>,
        command_name: impl Into<String>,
        scope: KeymapScope,
        vim_mode: KeymapVimMode,
        source: CommandSource,
    ) -> Result<(), KeymapError> {
        let chord = chord.into();
        let chord = normalize_chord(&chord);
        let command_name = command_name.into();
        let key = BindingKey {
            scope: scope.clone(),
            vim_mode,
            chord: chord.clone(),
        };

        if self.bindings.contains_key(&key) {
            return Err(KeymapError::DuplicateBinding {
                scope,
                vim_mode,
                chord,
            });
        }

        self.bindings.insert(
            key,
            KeyBinding::new(chord, command_name, scope, vim_mode, source),
        );
        Ok(())
    }

    /// Returns the number of registered bindings.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Returns whether the registry contains no bindings.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Returns whether a binding exists for the given scope and chord.
    pub fn contains(&self, scope: &KeymapScope, chord: &str) -> bool {
        self.get(scope, chord).is_some()
    }

    /// Returns a binding by scope and chord.
    pub fn get(&self, scope: &KeymapScope, chord: &str) -> Option<&KeyBinding> {
        let chord = normalize_chord(chord);
        self.get_normalized(scope, &chord)
    }

    fn get_normalized(&self, scope: &KeymapScope, chord: &str) -> Option<&KeyBinding> {
        self.bindings
            .get(&BindingKey {
                scope: scope.clone(),
                vim_mode: KeymapVimMode::Any,
                chord: chord.to_owned(),
            })
            .or_else(|| {
                self.bindings
                    .iter()
                    .find(|(key, _)| &key.scope == scope && key.chord == chord)
                    .map(|(_, binding)| binding)
            })
    }

    /// Returns whether a binding exists for the given scope, Vim mode, and chord.
    pub fn contains_for_mode(
        &self,
        scope: &KeymapScope,
        vim_mode: KeymapVimMode,
        chord: &str,
    ) -> bool {
        self.get_for_mode(scope, vim_mode, chord).is_some()
    }

    /// Returns whether any binding has a multi-token prefix matching the provided tokens.
    pub fn has_sequence_prefix_for_mode(
        &self,
        scope: &KeymapScope,
        vim_mode: KeymapVimMode,
        tokens: &[String],
    ) -> bool {
        if tokens.is_empty() {
            return false;
        }
        let normalized_tokens = tokens
            .iter()
            .map(|token| normalize_chord_token(token))
            .collect::<Vec<_>>();
        self.bindings.keys().any(|key| {
            if &key.scope != scope {
                return false;
            }
            if key.vim_mode != vim_mode && key.vim_mode != KeymapVimMode::Any {
                return false;
            }
            let mut iter = key.chord.split_whitespace();
            for token in &normalized_tokens {
                match iter.next() {
                    Some(part) if part == token => {}
                    _ => return false,
                }
            }
            iter.next().is_some()
        })
    }

    /// Returns a binding by scope, Vim mode, and chord.
    pub fn get_for_mode(
        &self,
        scope: &KeymapScope,
        vim_mode: KeymapVimMode,
        chord: &str,
    ) -> Option<&KeyBinding> {
        let chord = normalize_chord(chord);
        self.get_normalized_for_mode(scope, vim_mode, &chord)
    }

    fn get_normalized_for_mode(
        &self,
        scope: &KeymapScope,
        vim_mode: KeymapVimMode,
        chord: &str,
    ) -> Option<&KeyBinding> {
        self.bindings
            .get(&BindingKey {
                scope: scope.clone(),
                vim_mode,
                chord: chord.to_owned(),
            })
            .or_else(|| {
                self.bindings.get(&BindingKey {
                    scope: scope.clone(),
                    vim_mode: KeymapVimMode::Any,
                    chord: chord.to_owned(),
                })
            })
    }

    /// Returns the registered keybindings in registry order.
    pub fn bindings(&self) -> Vec<&KeyBinding> {
        self.bindings.values().collect()
    }

    pub(crate) fn resolve_for_mode(
        &self,
        scope: &KeymapScope,
        vim_mode: KeymapVimMode,
        chord: &str,
    ) -> Result<KeyBinding, KeymapError> {
        let chord = normalize_chord(chord);
        self.get_normalized_for_mode(scope, vim_mode, &chord)
            .cloned()
            .ok_or_else(|| KeymapError::UnknownBinding {
                scope: scope.clone(),
                vim_mode,
                chord,
            })
    }
}

/// Errors raised by keymap registration or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeymapError {
    /// Attempted to register a duplicate binding.
    DuplicateBinding {
        /// Binding scope.
        scope: KeymapScope,
        /// Vim mode.
        vim_mode: KeymapVimMode,
        /// Key chord.
        chord: String,
    },
    /// Attempted to execute a binding that does not exist.
    UnknownBinding {
        /// Binding scope.
        scope: KeymapScope,
        /// Vim mode.
        vim_mode: KeymapVimMode,
        /// Key chord.
        chord: String,
    },
    /// Attempted to register a binding for an unknown command.
    UnknownCommand(String),
    /// The resolved command failed when invoked through a keybinding.
    CommandExecution {
        /// Key chord used to trigger the command.
        chord: String,
        /// Command identifier.
        command: String,
        /// User-facing failure message.
        message: String,
    },
}

impl fmt::Display for KeymapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateBinding {
                scope,
                vim_mode,
                chord,
            } => {
                if *vim_mode == KeymapVimMode::Any {
                    write!(
                        formatter,
                        "keybinding `{chord}` is already registered in {scope} scope"
                    )
                } else {
                    write!(
                        formatter,
                        "keybinding `{chord}` is already registered in {scope} scope for {vim_mode} Vim mode"
                    )
                }
            }
            Self::UnknownBinding {
                scope,
                vim_mode,
                chord,
            } => {
                if *vim_mode == KeymapVimMode::Any {
                    write!(
                        formatter,
                        "keybinding `{chord}` is not registered in {scope} scope"
                    )
                } else {
                    write!(
                        formatter,
                        "keybinding `{chord}` is not registered in {scope} scope for {vim_mode} Vim mode"
                    )
                }
            }
            Self::UnknownCommand(command) => {
                write!(
                    formatter,
                    "keybinding references unknown command `{command}`"
                )
            }
            Self::CommandExecution {
                chord,
                command,
                message,
            } => write!(
                formatter,
                "keybinding `{chord}` failed while executing `{command}`: {message}"
            ),
        }
    }
}

impl std::error::Error for KeymapError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_chord_reorders_modifiers_and_aliases() {
        assert_eq!(normalize_chord("Shift+Ctrl+I"), "Ctrl+Shift+I");
        assert_eq!(normalize_chord("Gui+Shift+Ctrl+P"), "Ctrl+Shift+Gui+P");
        assert_eq!(normalize_chord("S-F5"), "Shift+F5");
        assert_eq!(normalize_chord("M-x scratch"), "Alt+x scratch");
        assert_eq!(normalize_chord("C-c C-c"), "Ctrl+c Ctrl+c");
        assert_eq!(normalize_chord("g g"), "g g");
    }

    #[test]
    fn registry_resolves_equivalent_sequence_spellings() -> Result<(), KeymapError> {
        let mut registry = KeymapRegistry::new();
        registry.register(
            "M-x scratch",
            "workspace.open-scratch",
            KeymapScope::Global,
            CommandSource::Core,
        )?;

        assert!(registry.contains(&KeymapScope::Global, "Alt+x scratch"));
        assert!(registry.contains(&KeymapScope::Global, "M-x scratch"));
        assert_eq!(
            registry
                .get(&KeymapScope::Global, "Alt+x scratch")
                .map(KeyBinding::chord),
            Some("Alt+x scratch")
        );
        assert!(registry.has_sequence_prefix_for_mode(
            &KeymapScope::Global,
            KeymapVimMode::Any,
            &[String::from("Alt+x")],
        ));
        assert!(registry.has_sequence_prefix_for_mode(
            &KeymapScope::Global,
            KeymapVimMode::Any,
            &[String::from("M-x")],
        ));

        Ok(())
    }

    #[test]
    fn duplicate_detection_uses_canonical_chords() -> Result<(), KeymapError> {
        let mut registry = KeymapRegistry::new();
        registry.register(
            "Shift+F5",
            "workspace.compile",
            KeymapScope::Global,
            CommandSource::Core,
        )?;

        let error = registry
            .register(
                "S-F5",
                "workspace.recompile",
                KeymapScope::Global,
                CommandSource::Core,
            )
            .expect_err("legacy alias should conflict with canonical chord");

        assert_eq!(
            error,
            KeymapError::DuplicateBinding {
                scope: KeymapScope::Global,
                vim_mode: KeymapVimMode::Any,
                chord: "Shift+F5".to_owned(),
            }
        );

        Ok(())
    }
}
