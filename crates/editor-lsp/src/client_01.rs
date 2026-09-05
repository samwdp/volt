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
use editor_jobs::{ProcessSupervisionMode, supervised_command_if_resolved};
use lsp_types::{
    ClientCapabilities, ClientInfo, CodeActionContext, CodeActionParams, CodeActionTriggerKind,
    CompletionParams, Diagnostic as LspDiagnostic, DiagnosticSeverity as LspDiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentFormattingParams, DocumentRangeFormattingParams,
    Documentation, FormattingOptions, GotoDefinitionParams, GotoDefinitionResponse, HoverContents,
    HoverParams, InitializeParams, InitializeResult, InitializedParams, Location, LocationLink,
    MarkedString, MarkupKind, NumberOrString, ParameterLabel, PartialResultParams, Position, Range,
    ReferenceContext, ReferenceParams, SignatureHelp, SignatureHelpParams,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
    TraceValue, Uri, VersionedTextDocumentIdentifier, WorkDoneProgressParams, WorkspaceFolder,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Initialized, Notification,
    },
    request::{
        CodeActionRequest, Completion, Formatting, GotoDefinition, GotoImplementation,
        HoverRequest, Initialize, RangeFormatting, References, Request, SignatureHelpRequest,
    },
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerSession, LspError,
    LspWorkspaceDiagnostic,
};

const REQUEST_TIMEOUT: Duration = Duration::from_millis(400);
const INITIALIZE_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CODE_ACTION_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
const INLINE_COMPLETION_REQUEST_TIMEOUT: Duration = Duration::from_millis(1200);
const TRANSPORT_LOG_MAX_ENTRIES: usize = 400;
const NOTIFICATION_LOG_MAX_ENTRIES: usize = 128;
const COPILOT_SERVER_ID: &str = "copilot-language-server";
const CSHARP_SERVER_ID: &str = "csharp-ls";
const ROSLYN_LANGUAGE_SERVER_ID: &str = "roslyn-language-server";
const CSHARP_WORKSPACE_SECTION: &str = "csharp";
const CSHARP_METADATA_REQUEST_METHOD: &str = "csharp/metadata";
const INLINE_COMPLETION_METHOD: &str = "textDocument/inlineCompletion";

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

type PendingResponseTx = Sender<Result<Value, LspClientError>>;
type PendingResponseMap = Arc<Mutex<BTreeMap<u64, PendingResponseTx>>>;
type DiagnosticsByPath = Arc<Mutex<BTreeMap<PathBuf, Vec<Diagnostic>>>>;
type TransportLog = Arc<Mutex<LspTransportLog>>;
type NotificationLog = Arc<Mutex<LspNotificationLog>>;

#[derive(Clone)]
struct LspSessionSharedState {
    transport_log: TransportLog,
    notifications: NotificationLog,
    diagnostics_generation: Arc<AtomicU64>,
    dirty_diagnostic_paths: Arc<Mutex<BTreeSet<PathBuf>>>,
    sessions_generation: Arc<AtomicU64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspCompletionItem {
    server_id: String,
    kind: Option<LspCompletionKind>,
    label: String,
    insert_text: String,
    edit_range: Option<TextRange>,
    detail: Option<String>,
    documentation: Option<String>,
    has_documentation: bool,
    raw_item: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspInlineCompletionItem {
    server_id: String,
    root: Option<PathBuf>,
    insert_text: String,
    range: TextRange,
    raw_item: Value,
}

impl LspInlineCompletionItem {
    fn new(
        server_id: impl Into<String>,
        root: Option<PathBuf>,
        insert_text: impl Into<String>,
        range: TextRange,
        raw_item: Value,
    ) -> Self {
        Self {
            server_id: server_id.into(),
            root,
            insert_text: insert_text.into(),
            range,
            raw_item,
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn insert_text(&self) -> &str {
        &self.insert_text
    }

    pub const fn range(&self) -> TextRange {
        self.range
    }
}

impl LspCompletionItem {
    fn new(
        server_id: impl Into<String>,
        kind: Option<LspCompletionKind>,
        label: impl Into<String>,
        insert_text: impl Into<String>,
        edit_range: Option<TextRange>,
        detail: Option<String>,
        documentation: Option<String>,
    ) -> Self {
        Self {
            server_id: server_id.into(),
            kind,
            label: label.into(),
            insert_text: insert_text.into(),
            edit_range,
            detail,
            documentation,
            has_documentation: false,
            raw_item: Value::Null,
        }
    }

    fn with_raw_item(mut self, raw_item: Value, has_documentation: bool) -> Self {
        self.raw_item = raw_item;
        self.has_documentation = has_documentation;
        self
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

    pub const fn edit_range(&self) -> Option<TextRange> {
        self.edit_range
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
    text: String,
    lines: Vec<String>,
    markdown: bool,
}

impl LspHoverContents {
    fn new(
        server_id: impl Into<String>,
        text: impl Into<String>,
        lines: Vec<String>,
        markdown: bool,
    ) -> Self {
        Self {
            server_id: server_id.into(),
            text: text.into(),
            lines,
            markdown,
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub const fn is_markdown(&self) -> bool {
        self.markdown
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspSignatureActiveParameter {
    pub signature_index: usize,
    pub label: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspSignatureHelpContents {
    server_id: String,
    signature_help: SignatureHelp,
}

impl LspSignatureHelpContents {
    fn new(server_id: impl Into<String>, signature_help: SignatureHelp) -> Self {
        Self {
            server_id: server_id.into(),
            signature_help,
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn markdown_text(&self, language: Option<&str>) -> String {
        signature_help_markdown(&self.signature_help, language).unwrap_or_default()
    }

    pub fn active_parameter_range(&self) -> Option<LspSignatureActiveParameter> {
        let active_signature_index = self
            .signature_help
            .active_signature
            .map(|index| index as usize)
            .filter(|index| *index < self.signature_help.signatures.len())
            .unwrap_or(0);
        let active_signature = self.signature_help.signatures.get(active_signature_index)?;
        let active_parameter_index = active_signature
            .active_parameter
            .or(self.signature_help.active_parameter)
            .map(|index| index as usize)?;
        let (start, end) = active_parameter_char_range(active_signature, active_parameter_index)?;
        Some(LspSignatureActiveParameter {
            signature_index: active_signature_index,
            label: active_signature.label.clone(),
            start,
            end,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspLocation {
    server_id: String,
    path: PathBuf,
    file_path: Option<PathBuf>,
    uri: String,
    range: TextRange,
}

impl LspLocation {
    fn from_uri(server_id: impl Into<String>, uri: impl Into<String>, range: TextRange) -> Self {
        let uri = uri.into();
        let file_path = file_uri_to_path(&uri);
        let path = file_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(uri.clone()));
        Self {
            server_id: server_id.into(),
            path,
            file_path,
            uri,
            range,
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn is_file_path(&self) -> bool {
        self.file_path.is_some()
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

/// UI action attached to an LSP notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspNotificationAction {
    /// Starts Copilot device authentication for the emitting session.
    CopilotSignIn,
    /// Opens the provided URL in Volt's browser popup.
    OpenBrowserPopup { url: String },
}

/// Executable LSP command returned by a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspServerCommand {
    title: String,
    command: String,
    arguments: Vec<Value>,
}

impl LspServerCommand {
    fn new(title: impl Into<String>, command: impl Into<String>, arguments: Vec<Value>) -> Self {
        Self {
            title: title.into(),
            command: command.into(),
            arguments,
        }
    }

    /// Returns the label the server suggests for this command.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the command identifier to execute.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Returns the JSON arguments supplied by the server.
    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }
}

/// Copilot device-flow prompt returned by `signIn`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopilotDeviceCodePrompt {
    user_code: String,
    command: LspServerCommand,
}

impl CopilotDeviceCodePrompt {
    fn new(user_code: impl Into<String>, command: LspServerCommand) -> Self {
        Self {
            user_code: user_code.into(),
            command,
        }
    }

    /// Returns the device code the user must enter in GitHub's auth page.
    pub fn user_code(&self) -> &str {
        &self.user_code
    }

    /// Returns the command that continues the device flow.
    pub fn command(&self) -> &LspServerCommand {
        &self.command
    }
}

/// UI-facing LSP notification entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspNotification {
    key: String,
    server_id: String,
    root: Option<PathBuf>,
    level: LspNotificationLevel,
    title: String,
    body_lines: Vec<String>,
    progress: Option<LspNotificationProgress>,
    active: bool,
    action: Option<LspNotificationAction>,
}

impl LspNotification {
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the originating language server id.
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    /// Returns the session root that emitted the notification, if any.
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
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

    /// Returns the UI action attached to this notification, if any.
    pub fn action(&self) -> Option<&LspNotificationAction> {
        self.action.as_ref()
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

    const fn revision(&self) -> u64 {
        self.revision
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

    const fn revision(&self) -> u64 {
        self.revision
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

/// Identity of one live Language Server Session (server id + root).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LspLiveSession {
    server_id: String,
    root: Option<PathBuf>,
}

impl LspLiveSession {
    /// Creates a live Session identity.
    pub fn new(server_id: impl Into<String>, root: Option<PathBuf>) -> Self {
        Self {
            server_id: server_id.into(),
            root: normalize_session_root(root.as_deref()),
        }
    }

    /// Returns the language server id.
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    /// Returns the Session root path, if any.
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Picker label: `server-id — <root>` or `server-id — (no root)`.
    pub fn picker_label(&self) -> String {
        match self.root.as_deref() {
            Some(root) => format!("{} — {}", self.server_id, root.display()),
            None => format!("{} — (no root)", self.server_id),
        }
    }
}

/// Whether a live Session belongs in the active Workspace stop/restart picker.
///
/// Include when it serves an open buffer, or (Project Workspace only) when its
/// root equals or lies under the Project Workspace root.
pub fn language_server_session_in_workspace_scope(
    session_root: Option<&Path>,
    tracked_paths: &[PathBuf],
    open_buffer_paths: &[PathBuf],
    project_workspace_root: Option<&Path>,
) -> bool {
    let open = open_buffer_paths
        .iter()
        .map(|path| normalize_path_for_compare(path))
        .collect::<BTreeSet<_>>();
    if tracked_paths
        .iter()
        .any(|path| open.contains(&normalize_path_for_compare(path)))
    {
        return true;
    }
    let Some(workspace_root) = project_workspace_root else {
        return false;
    };
    let Some(session_root) = session_root else {
        return false;
    };
    path_equals_or_under(session_root, workspace_root)
}

fn normalize_path_for_compare(path: &Path) -> PathBuf {
    normalize_session_root(Some(path)).unwrap_or_else(|| path.to_path_buf())
}

fn path_equals_or_under(path: &Path, ancestor: &Path) -> bool {
    let path = normalize_path_for_compare(path);
    let ancestor = normalize_path_for_compare(ancestor);
    path == ancestor || path.starts_with(&ancestor)
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
    diagnostics_generation: Arc<AtomicU64>,
    dirty_diagnostic_paths: Arc<Mutex<BTreeSet<PathBuf>>>,
    sessions_generation: Arc<AtomicU64>,
    diagnostics_lookups: Arc<AtomicU64>,
}

#[derive(Debug, Default)]
struct LspClientState {
    sessions: BTreeMap<SessionKey, Arc<LspSessionHandle>>,
    tracked_buffers: BTreeMap<PathBuf, TrackedBufferState>,
    settings_overrides: BTreeMap<SessionKey, Value>,
    initialization_options_overrides: BTreeMap<SessionKey, Value>,
    start_failures: BTreeMap<SessionKey, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SessionKey {
    server_id: String,
    root: Option<PathBuf>,
}

impl SessionKey {
    fn new(server_id: impl Into<String>, root: Option<&Path>) -> Self {
        Self {
            server_id: server_id.into(),
            root: normalize_session_root(root),
        }
    }
}

fn normalize_session_root(root: Option<&Path>) -> Option<PathBuf> {
    root.map(|root| {
        #[cfg(windows)]
        {
            PathBuf::from(root.to_string_lossy().as_ref().to_ascii_lowercase())
        }
        #[cfg(not(windows))]
        {
            root.to_path_buf()
        }
    })
}

#[derive(Debug, Default, Clone)]
struct TrackedBufferState {
    revision: u64,
    version: i32,
    sessions: BTreeSet<SessionKey>,
}

#[derive(Debug, Clone)]
struct SessionWorkspaceConfiguration {
    section: Option<String>,
    base_settings: Option<Value>,
    runtime_override: Option<Value>,
}

impl SessionWorkspaceConfiguration {
    fn new(session: &LanguageServerSession, runtime_override: Option<Value>) -> Self {
        let section = workspace_configuration_section_for_session(session).map(str::to_owned);
        Self {
            base_settings: normalized_workspace_configuration_settings(
                section.as_deref(),
                session.workspace_configuration_settings_json(),
            ),
            runtime_override: normalized_workspace_configuration_settings(
                section.as_deref(),
                runtime_override,
            ),
            section,
        }
    }

    fn response_for_request(&self, params: Option<&Value>) -> Value {
        let Some(items) = params
            .and_then(|params| params.get("items"))
            .and_then(Value::as_array)
        else {
            return Value::Array(Vec::new());
        };
        Value::Array(
            items
                .iter()
                .map(|item| {
                    if configuration_item_section(item) == self.section.as_deref() {
                        self.effective_settings().unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    }
                })
                .collect(),
        )
    }

    fn set_runtime_override(&mut self, runtime_override: Option<Value>) -> bool {
        let previous = self.effective_settings();
        self.runtime_override =
            normalized_workspace_configuration_settings(self.section.as_deref(), runtime_override);
        self.effective_settings() != previous
    }

    fn did_change_configuration_payload(&self, include_null_section: bool) -> Option<Value> {
        match (self.section.as_deref(), self.effective_settings()) {
            (Some(section), Some(settings)) => {
                Some(wrap_workspace_configuration_settings(section, settings))
            }
            (Some(section), None) if include_null_section => {
                Some(wrap_workspace_configuration_settings(section, Value::Null))
            }
            (None, Some(settings)) => Some(settings),
            _ => None,
        }
    }

    fn effective_settings(&self) -> Option<Value> {
        effective_workspace_configuration_settings(
            self.base_settings.as_ref(),
            self.runtime_override.as_ref(),
        )
    }
}
