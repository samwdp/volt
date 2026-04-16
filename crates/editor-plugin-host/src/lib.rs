#![doc = r#"Core services responsible for discovering and orchestrating user packages."#]

use editor_core::builtins;
use editor_core::{
    BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope, KeymapVimMode, ModelError,
    SectionTree,
};
use editor_path::PathPattern;
use editor_plugin_api::{
    AcpClient, AutocompleteProvider, GitStatusPrefix, HoverProvider, LigatureConfig, OilKeyAction,
    PdfOpenMode, PluginAction, PluginActionKind, PluginKeymapScope, PluginPackage, PluginVimMode,
    TerminalConfig, WorkspaceRoot,
};
pub use editor_plugin_api::{StatuslineContext, UserLibrary};

// ─── NullUserLibrary ─────────────────────────────────────────────────────────

/// A fallback [`UserLibrary`] implementation that returns safe defaults and
/// minimal built-in providers.  Used when no user library has been registered
/// (e.g. in tests or minimal shell invocations).
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
        vec![
            AutocompleteProvider {
                id: "lsp".to_owned(),
                label: "LSP".to_owned(),
                icon: editor_icons::symbols::md::MD_COMMENT_TEXT_OUTLINE.to_owned(),
                item_icon: editor_icons::symbols::cod::COD_SYMBOL_MISC.to_owned(),
                or_group: Some("source".to_owned()),
                buffer_kind: None,
                items: Vec::new(),
            },
            AutocompleteProvider {
                id: "buffer".to_owned(),
                label: "Buffer".to_owned(),
                icon: editor_icons::symbols::cod::COD_TEXT_SIZE.to_owned(),
                item_icon: editor_icons::symbols::cod::COD_TEXT_SIZE.to_owned(),
                or_group: Some("source".to_owned()),
                buffer_kind: None,
                items: Vec::new(),
            },
        ]
    }
    fn autocomplete_result_limit(&self) -> usize {
        8
    }
    fn autocomplete_token_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_FORM_TEXTBOX
    }
    fn hover_providers(&self) -> Vec<HoverProvider> {
        vec![
            HoverProvider {
                id: "lsp".to_owned(),
                label: "LSP".to_owned(),
                icon: editor_icons::symbols::md::MD_COMMENT_TEXT_OUTLINE.to_owned(),
                line_limit: 10,
                buffer_kind: None,
                topics: Vec::new(),
            },
            HoverProvider {
                id: "signature-help".to_owned(),
                label: "Signature".to_owned(),
                icon: editor_icons::symbols::md::MD_SIGNATURE.to_owned(),
                line_limit: 10,
                buffer_kind: None,
                topics: Vec::new(),
            },
            HoverProvider {
                id: "diagnostics".to_owned(),
                label: "Diagnostics".to_owned(),
                icon: editor_icons::symbols::md::MD_ALERT_CIRCLE_OUTLINE.to_owned(),
                line_limit: 10,
                buffer_kind: None,
                topics: Vec::new(),
            },
            HoverProvider {
                id: "test-hover".to_owned(),
                label: "Token".to_owned(),
                icon: editor_icons::symbols::cod::COD_INFO.to_owned(),
                line_limit: 10,
                buffer_kind: None,
                topics: Vec::new(),
            },
        ]
    }
    fn hover_line_limit(&self) -> usize {
        10
    }
    fn hover_token_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_HELP_CIRCLE_OUTLINE
    }
    fn hover_signature_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_SIGNATURE
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
    fn commandline_enabled(&self) -> bool {
        true
    }
    fn ligature_config(&self) -> LigatureConfig {
        LigatureConfig { enabled: false }
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
        strip_icon_prefix(label, editor_icons::all_symbols())
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
    fn browser_url_prompt(&self) -> String {
        "URL > ".to_owned()
    }
    fn browser_url_placeholder(&self) -> String {
        "https://example.com".to_owned()
    }
    fn pdf_open_mode(&self) -> PdfOpenMode {
        PdfOpenMode::Rendered
    }
    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String {
        format!(
            " {} | {}:{} ",
            context.buffer_name, context.line, context.column
        )
    }
    fn statusline_lsp_connected_icon(&self) -> &'static str {
        editor_icons::symbols::md::MD_LAN_CONNECT
    }
    fn statusline_lsp_error_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_ERROR
    }
    fn statusline_lsp_warning_icon(&self) -> &'static str {
        editor_icons::symbols::cod::COD_WARNING
    }
    fn lsp_diagnostic_icon(&self) -> &'static str {
        "●"
    }
    fn lsp_diagnostic_line_limit(&self) -> usize {
        8
    }
    fn lsp_show_buffer_diagnostics(&self) -> bool {
        true
    }
    fn gitfringe_token_added(&self) -> &'static str {
        "git.fringe.added"
    }
    fn gitfringe_token_modified(&self) -> &'static str {
        "git.fringe.modified"
    }
    fn gitfringe_token_removed(&self) -> &'static str {
        "git.fringe.removed"
    }
    fn gitfringe_symbol(&self) -> &'static str {
        "⏽"
    }
    fn icon_symbols(&self) -> &'static [editor_icons::IconFontSymbol] {
        editor_icons::all_symbols()
    }
    fn run_plugin_buffer_evaluator(&self, _handler: &str, _input: &str) -> Vec<String> {
        Vec::new()
    }
    fn default_build_command(&self, _language: &str) -> Option<String> {
        None
    }
}

fn strip_icon_prefix<'a>(label: &'a str, symbols: &[editor_icons::IconFontSymbol]) -> &'a str {
    let trimmed = label.trim_start();
    let Some((maybe_icon, rest)) = trimmed.split_once(' ') else {
        return trimmed;
    };
    if symbols.iter().any(|symbol| symbol.glyph == maybe_icon) {
        rest.trim_start()
    } else {
        trimmed
    }
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
        runtime.register_key_binding_for_mode(
            binding.chord(),
            binding.command_name(),
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

    use super::{auto_loaded_packages, bootstrap, load_auto_loaded_packages};

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
