#![doc = r#"Asynchronous job scheduling, process supervision, and compilation task coordination."#]

use std::{
    env,
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

fn configure_background_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

/// Environment variable used to advertise the Volt executable that can supervise child processes.
pub const PROCESS_SUPERVISOR_EXE_ENV: &str = "VOLT_PROCESS_SUPERVISOR_EXE";

/// Hidden flag used to run the Volt executable in child-process supervision mode.
pub const PROCESS_SUPERVISOR_FLAG: &str = "--process-supervisor";

const PROCESS_SUPERVISOR_BACKGROUND_FLAG: &str = "--background";

/// Controls how the supervised child should be launched on the current platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSupervisionMode {
    /// The child should stay hidden from the desktop shell when possible.
    Background,
    /// The child is interactive and should inherit the caller's visible terminal/PTY state.
    Interactive,
}

/// Resolves a launchable command path using the effective environment when possible.
pub fn resolve_command_path(
    program: &str,
    explicit_env: &[(String, String)],
    base_env: Option<&[(String, String)]>,
) -> Option<String> {
    if Path::new(program).components().count() != 1 {
        return Some(program.to_owned());
    }

    let path_value = environment_value(explicit_env, base_env, "PATH")?;
    let names = command_candidate_names(program, explicit_env, base_env);
    for directory in path_value
        .split(path_list_separator())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        for name in &names {
            let candidate = Path::new(directory).join(name);
            if is_launch_candidate(&candidate) {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Wraps a command with Volt's parent-death supervisor when the target program can be resolved.
pub fn supervised_command_if_resolved(
    program: &str,
    args: &[String],
    explicit_env: &[(String, String)],
    base_env: Option<&[(String, String)]>,
    mode: ProcessSupervisionMode,
) -> (String, Vec<String>) {
    let Some(program) = resolve_command_path(program, explicit_env, base_env) else {
        return (program.to_owned(), args.to_vec());
    };
    supervised_command(&program, args, mode)
}

/// Prepends Windows fnm/nvm PATH (and related vars) when available.
///
/// GUI-launched hosts often lack a live Node shim on `PATH`. Tree-sitter generate,
/// npm, and similar tools need that shim even when the parent process never loaded a
/// shell profile. Non-Windows platforms return `env` unchanged.
pub fn enrich_env_with_node_manager(
    cwd: Option<&Path>,
    env: Vec<(String, String)>,
) -> Vec<(String, String)> {
    #[cfg(windows)]
    {
        if let Some(runtime_env) =
            windows_fnm_environment(cwd, &env).or_else(|| windows_nvm_environment(cwd, &env))
        {
            return merge_windows_explicit_and_runtime_env(&env, &runtime_env);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = cwd;
    }
    env
}

/// Wraps a command with Volt's parent-death supervisor when the runtime advertises one.
pub fn supervised_command(
    program: &str,
    args: &[String],
    mode: ProcessSupervisionMode,
) -> (String, Vec<String>) {
    let supervisor_exe = env::var_os(PROCESS_SUPERVISOR_EXE_ENV)
        .map(PathBuf::from)
        .or_else(default_process_supervisor_executable);
    let Some(supervisor_exe) = supervisor_exe else {
        return (program.to_owned(), args.to_vec());
    };

    let mut supervised_args = vec![
        PROCESS_SUPERVISOR_FLAG.to_owned(),
        std::process::id().to_string(),
    ];
    if matches!(mode, ProcessSupervisionMode::Background) {
        supervised_args.push(PROCESS_SUPERVISOR_BACKGROUND_FLAG.to_owned());
    }
    supervised_args.push("--".to_owned());
    supervised_args.push(program.to_owned());
    supervised_args.extend(args.iter().cloned());

    (
        supervisor_exe.to_string_lossy().into_owned(),
        supervised_args,
    )
}

fn default_process_supervisor_executable() -> Option<PathBuf> {
    let current_exe = env::current_exe().ok()?;
    let stem = current_exe.file_stem()?.to_str()?;
    (stem == "volt").then_some(current_exe)
}

fn environment_value(
    explicit_env: &[(String, String)],
    base_env: Option<&[(String, String)]>,
    key: &str,
) -> Option<String> {
    lookup_env_value(explicit_env, key)
        .map(str::to_owned)
        .or_else(|| base_env.and_then(|env| lookup_env_value(env, key).map(str::to_owned)))
        .or_else(|| env::var(key).ok())
}

fn lookup_env_value<'a>(env_pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    #[cfg(windows)]
    {
        env_pairs
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
    }
    #[cfg(not(windows))]
    {
        env_pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }
}

fn command_candidate_names(
    program: &str,
    explicit_env: &[(String, String)],
    base_env: Option<&[(String, String)]>,
) -> Vec<String> {
    #[cfg(windows)]
    {
        if Path::new(program).extension().is_some() {
            return vec![program.to_owned()];
        }

        let mut names = windows_command_extensions(explicit_env, base_env)
            .into_iter()
            .map(|extension| format!("{program}{extension}"))
            .collect::<Vec<_>>();
        names.push(program.to_owned());
        names.dedup();
        names
    }
    #[cfg(not(windows))]
    {
        let _ = (explicit_env, base_env);
        vec![program.to_owned()]
    }
}

#[cfg(windows)]
fn windows_command_extensions(
    explicit_env: &[(String, String)],
    base_env: Option<&[(String, String)]>,
) -> Vec<String> {
    environment_value(explicit_env, base_env, "PATHEXT")
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

fn path_list_separator() -> char {
    #[cfg(windows)]
    {
        ';'
    }
    #[cfg(not(windows))]
    {
        ':'
    }
}

fn is_launch_candidate(path: &Path) -> bool {
    path.is_file()
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
    #[cfg_attr(not(windows), allow(unused_mut))]
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
        let should_retry_with_nvm = matches!(
            &output_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry_with_nvm
            && let Some(nvm_env) =
                windows_nvm_environment(spec.cwd().map(PathBuf::as_path), spec.env())
        {
            for candidate in windows_nvm_launch_program_candidates(spec.program(), &nvm_env) {
                output_result = build_job_command(&spec, &candidate, Some(&nvm_env)).output();
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
    #[cfg(windows)] runtime_env: Option<&[(String, String)]>,
    #[cfg(not(windows))] _runtime_env: Option<&[(String, String)]>,
) -> Command {
    let (program, args) = supervised_command_if_resolved(
        program,
        spec.args(),
        spec.env(),
        #[cfg(windows)]
        runtime_env,
        #[cfg(not(windows))]
        None,
        ProcessSupervisionMode::Background,
    );
    let mut command = Command::new(&program);
    configure_background_command(&mut command);
    command.args(&args);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(cwd) = spec.cwd() {
        command.current_dir(cwd);
    }
    #[cfg(windows)]
    if let Some(runtime_env) = runtime_env {
        apply_windows_runtime_environment(&mut command, spec.env(), runtime_env);
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
    if std::path::Path::new(program).extension().is_some() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for extension in windows_command_extensions(&[], None) {
        let candidate = format!("{program}{extension}");
        if candidate != program && !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

#[cfg(windows)]
fn windows_should_retry_spawn_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(193)
}

#[cfg(windows)]
fn windows_fnm_environment(
    cwd: Option<&std::path::Path>,
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
    windows_runtime_launch_program_candidates(program, fnm_env)
}

#[cfg(windows)]
fn windows_nvm_environment(
    cwd: Option<&std::path::Path>,
    env: &[(String, String)],
) -> Option<Vec<(String, String)>> {
    let nvm_home = windows_nvm_home(env)?;
    let nvm_exe = nvm_home.join("nvm.exe");
    nvm_exe.is_file().then_some(())?;

    let mut command = Command::new(&nvm_exe);
    configure_background_command(&mut command);
    command
        .arg("current")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    apply_command_environment(&mut command, env);
    let output = command.output().ok()?;
    output.status.success().then_some(())?;
    let version = parse_windows_nvm_current_version(&String::from_utf8_lossy(&output.stdout))?;
    let node_dir = windows_nvm_node_dir(&nvm_home, &version)?;

    let mut runtime_env = vec![("PATH".to_owned(), node_dir.to_string_lossy().into_owned())];
    runtime_env.push((
        "NVM_HOME".to_owned(),
        nvm_home.to_string_lossy().into_owned(),
    ));
    if let Some(nvm_symlink) = environment_value(env, None, "NVM_SYMLINK") {
        runtime_env.push(("NVM_SYMLINK".to_owned(), nvm_symlink));
    }
    Some(runtime_env)
}

#[cfg(windows)]
fn windows_nvm_launch_program_candidates(
    program: &str,
    nvm_env: &[(String, String)],
) -> Vec<String> {
    windows_runtime_launch_program_candidates(program, nvm_env)
}

#[cfg(windows)]
fn windows_runtime_launch_program_candidates(
    program: &str,
    runtime_env: &[(String, String)],
) -> Vec<String> {
    if std::path::Path::new(program).components().count() != 1 {
        return Vec::new();
    }

    let names = windows_launch_program_candidates(program)
        .into_iter()
        .chain(std::iter::once(program.to_owned()))
        .collect::<Vec<_>>();
    let Some(path_value) = explicit_windows_env_value(runtime_env, "PATH") else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for directory in path_value
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        for name in &names {
            let candidate = std::path::Path::new(directory).join(name);
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
fn windows_nvm_home(env: &[(String, String)]) -> Option<PathBuf> {
    environment_value(env, None, "NVM_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            environment_value(env, None, "APPDATA").map(|appdata| Path::new(&appdata).join("nvm"))
        })
}

#[cfg(windows)]
fn parse_windows_nvm_current_version(output: &str) -> Option<String> {
    let version = output
        .split_whitespace()
        .find(|token| {
            !token.is_empty()
                && !token.eq_ignore_ascii_case("none")
                && !token.eq_ignore_ascii_case("n/a")
                && token
                    .chars()
                    .next()
                    .is_some_and(|ch| ch == 'v' || ch.is_ascii_digit())
        })?
        .trim();
    Some(version.to_owned())
}

#[cfg(windows)]
fn windows_nvm_node_dir(nvm_home: &Path, version: &str) -> Option<PathBuf> {
    let mut candidates = vec![version.to_owned()];
    if let Some(stripped) = version.strip_prefix('v') {
        candidates.push(stripped.to_owned());
    } else {
        candidates.push(format!("v{version}"));
    }

    for candidate in candidates {
        let node_dir = nvm_home.join(candidate);
        if node_dir.join("node.exe").is_file() {
            return Some(node_dir);
        }
    }
    None
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
fn apply_windows_runtime_environment(
    command: &mut Command,
    env: &[(String, String)],
    runtime_env: &[(String, String)],
) {
    for (key, value) in merge_windows_explicit_and_runtime_env(env, runtime_env) {
        command.env(key, value);
    }
}

#[cfg(windows)]
fn merge_windows_explicit_and_runtime_env(
    env: &[(String, String)],
    runtime_env: &[(String, String)],
) -> Vec<(String, String)> {
    let explicit_path = explicit_windows_env_value(env, "PATH");
    let mut merged = Vec::new();
    let mut applied_path = false;
    for (key, value) in runtime_env {
        if key.eq_ignore_ascii_case("PATH") {
            let merged_path = explicit_path
                .map(|path| format!("{value};{path}"))
                .unwrap_or_else(|| value.clone());
            merged.push((key.clone(), merged_path));
            applied_path = true;
            continue;
        }
        merged.push((key.clone(), value.clone()));
    }
    for (key, value) in env {
        if !key.eq_ignore_ascii_case("PATH") {
            merged.push((key.clone(), value.clone()));
        }
    }
    if !applied_path && let Some(path) = explicit_path {
        merged.push(("PATH".to_owned(), path.clone()));
    }
    merged
}

#[cfg(windows)]
fn explicit_windows_env_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a String> {
    env.iter()
        .find_map(|(entry_key, value)| entry_key.eq_ignore_ascii_case(key).then_some(value))
}

#[cfg(test)]
mod tests;
