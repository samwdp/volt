//! Live DAP client: transport, handshake, and one Debug Session per Workspace.

use std::{
    collections::{BTreeMap, VecDeque},
    error::Error,
    fmt,
    io::{self, BufRead, BufReader, Read, Write},
    mem,
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use dap_types::{
    AttachRequestArguments, ContinueArguments, DisconnectArguments, InitializeRequestArguments,
    LaunchRequestArguments, NextArguments, PauseArguments, RestartArguments, ScopesArguments,
    SetBreakpointsArguments, Source, SourceBreakpoint, StackTraceArguments, StepInArguments,
    StepOutArguments, VariablesArguments,
    requests::{
        Attach, Continue, Disconnect, Initialize, Launch, Next, Pause, Request as DapRequest,
        Restart, Scopes, SetBreakpoints, StackTrace, StepIn, StepOut, Variables,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    BreakpointStore, BreakpointToggle, DebugAdapterRegistry, DebugAdapterSpec,
    DebugAdapterTransport, DebugConfiguration, DebugRequestKind, DebugSessionPlan,
    StoredBreakpoint,
};

const TRANSPORT_LOG_MAX_ENTRIES: usize = 256;
const READ_TIMEOUT: Duration = Duration::from_secs(10);
const TCP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TCP_CONNECT_RETRY: Duration = Duration::from_millis(50);

/// Errors produced by the DAP client host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DapClientError {
    /// Registry lookup or plan preparation failed.
    Registry(String),
    /// Adapter executable could not be started.
    AdapterMissing {
        adapter_id: String,
        program: String,
        detail: String,
    },
    /// Transport or protocol failure.
    Protocol(String),
    /// Workspace already has a live Debug Session.
    SessionExists(u64),
    /// Workspace has no live Debug Session.
    SessionMissing(u64),
    /// Internal lock failure.
    LockPoisoned,
    /// I/O failure.
    Io(String),
}

impl fmt::Display for DapClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(message) => write!(formatter, "{message}"),
            Self::AdapterMissing {
                adapter_id,
                program,
                detail,
            } => write!(
                formatter,
                "debug adapter `{adapter_id}` program `{program}` was not found or could not be started ({detail}); install the adapter and ensure it is on PATH"
            ),
            Self::Protocol(message) => write!(formatter, "{message}"),
            Self::SessionExists(workspace_id) => write!(
                formatter,
                "workspace {workspace_id} already has a live Debug Session"
            ),
            Self::SessionMissing(workspace_id) => {
                write!(
                    formatter,
                    "workspace {workspace_id} has no live Debug Session"
                )
            }
            Self::LockPoisoned => write!(formatter, "DAP client lock poisoned"),
            Self::Io(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for DapClientError {}

impl From<io::Error> for DapClientError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// Direction of a logged DAP transport message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DapLogDirection {
    /// Client → adapter.
    Send,
    /// Adapter → client.
    Receive,
    /// Local note.
    Event,
}

/// One transport log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapLogEntry {
    adapter_id: String,
    direction: DapLogDirection,
    message: String,
}

impl DapLogEntry {
    /// Returns the adapter id for this entry.
    pub fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    /// Returns the log direction.
    pub const fn direction(&self) -> DapLogDirection {
        self.direction
    }

    /// Returns the message body.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Snapshot of recent DAP transport traffic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DapLogSnapshot {
    entries: Vec<DapLogEntry>,
}

impl DapLogSnapshot {
    /// Returns logged entries oldest-first.
    pub fn entries(&self) -> &[DapLogEntry] {
        &self.entries
    }
}

/// Ring buffer of DAP transport log entries.
#[derive(Debug)]
pub struct DapTransportLog {
    max_entries: usize,
    entries: Vec<DapLogEntry>,
}

impl DapTransportLog {
    fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: Vec::new(),
        }
    }

    fn push(&mut self, entry: DapLogEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    fn snapshot(&self) -> DapLogSnapshot {
        DapLogSnapshot {
            entries: self.entries.clone(),
        }
    }
}

type TransportLog = Arc<Mutex<DapTransportLog>>;

/// Summary of a live Debug Session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapSessionInfo {
    workspace_id: u64,
    adapter_id: String,
    configuration_name: String,
    request: DebugRequestKind,
    support_terminate_debuggee: bool,
}

impl DapSessionInfo {
    /// Returns the Workspace id this Session belongs to.
    pub const fn workspace_id(&self) -> u64 {
        self.workspace_id
    }

    /// Returns the adapter id.
    pub fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    /// Returns the configuration name.
    pub fn configuration_name(&self) -> &str {
        &self.configuration_name
    }

    /// Returns launch vs attach.
    pub const fn request(&self) -> DebugRequestKind {
        self.request
    }

    /// Returns whether the adapter reported `supportTerminateDebuggee`.
    pub const fn support_terminate_debuggee(&self) -> bool {
        self.support_terminate_debuggee
    }
}

/// Source location of the current Debug execution position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapExecutionPosition {
    path: PathBuf,
    /// 1-based DAP line.
    line: u32,
    /// 1-based DAP column.
    column: u32,
}

impl DapExecutionPosition {
    /// Returns the source path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the 1-based line.
    pub const fn line(&self) -> u32 {
        self.line
    }

    /// Returns the 1-based column.
    pub const fn column(&self) -> u32 {
        self.column
    }
}

/// One Locals variable row for the Debug Layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapLocalVariable {
    name: String,
    value: String,
    type_name: Option<String>,
}

impl DapLocalVariable {
    /// Returns the variable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the display value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the optional type name.
    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }
}

/// Snapshot captured when a Session stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapStoppedSnapshot {
    thread_id: u64,
    reason: String,
    position: Option<DapExecutionPosition>,
    locals: Vec<DapLocalVariable>,
}

impl DapStoppedSnapshot {
    /// Returns the stopped thread id.
    pub const fn thread_id(&self) -> u64 {
        self.thread_id
    }

    /// Returns the stop reason string.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns the top-frame execution position, when known.
    pub fn position(&self) -> Option<&DapExecutionPosition> {
        self.position.as_ref()
    }

    /// Returns Locals rows for the top frame.
    pub fn locals(&self) -> &[DapLocalVariable] {
        &self.locals
    }
}

/// UI-facing Session events drained by the shell each frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DapSessionEvent {
    /// Adapter reported `stopped`; snapshot should be refreshed.
    Stopped {
        /// Workspace owning the Session.
        workspace_id: u64,
    },
    /// Adapter reported `continued`; clear execution UI.
    Continued {
        /// Workspace owning the Session.
        workspace_id: u64,
    },
}

struct PendingResponse {
    tx: std::sync::mpsc::Sender<Result<Value, DapClientError>>,
}

#[derive(Debug, Default)]
struct SessionStopState {
    last_thread_id: Option<u64>,
    last_reason: Option<String>,
    snapshot: Option<DapStoppedSnapshot>,
}

struct DapSessionHandle {
    info: DapSessionInfo,
    plan: DebugSessionPlan,
    supports_restart: bool,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pending: Arc<Mutex<BTreeMap<u64, PendingResponse>>>,
    next_request_id: AtomicU64,
    disconnected: Arc<AtomicBool>,
    last_disconnect: Arc<Mutex<Option<DisconnectArguments>>>,
    stop_state: Arc<Mutex<SessionStopState>>,
    _reader: JoinHandle<()>,
    child: Option<Mutex<Child>>,
    transport_log: TransportLog,
}

impl fmt::Debug for DapSessionHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DapSessionHandle")
            .field("workspace_id", &self.info.workspace_id)
            .field("adapter_id", &self.info.adapter_id)
            .finish_non_exhaustive()
    }
}

impl Drop for DapSessionHandle {
    fn drop(&mut self) {
        self.disconnected.store(true, Ordering::Release);
        if let Some(child) = self.child.as_ref()
            && let Ok(mut child) = child.lock()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Owns live Debug Sessions keyed by Workspace id.
#[derive(Debug)]
pub struct DapClientManager {
    registry: DebugAdapterRegistry,
    sessions: Mutex<BTreeMap<u64, Arc<DapSessionHandle>>>,
    breakpoints: Mutex<BreakpointStore>,
    transport_log: TransportLog,
    events: Arc<Mutex<VecDeque<DapSessionEvent>>>,
    history: Mutex<crate::DebugStartHistory>,
}

impl DapClientManager {
    /// Creates a manager around a populated adapter registry.
    pub fn new(registry: DebugAdapterRegistry) -> Self {
        Self {
            registry,
            sessions: Mutex::new(BTreeMap::new()),
            breakpoints: Mutex::new(BreakpointStore::new()),
            transport_log: Arc::new(Mutex::new(DapTransportLog::new(TRANSPORT_LOG_MAX_ENTRIES))),
            events: Arc::new(Mutex::new(VecDeque::new())),
            history: Mutex::new(crate::DebugStartHistory::new()),
        }
    }

    /// Returns the adapter registry.
    pub fn registry(&self) -> &DebugAdapterRegistry {
        &self.registry
    }

    /// Records a successful Debug Session start for last/recent replay.
    pub fn record_start(
        &self,
        adapter_id: impl Into<String>,
        configuration: DebugConfiguration,
    ) -> Result<(), DapClientError> {
        let mut history = self
            .history
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        history.record(adapter_id, configuration);
        Ok(())
    }

    /// Returns the last successful start, if any.
    pub fn last_start(&self) -> Result<Option<crate::DebugStartRecord>, DapClientError> {
        let history = self
            .history
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(history.last().cloned())
    }

    /// Returns recent successful starts, newest first.
    pub fn recent_starts(&self) -> Result<Vec<crate::DebugStartRecord>, DapClientError> {
        let history = self
            .history
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(history.recent().cloned().collect())
    }

    /// Returns a snapshot of Breakpoints for a Workspace.
    pub fn list_breakpoints(
        &self,
        workspace_id: u64,
    ) -> Result<Vec<StoredBreakpoint>, DapClientError> {
        let store = self
            .breakpoints
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(store.list(workspace_id))
    }

    /// Returns Breakpoints for one source path.
    pub fn breakpoints_for_path(
        &self,
        workspace_id: u64,
        path: &std::path::Path,
    ) -> Result<Vec<StoredBreakpoint>, DapClientError> {
        let store = self
            .breakpoints
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(store.for_path(workspace_id, path))
    }

    /// Toggles a Breakpoint at `path`:`line` (1-based). Syncs while a Session is live.
    pub fn toggle_breakpoint(
        &self,
        workspace_id: u64,
        path: impl Into<std::path::PathBuf>,
        line: u32,
    ) -> Result<BreakpointToggle, DapClientError> {
        let path = path.into();
        let toggle = {
            let mut store = self
                .breakpoints
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            store.toggle(workspace_id, path.clone(), line)
        };
        if self.session_info(workspace_id)?.is_some() {
            self.sync_breakpoints_for_source(workspace_id, &path)?;
        }
        Ok(toggle)
    }

    /// Deletes a Breakpoint at `path`:`line` (1-based). Syncs while a Session is live.
    pub fn delete_breakpoint(
        &self,
        workspace_id: u64,
        path: &std::path::Path,
        line: u32,
    ) -> Result<bool, DapClientError> {
        let removed = {
            let mut store = self
                .breakpoints
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            store.delete(workspace_id, path, line)
        };
        if removed && self.session_info(workspace_id)?.is_some() {
            self.sync_breakpoints_for_source(workspace_id, path)?;
        }
        Ok(removed)
    }

    /// Returns a snapshot of recent DAP transport traffic.
    pub fn log_snapshot(&self) -> DapLogSnapshot {
        self.transport_log
            .lock()
            .map(|log| log.snapshot())
            .unwrap_or_default()
    }

    /// Returns info for the live Session in a Workspace, if any.
    pub fn session_info(
        &self,
        workspace_id: u64,
    ) -> Result<Option<DapSessionInfo>, DapClientError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(sessions
            .get(&workspace_id)
            .map(|session| session.info.clone()))
    }

    /// Returns every live Session summary.
    pub fn sessions(&self) -> Result<Vec<DapSessionInfo>, DapClientError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(sessions
            .values()
            .map(|session| session.info.clone())
            .collect())
    }

    /// Drains pending Session UI events (oldest-first).
    pub fn drain_events(&self) -> Result<Vec<DapSessionEvent>, DapClientError> {
        let mut events = self
            .events
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(events.drain(..).collect())
    }

    /// Returns the last captured stopped snapshot for a Workspace, if any.
    pub fn stopped_snapshot(
        &self,
        workspace_id: u64,
    ) -> Result<Option<DapStoppedSnapshot>, DapClientError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        let Some(handle) = sessions.get(&workspace_id) else {
            return Ok(None);
        };
        let state = handle
            .stop_state
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(state.snapshot.clone())
    }

    /// Fetches stack/locals for the last stopped thread and stores the snapshot.
    pub fn refresh_stopped_snapshot(
        &self,
        workspace_id: u64,
    ) -> Result<DapStoppedSnapshot, DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let (thread_id, reason) = {
            let state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            let thread_id = state.last_thread_id.unwrap_or(1);
            let reason = state
                .last_reason
                .clone()
                .unwrap_or_else(|| "pause".to_owned());
            (thread_id, reason)
        };
        let snapshot = capture_stopped_snapshot(&handle, thread_id, reason)?;
        {
            let mut state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            state.snapshot = Some(snapshot.clone());
        }
        Ok(snapshot)
    }

    /// Continues the stopped thread for a Workspace Session.
    pub fn continue_session(&self, workspace_id: u64) -> Result<(), DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let thread_id = active_thread_id(&handle)?;
        clear_stopped_snapshot(&handle)?;
        handle.request::<Continue>(ContinueArguments {
            thread_id,
            single_thread: None,
        })?;
        Ok(())
    }

    /// Pauses the active (or default) thread for a Workspace Session.
    pub fn pause_session(&self, workspace_id: u64) -> Result<(), DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let thread_id = active_thread_id(&handle).unwrap_or(1);
        handle.request::<Pause>(PauseArguments { thread_id })?;
        Ok(())
    }

    /// Steps over (`next`) on the stopped thread.
    pub fn step_over(&self, workspace_id: u64) -> Result<(), DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let thread_id = active_thread_id(&handle)?;
        clear_stopped_snapshot(&handle)?;
        handle.request::<Next>(NextArguments {
            thread_id,
            single_thread: None,
            granularity: None,
        })?;
        Ok(())
    }

    /// Steps into on the stopped thread.
    pub fn step_into(&self, workspace_id: u64) -> Result<(), DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let thread_id = active_thread_id(&handle)?;
        clear_stopped_snapshot(&handle)?;
        handle.request::<StepIn>(StepInArguments {
            thread_id,
            single_thread: None,
            target_id: None,
            granularity: None,
        })?;
        Ok(())
    }

    /// Steps out on the stopped thread.
    pub fn step_out(&self, workspace_id: u64) -> Result<(), DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let thread_id = active_thread_id(&handle)?;
        clear_stopped_snapshot(&handle)?;
        handle.request::<StepOut>(StepOutArguments {
            thread_id,
            single_thread: None,
            granularity: None,
        })?;
        Ok(())
    }

    /// Restarts the Session using the same Debug Configuration.
    pub fn restart_session(&self, workspace_id: u64) -> Result<DapSessionInfo, DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        if handle.supports_restart {
            clear_stopped_snapshot(&handle)?;
            handle.request::<Restart>(RestartArguments { raw: json!({}) })?;
            return Ok(handle.info.clone());
        }
        let plan = handle.plan.clone();
        self.stop_session(workspace_id)?;
        self.start_session(workspace_id, plan)
    }

    fn session_handle(&self, workspace_id: u64) -> Result<Arc<DapSessionHandle>, DapClientError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        sessions
            .get(&workspace_id)
            .cloned()
            .ok_or(DapClientError::SessionMissing(workspace_id))
    }

    /// Starts a Debug Session for a Workspace from an explicit plan.
    pub fn start_session(
        &self,
        workspace_id: u64,
        plan: DebugSessionPlan,
    ) -> Result<DapSessionInfo, DapClientError> {
        {
            let sessions = self
                .sessions
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            if sessions.contains_key(&workspace_id) {
                return Err(DapClientError::SessionExists(workspace_id));
            }
        }

        let adapter = self
            .registry
            .adapter(plan.adapter_id())
            .cloned()
            .ok_or_else(|| {
                DapClientError::Registry(format!(
                    "debug adapter `{}` is not registered",
                    plan.adapter_id()
                ))
            })?;

        let handle = self.open_session(workspace_id, &adapter, plan)?;
        let info = handle.info.clone();
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        sessions.insert(workspace_id, Arc::new(handle));
        Ok(info)
    }

    /// Resolves an adapter + configuration and starts a Session.
    pub fn start(
        &self,
        workspace_id: u64,
        adapter_id: Option<&str>,
        extension: Option<&str>,
        configuration: DebugConfiguration,
    ) -> Result<DapSessionInfo, DapClientError> {
        let resolved_adapter_id = match adapter_id {
            Some(adapter_id) => adapter_id.to_owned(),
            None => {
                let extension = extension.ok_or_else(|| {
                    DapClientError::Registry(
                        "dap.start needs an adapter id or a file extension to resolve one"
                            .to_owned(),
                    )
                })?;
                self.registry
                    .resolve_adapter_for_extension(extension)
                    .map(|adapter| adapter.id().to_owned())
                    .map_err(|error| DapClientError::Registry(error.to_string()))?
            }
        };
        let plan = self
            .registry
            .prepare_session(&resolved_adapter_id, configuration.clone())
            .map_err(|error| DapClientError::Registry(error.to_string()))?;
        let info = self.start_session(workspace_id, plan)?;
        self.record_start(resolved_adapter_id, configuration)?;
        Ok(info)
    }

    /// Performs Debug Stop for the Workspace Session.
    pub fn stop_session(&self, workspace_id: u64) -> Result<DapSessionInfo, DapClientError> {
        let handle = {
            let mut sessions = self
                .sessions
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            sessions
                .remove(&workspace_id)
                .ok_or(DapClientError::SessionMissing(workspace_id))?
        };

        let terminate_debuggee = match handle.info.request {
            DebugRequestKind::Launch => handle.info.support_terminate_debuggee.then_some(true),
            DebugRequestKind::Attach => {
                if handle.info.support_terminate_debuggee {
                    Some(false)
                } else {
                    None
                }
            }
        };

        let disconnect = DisconnectArguments {
            restart: None,
            terminate_debuggee,
            suspend_debuggee: None,
        };

        let disconnect_result = handle.request::<Disconnect>(disconnect.clone());
        if let Ok(mut last) = handle.last_disconnect.lock() {
            *last = Some(disconnect);
        }
        handle.disconnected.store(true, Ordering::Release);

        if let Err(error) = disconnect_result {
            record_transport_event(
                &self.transport_log,
                handle.info.adapter_id(),
                format!("disconnect failed: {error}"),
            );
        }

        if let Some(child) = handle.child.as_ref()
            && let Ok(mut child) = child.lock()
        {
            let _ = child.kill();
            let _ = child.wait();
        }

        if let Ok(mut store) = self.breakpoints.lock() {
            store.mark_all_pending(workspace_id);
        }

        Ok(handle.info.clone())
    }

    fn sync_all_breakpoints(
        &self,
        workspace_id: u64,
        handle: &DapSessionHandle,
    ) -> Result<(), DapClientError> {
        let paths = {
            let store = self
                .breakpoints
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            store.source_paths(workspace_id)
        };
        for path in paths {
            self.sync_breakpoints_for_source_with_handle(workspace_id, &path, handle)?;
        }
        Ok(())
    }

    fn sync_breakpoints_for_source(
        &self,
        workspace_id: u64,
        path: &std::path::Path,
    ) -> Result<(), DapClientError> {
        let handle = {
            let sessions = self
                .sessions
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            sessions
                .get(&workspace_id)
                .cloned()
                .ok_or(DapClientError::SessionMissing(workspace_id))?
        };
        self.sync_breakpoints_for_source_with_handle(workspace_id, path, &handle)
    }

    fn sync_breakpoints_for_source_with_handle(
        &self,
        workspace_id: u64,
        path: &std::path::Path,
        handle: &DapSessionHandle,
    ) -> Result<(), DapClientError> {
        let source_breakpoints = {
            let store = self
                .breakpoints
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            store
                .for_path(workspace_id, path)
                .into_iter()
                .map(|bp| SourceBreakpoint {
                    line: u64::from(bp.line()),
                    column: None,
                    condition: None,
                    hit_condition: None,
                    log_message: None,
                    mode: None,
                })
                .collect::<Vec<_>>()
        };

        let response = handle.request::<SetBreakpoints>(SetBreakpointsArguments {
            source: Source {
                name: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned),
                path: Some(path.display().to_string()),
                source_reference: None,
                presentation_hint: None,
                origin: None,
                sources: None,
                adapter_data: None,
                checksums: None,
            },
            breakpoints: Some(source_breakpoints.clone()),
            lines: None,
            source_modified: Some(false),
        })?;

        let mut results = Vec::with_capacity(source_breakpoints.len());
        for (index, request_bp) in source_breakpoints.iter().enumerate() {
            let verified = response
                .breakpoints
                .get(index)
                .map(|bp| bp.verified)
                .unwrap_or(false);
            let line = response
                .breakpoints
                .get(index)
                .and_then(|bp| bp.line)
                .unwrap_or(request_bp.line);
            results.push((line as u32, verified));
        }

        let mut store = self
            .breakpoints
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        store.apply_verification(workspace_id, path, &results);
        Ok(())
    }

    fn open_session(
        &self,
        workspace_id: u64,
        adapter: &DebugAdapterSpec,
        plan: DebugSessionPlan,
    ) -> Result<DapSessionHandle, DapClientError> {
        let (writer, reader, child) = connect_transport(adapter, plan.transport())?;
        let writer: Arc<Mutex<Box<dyn Write + Send>>> = Arc::new(Mutex::new(writer));
        let pending: Arc<Mutex<BTreeMap<u64, PendingResponse>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        let disconnected = Arc::new(AtomicBool::new(false));
        let last_disconnect = Arc::new(Mutex::new(None));
        let initialized = Arc::new(Mutex::new(false));
        let stop_state = Arc::new(Mutex::new(SessionStopState::default()));

        let reader_handle = spawn_reader_thread(
            workspace_id,
            adapter.id().to_owned(),
            reader,
            Arc::clone(&pending),
            Arc::clone(&disconnected),
            Arc::clone(&self.transport_log),
            Arc::clone(&initialized),
            Arc::clone(&stop_state),
            Arc::clone(&self.events),
        );

        let next_request_id = AtomicU64::new(1);
        let mut handle = DapSessionHandle {
            info: DapSessionInfo {
                workspace_id,
                adapter_id: adapter.id().to_owned(),
                configuration_name: plan.configuration().name().to_owned(),
                request: plan.configuration().request(),
                support_terminate_debuggee: false,
            },
            plan: plan.clone(),
            supports_restart: false,
            writer: Arc::clone(&writer),
            pending: Arc::clone(&pending),
            next_request_id,
            disconnected: Arc::clone(&disconnected),
            last_disconnect: Arc::clone(&last_disconnect),
            stop_state: Arc::clone(&stop_state),
            _reader: reader_handle,
            child: child.map(Mutex::new),
            transport_log: Arc::clone(&self.transport_log),
        };

        let capabilities = handle.request::<Initialize>(InitializeRequestArguments {
            client_id: Some("volt".to_owned()),
            client_name: Some("Volt".to_owned()),
            adapter_id: adapter.id().to_owned(),
            locale: Some("en-US".to_owned()),
            lines_start_at1: Some(true),
            columns_start_at1: Some(true),
            path_format: None,
            supports_variable_type: Some(true),
            supports_variable_paging: None,
            supports_run_in_terminal_request: None,
            supports_memory_references: None,
            supports_progress_reporting: None,
            supports_invalidated_event: None,
            supports_memory_event: None,
            supports_args_can_be_interpreted_by_shell: None,
            supports_start_debugging_request: None,
        })?;

        wait_for_initialized(&initialized, READ_TIMEOUT)?;

        self.sync_all_breakpoints(workspace_id, &handle)?;

        match plan.configuration().request() {
            DebugRequestKind::Launch => {
                let body = launch_arguments(plan.configuration());
                handle.request::<Launch>(LaunchRequestArguments { raw: body })?;
            }
            DebugRequestKind::Attach => {
                let body = attach_arguments(plan.configuration());
                handle.request::<Attach>(AttachRequestArguments { raw: body })?;
            }
        }

        handle.info.support_terminate_debuggee =
            capabilities.support_terminate_debuggee.unwrap_or(false);
        handle.supports_restart = capabilities.supports_restart_request.unwrap_or(false);
        Ok(handle)
    }
}

impl DapSessionHandle {
    fn request<R>(&self, arguments: R::Arguments) -> Result<R::Response, DapClientError>
    where
        R: DapRequest,
        R::Response: for<'de> Deserialize<'de>,
    {
        if self.disconnected.load(Ordering::Acquire) {
            return Err(DapClientError::Protocol(format!(
                "debug adapter `{}` disconnected",
                self.info.adapter_id
            )));
        }

        let seq = self.next_request_id.fetch_add(1, Ordering::AcqRel);
        let (tx, rx) = std::sync::mpsc::channel();
        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            pending.insert(seq, PendingResponse { tx });
        }

        let message = json!({
            "seq": seq,
            "type": "request",
            "command": R::COMMAND,
            "arguments": arguments,
        });
        write_frame(
            &self.writer,
            &self.transport_log,
            &self.info.adapter_id,
            &message,
        )?;

        let response_body = rx.recv_timeout(READ_TIMEOUT).map_err(|_| {
            DapClientError::Protocol(format!(
                "timed out waiting for `{}` response from `{}`",
                R::COMMAND,
                self.info.adapter_id
            ))
        })??;

        parse_response_body::<R>(response_body, &self.info.adapter_id)
    }
}

fn parse_response_body<R>(body: Value, adapter_id: &str) -> Result<R::Response, DapClientError>
where
    R: DapRequest,
    R::Response: for<'de> Deserialize<'de>,
{
    match serde_json::from_value::<R::Response>(body.clone()) {
        Ok(value) => Ok(value),
        Err(error) if mem::size_of::<R::Response>() == 0 => {
            // Many adapters send `{}` for unit responses.
            serde_json::from_value(Value::Null).map_err(|_| {
                DapClientError::Protocol(format!(
                    "failed to decode `{}` response from `{adapter_id}`: {error}",
                    R::COMMAND
                ))
            })
        }
        Err(error) => Err(DapClientError::Protocol(format!(
            "failed to decode `{}` response from `{adapter_id}`: {error}",
            R::COMMAND
        ))),
    }
}

fn launch_arguments(configuration: &DebugConfiguration) -> Value {
    let mut body = json!({
        "name": configuration.name(),
        "noDebug": false,
    });
    if let Some(program) = configuration.target_program() {
        body["program"] = Value::String(program.display().to_string());
    }
    if let Some(cwd) = configuration.cwd() {
        body["cwd"] = Value::String(cwd.display().to_string());
    }
    if !configuration.args().is_empty() {
        body["args"] = Value::Array(
            configuration
                .args()
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        );
    }
    body
}

fn attach_arguments(configuration: &DebugConfiguration) -> Value {
    let mut body = json!({
        "name": configuration.name(),
    });
    if let Some(program) = configuration.target_program() {
        body["program"] = Value::String(program.display().to_string());
    }
    if let Some(cwd) = configuration.cwd() {
        body["cwd"] = Value::String(cwd.display().to_string());
    }
    if let Some(process_id) = configuration.process_id() {
        body["processId"] = Value::Number(process_id.into());
    }
    body
}

fn wait_for_initialized(
    initialized: &Arc<Mutex<bool>>,
    timeout: Duration,
) -> Result<(), DapClientError> {
    let deadline = Instant::now() + timeout;
    loop {
        if *initialized
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(DapClientError::Protocol(
                "timed out waiting for DAP `initialized` event".to_owned(),
            ));
        }
        thread::sleep(Duration::from_millis(5));
    }
}

type TransportEnds = (Box<dyn Write + Send>, Box<dyn Read + Send>, Option<Child>);

fn connect_transport(
    adapter: &DebugAdapterSpec,
    transport: &DebugAdapterTransport,
) -> Result<TransportEnds, DapClientError> {
    match transport {
        DebugAdapterTransport::Stdio => {
            let mut child = spawn_adapter_command(adapter)?;
            let stdin = child.stdin.take().ok_or_else(|| {
                DapClientError::Protocol(format!(
                    "debug adapter `{}` is missing stdin pipe",
                    adapter.id()
                ))
            })?;
            let stdout = child.stdout.take().ok_or_else(|| {
                DapClientError::Protocol(format!(
                    "debug adapter `{}` is missing stdout pipe",
                    adapter.id()
                ))
            })?;
            Ok((Box::new(stdin), Box::new(stdout), Some(child)))
        }
        DebugAdapterTransport::Tcp { host, port } => {
            // Empty program means connect-only (adapter already listening), used by tests
            // and remote adapters.
            let child = if adapter.program().is_empty() {
                None
            } else {
                Some(spawn_adapter_command(adapter)?)
            };
            let stream = connect_tcp(host, *port, child.is_some())?;
            let reader = stream
                .try_clone()
                .map_err(|error| DapClientError::Io(error.to_string()))?;
            Ok((Box::new(stream), Box::new(reader), child))
        }
    }
}

fn connect_tcp(host: &str, port: u16, expect_retry: bool) -> Result<TcpStream, DapClientError> {
    let address = format!("{host}:{port}");
    let deadline = Instant::now() + TCP_CONNECT_TIMEOUT;
    let mut last_error = None;
    while Instant::now() < deadline {
        match TcpStream::connect(&address) {
            Ok(stream) => {
                stream
                    .set_read_timeout(Some(READ_TIMEOUT))
                    .map_err(|error| DapClientError::Io(error.to_string()))?;
                stream
                    .set_write_timeout(Some(READ_TIMEOUT))
                    .map_err(|error| DapClientError::Io(error.to_string()))?;
                return Ok(stream);
            }
            Err(error) => {
                last_error = Some(error);
                if !expect_retry {
                    break;
                }
                thread::sleep(TCP_CONNECT_RETRY);
            }
        }
    }
    Err(DapClientError::Io(format!(
        "failed to connect to debug adapter at `{address}`: {}",
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "timed out".to_owned())
    )))
}

fn spawn_adapter_command(adapter: &DebugAdapterSpec) -> Result<Child, DapClientError> {
    let mut command = Command::new(adapter.program());
    command
        .args(adapter.args())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    match command.spawn() {
        Ok(child) => Ok(child),
        Err(error) => Err(DapClientError::AdapterMissing {
            adapter_id: adapter.id().to_owned(),
            program: adapter.program().to_owned(),
            detail: error.to_string(),
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_reader_thread(
    workspace_id: u64,
    adapter_id: String,
    reader: Box<dyn Read + Send>,
    pending: Arc<Mutex<BTreeMap<u64, PendingResponse>>>,
    disconnected: Arc<AtomicBool>,
    transport_log: TransportLog,
    initialized: Arc<Mutex<bool>>,
    stop_state: Arc<Mutex<SessionStopState>>,
    events: Arc<Mutex<VecDeque<DapSessionEvent>>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        loop {
            if disconnected.load(Ordering::Acquire) {
                break;
            }
            match read_frame(&mut reader) {
                Ok(value) => {
                    record_transport_message(
                        &transport_log,
                        &adapter_id,
                        DapLogDirection::Receive,
                        &value,
                    );
                    let message_type = value
                        .get("type")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    match message_type {
                        "response" => {
                            let request_seq = value
                                .get("request_seq")
                                .and_then(Value::as_u64)
                                .unwrap_or_default();
                            let success = value
                                .get("success")
                                .and_then(Value::as_bool)
                                .unwrap_or(false);
                            let body = value.get("body").cloned().unwrap_or(Value::Null);
                            let result = if success {
                                Ok(body)
                            } else {
                                let message = value
                                    .get("message")
                                    .and_then(Value::as_str)
                                    .unwrap_or("DAP request failed");
                                Err(DapClientError::Protocol(format!(
                                    "debug adapter `{adapter_id}` rejected request: {message}"
                                )))
                            };
                            if let Ok(mut pending) = pending.lock()
                                && let Some(pending_response) = pending.remove(&request_seq)
                            {
                                let _ = pending_response.tx.send(result);
                            }
                        }
                        "event" => {
                            let event = value
                                .get("event")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            match event {
                                "initialized" => {
                                    if let Ok(mut flag) = initialized.lock() {
                                        *flag = true;
                                    }
                                }
                                "stopped" => {
                                    let thread_id = value
                                        .pointer("/body/threadId")
                                        .and_then(Value::as_u64)
                                        .or_else(|| {
                                            value
                                                .get("body")
                                                .and_then(|body| body.get("threadId"))
                                                .and_then(Value::as_u64)
                                        });
                                    let reason = value
                                        .pointer("/body/reason")
                                        .and_then(Value::as_str)
                                        .unwrap_or("pause")
                                        .to_owned();
                                    if let Ok(mut state) = stop_state.lock() {
                                        state.last_thread_id = thread_id.or(Some(1));
                                        state.last_reason = Some(reason);
                                        state.snapshot = None;
                                    }
                                    if let Ok(mut queue) = events.lock() {
                                        queue.push_back(DapSessionEvent::Stopped { workspace_id });
                                    }
                                }
                                "continued" => {
                                    if let Ok(mut state) = stop_state.lock() {
                                        state.snapshot = None;
                                    }
                                    if let Ok(mut queue) = events.lock() {
                                        queue
                                            .push_back(DapSessionEvent::Continued { workspace_id });
                                    }
                                }
                                _ => {}
                            }
                        }
                        "request" => {
                            // Reverse requests are ignored in this milestone.
                        }
                        _ => {}
                    }
                }
                Err(error) => {
                    record_transport_event(
                        &transport_log,
                        &adapter_id,
                        format!("transport read error: {error}"),
                    );
                    disconnected.store(true, Ordering::Release);
                    if let Ok(mut pending) = pending.lock() {
                        let pending_responses = mem::take(&mut *pending);
                        for (_, pending_response) in pending_responses {
                            let _ =
                                pending_response
                                    .tx
                                    .send(Err(DapClientError::Protocol(format!(
                                        "debug adapter `{adapter_id}` disconnected"
                                    ))));
                        }
                    }
                    break;
                }
            }
        }
    })
}

fn write_frame(
    writer: &Arc<Mutex<Box<dyn Write + Send>>>,
    transport_log: &TransportLog,
    adapter_id: &str,
    message: &Value,
) -> Result<(), DapClientError> {
    let encoded = serde_json::to_vec(message).map_err(|error| {
        DapClientError::Protocol(format!("failed to encode DAP message: {error}"))
    })?;
    let header = format!("Content-Length: {}\r\n\r\n", encoded.len());
    let mut writer = writer.lock().map_err(|_| DapClientError::LockPoisoned)?;
    writer.write_all(header.as_bytes())?;
    writer.write_all(&encoded)?;
    writer.flush()?;
    record_transport_message(transport_log, adapter_id, DapLogDirection::Send, message);
    Ok(())
}

fn read_frame<R: BufRead>(reader: &mut R) -> Result<Value, DapClientError> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Err(DapClientError::Protocol(
                "debug adapter closed the transport".to_owned(),
            ));
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            let value = value.trim().parse::<usize>().map_err(|error| {
                DapClientError::Protocol(format!("invalid Content-Length: {error}"))
            })?;
            content_length = Some(value);
        }
    }
    let length = content_length.ok_or_else(|| {
        DapClientError::Protocol("received DAP frame without Content-Length".to_owned())
    })?;
    let mut buffer = vec![0_u8; length];
    reader.read_exact(&mut buffer)?;
    serde_json::from_slice(&buffer)
        .map_err(|error| DapClientError::Protocol(format!("failed to decode DAP frame: {error}")))
}

fn record_transport_message(
    transport_log: &TransportLog,
    adapter_id: &str,
    direction: DapLogDirection,
    message: &Value,
) {
    let rendered = serde_json::to_string(message).unwrap_or_else(|_| "<unprintable>".to_owned());
    record_transport_event_inner(transport_log, adapter_id, direction, rendered);
}

fn record_transport_event(
    transport_log: &TransportLog,
    adapter_id: &str,
    message: impl Into<String>,
) {
    record_transport_event_inner(
        transport_log,
        adapter_id,
        DapLogDirection::Event,
        message.into(),
    );
}

fn record_transport_event_inner(
    transport_log: &TransportLog,
    adapter_id: &str,
    direction: DapLogDirection,
    message: String,
) {
    if let Ok(mut log) = transport_log.lock() {
        log.push(DapLogEntry {
            adapter_id: adapter_id.to_owned(),
            direction,
            message,
        });
    }
}

fn active_thread_id(handle: &DapSessionHandle) -> Result<u64, DapClientError> {
    let state = handle
        .stop_state
        .lock()
        .map_err(|_| DapClientError::LockPoisoned)?;
    state.last_thread_id.ok_or_else(|| {
        DapClientError::Protocol(
            "Debug Session has no stopped thread; wait for a stop before stepping".to_owned(),
        )
    })
}

fn clear_stopped_snapshot(handle: &DapSessionHandle) -> Result<(), DapClientError> {
    let mut state = handle
        .stop_state
        .lock()
        .map_err(|_| DapClientError::LockPoisoned)?;
    state.snapshot = None;
    Ok(())
}

fn capture_stopped_snapshot(
    handle: &DapSessionHandle,
    thread_id: u64,
    reason: String,
) -> Result<DapStoppedSnapshot, DapClientError> {
    let stack = handle.request::<StackTrace>(StackTraceArguments {
        thread_id,
        start_frame: Some(0),
        levels: Some(1),
        format: None,
    })?;
    let top = stack.stack_frames.first();
    let position = top.and_then(|frame| {
        let path = frame
            .source
            .as_ref()
            .and_then(|source| source.path.as_ref())
            .map(PathBuf::from)?;
        let line = u32::try_from(frame.line).ok().filter(|line| *line > 0)?;
        let column = u32::try_from(frame.column).unwrap_or(1).max(1);
        Some(DapExecutionPosition { path, line, column })
    });

    let mut locals = Vec::new();
    if let Some(frame) = top {
        let scopes = handle.request::<Scopes>(ScopesArguments { frame_id: frame.id })?;
        let locals_scope = scopes
            .scopes
            .iter()
            .find(|scope| {
                scope.name.eq_ignore_ascii_case("locals")
                    || matches!(
                        scope.presentation_hint,
                        Some(dap_types::ScopePresentationHint::Locals)
                    )
            })
            .or_else(|| scopes.scopes.first());
        if let Some(scope) = locals_scope
            && scope.variables_reference > 0
        {
            let response = handle.request::<Variables>(VariablesArguments {
                variables_reference: scope.variables_reference,
                filter: None,
                start: None,
                count: None,
                format: None,
            })?;
            locals = response
                .variables
                .into_iter()
                .map(|variable| DapLocalVariable {
                    name: variable.name,
                    value: variable.value,
                    type_name: variable.type_,
                })
                .collect();
        }
    }

    Ok(DapStoppedSnapshot {
        thread_id,
        reason,
        position,
        locals,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write, pipe},
        net::TcpListener,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::{Duration, Instant},
    };

    use dap_types::DisconnectArguments;
    use serde_json::{Value, json};

    use super::{DapClientError, DapClientManager, DapLogDirection, DapSessionEvent, read_frame};
    use crate::{
        BreakpointState, BreakpointToggle, DebugAdapterRegistry, DebugAdapterSpec,
        DebugAdapterTransport, DebugConfiguration, DebugRequestKind,
    };

    fn write_frame_to(writer: &mut impl Write, message: &Value) {
        let encoded = serde_json::to_vec(message).expect("encode");
        write!(writer, "Content-Length: {}\r\n\r\n", encoded.len()).expect("header");
        writer.write_all(&encoded).expect("body");
        writer.flush().expect("flush");
    }

    fn fake_adapter_loop(
        reader: impl Read,
        mut writer: impl Write,
        last_disconnect: Arc<Mutex<Option<DisconnectArguments>>>,
    ) {
        let mut reader = std::io::BufReader::new(reader);
        let mut seq = 1_u64;
        let mut stopped_line = 10_u64;
        let mut program_path = "main.rs".to_owned();
        let mut running = false;
        while let Ok(message) = read_frame(&mut reader) {
            let command = message
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let request_seq = message.get("seq").and_then(Value::as_u64).unwrap_or(0);
            match command.as_str() {
                "initialize" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "initialize",
                        "body": {
                            "supportsConfigurationDoneRequest": false,
                            "supportTerminateDebuggee": true,
                            "supportsRestartRequest": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "initialized",
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "launch" | "attach" => {
                    if let Some(program) = message
                        .pointer("/arguments/program")
                        .and_then(Value::as_str)
                    {
                        program_path = program.to_owned();
                    }
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": command,
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = false;
                    stopped_line = 10;
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "entry",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "setBreakpoints" => {
                    let breakpoints = message
                        .pointer("/arguments/breakpoints")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let body_breakpoints: Vec<Value> = breakpoints
                        .iter()
                        .map(|bp| {
                            let line = bp.get("line").and_then(Value::as_u64).unwrap_or(1);
                            // Verify odd lines; leave even lines unverified for tests.
                            json!({
                                "verified": line % 2 == 1,
                                "line": line
                            })
                        })
                        .collect();
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "setBreakpoints",
                        "body": { "breakpoints": body_breakpoints }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "continue" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "continue",
                        "body": { "allThreadsContinued": true }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = true;
                }
                "next" | "stepIn" | "stepOut" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": command,
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = false;
                    stopped_line += 1;
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "step",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "pause" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "pause",
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = false;
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "pause",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "restart" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "restart",
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = false;
                    stopped_line = 10;
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "entry",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "stackTrace" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "stackTrace",
                        "body": {
                            "stackFrames": [{
                                "id": 1,
                                "name": "main",
                                "source": {
                                    "name": "main.rs",
                                    "path": program_path
                                },
                                "line": stopped_line,
                                "column": 1
                            }],
                            "totalFrames": 1
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "scopes" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "scopes",
                        "body": {
                            "scopes": [{
                                "name": "Locals",
                                "variablesReference": 1,
                                "expensive": false
                            }]
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "variables" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "variables",
                        "body": {
                            "variables": [{
                                "name": "x",
                                "value": "42",
                                "type": "i32",
                                "variablesReference": 0
                            }, {
                                "name": "running",
                                "value": if running { "true" } else { "false" },
                                "type": "bool",
                                "variablesReference": 0
                            }]
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "disconnect" => {
                    let args = message.get("arguments").cloned().unwrap_or(Value::Null);
                    let parsed: DisconnectArguments =
                        serde_json::from_value(args).unwrap_or(DisconnectArguments {
                            restart: None,
                            terminate_debuggee: None,
                            suspend_debuggee: None,
                        });
                    if let Ok(mut slot) = last_disconnect.lock() {
                        *slot = Some(parsed);
                    }
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "disconnect",
                        "body": {}
                    });
                    write_frame_to(&mut writer, &response);
                    break;
                }
                _ => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": false,
                        "command": command,
                        "message": "unsupported"
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
            }
        }
    }

    fn tcp_fake_spec(port: u16) -> DebugAdapterSpec {
        DebugAdapterSpec::new("fake-dap", "rust", ["rs"], "", [] as [&str; 0])
            .with_transport(DebugAdapterTransport::Tcp {
                host: "127.0.0.1".to_owned(),
                port,
            })
            .with_preference(10)
    }

    /// Prefer TCP fake for full client protocol tests; framing covered separately for stdio.
    fn start_tcp_fake() -> (
        u16,
        Arc<Mutex<Option<DisconnectArguments>>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let last_disconnect = Arc::new(Mutex::new(None));
        let last_disconnect_thread = Arc::clone(&last_disconnect);
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let reader = stream.try_clone().expect("clone");
            fake_adapter_loop(reader, stream, last_disconnect_thread);
        });
        (port, last_disconnect, handle)
    }

    #[test]
    fn client_initialize_launch_disconnect_against_fake_tcp_adapter() {
        let (port, last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");

        let manager = DapClientManager::new(registry);
        let info = manager
            .start(
                7,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("launch demo", DebugRequestKind::Launch)
                    .with_target_program("target/debug/demo"),
            )
            .expect("start");
        assert_eq!(info.workspace_id(), 7);
        assert_eq!(info.adapter_id(), "fake-dap");
        assert!(info.support_terminate_debuggee());
        assert!(manager.session_info(7).expect("info").is_some());

        let stopped = manager.stop_session(7).expect("stop");
        assert_eq!(stopped.adapter_id(), "fake-dap");
        assert!(manager.session_info(7).expect("info").is_none());

        // Give reader/fake a moment to record disconnect.
        thread::sleep(Duration::from_millis(50));
        let disconnect = last_disconnect
            .lock()
            .expect("lock")
            .clone()
            .expect("disconnect args");
        assert_eq!(disconnect.terminate_debuggee, Some(true));

        let _ = fake.join();
        let log = manager.log_snapshot();
        assert!(log.entries().iter().any(|entry| {
            entry.direction() == DapLogDirection::Send && entry.message().contains("initialize")
        }));
    }

    #[test]
    fn debug_stop_after_attach_leaves_process_running() {
        let (port, last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");

        let manager = DapClientManager::new(registry);
        manager
            .start(
                3,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("attach demo", DebugRequestKind::Attach),
            )
            .expect("start");
        manager.stop_session(3).expect("stop");
        thread::sleep(Duration::from_millis(50));
        let disconnect = last_disconnect
            .lock()
            .expect("lock")
            .clone()
            .expect("disconnect args");
        assert_eq!(disconnect.terminate_debuggee, Some(false));
        let _ = fake.join();
    }

    #[test]
    fn one_session_per_workspace_enforced() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                1,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("one", DebugRequestKind::Launch),
            )
            .expect("start");
        let err = manager
            .start(
                1,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("two", DebugRequestKind::Launch),
            )
            .expect_err("second start");
        assert!(matches!(err, DapClientError::SessionExists(1)));
        manager.stop_session(1).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn missing_adapter_binary_is_clear() {
        let mut registry = DebugAdapterRegistry::new();
        registry
            .register(DebugAdapterSpec::new(
                "missing",
                "rust",
                ["rs"],
                "volt-definitely-missing-dap-adapter-xyz",
                [] as [&str; 0],
            ))
            .expect("register");
        let manager = DapClientManager::new(registry);
        let error = manager
            .start(
                9,
                Some("missing"),
                None,
                DebugConfiguration::new("missing", DebugRequestKind::Launch),
            )
            .expect_err("missing binary");
        let message = error.to_string();
        assert!(message.contains("volt-definitely-missing-dap-adapter-xyz"));
        assert!(message.contains("install the adapter"));
        assert!(matches!(error, DapClientError::AdapterMissing { .. }));
    }

    #[test]
    fn toggle_breakpoint_without_session_stays_pending() {
        let manager = DapClientManager::new(DebugAdapterRegistry::new());
        assert_eq!(
            manager.toggle_breakpoint(1, "main.rs", 3).expect("toggle"),
            BreakpointToggle::Added
        );
        let listed = manager.list_breakpoints(1).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].state(), BreakpointState::Pending);
        assert_eq!(
            manager
                .toggle_breakpoint(1, "main.rs", 3)
                .expect("toggle off"),
            BreakpointToggle::Removed
        );
        assert!(manager.list_breakpoints(1).expect("list").is_empty());
    }

    #[test]
    fn start_session_syncs_stored_breakpoints_to_adapter() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .toggle_breakpoint(4, "src/main.rs", 5)
            .expect("toggle odd");
        manager
            .toggle_breakpoint(4, "src/main.rs", 8)
            .expect("toggle even");

        manager
            .start(
                4,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("sync", DebugRequestKind::Launch)
                    .with_target_program("target/debug/demo"),
            )
            .expect("start");

        let bps = manager
            .breakpoints_for_path(4, std::path::Path::new("src/main.rs"))
            .expect("bps");
        assert_eq!(bps.len(), 2);
        assert_eq!(bps[0].line(), 5);
        assert_eq!(bps[0].state(), BreakpointState::Verified);
        assert_eq!(bps[1].line(), 8);
        assert_eq!(bps[1].state(), BreakpointState::Unverified);

        let log = manager.log_snapshot();
        assert!(log.entries().iter().any(|entry| {
            entry.direction() == DapLogDirection::Send && entry.message().contains("setBreakpoints")
        }));

        manager.stop_session(4).expect("stop");
        let pending = manager.list_breakpoints(4).expect("list");
        assert!(
            pending
                .iter()
                .all(|bp| bp.state() == BreakpointState::Pending)
        );
        let _ = fake.join();
    }

    #[test]
    fn live_toggle_calls_set_breakpoints() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                5,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("live", DebugRequestKind::Launch)
                    .with_target_program("target/debug/demo"),
            )
            .expect("start");

        manager
            .toggle_breakpoint(5, "lib.rs", 11)
            .expect("live toggle");
        let bps = manager
            .breakpoints_for_path(5, std::path::Path::new("lib.rs"))
            .expect("bps");
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0].state(), BreakpointState::Verified);

        let set_count = manager
            .log_snapshot()
            .entries()
            .iter()
            .filter(|entry| {
                entry.direction() == DapLogDirection::Send
                    && entry.message().contains("setBreakpoints")
            })
            .count();
        assert!(set_count >= 1);

        manager.stop_session(5).expect("stop");
        let _ = fake.join();
    }

    fn wait_for_stopped(manager: &DapClientManager, workspace_id: u64) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            let events = manager.drain_events().expect("drain");
            if events.iter().any(|event| {
                matches!(
                    event,
                    DapSessionEvent::Stopped {
                        workspace_id: id
                    } if *id == workspace_id
                )
            }) {
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!("timed out waiting for stopped event");
    }

    #[test]
    fn continue_step_pause_and_locals_against_fake_adapter() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                11,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("step", DebugRequestKind::Launch)
                    .with_target_program("src/main.rs"),
            )
            .expect("start");

        wait_for_stopped(&manager, 11);
        let snapshot = manager.refresh_stopped_snapshot(11).expect("snapshot");
        assert_eq!(snapshot.reason(), "entry");
        let position = snapshot.position().expect("position");
        assert_eq!(position.path().as_os_str(), "src/main.rs");
        assert_eq!(position.line(), 10);
        assert!(
            snapshot
                .locals()
                .iter()
                .any(|local| local.name() == "x" && local.value() == "42"),
            "locals should include x=42"
        );

        manager.continue_session(11).expect("continue");
        manager.pause_session(11).expect("pause");
        wait_for_stopped(&manager, 11);
        let paused = manager.refresh_stopped_snapshot(11).expect("paused");
        assert_eq!(paused.reason(), "pause");

        manager.step_over(11).expect("step");
        wait_for_stopped(&manager, 11);
        let stepped = manager.refresh_stopped_snapshot(11).expect("stepped");
        assert_eq!(stepped.reason(), "step");
        assert_eq!(stepped.position().expect("pos").line(), 11);

        manager.step_into(11).expect("into");
        wait_for_stopped(&manager, 11);
        manager.step_out(11).expect("out");
        wait_for_stopped(&manager, 11);
        assert_eq!(
            manager
                .refresh_stopped_snapshot(11)
                .expect("out snap")
                .position()
                .expect("pos")
                .line(),
            13
        );

        manager.stop_session(11).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn restart_reuses_configuration_against_fake_adapter() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                12,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("restart-me", DebugRequestKind::Launch)
                    .with_target_program("target/debug/demo"),
            )
            .expect("start");
        wait_for_stopped(&manager, 12);
        manager.step_over(12).expect("step");
        wait_for_stopped(&manager, 12);
        assert_eq!(
            manager
                .refresh_stopped_snapshot(12)
                .expect("before")
                .position()
                .expect("pos")
                .line(),
            11
        );

        let info = manager.restart_session(12).expect("restart");
        assert_eq!(info.configuration_name(), "restart-me");
        assert!(manager.session_info(12).expect("still live").is_some());
        wait_for_stopped(&manager, 12);
        let after = manager.refresh_stopped_snapshot(12).expect("after");
        assert_eq!(after.position().expect("pos").line(), 10);

        manager.stop_session(12).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn stdio_framing_round_trips_initialize() {
        let (client_reader, adapter_writer) = pipe().expect("pipe");
        let (adapter_reader, mut client_writer) = pipe().expect("pipe");
        let last_disconnect = Arc::new(Mutex::new(None));
        let last_disconnect_thread = Arc::clone(&last_disconnect);
        let done = Arc::new(AtomicBool::new(false));
        let done_thread = Arc::clone(&done);
        let fake = thread::spawn(move || {
            fake_adapter_loop(adapter_reader, adapter_writer, last_disconnect_thread);
            done_thread.store(true, Ordering::Release);
        });

        // Drive a minimal handshake using raw frames to prove stdio framing works.
        write_frame_to(
            &mut client_writer,
            &json!({
                "seq": 1,
                "type": "request",
                "command": "initialize",
                "arguments": { "adapterID": "fake" }
            }),
        );
        let mut reader = std::io::BufReader::new(client_reader);
        let response = read_frame(&mut reader).expect("initialize response");
        assert_eq!(response["command"], "initialize");
        assert_eq!(response["success"], true);
        let event = read_frame(&mut reader).expect("initialized event");
        assert_eq!(event["event"], "initialized");

        write_frame_to(
            &mut client_writer,
            &json!({
                "seq": 2,
                "type": "request",
                "command": "launch",
                "arguments": { "program": "demo" }
            }),
        );
        let launch = read_frame(&mut reader).expect("launch response");
        assert_eq!(launch["command"], "launch");
        let stopped = read_frame(&mut reader).expect("stopped event");
        assert_eq!(stopped["event"], "stopped");

        write_frame_to(
            &mut client_writer,
            &json!({
                "seq": 3,
                "type": "request",
                "command": "disconnect",
                "arguments": { "terminateDebuggee": true }
            }),
        );
        let disconnect = read_frame(&mut reader).expect("disconnect response");
        assert_eq!(disconnect["command"], "disconnect");
        let _ = fake.join();
        assert!(done.load(Ordering::Acquire));
        assert_eq!(
            last_disconnect
                .lock()
                .expect("lock")
                .as_ref()
                .and_then(|args| args.terminate_debuggee),
            Some(true)
        );
    }
}
