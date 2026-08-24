//! Live DAP client: transport, handshake, and one Debug Session per Workspace.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
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
    AttachRequestArguments, ContinueArguments, DisconnectArguments, EvaluateArguments,
    EvaluateArgumentsContext, InitializeRequestArguments, LaunchRequestArguments, NextArguments,
    PauseArguments, RestartArguments, ScopesArguments, SetBreakpointsArguments, Source,
    SourceBreakpoint, StackTraceArguments, StepInArguments, StepOutArguments, VariablesArguments,
    requests::{
        Attach, Continue, Disconnect, Evaluate, Initialize, Launch, Next, Pause,
        Request as DapRequest, Restart, Scopes, SetBreakpoints, StackTrace, StepIn, StepOut,
        Threads, Variables,
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

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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

/// Structured Locals or Watch child node. `variables_reference > 0` means expandable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapVariableNode {
    name: String,
    value: String,
    type_name: Option<String>,
    variables_reference: u64,
    children: Vec<DapVariableNode>,
    expanded: bool,
}

/// One Locals variable row for the Debug Layout.
pub type DapLocalVariable = DapVariableNode;

impl DapVariableNode {
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

    /// Returns the DAP `variablesReference`. Non-zero means children can be fetched.
    pub const fn variables_reference(&self) -> u64 {
        self.variables_reference
    }

    /// Returns whether this node can be expanded.
    pub const fn expandable(&self) -> bool {
        self.variables_reference > 0
    }

    /// Returns whether children are currently shown.
    pub const fn expanded(&self) -> bool {
        self.expanded
    }

    /// Returns fetched children (empty when collapsed or a leaf).
    pub fn children(&self) -> &[DapVariableNode] {
        &self.children
    }
}

/// Stable expand path: Locals (`watch = None`) or a Watch Expression plus member names.
///
/// Paths use names, not live `variablesReference` values, so expansion survives a step.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DapVariablePath {
    watch: Option<String>,
    segments: Vec<String>,
}

impl DapVariablePath {
    /// Path into Locals, e.g. `["person", "Address"]`.
    pub fn locals(segments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            watch: None,
            segments: segments.into_iter().map(Into::into).collect(),
        }
    }

    /// Path under a Watch Expression. Empty `segments` is the watch root.
    pub fn watch(
        expression: impl Into<String>,
        segments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            watch: Some(expression.into()),
            segments: segments.into_iter().map(Into::into).collect(),
        }
    }

    /// Watch Expression text, or `None` for Locals.
    pub fn watch_expression(&self) -> Option<&str> {
        self.watch.as_deref()
    }

    /// Member-name segments under the Locals root or Watch Expression.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    fn is_prefix_of(&self, other: &Self) -> bool {
        self.watch == other.watch && other.segments.starts_with(&self.segments)
    }
}

/// Flattened tree row for Locals / Expressions rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapVariableRow {
    path: DapVariablePath,
    depth: usize,
    expandable: bool,
    expanded: bool,
    name: String,
    value: String,
    type_name: Option<String>,
    ok: bool,
}

impl DapVariableRow {
    /// Returns the expand path for this row.
    pub fn path(&self) -> &DapVariablePath {
        &self.path
    }

    /// Returns indent depth (0 = root).
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns whether this row can expand.
    pub const fn expandable(&self) -> bool {
        self.expandable
    }

    /// Returns whether this row is expanded.
    pub const fn expanded(&self) -> bool {
        self.expanded
    }

    /// Returns the display name (variable or Watch Expression).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the display value or error text.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the optional type name.
    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    /// Returns whether evaluation succeeded (Locals always `true`).
    pub const fn ok(&self) -> bool {
        self.ok
    }
}

/// Context for one-shot / Watch / REPL evaluate requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DapEvaluateContext {
    /// Watch Expressions section.
    Watch,
    /// Debug REPL Popup.
    Repl,
    /// Hover / eval-at-point.
    Hover,
}

impl From<DapEvaluateContext> for EvaluateArgumentsContext {
    fn from(value: DapEvaluateContext) -> Self {
        match value {
            DapEvaluateContext::Watch => Self::Watch,
            DapEvaluateContext::Repl => Self::Repl,
            DapEvaluateContext::Hover => Self::Hover,
        }
    }
}

/// One Watch Expression row for the Expressions section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapWatchExpression {
    expression: String,
    value: String,
    type_name: Option<String>,
    ok: bool,
    variables_reference: u64,
    children: Vec<DapVariableNode>,
    expanded: bool,
}

impl DapWatchExpression {
    /// Returns the watched expression text.
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Returns the evaluated value or error message.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the optional type name.
    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    /// Returns whether evaluation succeeded.
    pub const fn ok(&self) -> bool {
        self.ok
    }

    /// Returns the DAP `variablesReference`. Non-zero means children can be fetched.
    pub const fn variables_reference(&self) -> u64 {
        self.variables_reference
    }

    /// Returns whether this Watch Expression can be expanded.
    pub const fn expandable(&self) -> bool {
        self.ok && self.variables_reference > 0
    }

    /// Returns whether children are currently shown.
    pub const fn expanded(&self) -> bool {
        self.expanded
    }

    /// Returns fetched children (empty when collapsed or a leaf).
    pub fn children(&self) -> &[DapVariableNode] {
        &self.children
    }
}

/// Thread row for switch-thread pickers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapThreadInfo {
    id: u64,
    name: String,
}

impl DapThreadInfo {
    /// Returns the DAP thread id.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the thread name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Stack frame row for switch-stack-frame pickers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapStackFrameInfo {
    id: u64,
    name: String,
    path: Option<PathBuf>,
    line: u32,
    column: u32,
}

impl DapStackFrameInfo {
    /// Returns the DAP frame id.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the frame name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional source path.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
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

/// Snapshot captured when a Session stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapStoppedSnapshot {
    thread_id: u64,
    frame_id: Option<u64>,
    reason: String,
    position: Option<DapExecutionPosition>,
    locals: Vec<DapLocalVariable>,
    watches: Vec<DapWatchExpression>,
}

impl DapStoppedSnapshot {
    /// Returns the stopped thread id.
    pub const fn thread_id(&self) -> u64 {
        self.thread_id
    }

    /// Returns the active stack frame id, when known.
    pub const fn frame_id(&self) -> Option<u64> {
        self.frame_id
    }

    /// Returns the stop reason string.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns the top-frame execution position, when known.
    pub fn position(&self) -> Option<&DapExecutionPosition> {
        self.position.as_ref()
    }

    /// Returns Locals rows for the active frame.
    pub fn locals(&self) -> &[DapLocalVariable] {
        &self.locals
    }

    /// Returns Watch Expression rows for the active frame.
    pub fn watches(&self) -> &[DapWatchExpression] {
        &self.watches
    }

    /// Returns flattened Locals rows including expanded children.
    pub fn local_rows(&self) -> Vec<DapVariableRow> {
        let mut rows = Vec::new();
        flatten_variable_nodes(&self.locals, None, &[], 0, &mut rows);
        rows
    }

    /// Returns flattened Watch Expression rows including expanded children.
    pub fn watch_rows(&self) -> Vec<DapVariableRow> {
        let mut rows = Vec::new();
        for watch in &self.watches {
            rows.push(DapVariableRow {
                path: DapVariablePath::watch(&watch.expression, Vec::<String>::new()),
                depth: 0,
                expandable: watch.expandable(),
                expanded: watch.expanded,
                name: watch.expression.clone(),
                value: watch.value.clone(),
                type_name: watch.type_name.clone(),
                ok: watch.ok,
            });
            if watch.expanded {
                flatten_variable_nodes(
                    &watch.children,
                    Some(watch.expression.as_str()),
                    &[],
                    1,
                    &mut rows,
                );
            }
        }
        rows
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
    /// Adapter reported `exited` or `terminated`; run Debug Stop cleanup.
    Terminated {
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
    selected_thread_id: Option<u64>,
    selected_frame_id: Option<u64>,
    last_reason: Option<String>,
    snapshot: Option<DapStoppedSnapshot>,
    ended: bool,
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
    watches: Mutex<BTreeMap<u64, Vec<String>>>,
    expanded: Mutex<BTreeMap<u64, BTreeSet<DapVariablePath>>>,
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
            watches: Mutex::new(BTreeMap::new()),
            expanded: Mutex::new(BTreeMap::new()),
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

    /// Sets Breakpoint condition / hit condition / log message at the cursor line.
    /// Creates the Breakpoint when missing. Empty strings clear the field.
    pub fn set_breakpoint_extras(
        &self,
        workspace_id: u64,
        path: impl Into<std::path::PathBuf>,
        line: u32,
        condition: Option<Option<String>>,
        hit_condition: Option<Option<String>>,
        log_message: Option<Option<String>>,
    ) -> Result<(), DapClientError> {
        let path = path.into();
        {
            let mut store = self
                .breakpoints
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            store.upsert_extras(
                workspace_id,
                path.clone(),
                line,
                condition,
                hit_condition,
                log_message,
            );
        }
        if self.session_info(workspace_id)?.is_some() {
            self.sync_breakpoints_for_source(workspace_id, &path)?;
        }
        Ok(())
    }

    /// Lists Watch Expressions for a Workspace.
    pub fn list_expressions(&self, workspace_id: u64) -> Result<Vec<String>, DapClientError> {
        let watches = self
            .watches
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        Ok(watches.get(&workspace_id).cloned().unwrap_or_default())
    }

    /// Adds a Watch Expression (deduped). Re-evaluates when stopped.
    pub fn add_expression(
        &self,
        workspace_id: u64,
        expression: impl Into<String>,
    ) -> Result<Vec<String>, DapClientError> {
        let expression = expression.into().trim().to_owned();
        if expression.is_empty() {
            return Err(DapClientError::Protocol(
                "Watch Expression cannot be empty".to_owned(),
            ));
        }
        {
            let mut watches = self
                .watches
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            let entries = watches.entry(workspace_id).or_default();
            if !entries.iter().any(|existing| existing == &expression) {
                entries.push(expression);
            }
        }
        if self.stopped_snapshot(workspace_id)?.is_some() {
            let _ = self.refresh_stopped_snapshot(workspace_id)?;
        }
        self.list_expressions(workspace_id)
    }

    /// Removes a Watch Expression by exact text. Returns whether one was removed.
    pub fn remove_expression(
        &self,
        workspace_id: u64,
        expression: &str,
    ) -> Result<bool, DapClientError> {
        let removed = {
            let mut watches = self
                .watches
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            let Some(entries) = watches.get_mut(&workspace_id) else {
                return Ok(false);
            };
            let before = entries.len();
            entries.retain(|existing| existing != expression);
            let removed = entries.len() != before;
            if entries.is_empty() {
                watches.remove(&workspace_id);
            }
            removed
        };
        if removed && self.stopped_snapshot(workspace_id)?.is_some() {
            let _ = self.refresh_stopped_snapshot(workspace_id)?;
        }
        Ok(removed)
    }

    /// Replaces the Watch Expression list for a Workspace (empty entries dropped, order kept).
    pub fn set_expressions(
        &self,
        workspace_id: u64,
        expressions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Vec<String>, DapClientError> {
        let mut cleaned = Vec::new();
        for expression in expressions {
            let expression = expression.into().trim().to_owned();
            if expression.is_empty() {
                continue;
            }
            if !cleaned.iter().any(|existing| existing == &expression) {
                cleaned.push(expression);
            }
        }
        {
            let mut watches = self
                .watches
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            if cleaned.is_empty() {
                watches.remove(&workspace_id);
            } else {
                watches.insert(workspace_id, cleaned.clone());
            }
        }
        if self.stopped_snapshot(workspace_id)?.is_some() {
            let _ = self.refresh_stopped_snapshot(workspace_id)?;
        }
        self.list_expressions(workspace_id)
    }

    /// One-shot evaluate without adding a Watch Expression.
    pub fn evaluate(
        &self,
        workspace_id: u64,
        expression: &str,
        context: DapEvaluateContext,
    ) -> Result<DapWatchExpression, DapClientError> {
        let expression = expression.trim();
        if expression.is_empty() {
            return Err(DapClientError::Protocol(
                "evaluate expression cannot be empty".to_owned(),
            ));
        }
        let handle = self.session_handle(workspace_id)?;
        let frame_id = {
            let state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            state
                .selected_frame_id
                .or_else(|| state.snapshot.as_ref().and_then(|snap| snap.frame_id))
        };
        evaluate_expression(&handle, expression, frame_id, context.into())
    }

    /// Lists adapter threads for the live Session.
    pub fn list_threads(&self, workspace_id: u64) -> Result<Vec<DapThreadInfo>, DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let response = handle.request::<Threads>(())?;
        Ok(response
            .threads
            .into_iter()
            .map(|thread| DapThreadInfo {
                id: thread.id,
                name: thread.name,
            })
            .collect())
    }

    /// Switches the active stopped thread and refreshes Locals/watches.
    pub fn switch_thread(
        &self,
        workspace_id: u64,
        thread_id: u64,
    ) -> Result<DapStoppedSnapshot, DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        {
            let mut state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            state.selected_thread_id = Some(thread_id);
            state.last_thread_id = Some(thread_id);
            state.selected_frame_id = None;
        }
        self.refresh_stopped_snapshot(workspace_id)
    }

    /// Lists stack frames for the active (or selected) thread.
    pub fn list_stack_frames(
        &self,
        workspace_id: u64,
    ) -> Result<Vec<DapStackFrameInfo>, DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let thread_id = active_thread_id(&handle)?;
        let stack = handle.request::<StackTrace>(StackTraceArguments {
            thread_id,
            start_frame: Some(0),
            levels: Some(32),
            format: None,
        })?;
        Ok(stack
            .stack_frames
            .into_iter()
            .map(|frame| DapStackFrameInfo {
                id: frame.id,
                name: frame.name,
                path: frame
                    .source
                    .as_ref()
                    .and_then(|source| source.path.as_ref())
                    .map(PathBuf::from),
                line: u32::try_from(frame.line).unwrap_or(0),
                column: u32::try_from(frame.column).unwrap_or(1).max(1),
            })
            .collect())
    }

    /// Switches the active stack frame and refreshes Locals/watches.
    pub fn switch_stack_frame(
        &self,
        workspace_id: u64,
        frame_id: u64,
    ) -> Result<DapStoppedSnapshot, DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        {
            let mut state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            state.selected_frame_id = Some(frame_id);
        }
        self.refresh_stopped_snapshot(workspace_id)
    }

    /// Expands or collapses a Locals / Watch tree node and fetches children when expanding.
    pub fn toggle_variable_expand(
        &self,
        workspace_id: u64,
        path: &DapVariablePath,
    ) -> Result<DapStoppedSnapshot, DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let mut snapshot = {
            let state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            state.snapshot.clone().ok_or_else(|| {
                DapClientError::Protocol(
                    "Debug Session has no stopped snapshot; wait for a stop before expanding variables"
                        .to_owned(),
                )
            })?
        };

        let currently_expanded = {
            let expanded = self
                .expanded
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            expanded
                .get(&workspace_id)
                .is_some_and(|set| set.contains(path))
        };

        if currently_expanded {
            collapse_variable_path(&mut snapshot, path);
            let mut expanded = self
                .expanded
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            if let Some(set) = expanded.get_mut(&workspace_id) {
                set.retain(|existing| !path.is_prefix_of(existing));
                if set.is_empty() {
                    expanded.remove(&workspace_id);
                }
            }
        } else if expand_variable_path(&handle, &mut snapshot, path)? {
            let mut expanded = self
                .expanded
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            expanded
                .entry(workspace_id)
                .or_default()
                .insert(path.clone());
        }

        {
            let mut state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            state.snapshot = Some(snapshot.clone());
        }
        Ok(snapshot)
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

    /// Fetches stack/locals/watches for the active stopped thread/frame.
    pub fn refresh_stopped_snapshot(
        &self,
        workspace_id: u64,
    ) -> Result<DapStoppedSnapshot, DapClientError> {
        let handle = self.session_handle(workspace_id)?;
        let (thread_id, frame_id, reason) = {
            let state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            let thread_id = state
                .selected_thread_id
                .or(state.last_thread_id)
                .unwrap_or(1);
            let frame_id = state.selected_frame_id;
            let reason = state
                .last_reason
                .clone()
                .unwrap_or_else(|| "pause".to_owned());
            (thread_id, frame_id, reason)
        };
        let watches = self.list_expressions(workspace_id)?;
        let expanded = {
            let expanded = self
                .expanded
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            expanded.get(&workspace_id).cloned().unwrap_or_default()
        };
        let snapshot =
            capture_stopped_snapshot(&handle, thread_id, frame_id, reason, &watches, &expanded)?;
        {
            let mut state = handle
                .stop_state
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            if state.selected_frame_id.is_none() {
                state.selected_frame_id = snapshot.frame_id;
            }
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
                    condition: bp.condition().map(str::to_owned),
                    hit_condition: bp.hit_condition().map(str::to_owned),
                    log_message: bp.log_message().map(str::to_owned),
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

        wait_for_initialized(&initialized, READ_TIMEOUT)?;
        self.sync_all_breakpoints(workspace_id, &handle)?;
        if capabilities
            .supports_configuration_done_request
            .unwrap_or(false)
        {
            send_configuration_done(&handle)?;
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

        let mut message = json!({
            "seq": seq,
            "type": "request",
            "command": R::COMMAND,
            "arguments": arguments,
        });
        if let Some(args) = message.get_mut("arguments") {
            strip_null_fields(args);
        }
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

fn send_configuration_done(handle: &DapSessionHandle) -> Result<(), DapClientError> {
    let seq = handle.next_request_id.fetch_add(1, Ordering::AcqRel);
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut pending = handle
            .pending
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        pending.insert(seq, PendingResponse { tx });
    }
    let message = json!({
        "seq": seq,
        "type": "request",
        "command": "configurationDone",
        "arguments": {}
    });
    write_frame(
        &handle.writer,
        &handle.transport_log,
        &handle.info.adapter_id,
        &message,
    )?;
    let _ = rx.recv_timeout(READ_TIMEOUT).map_err(|_| {
        DapClientError::Protocol(format!(
            "timed out waiting for `configurationDone` response from `{}`",
            handle.info.adapter_id
        ))
    })??;
    Ok(())
}

fn parse_response_body<R>(body: Value, adapter_id: &str) -> Result<R::Response, DapClientError>
where
    R: DapRequest,
    R::Response: for<'de> Deserialize<'de>,
{
    // Adapters often omit `body` or send `null` for optional response structs.
    let body = match body {
        Value::Null => Value::Object(serde_json::Map::new()),
        other => other,
    };
    match serde_json::from_value::<R::Response>(body) {
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

fn strip_null_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, child| !child.is_null());
            for child in map.values_mut() {
                strip_null_fields(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                strip_null_fields(item);
            }
        }
        _ => {}
    }
}

fn launch_arguments(configuration: &DebugConfiguration) -> Value {
    let mut body = json!({
        "name": configuration.name(),
        "noDebug": false,
        // SharpDbg defaults here, but be explicit: Volt has no DAP terminal pane yet
        // (`runInTerminal` is ignored). `externalTerminal` would pop a console window.
        "console": "internalConsole",
    });
    if let Some(program) = configuration.target_program() {
        // SharpDbg.Cli (dotnet tool) requires `program` (DLL/EXE). VS Code's
        // projectPath helper is extension-side only and is ignored here.
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

fn configure_adapter_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn spawn_adapter_command(adapter: &DebugAdapterSpec) -> Result<Child, DapClientError> {
    let mut command = Command::new(adapter.program());
    configure_adapter_command(&mut command);
    command
        .args(adapter.args())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut env = Vec::new();
    editor_tool_install::merge_effective_path(&mut env);
    for (key, value) in env {
        command.env(key, value);
    }
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
                                        state.selected_thread_id = None;
                                        state.selected_frame_id = None;
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
                                "exited" | "terminated" => {
                                    mark_session_ended(&stop_state, &events, workspace_id);
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

fn mark_session_ended(
    stop_state: &Mutex<SessionStopState>,
    events: &Mutex<VecDeque<DapSessionEvent>>,
    workspace_id: u64,
) {
    let first_end = match stop_state.lock() {
        Ok(mut state) => {
            let first = !state.ended;
            state.ended = true;
            state.snapshot = None;
            first
        }
        Err(_) => false,
    };
    if !first_end {
        return;
    }
    if let Ok(mut queue) = events.lock() {
        queue.push_back(DapSessionEvent::Terminated { workspace_id });
    }
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
    state
        .selected_thread_id
        .or(state.last_thread_id)
        .ok_or_else(|| {
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

fn evaluate_expression(
    handle: &DapSessionHandle,
    expression: &str,
    frame_id: Option<u64>,
    context: EvaluateArgumentsContext,
) -> Result<DapWatchExpression, DapClientError> {
    match handle.request::<Evaluate>(EvaluateArguments {
        expression: expression.to_owned(),
        frame_id,
        context: Some(context),
        format: None,
    }) {
        Ok(response) => Ok(DapWatchExpression {
            expression: expression.to_owned(),
            value: response.result,
            type_name: response.type_,
            ok: true,
            variables_reference: response.variables_reference,
            children: Vec::new(),
            expanded: false,
        }),
        Err(error) => Ok(DapWatchExpression {
            expression: expression.to_owned(),
            value: error.to_string(),
            type_name: None,
            ok: false,
            variables_reference: 0,
            children: Vec::new(),
            expanded: false,
        }),
    }
}

fn capture_stopped_snapshot(
    handle: &DapSessionHandle,
    thread_id: u64,
    preferred_frame_id: Option<u64>,
    reason: String,
    watches: &[String],
    expanded: &BTreeSet<DapVariablePath>,
) -> Result<DapStoppedSnapshot, DapClientError> {
    let stack = handle.request::<StackTrace>(StackTraceArguments {
        thread_id,
        start_frame: Some(0),
        levels: Some(32),
        format: None,
    })?;
    let frame = preferred_frame_id
        .and_then(|frame_id| stack.stack_frames.iter().find(|frame| frame.id == frame_id))
        .or_else(|| stack.stack_frames.first());
    let frame_id = frame.map(|frame| frame.id);
    let position = frame.and_then(|frame| {
        let path = frame
            .source
            .as_ref()
            .and_then(|source| source.path.as_ref())
            .map(PathBuf::from)?;
        let line = u32::try_from(frame.line).ok().filter(|line| *line > 0)?;
        let column = u32::try_from(frame.column).unwrap_or(1).max(1);
        Some(DapExecutionPosition { path, line, column })
    });

    let mut locals = match frame {
        Some(frame) => match load_locals(handle, frame.id) {
            Ok(locals) => locals,
            Err(error) => {
                record_transport_event(
                    &handle.transport_log,
                    handle.info.adapter_id(),
                    format!("failed to load Locals: {error}"),
                );
                Vec::new()
            }
        },
        None => Vec::new(),
    };
    apply_expanded_paths(handle, &mut locals, None, &[], expanded);

    let mut watch_rows = Vec::with_capacity(watches.len());
    for expression in watches {
        watch_rows.push(evaluate_expression(
            handle,
            expression,
            frame_id,
            EvaluateArgumentsContext::Watch,
        )?);
    }
    apply_expanded_watch_roots(handle, &mut watch_rows, expanded);

    Ok(DapStoppedSnapshot {
        thread_id,
        frame_id,
        reason,
        position,
        locals,
        watches: watch_rows,
    })
}

fn load_locals(
    handle: &DapSessionHandle,
    frame_id: u64,
) -> Result<Vec<DapLocalVariable>, DapClientError> {
    let scopes = handle.request::<Scopes>(ScopesArguments { frame_id })?;
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
    let Some(scope) = locals_scope else {
        return Ok(Vec::new());
    };
    if scope.variables_reference == 0 {
        return Ok(Vec::new());
    }
    load_variable_children(handle, scope.variables_reference)
}

fn variable_node_from_dap(variable: dap_types::Variable) -> DapVariableNode {
    DapVariableNode {
        name: variable.name,
        value: variable.value,
        type_name: variable.type_,
        variables_reference: variable.variables_reference,
        children: Vec::new(),
        expanded: false,
    }
}

fn load_variable_children(
    handle: &DapSessionHandle,
    variables_reference: u64,
) -> Result<Vec<DapVariableNode>, DapClientError> {
    if variables_reference == 0 {
        return Ok(Vec::new());
    }
    let response = handle.request::<Variables>(VariablesArguments {
        variables_reference,
        filter: None,
        start: None,
        count: None,
        format: None,
    })?;
    Ok(response
        .variables
        .into_iter()
        .map(variable_node_from_dap)
        .collect())
}

fn flatten_variable_nodes(
    nodes: &[DapVariableNode],
    watch: Option<&str>,
    prefix: &[String],
    depth: usize,
    rows: &mut Vec<DapVariableRow>,
) {
    for node in nodes {
        let mut segments = prefix.to_vec();
        segments.push(node.name.clone());
        let path = match watch {
            Some(expression) => DapVariablePath::watch(expression, segments.clone()),
            None => DapVariablePath::locals(segments.clone()),
        };
        rows.push(DapVariableRow {
            path,
            depth,
            expandable: node.expandable(),
            expanded: node.expanded,
            name: node.name.clone(),
            value: node.value.clone(),
            type_name: node.type_name.clone(),
            ok: true,
        });
        if node.expanded {
            flatten_variable_nodes(&node.children, watch, &segments, depth + 1, rows);
        }
    }
}

fn apply_expanded_paths(
    handle: &DapSessionHandle,
    nodes: &mut [DapVariableNode],
    watch: Option<&str>,
    prefix: &[String],
    expanded: &BTreeSet<DapVariablePath>,
) {
    for node in nodes {
        let mut segments = prefix.to_vec();
        segments.push(node.name.clone());
        let path = match watch {
            Some(expression) => DapVariablePath::watch(expression, segments.clone()),
            None => DapVariablePath::locals(segments.clone()),
        };
        if expanded.contains(&path) && node.variables_reference > 0 {
            match load_variable_children(handle, node.variables_reference) {
                Ok(children) => {
                    node.children = children;
                    node.expanded = true;
                    apply_expanded_paths(handle, &mut node.children, watch, &segments, expanded);
                }
                Err(error) => {
                    record_transport_event(
                        &handle.transport_log,
                        handle.info.adapter_id(),
                        format!("failed to expand `{}`: {error}", node.name),
                    );
                }
            }
        }
    }
}

fn apply_expanded_watch_roots(
    handle: &DapSessionHandle,
    watches: &mut [DapWatchExpression],
    expanded: &BTreeSet<DapVariablePath>,
) {
    for watch in watches {
        let path = DapVariablePath::watch(&watch.expression, Vec::<String>::new());
        if expanded.contains(&path) && watch.expandable() {
            match load_variable_children(handle, watch.variables_reference) {
                Ok(children) => {
                    watch.children = children;
                    watch.expanded = true;
                    apply_expanded_paths(
                        handle,
                        &mut watch.children,
                        Some(watch.expression.as_str()),
                        &[],
                        expanded,
                    );
                }
                Err(error) => {
                    record_transport_event(
                        &handle.transport_log,
                        handle.info.adapter_id(),
                        format!(
                            "failed to expand Watch Expression `{}`: {error}",
                            watch.expression
                        ),
                    );
                }
            }
        }
    }
}

fn find_variable_node_mut<'a>(
    nodes: &'a mut [DapVariableNode],
    segments: &[String],
) -> Option<&'a mut DapVariableNode> {
    let (head, tail) = segments.split_first()?;
    let index = nodes.iter().position(|node| node.name == *head)?;
    let node = nodes.get_mut(index)?;
    if tail.is_empty() {
        Some(node)
    } else {
        find_variable_node_mut(&mut node.children, tail)
    }
}

fn expand_variable_node(
    handle: &DapSessionHandle,
    node: &mut DapVariableNode,
) -> Result<bool, DapClientError> {
    if node.variables_reference == 0 {
        return Ok(false);
    }
    node.children = load_variable_children(handle, node.variables_reference)?;
    node.expanded = true;
    Ok(true)
}

fn expand_watch_root(
    handle: &DapSessionHandle,
    watch: &mut DapWatchExpression,
) -> Result<bool, DapClientError> {
    if !watch.expandable() {
        return Ok(false);
    }
    watch.children = load_variable_children(handle, watch.variables_reference)?;
    watch.expanded = true;
    Ok(true)
}

fn expand_variable_path(
    handle: &DapSessionHandle,
    snapshot: &mut DapStoppedSnapshot,
    path: &DapVariablePath,
) -> Result<bool, DapClientError> {
    match path.watch_expression() {
        Some(expression) => {
            let watch = snapshot
                .watches
                .iter_mut()
                .find(|watch| watch.expression == expression)
                .ok_or_else(|| {
                    DapClientError::Protocol(format!("Watch Expression `{expression}` not found"))
                })?;
            if path.segments.is_empty() {
                expand_watch_root(handle, watch)
            } else {
                let node = find_variable_node_mut(&mut watch.children, &path.segments).ok_or_else(
                    || DapClientError::Protocol("variable path not found".to_owned()),
                )?;
                expand_variable_node(handle, node)
            }
        }
        None => {
            let node = find_variable_node_mut(&mut snapshot.locals, &path.segments)
                .ok_or_else(|| DapClientError::Protocol("variable path not found".to_owned()))?;
            expand_variable_node(handle, node)
        }
    }
}

fn collapse_variable_path(snapshot: &mut DapStoppedSnapshot, path: &DapVariablePath) {
    match path.watch_expression() {
        Some(expression) => {
            let Some(watch) = snapshot
                .watches
                .iter_mut()
                .find(|watch| watch.expression == expression)
            else {
                return;
            };
            if path.segments.is_empty() {
                watch.children.clear();
                watch.expanded = false;
                return;
            }
            if let Some(node) = find_variable_node_mut(&mut watch.children, &path.segments) {
                node.children.clear();
                node.expanded = false;
            }
        }
        None => {
            if let Some(node) = find_variable_node_mut(&mut snapshot.locals, &path.segments) {
                node.children.clear();
                node.expanded = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        io::{Read, Write, pipe},
        net::TcpListener,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, AtomicU64, Ordering},
        },
        thread,
        time::{Duration, Instant},
    };

    use dap_types::DisconnectArguments;
    use serde_json::{Value, json};

    use super::{
        DapClientError, DapClientManager, DapLogDirection, DapSessionEvent, DapVariablePath,
        read_frame,
    };
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

    fn json_value_contains_null(value: &Value) -> bool {
        match value {
            Value::Null => true,
            Value::Array(items) => items.iter().any(json_value_contains_null),
            Value::Object(map) => map.values().any(json_value_contains_null),
            _ => false,
        }
    }

    fn fake_variables_for_reference(reference: u64, running: bool) -> Vec<Value> {
        match reference {
            2 => vec![
                json!({
                    "name": "Name",
                    "value": "\"Ada\"",
                    "type": "string",
                    "variablesReference": 0
                }),
                json!({
                    "name": "Address",
                    "value": "Address { ... }",
                    "type": "Address",
                    "variablesReference": 3
                }),
            ],
            3 => vec![json!({
                "name": "City",
                "value": "\"London\"",
                "type": "string",
                "variablesReference": 0
            })],
            _ => vec![
                json!({
                    "name": "x",
                    "value": "42",
                    "type": "i32",
                    "variablesReference": 0
                }),
                json!({
                    "name": "running",
                    "value": if running { "true" } else { "false" },
                    "type": "bool",
                    "variablesReference": 0
                }),
                json!({
                    "name": "person",
                    "value": "Person { ... }",
                    "type": "Person",
                    "variablesReference": 2
                }),
            ],
        }
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
            if matches!(command.as_str(), "continue" | "next" | "stepIn" | "stepOut")
                && message
                    .get("arguments")
                    .is_some_and(json_value_contains_null)
            {
                let response = json!({
                    "seq": seq,
                    "type": "response",
                    "request_seq": request_seq,
                    "success": false,
                    "command": command,
                    "message": "null optional fields are not allowed"
                });
                seq += 1;
                write_frame_to(&mut writer, &response);
                continue;
            }
            match command.as_str() {
                "initialize" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "initialize",
                        "body": {
                            "supportsConfigurationDoneRequest": true,
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
                "configurationDone" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "configurationDone",
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
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
                    // Omit `body` like adapters that send success with no ContinueResponse fields.
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "continue"
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = true;
                    if program_path.contains("exit-on-continue") {
                        let exited = json!({
                            "seq": seq,
                            "type": "event",
                            "event": "exited",
                            "body": { "exitCode": 0 }
                        });
                        seq += 1;
                        write_frame_to(&mut writer, &exited);
                        let terminated = json!({
                            "seq": seq,
                            "type": "event",
                            "event": "terminated",
                            "body": {}
                        });
                        write_frame_to(&mut writer, &terminated);
                        break;
                    }
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
                            }, {
                                "id": 2,
                                "name": "caller",
                                "source": {
                                    "name": "main.rs",
                                    "path": program_path
                                },
                                "line": stopped_line.saturating_add(10),
                                "column": 1
                            }],
                            "totalFrames": 2
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "threads" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "threads",
                        "body": {
                            "threads": [
                                { "id": 1, "name": "main" },
                                { "id": 2, "name": "worker" }
                            ]
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "evaluate" => {
                    let expression = message
                        .pointer("/arguments/expression")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    let frame_id = message
                        .pointer("/arguments/frameId")
                        .and_then(Value::as_u64)
                        .unwrap_or(1);
                    if expression == "fail" {
                        let response = json!({
                            "seq": seq,
                            "type": "response",
                            "request_seq": request_seq,
                            "success": false,
                            "command": "evaluate",
                            "message": "cannot evaluate"
                        });
                        seq += 1;
                        write_frame_to(&mut writer, &response);
                        continue;
                    }
                    let (result, type_name, variables_reference) = if expression == "person" {
                        ("Person { ... }".to_owned(), Some("Person"), 2_u64)
                    } else {
                        (
                            format!("{expression}@{frame_id}={stopped_line}"),
                            None,
                            0_u64,
                        )
                    };
                    let mut body = json!({
                        "result": result,
                        "variablesReference": variables_reference
                    });
                    if let Some(type_name) = type_name
                        && let Some(object) = body.as_object_mut()
                    {
                        object.insert("type".to_owned(), Value::String(type_name.to_owned()));
                    }
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "evaluate",
                        "body": body
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
                    let reference = message
                        .pointer("/arguments/variablesReference")
                        .and_then(Value::as_u64)
                        .unwrap_or(1);
                    let variables = fake_variables_for_reference(reference, running);
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "variables",
                        "body": { "variables": variables }
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
        assert!(
            log.entries().iter().any(|entry| {
                entry.direction() == DapLogDirection::Send
                    && entry.message().contains("configurationDone")
            }),
            "SharpDbg-style adapters require configurationDone after setBreakpoints"
        );
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

    fn wait_for_terminated(manager: &DapClientManager, workspace_id: u64) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            let events = manager.drain_events().expect("drain");
            if events.iter().any(|event| {
                matches!(
                    event,
                    DapSessionEvent::Terminated {
                        workspace_id: id
                    } if *id == workspace_id
                )
            }) {
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!("timed out waiting for terminated event");
    }

    fn assert_control_requests_omit_nulls(manager: &DapClientManager) {
        let sends: Vec<String> = manager
            .log_snapshot()
            .entries()
            .iter()
            .filter(|entry| entry.direction() == DapLogDirection::Send)
            .map(|entry| entry.message().to_owned())
            .filter(|message| {
                message.contains("\"command\":\"continue\"")
                    || message.contains("\"command\":\"next\"")
                    || message.contains("\"command\":\"stepIn\"")
                    || message.contains("\"command\":\"stepOut\"")
            })
            .collect();
        assert!(
            !sends.is_empty(),
            "expected continue/step requests in the DAP send log"
        );
        for message in &sends {
            assert!(
                !message.contains(":null"),
                "control request must omit null optionals: {message}"
            );
        }
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
        assert_control_requests_omit_nulls(&manager);

        manager.stop_session(11).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn expand_collapse_and_reapply_nested_locals_and_watches() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                15,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("expand", DebugRequestKind::Launch)
                    .with_target_program("src/main.rs"),
            )
            .expect("start");

        wait_for_stopped(&manager, 15);
        let snapshot = manager.refresh_stopped_snapshot(15).expect("snapshot");
        let person = snapshot
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person");
        assert!(person.expandable());
        assert!(!person.expanded());
        assert!(person.children().is_empty());

        let expanded = manager
            .toggle_variable_expand(15, &DapVariablePath::locals(["person"]))
            .expect("expand person");
        let person = expanded
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person");
        assert!(person.expanded());
        assert_eq!(person.children().len(), 2);
        assert_eq!(person.children()[0].name(), "Name");
        assert_eq!(person.children()[0].value(), "\"Ada\"");
        assert_eq!(person.children()[1].name(), "Address");
        assert!(person.children()[1].expandable());

        let nested = manager
            .toggle_variable_expand(15, &DapVariablePath::locals(["person", "Address"]))
            .expect("expand address");
        let address = nested
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person")
            .children()
            .iter()
            .find(|child| child.name() == "Address")
            .expect("address");
        assert_eq!(address.children().len(), 1);
        assert_eq!(address.children()[0].name(), "City");

        let collapsed = manager
            .toggle_variable_expand(15, &DapVariablePath::locals(["person"]))
            .expect("collapse person");
        let person = collapsed
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person");
        assert!(!person.expanded());
        assert!(person.children().is_empty());

        manager
            .toggle_variable_expand(15, &DapVariablePath::locals(["person"]))
            .expect("re-expand");
        manager.step_over(15).expect("step");
        wait_for_stopped(&manager, 15);
        let stepped = manager.refresh_stopped_snapshot(15).expect("stepped");
        let person = stepped
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person");
        assert!(
            person.expanded() && person.children().iter().any(|child| child.name() == "Name"),
            "expand path must survive a step: {:?}",
            person.children()
        );

        manager.add_expression(15, "person").expect("watch");
        let with_watch = manager.refresh_stopped_snapshot(15).expect("watch snap");
        assert!(
            with_watch
                .watches()
                .iter()
                .any(|watch| watch.expression() == "person" && watch.expandable()),
            "person watch should be expandable: {:?}",
            with_watch.watches()
        );
        let watch_expanded = manager
            .toggle_variable_expand(15, &DapVariablePath::watch("person", Vec::<String>::new()))
            .expect("expand watch");
        let watch = watch_expanded
            .watches()
            .iter()
            .find(|watch| watch.expression() == "person")
            .expect("watch");
        assert!(watch.expanded());
        assert!(watch.children().iter().any(|child| child.name() == "Name"));

        manager.stop_session(15).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn continue_to_process_exit_queues_terminated() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                14,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("exit", DebugRequestKind::Launch)
                    .with_target_program("exit-on-continue.rs"),
            )
            .expect("start");

        wait_for_stopped(&manager, 14);
        manager.continue_session(14).expect("continue");
        wait_for_terminated(&manager, 14);
        manager.stop_session(14).expect("stop");
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
    fn watches_eval_switch_context_and_breakpoint_extras_against_fake_adapter() {
        use super::DapEvaluateContext;

        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                13,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("polish", DebugRequestKind::Launch)
                    .with_target_program("src/main.rs"),
            )
            .expect("start");
        wait_for_stopped(&manager, 13);

        manager
            .set_breakpoint_extras(
                13,
                "src/main.rs",
                11,
                Some(Some("x > 0".to_owned())),
                Some(Some("3".to_owned())),
                Some(Some("log {x}".to_owned())),
            )
            .expect("bp extras");
        let bp = manager
            .list_breakpoints(13)
            .expect("list")
            .into_iter()
            .find(|bp| bp.line() == 11)
            .expect("bp");
        assert_eq!(bp.condition(), Some("x > 0"));
        assert_eq!(bp.hit_condition(), Some("3"));
        assert_eq!(bp.log_message(), Some("log {x}"));

        manager.add_expression(13, "x").expect("add watch");
        let snapshot = manager.refresh_stopped_snapshot(13).expect("snap");
        assert!(
            snapshot
                .watches()
                .iter()
                .any(|watch| watch.expression() == "x" && watch.ok()),
            "watch should evaluate: {:?}",
            snapshot.watches()
        );

        let eval = manager
            .evaluate(13, "y", DapEvaluateContext::Repl)
            .expect("eval");
        assert!(eval.ok());
        assert!(eval.value().contains("y@"));

        let threads = manager.list_threads(13).expect("threads");
        assert_eq!(threads.len(), 2);
        manager.switch_thread(13, 2).expect("switch thread");
        assert_eq!(
            manager
                .stopped_snapshot(13)
                .expect("snap")
                .expect("present")
                .thread_id(),
            2
        );

        let frames = manager.list_stack_frames(13).expect("frames");
        assert!(frames.len() >= 2);
        let switched = manager.switch_stack_frame(13, 2).expect("switch frame");
        assert_eq!(switched.frame_id(), Some(2));
        assert_eq!(switched.position().expect("pos").line(), 20);

        assert!(manager.remove_expression(13, "x").expect("remove"));
        assert!(manager.list_expressions(13).expect("list").is_empty());

        manager.stop_session(13).expect("stop");
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

    #[test]
    fn launch_arguments_always_send_program_path() {
        let config = DebugConfiguration::new("Debug (dotnet)", DebugRequestKind::Launch)
            .with_target_program("bin/Debug/net10.0/App.dll")
            .with_cwd(".");
        let body = super::launch_arguments(&config);
        assert_eq!(body["program"], "bin/Debug/net10.0/App.dll");
        assert_eq!(body["console"], "internalConsole");
        assert!(body.get("projectPath").is_none());
    }

    const STRUCT_CTOR_PROGRAM: &str = r#"Console.WriteLine("Hello, World!");
var a = 1;
var b = new foo();
Console.WriteLine(a);

public struct foo{
public string bar => "bar";
}
"#;

    fn sharpdbg_spec() -> Option<DebugAdapterSpec> {
        Command::new("sharpdbg")
            .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()
            .filter(|status| status.success())
            .map(|_| {
                DebugAdapterSpec::new(
                    "sharpdbg",
                    "csharp",
                    ["cs"],
                    "sharpdbg",
                    ["--interpreter=vscode"],
                )
            })
    }

    fn wait_for_stopped_or_terminated(
        manager: &DapClientManager,
        workspace_id: u64,
        timeout: Duration,
    ) -> DapSessionEvent {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let events = manager.drain_events().expect("drain");
            if let Some(event) = events.into_iter().find(|event| {
                matches!(
                    event,
                    DapSessionEvent::Stopped {
                        workspace_id: id,
                    } if *id == workspace_id
                ) || matches!(
                    event,
                    DapSessionEvent::Terminated {
                        workspace_id: id,
                    } if *id == workspace_id
                )
            }) {
                return event;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!(
            "timed out waiting for stopped/terminated. log:\n{}",
            dap_log_text(manager)
        );
    }

    fn dap_log_text(manager: &DapClientManager) -> String {
        manager
            .log_snapshot()
            .entries()
            .iter()
            .map(|entry| format!("{:?} {}", entry.direction(), entry.message()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn find_named_dll(root: &Path, file_name: &str) -> Option<PathBuf> {
        let mut found = None;
        fn walk(dir: &Path, file_name: &str, found: &mut Option<PathBuf>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, file_name, found);
                } else if path.file_name().is_some_and(|name| name == file_name) {
                    *found = Some(path);
                }
            }
        }
        walk(&root.join("bin").join("Debug"), file_name, &mut found);
        found
    }

    fn build_csharp_fixture(source: &str) -> (PathBuf, PathBuf, PathBuf) {
        static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!("volt-sharpdbg-step-{}-{seq}", std::process::id()));
        fs::create_dir_all(&root).expect("temp project");
        fs::write(
            root.join("StepStruct.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <DebugType>portable</DebugType>
    <Optimize>false</Optimize>
  </PropertyGroup>
</Project>
"#,
        )
        .expect("csproj");
        let program = root.join("Program.cs");
        fs::write(&program, source).expect("program");
        static DOTNET_BUILD: Mutex<()> = Mutex::new(());
        let _build_lock = DOTNET_BUILD
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let build = Command::new("dotnet")
            .args(["build", "-c", "Debug", "--nologo"])
            .current_dir(&root)
            .output()
            .expect("dotnet build");
        assert!(
            build.status.success(),
            "dotnet build failed\nstdout:{}\nstderr:{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let dll = find_named_dll(&root, "StepStruct.dll").expect("built dll");
        (root, program, dll)
    }

    fn start_struct_ctor_session(
        workspace_id: u64,
        source: &str,
    ) -> Option<(DapClientManager, PathBuf)> {
        let spec = sharpdbg_spec()?;
        let (root, program, dll) = build_csharp_fixture(source);
        let mut registry = DebugAdapterRegistry::new();
        registry.register(spec).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .toggle_breakpoint(workspace_id, program.as_path(), 1)
            .expect("bp");
        manager
            .start(
                workspace_id,
                Some("sharpdbg"),
                None,
                DebugConfiguration::new("Debug (dotnet)", DebugRequestKind::Launch)
                    .with_target_program(&dll)
                    .with_cwd(&root),
            )
            .expect("start");
        let first = wait_for_stopped_or_terminated(&manager, workspace_id, Duration::from_secs(20));
        assert!(
            matches!(first, DapSessionEvent::Stopped { .. }),
            "expected first stop, got {first:?}\n{}",
            dap_log_text(&manager)
        );
        Some((manager, root))
    }

    fn snapshot_line(manager: &DapClientManager, workspace_id: u64) -> Option<u32> {
        manager
            .refresh_stopped_snapshot(workspace_id)
            .ok()
            .and_then(|snapshot| snapshot.position().map(super::DapExecutionPosition::line))
    }

    fn step_over_until_line(
        manager: &DapClientManager,
        workspace_id: u64,
        target: u32,
        budget: usize,
    ) -> Vec<Option<u32>> {
        let mut lines = Vec::new();
        for step in 0..budget {
            let line = snapshot_line(manager, workspace_id);
            lines.push(line);
            if line == Some(target) {
                return lines;
            }
            manager.step_over(workspace_id).expect("step");
            let event =
                wait_for_stopped_or_terminated(manager, workspace_id, Duration::from_secs(20));
            assert!(
                matches!(event, DapSessionEvent::Stopped { .. }),
                "Session ended after {step} steps, lines={lines:?}, event={event:?}\n{}",
                dap_log_text(manager)
            );
        }
        lines
    }

    #[test]
    fn sharpdbg_step_over_struct_construction_keeps_session() {
        let Some((manager, root)) = start_struct_ctor_session(91, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let to_ctor = step_over_until_line(&manager, 91, 3, 6);
        assert_eq!(
            to_ctor.last().copied().flatten(),
            Some(3),
            "never reached `var b`; lines={to_ctor:?}\n{}",
            dap_log_text(&manager)
        );
        manager.step_over(91).expect("step over ctor");
        let event = wait_for_stopped_or_terminated(&manager, 91, Duration::from_secs(20));
        assert!(
            matches!(event, DapSessionEvent::Stopped { .. }),
            "Session ended stepping over `new foo()`, lines={to_ctor:?}, event={event:?}\n{}",
            dap_log_text(&manager)
        );
        assert_eq!(
            snapshot_line(&manager, 91),
            Some(4),
            "expected Console.WriteLine(a) after ctor\n{}",
            dap_log_text(&manager)
        );
        manager.stop_session(91).expect("stop");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sharpdbg_step_into_struct_construction_keeps_session() {
        let Some((manager, root)) = start_struct_ctor_session(92, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let to_ctor = step_over_until_line(&manager, 92, 3, 8);
        assert_eq!(
            to_ctor.last().copied().flatten(),
            Some(3),
            "never reached `var b`; lines={to_ctor:?}\n{}",
            dap_log_text(&manager)
        );
        manager.step_into(92).expect("step into ctor");
        let event = wait_for_stopped_or_terminated(&manager, 92, Duration::from_secs(20));
        assert!(
            matches!(event, DapSessionEvent::Stopped { .. }),
            "Session ended on step-into `new foo()`, lines={to_ctor:?}, event={event:?}\n{}",
            dap_log_text(&manager)
        );
        assert!(
            snapshot_line(&manager, 92).is_some(),
            "lost execution position after step-into `new foo()`\n{}",
            dap_log_text(&manager)
        );
        manager.stop_session(92).expect("stop");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sharpdbg_double_step_over_struct_construction_keeps_session() {
        let Some((manager, root)) = start_struct_ctor_session(93, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let to_ctor = step_over_until_line(&manager, 93, 3, 6);
        assert_eq!(to_ctor.last().copied().flatten(), Some(3));
        manager.step_over(93).expect("first next");
        manager.step_over(93).expect("second next while running");
        let event = wait_for_stopped_or_terminated(&manager, 93, Duration::from_secs(20));
        assert!(
            matches!(event, DapSessionEvent::Stopped { .. }),
            "Session ended after stacked `next` on `new foo()`, lines={to_ctor:?}, event={event:?}\n{}",
            dap_log_text(&manager)
        );
        manager.stop_session(93).expect("stop");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sharpdbg_step_into_from_entry_keeps_session_through_struct_ctor() {
        let Some((manager, root)) = start_struct_ctor_session(94, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let mut lines = Vec::new();
        for step in 0..12 {
            let line = snapshot_line(&manager, 94);
            lines.push(line);
            if line == Some(4) {
                manager.stop_session(94).expect("stop");
                let _ = fs::remove_dir_all(&root);
                return;
            }
            manager.step_into(94).expect("step into");
            let event = wait_for_stopped_or_terminated(&manager, 94, Duration::from_secs(20));
            assert!(
                matches!(event, DapSessionEvent::Stopped { .. }),
                "Session ended on F11-from-entry after {step} steps, lines={lines:?}, event={event:?}\n{}",
                dap_log_text(&manager)
            );
        }
        manager.stop_session(94).expect("stop");
        let _ = fs::remove_dir_all(&root);
        panic!(
            "never reached Console.WriteLine(a) with F11-only; lines={lines:?}\n{}",
            dap_log_text(&manager)
        );
    }

    #[test]
    fn sharpdbg_expand_struct_local_keeps_session() {
        let Some((manager, root)) = start_struct_ctor_session(95, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let to_after = step_over_until_line(&manager, 95, 4, 8);
        assert_eq!(
            to_after.last().copied().flatten(),
            Some(4),
            "never reached line after ctor; lines={to_after:?}\n{}",
            dap_log_text(&manager)
        );
        let snapshot = manager.refresh_stopped_snapshot(95).expect("snapshot");
        assert!(
            snapshot
                .locals()
                .iter()
                .any(|local| local.name() == "b" && local.expandable()),
            "expected expandable local `b`: {:?}\n{}",
            snapshot
                .locals()
                .iter()
                .map(|local| (local.name(), local.value(), local.expandable()))
                .collect::<Vec<_>>(),
            dap_log_text(&manager)
        );
        let expanded = manager
            .toggle_variable_expand(95, &super::DapVariablePath::locals(["b"]))
            .expect("expand b");
        let b = expanded
            .locals()
            .iter()
            .find(|local| local.name() == "b")
            .expect("b");
        assert!(
            b.children().iter().any(|child| child.name() == "bar"),
            "expected property `bar` under `b`: {:?}\n{}",
            b.children()
                .iter()
                .map(|child| (child.name(), child.value()))
                .collect::<Vec<_>>(),
            dap_log_text(&manager)
        );
        let drained = manager.drain_events().expect("drain");
        assert!(
            !drained
                .iter()
                .any(|event| matches!(event, DapSessionEvent::Terminated { .. })),
            "Session ended expanding `b.bar`: {drained:?}\n{}",
            dap_log_text(&manager)
        );
        assert!(
            manager.stopped_snapshot(95).ok().flatten().is_some(),
            "lost stopped snapshot expanding `b`\n{}",
            dap_log_text(&manager)
        );
        manager.stop_session(95).expect("stop");
        let _ = fs::remove_dir_all(&root);
    }
}
