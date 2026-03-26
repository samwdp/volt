#![doc = r#"Asynchronous job scheduling, process supervision, and compilation task coordination."#]

use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Asynchronous job scheduling, process supervision, and compilation task coordination.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn configure_background_command(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        command.creation_flags(CREATE_NO_WINDOW);
    }
}

/// Classifies the type of work a process represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKind {
    /// Generic external command.
    Command,
    /// Compilation or build-oriented command.
    Compilation,
    /// Terminal-backed command execution.
    Terminal,
}

/// Declarative command specification for a spawned job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSpec {
    label: String,
    kind: JobKind,
    program: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    env: Vec<(String, String)>,
}

impl JobSpec {
    /// Creates a new generic command specification.
    pub fn command(
        label: impl Into<String>,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            label: label.into(),
            kind: JobKind::Command,
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            cwd: None,
            env: Vec::new(),
        }
    }

    /// Creates a new compilation command specification.
    pub fn compilation(
        label: impl Into<String>,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::command(label, program, args).with_kind(JobKind::Compilation)
    }

    /// Creates a new terminal command specification.
    pub fn terminal(
        label: impl Into<String>,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::command(label, program, args).with_kind(JobKind::Terminal)
    }

    /// Overrides the job kind.
    pub fn with_kind(mut self, kind: JobKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the current working directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Adds an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Returns the human-readable label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the job kind.
    pub const fn kind(&self) -> JobKind {
        self.kind
    }

    /// Returns the executable path.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Returns the argument list.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns the working directory, if present.
    pub fn cwd(&self) -> Option<&PathBuf> {
        self.cwd.as_ref()
    }

    /// Returns the explicit environment overrides.
    pub fn env(&self) -> &[(String, String)] {
        &self.env
    }
}

/// Final output collected from a spawned process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobResult {
    id: u64,
    spec: JobSpec,
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
    duration: Duration,
}

impl JobResult {
    /// Returns the job identifier.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the original job spec.
    pub fn spec(&self) -> &JobSpec {
        &self.spec
    }

    /// Returns collected stdout.
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Returns collected stderr.
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Returns the exit code, if one was produced.
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Returns the process duration.
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// Reports whether the process exited successfully.
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Returns a single combined transcript string.
    pub fn transcript(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}{}", self.stdout, self.stderr)
        }
    }
}

/// Errors raised while spawning or collecting jobs.
#[derive(Debug)]
pub enum JobError {
    /// Process creation or output capture failed.
    Io(std::io::Error),
    /// The background worker did not return a result.
    Disconnected,
    /// The background worker panicked.
    WorkerPanicked,
}

impl fmt::Display for JobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Disconnected => write!(formatter, "job worker disconnected before returning"),
            Self::WorkerPanicked => write!(formatter, "job worker panicked before returning"),
        }
    }
}

impl Error for JobError {}

impl From<std::io::Error> for JobError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Handle for an asynchronously running job.
#[derive(Debug)]
pub struct JobHandle {
    id: u64,
    receiver: Receiver<Result<JobResult, JobError>>,
    join_handle: thread::JoinHandle<()>,
}

impl JobHandle {
    /// Returns the job identifier.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Waits for the job to finish and returns its collected result.
    pub fn wait(self) -> Result<JobResult, JobError> {
        let join_result = self.join_handle.join();
        if join_result.is_err() {
            return Err(JobError::WorkerPanicked);
        }

        self.receiver.recv().map_err(|_| JobError::Disconnected)?
    }
}

/// Mutable process supervisor that assigns job identifiers and spawns workers.
#[derive(Debug, Default)]
pub struct JobManager {
    next_job_id: u64,
}

impl JobManager {
    /// Creates a new job manager.
    pub fn new() -> Self {
        Self { next_job_id: 1 }
    }

    /// Spawns an asynchronous job and returns a handle for later collection.
    pub fn spawn(&mut self, spec: JobSpec) -> Result<JobHandle, JobError> {
        let job_id = self.next_job_id;
        self.next_job_id += 1;

        let (sender, receiver) = mpsc::channel();
        let join_handle = thread::spawn(move || {
            let result = run_job(job_id, spec);
            let _ = sender.send(result);
        });

        Ok(JobHandle {
            id: job_id,
            receiver,
            join_handle,
        })
    }
}

/// Result wrapper for build or compile-oriented jobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationResult {
    job: JobResult,
}

impl CompilationResult {
    /// Returns the underlying job output.
    pub fn job(&self) -> &JobResult {
        &self.job
    }

    /// Reports whether the compilation succeeded.
    pub fn succeeded(&self) -> bool {
        self.job.succeeded()
    }

    /// Returns the combined build transcript.
    pub fn transcript(&self) -> String {
        self.job.transcript()
    }
}

/// Convenience wrapper for spawning compilation-oriented jobs.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CompilationRunner;

impl CompilationRunner {
    /// Creates a new compilation runner.
    pub const fn new() -> Self {
        Self
    }

    /// Spawns a compilation job.
    pub fn spawn(&self, jobs: &mut JobManager, spec: JobSpec) -> Result<JobHandle, JobError> {
        jobs.spawn(spec.with_kind(JobKind::Compilation))
    }

    /// Runs a compilation job to completion.
    pub fn run(&self, jobs: &mut JobManager, spec: JobSpec) -> Result<CompilationResult, JobError> {
        let handle = self.spawn(jobs, spec)?;
        Ok(CompilationResult {
            job: handle.wait()?,
        })
    }
}

fn run_job(id: u64, spec: JobSpec) -> Result<JobResult, JobError> {
    let started = Instant::now();
    let mut output_result = build_job_command(&spec, spec.program(), None).output();
    #[cfg(windows)]
    {
        let should_retry = matches!(
            &output_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry {
            for candidate in windows_launch_program_candidates(spec.program()) {
                output_result = build_job_command(&spec, &candidate, None).output();
                match &output_result {
                    Ok(_) => break,
                    Err(error) if windows_should_retry_spawn_error(error) => {}
                    Err(_) => break,
                }
            }
        }
        let should_retry_with_fnm = matches!(
            &output_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry_with_fnm
            && let Some(fnm_env) =
                windows_fnm_environment(spec.cwd().map(PathBuf::as_path), spec.env())
        {
            for candidate in windows_fnm_launch_program_candidates(spec.program(), &fnm_env) {
                output_result = build_job_command(&spec, &candidate, Some(&fnm_env)).output();
                match &output_result {
                    Ok(_) => break,
                    Err(error) if windows_should_retry_spawn_error(error) => {}
                    Err(_) => break,
                }
            }
        }
    }

    let output = output_result?;
    Ok(JobResult {
        id,
        spec,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code(),
        duration: started.elapsed(),
    })
}

fn build_job_command(
    spec: &JobSpec,
    program: &str,
    #[cfg(windows)] fnm_env: Option<&[(String, String)]>,
    #[cfg(not(windows))] _fnm_env: Option<&[(String, String)]>,
) -> Command {
    let mut command = Command::new(program);
    configure_background_command(&mut command);
    command.args(spec.args());
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(cwd) = spec.cwd() {
        command.current_dir(cwd);
    }
    #[cfg(windows)]
    if let Some(fnm_env) = fnm_env {
        apply_windows_fnm_environment(&mut command, spec.env(), fnm_env);
    } else {
        apply_command_environment(&mut command, spec.env());
    }
    #[cfg(not(windows))]
    apply_command_environment(&mut command, spec.env());
    command
}

fn apply_command_environment(command: &mut Command, env: &[(String, String)]) {
    for (key, value) in env {
        command.env(key, value);
    }
}

#[cfg(windows)]
fn windows_launch_program_candidates(program: &str) -> Vec<String> {
    if Path::new(program).extension().is_some() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for extension in windows_command_extensions() {
        let candidate = format!("{program}{extension}");
        if candidate != program && !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

#[cfg(windows)]
fn windows_command_extensions() -> Vec<String> {
    std::env::var("PATHEXT")
        .ok()
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
fn windows_should_retry_spawn_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(193)
}

#[cfg(windows)]
fn windows_fnm_environment(
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> Option<Vec<(String, String)>> {
    let mut command = Command::new("fnm");
    configure_background_command(&mut command);
    command
        .args(["env", "--shell", "cmd"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    apply_command_environment(&mut command, env);
    let output = command.output().ok()?;
    output.status.success().then_some(())?;
    parse_windows_cmd_environment(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(windows)]
fn windows_fnm_launch_program_candidates(
    program: &str,
    fnm_env: &[(String, String)],
) -> Vec<String> {
    if Path::new(program).components().count() != 1 {
        return Vec::new();
    }

    let names = windows_launch_program_candidates(program)
        .into_iter()
        .chain(std::iter::once(program.to_owned()))
        .collect::<Vec<_>>();
    let Some(path_value) = explicit_windows_env_value(fnm_env, "PATH") else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for directory in path_value
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        for name in &names {
            let candidate = Path::new(directory).join(name);
            if candidate.is_file() {
                let candidate = candidate.to_string_lossy().into_owned();
                if !candidates.iter().any(|existing| existing == &candidate) {
                    candidates.push(candidate);
                }
            }
        }
    }
    candidates
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
fn apply_windows_fnm_environment(
    command: &mut Command,
    env: &[(String, String)],
    fnm_env: &[(String, String)],
) {
    let explicit_path = explicit_windows_env_value(env, "PATH");
    let mut applied_path = false;
    for (key, value) in fnm_env {
        if key.eq_ignore_ascii_case("PATH") {
            let merged_path = explicit_path
                .map(|path| format!("{value};{path}"))
                .unwrap_or_else(|| value.clone());
            command.env(key, merged_path);
            applied_path = true;
            continue;
        }
        command.env(key, value);
    }
    for (key, value) in env {
        if !key.eq_ignore_ascii_case("PATH") {
            command.env(key, value);
        }
    }
    if !applied_path && let Some(path) = explicit_path {
        command.env("PATH", path);
    }
}

#[cfg(windows)]
fn explicit_windows_env_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a String> {
    env.iter()
        .find_map(|(entry_key, value)| entry_key.eq_ignore_ascii_case(key).then_some(value))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{CompilationRunner, JobKind, JobManager, JobSpec};

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    #[cfg(windows)]
    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{unique}"))
    }

    #[test]
    fn job_manager_runs_commands_and_collects_output() {
        let mut jobs = JobManager::new();
        let handle = must(jobs.spawn(JobSpec::command("rustc-version", "rustc", ["--version"])));
        let result = must(handle.wait());

        assert_eq!(result.spec().kind(), JobKind::Command);
        assert!(result.succeeded());
        assert!(result.stdout().contains("rustc"));
        assert!(result.duration().as_nanos() > 0);
    }

    #[test]
    fn compilation_runner_marks_jobs_as_compilation() {
        let mut jobs = JobManager::new();
        let compilation = must(CompilationRunner::new().run(
            &mut jobs,
            JobSpec::compilation("rustc-version", "rustc", ["--version"]),
        ));

        assert_eq!(compilation.job().spec().kind(), JobKind::Compilation);
        assert!(compilation.succeeded());
        assert!(compilation.transcript().contains("rustc"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_parse_cmd_environment_extracts_variables() {
        let env = super::parse_windows_cmd_environment(
            "SET PATH=C:\\fnm;C:\\tools\r\nSET FNM_DIR=C:\\Users\\sam\\AppData\\Roaming\\fnm\r\n",
        )
        .expect("fnm env should parse");
        assert_eq!(
            env,
            vec![
                ("PATH".to_owned(), "C:\\fnm;C:\\tools".to_owned()),
                (
                    "FNM_DIR".to_owned(),
                    "C:\\Users\\sam\\AppData\\Roaming\\fnm".to_owned()
                ),
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn build_job_command_keeps_fnm_path_ahead_of_explicit_path() {
        let spec = JobSpec::command("node-version", "node", ["--version"])
            .with_env("PATH", "C:\\custom")
            .with_env("NODE_OPTIONS", "--trace-warnings");
        let command = super::build_job_command(
            &spec,
            "node",
            Some(&[
                ("PATH".to_owned(), "C:\\fnm".to_owned()),
                (
                    "FNM_DIR".to_owned(),
                    "C:\\Users\\sam\\AppData\\Roaming\\fnm".to_owned(),
                ),
            ]),
        );
        let vars = command
            .get_envs()
            .filter_map(|(key, value)| {
                Some((
                    key.to_string_lossy().into_owned(),
                    value?.to_string_lossy().into_owned(),
                ))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            vars.get("PATH").map(String::as_str),
            Some("C:\\fnm;C:\\custom")
        );
        assert_eq!(
            vars.get("FNM_DIR").map(String::as_str),
            Some("C:\\Users\\sam\\AppData\\Roaming\\fnm")
        );
        assert_eq!(
            vars.get("NODE_OPTIONS").map(String::as_str),
            Some("--trace-warnings")
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_should_retry_invalid_exe_format() {
        let error = std::io::Error::from_raw_os_error(193);
        assert!(super::windows_should_retry_spawn_error(&error));
    }

    #[cfg(windows)]
    #[test]
    fn windows_fnm_launch_program_candidates_resolve_absolute_command_shims() {
        let temp_dir = temp_dir("volt-fnm-jobs");
        std::fs::create_dir_all(&temp_dir).expect("temp dir");
        let candidate_path = temp_dir.join("prettier.cmd");
        std::fs::write(&candidate_path, "@echo off\r\n").expect("candidate");

        let candidates = super::windows_fnm_launch_program_candidates(
            "prettier",
            &[("PATH".to_owned(), temp_dir.to_string_lossy().into_owned())],
        );
        assert!(candidates.contains(&candidate_path.to_string_lossy().into_owned()));

        let _ = std::fs::remove_file(candidate_path);
        let _ = std::fs::remove_dir(temp_dir);
    }

    #[cfg(windows)]
    #[test]
    fn windows_fnm_launch_program_candidates_prefer_windows_shims_over_extensionless_scripts() {
        let temp_dir = temp_dir("volt-fnm-jobs");
        std::fs::create_dir_all(&temp_dir).expect("temp dir");
        let script_path = temp_dir.join("prettier");
        let shim_path = temp_dir.join("prettier.cmd");
        std::fs::write(&script_path, "#!/bin/sh\n").expect("script");
        std::fs::write(&shim_path, "@echo off\r\n").expect("shim");

        let candidates = super::windows_fnm_launch_program_candidates(
            "prettier",
            &[("PATH".to_owned(), temp_dir.to_string_lossy().into_owned())],
        );
        assert_eq!(
            candidates.first().map(String::as_str),
            Some(shim_path.to_string_lossy().as_ref())
        );
        assert!(candidates.contains(&script_path.to_string_lossy().into_owned()));

        let _ = std::fs::remove_file(script_path);
        let _ = std::fs::remove_file(shim_path);
        let _ = std::fs::remove_dir(temp_dir);
    }
}
