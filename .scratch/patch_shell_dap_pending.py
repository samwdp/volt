from pathlib import Path
import re

p = Path(r"P:\volt\crates\editor-sdl\src\shell\mod.rs")
text = p.read_text(encoding="utf-8")

# 1) Add pending_dap_start field to ShellUiState
old_fields = '''    /// Per-workspace last-used build command.  Set when the user runs
    /// `workspace.compile`; reused by `workspace.recompile`.
    compile_commands: BTreeMap<WorkspaceId, String>,
    pending_syntax_prewarm_roots: VecDeque<PathBuf>,
    pending_workspace_readme_opens: VecDeque<PathBuf>,
}'''

new_fields = '''    /// Per-workspace last-used build command.  Set when the user runs
    /// `workspace.compile`; reused by `workspace.recompile`.
    compile_commands: BTreeMap<WorkspaceId, String>,
    pending_syntax_prewarm_roots: VecDeque<PathBuf>,
    pending_workspace_readme_opens: VecDeque<PathBuf>,
    /// Pending DAP start waiting on minibuffer hole fill.
    pending_dap_start: Option<PendingDapStartPrompt>,
}'''

if old_fields not in text:
    raise SystemExit("ShellUiState fields not found")
text = text.replace(old_fields, new_fields, 1)

old_init = '''            compile_commands: BTreeMap::new(),
            pending_syntax_prewarm_roots: VecDeque::new(),
            pending_workspace_readme_opens: VecDeque::new(),
        }
    }'''

# Need unique enough context - find ShellUiState::new init
# There may be only one such block
if old_init not in text:
    raise SystemExit("ShellUiState init not found")
text = text.replace(
    old_init,
    '''            compile_commands: BTreeMap::new(),
            pending_syntax_prewarm_roots: VecDeque::new(),
            pending_workspace_readme_opens: VecDeque::new(),
            pending_dap_start: None,
        }
    }''',
    1,
)

# 2) Insert PendingDapStartPrompt type before DAP_PROGRAM_PROMPT_ID
marker = "const DAP_PROGRAM_PROMPT_ID: &str = \"dap-program\";"
pending_type = '''#[derive(Debug, Clone)]
struct PendingDapStartPrompt {
    adapter_id: String,
    configuration: DebugConfiguration,
    ask_heuristic_compile: bool,
    kind: DapStartPromptKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DapStartPromptKind {
    Program,
    ProcessId,
}

const DAP_PROGRAM_PROMPT_ID: &str = "dap-program";'''

if marker not in text:
    raise SystemExit("DAP_PROGRAM_PROMPT_ID not found")
text = text.replace(marker, pending_type, 1)

# 3) Replace open_dap_program_prompt / open_dap_process_prompt / encode/decode / confirm helpers
# Find from open_dap_program_prompt through decode_dap_configuration_prompt end

start = text.find("fn open_dap_program_prompt(")
end = text.find("fn open_dap_compile_confirm_picker(")
if start < 0 or end < 0:
    raise SystemExit(f"prompt helpers markers missing {start} {end}")

replacement = r'''fn open_dap_program_prompt(
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
        kind: DapStartPromptKind::Program,
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
        kind: DapStartPromptKind::ProcessId,
    });
    let overlay = InputPromptOverlay::new(DAP_PROCESS_PROMPT_ID, "Attach process id: ", "");
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

'''

text = text[:start] + replacement + text[end:]

# 4) Replace dispatch_input_prompt_confirm DAP arms and confirm helpers
old_dispatch = '''    if id == COMPILE_PROMPT_ID {
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
    continue_dap_start(runtime, &adapter_id, configuration, false)
}'''

# The original match might still exist - check what's actually there now
# Read around dispatch after previous patch

new_dispatch = '''    match id {
        COMPILE_PROMPT_ID => run_compile_command_streamed(runtime, text),
        DAP_PROGRAM_PROMPT_ID => confirm_dap_program_prompt(runtime, text),
        DAP_PROCESS_PROMPT_ID => confirm_dap_process_prompt(runtime, text),
        _ => Ok(()),
    }
}

fn confirm_dap_program_prompt(runtime: &mut EditorRuntime, text: &str) -> Result<(), String> {
    let program = text.trim();
    if program.is_empty() {
        return Err("Debug program path required".to_owned());
    }
    let pending = shell_ui_mut(runtime)?
        .pending_dap_start
        .take()
        .ok_or_else(|| "dap program prompt has no pending start".to_owned())?;
    let configuration = pending
        .configuration
        .with_target_program(PathBuf::from(program));
    continue_dap_start(
        runtime,
        &pending.adapter_id,
        configuration,
        pending.ask_heuristic_compile,
    )
}

fn confirm_dap_process_prompt(runtime: &mut EditorRuntime, text: &str) -> Result<(), String> {
    let process_id = text
        .trim()
        .parse::<u32>()
        .map_err(|_| "Attach process id must be a number".to_owned())?;
    let pending = shell_ui_mut(runtime)?
        .pending_dap_start
        .take()
        .ok_or_else(|| "dap process prompt has no pending start".to_owned())?;
    let configuration = pending.configuration.with_process_id(process_id);
    continue_dap_start(
        runtime,
        &pending.adapter_id,
        configuration,
        pending.ask_heuristic_compile,
    )
}'''

if old_dispatch not in text:
    # maybe still old match form
    alt = '''    match id {
        COMPILE_PROMPT_ID => run_compile_command_streamed(runtime, text),
        _ => Ok(()),
    }'''
    if alt in text:
        text = text.replace(alt, new_dispatch.rstrip() + "\n", 1)
    else:
        # try the if-based form from previous patch - maybe partial
        raise SystemExit("dispatch block not found; dumping snippet")
else:
    text = text.replace(old_dispatch, new_dispatch.rstrip() + "\n", 1)

# 5) Fix compile-before-debug to use JobManager success directly
old_compile = '''fn run_dap_compile_before_debug(runtime: &mut EditorRuntime, command: &str) -> Result<(), String> {
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
}'''

new_compile = '''fn run_dap_compile_before_debug(runtime: &mut EditorRuntime, command: &str) -> Result<(), String> {
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
}'''

if old_compile not in text:
    raise SystemExit("compile-before-debug fn not found")
text = text.replace(old_compile, new_compile, 1)

p.write_text(text, encoding="utf-8")
print("pending dap prompt + compile fix ok")
