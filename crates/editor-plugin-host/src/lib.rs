#![doc = r#"Core services responsible for discovering and orchestrating user packages."#]

use editor_core::builtins;
use editor_core::{
    BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope, KeymapVimMode, ModelError,
};
use editor_path::PathPattern;
use editor_plugin_api::{
    PluginAction, PluginActionKind, PluginKeymapScope, PluginPackage, PluginVimMode,
};
pub use editor_plugin_api::{StatuslineContext, UserLibrary};

// ─── NullUserLibrary ─────────────────────────────────────────────────────────

/// A fallback [`UserLibrary`] implementation that returns safe defaults and
/// minimal built-in providers.  Used when no user library has been registered
/// (e.g. in tests or minimal shell invocations).
pub struct NullUserLibrary;

impl UserLibrary for NullUserLibrary {}

/// Foundation metadata describing the current host configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostBootstrap {
    /// Selected strategy for the core-to-user plugin ABI.
    pub plugin_abi: &'static str,
}

/// Returns the host bootstrap configuration used by the editor core.
pub const fn bootstrap() -> HostBootstrap {
    HostBootstrap {
        plugin_abi: "abi_stable",
    }
}

/// Errors raised while activating user packages inside the host runtime.
#[derive(Debug)]
pub enum HostError {
    /// Command registration failed.
    Command(editor_core::CommandError),
    /// Hook registration or dispatch setup failed.
    Hook(editor_core::HookError),
    /// Keybinding registration failed.
    Keymap(editor_core::KeymapError),
    /// The model state required by an action was unavailable.
    Model(ModelError),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(error) => error.fmt(formatter),
            Self::Hook(error) => error.fmt(formatter),
            Self::Keymap(error) => error.fmt(formatter),
            Self::Model(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for HostError {}

impl From<editor_core::CommandError> for HostError {
    fn from(error: editor_core::CommandError) -> Self {
        Self::Command(error)
    }
}

impl From<editor_core::HookError> for HostError {
    fn from(error: editor_core::HookError) -> Self {
        Self::Hook(error)
    }
}

impl From<editor_core::KeymapError> for HostError {
    fn from(error: editor_core::KeymapError) -> Self {
        Self::Keymap(error)
    }
}

impl From<ModelError> for HostError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

/// Returns only the packages configured to load automatically at startup.
pub fn auto_loaded_packages(packages: &[PluginPackage]) -> Vec<PluginPackage> {
    packages
        .iter()
        .filter(|package| package.auto_load())
        .cloned()
        .collect()
}

/// Clears runtime registrations produced by the provided user packages.
pub fn clear_package_registrations(
    runtime: &mut EditorRuntime,
    packages: &[PluginPackage],
) -> Result<(), HostError> {
    for package in packages {
        for binding in package.hook_bindings() {
            runtime.unsubscribe_hook(binding.hook_name(), binding.subscriber());
        }

        runtime.remove_package_keymap_registrations(package.name());
        runtime.remove_package_command_registrations(package.name());

        for declaration in package.hook_declarations() {
            runtime.remove_custom_hook(declaration.name())?;
        }
    }

    Ok(())
}

/// Replaces user-package registrations with metadata from the next package set.
pub fn reload_user_packages(
    runtime: &mut EditorRuntime,
    previous_packages: &[PluginPackage],
    next_packages: &[PluginPackage],
) -> Result<usize, HostError> {
    clear_package_registrations(runtime, previous_packages)?;
    load_auto_loaded_packages(runtime, next_packages)
}

/// Activates all auto-loaded user packages against the runtime and pre-registers
/// commands from self-contained manual packages so they remain globally discoverable.
pub fn load_auto_loaded_packages(
    runtime: &mut EditorRuntime,
    packages: &[PluginPackage],
) -> Result<usize, HostError> {
    for package in packages
        .iter()
        .filter(|package| !package.auto_load() && is_self_contained(package))
    {
        register_package_commands(runtime, package)?;
    }

    let auto_loaded = auto_loaded_packages(packages);

    for package in &auto_loaded {
        register_package(runtime, package)?;
    }

    Ok(auto_loaded.len())
}

fn register_package(runtime: &mut EditorRuntime, package: &PluginPackage) -> Result<(), HostError> {
    register_package_commands(runtime, package)?;

    for declaration in package.hook_declarations() {
        runtime.register_hook(declaration.name(), declaration.description())?;
    }

    for binding in package.key_bindings() {
        runtime.register_key_binding_for_mode_many(
            binding.chord(),
            binding
                .command_names()
                .iter()
                .map(|command_name| command_name.as_str().to_owned())
                .collect(),
            map_scope(binding.scope()),
            map_vim_mode(binding.vim_mode()),
            CommandSource::UserPackage(package.name().to_owned()),
        )?;
    }

    for binding in package.hook_bindings() {
        let is_file_open_hook = binding.hook_name() == builtins::FILE_OPEN;
        let subscriber = binding.subscriber().to_owned();
        let command_name = binding.command_name().to_owned();
        let detail_filter = binding.detail_filter().map(str::to_owned);

        runtime.subscribe_hook(
            binding.hook_name(),
            binding.subscriber(),
            move |event, runtime| {
                if !detail_filter_matches(
                    event.detail.as_deref(),
                    detail_filter.as_deref(),
                    is_file_open_hook,
                ) {
                    return Ok(());
                }

                runtime
                    .execute_command(&command_name)
                    .map_err(|error| error.to_string())?;

                println!("plugin hook subscriber `{subscriber}` executed `{command_name}`");
                Ok(())
            },
        )?;
    }

    Ok(())
}

fn detail_filter_matches(
    detail: Option<&str>,
    filter: Option<&str>,
    is_file_open_hook: bool,
) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    let Some(detail) = detail else {
        return false;
    };
    if detail == filter {
        return true;
    }
    if !is_file_open_hook {
        return false;
    }
    PathPattern::from_filter(filter).is_some_and(|pattern| pattern.matches_file_name(detail))
}

fn register_package_commands(
    runtime: &mut EditorRuntime,
    package: &PluginPackage,
) -> Result<(), HostError> {
    for command in package.commands() {
        let package_name = package.name().to_owned();
        let command_name = command.name().to_owned();
        let actions = command.actions().to_vec();

        runtime.register_command(
            command.name(),
            command.description(),
            CommandSource::UserPackage(package_name.clone()),
            move |runtime| run_actions(runtime, &package_name, &command_name, &actions),
        )?;
    }

    Ok(())
}

/// Returns whether a package can safely expose its commands without full activation.
///
/// Packages with no package-owned hook declarations or hook bindings are treated as
/// self-contained because their commands only rely on the host's built-in action support.
fn is_self_contained(package: &PluginPackage) -> bool {
    package.hook_declarations().is_empty() && package.hook_bindings().is_empty()
}

fn run_actions(
    runtime: &mut EditorRuntime,
    package_name: &str,
    command_name: &str,
    actions: &[PluginAction],
) -> Result<(), String> {
    for action in actions {
        match action.kind() {
            PluginActionKind::LogMessage => {
                let message = action.message().unwrap_or_default();
                println!("[plugin:{package_name}] {command_name}: {message}");
            }
            PluginActionKind::OpenBuffer => {
                let buffer = action
                    .buffer()
                    .ok_or_else(|| "open-buffer action missing payload".to_owned())?;
                open_buffer(
                    runtime,
                    buffer.buffer_name(),
                    buffer.buffer_kind(),
                    buffer.popup_title(),
                )
                .map_err(|error| error.to_string())?;
            }
            PluginActionKind::EmitHook => {
                let hook = action
                    .hook()
                    .ok_or_else(|| "emit-hook action missing payload".to_owned())?;
                let window_id = runtime.model().active_window_id();
                let workspace_id = runtime
                    .model()
                    .active_workspace_id()
                    .map_err(|error| error.to_string())?;
                let mut event = HookEvent::new().with_workspace(workspace_id);
                if let Some(window_id) = window_id {
                    event = event.with_window(window_id);
                }
                if let Ok(workspace) = runtime.model().workspace(workspace_id)
                    && let Some(pane_id) = workspace.active_pane_id()
                {
                    event = event.with_pane(pane_id);
                    if let Some(buffer_id) = workspace
                        .pane(pane_id)
                        .and_then(|pane| pane.active_buffer())
                    {
                        event = event.with_buffer(buffer_id);
                    }
                }
                if let Some(detail) = hook.detail() {
                    event = event.with_detail(detail);
                }

                runtime
                    .emit_hook(hook.hook_name(), event)
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    Ok(())
}

fn open_buffer(
    runtime: &mut EditorRuntime,
    buffer_name: &str,
    buffer_kind: &str,
    popup_title: Option<&str>,
) -> Result<(), ModelError> {
    let workspace_id = runtime.model().active_workspace_id()?;
    let buffer_id = if popup_title.is_some() {
        runtime.model_mut().create_popup_buffer(
            workspace_id,
            buffer_name,
            map_buffer_kind(buffer_kind),
            None,
        )?
    } else {
        runtime.model_mut().create_buffer(
            workspace_id,
            buffer_name,
            map_buffer_kind(buffer_kind),
            None,
        )?
    };

    if let Some(popup_title) = popup_title {
        runtime
            .model_mut()
            .open_popup_buffer(workspace_id, popup_title, buffer_id)?;
    }

    Ok(())
}

fn map_buffer_kind(buffer_kind: &str) -> BufferKind {
    match buffer_kind {
        "file" => BufferKind::File,
        "scratch" => BufferKind::Scratch,
        "picker" => BufferKind::Picker,
        "terminal" => BufferKind::Terminal,
        "git" => BufferKind::Git,
        "directory" => BufferKind::Directory,
        "diagnostics" => BufferKind::Diagnostics,
        other => BufferKind::Plugin(other.to_owned()),
    }
}

fn map_scope(scope: PluginKeymapScope) -> KeymapScope {
    match scope {
        PluginKeymapScope::Global => KeymapScope::Global,
        PluginKeymapScope::Workspace => KeymapScope::Workspace,
        PluginKeymapScope::Popup => KeymapScope::Popup,
        PluginKeymapScope::Autocomplete => KeymapScope::Autocomplete,
        PluginKeymapScope::Hover => KeymapScope::Hover,
        PluginKeymapScope::Dap => KeymapScope::Dap,
        PluginKeymapScope::WorkspaceDock => KeymapScope::WorkspaceDock,
        PluginKeymapScope::Multicursor => KeymapScope::Multicursor,
        PluginKeymapScope::AcpDock => KeymapScope::AcpDock,
    }
}

fn map_vim_mode(vim_mode: PluginVimMode) -> KeymapVimMode {
    match vim_mode {
        PluginVimMode::Any => KeymapVimMode::Any,
        PluginVimMode::Normal => KeymapVimMode::Normal,
        PluginVimMode::Insert => KeymapVimMode::Insert,
        PluginVimMode::Visual => KeymapVimMode::Visual,
    }
}

#[cfg(test)]
mod tests;
