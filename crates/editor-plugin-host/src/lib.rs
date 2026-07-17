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
        "compilation" => BufferKind::Compilation,
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
mod tests {
    use editor_core::{BufferKind, EditorRuntime, HookEvent, KeymapScope, builtins};
    use editor_plugin_api::{
        PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginKeyBinding,
        PluginKeymapScope, PluginPackage,
    };

    use super::{
        auto_loaded_packages, bootstrap, clear_package_registrations, load_auto_loaded_packages,
        reload_user_packages,
    };

    type HookContext = (
        Option<editor_core::WindowId>,
        Option<editor_core::WorkspaceId>,
        Option<editor_core::PaneId>,
        Option<editor_core::BufferId>,
    );

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct HookContextLog(Option<HookContext>);

    fn file_open_package(filter: &str, buffer_name: &str) -> PluginPackage {
        PluginPackage::new("tests", true, "File-open hook test package.")
            .with_commands(vec![PluginCommand::new(
                "tests.attach",
                "Attaches test behavior.",
                vec![PluginAction::open_buffer(
                    buffer_name,
                    "scratch",
                    None::<&str>,
                )],
            )])
            .with_hook_bindings(vec![PluginHookBinding::new(
                builtins::FILE_OPEN,
                format!("tests.auto-attach-{}", filter.replace('*', "star")),
                "tests.attach",
                Some(filter),
            )])
    }

    #[test]
    fn bootstrap_uses_the_selected_abi_strategy() {
        assert_eq!(bootstrap().plugin_abi, "abi_stable");
    }

    #[test]
    fn auto_loaded_packages_filters_manual_packages_out() {
        let packages = vec![
            PluginPackage::new("lsp", true, "Language server integration."),
            PluginPackage::new("git", false, "Git workflows."),
        ];

        let auto_loaded = auto_loaded_packages(&packages);
        assert_eq!(auto_loaded, vec![packages[0].clone()]);
    }

    #[test]
    fn host_loads_auto_packages_into_runtime() -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "scratch", None)?;

        let packages = vec![
            PluginPackage::new("terminal", true, "Builtin terminal package.")
                .with_commands(vec![PluginCommand::new(
                    "terminal.open",
                    "Opens the builtin terminal buffer.",
                    vec![PluginAction::open_buffer(
                        "*terminal*",
                        "terminal",
                        None::<&str>,
                    )],
                )])
                .with_key_bindings(vec![PluginKeyBinding::new(
                    "Ctrl+`",
                    "terminal.open",
                    PluginKeymapScope::Global,
                )]),
            PluginPackage::new("lang-rust", true, "Rust language defaults.")
                .with_hook_declarations(vec![PluginHookDeclaration::new(
                    "lang.rust.attached",
                    "Runs after Rust language support attaches.",
                )])
                .with_commands(vec![PluginCommand::new(
                    "lang-rust.attach",
                    "Attaches Rust language services.",
                    vec![
                        PluginAction::open_buffer(
                            "*rust-attachments*",
                            "diagnostics",
                            None::<&str>,
                        ),
                        PluginAction::emit_hook("lang.rust.attached", Some("rust")),
                    ],
                )])
                .with_hook_bindings(vec![PluginHookBinding::new(
                    builtins::FILE_OPEN,
                    "lang-rust.auto-attach",
                    "lang-rust.attach",
                    Some(".rs"),
                )]),
            PluginPackage::new("git", false, "Git workflows."),
        ];

        let loaded = load_auto_loaded_packages(&mut runtime, &packages)?;
        assert_eq!(loaded, 2);
        assert!(runtime.commands().contains("terminal.open"));
        assert!(runtime.keymaps().contains(&KeymapScope::Global, "Ctrl+`"));
        assert!(runtime.hooks().contains("lang.rust.attached"));

        runtime.execute_key_binding(&KeymapScope::Global, "Ctrl+`")?;
        runtime.emit_hook(
            builtins::FILE_OPEN,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_detail("main.rs"),
        )?;

        let workspace = runtime.model().workspace(workspace_id)?;
        assert_eq!(workspace.buffer_count(), 2);

        Ok(())
    }

    #[test]
    fn file_open_hook_filters_still_match_legacy_extension_details()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "scratch", None)?;
        let baseline_buffers = runtime.model().workspace(workspace_id)?.buffer_count();

        let packages = vec![file_open_package(".rs", "*legacy-extension*")];
        load_auto_loaded_packages(&mut runtime, &packages)?;

        runtime.emit_hook(
            builtins::FILE_OPEN,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_detail(".rs"),
        )?;

        let workspace = runtime.model().workspace(workspace_id)?;
        assert_eq!(workspace.buffer_count(), baseline_buffers + 1);
        Ok(())
    }

    #[test]
    fn file_open_hook_filters_match_exact_basenames() -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "scratch", None)?;
        let baseline_buffers = runtime.model().workspace(workspace_id)?.buffer_count();

        let packages = vec![file_open_package("Makefile", "*makefile*")];
        load_auto_loaded_packages(&mut runtime, &packages)?;

        runtime.emit_hook(
            builtins::FILE_OPEN,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_detail("Makefile"),
        )?;

        let workspace = runtime.model().workspace(workspace_id)?;
        assert_eq!(workspace.buffer_count(), baseline_buffers + 1);
        Ok(())
    }

    #[test]
    fn file_open_hook_filters_match_globs() -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "scratch", None)?;
        let baseline_buffers = runtime.model().workspace(workspace_id)?.buffer_count();

        let packages = vec![file_open_package("Dockerfile.*", "*dockerfile*")];
        load_auto_loaded_packages(&mut runtime, &packages)?;

        runtime.emit_hook(
            builtins::FILE_OPEN,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_detail("Dockerfile.dev"),
        )?;

        let workspace = runtime.model().workspace(workspace_id)?;
        assert_eq!(workspace.buffer_count(), baseline_buffers + 1);
        Ok(())
    }

    #[test]
    fn host_registers_self_contained_manual_package_commands()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "scratch", None)?;
        let baseline_buffers = runtime.model().workspace(workspace_id)?.buffer_count();

        let packages = vec![
            PluginPackage::new("calculator", false, "Self-contained calculator commands.")
                .with_commands(vec![PluginCommand::new(
                    "calculator.open",
                    "Opens the calculator buffer.",
                    vec![PluginAction::open_buffer(
                        "*calculator*",
                        "scratch",
                        None::<&str>,
                    )],
                )])
                .with_key_bindings(vec![PluginKeyBinding::new(
                    "Ctrl+=",
                    "calculator.open",
                    PluginKeymapScope::Global,
                )]),
            PluginPackage::new("lang-rust", false, "Rust language defaults.")
                .with_hook_declarations(vec![PluginHookDeclaration::new(
                    "lang.rust.attached",
                    "Runs after Rust language support attaches.",
                )])
                .with_commands(vec![PluginCommand::new(
                    "lang-rust.attach",
                    "Attaches Rust language services.",
                    vec![PluginAction::emit_hook("lang.rust.attached", Some("rust"))],
                )]),
        ];

        let loaded = load_auto_loaded_packages(&mut runtime, &packages)?;
        assert_eq!(loaded, 0);
        assert!(runtime.commands().contains("calculator.open"));
        assert!(!runtime.commands().contains("lang-rust.attach"));
        assert!(!runtime.keymaps().contains(&KeymapScope::Global, "Ctrl+="));

        runtime.execute_command("calculator.open")?;
        assert_eq!(
            runtime.model().workspace(workspace_id)?.buffer_count(),
            baseline_buffers + 1
        );

        Ok(())
    }

    #[test]
    fn reload_user_packages_replaces_commands_without_duplicates()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = EditorRuntime::new();
        let initial = vec![
            PluginPackage::new("terminal", true, "Builtin terminal package.").with_commands(vec![
                PluginCommand::new(
                    "terminal.open",
                    "Opens the builtin terminal buffer.",
                    vec![PluginAction::open_buffer(
                        "*terminal*",
                        "terminal",
                        None::<&str>,
                    )],
                ),
            ]),
        ];
        load_auto_loaded_packages(&mut runtime, &initial)?;

        let reloaded = vec![
            PluginPackage::new("terminal", true, "Builtin terminal package.").with_commands(vec![
                PluginCommand::new(
                    "terminal.open",
                    "Opens the builtin terminal buffer (reloaded).",
                    vec![PluginAction::log_message("reloaded terminal command")],
                ),
            ]),
        ];
        let loaded = reload_user_packages(&mut runtime, &initial, &reloaded)?;
        assert_eq!(loaded, 1);
        assert_eq!(
            runtime
                .commands()
                .get("terminal.open")
                .map(|command| command.description()),
            Some("Opens the builtin terminal buffer (reloaded).")
        );

        Ok(())
    }

    #[test]
    fn clear_package_registrations_removes_hook_bindings_and_declarations()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = EditorRuntime::new();
        let packages = vec![
            PluginPackage::new("lang-rust", true, "Rust language defaults.")
                .with_hook_declarations(vec![PluginHookDeclaration::new(
                    "lang.rust.attached",
                    "Runs after Rust language support attaches.",
                )])
                .with_hook_bindings(vec![PluginHookBinding::new(
                    builtins::FILE_OPEN,
                    "lang-rust.auto-attach",
                    "lang-rust.attach",
                    Some(".rs"),
                )])
                .with_commands(vec![PluginCommand::new(
                    "lang-rust.attach",
                    "Attaches Rust language services.",
                    vec![PluginAction::emit_hook("lang.rust.attached", Some("rust"))],
                )]),
        ];
        load_auto_loaded_packages(&mut runtime, &packages)?;
        assert!(runtime.hooks().contains("lang.rust.attached"));

        clear_package_registrations(&mut runtime, &packages)?;
        assert!(!runtime.commands().contains("lang-rust.attach"));
        assert!(!runtime.hooks().contains("lang.rust.attached"));

        Ok(())
    }

    #[test]
    fn emitted_hook_actions_include_active_window_pane_and_buffer()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("main");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "scratch", None)?;
        let buffer_id = runtime.model_mut().create_buffer(
            workspace_id,
            "*scratch*",
            BufferKind::Scratch,
            None,
        )?;
        runtime.model_mut().focus_buffer(workspace_id, buffer_id)?;
        let pane_id = runtime
            .model()
            .workspace(workspace_id)?
            .active_pane_id()
            .ok_or("active pane missing")?;

        runtime.register_hook(
            "tests.buffer-scoped",
            "Captures the active command context for emitted hooks.",
        )?;
        runtime.services_mut().insert(HookContextLog::default());
        runtime.subscribe_hook(
            "tests.buffer-scoped",
            "tests.capture-context",
            |event, runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<HookContextLog>()
                    .ok_or_else(|| "hook context log missing".to_owned())?;
                log.0 = Some((
                    event.window_id,
                    event.workspace_id,
                    event.pane_id,
                    event.buffer_id,
                ));
                Ok(())
            },
        )?;

        let packages = vec![
            PluginPackage::new("tests", true, "Hook context test package.").with_commands(vec![
                PluginCommand::new(
                    "tests.emit-context",
                    "Emits a hook with the active runtime context.",
                    vec![PluginAction::emit_hook("tests.buffer-scoped", None::<&str>)],
                ),
            ]),
        ];

        let loaded = load_auto_loaded_packages(&mut runtime, &packages)?;
        assert_eq!(loaded, 1);

        runtime.execute_command("tests.emit-context")?;

        assert_eq!(
            runtime.services().get::<HookContextLog>(),
            Some(&HookContextLog(Some((
                Some(window_id),
                Some(workspace_id),
                Some(pane_id),
                Some(buffer_id),
            ))))
        );

        Ok(())
    }
}
