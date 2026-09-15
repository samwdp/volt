use std::{
    env,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use editor_jobs::{
    OwnedProcessId, ProcessLaunchSpec, ProcessLaunchStdio, ProcessRegistry, ProcessSupervisionMode,
    WorkspaceId as ProcessWorkspaceId,
};
use tokio::process::Command;

#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) const ACP_EVENT_DRAIN_LIMIT: usize = 64;

pub(crate) fn configure_background_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BackgroundCommandPipes {
    pub(crate) stdin: bool,
    pub(crate) stdout: bool,
    pub(crate) stderr: bool,
}

impl BackgroundCommandPipes {
    pub(crate) const ACP_CLIENT: Self = Self {
        stdin: true,
        stdout: true,
        stderr: false,
    };

    pub(crate) const TERMINAL: Self = Self {
        stdin: false,
        stdout: true,
        stderr: true,
    };

    pub(crate) fn to_launch_stdio(self) -> ProcessLaunchStdio {
        if self.stdin && self.stdout && !self.stderr {
            ProcessLaunchStdio::Protocol
        } else if self.stdin || self.stdout || self.stderr {
            ProcessLaunchStdio::Piped
        } else {
            ProcessLaunchStdio::Null
        }
    }
}

pub(crate) struct OwnedBackgroundLaunch {
    pub(crate) owned_process_id: OwnedProcessId,
    pub(crate) stdin: Option<tokio::process::ChildStdin>,
    pub(crate) stdout: Option<tokio::process::ChildStdout>,
    pub(crate) stderr: Option<tokio::process::ChildStderr>,
}

/// One Process Launch try: program, env pairs, argv.
type LaunchAttempt = (String, Vec<(String, String)>, Vec<String>);

pub(crate) async fn launch_owned_background_command(
    process_registry: &Arc<Mutex<ProcessRegistry>>,
    workspace_id: ProcessWorkspaceId,
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    pipes: BackgroundCommandPipes,
) -> Result<OwnedBackgroundLaunch, String> {
    let stdio = pipes.to_launch_stdio();
    let mut attempts: Vec<LaunchAttempt> = Vec::new();
    // Prefer a PATH-resolved absolute program first so Process Launch can wrap the
    // real shim (.cmd/.exe) instead of a bare name that only the supervisor fails on.
    let initial =
        editor_jobs::resolve_command_path(program, env, None).unwrap_or_else(|| program.to_owned());
    push_launch_attempt(&mut attempts, initial, args, env);

    for candidate in background_command_candidates(program, env, None) {
        if attempts
            .iter()
            .any(|(existing, _, _)| existing == &candidate)
        {
            continue;
        }
        push_launch_attempt(&mut attempts, candidate, args, env);
    }

    let mut last_error = None;
    for (candidate, attempt_env, attempt_args) in &attempts {
        match try_owned_launch(
            process_registry,
            workspace_id,
            candidate,
            attempt_args,
            cwd,
            attempt_env,
            stdio,
        ) {
            Ok(launched) => return Ok(launched),
            Err(error) if owned_launch_should_retry(&error) => {
                last_error = Some(error);
            }
            Err(error) => return Err(error),
        }
    }

    if let Some(launch_env) = refreshed_launch_environment(cwd).await {
        for candidate in background_command_candidates(program, env, Some(&launch_env)) {
            let mut merged = launch_env.clone();
            merged.extend(env.iter().cloned());
            let mut retry_attempts = Vec::new();
            push_launch_attempt(&mut retry_attempts, candidate, args, &merged);
            for (candidate, attempt_env, attempt_args) in &retry_attempts {
                match try_owned_launch(
                    process_registry,
                    workspace_id,
                    candidate,
                    attempt_args,
                    cwd,
                    attempt_env,
                    stdio,
                ) {
                    Ok(launched) => return Ok(launched),
                    Err(error) if owned_launch_should_retry(&error) => {
                        last_error = Some(error);
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        #[cfg(windows)]
        if let Some(node_manager_env) =
            windows_node_manager_environment(cwd, env, Some(&launch_env)).await
        {
            for candidate in background_command_candidates(program, &[], Some(&node_manager_env)) {
                let mut retry_attempts = Vec::new();
                push_launch_attempt(&mut retry_attempts, candidate, args, &node_manager_env);
                for (candidate, attempt_env, attempt_args) in &retry_attempts {
                    match try_owned_launch(
                        process_registry,
                        workspace_id,
                        candidate,
                        attempt_args,
                        cwd,
                        attempt_env,
                        stdio,
                    ) {
                        Ok(launched) => return Ok(launched),
                        Err(error) if owned_launch_should_retry(&error) => {
                            last_error = Some(error);
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| format!("failed to launch `{program}`")))
}

fn push_launch_attempt(
    attempts: &mut Vec<LaunchAttempt>,
    program: String,
    args: &[String],
    env: &[(String, String)],
) {
    #[cfg(windows)]
    if let Some((rewritten, rewritten_args)) = rewrite_windows_script_for_protocol(&program, args) {
        if !attempts.iter().any(|(existing, _, existing_args)| {
            existing == &rewritten && existing_args == &rewritten_args
        }) {
            attempts.push((rewritten, env.to_vec(), rewritten_args));
        }
        return;
    }
    #[cfg(not(windows))]
    let _ = ();
    if !attempts
        .iter()
        .any(|(existing, _, existing_args)| existing == &program && existing_args == args)
    {
        attempts.push((program, env.to_vec(), args.to_vec()));
    }
}

/// Windows `.cmd`/`.bat` ACP shims (notably Cursor `agent.cmd`) nest `cmd` →
/// PowerShell → Node and drop redirected Protocol stdio. Prefer the underlying
/// Node entrypoint when the shim layout matches Cursor Agent installs.
#[cfg(windows)]
fn rewrite_windows_script_for_protocol(
    program: &str,
    args: &[String],
) -> Option<(String, Vec<String>)> {
    let path = Path::new(program);
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if ext != "cmd" && ext != "bat" {
        return None;
    }
    let dir = path.parent()?;
    let (node, index) = cursor_agent_node_entrypoint(dir)?;
    let mut rewritten_args = vec![index];
    rewritten_args.extend(args.iter().cloned());
    Some((node, rewritten_args))
}

#[cfg(windows)]
fn cursor_agent_node_entrypoint(dir: &Path) -> Option<(String, String)> {
    let root_node = dir.join("node.exe");
    let root_index = dir.join("index.js");
    if root_node.is_file() && root_index.is_file() {
        return Some((
            root_node.to_string_lossy().into_owned(),
            root_index.to_string_lossy().into_owned(),
        ));
    }

    let versions = dir.join("versions");
    let mut best: Option<(u32, PathBuf)> = None;
    let entries = std::fs::read_dir(&versions).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_str()?;
        let Some(sort_key) = cursor_agent_version_sort_key(name) else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|(best_key, _)| sort_key >= *best_key)
        {
            best = Some((sort_key, path));
        }
    }
    let version_dir = best?.1;
    let node = version_dir.join("node.exe");
    let index = version_dir.join("index.js");
    (node.is_file() && index.is_file()).then(|| {
        (
            node.to_string_lossy().into_owned(),
            index.to_string_lossy().into_owned(),
        )
    })
}

#[cfg(windows)]
fn cursor_agent_version_sort_key(name: &str) -> Option<u32> {
    // YYYY.M.D-commit or YYYY.M.D-HH-MM-SS-commit
    let date_part = name.split('-').next()?;
    let mut parts = date_part.split('.');
    let year: u32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(year * 10_000 + month * 100 + day)
}

fn try_owned_launch(
    process_registry: &Arc<Mutex<ProcessRegistry>>,
    workspace_id: ProcessWorkspaceId,
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    stdio: ProcessLaunchStdio,
) -> Result<OwnedBackgroundLaunch, String> {
    let mut env = env.to_vec();
    editor_tool_install::merge_effective_path(&mut env);
    let mut spec = ProcessLaunchSpec::new(program, args.to_vec())
        .with_stdio(stdio)
        .with_mode(ProcessSupervisionMode::Background)
        .with_workspace(workspace_id);
    if let Some(cwd) = cwd {
        spec = spec.with_current_dir(cwd);
    }
    spec.env = env;

    let mut registry = process_registry
        .lock()
        .map_err(|_| "process registry mutex poisoned".to_owned())?;
    let mut launched = registry.launch(spec).map_err(|error| error.to_string())?;
    let stdin = launched
        .stdin
        .take()
        .map(tokio::process::ChildStdin::from_std)
        .transpose()
        .map_err(|error| error.to_string())?;
    let stdout = launched
        .stdout
        .take()
        .map(tokio::process::ChildStdout::from_std)
        .transpose()
        .map_err(|error| error.to_string())?;
    let stderr = launched
        .stderr
        .take()
        .map(tokio::process::ChildStderr::from_std)
        .transpose()
        .map_err(|error| error.to_string())?;
    Ok(OwnedBackgroundLaunch {
        owned_process_id: launched.id,
        stdin,
        stdout,
        stderr,
    })
}

fn owned_launch_should_retry(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("not found")
        || lower.contains("cannot find")
        || lower.contains("the system cannot find")
        || lower.contains("os error 2")
        || lower.contains("os error 193")
}

pub(crate) fn apply_command_environment(command: &mut Command, env: &[(String, String)]) {
    for (key, value) in env {
        command.env(key, value);
    }
}

pub(crate) fn apply_launch_environment(
    command: &mut Command,
    env: &[(String, String)],
    launch_env: &[(String, String)],
) {
    for (key, value) in launch_env {
        command.env(key, value);
    }
    apply_command_environment(command, env);
}

pub(crate) fn background_command_candidates(
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

pub(crate) fn background_command_names(
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

pub(crate) fn environment_value(
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
    key: &str,
) -> Option<String> {
    explicit_environment_value(env, key)
        .cloned()
        .or_else(|| launch_env.and_then(|vars| explicit_environment_value(vars, key).cloned()))
        .or_else(|| std::env::var(key).ok())
}

pub(crate) fn explicit_environment_value<'a>(
    env: &'a [(String, String)],
    key: &str,
) -> Option<&'a String> {
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
pub(crate) fn windows_command_extensions(
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
pub(crate) fn path_list_separator() -> char {
    ';'
}

#[cfg(not(windows))]
pub(crate) fn path_list_separator() -> char {
    ':'
}

#[cfg(windows)]
pub(crate) fn is_launch_candidate(candidate: &Path) -> bool {
    candidate.is_file()
}

#[cfg(not(windows))]
pub(crate) fn is_launch_candidate(candidate: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;

    candidate
        .metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
pub(crate) async fn windows_node_manager_environment(
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
pub(crate) async fn windows_fnm_environment(
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
pub(crate) async fn windows_nvm_environment(
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
pub(crate) async fn windows_nvm_command_stdout(
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
pub(crate) async fn refreshed_launch_environment(
    cwd: Option<&Path>,
) -> Option<Vec<(String, String)>> {
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
pub(crate) async fn refreshed_launch_environment(
    cwd: Option<&Path>,
) -> Option<Vec<(String, String)>> {
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
pub(crate) async fn refreshed_launch_environment(
    _cwd: Option<&Path>,
) -> Option<Vec<(String, String)>> {
    None
}

#[cfg(windows)]
pub(crate) fn parse_line_environment(output: &str) -> Option<Vec<(String, String)>> {
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
pub(crate) fn parse_windows_cmd_environment(output: &str) -> Option<Vec<(String, String)>> {
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
pub(crate) struct WindowsNvmSettings {
    pub(crate) root: Option<PathBuf>,
    pub(crate) path: Option<PathBuf>,
}

#[cfg(windows)]
pub(crate) fn windows_nvm_settings(
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
pub(crate) fn windows_nvm_settings_paths(
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
pub(crate) fn parse_windows_nvm_settings(content: &str) -> WindowsNvmSettings {
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
pub(crate) fn parse_windows_nvm_current_output(output: &str) -> Option<String> {
    let current = output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    (!current.to_ascii_lowercase().starts_with("no current")).then(|| current.to_owned())
}

#[cfg(windows)]
pub(crate) fn parse_windows_nvm_root_output(output: &str) -> Option<PathBuf> {
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
pub(crate) fn windows_default_nvm_home(
    env: &[(String, String)],
    launch_env: Option<&[(String, String)]>,
) -> Option<PathBuf> {
    environment_value(env, launch_env, "APPDATA").map(|appdata| PathBuf::from(appdata).join("nvm"))
}

#[cfg(windows)]
pub(crate) fn windows_nvm_environment_from_parts(
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
pub(crate) fn windows_nvm_version_dir_candidates(root: &Path, current: &str) -> Vec<PathBuf> {
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
pub(crate) fn merge_node_manager_environment(
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
pub(crate) fn upsert_environment_value(
    env: &mut Vec<(String, String)>,
    key: String,
    value: String,
) {
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
pub(crate) fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) fn parse_nul_environment(output: &[u8]) -> Option<Vec<(String, String)>> {
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

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::{
        cursor_agent_node_entrypoint, cursor_agent_version_sort_key,
        rewrite_windows_script_for_protocol,
    };
    #[cfg(windows)]
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(windows)]
    fn unique_temp_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[cfg(windows)]
    #[test]
    fn cursor_agent_version_sort_key_parses_date_prefixes() {
        assert_eq!(
            cursor_agent_version_sort_key("2026.09.10-fd3934a"),
            Some(20_260_910)
        );
        assert_eq!(
            cursor_agent_version_sort_key("2026.9.2-c22c1a3"),
            Some(20_260_902)
        );
        assert_eq!(cursor_agent_version_sort_key("not-a-version"), None);
    }

    #[cfg(windows)]
    #[test]
    fn rewrite_windows_script_for_protocol_unwraps_cursor_agent_cmd() {
        let root = unique_temp_path("volt-acp-cursor-shim");
        let versions = root.join("versions").join("2026.09.10-aaaaaaa");
        fs::create_dir_all(&versions).expect("mkdir");
        fs::write(versions.join("node.exe"), b"fake").expect("node");
        fs::write(versions.join("index.js"), b"fake").expect("index");
        let cmd = root.join("agent.cmd");
        fs::write(&cmd, b"@echo off\r\n").expect("cmd");

        let rewritten = rewrite_windows_script_for_protocol(
            cmd.to_str().expect("utf8"),
            &["acp".to_owned(), "--yolo".to_owned()],
        )
        .expect("rewrite");
        assert_eq!(rewritten.0, versions.join("node.exe").to_string_lossy());
        assert_eq!(
            rewritten.1,
            vec![
                versions.join("index.js").to_string_lossy().into_owned(),
                "acp".to_owned(),
                "--yolo".to_owned(),
            ]
        );
        assert_eq!(
            cursor_agent_node_entrypoint(&root).map(|(node, _)| node),
            Some(versions.join("node.exe").to_string_lossy().into_owned())
        );
        let _ = fs::remove_dir_all(root);
    }
}
