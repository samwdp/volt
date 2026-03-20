use std::{collections::BTreeMap, fmt};

use crate::CommandSource;

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
        self.bindings
            .keys()
            .any(|key| &key.scope == scope && key.chord == chord)
    }

    /// Returns a binding by scope and chord.
    pub fn get(&self, scope: &KeymapScope, chord: &str) -> Option<&KeyBinding> {
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

    /// Returns a binding by scope, Vim mode, and chord.
    pub fn get_for_mode(
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
        self.get_for_mode(scope, vim_mode, chord)
            .cloned()
            .ok_or_else(|| KeymapError::UnknownBinding {
                scope: scope.clone(),
                vim_mode,
                chord: chord.to_owned(),
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
