#![doc = r#"Core services responsible for discovering and orchestrating user packages."#]

use editor_core::{
    BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope, KeymapVimMode, ModelError,
    SectionTree,
};
use editor_plugin_api::{
    AcpClient, AutocompleteProvider, GitStatusPrefix, HoverProvider, LspDiagnosticsInfo,
    OilDefaults, OilKeyAction, OilKeybindings, PluginAction, PluginActionKind, PluginKeymapScope,
    PluginPackage, PluginVimMode, TerminalConfig, WorkspaceRoot,
};

// ─── UserLibrary trait ───────────────────────────────────────────────────────

/// All configuration and behaviour provided by the compiled user extension
/// library.  The trait is the stable contract between the host (volt.exe /
/// editor-sdl) and the user crate (user.dll / libuser.so).
///
/// A *static* implementation wraps a direct call into the compiled-in user
/// module.  A *dynamic* implementation loads the symbols from a `.dll`/`.so`
/// at runtime, allowing users to recompile only their `user/` directory without
/// rebuilding the editor binary.
pub trait UserLibrary: Send + Sync {
    // ── Package registration ──────────────────────────────────────────────

    /// Returns all plugin packages compiled into the user library.
    fn packages(&self) -> Vec<PluginPackage>;

    /// Returns all themes compiled into the user library.
    fn themes(&self) -> Vec<editor_theme::Theme>;

    /// Returns all syntax-language registrations compiled into the user library.
    fn syntax_languages(&self) -> Vec<editor_syntax::LanguageConfiguration>;

    /// Returns all language-server specifications compiled into the user library.
    fn language_servers(&self) -> Vec<editor_lsp::LanguageServerSpec>;

    /// Returns all debug-adapter specifications compiled into the user library.
    fn debug_adapters(&self) -> Vec<editor_dap::DebugAdapterSpec>;

    // ── Autocomplete ──────────────────────────────────────────────────────

    /// Returns the ordered list of autocomplete providers.
    fn autocomplete_providers(&self) -> Vec<AutocompleteProvider>;

    // ── Hover ─────────────────────────────────────────────────────────────

    /// Returns the ordered list of hover providers.
    fn hover_providers(&self) -> Vec<HoverProvider>;

    // ── ACP ───────────────────────────────────────────────────────────────

    /// Returns all configured ACP client definitions.
    fn acp_clients(&self) -> Vec<AcpClient>;

    /// Looks up a single ACP client by its identifier.
    fn acp_client_by_id(&self, id: &str) -> Option<AcpClient>;

    // ── Workspace ─────────────────────────────────────────────────────────

    /// Returns the project search roots used by the workspace picker.
    fn workspace_roots(&self) -> Vec<WorkspaceRoot>;

    // ── Terminal ──────────────────────────────────────────────────────────

    /// Returns the default terminal programme and arguments.
    fn terminal_config(&self) -> TerminalConfig;

    // ── Oil directory browser ─────────────────────────────────────────────

    /// Returns user-configured default options for new oil buffers.
    fn oil_defaults(&self) -> OilDefaults;

    /// Returns user-configured keybindings for the oil directory browser.
    fn oil_keybindings(&self) -> OilKeybindings;

    /// Resolves an oil key action for a plain key-down event.
    fn oil_keydown_action(&self, chord: &str) -> Option<OilKeyAction>;

    /// Resolves an oil key action that may follow an active prefix.
    fn oil_chord_action(&self, had_prefix: bool, chord: &str) -> Option<OilKeyAction>;

    /// Returns the rendered help text shown in oil help popups.
    fn oil_help_lines(&self) -> Vec<String>;

    /// Builds the section tree for a directory listing.
    fn oil_directory_sections(
        &self,
        root: &std::path::Path,
        entries: &[editor_fs::DirectoryEntry],
        show_hidden: bool,
        sort_mode: editor_plugin_api::OilSortMode,
        trash_enabled: bool,
    ) -> SectionTree;

    /// Strips a leading icon prefix from an oil directory-entry label.
    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str;

    // ── Git ───────────────────────────────────────────────────────────────

    /// Builds the section tree for a git status snapshot.
    fn git_status_sections(&self, snapshot: &editor_git::GitStatusSnapshot) -> SectionTree;

    /// Returns the default commit-buffer template lines.
    fn git_commit_template(&self) -> Vec<String>;

    /// Returns the git status prefix (if any) indicated by a key chord.
    fn git_prefix_for_chord(&self, chord: &str) -> Option<GitStatusPrefix>;

    /// Returns the git command name (if any) for a prefix+chord combination.
    fn git_command_for_chord(
        &self,
        prefix: Option<GitStatusPrefix>,
        chord: &str,
    ) -> Option<&'static str>;

    // ── Browser ───────────────────────────────────────────────────────────

    /// Returns the rendered lines for a browser buffer.
    fn browser_buffer_lines(&self, url: Option<&str>) -> Vec<String>;

    /// Returns the input hint text for the browser URL bar.
    fn browser_input_hint(&self, url: Option<&str>) -> String;

    // ── Statusline ────────────────────────────────────────────────────────

    /// Renders the complete statusline string from a context snapshot.
    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String;

    /// Returns the icon used for LSP "connected" state in the statusline.
    fn statusline_lsp_connected_icon(&self) -> &'static str;

    /// Returns the icon used for LSP error diagnostics in the statusline.
    fn statusline_lsp_error_icon(&self) -> &'static str;

    /// Returns the icon used for LSP warning diagnostics in the statusline.
    fn statusline_lsp_warning_icon(&self) -> &'static str;

    // ── LSP ───────────────────────────────────────────────────────────────

    /// Returns the icon shown next to LSP diagnostics in the editor.
    fn lsp_diagnostic_icon(&self) -> &'static str;

    /// Returns the maximum number of diagnostic lines shown in hover popups.
    fn lsp_diagnostic_line_limit(&self) -> usize;

    // ── Icons ─────────────────────────────────────────────────────────────

    /// Returns the full icon font symbol table.
    fn icon_symbols(&self) -> &'static [editor_icons::IconFontSymbol];
}

// ─── NullUserLibrary ─────────────────────────────────────────────────────────

/// A no-op [`UserLibrary`] implementation that returns empty collections and
/// safe constant defaults.  Used as a fall-back when no user library has been
/// registered (e.g. in tests or minimal shell invocations).
pub struct NullUserLibrary;

impl UserLibrary for NullUserLibrary {
    fn packages(&self) -> Vec<PluginPackage> {
        Vec::new()
    }
    fn themes(&self) -> Vec<editor_theme::Theme> {
        Vec::new()
    }
    fn syntax_languages(&self) -> Vec<editor_syntax::LanguageConfiguration> {
        Vec::new()
    }
    fn language_servers(&self) -> Vec<editor_lsp::LanguageServerSpec> {
        Vec::new()
    }
    fn debug_adapters(&self) -> Vec<editor_dap::DebugAdapterSpec> {
        Vec::new()
    }
    fn autocomplete_providers(&self) -> Vec<AutocompleteProvider> {
        Vec::new()
    }
    fn hover_providers(&self) -> Vec<HoverProvider> {
        Vec::new()
    }
    fn acp_clients(&self) -> Vec<AcpClient> {
        Vec::new()
    }
    fn acp_client_by_id(&self, _id: &str) -> Option<AcpClient> {
        None
    }
    fn workspace_roots(&self) -> Vec<WorkspaceRoot> {
        Vec::new()
    }
    fn terminal_config(&self) -> TerminalConfig {
        #[cfg(target_os = "windows")]
        return TerminalConfig {
            program: "powershell.exe".to_owned(),
            args: vec!["-NoLogo".to_owned()],
        };
        #[cfg(not(target_os = "windows"))]
        return TerminalConfig {
            program: "bash".to_owned(),
            args: Vec::new(),
        };
    }
    fn oil_defaults(&self) -> editor_plugin_api::OilDefaults {
        editor_plugin_api::OilDefaults {
            show_hidden: false,
            sort_mode: editor_plugin_api::OilSortMode::TypeThenName,
            trash_enabled: false,
        }
    }
    fn oil_keybindings(&self) -> editor_plugin_api::OilKeybindings {
        editor_plugin_api::OilKeybindings {
            open_entry: "Enter",
            open_vertical_split: "s",
            open_horizontal_split: "S",
            open_new_pane: "p",
            preview_entry: "-",
            refresh: "gr",
            close: "q",
            prefix: "g",
            open_parent: "..",
            open_workspace_root: "~",
            set_root: "cd",
            show_help: "?",
            cycle_sort: "gs",
            toggle_hidden: "gh",
            toggle_trash: "gt",
            open_external: "gx",
            set_tab_local_root: "gl",
        }
    }
    fn oil_keydown_action(&self, _chord: &str) -> Option<OilKeyAction> {
        None
    }
    fn oil_chord_action(&self, _had_prefix: bool, _chord: &str) -> Option<OilKeyAction> {
        None
    }
    fn oil_help_lines(&self) -> Vec<String> {
        Vec::new()
    }
    fn oil_directory_sections(
        &self,
        _root: &std::path::Path,
        _entries: &[editor_fs::DirectoryEntry],
        _show_hidden: bool,
        _sort_mode: editor_plugin_api::OilSortMode,
        _trash_enabled: bool,
    ) -> SectionTree {
        SectionTree::default()
    }
    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str {
        label
    }
    fn git_status_sections(&self, _snapshot: &editor_git::GitStatusSnapshot) -> SectionTree {
        SectionTree::default()
    }
    fn git_commit_template(&self) -> Vec<String> {
        Vec::new()
    }
    fn git_prefix_for_chord(&self, _chord: &str) -> Option<GitStatusPrefix> {
        None
    }
    fn git_command_for_chord(
        &self,
        _prefix: Option<GitStatusPrefix>,
        _chord: &str,
    ) -> Option<&'static str> {
        None
    }
    fn browser_buffer_lines(&self, _url: Option<&str>) -> Vec<String> {
        Vec::new()
    }
    fn browser_input_hint(&self, _url: Option<&str>) -> String {
        String::new()
    }
    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String {
        format!(" {} | {}:{} ", context.buffer_name, context.line, context.column)
    }
    fn statusline_lsp_connected_icon(&self) -> &'static str {
        ""
    }
    fn statusline_lsp_error_icon(&self) -> &'static str {
        "E"
    }
    fn statusline_lsp_warning_icon(&self) -> &'static str {
        "W"
    }
    fn lsp_diagnostic_icon(&self) -> &'static str {
        "●"
    }
    fn lsp_diagnostic_line_limit(&self) -> usize {
        8
    }
    fn icon_symbols(&self) -> &'static [editor_icons::IconFontSymbol] {
        editor_icons::all_symbols()
    }
}

// ─── Statusline context ──────────────────────────────────────────────────────

/// Context passed to the user library when rendering the statusline.
#[derive(Debug, Clone)]
pub struct StatuslineContext<'a> {
    pub vim_mode: &'a str,
    pub recording_macro: Option<char>,
    pub workspace_name: &'a str,
    pub buffer_name: &'a str,
    pub buffer_modified: bool,
    pub language_id: Option<&'a str>,
    pub line: usize,
    pub column: usize,
    pub lsp_server: Option<&'a str>,
    pub lsp_diagnostics: Option<LspDiagnosticsInfo>,
    pub acp_connected: bool,
    pub git_branch: Option<&'a str>,
    pub git_added: usize,
    pub git_removed: usize,
}


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

/// Activates all auto-loaded user packages against the runtime.
pub fn load_auto_loaded_packages(
    runtime: &mut EditorRuntime,
    packages: &[PluginPackage],
) -> Result<usize, HostError> {
    let auto_loaded = auto_loaded_packages(packages);

    for package in &auto_loaded {
        register_package(runtime, package)?;
    }

    Ok(auto_loaded.len())
}

fn register_package(runtime: &mut EditorRuntime, package: &PluginPackage) -> Result<(), HostError> {
    for declaration in package.hook_declarations() {
        runtime.register_hook(declaration.name(), declaration.description())?;
    }

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

    for binding in package.key_bindings() {
        runtime.register_key_binding_for_mode(
            binding.chord(),
            binding.command_name(),
            map_scope(binding.scope()),
            map_vim_mode(binding.vim_mode()),
            CommandSource::UserPackage(package.name().to_owned()),
        )?;
    }

    for binding in package.hook_bindings() {
        let subscriber = binding.subscriber().to_owned();
        let command_name = binding.command_name().to_owned();
        let detail_filter = binding.detail_filter().map(str::to_owned);

        runtime.subscribe_hook(
            binding.hook_name(),
            binding.subscriber(),
            move |event, runtime| {
                if let Some(filter) = detail_filter.as_deref()
                    && event.detail.as_deref() != Some(filter)
                {
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
                let workspace_id = runtime
                    .model()
                    .active_workspace_id()
                    .map_err(|error| error.to_string())?;
                let mut event = HookEvent::new().with_workspace(workspace_id);
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
    use editor_core::{EditorRuntime, HookEvent, KeymapScope, builtins};
    use editor_plugin_api::{
        PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginKeyBinding,
        PluginKeymapScope, PluginPackage,
    };

    use super::{auto_loaded_packages, bootstrap, load_auto_loaded_packages};

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
                .with_detail(".rs"),
        )?;

        let workspace = runtime.model().workspace(workspace_id)?;
        assert_eq!(workspace.buffer_count(), 2);

        Ok(())
    }
}
