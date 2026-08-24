from pathlib import Path

p = Path(r"P:\volt\crates\editor-sdl\src\shell\mod.rs")
text = p.read_text(encoding="utf-8")

# Insert last/recent hook subscriptions after START subscription block
old_reg = '''    if runtime.hooks().contains(HOOK_DAP_START) {
        runtime
            .subscribe_hook(
                HOOK_DAP_START,
                "shell.start-dap-session",
                |event, runtime| start_dap_for_active_workspace(runtime, event.detail.as_deref()),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_STOP) {'''

new_reg = '''    if runtime.hooks().contains(HOOK_DAP_START) {
        runtime
            .subscribe_hook(
                HOOK_DAP_START,
                "shell.start-dap-session",
                |event, runtime| start_dap_for_active_workspace(runtime, event.detail.as_deref()),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_START_LAST) {
        runtime
            .subscribe_hook(
                HOOK_DAP_START_LAST,
                "shell.start-dap-last",
                |_, runtime| start_dap_last(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_START_RECENT) {
        runtime
            .subscribe_hook(
                HOOK_DAP_START_RECENT,
                "shell.start-dap-recent",
                |_, runtime| open_dap_start_recent_picker(runtime),
            )
            .map_err(|error| error.to_string())?;
    }
    if runtime.hooks().contains(HOOK_DAP_STOP) {'''

if old_reg not in text:
    raise SystemExit("register block not found")
text = text.replace(old_reg, new_reg, 1)

old_start = '''fn start_dap_for_active_workspace(
    runtime: &mut EditorRuntime,
    preferred_adapter_id: Option<&str>,
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

    let (extension, target_program, cwd) = dap_start_context(runtime)?;
    let configuration = DebugConfiguration::new("Debug", DebugRequestKind::Launch)
        .with_target_program(target_program)
        .with_cwd(cwd);

    dap_client
        .start(
            workspace_id.get(),
            preferred_adapter_id,
            extension.as_deref(),
            configuration,
        )
        .map_err(|error| error.to_string())?;
    install_debug_layout(runtime)?;
    refresh_dap_fringe_cache(runtime)?;
    refresh_dap_sessions_buffer(runtime, workspace_id)?;
    Ok(())
}'''

new_start = r'''const DAP_PROGRAM_PROMPT_ID: &str = "dap-program";
const DAP_PROCESS_PROMPT_ID: &str = "dap-process";

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
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries(
        "Choose Debug Adapter",
        entries,
    ));
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
        return continue_dap_start(
            runtime,
            adapter_id,
            candidate.into_configuration(),
            true,
        );
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
            let pinned = candidate
                .adapter_id()
                .unwrap_or(adapter_id)
                .to_owned();
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
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries(
        "Choose Debug Configuration",
        entries,
    ));
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
    let prefill = configuration
        .as_ref()
        .and_then(|config| config.target_program())
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let payload = serde_json::json!({
        "adapter_id": adapter_id,
        "configuration": encode_dap_configuration_prompt(configuration),
    })
    .to_string();
    let overlay = InputPromptOverlay::new(
        format!("{DAP_PROGRAM_PROMPT_ID}:{payload}"),
        "Debug program: ",
        &prefill,
    );
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn open_dap_process_prompt(
    runtime: &mut EditorRuntime,
    adapter_id: &str,
    configuration: DebugConfiguration,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "adapter_id": adapter_id,
        "configuration": encode_dap_configuration_prompt(Some(configuration)),
    })
    .to_string();
    let overlay = InputPromptOverlay::new(
        format!("{DAP_PROCESS_PROMPT_ID}:{payload}"),
        "Attach process id: ",
        "",
    );
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn encode_dap_configuration_prompt(configuration: Option<DebugConfiguration>) -> serde_json::Value {
    let Some(configuration) = configuration else {
        return serde_json::Value::Null;
    };
    serde_json::json!({
        "name": configuration.name(),
        "request": match configuration.request() {
            DebugRequestKind::Launch => "launch",
            DebugRequestKind::Attach => "attach",
        },
        "program": configuration.target_program().map(|path| path.display().to_string()),
        "cwd": configuration.cwd().map(|path| path.display().to_string()),
        "args": configuration.args(),
        "adapter_id": configuration.adapter_id(),
        "compile": configuration.compile_command(),
        "process_id": configuration.process_id(),
    })
}

fn decode_dap_configuration_prompt(value: &serde_json::Value) -> Result<DebugConfiguration, String> {
    if value.is_null() {
        return Ok(DebugConfiguration::new("Debug", DebugRequestKind::Launch));
    }
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Debug");
    let request = match value.get("request").and_then(|v| v.as_str()).unwrap_or("launch") {
        "attach" => DebugRequestKind::Attach,
        _ => DebugRequestKind::Launch,
    };
    let mut configuration = DebugConfiguration::new(name, request);
    if let Some(program) = value.get("program").and_then(|v| v.as_str()) {
        configuration = configuration.with_target_program(PathBuf::from(program));
    }
    if let Some(cwd) = value.get("cwd").and_then(|v| v.as_str()) {
        configuration = configuration.with_cwd(PathBuf::from(cwd));
    }
    if let Some(args) = value.get("args").and_then(|v| v.as_array()) {
        let args = args
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect::<Vec<_>>();
        if !args.is_empty() {
            configuration = configuration.with_args(args);
        }
    }
    if let Some(adapter) = value.get("adapter_id").and_then(|v| v.as_str()) {
        configuration = configuration.with_adapter_id(adapter);
    }
    if let Some(compile) = value.get("compile").and_then(|v| v.as_str()) {
        configuration = configuration.with_compile_command(compile);
    }
    if let Some(process_id) = value.get("process_id").and_then(|v| v.as_u64()) {
        configuration = configuration.with_process_id(process_id as u32);
    }
    Ok(configuration)
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
    if let Some(command) = compile_command {
        run_dap_compile_before_debug(runtime, &command)?;
    }
    finish_dap_session_start(runtime, adapter_id, configuration)
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
            format!("# compile-before-debug"),
            format!("$ {command}"),
            String::new(),
        ]);
    }
    run_shell_command_in_buffer(runtime, buffer_id, command)?;
    let succeeded = shell_buffer(runtime, buffer_id)?
        .lines()
        .iter()
        .any(|line| line.contains("Command succeeded"));
    if !succeeded {
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
}'''

if old_start not in text:
    raise SystemExit("start_dap block not found")
text = text.replace(old_start, new_start, 1)

# Replace dap_start_context
old_ctx = '''fn dap_start_context(
    runtime: &EditorRuntime,
) -> Result<(Option<String>, PathBuf, PathBuf), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let cwd = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .root()
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| "dap.start needs a Workspace root or current directory".to_owned())?;

    let buffer_id = active_shell_buffer_id(runtime).ok();
    let path = buffer_id.and_then(|buffer_id| {
        shell_ui(runtime)
            .ok()
            .and_then(|ui| ui.buffer(buffer_id))
            .and_then(|buffer| buffer.path().map(Path::to_path_buf))
    });
    let extension = path
        .as_ref()
        .and_then(|path| path.extension())
        .and_then(|ext| ext.to_str())
        .map(str::to_owned);
    let target_program = path.ok_or_else(|| {
        "dap.start needs an open file to infer the program, or an explicit Debug Configuration"
            .to_owned()
    })?;
    Ok((extension, target_program, cwd))
}'''

new_ctx = '''fn dap_start_context(runtime: &EditorRuntime) -> Result<DapStartContext, String> {
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
}'''

if old_ctx not in text:
    raise SystemExit("dap_start_context not found")
text = text.replace(old_ctx, new_ctx, 1)

# Update input prompt dispatch
old_prompt = '''    match id {
        COMPILE_PROMPT_ID => run_compile_command_streamed(runtime, text),
        _ => Ok(()),
    }'''

new_prompt = '''    if id == COMPILE_PROMPT_ID {
        return run_compile_command_streamed(runtime, text);
    }
    if let Some(payload) = id.strip_prefix(&format!("{DAP_PROGRAM_PROMPT_ID}:")) {
        return confirm_dap_program_prompt(runtime, payload, text);
    }
    if let Some(payload) = id.strip_prefix(&format!("{DAP_PROCESS_PROMPT_ID}:")) {
        return confirm_dap_process_prompt(runtime, payload, text);
    }
    Ok(())
}

fn confirm_dap_program_prompt(
    runtime: &mut EditorRuntime,
    payload: &str,
    text: &str,
) -> Result<(), String> {
    let program = text.trim();
    if program.is_empty() {
        return Err("Debug program path required".to_owned());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|error| error.to_string())?;
    let adapter_id = value
        .get("adapter_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "dap program prompt missing adapter id".to_owned())?
        .to_owned();
    let mut configuration = decode_dap_configuration_prompt(value.get("configuration").unwrap_or(&serde_json::Value::Null))?;
    configuration = configuration.with_target_program(PathBuf::from(program));
    continue_dap_start(runtime, &adapter_id, configuration, true)
}

fn confirm_dap_process_prompt(
    runtime: &mut EditorRuntime,
    payload: &str,
    text: &str,
) -> Result<(), String> {
    let process_id = text
        .trim()
        .parse::<u32>()
        .map_err(|_| "Attach process id must be a number".to_owned())?;
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|error| error.to_string())?;
    let adapter_id = value
        .get("adapter_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "dap process prompt missing adapter id".to_owned())?
        .to_owned();
    let mut configuration = decode_dap_configuration_prompt(value.get("configuration").unwrap_or(&serde_json::Value::Null))?;
    configuration = configuration.with_process_id(process_id);
    continue_dap_start(runtime, &adapter_id, configuration, false)'''

if old_prompt not in text:
    raise SystemExit("prompt dispatch not found")
text = text.replace(old_prompt, new_prompt, 1)

p.write_text(text, encoding="utf-8")
print("shell dap start flow patched")
