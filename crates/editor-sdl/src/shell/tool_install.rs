use super::{
    command_stream::{
        StreamedCommandExitAction, StreamedCommandSpec, continue_streamed_command_popup,
        open_streamed_command_popup,
    },
    *,
};
use editor_tool_install::{
    InstallCommand, InstallPlan, InstallRecipe, ProgramLocation, ToolKind as InstallToolKind,
    failed_install_key, finalize_install, locate_program, prepare_install,
};
use std::collections::VecDeque;

const LSP_INSTALL_POPUP_TITLE: &str = "Language Server Install";
const DAP_INSTALL_POPUP_TITLE: &str = "Debug Adapter Install";

#[derive(Debug, Clone)]
pub(super) struct QueuedToolInstall {
    kind: InstallToolKind,
    spec_id: String,
    program: String,
    recipe: InstallRecipe,
}

#[derive(Debug)]
pub(super) enum ToolInstallFinish {
    None,
    StartLspIfActiveWouldAutoStart {
        spec_id: String,
    },
    ResumeLspStart {
        preferred_server_id: Option<String>,
    },
    ResumeDapStart {
        adapter_id: String,
        configuration: DebugConfiguration,
    },
}

#[derive(Debug)]
pub(super) struct ToolInstallState {
    plan: InstallPlan,
    remaining_commands: VecDeque<InstallCommand>,
    remaining_specs: VecDeque<QueuedToolInstall>,
    after: ToolInstallFinish,
}

pub(super) fn handle_lsp_install_hook(
    runtime: &mut EditorRuntime,
    spec_id: Option<&str>,
) -> Result<(), String> {
    match spec_id.map(str::trim).filter(|id| !id.is_empty()) {
        Some(spec_id) => install_language_server_by_id(runtime, spec_id),
        None => open_language_server_install_picker(runtime),
    }
}

pub(super) fn handle_dap_install_hook(
    runtime: &mut EditorRuntime,
    spec_id: Option<&str>,
) -> Result<(), String> {
    match spec_id.map(str::trim).filter(|id| !id.is_empty()) {
        Some(spec_id) => install_debug_adapter_by_id(runtime, spec_id),
        None => open_debug_adapter_install_picker(runtime),
    }
}

pub(super) fn install_language_server_by_id(
    runtime: &mut EditorRuntime,
    spec_id: &str,
) -> Result<(), String> {
    let lsp_client = lsp_client_manager(runtime)?;
    let spec = lsp_client
        .registry()
        .server(spec_id)
        .ok_or_else(|| format!("language server `{spec_id}` is not registered"))?;
    let recipe = spec
        .install_recipe()
        .cloned()
        .ok_or_else(|| format!("language server `{spec_id}` has no Install Recipe"))?;
    begin_explicit_install(
        runtime,
        InstallToolKind::LanguageServer,
        spec.id(),
        spec.program(),
        recipe,
        ToolInstallFinish::StartLspIfActiveWouldAutoStart {
            spec_id: spec.id().to_owned(),
        },
    )
}

pub(super) fn install_debug_adapter_by_id(
    runtime: &mut EditorRuntime,
    spec_id: &str,
) -> Result<(), String> {
    let dap_client = dap_client_manager(runtime)?;
    let spec = dap_client
        .registry()
        .adapter(spec_id)
        .ok_or_else(|| format!("debug adapter `{spec_id}` is not registered"))?;
    let recipe = spec
        .install_recipe()
        .cloned()
        .ok_or_else(|| format!("debug adapter `{spec_id}` has no Install Recipe"))?;
    begin_explicit_install(
        runtime,
        InstallToolKind::DebugAdapter,
        spec.id(),
        spec.program(),
        recipe,
        ToolInstallFinish::None,
    )
}

pub(super) fn open_language_server_install_picker(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let lsp_client = lsp_client_manager(runtime)?;
    let entries = lsp_client
        .registry()
        .servers()
        .filter(|spec| spec.install_recipe().is_some())
        .map(|spec| {
            install_picker_entry(
                spec.id(),
                spec.program(),
                PickerAction::InstallLanguageServer(spec.id().to_owned()),
            )
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Err("no Language Servers declare an Install Recipe".to_owned());
    }
    shell_ui_mut(runtime)?.set_picker(
        PickerOverlay::from_entries("Install Language Server", entries)
            .with_result_order(PickerResultOrder::Source),
    );
    Ok(())
}

pub(super) fn open_debug_adapter_install_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let dap_client = dap_client_manager(runtime)?;
    let entries = dap_client
        .registry()
        .adapters()
        .filter(|spec| spec.install_recipe().is_some())
        .map(|spec| {
            install_picker_entry(
                spec.id(),
                spec.program(),
                PickerAction::InstallDebugAdapter(spec.id().to_owned()),
            )
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Err("no Debug Adapters declare an Install Recipe".to_owned());
    }
    shell_ui_mut(runtime)?.set_picker(
        PickerOverlay::from_entries("Install Debug Adapter", entries)
            .with_result_order(PickerResultOrder::Source),
    );
    Ok(())
}

pub(super) fn install_picker_label(available: bool, spec_id: &str) -> String {
    let icon = if available {
        editor_icons::symbols::fa::FA_CHECK
    } else {
        editor_icons::symbols::fa::FA_PLUS
    };
    format!("{icon} {spec_id}")
}

pub(super) fn queue_missing_language_server_installs(
    runtime: &mut EditorRuntime,
    manager: &LspClientManager,
    context: &ActiveLspBufferContext,
    preferred_server_id: Option<&str>,
) -> Result<bool, String> {
    let targets = language_servers_for_start(manager, context, preferred_server_id);
    let failed = shell_ui(runtime)?.failed_tool_installs.clone();
    let mut queued = VecDeque::new();
    for spec in targets {
        let Some(recipe) = spec.install_recipe() else {
            continue;
        };
        if !matches!(locate_program(spec.program()), ProgramLocation::Missing) {
            continue;
        }
        let key = failed_install_key(InstallToolKind::LanguageServer, spec.id());
        if failed.contains(&key) {
            continue;
        }
        queued.push_back(QueuedToolInstall {
            kind: InstallToolKind::LanguageServer,
            spec_id: spec.id().to_owned(),
            program: spec.program().to_owned(),
            recipe: recipe.clone(),
        });
    }
    if queued.is_empty() {
        return Ok(false);
    }
    start_queued_installs(
        runtime,
        None,
        queued,
        ToolInstallFinish::ResumeLspStart {
            preferred_server_id: preferred_server_id.map(str::to_owned),
        },
    )?;
    Ok(true)
}

pub(super) fn install_debug_adapter_then_start(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: DebugConfiguration,
) -> Result<bool, String> {
    let dap_client = dap_client_manager(runtime)?;
    let Some(spec) = dap_client.registry().adapter(adapter_id) else {
        return Ok(false);
    };
    let Some(recipe) = spec.install_recipe() else {
        return Ok(false);
    };
    if !matches!(locate_program(spec.program()), ProgramLocation::Missing) {
        return Ok(false);
    }
    let key = failed_install_key(InstallToolKind::DebugAdapter, spec.id());
    if shell_ui(runtime)?.failed_tool_installs.contains(&key) {
        return Ok(false);
    }
    let queued = VecDeque::from([QueuedToolInstall {
        kind: InstallToolKind::DebugAdapter,
        spec_id: spec.id().to_owned(),
        program: spec.program().to_owned(),
        recipe: recipe.clone(),
    }]);
    start_queued_installs(
        runtime,
        None,
        queued,
        ToolInstallFinish::ResumeDapStart {
            adapter_id: adapter_id.to_owned(),
            configuration,
        },
    )?;
    Ok(true)
}

pub(super) fn continue_tool_install(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mut state: ToolInstallState,
    success: bool,
) -> Result<(), String> {
    if !success {
        return fail_tool_install(runtime, buffer_id, state);
    }
    if let Some(next) = state.remaining_commands.pop_front() {
        return continue_streamed_command_popup(
            runtime,
            buffer_id,
            streamed_tool_install_command(
                popup_title(state.plan.kind()),
                tool_install_buffer_name(state.plan.kind(), state.plan.spec_id()),
                &next,
                StreamedCommandExitAction::ContinueToolInstall(Box::new(state)),
                false,
                true,
            ),
        );
    }
    if let Err(error) = finalize_install(&state.plan) {
        return fail_tool_install_with_message(runtime, buffer_id, state, error.to_string());
    }
    let key = failed_install_key(state.plan.kind(), state.plan.spec_id());
    shell_ui_mut(runtime)?.failed_tool_installs.remove(&key);
    notify_tool_install(
        runtime,
        &key,
        NotificationSeverity::Success,
        "Install complete",
        vec![format!("Installed `{}`.", state.plan.spec_id())],
    )?;
    start_next_or_finish(runtime, buffer_id, state)
}

fn begin_explicit_install(
    runtime: &mut EditorRuntime,
    kind: InstallToolKind,
    spec_id: &str,
    program: &str,
    recipe: InstallRecipe,
    after: ToolInstallFinish,
) -> Result<(), String> {
    match locate_program(program) {
        ProgramLocation::UserPath(path) => {
            return notify_tool_install(
                runtime,
                &failed_install_key(kind, spec_id),
                NotificationSeverity::Info,
                "Already on PATH",
                vec![format!(
                    "`{spec_id}` is already visible at `{}`.",
                    path.display()
                )],
            );
        }
        ProgramLocation::VoltInstall(_) | ProgramLocation::Missing => {}
    }
    let key = failed_install_key(kind, spec_id);
    shell_ui_mut(runtime)?.failed_tool_installs.remove(&key);
    start_queued_installs(
        runtime,
        None,
        VecDeque::from([QueuedToolInstall {
            kind,
            spec_id: spec_id.to_owned(),
            program: program.to_owned(),
            recipe,
        }]),
        after,
    )
}

fn start_queued_installs(
    runtime: &mut EditorRuntime,
    buffer_id: Option<BufferId>,
    mut queued: VecDeque<QueuedToolInstall>,
    after: ToolInstallFinish,
) -> Result<(), String> {
    loop {
        let Some(next) = queued.pop_front() else {
            if let Some(buffer_id) = buffer_id {
                close_popup_buffer_and_restore_focus(runtime, buffer_id)?;
            }
            return apply_tool_install_finish(runtime, after);
        };
        match prepare_install(next.kind, &next.spec_id, &next.program, &next.recipe) {
            Ok(plan) => {
                return launch_tool_install_plan(runtime, buffer_id, plan, queued, after);
            }
            Err(error) => {
                record_failed_install(runtime, next.kind, &next.spec_id);
                notify_tool_install(
                    runtime,
                    &failed_install_key(next.kind, &next.spec_id),
                    NotificationSeverity::Error,
                    "Install failed",
                    vec![error.to_string()],
                )?;
                if !matches!(&after, ToolInstallFinish::ResumeLspStart { .. }) {
                    return Ok(());
                }
            }
        }
    }
}

fn launch_tool_install_plan(
    runtime: &mut EditorRuntime,
    buffer_id: Option<BufferId>,
    plan: InstallPlan,
    remaining_specs: VecDeque<QueuedToolInstall>,
    after: ToolInstallFinish,
) -> Result<(), String> {
    let mut remaining_commands: VecDeque<InstallCommand> =
        plan.commands().iter().cloned().collect();
    let Some(first) = remaining_commands.pop_front() else {
        finalize_install(&plan).map_err(|error| error.to_string())?;
        return start_queued_installs(runtime, buffer_id, remaining_specs, after);
    };
    let kind = plan.kind();
    let spec_id = plan.spec_id().to_owned();
    let spec = streamed_tool_install_command(
        popup_title(kind),
        tool_install_buffer_name(kind, &spec_id),
        &first,
        StreamedCommandExitAction::ContinueToolInstall(Box::new(ToolInstallState {
            plan,
            remaining_commands,
            remaining_specs,
            after,
        })),
        false,
        true,
    );
    if let Some(buffer_id) = buffer_id {
        continue_streamed_command_popup(runtime, buffer_id, spec)
    } else {
        open_streamed_command_popup(runtime, spec).map(|_| ())
    }
}

fn start_next_or_finish(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    state: ToolInstallState,
) -> Result<(), String> {
    start_queued_installs(runtime, Some(buffer_id), state.remaining_specs, state.after)
}

fn apply_tool_install_finish(
    runtime: &mut EditorRuntime,
    after: ToolInstallFinish,
) -> Result<(), String> {
    match after {
        ToolInstallFinish::None => Ok(()),
        ToolInstallFinish::StartLspIfActiveWouldAutoStart { spec_id } => {
            if language_server_would_auto_start(runtime, &spec_id)?
                && let Err(error) = start_lsp_for_active_buffer(runtime, Some(&spec_id))
            {
                notify_tool_install(
                    runtime,
                    "lsp.install-start",
                    NotificationSeverity::Warning,
                    "Language Server start failed",
                    vec![error],
                )?;
            }
            Ok(())
        }
        ToolInstallFinish::ResumeLspStart {
            preferred_server_id,
        } => {
            if let Err(error) = start_lsp_for_active_buffer(runtime, preferred_server_id.as_deref())
            {
                notify_tool_install(
                    runtime,
                    "lsp.install-start",
                    NotificationSeverity::Warning,
                    "Language Server start failed",
                    vec![error],
                )?;
            }
            Ok(())
        }
        ToolInstallFinish::ResumeDapStart {
            adapter_id,
            configuration,
        } => {
            let result = finish_dap_session_start(runtime, &adapter_id, configuration);
            report_dap_result(runtime, "DAP start failed", result)
        }
    }
}

fn fail_tool_install(
    runtime: &mut EditorRuntime,
    _buffer_id: BufferId,
    state: ToolInstallState,
) -> Result<(), String> {
    record_failed_install(runtime, state.plan.kind(), state.plan.spec_id());
    Ok(())
}

fn fail_tool_install_with_message(
    runtime: &mut EditorRuntime,
    _buffer_id: BufferId,
    state: ToolInstallState,
    message: String,
) -> Result<(), String> {
    record_failed_install(runtime, state.plan.kind(), state.plan.spec_id());
    notify_tool_install(
        runtime,
        &failed_install_key(state.plan.kind(), state.plan.spec_id()),
        NotificationSeverity::Error,
        "Install failed",
        vec![format!("`{}`: {message}", state.plan.spec_id())],
    )
}

fn record_failed_install(runtime: &mut EditorRuntime, kind: InstallToolKind, spec_id: &str) {
    if let Ok(ui) = shell_ui_mut(runtime) {
        ui.failed_tool_installs
            .insert(failed_install_key(kind, spec_id));
    }
}

fn notify_tool_install(
    runtime: &mut EditorRuntime,
    key: &str,
    severity: NotificationSeverity,
    title: &str,
    body_lines: Vec<String>,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.apply_notification(
        NotificationUpdate {
            key: key.to_owned(),
            severity,
            title: title.to_owned(),
            body_lines,
            progress: None,
            active: false,
            action: None,
            workspace_id: None,
        },
        Instant::now(),
    );
    Ok(())
}

fn install_picker_entry(spec_id: &str, program: &str, action: PickerAction) -> PickerEntry {
    let available = !matches!(locate_program(program), ProgramLocation::Missing);
    PickerEntry {
        item: PickerItem::new(
            spec_id,
            install_picker_label(available, spec_id),
            String::new(),
            None::<String>,
        ),
        action,
        quickfix: None,
    }
}

fn language_servers_for_start<'a>(
    manager: &'a LspClientManager,
    context: &ActiveLspBufferContext,
    preferred_server_id: Option<&str>,
) -> Vec<&'a editor_lsp::LanguageServerSpec> {
    if let Some(server_id) = preferred_server_id {
        return manager.registry().server(server_id).into_iter().collect();
    }
    manager
        .registry()
        .default_enabled_servers_for_path_in_workspace(&context.path, context.root.as_deref())
}

fn language_server_would_auto_start(
    runtime: &EditorRuntime,
    spec_id: &str,
) -> Result<bool, String> {
    let Ok(context) = active_lsp_buffer_context(runtime) else {
        return Ok(false);
    };
    let lsp_client = lsp_client_manager(runtime)?;
    Ok(lsp_client
        .registry()
        .default_enabled_servers_for_path_in_workspace(&context.path, context.root.as_deref())
        .into_iter()
        .any(|server| server.id() == spec_id))
}

fn lsp_client_manager(runtime: &EditorRuntime) -> Result<Arc<LspClientManager>, String> {
    runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .cloned()
        .ok_or_else(|| "LSP client manager service missing".to_owned())
}

fn popup_title(kind: InstallToolKind) -> &'static str {
    match kind {
        InstallToolKind::LanguageServer => LSP_INSTALL_POPUP_TITLE,
        InstallToolKind::DebugAdapter => DAP_INSTALL_POPUP_TITLE,
    }
}

fn tool_install_buffer_name(kind: InstallToolKind, spec_id: &str) -> String {
    match kind {
        InstallToolKind::LanguageServer => format!("*lsp.install {spec_id}*"),
        InstallToolKind::DebugAdapter => format!("*dap.install {spec_id}*"),
    }
}

fn streamed_tool_install_command(
    popup_title: &str,
    buffer_name: String,
    command: &InstallCommand,
    on_exit: StreamedCommandExitAction,
    notify_on_success: bool,
    notify_on_failure: bool,
) -> StreamedCommandSpec {
    StreamedCommandSpec {
        popup_title: popup_title.to_owned(),
        buffer_name,
        command_label: command.label().to_owned(),
        program: command.program().to_owned(),
        args: command.args().to_vec(),
        env: command.env().to_vec(),
        cwd: command.cwd().to_path_buf(),
        on_exit,
        notify_on_success,
        notify_on_failure,
    }
}
