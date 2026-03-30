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
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentFormattingParams, DocumentRangeFormattingParams, Documentation, FormattingOptions,
    GotoDefinitionParams, GotoDefinitionResponse, HoverParams, InitializeParams, InitializedParams,
    Location, LocationLink, NumberOrString, ParameterLabel, PartialResultParams, Position,
    ReferenceContext, ReferenceParams, SignatureHelp, SignatureHelpParams,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TextEdit, TraceValue, Uri, VersionedTextDocumentIdentifier,
    WorkDoneProgressParams, WorkspaceFolder,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Initialized, Notification,
    },
    request::{
        Completion, Formatting, GotoDefinition, GotoImplementation, HoverRequest, Initialize,
        RangeFormatting, References, Request, SignatureHelpRequest,
    },
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerSession, LspError,
};

const REQUEST_TIMEOUT: Duration = Duration::from_millis(400);
const INITIALIZE_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const TRANSPORT_LOG_MAX_ENTRIES: usize = 400;
const NOTIFICATION_LOG_MAX_ENTRIES: usize = 128;
const CODE_ACTION_METHOD: &str = "textDocument/codeAction";

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

type PendingResponseTx = Sender<Result<Value, LspClientError>>;
type PendingResponseMap = Arc<Mutex<BTreeMap<u64, PendingResponseTx>>>;
type DiagnosticsByPath = Arc<Mutex<BTreeMap<PathBuf, Vec<Diagnostic>>>>;
type TransportLog = Arc<Mutex<LspTransportLog>>;
type NotificationLog = Arc<Mutex<LspNotificationLog>>;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspSignatureHelpContents {
    server_id: String,
    lines: Vec<String>,
}

impl LspSignatureHelpContents {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspLocation {
    server_id: String,
    path: PathBuf,
    range: TextRange,
}

impl LspLocation {
    fn new(server_id: impl Into<String>, path: PathBuf, range: TextRange) -> Self {
        Self {
            server_id: server_id.into(),
            path,
            range,
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn range(&self) -> TextRange {
        self.range
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspTextEdit {
    range: TextRange,
    new_text: String,
}

impl LspTextEdit {
    fn new(range: TextRange, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }

    pub const fn range(&self) -> TextRange {
        self.range
    }

    pub fn new_text(&self) -> &str {
        &self.new_text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspDocumentTextEdits {
    path: PathBuf,
    edits: Vec<LspTextEdit>,
}

impl LspDocumentTextEdits {
    fn new(path: PathBuf, edits: Vec<LspTextEdit>) -> Self {
        Self { path, edits }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn edits(&self) -> &[LspTextEdit] {
        &self.edits
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspCodeAction {
    server_id: String,
    title: String,
    kind: Option<String>,
    disabled_reason: Option<String>,
    preferred: bool,
    document_edits: Vec<LspDocumentTextEdits>,
    command_name: Option<String>,
    has_resource_operations: bool,
}

impl LspCodeAction {
    #[allow(clippy::too_many_arguments)]
    fn new(
        server_id: impl Into<String>,
        title: impl Into<String>,
        kind: Option<String>,
        disabled_reason: Option<String>,
        preferred: bool,
        document_edits: Vec<LspDocumentTextEdits>,
        command_name: Option<String>,
        has_resource_operations: bool,
    ) -> Self {
        Self {
            server_id: server_id.into(),
            title: title.into(),
            kind,
            disabled_reason,
            preferred,
            document_edits,
            command_name,
            has_resource_operations,
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn kind(&self) -> Option<&str> {
        self.kind.as_deref()
    }

    pub fn disabled_reason(&self) -> Option<&str> {
        self.disabled_reason.as_deref()
    }

    pub const fn is_preferred(&self) -> bool {
        self.preferred
    }

    pub fn document_edits(&self) -> &[LspDocumentTextEdits] {
        &self.document_edits
    }

    pub fn command_name(&self) -> Option<&str> {
        self.command_name.as_deref()
    }

    pub const fn has_resource_operations(&self) -> bool {
        self.has_resource_operations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspFormattingOptions {
    tab_size: u32,
    insert_spaces: bool,
}

impl LspFormattingOptions {
    pub const fn new(tab_size: u32, insert_spaces: bool) -> Self {
        Self {
            tab_size,
            insert_spaces,
        }
    }

    pub const fn tab_size(&self) -> u32 {
        self.tab_size
    }

    pub const fn insert_spaces(&self) -> bool {
        self.insert_spaces
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

/// Notification severity surfaced to the shell UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspNotificationLevel {
    /// Informational update.
    Info,
    /// Successful completion update.
    Success,
    /// Warning update.
    Warning,
    /// Error update.
    Error,
}

/// Optional progress metadata attached to an LSP notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspNotificationProgress {
    percentage: Option<u32>,
}

impl LspNotificationProgress {
    fn new(percentage: Option<u32>) -> Self {
        Self { percentage }
    }

    /// Returns the latest reported completion percentage, if available.
    pub const fn percentage(self) -> Option<u32> {
        self.percentage
    }
}

/// UI-facing LSP notification entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspNotification {
    key: String,
    server_id: String,
    level: LspNotificationLevel,
    title: String,
    body_lines: Vec<String>,
    progress: Option<LspNotificationProgress>,
    active: bool,
}

impl LspNotification {
    fn new(
        key: impl Into<String>,
        server_id: impl Into<String>,
        level: LspNotificationLevel,
        title: impl Into<String>,
        body_lines: Vec<String>,
        progress: Option<LspNotificationProgress>,
        active: bool,
    ) -> Self {
        Self {
            key: key.into(),
            server_id: server_id.into(),
            level,
            title: title.into(),
            body_lines,
            progress,
            active,
        }
    }

    /// Returns the deduplication key for this notification.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the originating language server id.
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    /// Returns the notification severity level.
    pub const fn level(&self) -> LspNotificationLevel {
        self.level
    }

    /// Returns the notification title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the body lines to display under the title.
    pub fn body_lines(&self) -> &[String] {
        &self.body_lines
    }

    /// Returns progress metadata, if the notification represents in-flight work.
    pub const fn progress(&self) -> Option<LspNotificationProgress> {
        self.progress
    }

    /// Returns whether the notification is still active and should stay pinned.
    pub const fn active(&self) -> bool {
        self.active
    }
}

/// Revision-tagged notification update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspNotificationEntry {
    revision: u64,
    notification: LspNotification,
}

impl LspNotificationEntry {
    /// Returns the monotonically increasing revision for this update.
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the notification payload.
    pub fn notification(&self) -> &LspNotification {
        &self.notification
    }
}

/// Snapshot of recent UI-facing LSP notifications.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LspNotificationSnapshot {
    revision: u64,
    entries: Vec<LspNotificationEntry>,
}

impl LspNotificationSnapshot {
    /// Returns the latest notification revision seen by the manager.
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the buffered notification updates.
    pub fn entries(&self) -> &[LspNotificationEntry] {
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
struct LspNotificationLog {
    revision: u64,
    entries: Vec<LspNotificationEntry>,
    max_entries: usize,
}

impl LspNotificationLog {
    fn new(max_entries: usize) -> Self {
        Self {
            revision: 0,
            entries: Vec::new(),
            max_entries,
        }
    }

    fn record(&mut self, notification: LspNotification) {
        self.revision = self.revision.saturating_add(1);
        self.entries.push(LspNotificationEntry {
            revision: self.revision,
            notification,
        });
        if self.entries.len() > self.max_entries {
            let overflow = self.entries.len() - self.max_entries;
            self.entries.drain(0..overflow);
        }
    }

    fn snapshot(&self) -> LspNotificationSnapshot {
        LspNotificationSnapshot {
            revision: self.revision,
            entries: self.entries.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ProgressTrack {
    title: Option<String>,
    message: Option<String>,
    percentage: Option<u32>,
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
    notifications: NotificationLog,
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
    next_progress_token: AtomicU64,
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
            notifications: Arc::new(Mutex::new(LspNotificationLog::new(
                NOTIFICATION_LOG_MAX_ENTRIES,
            ))),
        }
    }

    pub fn log_snapshot(&self) -> LspLogSnapshot {
        self.transport_log
            .lock()
            .map(|log| log.snapshot())
            .unwrap_or_default()
    }

    /// Returns a snapshot of recent UI-facing notifications emitted by the LSP client.
    pub fn notification_snapshot(&self) -> LspNotificationSnapshot {
        self.notifications
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

    pub fn signature_help(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspSignatureHelpContents>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut results = Vec::new();
        for session in sessions {
            if let Some(signature_help) = session.signature_help(path, position)? {
                results.push(signature_help);
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

    pub fn definitions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut locations = Vec::new();
        for session in sessions {
            locations.extend(session.definitions(path, position)?);
        }
        sort_locations(&mut locations);
        Ok(locations)
    }

    pub fn references(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut locations = Vec::new();
        for session in sessions {
            locations.extend(session.references(path, position)?);
        }
        sort_locations(&mut locations);
        Ok(locations)
    }

    pub fn implementations(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut locations = Vec::new();
        for session in sessions {
            locations.extend(session.implementations(path, position)?);
        }
        sort_locations(&mut locations);
        Ok(locations)
    }

    pub fn code_actions(
        &self,
        path: &Path,
        range: TextRange,
    ) -> Result<Vec<LspCodeAction>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut actions = Vec::new();
        for session in sessions {
            actions.extend(session.code_actions(path, range)?);
        }
        actions.sort_by(|left, right| {
            right
                .preferred
                .cmp(&left.preferred)
                .then_with(|| left.title.cmp(&right.title))
                .then_with(|| left.server_id.cmp(&right.server_id))
                .then_with(|| left.kind.cmp(&right.kind))
        });
        Ok(actions)
    }

    pub fn formatting(
        &self,
        path: &Path,
        options: LspFormattingOptions,
    ) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        for session in sessions {
            if let Some(edits) = session.formatting(path, options)? {
                return Ok(Some(edits));
            }
        }
        Ok(None)
    }

    pub fn range_formatting(
        &self,
        path: &Path,
        range: TextRange,
        options: LspFormattingOptions,
    ) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        for session in sessions {
            if let Some(edits) = session.range_formatting(path, range, options)? {
                return Ok(Some(edits));
            }
        }
        Ok(None)
    }

    pub fn session_labels_for_path(&self, path: &Path) -> Vec<String> {
        let mut labels = self
            .tracked_sessions_for_path(path)
            .unwrap_or_default()
            .into_iter()
            .map(|session| session.server_id().to_owned())
            .collect::<Vec<_>>();
        labels.sort();
        labels.dedup();
        labels
    }

    pub fn has_live_sessions_for_path(&self, path: &Path) -> bool {
        self.tracked_sessions_for_path(path)
            .map(|sessions| !sessions.is_empty())
            .unwrap_or(false)
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
        let session_plans = if let Some(server_id) = preferred_server_id {
            vec![
                self.registry
                    .prepare_session_for_path(server_id, path, root)?,
            ]
        } else {
            self.registry.prepare_sessions_for_path(path, root)?
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
            match LspSessionHandle::start(
                session,
                Arc::clone(&self.transport_log),
                Arc::clone(&self.notifications),
            ) {
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
                    record_notification(
                        &self.notifications,
                        session_lifecycle_notification(
                            &key.server_id,
                            key.root.as_deref(),
                            LspNotificationLevel::Error,
                            vec![
                                "Failed to start language server".to_owned(),
                                error.to_string(),
                            ],
                            false,
                        ),
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
        notifications: NotificationLog,
    ) -> Result<Arc<Self>, LspClientError> {
        let launch = session.launch();
        let launch_program = launch.program().to_owned();
        let launch_args = launch.args().to_vec();
        let launch_cwd = launch.cwd().cloned();
        let launch_env = launch.env().to_vec();
        let mut child = spawn_lsp_command(
            &launch_program,
            &launch_args,
            launch_cwd.as_deref(),
            &launch_env,
        )
        .map_err(|error| {
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
            next_progress_token: AtomicU64::new(1),
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
        record_notification(
            &notifications,
            session_lifecycle_notification(
                handle.server_id(),
                handle.key.root.as_deref(),
                LspNotificationLevel::Info,
                vec!["Starting language server".to_owned()],
                true,
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
            Arc::clone(&notifications),
        );
        handle.initialize()?;
        record_notification(
            &notifications,
            session_lifecycle_notification(
                handle.server_id(),
                handle.key.root.as_deref(),
                LspNotificationLevel::Success,
                vec!["Ready".to_owned()],
                false,
            ),
        );
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
        let capabilities = client_capabilities()?;
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
            work_done_progress_params: self.work_done_progress_params(Initialize::METHOD),
        };
        let _ = self.request_typed::<Initialize>(initialize_params)?;
        self.notify_typed::<Initialized>(InitializedParams {})?;
        Ok(())
    }

    fn did_open(&self, path: &Path, version: i32, text: &str) -> Result<(), LspClientError> {
        self.notify_typed::<DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem::new(
                path_to_uri(path)?,
                self.session.document_language_id_for_path(path).to_owned(),
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
            work_done_progress_params: self.work_done_progress_params(HoverRequest::METHOD),
        })?;
        Ok(parse_hover_response(self.server_id(), &response))
    }

    fn signature_help(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Option<LspSignatureHelpContents>, LspClientError> {
        let response = match self.request_typed::<SignatureHelpRequest>(SignatureHelpParams {
            context: None,
            text_document_position_params: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(SignatureHelpRequest::METHOD),
        }) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        parse_signature_help_response(self.server_id(), &response)
    }

    fn completions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspCompletionItem>, LspClientError> {
        let response = self.request_typed::<Completion>(CompletionParams {
            text_document_position: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(Completion::METHOD),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })?;
        Ok(parse_completion_response(self.server_id(), &response))
    }

    fn definitions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let response = self.request_typed::<GotoDefinition>(GotoDefinitionParams {
            text_document_position_params: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(GotoDefinition::METHOD),
            partial_result_params: PartialResultParams::default(),
        })?;
        parse_definition_response(self.server_id(), &response)
    }

    fn references(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let response = self.request_typed::<References>(ReferenceParams {
            text_document_position: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(References::METHOD),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration: false,
            },
        })?;
        parse_reference_response(self.server_id(), &response)
    }

    fn implementations(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let response = self.request_typed::<GotoImplementation>(GotoDefinitionParams {
            text_document_position_params: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(GotoImplementation::METHOD),
            partial_result_params: PartialResultParams::default(),
        })?;
        parse_definition_response(self.server_id(), &response)
    }

    fn code_actions(
        &self,
        path: &Path,
        range: TextRange,
    ) -> Result<Vec<LspCodeAction>, LspClientError> {
        let range = range.normalized();
        let diagnostics = self
            .diagnostics_for_path(path)
            .into_iter()
            .filter(|diagnostic| diagnostic_matches_request_range(diagnostic.range(), range))
            .map(|diagnostic| lsp_code_action_diagnostic(&diagnostic))
            .collect::<Vec<_>>();
        let response = match self.request(
            CODE_ACTION_METHOD,
            json!({
                "textDocument": {
                    "uri": path_to_uri(path)?,
                },
                "range": lsp_range_from_text_range(range),
                "context": {
                    "diagnostics": diagnostics,
                },
                "workDoneProgressParams": self.work_done_progress_params(CODE_ACTION_METHOD),
                "partialResultParams": {},
            }),
        ) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        parse_code_action_response(self.server_id(), &response)
    }

    fn formatting(
        &self,
        path: &Path,
        options: LspFormattingOptions,
    ) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
        let response = match self.request_typed::<Formatting>(DocumentFormattingParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
            options: lsp_formatting_options(options),
            work_done_progress_params: self.work_done_progress_params(Formatting::METHOD),
        }) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        parse_text_edit_response(self.server_id(), "formatting", &response)
    }

    fn range_formatting(
        &self,
        path: &Path,
        range: TextRange,
        options: LspFormattingOptions,
    ) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
        let response = match self.request_typed::<RangeFormatting>(DocumentRangeFormattingParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
            range: lsp_range_from_text_range(range),
            options: lsp_formatting_options(options),
            work_done_progress_params: self.work_done_progress_params(RangeFormatting::METHOD),
        }) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        parse_text_edit_response(self.server_id(), "range formatting", &response)
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

    fn work_done_progress_params(&self, method: &str) -> WorkDoneProgressParams {
        work_done_progress_params(&self.next_progress_token, method)
    }

    fn notify(&self, method: &str, params: Value) -> Result<(), LspClientError> {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
    }

    fn request(&self, method: &str, params: Value) -> Result<Value, LspClientError> {
        let timeout = request_timeout_for_method(method);
        self.request_with_timeout(method, params, timeout)
    }

    fn request_with_timeout(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, LspClientError> {
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
        match receiver.recv_timeout(timeout) {
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

#[allow(clippy::too_many_arguments)]
fn spawn_reader_thread(
    server_id: String,
    stdout: impl Read + Send + 'static,
    writer: Arc<Mutex<ChildStdin>>,
    pending: PendingResponseMap,
    diagnostics: DiagnosticsByPath,
    disconnected: Arc<AtomicBool>,
    transport_log: TransportLog,
    notifications: NotificationLog,
) {
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut progress_tracks = BTreeMap::<String, ProgressTrack>::new();
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
            if let Some(method) = object.get("method").and_then(Value::as_str) {
                if method == "textDocument/publishDiagnostics"
                    && let Some(params) = object.get("params")
                    && let Some((path, parsed)) = parse_publish_diagnostics(params)
                    && let Ok(mut guard) = diagnostics.lock()
                {
                    guard.insert(path, parsed);
                    continue;
                }
                if method == "$/progress"
                    && let Some(params) = object.get("params")
                    && let Some(notification) =
                        parse_progress_notification(&server_id, params, &mut progress_tracks)
                {
                    record_notification(&notifications, notification);
                    continue;
                }
                if method == "window/showMessage"
                    && let Some(params) = object.get("params")
                    && let Some(notification) = parse_show_message_notification(&server_id, params)
                {
                    record_notification(&notifications, notification);
                    continue;
                }
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

fn configure_lsp_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn spawn_lsp_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> std::io::Result<Child> {
    #[cfg(not(windows))]
    let spawn_result = build_lsp_command(program, args, cwd, env, None).spawn();

    #[cfg(windows)]
    let mut spawn_result = build_lsp_command(program, args, cwd, env, None).spawn();
    #[cfg(windows)]
    {
        let should_retry = matches!(
            &spawn_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry {
            for candidate in windows_launch_program_candidates(program) {
                spawn_result = build_lsp_command(&candidate, args, cwd, env, None).spawn();
                match &spawn_result {
                    Ok(_) => break,
                    Err(error) if windows_should_retry_spawn_error(error) => {}
                    Err(_) => break,
                }
            }
        }
        let should_retry_with_fnm = matches!(
            &spawn_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry_with_fnm && let Some(fnm_env) = windows_fnm_environment(cwd, env) {
            for candidate in windows_fnm_launch_program_candidates(program, &fnm_env) {
                spawn_result =
                    build_lsp_command(&candidate, args, cwd, env, Some(&fnm_env)).spawn();
                match &spawn_result {
                    Ok(_) => break,
                    Err(error) if windows_should_retry_spawn_error(error) => {}
                    Err(_) => break,
                }
            }
        }
    }
    spawn_result
}

fn build_lsp_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    #[cfg(windows)] fnm_env: Option<&[(String, String)]>,
    #[cfg(not(windows))] _fnm_env: Option<&[(String, String)]>,
) -> Command {
    let mut command = Command::new(program);
    configure_lsp_command(&mut command);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    #[cfg(windows)]
    if let Some(fnm_env) = fnm_env {
        apply_windows_fnm_environment(&mut command, env, fnm_env);
    } else {
        apply_command_environment(&mut command, env);
    }
    #[cfg(not(windows))]
    apply_command_environment(&mut command, env);
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
fn windows_should_retry_spawn_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(193)
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
fn windows_fnm_environment(
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> Option<Vec<(String, String)>> {
    let mut command = Command::new("fnm");
    configure_lsp_command(&mut command);
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

fn record_notification(notifications: &NotificationLog, notification: LspNotification) {
    if let Ok(mut log) = notifications.lock() {
        log.record(notification);
    }
}

fn session_notification_key(server_id: &str, root: Option<&Path>) -> String {
    match root {
        Some(root) => format!("session:{server_id}:{}", root.display()),
        None => format!("session:{server_id}:global"),
    }
}

fn client_capabilities() -> Result<ClientCapabilities, LspClientError> {
    serde_json::from_value::<ClientCapabilities>(json!({
        "workspace": {
            "workspaceEdit": {
                "documentChanges": true
            },
            "configuration": true,
            "workspaceFolders": true
        },
        "window": {
            "workDoneProgress": true
        },
        "textDocument": {
            "hover": {
                "contentFormat": ["markdown", "plaintext"]
            },
            "signatureHelp": {
                "signatureInformation": {
                    "documentationFormat": ["markdown", "plaintext"],
                    "parameterInformation": {
                        "labelOffsetSupport": true
                    },
                    "activeParameterSupport": true
                }
            },
            "completion": {
                "completionItem": {
                    "documentationFormat": ["markdown", "plaintext"],
                    "snippetSupport": false
                }
            },
            "codeAction": {
                "dynamicRegistration": false,
                "isPreferredSupport": true,
                "disabledSupport": true,
                "codeActionLiteralSupport": {
                    "codeActionKind": {
                        "valueSet": [
                            "",
                            "quickfix",
                            "refactor",
                            "refactor.extract",
                            "refactor.inline",
                            "refactor.rewrite",
                            "source",
                            "source.fixAll",
                            "source.organizeImports"
                        ]
                    }
                }
            },
            "formatting": {
                "dynamicRegistration": false
            },
            "rangeFormatting": {
                "dynamicRegistration": false
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
    })
}

fn work_done_progress_params(
    next_progress_token: &AtomicU64,
    method: &str,
) -> WorkDoneProgressParams {
    let token = next_progress_token.fetch_add(1, Ordering::AcqRel);
    WorkDoneProgressParams {
        work_done_token: Some(NumberOrString::String(format!("progress:{method}:{token}"))),
    }
}

fn request_timeout_for_method(method: &str) -> Duration {
    if method == Initialize::METHOD {
        INITIALIZE_REQUEST_TIMEOUT
    } else {
        REQUEST_TIMEOUT
    }
}

fn session_lifecycle_notification(
    server_id: &str,
    root: Option<&Path>,
    level: LspNotificationLevel,
    body_lines: Vec<String>,
    active: bool,
) -> LspNotification {
    let mut lines = body_lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    if let Some(root) = root {
        lines.push(root.display().to_string());
    }
    LspNotification::new(
        session_notification_key(server_id, root),
        server_id,
        level,
        format!("LSP · {server_id}"),
        lines,
        None,
        active,
    )
}

fn progress_notification_key(server_id: &str, token: &str) -> String {
    format!("progress:{server_id}:{token}")
}

fn parse_progress_token_key(value: Option<&Value>) -> Option<String> {
    let token = value?;
    if let Some(token) = token.as_str() {
        return Some(token.to_owned());
    }
    token.as_u64().map(|token| token.to_string())
}

fn parse_optional_progress_text(value: Option<&Value>) -> Option<Option<String>> {
    value.map(|value| {
        value
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    })
}

fn parse_progress_percentage(value: Option<&Value>) -> Option<Option<u32>> {
    value.map(|value| {
        value
            .as_u64()
            .and_then(|percentage| u32::try_from(percentage.min(100)).ok())
    })
}

fn progress_body_lines(track: &ProgressTrack) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(title) = track.title.as_deref() {
        lines.push(title.to_owned());
    }
    if let Some(message) = track.message.as_deref()
        && lines.last().is_none_or(|title| title != message)
    {
        lines.push(message.to_owned());
    }
    if lines.is_empty() {
        lines.push("Working".to_owned());
    }
    lines
}

fn completion_level_for_message(message: Option<&str>) -> LspNotificationLevel {
    let Some(message) = message else {
        return LspNotificationLevel::Success;
    };
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("fail") || normalized.contains("error") {
        LspNotificationLevel::Error
    } else if normalized.contains("warn") {
        LspNotificationLevel::Warning
    } else {
        LspNotificationLevel::Success
    }
}

fn parse_progress_notification(
    server_id: &str,
    params: &Value,
    progress_tracks: &mut BTreeMap<String, ProgressTrack>,
) -> Option<LspNotification> {
    let token = parse_progress_token_key(params.get("token"))?;
    let value = params.get("value")?;
    let kind = value.get("kind")?.as_str()?;
    match kind {
        "begin" => {
            let title = parse_optional_progress_text(value.get("title")).flatten();
            let message = parse_optional_progress_text(value.get("message")).flatten();
            let percentage = parse_progress_percentage(value.get("percentage")).flatten();
            let track = ProgressTrack {
                title,
                message,
                percentage,
            };
            let progress = track.percentage.map(Some).unwrap_or(None);
            let body_lines = progress_body_lines(&track);
            progress_tracks.insert(token.clone(), track);
            Some(LspNotification::new(
                progress_notification_key(server_id, &token),
                server_id,
                LspNotificationLevel::Info,
                format!("LSP · {server_id}"),
                body_lines,
                Some(LspNotificationProgress::new(progress)),
                true,
            ))
        }
        "report" => {
            let track = progress_tracks.entry(token.clone()).or_default();
            if let Some(title) = parse_optional_progress_text(value.get("title")) {
                track.title = title;
            }
            if let Some(message) = parse_optional_progress_text(value.get("message")) {
                track.message = message;
            }
            if let Some(percentage) = parse_progress_percentage(value.get("percentage")) {
                track.percentage = percentage;
            }
            Some(LspNotification::new(
                progress_notification_key(server_id, &token),
                server_id,
                LspNotificationLevel::Info,
                format!("LSP · {server_id}"),
                progress_body_lines(track),
                Some(LspNotificationProgress::new(track.percentage)),
                true,
            ))
        }
        "end" => {
            let mut track = progress_tracks.remove(&token).unwrap_or_default();
            if let Some(message) = parse_optional_progress_text(value.get("message")) {
                track.message = message;
            }
            Some(LspNotification::new(
                progress_notification_key(server_id, &token),
                server_id,
                completion_level_for_message(track.message.as_deref()),
                format!("LSP · {server_id}"),
                progress_body_lines(&track),
                track
                    .percentage
                    .map(|percentage| LspNotificationProgress::new(Some(percentage))),
                false,
            ))
        }
        _ => None,
    }
}

fn parse_show_message_notification(server_id: &str, params: &Value) -> Option<LspNotification> {
    let level = match params.get("type").and_then(Value::as_u64) {
        Some(1) => LspNotificationLevel::Error,
        Some(2) => LspNotificationLevel::Warning,
        Some(3) | Some(4) => LspNotificationLevel::Info,
        _ => LspNotificationLevel::Info,
    };
    let message = params
        .get("message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())?
        .to_owned();
    Some(LspNotification::new(
        format!("message:{server_id}:{level:?}:{message}"),
        server_id,
        level,
        format!("LSP · {server_id}"),
        vec![message],
        None,
        false,
    ))
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

fn parse_signature_help_response(
    server_id: &str,
    value: &Value,
) -> Result<Option<LspSignatureHelpContents>, LspClientError> {
    let signature_help =
        serde_json::from_value::<Option<SignatureHelp>>(value.clone()).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to decode signature help response from `{server_id}`: {error}"
            ))
        })?;
    let Some(signature_help) = signature_help else {
        return Ok(None);
    };
    let lines = signature_help_lines(&signature_help);
    Ok((!lines.is_empty()).then(|| LspSignatureHelpContents::new(server_id, lines)))
}

fn parse_definition_response(
    server_id: &str,
    value: &Value,
) -> Result<Vec<LspLocation>, LspClientError> {
    if value.is_null() {
        return Ok(Vec::new());
    }
    let response =
        serde_json::from_value::<GotoDefinitionResponse>(value.clone()).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to decode location response from `{server_id}`: {error}"
            ))
        })?;
    Ok(match response {
        GotoDefinitionResponse::Scalar(location) => location_from_lsp(server_id, &location)
            .into_iter()
            .collect(),
        GotoDefinitionResponse::Array(locations) => locations
            .iter()
            .filter_map(|location| location_from_lsp(server_id, location))
            .collect(),
        GotoDefinitionResponse::Link(links) => links
            .iter()
            .filter_map(|link| location_from_link(server_id, link))
            .collect(),
    })
}

fn parse_reference_response(
    server_id: &str,
    value: &Value,
) -> Result<Vec<LspLocation>, LspClientError> {
    if value.is_null() {
        return Ok(Vec::new());
    }
    let locations = serde_json::from_value::<Vec<Location>>(value.clone()).map_err(|error| {
        LspClientError::Protocol(format!(
            "failed to decode reference response from `{server_id}`: {error}"
        ))
    })?;
    Ok(locations
        .iter()
        .filter_map(|location| location_from_lsp(server_id, location))
        .collect())
}

fn parse_text_edit_response(
    server_id: &str,
    format_kind: &str,
    value: &Value,
) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
    let edits =
        serde_json::from_value::<Option<Vec<TextEdit>>>(value.clone()).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to decode {format_kind} response from `{server_id}`: {error}"
            ))
        })?;
    Ok(edits.map(|edits| edits.iter().map(lsp_text_edit_from_lsp).collect::<Vec<_>>()))
}

fn parse_code_action_response(
    server_id: &str,
    value: &Value,
) -> Result<Vec<LspCodeAction>, LspClientError> {
    if value.is_null() {
        return Ok(Vec::new());
    }
    let Some(actions) = value.as_array() else {
        return Err(LspClientError::Protocol(format!(
            "failed to decode code action response from `{server_id}`: expected an array"
        )));
    };
    Ok(actions
        .iter()
        .filter_map(|action| parse_code_action_item(server_id, action))
        .collect())
}

fn parse_code_action_item(server_id: &str, value: &Value) -> Option<LspCodeAction> {
    let title = value.get("title")?.as_str()?.trim();
    if title.is_empty() {
        return None;
    }
    let kind = value.get("kind").and_then(Value::as_str).map(str::to_owned);
    let disabled_reason = value
        .get("disabled")
        .and_then(|disabled| disabled.get("reason"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let preferred = value
        .get("isPreferred")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let (document_edits, has_resource_operations) =
        parse_code_action_workspace_edit(value.get("edit"));
    let command_name = parse_code_action_command_name(value);
    Some(LspCodeAction::new(
        server_id,
        title,
        kind,
        disabled_reason,
        preferred,
        document_edits,
        command_name,
        has_resource_operations,
    ))
}

fn parse_code_action_command_name(value: &Value) -> Option<String> {
    match value.get("command") {
        Some(Value::String(command)) => Some(command.to_owned()),
        Some(Value::Object(command)) => command
            .get("command")
            .and_then(Value::as_str)
            .map(str::to_owned),
        _ => None,
    }
}

fn parse_code_action_workspace_edit(value: Option<&Value>) -> (Vec<LspDocumentTextEdits>, bool) {
    let Some(value) = value else {
        return (Vec::new(), false);
    };
    let mut document_edits = Vec::new();
    let mut has_resource_operations = false;

    if let Some(changes) = value.get("changes").and_then(Value::as_object) {
        for (uri, edits_value) in changes {
            let Some(path) = file_uri_to_path(uri) else {
                continue;
            };
            let edits = parse_inline_text_edits(edits_value);
            if edits.is_empty() {
                continue;
            }
            document_edits.push(LspDocumentTextEdits::new(path, edits));
        }
    }

    if let Some(changes) = value.get("documentChanges").and_then(Value::as_array) {
        for change in changes {
            if let Some(document_edit) = parse_code_action_document_change(change) {
                document_edits.push(document_edit);
            } else if change.get("kind").is_some() {
                has_resource_operations = true;
            }
        }
    }

    (document_edits, has_resource_operations)
}

fn parse_code_action_document_change(value: &Value) -> Option<LspDocumentTextEdits> {
    let path = value
        .get("textDocument")
        .and_then(|text_document| text_document.get("uri"))
        .and_then(Value::as_str)
        .and_then(file_uri_to_path)?;
    let edits = parse_inline_text_edits(value.get("edits")?);
    (!edits.is_empty()).then(|| LspDocumentTextEdits::new(path, edits))
}

fn parse_inline_text_edits(value: &Value) -> Vec<LspTextEdit> {
    value
        .as_array()
        .map(|edits| {
            edits
                .iter()
                .filter_map(parse_inline_text_edit)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_inline_text_edit(value: &Value) -> Option<LspTextEdit> {
    let range = parse_inline_text_range(value.get("range")?)?;
    let new_text = value.get("newText")?.as_str()?;
    Some(LspTextEdit::new(range, new_text))
}

fn parse_inline_text_range(value: &Value) -> Option<TextRange> {
    Some(TextRange::new(
        parse_inline_text_point(value.get("start")?)?,
        parse_inline_text_point(value.get("end")?)?,
    ))
}

fn parse_inline_text_point(value: &Value) -> Option<TextPoint> {
    let line = value.get("line").and_then(Value::as_u64)?;
    let character = value.get("character").and_then(Value::as_u64)?;
    Some(TextPoint::new(
        usize::try_from(line).ok()?,
        usize::try_from(character).ok()?,
    ))
}

fn diagnostic_matches_request_range(diagnostic_range: TextRange, request_range: TextRange) -> bool {
    let diagnostic_range = diagnostic_range.normalized();
    let request_range = request_range.normalized();
    if request_range.start() == request_range.end() {
        let point = request_range.start();
        return diagnostic_range.start() <= point && point <= diagnostic_range.end();
    }
    diagnostic_range.start() < request_range.end() && request_range.start() < diagnostic_range.end()
}

fn lsp_code_action_diagnostic(diagnostic: &Diagnostic) -> Value {
    json!({
        "range": lsp_range_from_text_range(diagnostic.range()),
        "severity": lsp_diagnostic_severity(diagnostic.severity()),
        "source": diagnostic.source(),
        "message": diagnostic.message(),
    })
}

fn lsp_diagnostic_severity(severity: DiagnosticSeverity) -> u8 {
    match severity {
        DiagnosticSeverity::Error => 1,
        DiagnosticSeverity::Warning => 2,
        DiagnosticSeverity::Information => 3,
    }
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

fn signature_help_lines(signature_help: &SignatureHelp) -> Vec<String> {
    if signature_help.signatures.is_empty() {
        return Vec::new();
    }
    let active_signature_index = signature_help
        .active_signature
        .map(|index| index as usize)
        .filter(|index| *index < signature_help.signatures.len())
        .unwrap_or(0);
    let active_signature = &signature_help.signatures[active_signature_index];
    let active_parameter_index = active_signature
        .active_parameter
        .or(signature_help.active_parameter)
        .map(|index| index as usize);
    let mut lines = vec![active_signature.label.clone()];
    if signature_help.signatures.len() > 1 {
        lines.push(format!(
            "Overload {}/{}",
            active_signature_index + 1,
            signature_help.signatures.len()
        ));
    }
    if let Some(parameter_label) =
        active_parameter_index.and_then(|index| active_parameter_label(active_signature, index))
    {
        lines.push(format!("Parameter: {parameter_label}"));
    }
    if let Some(parameter_documentation) = active_parameter_index
        .and_then(|index| {
            active_signature
                .parameters
                .as_ref()
                .and_then(|parameters| parameters.get(index))
        })
        .and_then(|parameter| parameter.documentation.as_ref())
    {
        lines.extend(documentation_lines(parameter_documentation));
    }
    if let Some(documentation) = active_signature.documentation.as_ref() {
        lines.extend(documentation_lines(documentation));
    }
    lines
}

fn active_parameter_label(
    signature: &lsp_types::SignatureInformation,
    active_parameter_index: usize,
) -> Option<String> {
    let parameter = signature.parameters.as_ref()?.get(active_parameter_index)?;
    parameter_label_text(&parameter.label, &signature.label)
}

fn parameter_label_text(label: &ParameterLabel, signature_label: &str) -> Option<String> {
    match label {
        ParameterLabel::Simple(text) => {
            let trimmed = text.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_owned())
        }
        ParameterLabel::LabelOffsets([start, end]) => {
            let label = string_slice_by_chars(signature_label, *start as usize, *end as usize)?
                .trim()
                .to_owned();
            (!label.is_empty()).then_some(label)
        }
    }
}

fn documentation_lines(documentation: &Documentation) -> Vec<String> {
    match documentation {
        Documentation::String(text) => normalize_lines(text),
        Documentation::MarkupContent(content) => normalize_lines(&content.value),
    }
}

fn string_slice_by_chars(text: &str, start: usize, end: usize) -> Option<&str> {
    if start > end {
        return None;
    }
    let start_byte = char_to_byte_offset(text, start)?;
    let end_byte = char_to_byte_offset(text, end)?;
    text.get(start_byte..end_byte)
}

fn char_to_byte_offset(text: &str, char_index: usize) -> Option<usize> {
    if char_index == text.chars().count() {
        return Some(text.len());
    }
    text.char_indices().nth(char_index).map(|(index, _)| index)
}

fn location_from_lsp(server_id: &str, location: &Location) -> Option<LspLocation> {
    let path = file_uri_to_path(location.uri.as_str())?;
    Some(LspLocation::new(
        server_id,
        path,
        text_range_from_lsp_range(&location.range),
    ))
}

fn location_from_link(server_id: &str, link: &LocationLink) -> Option<LspLocation> {
    let path = file_uri_to_path(link.target_uri.as_str())?;
    Some(LspLocation::new(
        server_id,
        path,
        text_range_from_lsp_range(&link.target_selection_range),
    ))
}

fn lsp_text_edit_from_lsp(edit: &TextEdit) -> LspTextEdit {
    LspTextEdit::new(
        text_range_from_lsp_range(&edit.range),
        edit.new_text.clone(),
    )
}

fn text_range_from_lsp_range(range: &lsp_types::Range) -> TextRange {
    TextRange::new(
        text_point_from_lsp_position(range.start),
        text_point_from_lsp_position(range.end),
    )
}

fn text_point_from_lsp_position(position: Position) -> TextPoint {
    TextPoint::new(position.line as usize, position.character as usize)
}

fn lsp_range_from_text_range(range: TextRange) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_position_from_text_point(range.start()),
        end: lsp_position_from_text_point(range.end()),
    }
}

fn lsp_position_from_text_point(point: TextPoint) -> Position {
    Position::new(point.line as u32, point.column as u32)
}

fn lsp_formatting_options(options: LspFormattingOptions) -> FormattingOptions {
    FormattingOptions {
        tab_size: options.tab_size(),
        insert_spaces: options.insert_spaces(),
        ..FormattingOptions::default()
    }
}

fn unsupported_lsp_request(error: &LspClientError) -> bool {
    let LspClientError::Protocol(message) = error else {
        return false;
    };
    let lower = message.to_ascii_lowercase();
    lower.contains("-32601")
        || lower.contains("method not found")
        || lower.contains("method not supported")
}

fn sort_locations(locations: &mut Vec<LspLocation>) {
    locations.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.range.start().line.cmp(&right.range.start().line))
            .then_with(|| left.range.start().column.cmp(&right.range.start().column))
            .then_with(|| left.range.end().line.cmp(&right.range.end().line))
            .then_with(|| left.range.end().column.cmp(&right.range.end().column))
    });
    locations.dedup_by(|left, right| left.path == right.path && left.range == right.range);
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

    #[cfg(windows)]
    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-fnm-lsp-{unique}"))
    }

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
    fn signature_help_parser_surfaces_active_parameter_and_docs() {
        let response = json!({
            "signatures": [
                {
                    "label": "do_thing(value: String)",
                    "documentation": "Fallback overload"
                },
                {
                    "label": "do_thing(value: String, count: usize)",
                    "documentation": {
                        "kind": "markdown",
                        "value": "Formats the value multiple times."
                    },
                    "parameters": [
                        {
                            "label": "value: String",
                            "documentation": "Value to format."
                        },
                        {
                            "label": "count: usize",
                            "documentation": {
                                "kind": "markdown",
                                "value": "Number of repetitions."
                            }
                        }
                    ],
                    "activeParameter": 1
                }
            ],
            "activeSignature": 1,
            "activeParameter": 0
        });
        let signature_help = parse_signature_help_response("rust-analyzer", &response)
            .expect("signature help response")
            .expect("signature help");
        assert_eq!(signature_help.server_id(), "rust-analyzer");
        assert_eq!(
            signature_help.lines()[0],
            "do_thing(value: String, count: usize)"
        );
        assert!(
            signature_help
                .lines()
                .iter()
                .any(|line| line == "Overload 2/2")
        );
        assert!(
            signature_help
                .lines()
                .iter()
                .any(|line| line == "Parameter: count: usize")
        );
        assert!(
            signature_help
                .lines()
                .iter()
                .any(|line| line.contains("Number of repetitions"))
        );
        assert!(
            signature_help
                .lines()
                .iter()
                .any(|line| line.contains("Formats the value multiple times"))
        );
    }

    #[test]
    fn signature_help_parser_supports_label_offsets() {
        let response = json!({
            "signatures": [
                {
                    "label": "call(alpha, beta)",
                    "parameters": [
                        {
                            "label": [5, 10]
                        },
                        {
                            "label": [12, 16]
                        }
                    ]
                }
            ],
            "activeSignature": 0,
            "activeParameter": 1
        });
        let signature_help = parse_signature_help_response("rust-analyzer", &response)
            .expect("signature help response")
            .expect("signature help");
        assert!(
            signature_help
                .lines()
                .iter()
                .any(|line| line == "Parameter: beta")
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

    #[cfg(windows)]
    #[test]
    fn file_uri_roundtrip_handles_windows_paths() {
        let path = PathBuf::from(r"P:\volt\src\main.rs");
        let uri = path_to_file_uri(&path);
        assert_eq!(file_uri_to_path(&uri), Some(path));
    }

    #[test]
    fn formatting_parser_maps_text_edits() {
        let response = json!([
            {
                "range": {
                    "start": { "line": 2, "character": 4 },
                    "end": { "line": 2, "character": 9 }
                },
                "newText": "value"
            }
        ]);

        let edits = parse_text_edit_response("rust-analyzer", "formatting", &response)
            .expect("formatting response")
            .expect("text edits");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range().start(), TextPoint::new(2, 4));
        assert_eq!(edits[0].range().end(), TextPoint::new(2, 9));
        assert_eq!(edits[0].new_text(), "value");
    }

    #[test]
    fn definition_parser_supports_location_links() {
        let response = json!([
            {
                "targetUri": "file:///P:/volt/src/lib.rs",
                "targetRange": {
                    "start": { "line": 10, "character": 0 },
                    "end": { "line": 12, "character": 1 }
                },
                "targetSelectionRange": {
                    "start": { "line": 11, "character": 4 },
                    "end": { "line": 11, "character": 10 }
                }
            }
        ]);

        let locations = parse_definition_response("rust-analyzer", &response).expect("locations");
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].server_id(), "rust-analyzer");
        assert!(
            locations[0]
                .path()
                .ends_with(Path::new("src").join("lib.rs"))
        );
        assert_eq!(locations[0].range().start(), TextPoint::new(11, 4));
        assert_eq!(locations[0].range().end(), TextPoint::new(11, 10));
    }

    #[test]
    fn location_sorting_deduplicates_reference_results() {
        let response = json!([
            {
                "uri": "file:///P:/volt/src/main.rs",
                "range": {
                    "start": { "line": 7, "character": 3 },
                    "end": { "line": 7, "character": 8 }
                }
            },
            {
                "uri": "file:///P:/volt/src/lib.rs",
                "range": {
                    "start": { "line": 2, "character": 1 },
                    "end": { "line": 2, "character": 6 }
                }
            },
            {
                "uri": "file:///P:/volt/src/main.rs",
                "range": {
                    "start": { "line": 7, "character": 3 },
                    "end": { "line": 7, "character": 8 }
                }
            }
        ]);

        let mut locations =
            parse_reference_response("rust-analyzer", &response).expect("locations");
        sort_locations(&mut locations);
        assert_eq!(locations.len(), 2);
        assert!(
            locations[0]
                .path()
                .ends_with(Path::new("src").join("lib.rs"))
        );
        assert!(
            locations[1]
                .path()
                .ends_with(Path::new("src").join("main.rs"))
        );
    }

    #[test]
    fn code_action_parser_collects_active_file_edits() {
        let response = json!([
            {
                "title": "Fix unused import",
                "kind": "quickfix",
                "isPreferred": true,
                "edit": {
                    "changes": {
                        "file:///P:/volt/src/main.rs": [
                            {
                                "range": {
                                    "start": { "line": 3, "character": 0 },
                                    "end": { "line": 4, "character": 0 }
                                },
                                "newText": ""
                            }
                        ]
                    }
                }
            }
        ]);

        let actions = parse_code_action_response("rust-analyzer", &response).expect("code actions");
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].server_id(), "rust-analyzer");
        assert_eq!(actions[0].title(), "Fix unused import");
        assert_eq!(actions[0].kind(), Some("quickfix"));
        assert!(actions[0].is_preferred());
        assert_eq!(actions[0].document_edits().len(), 1);
        assert!(
            actions[0].document_edits()[0]
                .path()
                .ends_with(Path::new("src").join("main.rs"))
        );
        assert_eq!(actions[0].document_edits()[0].edits().len(), 1);
        assert_eq!(
            actions[0].document_edits()[0].edits()[0].range().start(),
            TextPoint::new(3, 0)
        );
    }

    #[test]
    fn code_action_parser_tracks_command_and_resource_operations() {
        let response = json!([
            {
                "title": "Apply workspace fix",
                "kind": "quickfix",
                "disabled": {
                    "reason": "build script output is stale"
                },
                "command": {
                    "title": "Apply workspace edit",
                    "command": "rust-analyzer.applySourceChange"
                },
                "edit": {
                    "documentChanges": [
                        {
                            "textDocument": {
                                "uri": "file:///P:/volt/src/lib.rs",
                                "version": 3
                            },
                            "edits": [
                                {
                                    "range": {
                                        "start": { "line": 0, "character": 0 },
                                        "end": { "line": 0, "character": 0 }
                                    },
                                    "newText": "use std::fmt;\n"
                                }
                            ]
                        },
                        {
                            "kind": "rename",
                            "oldUri": "file:///P:/volt/src/old.rs",
                            "newUri": "file:///P:/volt/src/new.rs"
                        }
                    ]
                }
            },
            {
                "title": "Trigger organize imports",
                "command": "rust-analyzer.organizeImports"
            }
        ]);

        let actions = parse_code_action_response("rust-analyzer", &response).expect("code actions");
        assert_eq!(actions.len(), 2);
        assert_eq!(
            actions[0].disabled_reason(),
            Some("build script output is stale")
        );
        assert_eq!(
            actions[0].command_name(),
            Some("rust-analyzer.applySourceChange")
        );
        assert!(actions[0].has_resource_operations());
        assert_eq!(actions[0].document_edits().len(), 1);
        assert_eq!(
            actions[1].command_name(),
            Some("rust-analyzer.organizeImports")
        );
        assert!(actions[1].document_edits().is_empty());
    }

    #[test]
    fn point_requests_match_covering_diagnostics() {
        let point = TextPoint::new(4, 12);
        let range = TextRange::new(point, point);
        let diagnostic = TextRange::new(TextPoint::new(4, 0), TextPoint::new(4, 20));
        assert!(diagnostic_matches_request_range(diagnostic, range));
        assert!(!diagnostic_matches_request_range(
            TextRange::new(TextPoint::new(5, 0), TextPoint::new(5, 4)),
            range,
        ));
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

    #[test]
    fn notification_log_snapshot_is_bounded_and_tracks_revision() {
        let mut log = LspNotificationLog::new(2);
        log.record(LspNotification::new(
            "session:rust-analyzer:global",
            "rust-analyzer",
            LspNotificationLevel::Info,
            "LSP · rust-analyzer",
            vec!["Starting".to_owned()],
            None,
            true,
        ));
        log.record(LspNotification::new(
            "progress:rust-analyzer:token-1",
            "rust-analyzer",
            LspNotificationLevel::Info,
            "LSP · rust-analyzer",
            vec!["Indexing".to_owned()],
            Some(LspNotificationProgress::new(Some(25))),
            true,
        ));
        log.record(LspNotification::new(
            "session:rust-analyzer:global",
            "rust-analyzer",
            LspNotificationLevel::Success,
            "LSP · rust-analyzer",
            vec!["Ready".to_owned()],
            None,
            false,
        ));

        let snapshot = log.snapshot();
        assert_eq!(snapshot.revision(), 3);
        assert_eq!(snapshot.entries().len(), 2);
        assert_eq!(
            snapshot.entries()[0].notification().key(),
            "progress:rust-analyzer:token-1"
        );
        assert_eq!(
            snapshot.entries()[1].notification().level(),
            LspNotificationLevel::Success
        );
    }

    #[test]
    fn progress_notifications_update_existing_track() {
        let begin = json!({
            "token": "rust-analyzer/index",
            "value": {
                "kind": "begin",
                "title": "Indexing",
                "message": "Scanning workspace",
                "percentage": 12
            }
        });
        let report = json!({
            "token": "rust-analyzer/index",
            "value": {
                "kind": "report",
                "message": "Building symbol graph",
                "percentage": 58
            }
        });
        let end = json!({
            "token": "rust-analyzer/index",
            "value": {
                "kind": "end",
                "message": "Indexed workspace"
            }
        });
        let mut tracks = BTreeMap::new();

        let begin = parse_progress_notification("rust-analyzer", &begin, &mut tracks)
            .expect("begin progress notification");
        assert!(begin.active());
        assert_eq!(begin.body_lines(), ["Indexing", "Scanning workspace"]);
        assert_eq!(
            begin
                .progress()
                .and_then(LspNotificationProgress::percentage),
            Some(12)
        );

        let report = parse_progress_notification("rust-analyzer", &report, &mut tracks)
            .expect("report progress notification");
        assert!(report.active());
        assert_eq!(report.body_lines(), ["Indexing", "Building symbol graph"]);
        assert_eq!(
            report
                .progress()
                .and_then(LspNotificationProgress::percentage),
            Some(58)
        );

        let end = parse_progress_notification("rust-analyzer", &end, &mut tracks)
            .expect("end progress notification");
        assert!(!end.active());
        assert_eq!(
            end.progress().and_then(LspNotificationProgress::percentage),
            Some(58)
        );
        assert_eq!(end.body_lines(), ["Indexing", "Indexed workspace"]);
        assert!(tracks.is_empty());
    }

    #[test]
    fn client_capabilities_enable_window_work_done_progress() {
        let capabilities = client_capabilities().expect("client capabilities");
        assert_eq!(
            capabilities
                .window
                .and_then(|window| window.work_done_progress),
            Some(true)
        );
    }

    #[test]
    fn work_done_progress_params_generate_unique_tokens() {
        let next_progress_token = std::sync::atomic::AtomicU64::new(1);
        let hover = work_done_progress_params(&next_progress_token, HoverRequest::METHOD);
        let signature =
            work_done_progress_params(&next_progress_token, SignatureHelpRequest::METHOD);

        assert_eq!(
            hover.work_done_token,
            Some(lsp_types::NumberOrString::String(format!(
                "progress:{}:1",
                HoverRequest::METHOD
            )))
        );
        assert_eq!(
            signature.work_done_token,
            Some(lsp_types::NumberOrString::String(format!(
                "progress:{}:2",
                SignatureHelpRequest::METHOD
            )))
        );
    }

    #[test]
    fn initialize_request_timeout_is_extended() {
        assert_eq!(
            request_timeout_for_method(Initialize::METHOD),
            INITIALIZE_REQUEST_TIMEOUT
        );
        assert_eq!(
            request_timeout_for_method(HoverRequest::METHOD),
            REQUEST_TIMEOUT
        );
    }

    #[test]
    fn session_labels_ignore_stale_tracked_session_keys() {
        let manager = LspClientManager::new(LanguageServerRegistry::new());
        let path = PathBuf::from("src\\main.rs");
        let mut tracked = TrackedBufferState {
            revision: 1,
            version: 1,
            ..TrackedBufferState::default()
        };
        tracked.sessions.insert(SessionKey {
            server_id: "rust-analyzer".to_owned(),
            root: None,
        });
        manager
            .state
            .lock()
            .expect("state lock")
            .tracked_buffers
            .insert(path.clone(), tracked);

        assert!(manager.session_labels_for_path(&path).is_empty());
        assert!(!manager.has_live_sessions_for_path(&path));
    }

    #[cfg(windows)]
    #[test]
    fn windows_launch_program_candidates_include_command_shims() {
        let candidates = windows_launch_program_candidates("vscode-json-language-server");
        assert!(candidates.contains(&"vscode-json-language-server.cmd".to_owned()));
    }

    #[cfg(windows)]
    #[test]
    fn windows_parse_cmd_environment_extracts_variables() {
        let env = parse_windows_cmd_environment(
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
    fn windows_fnm_environment_keeps_fnm_path_ahead_of_explicit_path() {
        let command = build_lsp_command(
            "node",
            &["--version".to_owned()],
            None,
            &[
                ("PATH".to_owned(), "C:\\custom".to_owned()),
                ("NODE_OPTIONS".to_owned(), "--trace-warnings".to_owned()),
            ],
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
            .collect::<BTreeMap<_, _>>();
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
        assert!(windows_should_retry_spawn_error(&error));
    }

    #[cfg(windows)]
    #[test]
    fn windows_fnm_launch_program_candidates_resolve_absolute_command_shims() {
        let temp_dir = temp_dir();
        std::fs::create_dir_all(&temp_dir).expect("temp dir");
        let candidate_path = temp_dir.join("vscode-json-language-server.cmd");
        std::fs::write(&candidate_path, "@echo off\r\n").expect("candidate");

        let candidates = windows_fnm_launch_program_candidates(
            "vscode-json-language-server",
            &[("PATH".to_owned(), temp_dir.to_string_lossy().into_owned())],
        );
        assert!(candidates.contains(&candidate_path.to_string_lossy().into_owned()));

        let _ = std::fs::remove_file(candidate_path);
        let _ = std::fs::remove_dir(temp_dir);
    }

    #[cfg(windows)]
    #[test]
    fn windows_fnm_launch_program_candidates_prefer_windows_shims_over_extensionless_scripts() {
        let temp_dir = temp_dir();
        std::fs::create_dir_all(&temp_dir).expect("temp dir");
        let script_path = temp_dir.join("typescript-language-server");
        let shim_path = temp_dir.join("typescript-language-server.cmd");
        std::fs::write(&script_path, "#!/bin/sh\n").expect("script");
        std::fs::write(&shim_path, "@echo off\r\n").expect("shim");

        let candidates = windows_fnm_launch_program_candidates(
            "typescript-language-server",
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

    #[test]
    fn show_message_notifications_map_severity_levels() {
        let params = json!({
            "type": 1,
            "message": "failed to load workspace"
        });
        let notification =
            parse_show_message_notification("rust-analyzer", &params).expect("notification");
        assert_eq!(notification.level(), LspNotificationLevel::Error);
        assert_eq!(notification.body_lines(), ["failed to load workspace"]);
        assert!(!notification.active());
    }
}
