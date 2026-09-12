//! Process Registry and Process Launch for Owned Process trees.
//!
//! Every Launch registers a tree root (platform kill-tree) under hybrid
//! supervision (kill-tree plus `volt --process-supervisor` when applicable).

use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::{Path, PathBuf},
    process::{ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use command_group::{CommandGroup, GroupChild};

use crate::{ProcessSupervisionMode, supervised_command_with_exe};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Primary key for an Owned Process tree root in the Process Registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnedProcessId(u64);

impl OwnedProcessId {
    /// Returns the numeric identifier.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for OwnedProcessId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Workspace tag used for ownership matching on Workspace Close.
///
/// This is the Process Registry's Workspace identity. Callers map from the
/// editor-core `WorkspaceId` via [`WorkspaceId::from_raw`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceId(u64);

impl WorkspaceId {
    /// Creates a Workspace tag from its raw numeric id.
    pub const fn from_raw(id: u64) -> Self {
        Self(id)
    }

    /// Returns the numeric identifier.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Identity for an Owned Process shared across Workspaces.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShareKey(String);

impl ShareKey {
    /// Creates a Share Key from a string identity.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// Returns the key string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ShareKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Stdio configuration for Process Launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessLaunchStdio {
    /// Discard stdin/stdout/stderr (default for fire-and-forget trees).
    #[default]
    Null,
    /// Pipe stdout and stderr for callers that need to stream or capture output.
    Piped,
    /// Pipe stdin and stdout for protocol children (LSP/DAP/ACP); discard stderr.
    Protocol,
}

/// How Process Launch should start an Owned Process tree.
#[derive(Debug, Clone)]
pub struct ProcessLaunchSpec {
    /// Program to execute (before supervisor wrapping).
    pub program: String,
    /// Arguments for `program`.
    pub args: Vec<String>,
    /// Initial Workspace ownership tags.
    pub workspace_ids: Vec<WorkspaceId>,
    /// Optional Share Key for multi-Workspace refcounted ownership.
    pub share_key: Option<ShareKey>,
    /// Supervision mode passed to the process supervisor wrapper.
    pub mode: ProcessSupervisionMode,
    /// Working directory for the launched tree root.
    pub current_dir: Option<PathBuf>,
    /// Extra environment variables for the launched tree root.
    pub env: Vec<(String, String)>,
    /// Optional process supervisor executable override (tests / embedding).
    pub process_supervisor_exe: Option<PathBuf>,
    /// Stdio mode for the launched tree root.
    pub stdio: ProcessLaunchStdio,
}

impl ProcessLaunchSpec {
    /// Builds a launch spec for `program` with `args`.
    pub fn new(
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            workspace_ids: Vec::new(),
            share_key: None,
            mode: ProcessSupervisionMode::Background,
            current_dir: None,
            env: Vec::new(),
            process_supervisor_exe: None,
            stdio: ProcessLaunchStdio::Null,
        }
    }

    /// Tags the Owned Process with a Workspace.
    pub fn with_workspace(mut self, workspace_id: WorkspaceId) -> Self {
        self.workspace_ids.push(workspace_id);
        self
    }

    /// Sets the Share Key for shared ownership.
    pub fn with_share_key(mut self, share_key: ShareKey) -> Self {
        self.share_key = Some(share_key);
        self
    }

    /// Sets the process supervision mode.
    pub fn with_mode(mut self, mode: ProcessSupervisionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the working directory.
    pub fn with_current_dir(mut self, current_dir: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(current_dir.into());
        self
    }

    /// Overrides the Volt process supervisor executable used for hybrid Launch.
    pub fn with_process_supervisor_exe(mut self, exe: impl Into<PathBuf>) -> Self {
        self.process_supervisor_exe = Some(exe.into());
        self
    }

    /// Configures stdio for the launched tree root.
    pub fn with_stdio(mut self, stdio: ProcessLaunchStdio) -> Self {
        self.stdio = stdio;
        self
    }
}

/// Result of Process Launch: registry id plus optional pipes.
#[derive(Debug)]
pub struct LaunchedProcess {
    /// Registry id for the Owned Process tree.
    pub id: OwnedProcessId,
    /// Piped stdin when launched with [`ProcessLaunchStdio::Protocol`].
    pub stdin: Option<ChildStdin>,
    /// Piped stdout when launched with [`ProcessLaunchStdio::Piped`] or
    /// [`ProcessLaunchStdio::Protocol`].
    pub stdout: Option<ChildStdout>,
    /// Piped stderr when launched with [`ProcessLaunchStdio::Piped`].
    pub stderr: Option<ChildStderr>,
}

/// Builds a Share Key for a language server serving one on-disk root.
pub fn language_server_share_key(server_id: &str, root: Option<&Path>) -> ShareKey {
    match root {
        Some(root) => ShareKey::new(format!("lsp:{server_id}:{}", root.display())),
        None => ShareKey::new(format!("lsp:{server_id}:global")),
    }
}

/// Errors from Process Registry operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessRegistryError {
    /// The Owned Process id is not registered.
    UnknownProcess(OwnedProcessId),
    /// Spawning or configuring the Owned Process tree failed.
    Launch(String),
    /// Teardown of an Owned Process tree failed.
    Teardown(String),
}

impl fmt::Display for ProcessRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProcess(id) => write!(formatter, "unknown owned process `{id}`"),
            Self::Launch(message) => write!(formatter, "process launch failed: {message}"),
            Self::Teardown(message) => write!(formatter, "process teardown failed: {message}"),
        }
    }
}

impl std::error::Error for ProcessRegistryError {}

/// App-wide index of Owned Processes.
#[derive(Debug, Default)]
pub struct ProcessRegistry {
    next_id: u64,
    entries: HashMap<OwnedProcessId, OwnedProcessEntry>,
    share_keys: HashMap<ShareKey, OwnedProcessId>,
}

struct OwnedProcessEntry {
    workspace_ids: HashSet<WorkspaceId>,
    share_key: Option<ShareKey>,
    tree: OwnedProcessTree,
}

impl fmt::Debug for OwnedProcessEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnedProcessEntry")
            .field("workspace_ids", &self.workspace_ids)
            .field("share_key", &self.share_key)
            .field("root_pid", &self.tree.root_pid())
            .finish()
    }
}

impl ProcessRegistry {
    /// Creates an empty Process Registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process Launch: starts an Owned Process tree under hybrid supervision and
    /// registers it. When a Share Key already exists, adds Workspace tags to the
    /// existing entry instead of spawning again.
    pub fn launch(
        &mut self,
        spec: ProcessLaunchSpec,
    ) -> Result<LaunchedProcess, ProcessRegistryError> {
        if let Some(share_key) = &spec.share_key
            && let Some(&existing_id) = self.share_keys.get(share_key)
        {
            let entry = self
                .entries
                .get_mut(&existing_id)
                .ok_or(ProcessRegistryError::UnknownProcess(existing_id))?;
            entry.workspace_ids.extend(spec.workspace_ids);
            return Ok(LaunchedProcess {
                id: existing_id,
                stdin: None,
                stdout: None,
                stderr: None,
            });
        }

        let (tree, stdin, stdout, stderr) = OwnedProcessTree::launch(&spec)?;
        let id = self.allocate_id();
        let share_key = spec.share_key.clone();
        if let Some(share_key) = share_key.clone() {
            self.share_keys.insert(share_key, id);
        }

        self.entries.insert(
            id,
            OwnedProcessEntry {
                workspace_ids: spec.workspace_ids.into_iter().collect(),
                share_key,
                tree,
            },
        );
        Ok(LaunchedProcess {
            id,
            stdin,
            stdout,
            stderr,
        })
    }

    /// Process Launch for an interactive shell tree the caller starts inside
    /// `spawn` (ConPTY/PTY). The OS spawn happens under Launch; the returned
    /// root PID is registered with Workspace tags. Supervisor wrapping is not
    /// applicable for interactive shells.
    pub fn launch_interactive_session<T, E>(
        &mut self,
        workspace_ids: impl IntoIterator<Item = WorkspaceId>,
        spawn: impl FnOnce() -> Result<(T, u32), E>,
    ) -> Result<(OwnedProcessId, T), ProcessRegistryError>
    where
        E: std::fmt::Display,
    {
        let (payload, root_pid) =
            spawn().map_err(|error| ProcessRegistryError::Launch(error.to_string()))?;
        let id = self.launch_interactive_root(root_pid, workspace_ids)?;
        Ok((id, payload))
    }

    /// Registers an already-started interactive root PID. Prefer
    /// [`Self::launch_interactive_session`] so the OS spawn stays inside Process Launch.
    pub fn launch_interactive_root(
        &mut self,
        root_pid: u32,
        workspace_ids: impl IntoIterator<Item = WorkspaceId>,
    ) -> Result<OwnedProcessId, ProcessRegistryError> {
        let tree = OwnedProcessTree::adopt_interactive_root(root_pid)?;
        let id = self.allocate_id();
        self.entries.insert(
            id,
            OwnedProcessEntry {
                workspace_ids: workspace_ids.into_iter().collect(),
                share_key: None,
                tree,
            },
        );
        Ok(id)
    }

    /// Returns whether the Owned Process tree root is still alive.
    pub fn is_alive(&mut self, id: OwnedProcessId) -> bool {
        let Some(entry) = self.entries.get_mut(&id) else {
            return false;
        };
        entry.tree.is_alive()
    }

    /// Returns the OS process id of the Owned Process tree root, if registered.
    pub fn root_pid(&self, id: OwnedProcessId) -> Option<u32> {
        self.entries.get(&id).map(|entry| entry.tree.root_pid())
    }

    /// Returns how many Owned Processes are currently registered and alive.
    pub fn alive_count(&mut self) -> usize {
        let ids = self.entries.keys().copied().collect::<Vec<_>>();
        ids.into_iter().filter(|id| self.is_alive(*id)).count()
    }

    /// Returns the exit code when the Owned Process has exited.
    pub fn exit_code(&mut self, id: OwnedProcessId) -> Option<i32> {
        self.entries.get_mut(&id)?.tree.exit_code()
    }

    /// Removes a naturally-exited Owned Process from the registry without force-kill.
    pub fn reclaim(&mut self, id: OwnedProcessId) -> Result<(), ProcessRegistryError> {
        let Some(mut entry) = self.entries.remove(&id) else {
            return Ok(());
        };
        if let Some(share_key) = entry.share_key.take() {
            self.share_keys.remove(&share_key);
        }
        entry.tree.reclaim();
        Ok(())
    }

    /// Workspace Close: remove the Workspace tag from matching entries; tear down
    /// any Owned Process whose last tag was removed.
    pub fn workspace_close(
        &mut self,
        workspace_id: WorkspaceId,
        grace: Duration,
    ) -> Result<(), ProcessRegistryError> {
        let mut teardown_ids = Vec::new();
        for (id, entry) in &mut self.entries {
            if entry.workspace_ids.remove(&workspace_id) && entry.workspace_ids.is_empty() {
                teardown_ids.push(*id);
            }
        }
        for id in teardown_ids {
            self.teardown(id, grace)?;
        }
        Ok(())
    }

    /// Application Quit: one global graceful-then-force over the entire registry.
    pub fn application_quit(&mut self, grace: Duration) -> Result<(), ProcessRegistryError> {
        let ids = self.entries.keys().copied().collect::<Vec<_>>();
        for id in ids {
            self.teardown(id, grace)?;
        }
        Ok(())
    }

    /// Returns whether removing `workspace_id` would leave this Owned Process with
    /// no Workspace tags (so Workspace Close would tear it down).
    pub fn will_teardown_on_workspace_close(
        &self,
        id: OwnedProcessId,
        workspace_id: WorkspaceId,
    ) -> bool {
        self.entries.get(&id).is_some_and(|entry| {
            entry.workspace_ids.contains(&workspace_id) && entry.workspace_ids.len() == 1
        })
    }

    /// Adds a Workspace tag to an existing Owned Process.
    pub fn tag_workspace(
        &mut self,
        id: OwnedProcessId,
        workspace_id: WorkspaceId,
    ) -> Result<(), ProcessRegistryError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(ProcessRegistryError::UnknownProcess(id))?;
        entry.workspace_ids.insert(workspace_id);
        Ok(())
    }

    /// Drops the platform kill-tree as if the owning UI process crashed, so
    /// kill-on-job-close / equivalent should reap the tree without an orderly quit.
    pub fn simulate_owner_crash(&mut self, id: OwnedProcessId) -> Result<(), ProcessRegistryError> {
        let Some(mut entry) = self.entries.remove(&id) else {
            return Err(ProcessRegistryError::UnknownProcess(id));
        };
        if let Some(share_key) = entry.share_key.take() {
            self.share_keys.remove(&share_key);
        }
        entry.tree.crash_close()
    }

    /// Graceful-then-force teardown for one Owned Process by id.
    pub fn teardown_process(
        &mut self,
        id: OwnedProcessId,
        grace: Duration,
    ) -> Result<(), ProcessRegistryError> {
        self.teardown(id, grace)
    }

    fn teardown(
        &mut self,
        id: OwnedProcessId,
        grace: Duration,
    ) -> Result<(), ProcessRegistryError> {
        let Some(mut entry) = self.entries.remove(&id) else {
            return Ok(());
        };
        if let Some(share_key) = entry.share_key.take() {
            self.share_keys.remove(&share_key);
        }
        entry.tree.graceful_then_force(grace)
    }

    fn allocate_id(&mut self) -> OwnedProcessId {
        self.next_id = self.next_id.saturating_add(1);
        OwnedProcessId(self.next_id)
    }
}

enum OwnedProcessTree {
    Group(GroupChild),
    External(ExternalOwnedTree),
}

struct ExternalOwnedTree {
    root_pid: u32,
}

type LaunchedTreeParts = (
    OwnedProcessTree,
    Option<ChildStdin>,
    Option<ChildStdout>,
    Option<ChildStderr>,
);

impl OwnedProcessTree {
    fn launch(spec: &ProcessLaunchSpec) -> Result<LaunchedTreeParts, ProcessRegistryError> {
        let (program, args) = supervised_command_for_launch(spec);
        let mut command = Command::new(&program);
        command.args(&args);
        let (stdin, stdout, stderr) = match spec.stdio {
            ProcessLaunchStdio::Null => {
                command
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                (false, false, false)
            }
            ProcessLaunchStdio::Piped => {
                command
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                (false, true, true)
            }
            ProcessLaunchStdio::Protocol => {
                command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null());
                (true, true, false)
            }
        };
        if let Some(current_dir) = &spec.current_dir {
            command.current_dir(current_dir);
        }
        for (key, value) in &spec.env {
            command.env(key, value);
        }

        let mut builder = command.group();
        #[cfg(windows)]
        {
            builder.kill_on_drop(true);
            if matches!(spec.mode, ProcessSupervisionMode::Background) {
                builder.creation_flags(CREATE_NO_WINDOW);
            }
        }
        #[cfg(unix)]
        {
            let _ = spec.mode;
        }

        let mut child = builder
            .spawn()
            .map_err(|error| ProcessRegistryError::Launch(format!("failed to spawn: {error}")))?;
        let stdin = if stdin {
            child.inner().stdin.take()
        } else {
            None
        };
        let stdout = if stdout {
            child.inner().stdout.take()
        } else {
            None
        };
        let stderr = if stderr {
            child.inner().stderr.take()
        } else {
            None
        };
        Ok((Self::Group(child), stdin, stdout, stderr))
    }

    fn adopt_interactive_root(root_pid: u32) -> Result<Self, ProcessRegistryError> {
        if root_pid == 0 {
            return Err(ProcessRegistryError::Launch(
                "interactive root pid must be non-zero".to_owned(),
            ));
        }
        // Interactive ConPTY roots are adopted by PID. Teardown uses platform
        // kill-tree signaling (`taskkill /T` / process-group kill). Job Object
        // adoption would need `unsafe` Windows APIs forbidden in this crate;
        // spawned Launch trees still get Job Objects via `command_group`.
        Ok(Self::External(ExternalOwnedTree { root_pid }))
    }

    fn root_pid(&self) -> u32 {
        match self {
            Self::Group(child) => child.id(),
            Self::External(tree) => tree.root_pid,
        }
    }

    fn is_alive(&mut self) -> bool {
        match self {
            Self::Group(child) => match child.try_wait() {
                Ok(None) => true,
                Ok(Some(_)) => false,
                Err(_) => owned_process_pid_alive(child.id()),
            },
            Self::External(tree) => owned_process_pid_alive(tree.root_pid),
        }
    }

    fn exit_code(&mut self) -> Option<i32> {
        match self {
            Self::Group(child) => match child.try_wait() {
                Ok(Some(status)) => status.code(),
                Ok(None) | Err(_) => None,
            },
            Self::External(_) => None,
        }
    }

    fn reclaim(&mut self) {
        match self {
            Self::Group(child) => {
                let _ = child.try_wait();
            }
            Self::External(_) => {}
        }
    }

    fn graceful_then_force(&mut self, grace: Duration) -> Result<(), ProcessRegistryError> {
        if !self.is_alive() {
            return Ok(());
        }

        let root_pid = self.root_pid();
        signal_graceful(root_pid).map_err(ProcessRegistryError::Teardown)?;

        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            if !self.is_alive() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(20));
        }

        if !self.is_alive() {
            return Ok(());
        }

        match self {
            Self::Group(child) => {
                child.kill().map_err(|error| {
                    ProcessRegistryError::Teardown(format!("force kill failed: {error}"))
                })?;
            }
            Self::External(tree) => {
                force_kill_tree(tree.root_pid).map_err(ProcessRegistryError::Teardown)?;
            }
        }

        wait_until_dead(root_pid, Duration::from_secs(2)).map_err(|_| {
            ProcessRegistryError::Teardown("owned process survived force kill".to_owned())
        })?;
        if let Self::Group(child) = self {
            let _ = child.try_wait();
        }
        Ok(())
    }

    #[cfg(windows)]
    fn crash_close(self) -> Result<(), ProcessRegistryError> {
        let pid = self.root_pid();
        match self {
            Self::Group(child) => {
                // Dropping GroupChild closes the Job Object; with kill-on-job-close the
                // OS reaps the tree — the same path as UI-process crash handle cleanup.
                drop(child);
            }
            Self::External(_) => {
                // Adopted interactive roots lack a Job Object handle in this crate
                // (no `unsafe`); simulate crash reclaim via force kill-tree.
                force_kill_tree(pid).map_err(ProcessRegistryError::Teardown)?;
            }
        }
        wait_until_dead(pid, Duration::from_secs(2)).map_err(|_| {
            ProcessRegistryError::Teardown(
                "owned process tree survived owner-crash kill-tree close".to_owned(),
            )
        })
    }

    #[cfg(unix)]
    fn crash_close(mut self) -> Result<(), ProcessRegistryError> {
        let pid = self.root_pid();
        match &mut self {
            Self::Group(child) => {
                child.kill().map_err(|error| {
                    ProcessRegistryError::Teardown(format!("crash reclaim failed: {error}"))
                })?;
            }
            Self::External(_) => {
                force_kill_tree(pid).map_err(ProcessRegistryError::Teardown)?;
            }
        }
        drop(self);
        wait_until_dead(pid, Duration::from_secs(2)).map_err(|_| {
            ProcessRegistryError::Teardown(
                "owned process tree survived owner-crash kill-tree close".to_owned(),
            )
        })
    }
}

fn supervised_command_for_launch(spec: &ProcessLaunchSpec) -> (String, Vec<String>) {
    // Interactive / ConPTY shells cannot be wrapped by the supervisor binary.
    if matches!(spec.mode, ProcessSupervisionMode::Interactive) {
        return (spec.program.clone(), spec.args.clone());
    }
    supervised_command_with_exe(
        spec.process_supervisor_exe.as_deref(),
        &spec.program,
        &spec.args,
        spec.mode,
    )
}

fn signal_graceful(root_pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        let _ = Command::new("taskkill")
            .args(["/T", "/PID", &root_pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|error| format!("taskkill graceful failed: {error}"))?;
        Ok(())
    }
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process_group};
        let Some(pid) = Pid::from_raw(root_pid as i32) else {
            return Err(format!("invalid process group pid {root_pid}"));
        };
        let _ = kill_process_group(pid, Signal::TERM);
        Ok(())
    }
}

fn force_kill_tree(root_pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &root_pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|error| format!("taskkill force failed: {error}"))?;
        Ok(())
    }
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process, kill_process_group};
        let Some(pid) = Pid::from_raw(root_pid as i32) else {
            return Err(format!("invalid process group pid {root_pid}"));
        };
        let _ = kill_process_group(pid, Signal::KILL);
        let _ = kill_process(pid, Signal::KILL);
        Ok(())
    }
}

fn wait_until_dead(pid: u32, timeout: Duration) -> Result<(), ()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !owned_process_pid_alive(pid) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    if owned_process_pid_alive(pid) {
        Err(())
    } else {
        Ok(())
    }
}

/// Returns true when `pid` is still running according to the OS.
pub fn owned_process_pid_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        let output = Command::new("tasklist")
            .args(["/NH", "/FI", &format!("PID eq {pid}")])
            .stdin(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process};
        let Some(pid) = Pid::from_raw(pid as i32) else {
            return false;
        };
        kill_process(pid, Signal::EXISTS).is_ok()
    }
}

/// Helper for tests and callers that need a long-lived synthetic workload.
#[cfg(test)]
pub(crate) fn synthetic_sleep_command(seconds: u64) -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        (
            "ping".to_owned(),
            vec![
                "-n".to_owned(),
                (seconds.saturating_add(1)).to_string(),
                "127.0.0.1".to_owned(),
            ],
        )
    }
    #[cfg(unix)]
    {
        ("sleep".to_owned(), vec![seconds.to_string()])
    }
}

/// Resolves a local Volt executable suitable for hybrid Process Launch tests.
#[cfg(test)]
pub(crate) fn discover_volt_supervisor_exe() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/volt.exe"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/volt"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/volt.exe"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/volt"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}
