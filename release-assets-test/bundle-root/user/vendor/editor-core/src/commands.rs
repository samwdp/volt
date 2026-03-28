use std::{collections::BTreeMap, sync::Arc};

use crate::EditorRuntime;

type CommandHandler = Arc<dyn Fn(&mut EditorRuntime) -> Result<(), String> + Send + Sync>;

/// Identifies where a command originated from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSource {
    /// Core editor command defined by the runtime.
    Core,
    /// Command defined by a user package.
    UserPackage(String),
}

/// Public metadata describing a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDefinition {
    name: String,
    description: String,
    source: CommandSource,
}

impl CommandDefinition {
    fn new(name: String, description: String, source: CommandSource) -> Self {
        Self {
            name,
            description,
            source,
        }
    }

    /// Returns the command identifier.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the command summary.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the command origin.
    pub const fn source(&self) -> &CommandSource {
        &self.source
    }
}

#[derive(Clone)]
pub(crate) struct RegisteredCommand {
    definition: CommandDefinition,
    handler: CommandHandler,
}

impl RegisteredCommand {
    pub(crate) const fn definition(&self) -> &CommandDefinition {
        &self.definition
    }

    pub(crate) fn handler(&self) -> CommandHandler {
        Arc::clone(&self.handler)
    }
}

/// Registry of named runtime commands.
#[derive(Default)]
pub struct CommandRegistry {
    commands: BTreeMap<String, RegisteredCommand>,
}

impl CommandRegistry {
    /// Creates an empty command registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new named command.
    pub fn register<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        source: CommandSource,
        handler: F,
    ) -> Result<(), CommandError>
    where
        F: Fn(&mut EditorRuntime) -> Result<(), String> + Send + Sync + 'static,
    {
        let name = name.into();

        if self.commands.contains_key(&name) {
            return Err(CommandError::DuplicateCommand(name));
        }

        let definition = CommandDefinition::new(name.clone(), description.into(), source);
        let registered = RegisteredCommand {
            definition,
            handler: Arc::new(handler),
        };

        self.commands.insert(name, registered);
        Ok(())
    }

    /// Returns whether a command exists.
    pub fn contains(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    /// Returns the number of registered commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns whether no commands are registered.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Returns the definition for a command.
    pub fn get(&self, name: &str) -> Option<&CommandDefinition> {
        self.commands.get(name).map(RegisteredCommand::definition)
    }

    /// Returns registered command definitions in registry order.
    pub fn definitions(&self) -> Vec<&CommandDefinition> {
        self.commands
            .values()
            .map(RegisteredCommand::definition)
            .collect()
    }

    /// Returns the registered command names in sorted order.
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(String::as_str).collect()
    }

    pub(crate) fn resolve(&self, name: &str) -> Result<RegisteredCommand, CommandError> {
        self.commands
            .get(name)
            .cloned()
            .ok_or_else(|| CommandError::UnknownCommand(name.to_owned()))
    }
}

/// Errors raised by command registration or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    /// Attempted to register a command with a name that already exists.
    DuplicateCommand(String),
    /// Attempted to execute a command that does not exist.
    UnknownCommand(String),
    /// A command handler returned an execution failure message.
    ExecutionFailed {
        /// Command identifier.
        name: String,
        /// User-facing failure message.
        message: String,
    },
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCommand(name) => {
                write!(formatter, "command `{name}` is already registered")
            }
            Self::UnknownCommand(name) => write!(formatter, "command `{name}` is not registered"),
            Self::ExecutionFailed { name, message } => {
                write!(formatter, "command `{name}` failed: {message}")
            }
        }
    }
}

impl std::error::Error for CommandError {}
