use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, SystemTime},
};

use editor_buffer::{TextPoint, TextRange};
use lsp_types::{
    ClientCapabilities, ClientInfo, CompletionParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams, HoverParams,
    InitializeParams, InitializedParams, PartialResultParams, Position,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TraceValue, Uri, VersionedTextDocumentIdentifier,
    WorkDoneProgressParams, WorkspaceFolder,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Initialized, Notification,
    },
    request::{Completion, HoverRequest, Initialize, Request},
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerSession, LspError,
};

const REQUEST_TIMEOUT: Duration = Duration::from_millis(400);
const TRANSPORT_LOG_MAX_ENTRIES: usize = 400;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

type PendingResponseTx = Sender<Result<Value, LspClientError>>;
type PendingResponseMap = Arc<Mutex<BTreeMap<u64, PendingResponseTx>>>;
type DiagnosticsByPath = Arc<Mutex<BTreeMap<PathBuf, Vec<Diagnostic>>>>;
type TransportLog = Arc<Mutex<LspTransportLog>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspCompletionItem {
    server_id: String,
    kind: Option<LspCompletionKind>,
    label: String,
    insert_text: String,
    detail: Option<String>,
    documentation: Option<String>,
}

impl LspCompletionItem {
    fn new(
        server_id: impl Into<String>,
        kind: Option<LspCompletionKind>,
        label: impl Into<String>,
        insert_text: impl Into<String>,
        detail: Option<String>,
        documentation: Option<String>,
    ) -> Self {
        Self {
            server_id: server_id.into(),
            kind,
            label: label.into(),
            insert_text: insert_text.into(),
            detail,
            documentation,
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub const fn kind(&self) -> Option<LspCompletionKind> {
        self.kind
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn insert_text(&self) -> &str {
        &self.insert_text
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn documentation(&self) -> Option<&str> {
        self.documentation.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspCompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspHoverContents {
    server_id: String,
    lines: Vec<String>,
}

impl LspHoverContents {
    fn new(server_id: impl Into<String>, lines: Vec<String>) -> Self {
        Self {
            server_id: server_id.into(),
            lines,
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspLogDirection {
    Incoming,
    Outgoing,
    Event,
}

impl LspLogDirection {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Incoming => "IN",
            Self::Outgoing => "OUT",
            Self::Event => "EVENT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspLogEntry {
    timestamp: SystemTime,
    server_id: String,
    direction: LspLogDirection,
    body: String,
}

impl LspLogEntry {
    pub fn new(
        direction: LspLogDirection,
        server_id: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: SystemTime::now(),
            server_id: server_id.into(),
            direction,
            body: body.into(),
        }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub const fn direction(&self) -> LspLogDirection {
        self.direction
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LspLogSnapshot {
    revision: u64,
    entries: Vec<LspLogEntry>,
}

impl LspLogSnapshot {
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn entries(&self) -> &[LspLogEntry] {
        &self.entries
    }
}

#[derive(Debug)]
struct LspTransportLog {
    revision: u64,
    entries: Vec<LspLogEntry>,
    max_entries: usize,
}

impl LspTransportLog {
    fn new(max_entries: usize) -> Self {
        Self {
            revision: 0,
            entries: Vec::new(),
            max_entries,
        }
    }

    fn record(&mut self, entry: LspLogEntry) {
        self.revision = self.revision.saturating_add(1);
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            let overflow = self.entries.len() - self.max_entries;
            self.entries.drain(0..overflow);
        }
    }

    fn snapshot(&self) -> LspLogSnapshot {
        LspLogSnapshot {
            revision: self.revision,
            entries: self.entries.clone(),
        }
    }
}

#[derive(Debug)]
pub enum LspClientError {
    Registry(LspError),
    Io(std::io::Error),
    Protocol(String),
    Timeout(String),
    Disconnected(String),
}

impl std::fmt::Display for LspClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registry(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
            Self::Protocol(message) => formatter.write_str(message),
            Self::Timeout(method) => write!(
                formatter,
                "timed out waiting for LSP response to `{method}`"
            ),
            Self::Disconnected(server_id) => {
                write!(formatter, "language server `{server_id}` disconnected")
            }
        }
    }
}

impl std::error::Error for LspClientError {}

impl From<LspError> for LspClientError {
    fn from(error: LspError) -> Self {
        Self::Registry(error)
    }
}

impl From<std::io::Error> for LspClientError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone)]
pub struct LspClientManager {
    registry: LanguageServerRegistry,
    state: Arc<Mutex<LspClientState>>,
    transport_log: TransportLog,
}

#[derive(Debug, Default)]
struct LspClientState {
    sessions: BTreeMap<SessionKey, Arc<LspSessionHandle>>,
    tracked_buffers: BTreeMap<PathBuf, TrackedBufferState>,
    start_failures: BTreeMap<SessionKey, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SessionKey {
    server_id: String,
    root: Option<PathBuf>,
}

#[derive(Debug, Default, Clone)]
struct TrackedBufferState {
    revision: u64,
    version: i32,
    sessions: BTreeSet<SessionKey>,
}

struct LspSessionHandle {
    key: SessionKey,
    session: LanguageServerSession,
    child: Mutex<Child>,
    writer: Arc<Mutex<ChildStdin>>,
    pending: PendingResponseMap,
    diagnostics: DiagnosticsByPath,
    transport_log: TransportLog,
    next_request_id: AtomicU64,
    disconnected: Arc<AtomicBool>,
}

impl std::fmt::Debug for LspSessionHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LspSessionHandle")
            .field("server_id", &self.key.server_id)
            .field("root", &self.key.root)
            .finish_non_exhaustive()
    }
}

impl Drop for LspSessionHandle {
    fn drop(&mut self) {
        record_transport_event(
            &self.transport_log,
            &self.key.server_id,
            "terminating language server process",
        );
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl LspClientManager {
    pub fn new(registry: LanguageServerRegistry) -> Self {
        Self {
            registry,
            state: Arc::new(Mutex::new(LspClientState::default())),
            transport_log: Arc::new(Mutex::new(LspTransportLog::new(TRANSPORT_LOG_MAX_ENTRIES))),
        }
    }

    pub fn log_snapshot(&self) -> LspLogSnapshot {
        self.transport_log
            .lock()
            .map(|log| log.snapshot())
            .unwrap_or_default()
    }

    pub fn needs_sync(&self, path: &Path, revision: u64) -> bool {
        let Ok(state) = self.state.lock() else {
            return false;
        };
        if let Some(tracked) = state.tracked_buffers.get(path) {
            let has_live_session = tracked.sessions.iter().any(|key| {
                state
                    .sessions
                    .get(key)
                    .map(|session| !session.is_disconnected())
                    .unwrap_or(false)
            });
            if tracked.revision == revision && has_live_session {
                return false;
            }
        }
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return false;
        };
        let servers = self.registry.servers_for_extension(extension);
        if servers.is_empty() {
            return false;
        }
        let failed_server_ids = state
            .start_failures
            .keys()
            .map(|key| key.server_id.as_str())
            .collect::<BTreeSet<_>>();
        servers
            .into_iter()
            .any(|server| !failed_server_ids.contains(server.id()))
    }

    pub fn sync_buffer(
        &self,
        path: &Path,
        text: &str,
        revision: u64,
        root: Option<&Path>,
    ) -> Result<Vec<String>, LspClientError> {
        let sessions = self.ensure_sessions_for_path(path, root, None, false)?;
        self.sync_buffer_to_sessions(path, text, revision, sessions)
    }

    pub fn start_buffer_server(
        &self,
        path: &Path,
        text: &str,
        revision: u64,
        root: Option<&Path>,
        server_id: &str,
    ) -> Result<Vec<String>, LspClientError> {
        let sessions = self.ensure_sessions_for_path(path, root, Some(server_id), true)?;
        self.sync_buffer_to_sessions(path, text, revision, sessions)
    }

    pub fn save_buffer(&self, path: &Path) -> Result<(), LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        for session in sessions {
            session.did_save(path)?;
        }
        Ok(())
    }

    pub fn close_buffer(&self, path: &Path) -> Result<(), LspClientError> {
        let sessions = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            let Some(tracked) = state.tracked_buffers.remove(path) else {
                return Ok(());
            };
            let session_keys = tracked.sessions;
            let sessions = session_keys
                .iter()
                .filter_map(|key| state.sessions.get(key).cloned())
                .collect::<Vec<_>>();
            cleanup_unused_sessions(&mut state, &session_keys);
            sessions
        };
        for session in sessions {
            session.did_close(path)?;
        }
        Ok(())
    }

    pub fn stop_buffer(&self, path: &Path) -> Result<(), LspClientError> {
        self.close_buffer(path)
    }

    pub fn restart_buffer(
        &self,
        path: &Path,
        text: &str,
        revision: u64,
        root: Option<&Path>,
        preferred_server_id: Option<&str>,
    ) -> Result<Vec<String>, LspClientError> {
        self.close_buffer(path)?;
        if let Some(server_id) = preferred_server_id {
            return self.start_buffer_server(path, text, revision, root, server_id);
        }
        self.sync_buffer(path, text, revision, root)
    }

    pub fn supports_path(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| !self.registry.servers_for_extension(extension).is_empty())
            .unwrap_or(false)
    }

    pub fn registered_server_ids_for_path(&self, path: &Path) -> Vec<String> {
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return Vec::new();
        };
        self.registry
            .servers_for_extension(extension)
            .into_iter()
            .map(|server| server.id().to_owned())
            .collect()
    }

    pub fn diagnostics_for_path(&self, path: &Path) -> Vec<Diagnostic> {
        let sessions = self.tracked_sessions_for_path(path).unwrap_or_default();
        let mut diagnostics = Vec::new();
        for session in sessions {
            diagnostics.extend(session.diagnostics_for_path(path));
        }
        diagnostics.sort_by_key(|diagnostic| {
            (
                diagnostic.range().start().line,
                diagnostic.range().start().column,
                diagnostic.severity() as u8,
            )
        });
        diagnostics
    }

    pub fn hover(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspHoverContents>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut results = Vec::new();
        for session in sessions {
            if let Some(hover) = session.hover(path, position)? {
                results.push(hover);
            }
        }
        Ok(results)
    }

    pub fn completions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspCompletionItem>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut items = Vec::new();
        for session in sessions {
            items.extend(session.completions(path, position)?);
        }
        Ok(items)
    }

    pub fn session_labels_for_path(&self, path: &Path) -> Vec<String> {
        let mut labels = self
            .state
            .lock()
            .ok()
            .and_then(|state| state.tracked_buffers.get(path).cloned())
            .map(|tracked| {
                tracked
                    .sessions
                    .into_iter()
                    .map(|key| key.server_id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        labels.sort();
        labels.dedup();
        labels
    }

    fn sync_buffer_to_sessions(
        &self,
        path: &Path,
        text: &str,
        revision: u64,
        sessions: Vec<Arc<LspSessionHandle>>,
    ) -> Result<Vec<String>, LspClientError> {
        if sessions.is_empty() {
            if let Ok(mut state) = self.state.lock() {
                state.tracked_buffers.remove(path);
            }
            return Ok(Vec::new());
        }

        let session_keys = sessions
            .iter()
            .map(|session| session.key.clone())
            .collect::<BTreeSet<_>>();
        let (version, previously_open) = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            let tracked = state
                .tracked_buffers
                .entry(path.to_path_buf())
                .or_insert_with(TrackedBufferState::default);
            let previously_open = tracked.sessions.clone();
            tracked.version = tracked.version.saturating_add(1).max(1);
            tracked.revision = revision;
            tracked.sessions = session_keys;
            (tracked.version, previously_open)
        };

        for session in &sessions {
            if previously_open.contains(&session.key) {
                session.did_change(path, version, text)?;
            } else {
                session.did_open(path, version, text)?;
            }
        }

        let mut labels = sessions
            .iter()
            .map(|session| session.server_id().to_owned())
            .collect::<Vec<_>>();
        labels.sort();
        labels.dedup();
        Ok(labels)
    }

    fn tracked_sessions_for_path(
        &self,
        path: &Path,
    ) -> Result<Vec<Arc<LspSessionHandle>>, LspClientError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
        let Some(tracked) = state.tracked_buffers.get(path).cloned() else {
            return Ok(Vec::new());
        };
        let mut sessions = Vec::new();
        let mut stale_keys = Vec::new();
        for key in tracked.sessions {
            if let Some(session) = state.sessions.get(&key) {
                if session.is_disconnected() {
                    stale_keys.push(key);
                } else {
                    sessions.push(Arc::clone(session));
                }
            }
        }
        for key in stale_keys {
            state.sessions.remove(&key);
        }
        Ok(sessions)
    }

    fn ensure_sessions_for_path(
        &self,
        path: &Path,
        root: Option<&Path>,
        preferred_server_id: Option<&str>,
        force_retry: bool,
    ) -> Result<Vec<Arc<LspSessionHandle>>, LspClientError> {
        let root = root.map(Path::to_path_buf);
        let session_plans = if let Some(server_id) = preferred_server_id {
            vec![self.registry.prepare_session(server_id, root.clone())?]
        } else {
            let extension = path
                .extension()
                .and_then(|extension| extension.to_str())
                .ok_or_else(|| {
                    LspClientError::Protocol(format!(
                        "buffer `{}` does not have a file extension for LSP lookup",
                        path.display()
                    ))
                })?;
            self.registry
                .prepare_sessions_for_extension(extension, root.clone())?
        };

        let mut handles = Vec::new();
        for session in session_plans {
            let key = SessionKey {
                server_id: session.server_id().to_owned(),
                root: session.root().cloned(),
            };
            let existing = {
                let mut state = self
                    .state
                    .lock()
                    .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
                if force_retry {
                    state.start_failures.remove(&key);
                    if state
                        .sessions
                        .get(&key)
                        .map(|session| session.is_disconnected())
                        .unwrap_or(false)
                    {
                        state.sessions.remove(&key);
                    }
                }
                if let Some(session) = state.sessions.get(&key) {
                    Some(Arc::clone(session))
                } else {
                    None
                }
            };
            if let Some(existing) = existing {
                handles.push(existing);
                continue;
            }
            {
                let state = self
                    .state
                    .lock()
                    .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
                if state.start_failures.contains_key(&key) {
                    continue;
                }
            }
            match LspSessionHandle::start(session, Arc::clone(&self.transport_log)) {
                Ok(handle) => {
                    self.state
                        .lock()
                        .map_err(|_| {
                            LspClientError::Protocol("LSP state mutex poisoned".to_owned())
                        })?
                        .sessions
                        .insert(key, Arc::clone(&handle));
                    handles.push(handle);
                }
                Err(error) => {
                    record_transport_event(
                        &self.transport_log,
                        &key.server_id,
                        format!("failed to start language server: {error}"),
                    );
                    self.state
                        .lock()
                        .map_err(|_| {
                            LspClientError::Protocol("LSP state mutex poisoned".to_owned())
                        })?
                        .start_failures
                        .insert(key.clone(), error.to_string());
                    if preferred_server_id.is_some() {
                        return Err(error);
                    }
                }
            }
        }
        Ok(handles)
    }
}

impl LspSessionHandle {
    fn start(
        session: LanguageServerSession,
        transport_log: TransportLog,
    ) -> Result<Arc<Self>, LspClientError> {
        let launch = session.launch();
        let launch_program = launch.program().to_owned();
        let launch_args = launch.args().to_vec();
        let launch_cwd = launch.cwd().cloned();
        let mut command = Command::new(launch.program());
        configure_lsp_command(&mut command);
        command
            .args(launch.args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(cwd) = launch_cwd.as_deref() {
            command.current_dir(cwd);
        }
        for (key, value) in launch.env() {
            command.env(key, value);
        }
        let mut child = command.spawn().map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to start language server `{}`: {error}",
                session.server_id()
            ))
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            LspClientError::Protocol(format!(
                "language server `{}` is missing stdin pipe",
                session.server_id()
            ))
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            LspClientError::Protocol(format!(
                "language server `{}` is missing stdout pipe",
                session.server_id()
            ))
        })?;

        let key = SessionKey {
            server_id: session.server_id().to_owned(),
            root: session.root().cloned(),
        };
        let writer = Arc::new(Mutex::new(stdin));
        let pending = Arc::new(Mutex::new(BTreeMap::new()));
        let diagnostics = Arc::new(Mutex::new(BTreeMap::new()));
        let disconnected = Arc::new(AtomicBool::new(false));
        let pid = child.id();
        let handle = Arc::new(Self {
            key,
            session,
            child: Mutex::new(child),
            writer: Arc::clone(&writer),
            pending: Arc::clone(&pending),
            diagnostics: Arc::clone(&diagnostics),
            transport_log: Arc::clone(&transport_log),
            next_request_id: AtomicU64::new(1),
            disconnected: Arc::clone(&disconnected),
        });
        record_transport_event(
            &transport_log,
            handle.server_id(),
            launch_summary(
                pid,
                &launch_program,
                &launch_args,
                launch_cwd.as_deref(),
                handle.key.root.as_deref(),
            ),
        );
        spawn_reader_thread(
            handle.server_id().to_owned(),
            stdout,
            writer,
            pending,
            diagnostics,
            disconnected,
            transport_log,
        );
        handle.initialize()?;
        Ok(handle)
    }

    fn server_id(&self) -> &str {
        self.session.server_id()
    }

    fn is_disconnected(&self) -> bool {
        self.disconnected.load(Ordering::Acquire)
    }

    fn initialize(&self) -> Result<(), LspClientError> {
        let root_uri = self
            .session
            .root()
            .map(|root| path_to_uri(root.as_path()))
            .transpose()?;
        let workspace_folders = root_uri.as_ref().map(|uri: &Uri| {
            vec![WorkspaceFolder {
                uri: uri.clone(),
                name: self
                    .session
                    .root()
                    .and_then(|root| root.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or("workspace")
                    .to_owned(),
            }]
        });
        let capabilities = serde_json::from_value::<ClientCapabilities>(json!({
            "workspace": {
                "configuration": true,
                "workspaceFolders": true
            },
            "textDocument": {
                "hover": {
                    "contentFormat": ["markdown", "plaintext"]
                },
                "completion": {
                    "completionItem": {
                        "documentationFormat": ["markdown", "plaintext"],
                        "snippetSupport": false
                    }
                },
                "publishDiagnostics": {
                    "relatedInformation": false
                },
                "synchronization": {
                    "didSave": true
                }
            }
        }))
        .map_err(|error| {
            LspClientError::Protocol(format!("failed to build LSP client capabilities: {error}"))
        })?;
        #[allow(deprecated)]
        let initialize_params = InitializeParams {
            process_id: Some(std::process::id()),
            root_path: None,
            root_uri,
            initialization_options: None,
            capabilities,
            trace: Some(TraceValue::Off),
            workspace_folders,
            client_info: Some(ClientInfo {
                name: "volt".to_owned(),
                version: None,
            }),
            locale: None,
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        let _ = self.request_typed::<Initialize>(initialize_params)?;
        self.notify_typed::<Initialized>(InitializedParams {})?;
        Ok(())
    }

    fn did_open(&self, path: &Path, version: i32, text: &str) -> Result<(), LspClientError> {
        self.notify_typed::<DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem::new(
                path_to_uri(path)?,
                self.session.language_id().to_owned(),
                version,
                text.to_owned(),
            ),
        })
    }

    fn did_change(&self, path: &Path, version: i32, text: &str) -> Result<(), LspClientError> {
        self.notify_typed::<DidChangeTextDocument>(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier::new(path_to_uri(path)?, version),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.to_owned(),
            }],
        })
    }

    fn did_save(&self, path: &Path) -> Result<(), LspClientError> {
        self.notify_typed::<DidSaveTextDocument>(DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
            text: None,
        })
    }

    fn did_close(&self, path: &Path) -> Result<(), LspClientError> {
        self.notify_typed::<DidCloseTextDocument>(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
        })
    }

    fn hover(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Option<LspHoverContents>, LspClientError> {
        let response = self.request_typed::<HoverRequest>(HoverParams {
            text_document_position_params: text_document_position_params(path, position)?,
            work_done_progress_params: WorkDoneProgressParams::default(),
        })?;
        Ok(parse_hover_response(self.server_id(), &response))
    }

    fn completions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspCompletionItem>, LspClientError> {
        let response = self.request_typed::<Completion>(CompletionParams {
            text_document_position: text_document_position_params(path, position)?,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })?;
        Ok(parse_completion_response(self.server_id(), &response))
    }

    fn diagnostics_for_path(&self, path: &Path) -> Vec<Diagnostic> {
        self.diagnostics
            .lock()
            .ok()
            .and_then(|diagnostics| diagnostics.get(path).cloned())
            .unwrap_or_default()
    }

    fn notify_typed<N>(&self, params: N::Params) -> Result<(), LspClientError>
    where
        N: Notification,
        N::Params: Serialize,
    {
        let params = serde_json::to_value(params).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to encode LSP notification params for `{}`: {error}",
                N::METHOD
            ))
        })?;
        self.notify(N::METHOD, params)
    }

    fn request_typed<R>(&self, params: R::Params) -> Result<Value, LspClientError>
    where
        R: Request,
        R::Params: Serialize,
    {
        let params = serde_json::to_value(params).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to encode LSP request params for `{}`: {error}",
                R::METHOD
            ))
        })?;
        self.request(R::METHOD, params)
    }

    fn notify(&self, method: &str, params: Value) -> Result<(), LspClientError> {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
    }

    fn request(&self, method: &str, params: Value) -> Result<Value, LspClientError> {
        let id = self.next_request_id.fetch_add(1, Ordering::AcqRel);
        let (sender, receiver) = mpsc::channel();
        self.pending
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP pending map mutex poisoned".to_owned()))?
            .insert(id, sender);
        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        if let Err(error) = self.send_message(&message) {
            if let Ok(mut pending) = self.pending.lock() {
                pending.remove(&id);
            }
            return Err(error);
        }
        match receiver.recv_timeout(REQUEST_TIMEOUT) {
            Ok(result) => result,
            Err(_) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&id);
                }
                record_transport_event(
                    &self.transport_log,
                    self.server_id(),
                    format!("timed out waiting for response to `{method}`"),
                );
                Err(LspClientError::Timeout(method.to_owned()))
            }
        }
    }

    fn send_message(&self, message: &Value) -> Result<(), LspClientError> {
        if self.is_disconnected() {
            record_transport_event(
                &self.transport_log,
                self.server_id(),
                "attempted to write after the server disconnected",
            );
            return Err(LspClientError::Disconnected(self.server_id().to_owned()));
        }
        let encoded = serde_json::to_vec(message).map_err(|error| {
            LspClientError::Protocol(format!("failed to encode JSON-RPC message: {error}"))
        })?;
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP writer mutex poisoned".to_owned()))?;
        write!(writer, "Content-Length: {}\r\n\r\n", encoded.len())?;
        writer.write_all(&encoded)?;
        writer.flush()?;
        record_transport_message(
            &self.transport_log,
            self.server_id(),
            LspLogDirection::Outgoing,
            message,
        );
        Ok(())
    }
}

fn cleanup_unused_sessions(state: &mut LspClientState, removed_keys: &BTreeSet<SessionKey>) {
    let still_referenced = state
        .tracked_buffers
        .values()
        .flat_map(|tracked| tracked.sessions.iter().cloned())
        .collect::<BTreeSet<_>>();
    for key in removed_keys {
        if !still_referenced.contains(key) {
            state.sessions.remove(key);
        }
    }
}

fn spawn_reader_thread(
    server_id: String,
    stdout: impl Read + Send + 'static,
    writer: Arc<Mutex<ChildStdin>>,
    pending: PendingResponseMap,
    diagnostics: DiagnosticsByPath,
    disconnected: Arc<AtomicBool>,
    transport_log: TransportLog,
) {
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let message = match read_message(&mut reader) {
                Ok(Some(message)) => message,
                Ok(None) => {
                    record_transport_event(
                        &transport_log,
                        &server_id,
                        "language server closed the transport",
                    );
                    break;
                }
                Err(error) => {
                    record_transport_event(
                        &transport_log,
                        &server_id,
                        format!("transport read error: {error}"),
                    );
                    break;
                }
            };
            record_transport_message(
                &transport_log,
                &server_id,
                LspLogDirection::Incoming,
                &message,
            );
            let Some(object) = message.as_object() else {
                continue;
            };
            if object.contains_key("method") && object.contains_key("id") {
                let response = server_request_response(object.get("method"), object.get("params"));
                let id = object.get("id").cloned().unwrap_or(Value::Null);
                let response_message = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": response,
                });
                if let Err(error) =
                    write_response(&server_id, &transport_log, &writer, response_message)
                {
                    record_transport_event(
                        &transport_log,
                        &server_id,
                        format!("failed to reply to server request: {error}"),
                    );
                    break;
                }
                continue;
            }
            if let Some(id) = object.get("id").and_then(Value::as_u64) {
                let result = if let Some(error) = object.get("error") {
                    Err(LspClientError::Protocol(format!(
                        "language server `{server_id}` returned an error: {error}"
                    )))
                } else {
                    Ok(object.get("result").cloned().unwrap_or(Value::Null))
                };
                if let Ok(mut pending) = pending.lock()
                    && let Some(sender) = pending.remove(&id)
                {
                    let _ = sender.send(result);
                }
                continue;
            }
            if object.get("method").and_then(Value::as_str)
                == Some("textDocument/publishDiagnostics")
                && let Some(params) = object.get("params")
                && let Some((path, parsed)) = parse_publish_diagnostics(params)
                && let Ok(mut guard) = diagnostics.lock()
            {
                guard.insert(path, parsed);
            }
        }
        disconnected.store(true, Ordering::Release);
        record_transport_event(&transport_log, &server_id, "marked session disconnected");
        if let Ok(mut pending) = pending.lock() {
            for sender in pending.values() {
                let _ = sender.send(Err(LspClientError::Disconnected(server_id.clone())));
            }
            pending.clear();
        }
    });
}

fn configure_lsp_command(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, LspClientError> {
    let mut content_length = None;
    loop {
        let mut header = String::new();
        let read = reader.read_line(&mut header)?;
        if read == 0 {
            return Ok(None);
        }
        let trimmed = header.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(raw_length) = trimmed.strip_prefix("Content-Length:") {
            content_length = raw_length.trim().parse::<usize>().ok();
        }
    }
    let content_length = content_length.ok_or_else(|| {
        LspClientError::Protocol("received JSON-RPC frame without Content-Length".to_owned())
    })?;
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body).map_err(|error| {
        LspClientError::Protocol(format!("failed to parse JSON-RPC payload: {error}"))
    })
}

fn write_response(
    server_id: &str,
    transport_log: &TransportLog,
    writer: &Arc<Mutex<ChildStdin>>,
    message: Value,
) -> Result<(), LspClientError> {
    let encoded = serde_json::to_vec(&message).map_err(|error| {
        LspClientError::Protocol(format!("failed to encode JSON-RPC response: {error}"))
    })?;
    let mut writer = writer
        .lock()
        .map_err(|_| LspClientError::Protocol("LSP writer mutex poisoned".to_owned()))?;
    write!(writer, "Content-Length: {}\r\n\r\n", encoded.len())?;
    writer.write_all(&encoded)?;
    writer.flush()?;
    record_transport_message(
        transport_log,
        server_id,
        LspLogDirection::Outgoing,
        &message,
    );
    Ok(())
}

fn launch_summary(
    pid: u32,
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    root: Option<&Path>,
) -> String {
    let mut lines = Vec::with_capacity(3);
    let mut command_line = format!("started process {pid}: {program}");
    if !args.is_empty() {
        command_line.push(' ');
        command_line.push_str(&args.join(" "));
    }
    lines.push(command_line);
    if let Some(cwd) = cwd {
        lines.push(format!("cwd: {}", cwd.display()));
    }
    if let Some(root) = root {
        lines.push(format!("root: {}", root.display()));
    }
    lines.join("\n")
}

fn format_transport_message(message: &Value) -> String {
    serde_json::to_string_pretty(message).unwrap_or_else(|_| message.to_string())
}

fn record_transport_message(
    transport_log: &TransportLog,
    server_id: &str,
    direction: LspLogDirection,
    message: &Value,
) {
    record_transport_entry(
        transport_log,
        LspLogEntry::new(direction, server_id, format_transport_message(message)),
    );
}

fn record_transport_event(
    transport_log: &TransportLog,
    server_id: &str,
    message: impl Into<String>,
) {
    record_transport_entry(
        transport_log,
        LspLogEntry::new(LspLogDirection::Event, server_id, message),
    );
}

fn record_transport_entry(transport_log: &TransportLog, entry: LspLogEntry) {
    if let Ok(mut log) = transport_log.lock() {
        log.record(entry);
    }
}

fn server_request_response(method: Option<&Value>, params: Option<&Value>) -> Value {
    match method.and_then(Value::as_str) {
        Some("workspace/configuration") => {
            let item_count = params
                .and_then(|params| params.get("items"))
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0);
            Value::Array((0..item_count).map(|_| Value::Null).collect())
        }
        Some("workspace/workspaceFolders") => Value::Array(Vec::new()),
        Some("client/registerCapability")
        | Some("client/unregisterCapability")
        | Some("window/workDoneProgress/create") => Value::Null,
        _ => Value::Null,
    }
}

fn parse_publish_diagnostics(params: &Value) -> Option<(PathBuf, Vec<Diagnostic>)> {
    let uri = params.get("uri")?.as_str()?;
    let path = file_uri_to_path(uri)?;
    let diagnostics = params
        .get("diagnostics")
        .and_then(Value::as_array)
        .map(|diagnostics| {
            diagnostics
                .iter()
                .filter_map(parse_diagnostic)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some((path, diagnostics))
}

fn parse_diagnostic(value: &Value) -> Option<Diagnostic> {
    let range = value.get("range")?;
    let start = range.get("start")?;
    let end = range.get("end")?;
    let start = TextPoint::new(
        start.get("line")?.as_u64()? as usize,
        start.get("character")?.as_u64()? as usize,
    );
    let end = TextPoint::new(
        end.get("line")?.as_u64()? as usize,
        end.get("character")?.as_u64()? as usize,
    );
    let severity = match value.get("severity").and_then(Value::as_u64).unwrap_or(3) {
        1 => DiagnosticSeverity::Error,
        2 => DiagnosticSeverity::Warning,
        _ => DiagnosticSeverity::Information,
    };
    Some(Diagnostic::new(
        value
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("lsp")
            .to_owned(),
        value
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        severity,
        TextRange::new(start, end),
    ))
}

fn parse_hover_response(server_id: &str, value: &Value) -> Option<LspHoverContents> {
    let contents = value.get("contents")?;
    let lines = hover_lines(contents);
    (!lines.is_empty()).then(|| LspHoverContents::new(server_id, lines))
}

fn hover_lines(value: &Value) -> Vec<String> {
    match value {
        Value::String(text) => normalize_lines(text),
        Value::Array(values) => values.iter().flat_map(hover_lines).collect::<Vec<_>>(),
        Value::Object(map) => {
            if let Some(text) = map.get("value").and_then(Value::as_str) {
                return normalize_lines(text);
            }
            if let Some(text) = map.get("language").and_then(Value::as_str) {
                let mut lines = vec![format!("Language: {text}")];
                if let Some(value) = map.get("value").and_then(Value::as_str) {
                    lines.extend(normalize_lines(value));
                }
                return lines;
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

fn parse_completion_response(server_id: &str, value: &Value) -> Vec<LspCompletionItem> {
    let empty = Vec::new();
    let items = match value {
        Value::Array(items) => items,
        Value::Object(map) => map.get("items").and_then(Value::as_array).unwrap_or(&empty),
        _ => return Vec::new(),
    };
    items
        .iter()
        .filter_map(|item| parse_completion_item(server_id, item))
        .collect()
}

fn parse_completion_item(server_id: &str, value: &Value) -> Option<LspCompletionItem> {
    let label = value.get("label")?.as_str()?.to_owned();
    let kind = value
        .get("kind")
        .and_then(Value::as_u64)
        .and_then(parse_completion_kind);
    let insert_text = value
        .get("insertText")
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .get("textEdit")
                .and_then(|text_edit| text_edit.get("newText"))
                .and_then(Value::as_str)
        })
        .unwrap_or(&label)
        .to_owned();
    let detail = value
        .get("detail")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let documentation = value
        .get("documentation")
        .and_then(completion_documentation)
        .or_else(|| detail.clone());
    Some(LspCompletionItem::new(
        server_id,
        kind,
        label,
        insert_text,
        detail,
        documentation,
    ))
}

fn completion_documentation(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.to_owned()),
        Value::Object(map) => map.get("value").and_then(Value::as_str).map(str::to_owned),
        _ => None,
    }
}

fn parse_completion_kind(kind: u64) -> Option<LspCompletionKind> {
    match kind {
        1 => Some(LspCompletionKind::Text),
        2 => Some(LspCompletionKind::Method),
        3 => Some(LspCompletionKind::Function),
        4 => Some(LspCompletionKind::Constructor),
        5 => Some(LspCompletionKind::Field),
        6 => Some(LspCompletionKind::Variable),
        7 => Some(LspCompletionKind::Class),
        8 => Some(LspCompletionKind::Interface),
        9 => Some(LspCompletionKind::Module),
        10 => Some(LspCompletionKind::Property),
        11 => Some(LspCompletionKind::Unit),
        12 => Some(LspCompletionKind::Value),
        13 => Some(LspCompletionKind::Enum),
        14 => Some(LspCompletionKind::Keyword),
        15 => Some(LspCompletionKind::Snippet),
        16 => Some(LspCompletionKind::Color),
        17 => Some(LspCompletionKind::File),
        18 => Some(LspCompletionKind::Reference),
        19 => Some(LspCompletionKind::Folder),
        20 => Some(LspCompletionKind::EnumMember),
        21 => Some(LspCompletionKind::Constant),
        22 => Some(LspCompletionKind::Struct),
        23 => Some(LspCompletionKind::Event),
        24 => Some(LspCompletionKind::Operator),
        25 => Some(LspCompletionKind::TypeParameter),
        _ => None,
    }
}

fn normalize_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn path_to_file_uri(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    let mut uri = String::from("file://");
    if !raw.starts_with('/') {
        uri.push('/');
    }
    for byte in raw.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'.' | b'_' | b'~') {
            uri.push(char::from(byte));
        } else {
            uri.push('%');
            uri.push_str(&format!("{byte:02X}"));
        }
    }
    uri
}

fn path_to_uri(path: &Path) -> Result<Uri, LspClientError> {
    path_to_file_uri(path).parse().map_err(|error| {
        LspClientError::Protocol(format!(
            "failed to convert `{}` into a valid file URI: {error}",
            path.display()
        ))
    })
}

fn text_document_position_params(
    path: &Path,
    position: TextPoint,
) -> Result<TextDocumentPositionParams, LspClientError> {
    let line = u32::try_from(position.line).map_err(|_| {
        LspClientError::Protocol(format!(
            "line {} does not fit in LSP position range",
            position.line
        ))
    })?;
    let character = u32::try_from(position.column).map_err(|_| {
        LspClientError::Protocol(format!(
            "column {} does not fit in LSP position range",
            position.column
        ))
    })?;
    Ok(TextDocumentPositionParams {
        text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
        position: Position::new(line, character),
    })
}

fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let raw = uri.strip_prefix("file://")?;
    let decoded = percent_decode(raw);
    #[cfg(windows)]
    {
        let trimmed = decoded
            .strip_prefix('/')
            .filter(|value| value.as_bytes().get(1) == Some(&b':'))
            .unwrap_or(decoded.as_str());
        Some(PathBuf::from(trimmed.replace('/', "\\")))
    }
    #[cfg(not(windows))]
    {
        Some(PathBuf::from(decoded))
    }
}

fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let high = bytes[index + 1] as char;
            let low = bytes[index + 2] as char;
            let value = [high, low].iter().collect::<String>();
            if let Ok(byte) = u8::from_str_radix(&value, 16) {
                decoded.push(byte);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_parser_handles_lists_and_docs() {
        let response = json!({
            "isIncomplete": false,
            "items": [
                {
                    "label": "println!",
                    "kind": 3,
                    "insertText": "println!",
                    "detail": "macro_rules! println",
                    "documentation": { "kind": "markdown", "value": "Prints to stdout." }
                }
            ]
        });
        let items = parse_completion_response("rust-analyzer", &response);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label(), "println!");
        assert_eq!(items[0].kind(), Some(LspCompletionKind::Function));
        assert_eq!(items[0].documentation(), Some("Prints to stdout."));
    }

    #[test]
    fn hover_parser_flattens_markup_content() {
        let response = json!({
            "contents": {
                "kind": "markdown",
                "value": "```rust\nfn example()\n```\n\nSample docs"
            }
        });
        let hover = parse_hover_response("rust-analyzer", &response).expect("hover");
        assert_eq!(hover.server_id(), "rust-analyzer");
        assert!(hover.lines().iter().any(|line| line.contains("fn example")));
        assert!(
            hover
                .lines()
                .iter()
                .any(|line| line.contains("Sample docs"))
        );
    }

    #[test]
    fn diagnostics_parser_maps_lsp_fields() {
        let params = json!({
            "uri": "file:///P:/volt/src/main.rs",
            "diagnostics": [
                {
                    "range": {
                        "start": { "line": 1, "character": 2 },
                        "end": { "line": 1, "character": 5 }
                    },
                    "severity": 2,
                    "source": "rust-analyzer",
                    "message": "unused binding"
                }
            ]
        });
        let (path, diagnostics) = parse_publish_diagnostics(&params).expect("diagnostics");
        assert!(path.ends_with(Path::new("src").join("main.rs")));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity(), DiagnosticSeverity::Warning);
        assert_eq!(diagnostics[0].message(), "unused binding");
    }

    #[test]
    fn file_uri_roundtrip_handles_windows_paths() {
        let path = PathBuf::from(r"P:\volt\src\main.rs");
        let uri = path_to_file_uri(&path);
        assert_eq!(file_uri_to_path(&uri), Some(path));
    }

    #[test]
    fn transport_log_snapshot_is_bounded_and_tracks_revision() {
        let mut log = LspTransportLog::new(2);
        log.record(LspLogEntry::new(
            LspLogDirection::Event,
            "rust-analyzer",
            "started",
        ));
        log.record(LspLogEntry::new(
            LspLogDirection::Outgoing,
            "rust-analyzer",
            "{\"id\":1}",
        ));
        log.record(LspLogEntry::new(
            LspLogDirection::Incoming,
            "rust-analyzer",
            "{\"result\":1}",
        ));

        let snapshot = log.snapshot();
        assert_eq!(snapshot.revision(), 3);
        assert_eq!(snapshot.entries().len(), 2);
        assert_eq!(snapshot.entries()[0].direction(), LspLogDirection::Outgoing);
        assert_eq!(snapshot.entries()[1].direction(), LspLogDirection::Incoming);
    }
}
