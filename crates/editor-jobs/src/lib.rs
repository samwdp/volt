#![doc = r#"Asynchronous job scheduling, process supervision, and compilation task coordination."#]

use std::{
    error::Error,
    fmt,
    path::PathBuf,
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
    let mut command = Command::new(spec.program());
    configure_background_command(&mut command);
    command.args(spec.args());
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(cwd) = spec.cwd() {
        command.current_dir(cwd);
    }
    for (key, value) in spec.env() {
        command.env(key, value);
    }

    let output = command.output()?;
    Ok(JobResult {
        id,
        spec,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code(),
        duration: started.elapsed(),
    })
}

#[cfg(test)]
mod tests {
    use super::{CompilationRunner, JobKind, JobManager, JobSpec};

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
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
}
