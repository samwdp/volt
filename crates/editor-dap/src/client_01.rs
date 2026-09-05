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
            reader,
            DapReaderSession {
                workspace_id,
                adapter_id: adapter.id().to_owned(),
                pending: Arc::clone(&pending),
                disconnected: Arc::clone(&disconnected),
                transport_log: Arc::clone(&self.transport_log),
                initialized: Arc::clone(&initialized),
                stop_state: Arc::clone(&stop_state),
                events: Arc::clone(&self.events),
            },
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
