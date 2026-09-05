use std::{

    cell::RefCell,
    collections::{HashMap, VecDeque},
    env,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
    thread,
};

use agent_client_protocol::{Agent, Client, ClientSideConnection};
use agent_client_protocol::{
    AuthCapabilities, AvailableCommand, ClientCapabilities, ContentBlock, CreateTerminalRequest,
    CreateTerminalResponse, Error, FileSystemCapabilities, ImageContent, Implementation,
    InitializeRequest, KillTerminalRequest, KillTerminalResponse, ListSessionsRequest,
    LoadSessionRequest, Meta, ModelId, ModelInfo, NewSessionRequest, PermissionOption,
    PermissionOptionId, PermissionOptionKind, Plan, ProtocolVersion, ReadTextFileRequest,
    ReadTextFileResponse, ReleaseTerminalRequest, ReleaseTerminalResponse,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse, ResourceLink,
    SelectedPermissionOutcome, SessionConfigId, SessionConfigKind, SessionConfigOption,
    SessionConfigOptionCategory, SessionConfigSelectOption, SessionConfigSelectOptions,
    SessionConfigValueId, SessionInfo, SessionInfoUpdate, SessionMode, SessionModeId,
    SessionModeState, SessionModelState, SessionNotification, SessionUpdate,
    SetSessionConfigOptionRequest, SetSessionModeRequest, SetSessionModelRequest, StopReason,
    TerminalExitStatus, TerminalId, TerminalOutputRequest, TerminalOutputResponse, ToolCall,
    ToolCallUpdate, WaitForTerminalExitRequest, WaitForTerminalExitResponse, WriteTextFileRequest,
    WriteTextFileResponse,
};
use async_trait::async_trait;
use editor_jobs::{ProcessSupervisionMode, supervised_command_if_resolved};
use editor_picker::PickerResultOrder;
use editor_plugin_api::AcpClient as AcpClientConfig;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
    sync::{mpsc as tokio_mpsc, oneshot},
    task::LocalSet,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const ACP_EVENT_DRAIN_LIMIT: usize = 64;

fn configure_background_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

#[derive(Clone, Copy)]
struct BackgroundCommandPipes {
    stdin: bool,
    stdout: bool,
    stderr: bool,
}

impl BackgroundCommandPipes {
    const ACP_CLIENT: Self = Self {
        stdin: true,
        stdout: true,
        stderr: true,
    };

    const TERMINAL: Self = Self {
        stdin: false,
        stdout: true,
        stderr: true,
    };
}

async fn spawn_background_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    pipes: BackgroundCommandPipes,
) -> std::io::Result<tokio::process::Child> {
    let mut spawn_result = build_background_command(program, args, cwd, env, pipes, None).spawn();

    let should_retry = matches!(
        &spawn_result,
        Err(error) if background_spawn_should_retry(error)
    );
    if should_retry {
        for candidate in background_command_candidates(program, env, None) {
            spawn_result =
                build_background_command(&candidate, args, cwd, env, pipes, None).spawn();
            match &spawn_result {
                Ok(_) => break,
                Err(error) if background_spawn_should_retry(error) => {}
                Err(_) => break,
            }
        }
    }

    // `launch_env` carries the refreshed environment into the Windows-only node-manager
    // retry below; on other platforms the initial value is intentionally never read.
    #[cfg_attr(not(windows), allow(unused_assignments))]
    let mut launch_env = None;
    let should_retry = matches!(
        &spawn_result,
        Err(error) if background_spawn_should_retry(error)
    );
    if should_retry {
        launch_env = refreshed_launch_environment(cwd).await;
        if let Some(launch_env) = launch_env.as_deref() {
            for candidate in background_command_candidates(program, env, Some(launch_env)) {
                spawn_result =
                    build_background_command(&candidate, args, cwd, env, pipes, Some(launch_env))
                        .spawn();
                match &spawn_result {
                    Ok(_) => break,
                    Err(error) if background_spawn_should_retry(error) => {}
                    Err(_) => break,
                }
            }
        }
    }

    #[cfg(windows)]
    {
        let should_retry_with_node_manager = matches!(
            &spawn_result,
            Err(error) if background_spawn_should_retry(error)
        );
        if should_retry_with_node_manager
            && let Some(node_manager_env) =
                windows_node_manager_environment(cwd, env, launch_env.as_deref()).await
        {
            for candidate in background_command_candidates(program, &[], Some(&node_manager_env)) {
                spawn_result = build_background_command(
                    &candidate,
                    args,
                    cwd,
                    &[],
                    pipes,
                    Some(&node_manager_env),
                )
                .spawn();
                match &spawn_result {
                    Ok(_) => break,
                    Err(error) if background_spawn_should_retry(error) => {}
                    Err(_) => break,
                }
            }
        }
    }

    spawn_result
}

fn build_background_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    pipes: BackgroundCommandPipes,
    launch_env: Option<&[(String, String)]>,
) -> Command {
    let (program, args) = supervised_command_if_resolved(
        program,
        args,
        env,
        launch_env,
        ProcessSupervisionMode::Background,
    );
    let mut command = Command::new(&program);
    configure_background_command(&mut command);
    command.args(&args);
    apply_background_pipes(&mut command, pipes);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    if let Some(launch_env) = launch_env {
        apply_launch_environment(&mut command, env, launch_env);
    } else {
        apply_command_environment(&mut command, env);
    }
    command
}

fn apply_background_pipes(command: &mut Command, pipes: BackgroundCommandPipes) {
    if pipes.stdin {
        command.stdin(std::process::Stdio::piped());
    }
    if pipes.stdout {
        command.stdout(std::process::Stdio::piped());
    }
    if pipes.stderr {
        command.stderr(std::process::Stdio::piped());
    }
}

fn apply_command_environment(command: &mut Command, env: &[(String, String)]) {
    for (key, value) in env {
        command.env(key, value);
    }
}

fn apply_launch_environment(
    command: &mut Command,
    env: &[(String, String)],
    launch_env: &[(String, String)],
) {
    for (key, value) in launch_env {
        command.env(key, value);
    }
    apply_command_environment(command, env);
}

fn background_command_candidates(
    program: &str,
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Vec<String> {
    if Path::new(program).components().count() != 1 {
        return Vec::new();
    }

    let Some(path_value) = environment_value(env, launch_env, "PATH") else {
        return Vec::new();
    };

    let names = background_command_names(program, env, launch_env);
    let mut candidates = Vec::new();
    for directory in path_value
        .split(path_list_separator())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        for name in &names {
            let candidate = Path::new(directory).join(name);
            if is_launch_candidate(&candidate) {
                let candidate = candidate.to_string_lossy().into_owned();
                if !candidates.iter().any(|existing| existing == &candidate) {
                    candidates.push(candidate);
                }
            }
        }
    }
    candidates
}

fn background_command_names(
    program: &str,
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Vec<String> {
    #[cfg(windows)]
    {
        if Path::new(program).extension().is_some() {
            return vec![program.to_owned()];
        }

        let mut names = windows_command_extensions(env, launch_env)
            .into_iter()
            .map(|extension| format!("{program}{extension}"))
            .collect::<Vec<_>>();
        names.push(program.to_owned());
        names.dedup();
        names
    }

    #[cfg(not(windows))]
    {
        let _ = (env, launch_env);
        vec![program.to_owned()]
    }
}

fn environment_value(
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
    key: &str,
) -> Option<String> {
    explicit_environment_value(env, key)
        .cloned()
        .or_else(|| launch_env.and_then(|vars| explicit_environment_value(vars, key).cloned()))
        .or_else(|| std::env::var(key).ok())
}

fn explicit_environment_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a String> {
    env.iter().find_map(|(entry_key, value)| {
        #[cfg(windows)]
        {
            entry_key.eq_ignore_ascii_case(key).then_some(value)
        }
        #[cfg(not(windows))]
        {
            (entry_key == key).then_some(value)
        }
    })
}

#[cfg(windows)]
fn windows_command_extensions(
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Vec<String> {
    environment_value(env, launch_env, "PATHEXT")
        .map(|value| {
            value
                .split(';')
                .map(str::trim)
                .filter(|extension| !extension.is_empty())
                .map(|extension| extension.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .filter(|extensions| !extensions.is_empty())
        .unwrap_or_else(|| {
            [".com", ".exe", ".bat", ".cmd"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        })
}

#[cfg(windows)]
fn path_list_separator() -> char {
    ';'
}

#[cfg(not(windows))]
fn path_list_separator() -> char {
    ':'
}

#[cfg(windows)]
fn is_launch_candidate(candidate: &Path) -> bool {
    candidate.is_file()
}

#[cfg(not(windows))]
fn is_launch_candidate(candidate: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;

    candidate
        .metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn background_spawn_should_retry(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(193)
}

#[cfg(not(windows))]
fn background_spawn_should_retry(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::NotFound
}

#[cfg(windows)]
async fn windows_node_manager_environment(
    cwd: Option<&Path>,
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Option<Vec<(String, String)>> {
    if let Some(fnm_env) = windows_fnm_environment(cwd, env, launch_env).await {
        return Some(merge_node_manager_environment(env, launch_env, fnm_env));
    }
    windows_nvm_environment(cwd, env, launch_env)
        .await
        .map(|nvm_env| merge_node_manager_environment(env, launch_env, nvm_env))
}

#[cfg(windows)]
async fn windows_fnm_environment(
    cwd: Option<&Path>,
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Option<Vec<(String, String)>> {
    let program = editor_jobs::resolve_command_path("fnm", env, launch_env)?;
    let mut command = Command::new(program);
    configure_background_command(&mut command);
    command
        .args(["env", "--shell", "cmd"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    if let Some(launch_env) = launch_env {
        apply_launch_environment(&mut command, env, launch_env);
    } else {
        apply_command_environment(&mut command, env);
    }
    let output = command.output().await.ok()?;
    output.status.success().then_some(())?;
    parse_windows_cmd_environment(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(windows)]
async fn windows_nvm_environment(
    cwd: Option<&Path>,
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Option<Vec<(String, String)>> {
    let settings = windows_nvm_settings(env, launch_env);
    let nvm_program = editor_jobs::resolve_command_path("nvm", env, launch_env);
    let current = if let Some(program) = nvm_program.as_deref() {
        windows_nvm_command_stdout(program, ["current"], cwd, env, launch_env)
            .await
            .and_then(|output| parse_windows_nvm_current_output(&output))
    } else {
        None
    };
    let command_root = if let Some(program) = nvm_program.as_deref() {
        windows_nvm_command_stdout(program, ["root"], cwd, env, launch_env)
            .await
            .and_then(|output| parse_windows_nvm_root_output(&output))
    } else {
        None
    };
    let root = command_root
        .or(settings.root)
        .or_else(|| environment_value(env, launch_env, "NVM_HOME").map(PathBuf::from))
        .or_else(|| windows_default_nvm_home(env, launch_env));
    let symlink = settings
        .path
        .or_else(|| environment_value(env, launch_env, "NVM_SYMLINK").map(PathBuf::from));
    windows_nvm_environment_from_parts(root, symlink, current.as_deref(), env, launch_env)
}

#[cfg(windows)]
async fn windows_nvm_command_stdout(
    program: &str,
    args: impl IntoIterator<Item = &'static str>,
    cwd: Option<&Path>,
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Option<String> {
    let mut command = Command::new(program);
    configure_background_command(&mut command);
    command
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    if let Some(launch_env) = launch_env {
        apply_launch_environment(&mut command, env, launch_env);
    } else {
        apply_command_environment(&mut command, env);
    }
    let output = command.output().await.ok()?;
    output.status.success().then_some(())?;
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(windows)]
async fn refreshed_launch_environment(cwd: Option<&Path>) -> Option<Vec<(String, String)>> {
    let system_root = env::var_os("SystemRoot")
        .or_else(|| env::var_os("WINDIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let powershell = system_root
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    let script = r#"
$machine = [Environment]::GetEnvironmentVariables('Machine')
$user = [Environment]::GetEnvironmentVariables('User')
$result = @{}
foreach ($key in $machine.Keys) {
    $name = [string]$key
    $result[$name] = [Environment]::ExpandEnvironmentVariables([string]$machine[$key])
}
foreach ($key in $user.Keys) {
    $name = [string]$key
    $value = [Environment]::ExpandEnvironmentVariables([string]$user[$key])
    if ($name -ieq 'Path' -and $result.ContainsKey('Path')) {
        if ([string]::IsNullOrEmpty($value)) {
            continue
        }
        if ([string]::IsNullOrEmpty([string]$result['Path'])) {
            $result['Path'] = $value
        } else {
            $result['Path'] = '{0};{1}' -f $result['Path'], $value
        }
        continue
    }
    $result[$name] = $value
}
$result.GetEnumerator() | ForEach-Object { '{0}={1}' -f $_.Key, $_.Value }
"#;

    let mut command = Command::new(powershell);
    configure_background_command(&mut command);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command.output().await.ok()?;
    output.status.success().then_some(())?;
    parse_line_environment(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn refreshed_launch_environment(cwd: Option<&Path>) -> Option<Vec<(String, String)>> {
    let shell = env::var_os("SHELL")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("/bin/sh"));
    let shell_name = shell
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let shell_args: &[&str] = match shell_name.as_str() {
        "bash" | "zsh" | "fish" | "ksh" | "mksh" => &["-l", "-c", "env -0"],
        _ => &["-c", "env -0"],
    };

    let mut command = Command::new(shell);
    configure_background_command(&mut command);
    command
        .args(shell_args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command.output().await.ok()?;
    output.status.success().then_some(())?;
    parse_nul_environment(&output.stdout)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
async fn refreshed_launch_environment(_cwd: Option<&Path>) -> Option<Vec<(String, String)>> {
    None
}

#[cfg(windows)]
fn parse_line_environment(output: &str) -> Option<Vec<(String, String)>> {
    let vars = output
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            (!key.is_empty()).then_some((key.to_owned(), value.to_owned()))
        })
        .collect::<Vec<_>>();
    (!vars.is_empty()).then_some(vars)
}

#[cfg(windows)]
fn parse_windows_cmd_environment(output: &str) -> Option<Vec<(String, String)>> {
    let vars = output
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("SET ")?;
            let (key, value) = rest.split_once('=')?;
            (!key.is_empty()).then_some((key.to_owned(), value.to_owned()))
        })
        .collect::<Vec<_>>();
    (!vars.is_empty()).then_some(vars)
}

#[cfg(windows)]
#[derive(Default)]
struct WindowsNvmSettings {
    root: Option<PathBuf>,
    path: Option<PathBuf>,
}

#[cfg(windows)]
fn windows_nvm_settings(
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> WindowsNvmSettings {
    windows_nvm_settings_paths(env, launch_env)
        .into_iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .map(|content| parse_windows_nvm_settings(&content))
        .unwrap_or_default()
}

#[cfg(windows)]
fn windows_nvm_settings_paths(
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = environment_value(env, launch_env, "NVM_HOME") {
        paths.push(PathBuf::from(home).join("settings.txt"));
    }
    if let Some(appdata) = environment_value(env, launch_env, "APPDATA") {
        let path = PathBuf::from(appdata).join("nvm").join("settings.txt");
        if !paths.iter().any(|existing| existing == &path) {
            paths.push(path);
        }
    }
    paths
}

#[cfg(windows)]
fn parse_windows_nvm_settings(content: &str) -> WindowsNvmSettings {
    let mut settings = WindowsNvmSettings::default();
    for line in content.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        if value.is_empty() {
            continue;
        }
        match key.trim().to_ascii_lowercase().as_str() {
            "root" => settings.root = Some(PathBuf::from(value)),
            "path" => settings.path = Some(PathBuf::from(value)),
            _ => {}
        }
    }
    settings
}

#[cfg(windows)]
fn parse_windows_nvm_current_output(output: &str) -> Option<String> {
    let current = output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    (!current.to_ascii_lowercase().starts_with("no current")).then(|| current.to_owned())
}

#[cfg(windows)]
fn parse_windows_nvm_root_output(output: &str) -> Option<PathBuf> {
    let root = output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    let root = root
        .strip_prefix("Current Root:")
        .unwrap_or(root)
        .trim()
        .trim_matches('"');
    (!root.is_empty()).then(|| PathBuf::from(root))
}

#[cfg(windows)]
fn windows_default_nvm_home(
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Option<PathBuf> {
    environment_value(env, launch_env, "APPDATA").map(|appdata| PathBuf::from(appdata).join("nvm"))
}

#[cfg(windows)]
fn windows_nvm_environment_from_parts(
    root: Option<PathBuf>,
    symlink: Option<PathBuf>,
    current: Option<&str>,
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Option<Vec<(String, String)>> {
    let mut node_dirs = Vec::new();
    if let (Some(root), Some(current)) = (root.as_ref(), current) {
        for candidate in windows_nvm_version_dir_candidates(root, current) {
            if candidate.join("node.exe").is_file() {
                push_unique_path(&mut node_dirs, candidate);
            }
        }
    }
    if let Some(symlink) = symlink.as_ref()
        && symlink.join("node.exe").is_file()
    {
        push_unique_path(&mut node_dirs, symlink.clone());
    }
    if node_dirs.is_empty() {
        return None;
    }

    let existing_path = environment_value(env, launch_env, "PATH").unwrap_or_default();
    let mut path_parts = node_dirs
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if !existing_path.is_empty() {
        path_parts.push(existing_path);
    }

    let mut vars = vec![("PATH".to_owned(), path_parts.join(";"))];
    if let Some(root) = root {
        vars.push(("NVM_HOME".to_owned(), root.to_string_lossy().into_owned()));
    }
    if let Some(symlink) = symlink {
        vars.push((
            "NVM_SYMLINK".to_owned(),
            symlink.to_string_lossy().into_owned(),
        ));
    }
    Some(vars)
}

#[cfg(windows)]
fn windows_nvm_version_dir_candidates(root: &Path, current: &str) -> Vec<PathBuf> {
    let current = current.trim();
    let without_v = current.strip_prefix('v').unwrap_or(current);
    let with_v = format!("v{without_v}");
    let mut candidates = Vec::new();
    push_unique_path(&mut candidates, root.join(current));
    push_unique_path(&mut candidates, root.join(without_v));
    push_unique_path(&mut candidates, root.join(with_v));
    candidates
}

#[cfg(windows)]
fn merge_node_manager_environment(
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
    manager_env: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let explicit_path = explicit_environment_value(env, "PATH");
    let mut merged = launch_env.map_or_else(Vec::new, |vars| vars.to_vec());
    let mut manager_path_seen = false;

    for (key, value) in manager_env {
        if key.eq_ignore_ascii_case("PATH") {
            manager_path_seen = true;
            let value = explicit_path
                .map(|path| format!("{value};{path}"))
                .unwrap_or(value);
            upsert_environment_value(&mut merged, key, value);
        } else {
            upsert_environment_value(&mut merged, key, value);
        }
    }

    for (key, value) in env {
        if !key.eq_ignore_ascii_case("PATH") {
            upsert_environment_value(&mut merged, key.clone(), value.clone());
        }
    }
    if !manager_path_seen && let Some(path) = explicit_path {
        upsert_environment_value(&mut merged, "PATH".to_owned(), path.clone());
    }
    merged
}

#[cfg(windows)]
fn upsert_environment_value(env: &mut Vec<(String, String)>, key: String, value: String) {
    if let Some((_, existing_value)) = env
        .iter_mut()
        .find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(&key))
    {
        *existing_value = value;
    } else {
        env.push((key, value));
    }
}

#[cfg(windows)]
fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn parse_nul_environment(output: &[u8]) -> Option<Vec<(String, String)>> {
    let vars = output
        .split(|byte| *byte == 0)
        .filter_map(|entry| {
            if entry.is_empty() {
                return None;
            }
            let line = String::from_utf8_lossy(entry);
            let (key, value) = line.split_once('=')?;
            (!key.is_empty()).then_some((key.to_owned(), value.to_owned()))
        })
        .collect::<Vec<_>>();
    (!vars.is_empty()).then_some(vars)
}

pub(super) fn init_acp_manager(runtime: &mut EditorRuntime) -> Result<(), ShellError> {
    let manager = AcpManager::new().map_err(ShellError::Runtime)?;
    runtime.services_mut().insert(Arc::new(Mutex::new(manager)));
    Ok(())
}

pub(super) fn refresh_pending_acp(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let Some(manager) = runtime.services().get::<Arc<Mutex<AcpManager>>>().cloned() else {
        return Ok(false);
    };
    let (events_changed, actions) = {
        let mut manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        let events_changed = manager.drain_events(runtime)?;
        (events_changed, manager.take_pending_ui_actions())
    };
    let mut changed = events_changed || !actions.is_empty();
    for action in actions {
        handle_acp_ui_action(runtime, action)?;
        changed = true;
    }
    if changed {
        remap_acp_output_visual_anchors(runtime)?;
    }
    Ok(changed)
}

pub(super) fn acp_connected(runtime: &EditorRuntime) -> Result<bool, String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    Ok(manager.has_sessions())
}

pub(super) fn open_acp_client(runtime: &mut EditorRuntime, client_id: &str) -> Result<(), String> {
    open_acp_client_buffer(runtime, client_id, true, None).map(|_| ())
}

pub(super) fn acp_new_session(runtime: &mut EditorRuntime) -> Result<(), String> {
    let client = active_acp_client(runtime)?;
    open_acp_client_with_config(runtime, client, false, None).map(|_| ())
}

pub(super) fn close_acp_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let Some(manager) = runtime.services().get::<Arc<Mutex<AcpManager>>>().cloned() else {
        return Ok(());
    };
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.close_buffer(buffer_id);
    Ok(())
}

pub(super) fn close_acp_workspace_buffers(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let Some(manager) = runtime.services().get::<Arc<Mutex<AcpManager>>>().cloned() else {
        return Ok(());
    };
    let buffer_ids = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffers()
        .filter(|buffer| {
            matches!(
                buffer.kind(),
                BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND
            )
        })
        .map(|buffer| buffer.id())
        .collect::<Vec<_>>();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    for buffer_id in buffer_ids {
        manager.close_buffer(buffer_id);
    }
    Ok(())
}

fn open_acp_client_buffer(
    runtime: &mut EditorRuntime,
    client_id: &str,
    reuse_existing: bool,
    load_session: Option<PendingAcpLoadSession>,
) -> Result<BufferId, String> {
    let client = shell_user_library(runtime)
        .acp_client_by_id(client_id)
        .ok_or_else(|| format!("unknown ACP client `{client_id}`"))?;
    open_acp_client_with_config(runtime, client, reuse_existing, load_session)
}

fn open_acp_client_with_config(
    runtime: &mut EditorRuntime,
    client: AcpClientConfig,
    reuse_existing: bool,
    load_session: Option<PendingAcpLoadSession>,
) -> Result<BufferId, String> {
    let active_workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    if reuse_existing
        && let Some(buffer_id) = {
            let manager = manager
                .lock()
                .map_err(|_| "acp manager lock was poisoned".to_owned())?;
            manager.buffer_for_client(active_workspace_id, &client.id)
        }
    {
        if shell_ui(runtime)
            .ok()
            .and_then(|ui| ui.buffer(buffer_id))
            .is_none()
        {
            let mut manager = manager
                .lock()
                .map_err(|_| "acp manager lock was poisoned".to_owned())?;
            manager.close_buffer(buffer_id);
        } else {
            focus_acp_buffer(runtime, buffer_id)?;
            return Ok(buffer_id);
        }
    }

    let (buffer_id, workspace_id, workspace_name) = create_acp_buffer(runtime, &client)?;
    let workspace_root = active_workspace_root(runtime)?
        .or_else(|| env::current_dir().ok())
        .ok_or_else(|| "ACP requires a workspace root or current directory".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.connect(
        client,
        workspace_root,
        workspace_id,
        buffer_id,
        load_session,
        workspace_name,
    )?;
    Ok(buffer_id)
}

fn create_acp_buffer(
    runtime: &mut EditorRuntime,
    client: &AcpClientConfig,
) -> Result<(BufferId, WorkspaceId, String), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace_name = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .name()
        .to_owned();
    let buffer_name = format!("*acp {}*", client.label);
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            buffer_name.as_str(),
            BufferKind::Plugin(ACP_BUFFER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, Vec::new(), &*user_library);
    shell_buffer.init_acp_view(&client.label);
    shell_buffer.clear_input();
    shell_buffer.set_forced_language_id("markdown");
    shell_ui_mut(runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
    Ok((buffer_id, workspace_id, workspace_name))
}

pub(super) fn focus_acp_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
    Ok(())
}
