#![doc = r#"Core runtime types shared by the editor application and future subsystems."#]

mod commands;
mod hooks;
mod keymaps;
mod model;
mod services;

pub use commands::{CommandDefinition, CommandError, CommandRegistry, CommandSource};
pub use hooks::{HookBus, HookDefinition, HookError, HookEvent, builtins};
pub use keymaps::{KeyBinding, KeymapError, KeymapRegistry, KeymapScope, KeymapVimMode};
pub use model::{
    Buffer, BufferId, BufferKind, EditorModel, ModelError, Pane, PaneId, Popup, PopupId, Window,
    WindowId, Workspace, WorkspaceId,
};
pub use services::ServiceRegistry;

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
        let command_name = command_name.into();

        if !self.commands.contains(&command_name) {
            return Err(KeymapError::UnknownCommand(command_name));
        }

        self.keymaps
            .register_for_mode(chord, command_name, scope, vim_mode, source)
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
        let command_name = binding.command_name().to_owned();

        self.execute_command(&command_name)
            .map_err(|error| KeymapError::CommandExecution {
                chord: chord.to_owned(),
                command: command_name,
                message: error.to_string(),
            })
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
mod tests {
    use std::path::PathBuf;

    use super::{
        BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope, KeymapVimMode,
        ModelError, builtins, runtime_descriptor,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ThemeService(&'static str);

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct EventLog(Vec<String>);

    #[test]
    fn runtime_descriptor_matches_expected_foundation_values() {
        let descriptor = runtime_descriptor();
        assert_eq!(descriptor.application_name, "volt");
        assert_eq!(descriptor.plugin_abi, "abi_stable");
    }

    #[test]
    fn runtime_bootstrap_tracks_editor_graph_and_services() -> Result<(), ModelError> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "scratch", None)?;
        let scratch_buffer = runtime.model_mut().create_buffer(
            workspace_id,
            "*scratch*",
            BufferKind::Scratch,
            None,
        )?;
        let command_buffer = runtime.model_mut().create_buffer(
            workspace_id,
            "*commands*",
            BufferKind::Picker,
            None,
        )?;
        let popup_id = runtime.model_mut().open_popup(
            workspace_id,
            "Command Palette",
            vec![scratch_buffer, command_buffer],
            command_buffer,
        )?;

        runtime.services_mut().insert(ThemeService("default"));

        let workspace = runtime.model().workspace(workspace_id)?;

        assert_eq!(runtime.model().window_count(), 1);
        assert_eq!(workspace.pane_count(), 1);
        assert_eq!(workspace.buffer_count(), 2);
        assert_eq!(workspace.popup_count(), 1);
        assert_eq!(
            workspace
                .active_pane()
                .and_then(|pane| pane.active_buffer()),
            Some(command_buffer)
        );
        assert_eq!(
            workspace.popup(popup_id).map(|popup| popup.active_buffer()),
            Some(command_buffer)
        );
        assert_eq!(
            runtime.services().get::<ThemeService>(),
            Some(&ThemeService("default"))
        );

        Ok(())
    }

    #[test]
    fn command_registry_executes_commands_and_hooks_dispatch_events() -> Result<(), String> {
        let mut runtime = EditorRuntime::new();
        runtime.services_mut().insert(EventLog::default());

        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "scratch", None)
            .map_err(|error| error.to_string())?;

        runtime
            .register_hook(
                "user.after-open-scratch",
                "Runs after the scratch buffer command completes.",
            )
            .map_err(|error| error.to_string())?;

        runtime
            .subscribe_hook(
                builtins::WORKSPACE_OPEN,
                "core.workspace-open-log",
                |event, runtime| {
                    let log = runtime
                        .services_mut()
                        .get_mut::<EventLog>()
                        .ok_or_else(|| "event log service missing".to_owned())?;
                    let workspace = event
                        .workspace_id
                        .map(|workspace_id| workspace_id.get())
                        .unwrap_or_default();
                    log.0.push(format!("workspace-open:{workspace}"));
                    Ok(())
                },
            )
            .map_err(|error| error.to_string())?;

        runtime
            .subscribe_hook(
                "user.after-open-scratch",
                "user.after-open-scratch-log",
                |event, runtime| {
                    let log = runtime
                        .services_mut()
                        .get_mut::<EventLog>()
                        .ok_or_else(|| "event log service missing".to_owned())?;
                    let detail = event.detail.as_deref().unwrap_or("unknown");
                    log.0.push(format!("after-open-scratch:{detail}"));
                    Ok(())
                },
            )
            .map_err(|error| error.to_string())?;

        runtime
            .register_command(
                "workspace.open-scratch",
                "Create a scratch buffer and emit a follow-up hook.",
                CommandSource::Core,
                move |runtime| {
                    let buffer_id = runtime
                        .model_mut()
                        .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)
                        .map_err(|error| error.to_string())?;

                    runtime
                        .emit_hook(
                            "user.after-open-scratch",
                            HookEvent::new()
                                .with_workspace(workspace_id)
                                .with_buffer(buffer_id)
                                .with_detail("scratch"),
                        )
                        .map_err(|error| error.to_string())?;

                    let log = runtime
                        .services_mut()
                        .get_mut::<EventLog>()
                        .ok_or_else(|| "event log service missing".to_owned())?;
                    log.0.push("command:workspace.open-scratch".to_owned());
                    Ok(())
                },
            )
            .map_err(|error| error.to_string())?;

        runtime
            .register_key_binding(
                "M-x scratch",
                "workspace.open-scratch",
                KeymapScope::Global,
                CommandSource::Core,
            )
            .map_err(|error| error.to_string())?;

        runtime
            .emit_hook(
                builtins::WORKSPACE_OPEN,
                HookEvent::new()
                    .with_window(window_id)
                    .with_workspace(workspace_id),
            )
            .map_err(|error| error.to_string())?;
        runtime
            .execute_key_binding(&KeymapScope::Global, "M-x scratch")
            .map_err(|error| error.to_string())?;

        let log = runtime
            .services()
            .get::<EventLog>()
            .ok_or_else(|| "event log service missing".to_owned())?;

        assert_eq!(
            log.0,
            vec![
                "workspace-open:1".to_owned(),
                "after-open-scratch:scratch".to_owned(),
                "command:workspace.open-scratch".to_owned(),
            ]
        );
        assert!(runtime.commands().contains("workspace.open-scratch"));
        assert!(runtime.hooks().contains("user.after-open-scratch"));
        assert!(
            runtime
                .keymaps()
                .contains(&KeymapScope::Global, "M-x scratch")
        );

        Ok(())
    }

    #[test]
    fn runtime_resolves_mode_specific_keybindings() -> Result<(), String> {
        let mut runtime = EditorRuntime::new();
        runtime.services_mut().insert(EventLog::default());

        runtime
            .register_command(
                "vim.normal-x",
                "Normal mode x",
                CommandSource::Core,
                |runtime| {
                    let log = runtime
                        .services_mut()
                        .get_mut::<EventLog>()
                        .ok_or_else(|| "event log service missing".to_owned())?;
                    log.0.push("normal-x".to_owned());
                    Ok(())
                },
            )
            .map_err(|error| error.to_string())?;
        runtime
            .register_command(
                "vim.visual-x",
                "Visual mode x",
                CommandSource::Core,
                |runtime| {
                    let log = runtime
                        .services_mut()
                        .get_mut::<EventLog>()
                        .ok_or_else(|| "event log service missing".to_owned())?;
                    log.0.push("visual-x".to_owned());
                    Ok(())
                },
            )
            .map_err(|error| error.to_string())?;
        runtime
            .register_command("vim.undo", "Undo", CommandSource::Core, |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<EventLog>()
                    .ok_or_else(|| "event log service missing".to_owned())?;
                log.0.push("undo".to_owned());
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        runtime
            .register_key_binding_for_mode(
                "x",
                "vim.normal-x",
                KeymapScope::Workspace,
                KeymapVimMode::Normal,
                CommandSource::Core,
            )
            .map_err(|error| error.to_string())?;
        runtime
            .register_key_binding_for_mode(
                "x",
                "vim.visual-x",
                KeymapScope::Workspace,
                KeymapVimMode::Visual,
                CommandSource::Core,
            )
            .map_err(|error| error.to_string())?;
        runtime
            .register_key_binding("u", "vim.undo", KeymapScope::Workspace, CommandSource::Core)
            .map_err(|error| error.to_string())?;

        runtime
            .execute_key_binding_for_mode(&KeymapScope::Workspace, KeymapVimMode::Normal, "x")
            .map_err(|error| error.to_string())?;
        runtime
            .execute_key_binding_for_mode(&KeymapScope::Workspace, KeymapVimMode::Visual, "x")
            .map_err(|error| error.to_string())?;
        runtime
            .execute_key_binding_for_mode(&KeymapScope::Workspace, KeymapVimMode::Visual, "u")
            .map_err(|error| error.to_string())?;

        let log = runtime
            .services()
            .get::<EventLog>()
            .ok_or_else(|| "event log service missing".to_owned())?;
        assert_eq!(
            log.0,
            vec![
                "normal-x".to_owned(),
                "visual-x".to_owned(),
                "undo".to_owned(),
            ]
        );

        Ok(())
    }

    #[test]
    fn model_switches_and_closes_workspaces() -> Result<(), ModelError> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let default_workspace = runtime
            .model_mut()
            .open_workspace(window_id, "default", None)?;
        let project_workspace = runtime.model_mut().open_workspace(
            window_id,
            "project",
            Some(PathBuf::from("C:\\projects\\demo")),
        )?;

        let workspace_names = runtime
            .model()
            .active_window()?
            .workspaces()
            .map(|workspace| workspace.name().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            workspace_names,
            vec!["default".to_owned(), "project".to_owned()]
        );
        assert_eq!(runtime.model().active_workspace_id()?, project_workspace);

        runtime.model_mut().switch_workspace(default_workspace)?;
        assert_eq!(runtime.model().active_workspace_id()?, default_workspace);

        let removed = runtime.model_mut().close_workspace(project_workspace)?;
        assert_eq!(removed.name(), "project");
        assert_eq!(runtime.model().active_window()?.workspace_count(), 1);
        assert_eq!(runtime.model().active_workspace_id()?, default_workspace);

        Ok(())
    }

    #[test]
    fn model_focuses_existing_buffer_in_active_pane() -> Result<(), ModelError> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "default", None)?;
        let scratch_id = runtime.model_mut().create_buffer(
            workspace_id,
            "*scratch*",
            BufferKind::Scratch,
            None,
        )?;
        let notes_id = runtime.model_mut().create_buffer(
            workspace_id,
            "*notes*",
            BufferKind::Scratch,
            None,
        )?;

        runtime.model_mut().focus_buffer(workspace_id, scratch_id)?;

        let active_buffer = runtime
            .model()
            .workspace(workspace_id)?
            .active_pane()
            .and_then(|pane| pane.active_buffer());
        assert_eq!(active_buffer, Some(scratch_id));
        assert!(
            runtime
                .model()
                .workspace(workspace_id)?
                .active_pane()
                .map(|pane| pane.buffer_ids().contains(&notes_id))
                .unwrap_or(false)
        );

        Ok(())
    }
}
