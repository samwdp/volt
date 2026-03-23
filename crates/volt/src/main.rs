#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use std::{error::Error, sync::Mutex};

use editor_buffer::TextBuffer;
use editor_core::{BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope, builtins};
use editor_dap::{DebugAdapterRegistry, DebugConfiguration, DebugRequestKind, DebugSessionPlan};
use editor_fs::DirectoryBuffer;
use editor_git::parse_status;
use editor_jobs::{CompilationRunner, JobManager, JobSpec};
use editor_lsp::{LanguageServerRegistry, LanguageServerSession};
use editor_picker::{PickerItem, PickerSession};
use editor_plugin_host::{bootstrap, load_auto_loaded_packages};
use editor_sdl::{ShellConfig, run_demo_shell};
use editor_syntax::SyntaxRegistry;
use editor_terminal::TerminalSession;
use editor_theme::ThemeRegistry;

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("--shell-demo") => {
            let summary = run_demo_shell(ShellConfig::default())?;
            println!(
                "volt shell demo: frames={}, panes={}, popup_visible={}, backend={:?}, renderer={}, font={}",
                summary.frames_rendered,
                summary.pane_count,
                summary.popup_visible,
                summary.render_backend,
                summary.renderer_name,
                summary.font_path,
            );
            return Ok(());
        }
        Some("--shell-hidden") => {
            let summary = run_demo_shell(ShellConfig {
                hidden: true,
                frame_limit: Some(1),
                ..ShellConfig::default()
            })?;
            println!(
                "volt hidden shell smoke test: frames={}, panes={}, popup_visible={}, backend={:?}, renderer={}, font={}",
                summary.frames_rendered,
                summary.pane_count,
                summary.popup_visible,
                summary.render_backend,
                summary.renderer_name,
                summary.font_path,
            );
            return Ok(());
        }
        Some("--bootstrap-demo") => {}
        Some(mode) => {
            return Err(format!(
                "unknown mode `{mode}`; expected `--shell-demo`, `--shell-hidden`, or `--bootstrap-demo`"
            )
            .into())
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

    let loaded_packages = load_auto_loaded_packages(&mut runtime, &user::packages())?;
    let mut command_palette_preview =
        PickerSession::new("Command Palette", command_palette_items(&runtime))
            .with_result_limit(16);
    command_palette_preview.set_query("term");
    let mut lsp_registry = LanguageServerRegistry::new();
    lsp_registry.register_all(user::language_servers())?;
    runtime.services_mut().insert(lsp_registry);
    runtime.services_mut().insert(LspState::default());
    let mut dap_registry = DebugAdapterRegistry::new();
    dap_registry.register_all(user::debug_adapters())?;
    runtime.services_mut().insert(dap_registry);
    runtime.services_mut().insert(DapState::default());
    let mut syntax_registry = SyntaxRegistry::new();
    syntax_registry.register_all(user::syntax_languages())?;
    let mut theme_registry = ThemeRegistry::new();
    theme_registry.register_all(user::themes())?;
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
