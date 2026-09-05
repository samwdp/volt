mod commands;
mod hooks;
mod key_sequence;
mod keymaps;
mod model;
mod sections;
mod services;
mod workspace_nav;

pub use commands::{CommandDefinition, CommandError, CommandRegistry, CommandSource};
pub use hooks::{HookBus, HookDefinition, HookError, HookEvent, builtins};
pub use key_sequence::{
    DEFAULT_AMBIGUOUS_PREFIX_TIMEOUT_MS, DEFAULT_SEQUENCE_IDLE_TIMEOUT_MS, KeySequenceOptions,
    KeySequencePush, KeySequenceTick, PendingKeySequence, push_key_sequence, tick_key_sequence,
};
pub use keymaps::{KeyBinding, KeymapError, KeymapRegistry, KeymapScope, KeymapVimMode};
pub use model::{
    Buffer, BufferId, BufferKind, EditorModel, ModelError, Pane, PaneId, Popup, PopupId, Window,
    WindowId, Workspace, WorkspaceId,
};
pub use sections::{
    Section, SectionAction, SectionCollapseState, SectionItem, SectionRenderLine,
    SectionRenderLineKind, SectionTree,
};
pub use services::ServiceRegistry;
pub use workspace_nav::{
    CycleDirection, MarkList, MarkedWorkspaceJump, WorktreeRemovePlan, WorktreeRemoveRequest,
    cycle_project_workspace, marked_workspace_jump, normalize_project_root_path,
    plan_worktree_remove, project_roots_equal,
};

/// Describes the high-level runtime identity of the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeDescriptor {
    /// Stable application name used in logs and startup flows.
    pub application_name: &'static str,
    /// Selected strategy for the core-to-user plugin ABI.
    pub plugin_abi: &'static str,
}

/// Returns the current runtime descriptor for the editor.
pub const fn runtime_descriptor() -> RuntimeDescriptor {
    RuntimeDescriptor {
        application_name: "volt",
        plugin_abi: "abi_stable",
    }
}

/// Bundles the editor model and shared service registry used by the runtime.
pub struct EditorRuntime {
    descriptor: RuntimeDescriptor,
    model: EditorModel,
    services: ServiceRegistry,
    commands: CommandRegistry,
    hooks: HookBus,
    keymaps: KeymapRegistry,
}

impl EditorRuntime {
    /// Creates a new runtime with the default descriptor, model, and service registry.
    pub fn new() -> Self {
        Self {
            descriptor: runtime_descriptor(),
            model: EditorModel::new(),
            services: ServiceRegistry::new(),
            commands: CommandRegistry::new(),
            hooks: HookBus::new(),
            keymaps: KeymapRegistry::new(),
        }
    }

    /// Returns the static runtime descriptor.
    pub const fn descriptor(&self) -> RuntimeDescriptor {
        self.descriptor
    }

    /// Returns an immutable reference to the editor model.
    pub const fn model(&self) -> &EditorModel {
        &self.model
    }

    /// Returns a mutable reference to the editor model.
    pub fn model_mut(&mut self) -> &mut EditorModel {
        &mut self.model
    }

    /// Returns an immutable reference to the service registry.
    pub const fn services(&self) -> &ServiceRegistry {
        &self.services
    }

    /// Returns a mutable reference to the service registry.
    pub fn services_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.services
    }

    /// Returns the registered command definitions.
    pub const fn commands(&self) -> &CommandRegistry {
        &self.commands
    }

    /// Returns the known hook definitions and subscriptions.
    pub const fn hooks(&self) -> &HookBus {
        &self.hooks
    }

    /// Returns the registered keybindings.
    pub const fn keymaps(&self) -> &KeymapRegistry {
        &self.keymaps
    }

    /// Registers a new command with the runtime.
    pub fn register_command<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        source: CommandSource,
        handler: F,
    ) -> Result<(), CommandError>
    where
        F: Fn(&mut EditorRuntime) -> Result<(), String> + Send + Sync + 'static,
    {
        self.commands.register(name, description, source, handler)
    }

    /// Executes a registered command by name.
    pub fn execute_command(&mut self, command_name: &str) -> Result<(), CommandError> {
        let command = self.commands.resolve(command_name)?;
        let command_name = command.definition().name().to_owned();
        let handler = command.handler();

        handler(self).map_err(|message| CommandError::ExecutionFailed {
            name: command_name,
            message,
        })
    }

    /// Registers a keybinding for an already-known command.
    pub fn register_key_binding(
        &mut self,
        chord: impl Into<String>,
        command_name: impl Into<String>,
        scope: KeymapScope,
        source: CommandSource,
    ) -> Result<(), KeymapError> {
        self.register_key_binding_for_mode(chord, command_name, scope, KeymapVimMode::Any, source)
    }

    /// Registers a keybinding for an already-known command in a specific Vim mode.
    pub fn register_key_binding_for_mode(
        &mut self,
        chord: impl Into<String>,
        command_name: impl Into<String>,
        scope: KeymapScope,
        vim_mode: KeymapVimMode,
        source: CommandSource,
    ) -> Result<(), KeymapError> {
        self.register_key_binding_for_mode_many(
            chord,
            vec![command_name.into()],
            scope,
            vim_mode,
            source,
        )
    }

    /// Registers a keybinding for already-known commands in a specific Vim mode.
    pub fn register_key_binding_for_mode_many(
        &mut self,
        chord: impl Into<String>,
        command_names: Vec<String>,
        scope: KeymapScope,
        vim_mode: KeymapVimMode,
        source: CommandSource,
    ) -> Result<(), KeymapError> {
        assert!(
            !command_names.is_empty(),
            "keybinding requires at least one command"
        );
        for command_name in &command_names {
            if !self.commands.contains(command_name) {
                return Err(KeymapError::UnknownCommand(command_name.clone()));
            }
        }

        self.keymaps
            .register_for_mode_many(chord, command_names, scope, vim_mode, source)
    }

    /// Resolves a keybinding and executes its target command.
    pub fn execute_key_binding(
        &mut self,
        scope: &KeymapScope,
        chord: &str,
    ) -> Result<(), KeymapError> {
        self.execute_key_binding_for_mode(scope, KeymapVimMode::Any, chord)
    }

    /// Resolves a keybinding for a specific Vim mode and executes its target command.
    pub fn execute_key_binding_for_mode(
        &mut self,
        scope: &KeymapScope,
        vim_mode: KeymapVimMode,
        chord: &str,
    ) -> Result<(), KeymapError> {
        let binding = self.keymaps.resolve_for_mode(scope, vim_mode, chord)?;
        self.execute_resolved_key_binding(chord, &binding)
    }

    /// Resolves via active Minor Modes (then Global fallback) and executes when found.
    ///
    /// Returns `true` when a binding ran.
    pub fn execute_key_binding_with_minor_modes(
        &mut self,
        active_minor_modes: &[KeymapScope],
        vim_mode: KeymapVimMode,
        chord: &str,
    ) -> Result<bool, KeymapError> {
        let Some(binding) = self
            .keymaps
            .resolve_with_minor_modes(active_minor_modes, vim_mode, chord)
            .cloned()
        else {
            return Ok(false);
        };
        self.execute_resolved_key_binding(chord, &binding)?;
        Ok(true)
    }

    /// Executes a binding found only in the given Minor Mode scopes (no Global fallback).
    ///
    /// Returns `true` when a binding ran.
    pub fn execute_key_binding_in_scopes(
        &mut self,
        scopes: &[KeymapScope],
        vim_mode: KeymapVimMode,
        chord: &str,
    ) -> Result<bool, KeymapError> {
        let Some(binding) = self
            .keymaps
            .find_in_scopes(scopes, vim_mode, chord)
            .cloned()
        else {
            return Ok(false);
        };
        self.execute_resolved_key_binding(chord, &binding)?;
        Ok(true)
    }

    fn execute_resolved_key_binding(
        &mut self,
        chord: &str,
        binding: &KeyBinding,
    ) -> Result<(), KeymapError> {
        for command_name in binding.command_names() {
            self.execute_command(command_name)
                .map_err(|error| KeymapError::CommandExecution {
                    chord: chord.to_owned(),
                    command: command_name.clone(),
                    message: error.to_string(),
                })?;
        }
        Ok(())
    }

    /// Registers a new custom hook.
    pub fn register_hook(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<(), HookError> {
        self.hooks.register_hook(name, description)
    }

    /// Subscribes a callback to an existing hook.
    pub fn subscribe_hook<F>(
        &mut self,
        hook_name: impl Into<String>,
        subscriber: impl Into<String>,
        callback: F,
    ) -> Result<(), HookError>
    where
        F: Fn(&HookEvent, &mut EditorRuntime) -> Result<(), String> + Send + Sync + 'static,
    {
        self.hooks.subscribe(hook_name, subscriber, callback)
    }

    /// Removes a hook subscriber when present.
    pub fn unsubscribe_hook(&mut self, hook_name: &str, subscriber: &str) -> bool {
        self.hooks.unsubscribe(hook_name, subscriber)
    }

    /// Removes a custom hook declaration and its subscriptions.
    pub fn remove_custom_hook(&mut self, hook_name: &str) -> Result<bool, HookError> {
        self.hooks.remove_custom_hook(hook_name)
    }

    /// Removes commands registered by the given user package.
    pub fn remove_package_command_registrations(&mut self, package_name: &str) -> usize {
        self.commands.remove_by_package(package_name)
    }

    /// Removes keybindings registered by the given user package.
    pub fn remove_package_keymap_registrations(&mut self, package_name: &str) -> usize {
        self.keymaps.remove_by_package(package_name)
    }

    /// Emits a hook event to all current subscribers.
    pub fn emit_hook(&mut self, hook_name: &str, event: HookEvent) -> Result<(), HookError> {
        let subscriptions = self.hooks.subscriptions_for(hook_name)?;

        for subscription in subscriptions {
            let subscriber = subscription.subscriber().to_owned();
            let callback = subscription.callback();

            callback(&event, self).map_err(|message| HookError::HandlerFailed {
                hook: hook_name.to_owned(),
                subscriber,
                message,
            })?;
        }

        Ok(())
    }
}

impl Default for EditorRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
