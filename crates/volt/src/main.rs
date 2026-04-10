#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use abi_stable::library::RootModule;
use editor_buffer::TextBuffer;
use editor_core::{BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope, builtins};
use editor_dap::{DebugAdapterRegistry, DebugConfiguration, DebugRequestKind, DebugSessionPlan};
use editor_fs::DirectoryBuffer;
use editor_git::parse_status;
use editor_jobs::{CompilationRunner, JobManager, JobSpec};
use editor_lsp::{LanguageServerRegistry, LanguageServerSession};
use editor_picker::{PickerItem, PickerSession};
use editor_plugin_api::abi::{
    AbiDirectoryEntry, AbiGhostTextContext, AbiGitStatusPrefix, AbiStatuslineContext,
    UserLibraryModuleRef,
};
use editor_plugin_host::{UserLibrary, bootstrap, load_auto_loaded_packages};
use editor_sdl::{ShellConfig, run_demo_shell};
use editor_syntax::SyntaxRegistry;
use editor_terminal::TerminalSession;
use editor_theme::ThemeRegistry;

#[cfg(test)]
mod standalone_user;
#[cfg(test)]
mod standalone_user_manifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StartupProfile {
    name: &'static str,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct EventLog(Vec<String>);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct CommandPaletteState {
    visible_items: usize,
    selected_command: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct CompilationState {
    success: bool,
    transcript_bytes: usize,
    exit_code: Option<i32>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct TerminalState {
    success: bool,
    line_count: usize,
    exit_code: Option<i32>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LspState {
    sessions: Vec<LanguageServerSession>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct DapState {
    sessions: Vec<DebugSessionPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchMode {
    ShellDemo,
    ShellHidden,
    BootstrapDemo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LaunchOptions {
    mode: LaunchMode,
    profile_input_latency: bool,
}

struct DynamicUserLibrary {
    module: UserLibraryModuleRef,
    icon_symbols: &'static [editor_icons::IconFontSymbol],
}

impl DynamicUserLibrary {
    fn new(module: UserLibraryModuleRef) -> Self {
        let icon_symbols = module.icon_symbols()()
            .into_iter()
            .map(editor_icons::IconFontSymbol::from)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            module,
            icon_symbols: Box::leak(icon_symbols),
        }
    }
}

impl UserLibrary for DynamicUserLibrary {
    fn packages(&self) -> Vec<editor_plugin_api::PluginPackage> {
        self.module.packages()().into_iter().collect()
    }

    fn themes(&self) -> Vec<editor_theme::Theme> {
        self.module.themes()().into_iter().map(Into::into).collect()
    }

    fn syntax_languages(&self) -> Vec<editor_syntax::LanguageConfiguration> {
        self.module.syntax_languages()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn language_servers(&self) -> Vec<editor_lsp::LanguageServerSpec> {
        self.module.language_servers()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn debug_adapters(&self) -> Vec<editor_dap::DebugAdapterSpec> {
        self.module.debug_adapters()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn autocomplete_providers(&self) -> Vec<editor_plugin_api::AutocompleteProvider> {
        self.module.autocomplete_providers()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn autocomplete_result_limit(&self) -> usize {
        self.module.autocomplete_result_limit()()
    }

    fn autocomplete_token_icon(&self) -> &'static str {
        self.module.autocomplete_token_icon()().as_str()
    }

    fn hover_providers(&self) -> Vec<editor_plugin_api::HoverProvider> {
        self.module.hover_providers()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn hover_line_limit(&self) -> usize {
        self.module.hover_line_limit()()
    }

    fn hover_token_icon(&self) -> &'static str {
        self.module.hover_token_icon()().as_str()
    }

    fn hover_signature_icon(&self) -> &'static str {
        self.module.hover_signature_icon()().as_str()
    }

    fn acp_clients(&self) -> Vec<editor_plugin_api::AcpClient> {
        self.module.acp_clients()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn acp_client_by_id(&self, id: &str) -> Option<editor_plugin_api::AcpClient> {
        self.module.acp_client_by_id()(id.to_owned().into())
            .into_option()
            .map(Into::into)
    }

    fn workspace_roots(&self) -> Vec<editor_plugin_api::WorkspaceRoot> {
        self.module.workspace_roots()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn terminal_config(&self) -> editor_plugin_api::TerminalConfig {
        self.module.terminal_config()().into()
    }

    fn commandline_enabled(&self) -> bool {
        self.module.commandline_enabled()()
    }

    fn ligature_config(&self) -> editor_plugin_api::LigatureConfig {
        self.module.ligature_config_v1()().into()
    }

    fn oil_defaults(&self) -> editor_plugin_api::OilDefaults {
        self.module.oil_defaults()().into()
    }

    fn oil_keybindings(&self) -> editor_plugin_api::OilKeybindings {
        self.module.oil_keybindings()().into()
    }

    fn oil_keydown_action(&self, chord: &str) -> Option<editor_plugin_api::OilKeyAction> {
        self.module.oil_keydown_action()(chord.to_owned().into())
            .into_option()
            .map(Into::into)
    }

    fn oil_chord_action(
        &self,
        had_prefix: bool,
        chord: &str,
    ) -> Option<editor_plugin_api::OilKeyAction> {
        self.module.oil_chord_action()(had_prefix, chord.to_owned().into())
            .into_option()
            .map(Into::into)
    }

    fn oil_help_lines(&self) -> Vec<String> {
        self.module.oil_help_lines()()
            .into_iter()
            .map(|line| line.into_string())
            .collect()
    }

    fn oil_directory_sections(
        &self,
        root: &Path,
        entries: &[editor_fs::DirectoryEntry],
        show_hidden: bool,
        sort_mode: editor_plugin_api::OilSortMode,
        trash_enabled: bool,
    ) -> editor_core::SectionTree {
        let entries = entries
            .iter()
            .cloned()
            .map(AbiDirectoryEntry::from)
            .collect::<Vec<_>>();
        self.module.oil_directory_sections()(
            root.to_string_lossy().into_owned().into(),
            entries.into(),
            show_hidden,
            sort_mode.into(),
            trash_enabled,
        )
        .into()
    }

    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str {
        let stripped = self.module.oil_strip_entry_icon_prefix()(label.to_owned().into());
        if stripped.as_str() == label {
            label
        } else {
            label
                .find(stripped.as_str())
                .map(|start| &label[start..start + stripped.len()])
                .unwrap_or(label)
        }
    }

    fn git_status_sections(
        &self,
        snapshot: &editor_git::GitStatusSnapshot,
    ) -> editor_core::SectionTree {
        self.module.git_status_sections()(snapshot.clone().into()).into()
    }

    fn git_commit_template(&self) -> Vec<String> {
        self.module.git_commit_template()()
            .into_iter()
            .map(|line| line.into_string())
            .collect()
    }

    fn git_prefix_for_chord(&self, chord: &str) -> Option<editor_plugin_api::GitStatusPrefix> {
        self.module.git_prefix_for_chord()(chord.to_owned().into())
            .into_option()
            .map(Into::into)
    }

    fn git_command_for_chord(
        &self,
        prefix: Option<editor_plugin_api::GitStatusPrefix>,
        chord: &str,
    ) -> Option<&'static str> {
        let command = self.module.git_command_for_chord()(
            prefix.map(AbiGitStatusPrefix::from).into(),
            chord.to_owned().into(),
        )
        .into_option();
        command.map(|command| command.as_str())
    }

    fn browser_buffer_lines(&self, url: Option<&str>) -> Vec<String> {
        let url = url.map(|value| value.to_owned().into());
        self.module.browser_buffer_lines()(url.into())
            .into_iter()
            .map(|line| line.into_string())
            .collect()
    }

    fn browser_input_hint(&self, url: Option<&str>) -> String {
        let url = url.map(|value| value.to_owned().into());
        self.module.browser_input_hint()(url.into()).into()
    }

    fn browser_url_prompt(&self) -> String {
        self.module.browser_url_prompt()().into()
    }

    fn browser_url_placeholder(&self) -> String {
        self.module.browser_url_placeholder()().into()
    }

    fn ghost_text_lines(
        &self,
        context: &editor_plugin_api::GhostTextContext<'_>,
    ) -> Vec<editor_plugin_api::GhostTextLine> {
        self.module.ghost_text_lines()(AbiGhostTextContext::from(*context))
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn headerline_lines(&self, context: &editor_plugin_api::GhostTextContext<'_>) -> Vec<String> {
        self.module.headerline_lines()(AbiGhostTextContext::from(*context))
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn statusline_render(&self, context: &editor_plugin_api::StatuslineContext<'_>) -> String {
        self.module.statusline_render()(AbiStatuslineContext::from(*context)).into()
    }

    fn statusline_lsp_connected_icon(&self) -> &'static str {
        self.module.statusline_lsp_connected_icon()().as_str()
    }

    fn statusline_lsp_error_icon(&self) -> &'static str {
        self.module.statusline_lsp_error_icon()().as_str()
    }

    fn statusline_lsp_warning_icon(&self) -> &'static str {
        self.module.statusline_lsp_warning_icon()().as_str()
    }

    fn lsp_diagnostic_icon(&self) -> &'static str {
        self.module.lsp_diagnostic_icon()().as_str()
    }

    fn lsp_diagnostic_line_limit(&self) -> usize {
        self.module.lsp_diagnostic_line_limit()()
    }

    fn lsp_show_buffer_diagnostics(&self) -> bool {
        self.module.lsp_show_buffer_diagnostics()()
    }

    fn gitfringe_token_added(&self) -> &'static str {
        self.module.gitfringe_token_added()().as_str()
    }

    fn gitfringe_token_modified(&self) -> &'static str {
        self.module.gitfringe_token_modified()().as_str()
    }

    fn gitfringe_token_removed(&self) -> &'static str {
        self.module.gitfringe_token_removed()().as_str()
    }

    fn gitfringe_symbol(&self) -> &'static str {
        self.module.gitfringe_symbol()().as_str()
    }

    fn icon_symbols(&self) -> &'static [editor_icons::IconFontSymbol] {
        self.icon_symbols
    }

    fn run_plugin_buffer_evaluator(&self, handler: &str, input: &str) -> Vec<String> {
        self.module.run_plugin_buffer_evaluator()(
            handler.to_owned().into(),
            input.to_owned().into(),
        )
        .into_iter()
        .map(|line| line.into_string())
        .collect()
    }

    fn default_build_command(&self, language: &str) -> Option<String> {
        self.module.default_build_command()(language.to_owned().into())
            .into_option()
            .map(|command| command.into_string())
    }
}

fn user_library_candidates(exe_path: Option<&Path>, env_path: Option<&str>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    if let Some(env_path) = env_path {
        let path = PathBuf::from(env_path);
        if seen.insert(path.clone()) {
            candidates.push(path);
        }
    }
    if let Some(exe_path) = exe_path.and_then(Path::parent) {
        let path = UserLibraryModuleRef::get_library_path(exe_path);
        if seen.insert(path.clone()) {
            candidates.push(path);
        }
    }
    candidates
}

fn load_user_library() -> Arc<dyn UserLibrary> {
    let current_exe = std::env::current_exe().ok();
    for path in user_library_candidates(
        current_exe.as_deref(),
        std::env::var("VOLT_USER_LIBRARY").ok().as_deref(),
    ) {
        if !path.is_file() {
            continue;
        }
        match UserLibraryModuleRef::load_from_file(&path) {
            Ok(module) => {
                eprintln!("loaded user library from `{}`", path.display());
                return Arc::new(DynamicUserLibrary::new(module));
            }
            Err(error) => {
                eprintln!(
                    "failed to load runtime user library `{}`: {error}",
                    path.display()
                );
            }
        }
    }
    Arc::new(user::UserLibraryImpl)
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_launch_options(std::env::args().skip(1))?;
    let user_library = load_user_library();
    match options.mode {
        LaunchMode::ShellDemo => {
            let summary = run_demo_shell(ShellConfig {
                profile_input_latency: options.profile_input_latency,
                user_library: Some(Arc::clone(&user_library)),
                ..ShellConfig::default()
            })?;
            print_shell_summary("Volt", &summary);
            return Ok(());
        }
        LaunchMode::ShellHidden => {
            let summary = run_demo_shell(ShellConfig {
                hidden: true,
                frame_limit: Some(1),
                profile_input_latency: options.profile_input_latency,
                user_library: Some(Arc::clone(&user_library)),
                ..ShellConfig::default()
            })?;
            print_shell_summary("volt hidden shell smoke test", &summary);
            return Ok(());
        }
        LaunchMode::BootstrapDemo => {
            if options.profile_input_latency {
                return Err("`--profile-input` is only supported with shell modes".into());
            }
        }
    }

    let mut runtime = EditorRuntime::new();
    runtime
        .services_mut()
        .insert(StartupProfile { name: "foundation" });
    runtime.services_mut().insert(EventLog::default());
    runtime.services_mut().insert(Mutex::new(JobManager::new()));

    let window_id = runtime.model_mut().create_window("volt");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)?;

    runtime.register_hook(
        "user.after-open-scratch",
        "Runs after the scratch buffer command completes.",
    )?;

    runtime.subscribe_hook(
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
    )?;

    runtime.subscribe_hook(builtins::STARTUP, "core.startup-log", |event, runtime| {
        let log = runtime
            .services_mut()
            .get_mut::<EventLog>()
            .ok_or_else(|| "event log service missing".to_owned())?;
        let detail = event.detail.as_deref().unwrap_or("bootstrap");
        log.0.push(format!("startup:{detail}"));
        Ok(())
    })?;

    runtime.subscribe_hook(
        builtins::FILE_OPEN,
        "core.file-open-log",
        |event, runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            let detail = event.detail.as_deref().unwrap_or("unknown");
            log.0.push(format!("file-open:{detail}"));
            Ok(())
        },
    )?;

    runtime.subscribe_hook(
        "user.after-open-scratch",
        "user.after-open-scratch-log",
        |event, runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            let detail = event.detail.as_deref().unwrap_or("scratch");
            log.0.push(format!("after-open-scratch:{detail}"));
            Ok(())
        },
    )?;

    runtime.register_command(
        "workspace.open-scratch",
        "Create a scratch buffer and emit a follow-up hook.",
        CommandSource::Core,
        move |runtime| {
            let scratch_buffer = runtime
                .model_mut()
                .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)
                .map_err(|error| error.to_string())?;

            runtime
                .emit_hook(
                    "user.after-open-scratch",
                    HookEvent::new()
                        .with_workspace(workspace_id)
                        .with_buffer(scratch_buffer)
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
    )?;

    runtime.register_command(
        "jobs.compile-self-check",
        "Runs a compilation-style external command and emits the compile-finish hook.",
        CommandSource::Core,
        move |runtime| {
            let manager = runtime
                .services()
                .get::<Mutex<JobManager>>()
                .ok_or_else(|| "job manager service missing".to_owned())?;
            let mut manager = manager
                .lock()
                .map_err(|_| "job manager lock poisoned".to_owned())?;
            let compilation = CompilationRunner::new()
                .run(
                    &mut manager,
                    JobSpec::compilation("rustc-version", "rustc", ["--version"]),
                )
                .map_err(|error| error.to_string())?;
            drop(manager);

            let workspace_id = runtime
                .model()
                .active_workspace_id()
                .map_err(|error| error.to_string())?;
            let compilation_buffer = runtime
                .model_mut()
                .create_buffer(
                    workspace_id,
                    "*compile-self-check*",
                    BufferKind::Compilation,
                    None,
                )
                .map_err(|error| error.to_string())?;

            runtime.services_mut().insert(CompilationState {
                success: compilation.succeeded(),
                transcript_bytes: compilation.transcript().len(),
                exit_code: compilation.job().exit_code(),
            });

            runtime
                .emit_hook(
                    builtins::COMPILATION_FINISH,
                    HookEvent::new()
                        .with_workspace(workspace_id)
                        .with_buffer(compilation_buffer)
                        .with_detail(if compilation.succeeded() {
                            "success"
                        } else {
                            "failure"
                        }),
                )
                .map_err(|error| error.to_string())?;

            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            log.0.push("command:jobs.compile-self-check".to_owned());
            Ok(())
        },
    )?;

    runtime.register_command(
        "terminal.run-self-check",
        "Runs a terminal-style external command and emits the terminal-exit hook.",
        CommandSource::Core,
        move |runtime| {
            let manager = runtime
                .services()
                .get::<Mutex<JobManager>>()
                .ok_or_else(|| "job manager service missing".to_owned())?;
            let mut manager = manager
                .lock()
                .map_err(|_| "job manager lock poisoned".to_owned())?;
            let session = TerminalSession::run(
                &mut manager,
                "Self Check Terminal",
                JobSpec::terminal("cargo-version", "cargo", ["--version"]),
            )
            .map_err(|error| error.to_string())?;
            drop(manager);

            let workspace_id = runtime
                .model()
                .active_workspace_id()
                .map_err(|error| error.to_string())?;
            let terminal_buffer = runtime
                .model_mut()
                .create_buffer(
                    workspace_id,
                    "*terminal-self-check*",
                    BufferKind::Terminal,
                    None,
                )
                .map_err(|error| error.to_string())?;

            runtime.services_mut().insert(TerminalState {
                success: session.transcript().succeeded(),
                line_count: session.transcript().line_count(),
                exit_code: session.transcript().exit_code(),
            });

            runtime
                .emit_hook(
                    builtins::TERMINAL_EXIT,
                    HookEvent::new()
                        .with_workspace(workspace_id)
                        .with_buffer(terminal_buffer)
                        .with_detail(if session.transcript().succeeded() {
                            "success"
                        } else {
                            "failure"
                        }),
                )
                .map_err(|error| error.to_string())?;

            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            log.0.push("command:terminal.run-self-check".to_owned());
            Ok(())
        },
    )?;

    runtime.register_command(
        "ui.command-palette",
        "Open a picker-style popup buffer.",
        CommandSource::Core,
        move |runtime| {
            let picker = PickerSession::new("Command Palette", command_palette_items(runtime))
                .with_result_limit(32);
            let palette_state = CommandPaletteState {
                visible_items: picker.match_count(),
                selected_command: picker
                    .selected()
                    .map(|matched| matched.item().id().to_owned()),
            };
            runtime.services_mut().insert(palette_state);

            let command_buffer = runtime
                .model_mut()
                .create_buffer(workspace_id, "*commands*", BufferKind::Picker, None)
                .map_err(|error| error.to_string())?;

            runtime
                .model_mut()
                .open_popup(
                    workspace_id,
                    "Command Palette",
                    vec![command_buffer],
                    command_buffer,
                )
                .map_err(|error| error.to_string())?;

            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            log.0.push("command:ui.command-palette".to_owned());
            Ok(())
        },
    )?;

    runtime.register_key_binding(
        "M-x",
        "ui.command-palette",
        KeymapScope::Global,
        CommandSource::Core,
    )?;

    let loaded_packages = load_auto_loaded_packages(&mut runtime, &user_library.packages())?;
    let mut command_palette_preview =
        PickerSession::new("Command Palette", command_palette_items(&runtime))
            .with_result_limit(16);
    command_palette_preview.set_query("term");
    let mut lsp_registry = LanguageServerRegistry::new();
    lsp_registry.register_all(user_library.language_servers())?;
    runtime.services_mut().insert(lsp_registry);
    runtime.services_mut().insert(LspState::default());
    let mut dap_registry = DebugAdapterRegistry::new();
    dap_registry.register_all(user_library.debug_adapters())?;
    runtime.services_mut().insert(dap_registry);
    runtime.services_mut().insert(DapState::default());
    let mut syntax_registry = SyntaxRegistry::new();
    syntax_registry.register_all(user_library.syntax_languages())?;
    let mut theme_registry = ThemeRegistry::new();
    theme_registry.register_all(user_library.themes())?;
    let rust_syntax = syntax_registry.highlight_buffer_for_extension(
        "rs",
        &TextBuffer::from_text(
            "fn main() {\n    let greeting = \"volt\";\n    println!(\"{greeting}\");\n}",
        ),
    )?;
    let resolved_theme_tokens = rust_syntax
        .highlight_spans
        .iter()
        .filter(|span| theme_registry.resolve(&span.theme_token).is_some())
        .count();
    let visible_rust_highlights = rust_syntax.visible_spans(0, 8).len();
    let directory_buffer = DirectoryBuffer::read(std::env::current_dir()?)?;
    let git_status = parse_status(
        "## main...origin/main [ahead 1]\nM  src/main.rs\n M README.md\n?? scratch.txt\n",
    )?;

    runtime.subscribe_hook(
        "lsp.server-start",
        "core.lsp-start-log",
        |event, runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            let detail = event.detail.as_deref().unwrap_or("unknown");
            log.0.push(format!("lsp-start:{detail}"));
            Ok(())
        },
    )?;

    runtime.subscribe_hook(
        "lsp.server-start",
        "core.lsp-session-plan",
        |event, runtime| {
            let server_id = event.detail.as_deref().unwrap_or("rust-analyzer");
            let registry = runtime
                .services()
                .get::<LanguageServerRegistry>()
                .ok_or_else(|| "lsp registry missing".to_owned())?;
            let session = registry
                .prepare_session(server_id, std::env::current_dir().ok())
                .map_err(|error| error.to_string())?;
            let state = runtime
                .services_mut()
                .get_mut::<LspState>()
                .ok_or_else(|| "lsp state missing".to_owned())?;
            state.sessions.push(session);
            Ok(())
        },
    )?;

    runtime.subscribe_hook(
        "lang.rust.attached",
        "core.lang-rust-log",
        |event, runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            let detail = event.detail.as_deref().unwrap_or("unknown");
            log.0.push(format!("lang-rust:{detail}"));
            Ok(())
        },
    )?;

    runtime.subscribe_hook(
        "dap.session-start",
        "core.dap-session-plan",
        |event, runtime| {
            let adapter_id = event.detail.as_deref().unwrap_or("codelldb");
            let registry = runtime
                .services()
                .get::<DebugAdapterRegistry>()
                .ok_or_else(|| "dap registry missing".to_owned())?;
            let plan = registry
                .prepare_session(
                    adapter_id,
                    DebugConfiguration::new("Debug volt", DebugRequestKind::Launch)
                        .with_target_program("target\\debug\\volt.exe")
                        .with_cwd(std::env::current_dir().map_err(|error| error.to_string())?)
                        .with_args(["--shell-hidden"]),
                )
                .map_err(|error| error.to_string())?;
            let state = runtime
                .services_mut()
                .get_mut::<DapState>()
                .ok_or_else(|| "dap state missing".to_owned())?;
            state.sessions.push(plan);
            Ok(())
        },
    )?;

    runtime.subscribe_hook(
        "dap.session-start",
        "core.dap-start-log",
        |event, runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            let detail = event.detail.as_deref().unwrap_or("unknown");
            log.0.push(format!("dap-start:{detail}"));
            Ok(())
        },
    )?;

    runtime.subscribe_hook(
        builtins::COMPILATION_FINISH,
        "core.compilation-finish-log",
        |event, runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            let detail = event.detail.as_deref().unwrap_or("unknown");
            log.0.push(format!("compile-finish:{detail}"));
            Ok(())
        },
    )?;

    runtime.subscribe_hook(
        builtins::TERMINAL_EXIT,
        "core.terminal-exit-log",
        |event, runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            let detail = event.detail.as_deref().unwrap_or("unknown");
            log.0.push(format!("terminal-exit:{detail}"));
            Ok(())
        },
    )?;

    runtime.emit_hook(
        builtins::WORKSPACE_OPEN,
        HookEvent::new()
            .with_window(window_id)
            .with_workspace(workspace_id),
    )?;
    runtime.emit_hook(
        builtins::STARTUP,
        HookEvent::new()
            .with_window(window_id)
            .with_workspace(workspace_id)
            .with_detail("foundation-bootstrap"),
    )?;
    runtime.execute_command("workspace.open-scratch")?;
    runtime.execute_command("jobs.compile-self-check")?;
    runtime.execute_key_binding(&KeymapScope::Global, "M-x")?;
    runtime.execute_key_binding(&KeymapScope::Global, "Ctrl+`")?;
    runtime.execute_command("terminal.run-self-check")?;
    runtime.execute_command("dap.start-codelldb")?;
    runtime.emit_hook(
        builtins::FILE_OPEN,
        HookEvent::new()
            .with_workspace(workspace_id)
            .with_detail(".rs"),
    )?;

    let descriptor = runtime.descriptor();
    let host = bootstrap();
    let workspace = runtime.model().workspace(workspace_id)?;
    let event_count = runtime
        .services()
        .get::<EventLog>()
        .map(|log| log.0.len())
        .unwrap_or_default();
    let palette_state = runtime.services().get::<CommandPaletteState>().cloned();
    let compilation_state = runtime.services().get::<CompilationState>().cloned();
    let terminal_state = runtime.services().get::<TerminalState>().cloned();
    let lsp_state = runtime.services().get::<LspState>().cloned();
    let dap_state = runtime.services().get::<DapState>().cloned();
    let lsp_server_count = runtime
        .services()
        .get::<LanguageServerRegistry>()
        .map(|registry| registry.len())
        .unwrap_or_default();
    let dap_adapter_count = runtime
        .services()
        .get::<DebugAdapterRegistry>()
        .map(|registry| registry.len())
        .unwrap_or_default();

    println!(
        concat!(
            "{name} ready: ws=`{workspace}`, buffers={buffers}, popups={popups}, ",
            "commands={commands}, hooks={hooks}, subs={subs}, keybindings={keybindings}, ",
            "services={services}, events={events}, auto_packages={packages}, ",
            "picker_items={picker_items}, picker_preview_matches={picker_preview_matches}, ",
            "picker_preview_top={picker_preview_top}, palette_selected={palette_selected}, ",
            "compile_success={compile_success}, compile_bytes={compile_bytes}, ",
            "terminal_success={terminal_success}, terminal_lines={terminal_lines}, ",
            "lsp_servers={lsp_servers}, lsp_sessions={lsp_sessions}, ",
            "dap_adapters={dap_adapters}, dap_sessions={dap_sessions}, ",
            "themes={themes}, themed_spans={themed_spans}, visible_highlights={visible_highlights}, ",
            "oil_entries={oil_entries}, ",
            "git_staged={git_staged}, git_untracked={git_untracked}, ",
            "syntax_languages={syntax_languages}, rust_highlights={rust_highlights}, ",
            "abi={abi}/{host_abi}"
        ),
        name = descriptor.application_name,
        workspace = workspace.name(),
        buffers = workspace.buffer_count(),
        popups = workspace.popup_count(),
        commands = runtime.commands().len(),
        hooks = runtime.hooks().hook_count(),
        subs = runtime.hooks().total_subscription_count(),
        keybindings = runtime.keymaps().len(),
        services = runtime.services().len(),
        events = event_count,
        packages = loaded_packages,
        picker_items = palette_state
            .as_ref()
            .map(|state| state.visible_items)
            .unwrap_or_default(),
        picker_preview_matches = command_palette_preview.match_count(),
        picker_preview_top = command_palette_preview
            .selected()
            .map(|matched| matched.item().id())
            .unwrap_or("none"),
        palette_selected = palette_state
            .as_ref()
            .and_then(|state| state.selected_command.as_deref())
            .unwrap_or("none"),
        compile_success = compilation_state
            .as_ref()
            .map(|state| state.success)
            .unwrap_or(false),
        compile_bytes = compilation_state
            .as_ref()
            .map(|state| state.transcript_bytes)
            .unwrap_or_default(),
        terminal_success = terminal_state
            .as_ref()
            .map(|state| state.success)
            .unwrap_or(false),
        terminal_lines = terminal_state
            .as_ref()
            .map(|state| state.line_count)
            .unwrap_or_default(),
        lsp_servers = lsp_server_count,
        lsp_sessions = lsp_state
            .as_ref()
            .map(|state| state.sessions.len())
            .unwrap_or_default(),
        dap_adapters = dap_adapter_count,
        dap_sessions = dap_state
            .as_ref()
            .map(|state| state.sessions.len())
            .unwrap_or_default(),
        themes = theme_registry.len(),
        themed_spans = resolved_theme_tokens,
        visible_highlights = visible_rust_highlights,
        oil_entries = directory_buffer.entries().len(),
        git_staged = git_status.staged().len(),
        git_untracked = git_status.untracked().len(),
        syntax_languages = syntax_registry.len(),
        rust_highlights = rust_syntax.highlight_count(),
        abi = descriptor.plugin_abi,
        host_abi = host.plugin_abi,
    );

    Ok(())
}

fn parse_launch_options(args: impl IntoIterator<Item = String>) -> Result<LaunchOptions, String> {
    let mut mode = LaunchMode::ShellDemo;
    let mut explicit_mode = false;
    let mut profile_input_latency = false;

    for arg in args {
        match arg.as_str() {
            "--shell-demo" => {
                if explicit_mode {
                    return Err("shell mode was provided more than once".to_owned());
                }
                explicit_mode = true;
                mode = LaunchMode::ShellDemo;
            }
            "--shell-hidden" => {
                if explicit_mode {
                    return Err("shell mode was provided more than once".to_owned());
                }
                explicit_mode = true;
                mode = LaunchMode::ShellHidden;
            }
            "--bootstrap-demo" => {
                if explicit_mode {
                    return Err("shell mode was provided more than once".to_owned());
                }
                explicit_mode = true;
                mode = LaunchMode::BootstrapDemo;
            }
            "--profile-input" | "--profile-typing" => {
                profile_input_latency = true;
            }
            other => {
                return Err(format!(
                    "unknown mode `{other}`; expected `--shell-demo`, `--shell-hidden`, `--bootstrap-demo`, `--profile-input`, or `--profile-typing`"
                ));
            }
        }
    }

    Ok(LaunchOptions {
        mode,
        profile_input_latency,
    })
}

fn print_shell_summary(prefix: &str, summary: &editor_sdl::ShellSummary) {
    print!(
        "{prefix}: frames={}, panes={}, popup_visible={}, backend={:?}, renderer={}, font={}",
        summary.frames_rendered,
        summary.pane_count,
        summary.popup_visible,
        summary.render_backend,
        summary.renderer_name,
        summary.font_path,
    );
    if let Some(profile) = summary.typing_profile.as_ref() {
        print!(
            ", input_profile={}, profiled_frames={}, input_frames={}, slowest_frame={}",
            profile.log_path,
            profile.frames_captured,
            profile.input_frames_captured,
            format_micros_as_millis(profile.slowest_frame_micros),
        );
    }
    println!();
}

fn format_micros_as_millis(micros: u128) -> String {
    format!("{}.{:03}ms", micros / 1_000, micros % 1_000)
}

fn command_palette_items(runtime: &EditorRuntime) -> Vec<PickerItem> {
    runtime
        .commands()
        .definitions()
        .into_iter()
        .map(|definition| {
            PickerItem::new(
                definition.name(),
                definition.name(),
                definition.description(),
                Some(definition.description()),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_launch_options_defaults_to_shell_demo() {
        let options = parse_launch_options(Vec::<String>::new())
            .expect("empty args should default to shell demo mode");
        assert_eq!(
            options,
            LaunchOptions {
                mode: LaunchMode::ShellDemo,
                profile_input_latency: false,
            }
        );
    }

    #[test]
    fn parse_launch_options_accepts_profile_alias() {
        let options =
            parse_launch_options(["--shell-hidden".to_owned(), "--profile-typing".to_owned()])
                .expect("profile alias should enable typing profile mode");
        assert_eq!(
            options,
            LaunchOptions {
                mode: LaunchMode::ShellHidden,
                profile_input_latency: true,
            }
        );
    }

    #[test]
    fn parse_launch_options_rejects_multiple_modes() {
        let error =
            parse_launch_options(["--shell-demo".to_owned(), "--bootstrap-demo".to_owned()])
                .expect_err("multiple launch modes should be rejected");
        assert!(error.contains("more than once"));
    }

    #[test]
    fn user_library_candidates_prefer_env_then_executable_directory() {
        let candidates = user_library_candidates(
            Some(Path::new("/tmp/volt/bin/volt")),
            Some("/tmp/custom/user/libuser.so"),
        );
        assert_eq!(
            candidates,
            vec![
                PathBuf::from("/tmp/custom/user/libuser.so"),
                UserLibraryModuleRef::get_library_path(Path::new("/tmp/volt/bin")),
            ]
        );
    }

    #[test]
    fn dynamic_user_library_can_wrap_exported_module() {
        let library = DynamicUserLibrary::new(user::user_library_module());
        assert!(!library.packages().is_empty());
        assert!(!library.themes().is_empty());
        assert!(!library.icon_symbols().is_empty());
    }
}
