fn register_dap_hooks(runtime: &mut EditorRuntime) -> Result<(), String> {
    if runtime.hooks().contains(HOOK_DAP_START) {
        runtime
            .subscribe_hook(
                HOOK_DAP_START,
                "shell.start-dap-session",
                |event, runtime| {
                    let result = start_dap_for_active_workspace(runtime, event.detail.as_deref());
                    report_dap_result(runtime, "DAP start failed", result)
                },
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_START_LAST) {
        runtime
            .subscribe_hook(HOOK_DAP_START_LAST, "shell.start-dap-last", |_, runtime| {
                let result = start_dap_last(runtime);
                report_dap_result(runtime, "DAP start failed", result)
            })
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_START_RECENT) {
        runtime
            .subscribe_hook(
                HOOK_DAP_START_RECENT,
                "shell.start-dap-recent",
                |_, runtime| {
                    let result = open_dap_start_recent_picker(runtime);
                    report_dap_result(runtime, "DAP start failed", result)
                },
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_STOP) {
        runtime
            .subscribe_hook(HOOK_DAP_STOP, "shell.stop-dap-session", |_, runtime| {
                stop_dap_for_active_workspace(runtime)
            })
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_RESTART) {
        runtime
            .subscribe_hook(
                HOOK_DAP_RESTART,
                "shell.restart-dap-session",
                |_, runtime| restart_dap_for_active_workspace(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_CONTINUE) {
        runtime
            .subscribe_hook(
                HOOK_DAP_CONTINUE,
                "shell.continue-dap-session",
                |_, runtime| dap_control_for_active_workspace(runtime, DapControl::Continue),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_PAUSE) {
        runtime
            .subscribe_hook(HOOK_DAP_PAUSE, "shell.pause-dap-session", |_, runtime| {
                dap_control_for_active_workspace(runtime, DapControl::Pause)
            })
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_STEP) {
        runtime
            .subscribe_hook(HOOK_DAP_STEP, "shell.step-dap-session", |_, runtime| {
                dap_control_for_active_workspace(runtime, DapControl::StepOver)
            })
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_STEP_INTO) {
        runtime
            .subscribe_hook(
                HOOK_DAP_STEP_INTO,
                "shell.step-into-dap-session",
                |_, runtime| dap_control_for_active_workspace(runtime, DapControl::StepInto),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_STEP_OUT) {
        runtime
            .subscribe_hook(
                HOOK_DAP_STEP_OUT,
                "shell.step-out-dap-session",
                |_, runtime| dap_control_for_active_workspace(runtime, DapControl::StepOut),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_LOG) {
        runtime
            .subscribe_hook(HOOK_DAP_LOG, "shell.open-dap-log", |_, runtime| {
                open_dap_log_buffer(runtime)
            })
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_TOGGLE_BREAKPOINT) {
        runtime
            .subscribe_hook(
                HOOK_DAP_TOGGLE_BREAKPOINT,
                "shell.toggle-dap-breakpoint",
                |_, runtime| toggle_dap_breakpoint_at_cursor(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_DELETE_BREAKPOINT) {
        runtime
            .subscribe_hook(
                HOOK_DAP_DELETE_BREAKPOINT,
                "shell.delete-dap-breakpoint",
                |_, runtime| delete_dap_breakpoint_at_cursor(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_OPEN_BREAKPOINTS) {
        runtime
            .subscribe_hook(
                HOOK_DAP_OPEN_BREAKPOINTS,
                "shell.open-dap-breakpoints",
                |_, runtime| open_dap_breakpoints_buffer(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_EXPRESSIONS_ADD) {
        runtime
            .subscribe_hook(
                HOOK_DAP_EXPRESSIONS_ADD,
                "shell.dap-expressions-add",
                |event, runtime| dap_expressions_add(runtime, event.detail.as_deref()),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_EXPRESSIONS_REMOVE) {
        runtime
            .subscribe_hook(
                HOOK_DAP_EXPRESSIONS_REMOVE,
                "shell.dap-expressions-remove",
                |event, runtime| dap_expressions_remove(runtime, event.detail.as_deref()),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_EVAL) {
        runtime
            .subscribe_hook(HOOK_DAP_EVAL, "shell.dap-eval", |event, runtime| {
                dap_eval(runtime, event.detail.as_deref())
            })
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_EVAL_AT_POINT) {
        runtime
            .subscribe_hook(
                HOOK_DAP_EVAL_AT_POINT,
                "shell.dap-eval-at-point",
                |_, runtime| dap_eval_at_point(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_REPL) {
        runtime
            .subscribe_hook(HOOK_DAP_REPL, "shell.dap-repl", |_, runtime| {
                open_dap_repl(runtime)
            })
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_SWITCH_THREAD) {
        runtime
            .subscribe_hook(
                HOOK_DAP_SWITCH_THREAD,
                "shell.dap-switch-thread",
                |_, runtime| open_dap_switch_thread_picker(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_SWITCH_STACK_FRAME) {
        runtime
            .subscribe_hook(
                HOOK_DAP_SWITCH_STACK_FRAME,
                "shell.dap-switch-stack-frame",
                |_, runtime| open_dap_switch_stack_frame_picker(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_BREAKPOINT_CONDITION) {
        runtime
            .subscribe_hook(
                HOOK_DAP_BREAKPOINT_CONDITION,
                "shell.dap-breakpoint-condition",
                |event, runtime| {
                    dap_breakpoint_extra_prompt(
                        runtime,
                        DapBreakpointExtraKind::Condition,
                        event.detail.as_deref(),
                    )
                },
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_BREAKPOINT_HIT_CONDITION) {
        runtime
            .subscribe_hook(
                HOOK_DAP_BREAKPOINT_HIT_CONDITION,
                "shell.dap-breakpoint-hit-condition",
                |event, runtime| {
                    dap_breakpoint_extra_prompt(
                        runtime,
                        DapBreakpointExtraKind::HitCondition,
                        event.detail.as_deref(),
                    )
                },
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_BREAKPOINT_LOG_MESSAGE) {
        runtime
            .subscribe_hook(
                HOOK_DAP_BREAKPOINT_LOG_MESSAGE,
                "shell.dap-breakpoint-log-message",
                |event, runtime| {
                    dap_breakpoint_extra_prompt(
                        runtime,
                        DapBreakpointExtraKind::LogMessage,
                        event.detail.as_deref(),
                    )
                },
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_TOGGLE_VARIABLE) {
        runtime
            .subscribe_hook(
                HOOK_DAP_TOGGLE_VARIABLE,
                "shell.dap-toggle-variable",
                |_, runtime| dap_toggle_variable(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_GOTO_BREAKPOINT) {
        runtime
            .subscribe_hook(
                HOOK_DAP_GOTO_BREAKPOINT,
                "shell.dap-goto-breakpoint",
                |_, runtime| dap_goto_breakpoint(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_INSTALL) {
        runtime
            .subscribe_hook(
                HOOK_DAP_INSTALL,
                "shell.install-dap-adapter",
                |event, runtime| {
                    tool_install::handle_dap_install_hook(runtime, event.detail.as_deref())
                },
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DapControl {
    Continue,
    Pause,
    StepOver,
    StepInto,
    StepOut,
}

#[derive(Debug, Clone)]
struct PendingDapStartPrompt {
    adapter_id: String,
    configuration: DebugConfiguration,
    ask_heuristic_compile: bool,
}

const DAP_PROGRAM_PROMPT_ID: &str = "dap-program";
const DAP_PROCESS_PROMPT_ID: &str = "dap-process";
const DAP_EXPRESSION_ADD_PROMPT_ID: &str = "dap-expression-add";
const DAP_EVAL_PROMPT_ID: &str = "dap-eval";
const DAP_REPL_PROMPT_ID: &str = "dap-repl";
const DAP_BP_CONDITION_PROMPT_ID: &str = "dap-bp-condition";
const DAP_BP_HIT_CONDITION_PROMPT_ID: &str = "dap-bp-hit-condition";
const DAP_BP_LOG_MESSAGE_PROMPT_ID: &str = "dap-bp-log-message";

fn start_dap_for_active_workspace(
    runtime: &mut EditorRuntime,
    preferred_adapter_id: Option<&str>,
) -> Result<(), String> {
    let ctx = dap_start_context(runtime)?;
    let preferred = preferred_adapter_id
        .map(str::to_owned)
        .or_else(|| ctx.preferred_from_config.clone());

    let adapter_id = match preferred {
        Some(adapter_id) => adapter_id,
        None => {
            let extension = ctx.extension.as_deref().ok_or_else(|| {
                "dap.start needs an adapter id, open file extension, or explicit Debug Configuration"
                    .to_owned()
            })?;
            let dap_client = dap_client_manager(runtime)?;
            let adapters = dap_client
                .registry()
                .enabled_adapters_for_extension(extension);
            match adapters.as_slice() {
                [] => {
                    return Err(format!(
                        "no enabled Debug Adapter registered for `{extension}`"
                    ));
                }
                [only] => only.id().to_owned(),
                many => {
                    open_dap_adapter_picker(runtime, many, &ctx)?;
                    return Ok(());
                }
            }
        }
    };

    resolve_dap_configuration_then_start(runtime, &adapter_id, &ctx)
}

fn start_dap_last(runtime: &mut EditorRuntime) -> Result<(), String> {
    let dap_client = dap_client_manager(runtime)?;
    let Some(record) = dap_client.last_start().map_err(|error| error.to_string())? else {
        return Err("no previous Debug Configuration to replay".to_owned());
    };
    continue_dap_start(
        runtime,
        record.adapter_id(),
        record.configuration().clone(),
        false,
    )
}

fn open_dap_start_recent_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let dap_client = dap_client_manager(runtime)?;
    let recent = dap_client
        .recent_starts()
        .map_err(|error| error.to_string())?;
    if recent.is_empty() {
        return Err("no recent Debug Configurations".to_owned());
    }
    let entries = recent
        .into_iter()
        .enumerate()
        .map(|(index, record)| {
            let candidate = record.to_candidate();
            PickerEntry {
                item: PickerItem::new(
                    format!("dap-recent:{index}"),
                    candidate.picker_label(),
                    candidate.picker_detail(),
                    None::<String>,
                ),
                action: PickerAction::StartDapSession {
                    adapter_id: record.adapter_id().to_owned(),
                    configuration: record.configuration().clone(),
                    ask_heuristic_compile: false,
                },
                quickfix: None,
            }
        })
        .collect();
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries(
        "Recent Debug Configurations",
        entries,
    ));
    Ok(())
}

fn open_dap_adapter_picker(
    runtime: &mut EditorRuntime,
    adapters: &[&editor_dap::DebugAdapterSpec],
    ctx: &DapStartContext,
) -> Result<(), String> {
    let entries = adapters
        .iter()
        .enumerate()
        .map(|(index, adapter)| PickerEntry {
            item: PickerItem::new(
                format!("dap-adapter:{index}:{}", adapter.id()),
                adapter.id(),
                format!(
                    "{} · preference {}",
                    adapter.language_id(),
                    adapter.preference()
                ),
                None::<String>,
            ),
            action: PickerAction::EmitHook {
                hook: HOOK_DAP_START.to_owned(),
                detail: Some(adapter.id().to_owned()),
            },
            quickfix: None,
        })
        .collect();
    let _ = ctx;
    shell_ui_mut(runtime)?.set_picker(
        PickerOverlay::from_entries("Choose Debug Adapter", entries)
            .with_result_order(PickerResultOrder::Source),
    );
    Ok(())
}

fn resolve_dap_configuration_then_start(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    ctx: &DapStartContext,
) -> Result<(), String> {
    let infer = DebugInferContext {
        workspace_root: ctx.workspace_root.as_deref(),
        active_file: ctx.active_file.as_deref(),
        preferred_adapter_id: Some(adapter_id),
        allow_deep_inference: ctx.allow_deep_inference,
    };
    let mut candidates =
        collect_configuration_candidates(&infer).map_err(|error| error.to_string())?;
    if candidates.is_empty() {
        if !ctx.allow_deep_inference {
            return open_dap_program_prompt(runtime, adapter_id, None);
        }
        return Err(
            "dap.start needs an open file to infer a Debug Configuration, or a project `.volt/debug.json`"
                .to_owned(),
        );
    }
    if candidates.len() == 1 {
        let candidate = candidates.remove(0);
        return continue_dap_start(runtime, adapter_id, candidate.into_configuration(), true);
    }
    open_dap_configuration_picker(runtime, adapter_id, candidates)
}

fn open_dap_configuration_picker(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    candidates: Vec<DebugConfigurationCandidate>,
) -> Result<(), String> {
    let entries = candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let pinned = candidate.adapter_id().unwrap_or(adapter_id).to_owned();
            PickerEntry {
                item: PickerItem::new(
                    format!("dap-config:{index}"),
                    candidate.picker_label(),
                    candidate.picker_detail(),
                    None::<String>,
                ),
                action: PickerAction::StartDapSession {
                    adapter_id: pinned,
                    configuration: candidate.into_configuration(),
                    ask_heuristic_compile: true,
                },
                quickfix: None,
            }
        })
        .collect();
    shell_ui_mut(runtime)?.set_picker(
        PickerOverlay::from_entries("Choose Debug Configuration", entries)
            .with_result_order(PickerResultOrder::Source),
    );
    Ok(())
}

fn continue_dap_start(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: DebugConfiguration,
    ask_heuristic_compile: bool,
) -> Result<(), String> {
    let holes = configuration_holes(&configuration);
    if !holes.is_empty() {
        return fill_dap_configuration_holes(runtime, adapter_id, configuration, holes);
    }

    if let Some(command) = configuration.compile_command().map(str::to_owned) {
        return run_dap_prelaunch_then_start(runtime, adapter_id, configuration, Some(command));
    }

    if ask_heuristic_compile {
        let ctx = dap_start_context(runtime)?;
        if let Some(heuristic) =
            infer_compile_heuristic(ctx.workspace_root.as_deref(), ctx.active_file.as_deref())
        {
            open_dap_compile_confirm_picker(runtime, adapter_id, configuration, heuristic)?;
            return Ok(());
        }
    }

    run_dap_prelaunch_then_start(runtime, adapter_id, configuration, None)
}

fn fill_dap_configuration_holes(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: DebugConfiguration,
    holes: Vec<&'static str>,
) -> Result<(), String> {
    if holes.iter().any(|hole| hole.contains("process"))
        && configuration.request() == DebugRequestKind::Attach
    {
        return open_dap_process_prompt(runtime, adapter_id, configuration);
    }
    open_dap_program_prompt(runtime, adapter_id, Some(configuration))
}

fn open_dap_program_prompt(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: Option<DebugConfiguration>,
) -> Result<(), String> {
    let configuration =
        configuration.unwrap_or_else(|| DebugConfiguration::new("Debug", DebugRequestKind::Launch));
    let prefill = configuration
        .target_program()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    shell_ui_mut(runtime)?.pending_dap_start = Some(PendingDapStartPrompt {
        adapter_id: adapter_id.to_owned(),
        configuration,
        ask_heuristic_compile: true,
    });
    let overlay = InputPromptOverlay::new(DAP_PROGRAM_PROMPT_ID, "Debug program: ", &prefill);
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn open_dap_process_prompt(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: DebugConfiguration,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.pending_dap_start = Some(PendingDapStartPrompt {
        adapter_id: adapter_id.to_owned(),
        configuration,
        ask_heuristic_compile: false,
    });
    let overlay = InputPromptOverlay::new(DAP_PROCESS_PROMPT_ID, "Attach process id: ", "");
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn open_dap_compile_confirm_picker(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: DebugConfiguration,
    command: String,
) -> Result<(), String> {
    let entries = vec![
        PickerEntry {
            item: PickerItem::new(
                "dap-compile:yes",
                format!("Run `{command}` then start"),
                "compile-before-debug",
                None::<String>,
            ),
            action: PickerAction::ConfirmDapCompile {
                adapter_id: adapter_id.to_owned(),
                configuration: configuration.clone(),
                command: command.clone(),
            },
            quickfix: None,
        },
        PickerEntry {
            item: PickerItem::new(
                "dap-compile:no",
                "Start without compiling",
                "skip compile-before-debug",
                None::<String>,
            ),
            action: PickerAction::SkipDapCompile {
                adapter_id: adapter_id.to_owned(),
                configuration,
            },
            quickfix: None,
        },
    ];
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries(
        "Compile before debug?",
        entries,
    ));
    Ok(())
}

fn run_dap_prelaunch_then_start(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: DebugConfiguration,
    compile_command: Option<String>,
) -> Result<(), String> {
    save_workspace_for_dap_start(runtime)?;
    if let Some(command) = compile_command {
        run_dap_compile_before_debug(runtime, &command)?;
    }
    finish_dap_session_start(runtime, adapter_id, configuration)
}

fn save_workspace_for_dap_start(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = active_shell_workspace_id(runtime)
        .or_else(|| runtime.model().active_workspace_id().ok())
        .ok_or_else(|| "dap.start needs an active Workspace to save".to_owned())?;
    save_workspace(runtime, workspace_id)
}

fn run_dap_compile_before_debug(runtime: &mut EditorRuntime, command: &str) -> Result<(), String> {
    let command = command.trim();
    if command.is_empty() {
        return Ok(());
    }
    let buffer_id = open_command_output_buffer(runtime)?;
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.replace_with_lines_follow_output(vec![
            "# compile-before-debug".to_owned(),
            format!("$ {command}"),
            String::new(),
        ]);
    }
    let terminal_config = shell_user_library(runtime).terminal_config();
    let cwd = active_workspace_root(runtime)
        .ok()
        .flatten()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut args = terminal_config.args;
    let shell_program = terminal_config.program;
    args.extend(
        shell_command_eval_args(&shell_program)
            .into_iter()
            .map(str::to_owned),
    );
    args.push(command.to_owned());
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!("$ {command}"), String::new()]);
    }
    let spec = JobSpec::command("dap-prelaunch", shell_program, args).with_cwd(cwd);
    let manager = runtime
        .services()
        .get::<Mutex<JobManager>>()
        .ok_or_else(|| "job manager service missing".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "job manager lock poisoned".to_owned())?;
    let handle = manager.spawn(spec).map_err(|error| error.to_string())?;
    drop(manager);
    let result = handle.wait().map_err(|error| error.to_string())?;
    let transcript = result.transcript();
    let output_lines: Vec<String> = transcript.lines().map(str::to_owned).collect();
    let status_line = if result.succeeded() {
        "── ✓ compile-before-debug succeeded ───────────────────────────────────".to_owned()
    } else {
        format!(
            "── ✗ compile-before-debug failed (exit {}) ──────────────────────────",
            result.exit_code().unwrap_or(-1)
        )
    };
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&output_lines);
        buffer.append_output_lines(&[status_line]);
    }
    if !result.succeeded() {
        return Err(format!(
            "compile-before-debug failed for `{command}`; see command output"
        ));
    }
    Ok(())
}

fn finish_dap_session_start(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: DebugConfiguration,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let dap_client = dap_client_manager(runtime)?;
    if tool_install::install_debug_adapter_then_start(runtime, adapter_id, configuration.clone())? {
        return Ok(());
    }
    let extension = dap_start_context(runtime)?.extension;
    dap_client
        .start(
            workspace_id.get(),
            Some(adapter_id),
            extension.as_deref(),
            configuration,
        )
        .map_err(|error| error.to_string())?;
    install_debug_layout(runtime)?;
    refresh_dap_fringe_cache(runtime)?;
    refresh_dap_sessions_buffer(runtime, workspace_id)?;
    Ok(())
}

fn notify_dap_failure(
    runtime: &mut EditorRuntime,
    title: &str,
    message: &str,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.apply_notification(
        NotificationUpdate {
            key: "dap.failure".to_owned(),
            severity: NotificationSeverity::Error,
            title: title.to_owned(),
            body_lines: vec![message.to_owned()],
            progress: None,
            active: false,
            action: None,
            workspace_id: None,
        },
        Instant::now(),
    );
    Ok(())
}

fn report_dap_result(
    runtime: &mut EditorRuntime,
    title: &str,
    result: Result<(), String>,
) -> Result<(), String> {
    if let Err(error) = &result {
        let _ = notify_dap_failure(runtime, title, error);
    }
    result
}

fn dap_client_manager(runtime: &EditorRuntime) -> Result<Arc<DapClientManager>, String> {
    runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .cloned()
        .ok_or_else(|| "DAP client manager service missing".to_owned())
}

#[derive(Debug, Clone)]
struct DapStartContext {
    extension: Option<String>,
    active_file: Option<PathBuf>,
    workspace_root: Option<PathBuf>,
    allow_deep_inference: bool,
    preferred_from_config: Option<String>,
}

fn stop_dap_for_active_workspace(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    finish_dap_session(runtime, workspace_id.get())
}

fn finish_dap_session(runtime: &mut EditorRuntime, workspace_id: u64) -> Result<(), String> {
    let dap_client = dap_client_manager(runtime)?;
    match dap_client.stop_session(workspace_id) {
        Ok(_) | Err(DapClientError::SessionMissing(_)) => {}
        Err(error) => return Err(error.to_string()),
    }
    let active = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if active.get() != workspace_id {
        return Ok(());
    }
    teardown_debug_layout(runtime)?;
    refresh_dap_fringe_cache(runtime)?;
    refresh_dap_sessions_buffer(runtime, active)?;
    Ok(())
}

fn restart_dap_for_active_workspace(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let dap_client = runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .cloned()
        .ok_or_else(|| "DAP client manager service missing".to_owned())?;
    dap_client
        .restart_session(workspace_id.get())
        .map_err(|error| error.to_string())?;
    if !shell_ui(runtime)?.is_debug_layout_active() {
        install_debug_layout(runtime)?;
    }
    refresh_dap_fringe_cache(runtime)?;
    refresh_dap_sessions_buffer(runtime, workspace_id)?;
    Ok(())
}

fn dap_control_for_active_workspace(
    runtime: &mut EditorRuntime,
    control: DapControl,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let dap_client = runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .cloned()
        .ok_or_else(|| "DAP client manager service missing".to_owned())?;
    match control {
        DapControl::Continue => dap_client
            .continue_session(workspace_id.get())
            .map_err(|error| error.to_string())?,
        DapControl::Pause => dap_client
            .pause_session(workspace_id.get())
            .map_err(|error| error.to_string())?,
        DapControl::StepOver => dap_client
            .step_over(workspace_id.get())
            .map_err(|error| error.to_string())?,
        DapControl::StepInto => dap_client
            .step_into(workspace_id.get())
            .map_err(|error| error.to_string())?,
        DapControl::StepOut => dap_client
            .step_out(workspace_id.get())
            .map_err(|error| error.to_string())?,
    }
    if !matches!(control, DapControl::Pause) {
        // SharpDbg never sends `continued`; clear execution UI until the next stop.
        apply_dap_continued_ui(runtime, workspace_id.get())?;
    }
    Ok(())
}

fn dap_start_context(runtime: &EditorRuntime) -> Result<DapStartContext, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let ui = shell_ui(runtime)?;
    let allow_deep_inference = workspace_id != ui.default_workspace();
    let workspace_root = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .root()
        .map(Path::to_path_buf);

    let buffer_id = active_shell_buffer_id(runtime).ok();
    let active_file = buffer_id.and_then(|buffer_id| {
        shell_ui(runtime)
            .ok()
            .and_then(|ui| ui.buffer(buffer_id))
            .and_then(|buffer| buffer.path().map(Path::to_path_buf))
    });
    let extension = active_file
        .as_ref()
        .and_then(|path| path.extension())
        .and_then(|ext| ext.to_str())
        .map(str::to_owned);
    Ok(DapStartContext {
        extension,
        active_file,
        workspace_root,
        allow_deep_inference,
        preferred_from_config: None,
    })
}

fn refresh_dap_sessions_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let lines = dap_sessions_buffer_lines(runtime)?;
    let buffer_id = ensure_dap_surface_buffer(
        runtime,
        workspace_id,
        DAP_SESSIONS_BUFFER_NAME,
        Some("Debug Sessions"),
    )?;
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(buffer_id) {
            buffer.replace_with_lines_follow_output(lines);
        }
    }
    Ok(())
}

fn dap_sessions_buffer_lines(runtime: &EditorRuntime) -> Result<Vec<String>, String> {
    let Some(dap_client) = runtime.services().get::<Arc<DapClientManager>>() else {
        return Ok(vec!["DAP client manager is not available.".to_owned()]);
    };
    let sessions = dap_client.sessions().map_err(|error| error.to_string())?;
    if sessions.is_empty() {
        return Ok(vec!["No live Debug Sessions.".to_owned()]);
    }
    let mut lines = vec!["Live Debug Sessions:".to_owned(), String::new()];
    for session in sessions {
        let request = match session.request() {
            DebugRequestKind::Launch => "launch",
            DebugRequestKind::Attach => "attach",
        };
        lines.push(format!(
            "- workspace {} · {} · {} · {}",
            session.workspace_id(),
            session.adapter_id(),
            session.configuration_name(),
            request
        ));
    }
    Ok(lines)
}

fn open_dap_log_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let snapshot = runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .map(|manager| manager.log_snapshot())
        .unwrap_or_default();
    let lines = dap_log_buffer_lines(&snapshot);
    let buffer_id =
        ensure_dap_surface_buffer(runtime, workspace_id, DAP_LOG_BUFFER_NAME, Some("DAP Log"))?;
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(buffer_id) {
            buffer.replace_with_lines_follow_output(lines);
        }
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn dap_log_buffer_lines(snapshot: &DapLogSnapshot) -> Vec<String> {
    if snapshot.entries().is_empty() {
        return vec!["No DAP transport traffic yet.".to_owned()];
    }
    let mut lines = Vec::new();
    for entry in snapshot.entries() {
        let direction = match entry.direction() {
            DapLogDirection::Send => "→",
            DapLogDirection::Receive => "←",
            DapLogDirection::Event => "•",
        };
        lines.push(format!(
            "{direction} [{}] {}",
            entry.adapter_id(),
            entry.message()
        ));
    }
    lines
}

fn refresh_dap_fringe_cache(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(dap_client) = runtime.services().get::<Arc<DapClientManager>>() else {
        let ui = shell_ui_mut(runtime)?;
        for buffer in &mut ui.buffers {
            buffer.clear_dap_fringe();
        }
        return Ok(());
    };
    let sessions = dap_client.sessions().map_err(|error| error.to_string())?;
    let live: BTreeSet<u64> = sessions
        .iter()
        .map(|session| session.workspace_id())
        .collect();
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map(|id| id.get())
        .unwrap_or_else(|_| {
            shell_ui(runtime)
                .map(|ui| ui.active_workspace.get())
                .unwrap_or(0)
        });
    let session_live = live.contains(&workspace_id);
    let workspace_bps = dap_client
        .list_breakpoints(workspace_id)
        .map_err(|error| error.to_string())?;
    let execution = if session_live {
        dap_client
            .stopped_snapshot(workspace_id)
            .map_err(|error| error.to_string())?
            .and_then(|snapshot| snapshot.position().cloned())
    } else {
        None
    };
    let ui = shell_ui_mut(runtime)?;
    for buffer in &mut ui.buffers {
        let Some(path) = buffer.path().map(Path::to_path_buf) else {
            buffer.clear_dap_fringe();
            continue;
        };
        let mut markers = BTreeMap::new();
        for bp in &workspace_bps {
            if editor_dap::debug_source_paths_eq(bp.path(), path.as_path()) {
                markers.insert((bp.line() as usize).saturating_sub(1), bp.state());
            }
        }
        let execution_line = execution.as_ref().and_then(|position| {
            editor_dap::debug_source_paths_eq(position.path(), path.as_path())
                .then_some((position.line() as usize).saturating_sub(1))
        });
        // Breakpoints show in the Debug Fringe without a live Session (one cell; may
        // replace git on that line). Two-cell widening only while the Session is live.
        if session_live || !markers.is_empty() {
            buffer.set_dap_fringe(session_live, markers, execution_line);
        } else {
            buffer.clear_dap_fringe();
        }
    }
    Ok(())
}

fn toggle_dap_breakpoint_at_cursor(runtime: &mut EditorRuntime) -> Result<(), String> {
    let (workspace_id, path, line) = dap_breakpoint_cursor_target(runtime)?;
    let dap_client = runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .cloned()
        .ok_or_else(|| "DAP client manager service missing".to_owned())?;
    let _ = dap_client
        .toggle_breakpoint(workspace_id.get(), path, line)
        .map_err(|error| error.to_string())?;
    refresh_dap_fringe_cache(runtime)?;
    let _ = refresh_dap_breakpoints_buffer(runtime, workspace_id);
    Ok(())
}

fn delete_dap_breakpoint_at_cursor(runtime: &mut EditorRuntime) -> Result<(), String> {
    let (workspace_id, path, line) = dap_breakpoint_cursor_target(runtime)?;
    let dap_client = runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .cloned()
        .ok_or_else(|| "DAP client manager service missing".to_owned())?;
    let _ = dap_client
        .delete_breakpoint(workspace_id.get(), &path, line)
        .map_err(|error| error.to_string())?;
    refresh_dap_fringe_cache(runtime)?;
    let _ = refresh_dap_breakpoints_buffer(runtime, workspace_id);
    Ok(())
}

fn dap_breakpoint_cursor_target(
    runtime: &EditorRuntime,
) -> Result<(WorkspaceId, PathBuf, u32), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_ui(runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let path = buffer
        .path()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Breakpoints require a file buffer".to_owned())?;
    let line = (buffer.cursor_row() as u32).saturating_add(1);
    Ok((workspace_id, path, line))
}

fn open_dap_breakpoints_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = ensure_dap_workspace_buffer(
        runtime,
        workspace_id,
        DAP_BREAKPOINTS_BUFFER_NAME,
        DAP_BREAKPOINTS_KIND,
    )?;
    refresh_dap_breakpoints_buffer(runtime, workspace_id)?;
    if shell_ui(runtime)?.is_debug_layout_active() {
        focus_debug_layout_pane(runtime, 0)?;
        return Ok(());
    }
    {
        let ui = shell_ui_mut(runtime)?;
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn refresh_dap_breakpoints_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let Some(buffer_id) = find_workspace_named_buffer(
        runtime,
        workspace_id,
        DAP_BREAKPOINTS_BUFFER_NAME,
        &BufferKind::Plugin(DAP_BREAKPOINTS_KIND.to_owned()),
    )?
    else {
        return Ok(());
    };
    ensure_shell_buffer(runtime, buffer_id)?;
    let (lines, syntax_lines) = dap_breakpoints_buffer_content(runtime, workspace_id)?;
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(buffer_id) {
            buffer.replace_with_lines_preserve_view(lines);
            buffer.set_indexed_syntax_lines(Some(syntax_lines), None);
        }
    }
    Ok(())
}

fn dap_breakpoints_buffer_content(
    runtime: &EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(Vec<String>, IndexedSyntaxLines), String> {
    let Some(dap_client) = runtime.services().get::<Arc<DapClientManager>>() else {
        let message = "DAP client manager is not available.";
        let mut syntax_lines = IndexedSyntaxLines::new();
        let mut spans = Vec::new();
        push_span_bytes(
            &mut spans,
            message,
            0,
            message.len(),
            TOKEN_DEBUG_BREAKPOINT_EMPTY,
        );
        syntax_lines.insert(0, spans);
        return Ok((vec![message.to_owned()], syntax_lines));
    };
    let breakpoints = dap_client
        .list_breakpoints(workspace_id.get())
        .map_err(|error| error.to_string())?;
    let root = runtime
        .model()
        .workspace(workspace_id)
        .ok()
        .and_then(|workspace| workspace.root().map(Path::to_path_buf));
    Ok((
        dap_breakpoint_lines(&breakpoints, root.as_deref()),
        dap_breakpoint_syntax_lines(&breakpoints, root.as_deref()),
    ))
}

fn refresh_dap_locals_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let buffer_id = ensure_dap_workspace_buffer(
        runtime,
        workspace_id,
        DAP_LOCALS_BUFFER_NAME,
        DAP_LOCALS_KIND,
    )?;
    if dap_locals_insert_in_progress(runtime, buffer_id)? {
        return Ok(());
    }
    let (local_lines, expression_lines) =
        dap_locals_and_expression_lines(runtime, workspace_id.get())?;
    let local_syntax = dap_variable_syntax_lines(&local_lines, false);
    let expression_syntax = dap_variable_syntax_lines(&expression_lines, true);
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(buffer_id) {
            set_named_plugin_section_lines(buffer, DAP_LOCALS_SECTION, local_lines, local_syntax);
            set_named_plugin_section_lines(
                buffer,
                DAP_EXPRESSIONS_SECTION,
                expression_lines,
                expression_syntax,
            );
        }
    }
    Ok(())
}

fn dap_locals_insert_in_progress(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<bool, String> {
    let ui = shell_ui(runtime)?;
    if ui.active_buffer_id() != Some(buffer_id) {
        return Ok(false);
    }
    if !matches!(ui.input_mode(), InputMode::Insert | InputMode::Replace) {
        return Ok(false);
    }
    let buffer = shell_buffer(runtime, buffer_id)?;
    Ok(buffer
        .plugin_active_section_name()
        .unwrap_or(DAP_LOCALS_SECTION)
        == DAP_LOCALS_SECTION)
}

fn set_named_plugin_section_lines(
    buffer: &mut ShellBuffer,
    section_name: &str,
    lines: Vec<String>,
    syntax_lines: IndexedSyntaxLines,
) {
    let index = {
        let Some(state) = buffer.plugin_section_state.as_ref() else {
            buffer.replace_with_lines_preserve_view(lines);
            buffer.set_indexed_syntax_lines(Some(syntax_lines), None);
            return;
        };
        let Some(index) = state.section_index_by_name(section_name) else {
            return;
        };
        index
    };
    if index == 0 {
        buffer.replace_with_lines_preserve_view(lines);
        buffer.set_indexed_syntax_lines(Some(syntax_lines), None);
        return;
    }
    if let Some(state) = buffer.plugin_section_state.as_mut()
        && let Some(pane) = state.attached_section_mut(index)
    {
        pane.replace_lines(lines, false);
        pane.set_indexed_syntax_lines(syntax_lines);
    }
}

fn dap_locals_and_expression_lines(
    runtime: &EditorRuntime,
    workspace_id: u64,
) -> Result<(Vec<String>, Vec<String>), String> {
    let Some(dap_client) = runtime.services().get::<Arc<DapClientManager>>() else {
        return Ok((
            dap_locals_section_lines(&[], &[], &[], false),
            dap_expression_section_lines(&[], &[], false),
        ));
    };
    let idle_watches = dap_client
        .list_expressions(workspace_id)
        .map_err(|error| error.to_string())?;
    let Some(snapshot) = dap_client
        .stopped_snapshot(workspace_id)
        .map_err(|error| error.to_string())?
    else {
        return Ok((
            dap_locals_section_lines(&[], &[], &idle_watches, false),
            dap_expression_section_lines(&[], &idle_watches, false),
        ));
    };
    let local_rows = snapshot.local_rows();
    let watch_rows = snapshot.watch_rows();
    Ok((
        dap_locals_section_lines(&local_rows, &watch_rows, &idle_watches, true),
        dap_expression_section_lines(&watch_rows, &idle_watches, true),
    ))
}

fn apply_dap_locals_edits(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let (workspace_id, lines) = {
        let Ok(buffer) = shell_buffer(runtime, buffer_id) else {
            return Ok(());
        };
        match &buffer.kind {
            BufferKind::Plugin(kind) if kind == DAP_LOCALS_KIND => {}
            _ => return Ok(()),
        }
        if buffer
            .plugin_active_section_name()
            .unwrap_or(DAP_LOCALS_SECTION)
            != DAP_LOCALS_SECTION
        {
            return Ok(());
        }
        let workspace_id = runtime
            .model()
            .active_workspace_id()
            .map_err(|error| error.to_string())?;
        let lines: Vec<String> = (0..buffer.text.line_count())
            .filter_map(|line_index| buffer.text.line(line_index))
            .collect();
        (workspace_id, lines)
    };
    if !lines.iter().any(|line| line == DAP_WATCHES_HEADER) {
        refresh_dap_locals_buffer(runtime, workspace_id)?;
        return Ok(());
    }
    let Some(dap_client) = runtime.services().get::<Arc<DapClientManager>>() else {
        return Ok(());
    };
    dap_client
        .set_expressions(workspace_id.get(), extract_watch_expressions(&lines))
        .map_err(|error| error.to_string())?;
    refresh_dap_locals_buffer(runtime, workspace_id)
}

fn dap_toggle_variable(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (section, line_index, lines) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        match &buffer.kind {
            BufferKind::Plugin(kind) if kind == DAP_LOCALS_KIND => {}
            _ => {
                return Err("dap.toggle-variable requires the Locals buffer".to_owned());
            }
        }
        let section = buffer
            .plugin_active_section_name()
            .unwrap_or(DAP_LOCALS_SECTION)
            .to_owned();
        let line_index = buffer
            .plugin_attached_pane_state()
            .map(|pane| pane.cursor().line)
            .unwrap_or_else(|| buffer.cursor_row());
        let lines: Vec<String> = (0..buffer.text.line_count())
            .filter_map(|index| buffer.text.line(index))
            .collect();
        (section, line_index, lines)
    };
    let dap_client = dap_client_manager(runtime)?;
    let Some(snapshot) = dap_client
        .stopped_snapshot(workspace_id.get())
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };
    let local_rows = snapshot.local_rows();
    let watch_rows = snapshot.watch_rows();
    let row = if section == DAP_EXPRESSIONS_SECTION {
        watch_rows.get(line_index)
    } else {
        match locals_line_variable_kind(&lines, line_index, local_rows.len(), watch_rows.len()) {
            Some(DapLocalsLineTarget::Local(index)) => local_rows.get(index),
            Some(DapLocalsLineTarget::Watch(index)) => watch_rows.get(index),
            None => return Ok(()),
        }
    };
    let Some(row) = row else {
        return Ok(());
    };
    if !row.expandable() {
        return Ok(());
    }
    dap_client
        .toggle_variable_expand(workspace_id.get(), row.path())
        .map_err(|error| error.to_string())?;
    refresh_dap_locals_buffer(runtime, workspace_id)
}

fn dap_goto_breakpoint(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let line_index = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        match &buffer.kind {
            BufferKind::Plugin(kind) if kind == DAP_BREAKPOINTS_KIND => buffer.cursor_row(),
            _ => {
                return Err("dap.goto-breakpoint requires the Breakpoints buffer".to_owned());
            }
        }
    };
    let dap_client = dap_client_manager(runtime)?;
    let breakpoints = dap_client
        .list_breakpoints(workspace_id.get())
        .map_err(|error| error.to_string())?;
    let Some(bp) = breakpoints.get(line_index) else {
        return Ok(());
    };
    let path = bp.path().to_path_buf();
    let line = bp.line();
    if shell_ui(runtime)?.is_debug_layout_active() {
        focus_debug_layout_pane(runtime, 1)?;
    }
    open_workspace_file_at(
        runtime,
        &path,
        TextPoint::new((line as usize).saturating_sub(1), 0),
    )?;
    if shell_ui(runtime)?.is_debug_layout_active() {
        focus_debug_layout_pane(runtime, 1)?;
    }
    Ok(())
}

fn apply_dap_stopped_ui(runtime: &mut EditorRuntime, workspace_id: u64) -> Result<(), String> {
    let dap_client = runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .cloned()
        .ok_or_else(|| "DAP client manager service missing".to_owned())?;
    let snapshot = dap_client
        .refresh_stopped_snapshot(workspace_id)
        .map_err(|error| error.to_string())?;
    let active_workspace = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if active_workspace.get() != workspace_id {
        return Ok(());
    }
    refresh_dap_locals_buffer(runtime, active_workspace)?;
    if let Some(position) = snapshot.position() {
        if shell_ui(runtime)?.is_debug_layout_active() {
            focus_debug_layout_pane(runtime, 1)?;
        }
        open_workspace_file_at(
            runtime,
            position.path(),
            TextPoint::new(
                (position.line() as usize).saturating_sub(1),
                (position.column() as usize).saturating_sub(1),
            ),
        )?;
        if shell_ui(runtime)?.is_debug_layout_active() {
            focus_debug_layout_pane(runtime, 1)?;
        }
    }
    refresh_dap_fringe_cache(runtime)?;
    Ok(())
}

fn apply_dap_continued_ui(runtime: &mut EditorRuntime, workspace_id: u64) -> Result<(), String> {
    let active_workspace = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if active_workspace.get() != workspace_id {
        return Ok(());
    }
    refresh_dap_locals_buffer(runtime, active_workspace)?;
    refresh_dap_fringe_cache(runtime)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DapBreakpointExtraKind {
    Condition,
    HitCondition,
    LogMessage,
}

fn dap_expressions_add(runtime: &mut EditorRuntime, detail: Option<&str>) -> Result<(), String> {
    if let Some(expression) = detail.map(str::trim).filter(|text| !text.is_empty()) {
        return add_dap_expression(runtime, expression);
    }
    let overlay = InputPromptOverlay::new(DAP_EXPRESSION_ADD_PROMPT_ID, "Watch expression: ", "");
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn add_dap_expression(runtime: &mut EditorRuntime, expression: &str) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let dap_client = dap_client_manager(runtime)?;
    dap_client
        .add_expression(workspace_id.get(), expression)
        .map_err(|error| error.to_string())?;
    refresh_dap_locals_buffer(runtime, workspace_id)?;
    Ok(())
}

fn dap_expressions_remove(runtime: &mut EditorRuntime, detail: Option<&str>) -> Result<(), String> {
    if let Some(expression) = detail.map(str::trim).filter(|text| !text.is_empty()) {
        return remove_dap_expression(runtime, expression);
    }
    open_dap_expressions_remove_picker(runtime)
}

fn remove_dap_expression(runtime: &mut EditorRuntime, expression: &str) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let dap_client = dap_client_manager(runtime)?;
    let removed = dap_client
        .remove_expression(workspace_id.get(), expression)
        .map_err(|error| error.to_string())?;
    if !removed {
        return Err(format!("Watch Expression `{expression}` not found"));
    }
    refresh_dap_locals_buffer(runtime, workspace_id)?;
    Ok(())
}

fn open_dap_expressions_remove_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let expressions = dap_client_manager(runtime)?
        .list_expressions(workspace_id.get())
        .map_err(|error| error.to_string())?;
    if expressions.is_empty() {
        return Err("no Watch Expressions to remove".to_owned());
    }
    let entries = expressions
        .into_iter()
        .enumerate()
        .map(|(index, expression)| PickerEntry {
            item: PickerItem::new(
                format!("dap-watch:{index}"),
                expression.clone(),
                "remove Watch Expression",
                None::<String>,
            ),
            action: PickerAction::RemoveDapExpression { expression },
            quickfix: None,
        })
        .collect();
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries(
        "Remove Watch Expression",
        entries,
    ));
    Ok(())
}

fn dap_eval(runtime: &mut EditorRuntime, detail: Option<&str>) -> Result<(), String> {
    if let Some(expression) = detail.map(str::trim).filter(|text| !text.is_empty()) {
        return show_dap_eval_result(runtime, expression, DapEvaluateContext::Repl);
    }
    let overlay = InputPromptOverlay::new(DAP_EVAL_PROMPT_ID, "Eval: ", "");
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn dap_eval_at_point(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let expression = {
        let buffer = shell_ui(runtime)?
            .buffer(buffer_id)
            .ok_or_else(|| "active buffer missing".to_owned())?;
        completion_token_at_cursor(buffer)
            .map(|(_, token)| token)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| "no identifier at point".to_owned())?
    };
    show_dap_eval_result(runtime, &expression, DapEvaluateContext::Hover)
}

fn show_dap_eval_result(
    runtime: &mut EditorRuntime,
    expression: &str,
    context: DapEvaluateContext,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let result = dap_client_manager(runtime)?
        .evaluate(workspace_id.get(), expression, context)
        .map_err(|error| error.to_string())?;
    let lines = vec![
        format!("expression: {}", result.expression()),
        format!(
            "result: {}{}",
            if result.ok() { "" } else { "! " },
            result.value()
        ),
    ];
    let buffer_id = ensure_dap_surface_buffer(
        runtime,
        workspace_id,
        DAP_EVAL_BUFFER_NAME,
        Some("DAP Eval"),
    )?;
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(buffer_id) {
            buffer.replace_with_lines_follow_output(lines);
        }
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn open_dap_repl(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let _ = dap_client_manager(runtime)?
        .session_info(workspace_id.get())
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "no live Debug Session".to_owned())?;
    let buffer_id = ensure_dap_surface_buffer(
        runtime,
        workspace_id,
        DAP_REPL_BUFFER_NAME,
        Some("DAP REPL"),
    )?;
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(buffer_id)
            && buffer.line_count() <= 1
            && buffer
                .text
                .line(0)
                .map(|line| line.is_empty() || line.starts_with('('))
                .unwrap_or(true)
        {
            buffer.replace_with_lines_follow_output(vec!["(debug REPL)".to_owned(), String::new()]);
        }
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let overlay = InputPromptOverlay::new(DAP_REPL_PROMPT_ID, "DAP REPL: ", "");
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn submit_dap_repl_expression(runtime: &mut EditorRuntime, expression: &str) -> Result<(), String> {
    let expression = expression.trim();
    if expression.is_empty() {
        return open_dap_repl(runtime);
    }
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let result = dap_client_manager(runtime)?
        .evaluate(workspace_id.get(), expression, DapEvaluateContext::Repl)
        .map_err(|error| error.to_string())?;
    let buffer_id = ensure_dap_surface_buffer(
        runtime,
        workspace_id,
        DAP_REPL_BUFFER_NAME,
        Some("DAP REPL"),
    )?;
    {
        let ui = shell_ui_mut(runtime)?;
        if let Some(buffer) = ui.buffer_mut(buffer_id) {
            let mut lines = (0..buffer.line_count())
                .map(|index| buffer.text.line(index).unwrap_or_default())
                .collect::<Vec<_>>();
            if lines.len() == 1 && lines[0].is_empty() {
                lines.clear();
            }
            lines.push(format!("> {expression}"));
            lines.push(format!(
                "{}{}",
                if result.ok() { "" } else { "! " },
                result.value()
            ));
            lines.push(String::new());
            buffer.replace_with_lines_follow_output(lines);
        }
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let overlay = InputPromptOverlay::new(DAP_REPL_PROMPT_ID, "DAP REPL: ", "");
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn open_dap_switch_thread_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let threads = dap_client_manager(runtime)?
        .list_threads(workspace_id.get())
        .map_err(|error| error.to_string())?;
    if threads.is_empty() {
        return Err("Debug Adapter reported no threads".to_owned());
    }
    let entries = threads
        .into_iter()
        .map(|thread| PickerEntry {
            item: PickerItem::new(
                format!("dap-thread:{}", thread.id()),
                format!("{} ({})", thread.name(), thread.id()),
                "switch active thread",
                None::<String>,
            ),
            action: PickerAction::SwitchDapThread {
                thread_id: thread.id(),
            },
            quickfix: None,
        })
        .collect();
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries("Switch Thread", entries));
    Ok(())
}

fn open_dap_switch_stack_frame_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let frames = dap_client_manager(runtime)?
        .list_stack_frames(workspace_id.get())
        .map_err(|error| error.to_string())?;
    if frames.is_empty() {
        return Err("no stack frames for the active thread".to_owned());
    }
    let entries = frames
        .into_iter()
        .map(|frame| {
            let detail = match frame.path() {
                Some(path) => format!("{}:{}", path.display(), frame.line()),
                None => format!("line {}", frame.line()),
            };
            PickerEntry {
                item: PickerItem::new(
                    format!("dap-frame:{}", frame.id()),
                    frame.name(),
                    detail,
                    None::<String>,
                ),
                action: PickerAction::SwitchDapStackFrame {
                    frame_id: frame.id(),
                },
                quickfix: None,
            }
        })
        .collect();
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries("Switch Stack Frame", entries));
    Ok(())
}

fn switch_dap_thread(runtime: &mut EditorRuntime, thread_id: u64) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    dap_client_manager(runtime)?
        .switch_thread(workspace_id.get(), thread_id)
        .map_err(|error| error.to_string())?;
    apply_dap_stopped_ui(runtime, workspace_id.get())
}

fn switch_dap_stack_frame(runtime: &mut EditorRuntime, frame_id: u64) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    dap_client_manager(runtime)?
        .switch_stack_frame(workspace_id.get(), frame_id)
        .map_err(|error| error.to_string())?;
    apply_dap_stopped_ui(runtime, workspace_id.get())
}

fn dap_breakpoint_extra_prompt(
    runtime: &mut EditorRuntime,
    kind: DapBreakpointExtraKind,
    detail: Option<&str>,
) -> Result<(), String> {
    if let Some(value) = detail {
        return apply_dap_breakpoint_extra(runtime, kind, value);
    }
    let (workspace_id, path, line) = dap_breakpoint_cursor_target(runtime)?;
    let existing = dap_client_manager(runtime)?
        .list_breakpoints(workspace_id.get())
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|bp| bp.path() == path.as_path() && bp.line() == line);
    let (prompt_id, label, prefill) = match kind {
        DapBreakpointExtraKind::Condition => (
            DAP_BP_CONDITION_PROMPT_ID,
            "Breakpoint condition: ",
            existing
                .as_ref()
                .and_then(|bp| bp.condition())
                .unwrap_or(""),
        ),
        DapBreakpointExtraKind::HitCondition => (
            DAP_BP_HIT_CONDITION_PROMPT_ID,
            "Breakpoint hit condition: ",
            existing
                .as_ref()
                .and_then(|bp| bp.hit_condition())
                .unwrap_or(""),
        ),
        DapBreakpointExtraKind::LogMessage => (
            DAP_BP_LOG_MESSAGE_PROMPT_ID,
            "Breakpoint log message: ",
            existing
                .as_ref()
                .and_then(|bp| bp.log_message())
                .unwrap_or(""),
        ),
    };
    let overlay = InputPromptOverlay::new(prompt_id, label, prefill);
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn apply_dap_breakpoint_extra(
    runtime: &mut EditorRuntime,
    kind: DapBreakpointExtraKind,
    text: &str,
) -> Result<(), String> {
    let (workspace_id, path, line) = dap_breakpoint_cursor_target(runtime)?;
    let value = {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    };
    let (condition, hit_condition, log_message) = match kind {
        DapBreakpointExtraKind::Condition => (Some(value), None, None),
        DapBreakpointExtraKind::HitCondition => (None, Some(value), None),
        DapBreakpointExtraKind::LogMessage => (None, None, Some(value)),
    };
    dap_client_manager(runtime)?
        .set_breakpoint_extras(
            workspace_id.get(),
            path,
            line,
            condition,
            hit_condition,
            log_message,
        )
        .map_err(|error| error.to_string())?;
    refresh_dap_fringe_cache(runtime)?;
    refresh_dap_breakpoints_buffer(runtime, workspace_id)?;
    Ok(())
}

fn refresh_pending_dap(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let Some(dap_client) = runtime.services().get::<Arc<DapClientManager>>().cloned() else {
        return Ok(false);
    };
    let events = dap_client
        .drain_events()
        .map_err(|error| error.to_string())?;
    if events.is_empty() {
        return Ok(false);
    }
    for event in events {
        match event {
            DapSessionEvent::Stopped { workspace_id } => {
                apply_dap_stopped_ui(runtime, workspace_id)?;
            }
            DapSessionEvent::Continued { workspace_id } => {
                apply_dap_continued_ui(runtime, workspace_id)?;
            }
            DapSessionEvent::Terminated { workspace_id } => {
                finish_dap_session(runtime, workspace_id)?;
            }
        }
    }
    Ok(true)
}

fn ensure_dap_workspace_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    name: &str,
    kind: &str,
) -> Result<BufferId, String> {
    let buffer_kind = BufferKind::Plugin(kind.to_owned());
    if let Some(buffer_id) = find_workspace_named_buffer(runtime, workspace_id, name, &buffer_kind)?
    {
        ensure_shell_buffer(runtime, buffer_id)?;
        return Ok(buffer_id);
    }
    let buffer_id = runtime
        .model_mut()
        .create_popup_buffer(workspace_id, name, buffer_kind.clone(), None)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(buffer_id, name, buffer_kind, &*user_library);
    }
    Ok(buffer_id)
}

fn ensure_dap_surface_buffer(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    name: &str,
    popup_title: Option<&str>,
) -> Result<BufferId, String> {
    let kind = BufferKind::Plugin("dap".to_owned());
    if let Some(buffer_id) = find_workspace_named_buffer(runtime, workspace_id, name, &kind)? {
        return Ok(buffer_id);
    }
    let buffer_id = if popup_title.is_some() {
        runtime
            .model_mut()
            .create_popup_buffer(workspace_id, name, kind.clone(), None)
            .map_err(|error| error.to_string())?
    } else {
        runtime
            .model_mut()
            .create_buffer(workspace_id, name, kind.clone(), None)
            .map_err(|error| error.to_string())?
    };
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(buffer_id, name, kind, &*user_library);
    }
    if let Some(popup_title) = popup_title {
        runtime
            .model_mut()
            .open_popup_buffer(workspace_id, popup_title, buffer_id)
            .map_err(|error| error.to_string())?;
    }
    Ok(buffer_id)
}

fn debug_layout_editor_buffer_id(runtime: &EditorRuntime) -> Result<BufferId, String> {
    let ui = shell_ui(runtime)?;
    let view = ui
        .workspace_view()
        .ok_or_else(|| "workspace view is missing".to_owned())?;
    if let Some(active) = view.panes.get(view.active_pane)
        && let Some(buffer) = ui.buffer(active.buffer_id)
        && !buffer_is_dap_layout_side(&buffer.kind)
    {
        return Ok(active.buffer_id);
    }
    view.panes
        .iter()
        .find_map(|pane| {
            ui.buffer(pane.buffer_id)
                .filter(|buffer| !buffer_is_dap_layout_side(&buffer.kind))
                .map(|_| pane.buffer_id)
        })
        .or_else(|| view.buffer_ids.first().copied())
        .ok_or_else(|| "Debug Layout needs an editor buffer".to_owned())
}

fn buffer_is_dap_layout_side(kind: &BufferKind) -> bool {
    matches!(
        kind,
        BufferKind::Plugin(plugin_kind)
            if plugin_kind == DAP_BREAKPOINTS_KIND || plugin_kind == DAP_LOCALS_KIND
    )
}

fn install_debug_layout(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if shell_ui(runtime)?.is_debug_layout_active() {
        refresh_dap_breakpoints_buffer(runtime, workspace_id)?;
        refresh_dap_locals_buffer(runtime, workspace_id)?;
        return Ok(());
    }

    let editor_buffer_id = debug_layout_editor_buffer_id(runtime)?;
    let breakpoints_id = ensure_dap_workspace_buffer(
        runtime,
        workspace_id,
        DAP_BREAKPOINTS_BUFFER_NAME,
        DAP_BREAKPOINTS_KIND,
    )?;
    let locals_id = ensure_dap_workspace_buffer(
        runtime,
        workspace_id,
        DAP_LOCALS_BUFFER_NAME,
        DAP_LOCALS_KIND,
    )?;
    refresh_dap_breakpoints_buffer(runtime, workspace_id)?;
    refresh_dap_locals_buffer(runtime, workspace_id)?;

    let existing_pane_ids: Vec<PaneId> = shell_ui(runtime)?
        .workspace_view()
        .map(|view| view.panes.iter().map(|pane| pane.pane_id).collect())
        .unwrap_or_default();
    if existing_pane_ids.is_empty() {
        return Err("workspace has no panes".to_owned());
    }

    let mut pane_ids = existing_pane_ids;
    let mut created_pane_ids = Vec::new();
    while pane_ids.len() < 3 {
        let pane_id = runtime
            .model_mut()
            .split_pane(workspace_id, editor_buffer_id)
            .map_err(|error| error.to_string())?;
        created_pane_ids.push(pane_id);
        pane_ids.push(pane_id);
    }
    let pane_ids = [pane_ids[0], pane_ids[1], pane_ids[2]];

    if !shell_ui_mut(runtime)?.begin_debug_layout(created_pane_ids) {
        return Err("failed to snapshot Debug Layout state".to_owned());
    }
    shell_ui_mut(runtime)?.set_debug_layout_panes(
        vec![
            (pane_ids[0], breakpoints_id),
            (pane_ids[1], editor_buffer_id),
            (pane_ids[2], locals_id),
        ],
        1,
    )?;

    for (pane_id, buffer_id) in [
        (pane_ids[0], breakpoints_id),
        (pane_ids[1], editor_buffer_id),
        (pane_ids[2], locals_id),
    ] {
        runtime
            .model_mut()
            .focus_pane(workspace_id, pane_id)
            .map_err(|error| error.to_string())?;
        runtime
            .model_mut()
            .focus_buffer(workspace_id, buffer_id)
            .map_err(|error| error.to_string())?;
    }
    runtime
        .model_mut()
        .focus_pane(workspace_id, pane_ids[1])
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, editor_buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_pane(pane_ids[1]);
    Ok(())
}

fn teardown_debug_layout(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(state) = shell_ui_mut(runtime)?.take_debug_layout_state() else {
        return Ok(());
    };
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    for pane_id in &state.created_pane_ids {
        let _ = runtime.model_mut().close_pane(workspace_id, *pane_id);
    }
    let restore_active_pane = state
        .saved_panes
        .get(state.saved_active_pane)
        .map(|pane| pane.pane_id);
    let restore_active_buffer = state
        .saved_panes
        .get(state.saved_active_pane)
        .map(|pane| pane.buffer_id);
    shell_ui_mut(runtime)?.restore_debug_layout_snapshot(state);
    if let Some(pane_id) = restore_active_pane {
        let _ = runtime.model_mut().focus_pane(workspace_id, pane_id);
        shell_ui_mut(runtime)?.focus_pane(pane_id);
    }
    if let Some(buffer_id) = restore_active_buffer {
        let _ = runtime.model_mut().focus_buffer(workspace_id, buffer_id);
    }
    Ok(())
}

fn focus_debug_layout_pane(runtime: &mut EditorRuntime, index: usize) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let pane_id = shell_ui(runtime)?
        .workspace_view()
        .and_then(|view| view.panes.get(index).map(|pane| pane.pane_id))
        .ok_or_else(|| "Debug Layout pane missing".to_owned())?;
    let buffer_id = shell_ui(runtime)?
        .workspace_view()
        .and_then(|view| view.panes.get(index).map(|pane| pane.buffer_id))
        .ok_or_else(|| "Debug Layout pane buffer missing".to_owned())?;
    runtime
        .model_mut()
        .focus_pane(workspace_id, pane_id)
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_pane(pane_id);
    shell_ui_mut(runtime)?.enter_normal_mode();
    Ok(())
}

fn prepare_workspace_leave_for_debug_layout(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.is_debug_layout_active() {
        teardown_debug_layout(runtime)?;
    }
    Ok(())
}

fn prepare_workspace_enter_for_debug_layout(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let Some(dap_client) = runtime.services().get::<Arc<DapClientManager>>().cloned() else {
        return Ok(());
    };
    let live = dap_client
        .session_info(workspace_id.get())
        .map_err(|error| error.to_string())?
        .is_some();
    if live {
        install_debug_layout(runtime)?;
        refresh_dap_fringe_cache(runtime)?;
        let has_stopped = dap_client
            .stopped_snapshot(workspace_id.get())
            .map_err(|error| error.to_string())?
            .is_some();
        if has_stopped {
            apply_dap_stopped_ui(runtime, workspace_id.get())?;
        }
    }
    Ok(())
}
