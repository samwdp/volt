use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::Sender,
    },
    thread,
    time::{Duration, SystemTime},
};

use editor_buffer::{TextPoint, TextRange};
use editor_jobs::{ProcessSupervisionMode, supervised_command_if_resolved};
use lsp_types::{
    ClientCapabilities, CodeActionContext, CodeActionParams, CodeActionTriggerKind,
    Diagnostic as LspDiagnostic, DiagnosticSeverity as LspDiagnosticSeverity, Documentation,
    FormattingOptions, GotoDefinitionResponse, HoverContents, Location, LocationLink, MarkedString,
    MarkupKind, NumberOrString, ParameterLabel, PartialResultParams, Position, Range,
    SignatureHelp, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Uri,
    WorkDoneProgressParams,
    request::{CodeActionRequest, Initialize, Request},
};
use serde_json::{Value, json};

use crate::{
    Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerSession, LspError,
};

use super::session::*;

pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_millis(400);

pub(crate) const INITIALIZE_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) const CODE_ACTION_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

pub(crate) const INLINE_COMPLETION_REQUEST_TIMEOUT: Duration = Duration::from_millis(1200);

pub(crate) const TRANSPORT_LOG_MAX_ENTRIES: usize = 400;

pub(crate) const NOTIFICATION_LOG_MAX_ENTRIES: usize = 128;

pub(crate) const COPILOT_SERVER_ID: &str = "copilot-language-server";

pub(crate) const CSHARP_SERVER_ID: &str = "csharp-ls";

pub(crate) const ROSLYN_LANGUAGE_SERVER_ID: &str = "roslyn-language-server";

pub(crate) const CSHARP_WORKSPACE_SECTION: &str = "csharp";

pub(crate) const CSHARP_METADATA_REQUEST_METHOD: &str = "csharp/metadata";

pub(crate) const INLINE_COMPLETION_METHOD: &str = "textDocument/inlineCompletion";

#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) type PendingResponseTx = Sender<Result<Value, LspClientError>>;

pub(crate) type PendingResponseMap = Arc<Mutex<BTreeMap<u64, PendingResponseTx>>>;

pub(crate) type DiagnosticsByPath = Arc<Mutex<BTreeMap<PathBuf, Vec<Diagnostic>>>>;

pub(crate) type TransportLog = Arc<Mutex<LspTransportLog>>;

pub(crate) type NotificationLog = Arc<Mutex<LspNotificationLog>>;

#[derive(Clone)]
pub(crate) struct LspSessionSharedState {
    pub(crate) transport_log: TransportLog,
    pub(crate) notifications: NotificationLog,
    pub(crate) diagnostics_generation: Arc<AtomicU64>,
    pub(crate) dirty_diagnostic_paths: Arc<Mutex<BTreeSet<PathBuf>>>,
    pub(crate) sessions_generation: Arc<AtomicU64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspCompletionItem {
    pub(crate) server_id: String,
    pub(crate) kind: Option<LspCompletionKind>,
    pub(crate) label: String,
    pub(crate) insert_text: String,
    pub(crate) edit_range: Option<TextRange>,
    pub(crate) detail: Option<String>,
    pub(crate) documentation: Option<String>,
    pub(crate) has_documentation: bool,
    pub(crate) raw_item: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspInlineCompletionItem {
    pub(crate) server_id: String,
    pub(crate) root: Option<PathBuf>,
    pub(crate) insert_text: String,
    pub(crate) range: TextRange,
    pub(crate) raw_item: Value,
}

impl LspInlineCompletionItem {
    pub(crate) fn new(
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
    pub(crate) fn new(
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

    pub(crate) fn with_raw_item(mut self, raw_item: Value, has_documentation: bool) -> Self {
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
    pub(crate) server_id: String,
    pub(crate) text: String,
    pub(crate) lines: Vec<String>,
    pub(crate) markdown: bool,
}

impl LspHoverContents {
    pub(crate) fn new(
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
    pub(crate) server_id: String,
    pub(crate) signature_help: SignatureHelp,
}

impl LspSignatureHelpContents {
    pub(crate) fn new(server_id: impl Into<String>, signature_help: SignatureHelp) -> Self {
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
    pub(crate) server_id: String,
    pub(crate) path: PathBuf,
    pub(crate) file_path: Option<PathBuf>,
    pub(crate) uri: String,
    pub(crate) range: TextRange,
}

impl LspLocation {
    pub(crate) fn from_uri(
        server_id: impl Into<String>,
        uri: impl Into<String>,
        range: TextRange,
    ) -> Self {
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
    pub(crate) range: TextRange,
    pub(crate) new_text: String,
}

impl LspTextEdit {
    pub(crate) fn new(range: TextRange, new_text: impl Into<String>) -> Self {
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
    pub(crate) path: PathBuf,
    pub(crate) edits: Vec<LspTextEdit>,
}

impl LspDocumentTextEdits {
    pub(crate) fn new(path: PathBuf, edits: Vec<LspTextEdit>) -> Self {
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
    pub(crate) server_id: String,
    pub(crate) title: String,
    pub(crate) kind: Option<String>,
    pub(crate) disabled_reason: Option<String>,
    pub(crate) preferred: bool,
    pub(crate) document_edits: Vec<LspDocumentTextEdits>,
    pub(crate) command_name: Option<String>,
    pub(crate) has_resource_operations: bool,
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
    pub(crate) tab_size: u32,
    pub(crate) insert_spaces: bool,
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
    pub(crate) timestamp: SystemTime,
    pub(crate) server_id: String,
    pub(crate) direction: LspLogDirection,
    pub(crate) body: String,
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
    pub(crate) revision: u64,
    pub(crate) entries: Vec<LspLogEntry>,
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
    pub(crate) percentage: Option<u32>,
}

impl LspNotificationProgress {
    pub(crate) fn new(percentage: Option<u32>) -> Self {
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
    pub(crate) title: String,
    pub(crate) command: String,
    pub(crate) arguments: Vec<Value>,
}

impl LspServerCommand {
    pub(crate) fn new(
        title: impl Into<String>,
        command: impl Into<String>,
        arguments: Vec<Value>,
    ) -> Self {
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
    pub(crate) user_code: String,
    pub(crate) command: LspServerCommand,
}

impl CopilotDeviceCodePrompt {
    pub(crate) fn new(user_code: impl Into<String>, command: LspServerCommand) -> Self {
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
    pub(crate) key: String,
    pub(crate) server_id: String,
    pub(crate) root: Option<PathBuf>,
    pub(crate) level: LspNotificationLevel,
    pub(crate) title: String,
    pub(crate) body_lines: Vec<String>,
    pub(crate) progress: Option<LspNotificationProgress>,
    pub(crate) active: bool,
    pub(crate) action: Option<LspNotificationAction>,
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
    pub(crate) revision: u64,
    pub(crate) notification: LspNotification,
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
    pub(crate) revision: u64,
    pub(crate) entries: Vec<LspNotificationEntry>,
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
pub(crate) struct LspTransportLog {
    pub(crate) revision: u64,
    pub(crate) entries: Vec<LspLogEntry>,
    pub(crate) max_entries: usize,
}

impl LspTransportLog {
    pub(crate) fn new(max_entries: usize) -> Self {
        Self {
            revision: 0,
            entries: Vec::new(),
            max_entries,
        }
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn record(&mut self, entry: LspLogEntry) {
        self.revision = self.revision.saturating_add(1);
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            let overflow = self.entries.len() - self.max_entries;
            self.entries.drain(0..overflow);
        }
    }

    pub(crate) fn snapshot(&self) -> LspLogSnapshot {
        LspLogSnapshot {
            revision: self.revision,
            entries: self.entries.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct LspNotificationLog {
    pub(crate) revision: u64,
    pub(crate) entries: Vec<LspNotificationEntry>,
    pub(crate) max_entries: usize,
}

impl LspNotificationLog {
    pub(crate) fn new(max_entries: usize) -> Self {
        Self {
            revision: 0,
            entries: Vec::new(),
            max_entries,
        }
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn record(&mut self, notification: LspNotification) {
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

    pub(crate) fn snapshot(&self) -> LspNotificationSnapshot {
        LspNotificationSnapshot {
            revision: self.revision,
            entries: self.entries.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ProgressTrack {
    pub(crate) title: Option<String>,
    pub(crate) message: Option<String>,
    pub(crate) percentage: Option<u32>,
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
    pub(crate) server_id: String,
    pub(crate) root: Option<PathBuf>,
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

pub(crate) fn normalize_path_for_compare(path: &Path) -> PathBuf {
    normalize_session_root(Some(path)).unwrap_or_else(|| path.to_path_buf())
}

pub(crate) fn path_equals_or_under(path: &Path, ancestor: &Path) -> bool {
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
    pub(crate) registry: LanguageServerRegistry,
    pub(crate) state: Arc<Mutex<LspClientState>>,
    pub(crate) transport_log: TransportLog,
    pub(crate) notifications: NotificationLog,
    pub(crate) diagnostics_generation: Arc<AtomicU64>,
    pub(crate) dirty_diagnostic_paths: Arc<Mutex<BTreeSet<PathBuf>>>,
    pub(crate) sessions_generation: Arc<AtomicU64>,
    pub(crate) diagnostics_lookups: Arc<AtomicU64>,
}

#[derive(Debug, Default)]
pub(crate) struct LspClientState {
    pub(crate) sessions: BTreeMap<SessionKey, Arc<LspSessionHandle>>,
    pub(crate) tracked_buffers: BTreeMap<PathBuf, TrackedBufferState>,
    pub(crate) settings_overrides: BTreeMap<SessionKey, Value>,
    pub(crate) initialization_options_overrides: BTreeMap<SessionKey, Value>,
    pub(crate) start_failures: BTreeMap<SessionKey, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SessionKey {
    pub(crate) server_id: String,
    pub(crate) root: Option<PathBuf>,
}

impl SessionKey {
    pub(crate) fn new(server_id: impl Into<String>, root: Option<&Path>) -> Self {
        Self {
            server_id: server_id.into(),
            root: normalize_session_root(root),
        }
    }
}

pub(crate) fn normalize_session_root(root: Option<&Path>) -> Option<PathBuf> {
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
pub(crate) struct TrackedBufferState {
    pub(crate) revision: u64,
    pub(crate) version: i32,
    pub(crate) sessions: BTreeSet<SessionKey>,
}

#[derive(Debug, Clone)]
pub(crate) struct SessionWorkspaceConfiguration {
    pub(crate) section: Option<String>,
    pub(crate) base_settings: Option<Value>,
    pub(crate) runtime_override: Option<Value>,
}

impl SessionWorkspaceConfiguration {
    pub(crate) fn new(session: &LanguageServerSession, runtime_override: Option<Value>) -> Self {
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

    pub(crate) fn response_for_request(&self, params: Option<&Value>) -> Value {
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

    pub(crate) fn set_runtime_override(&mut self, runtime_override: Option<Value>) -> bool {
        let previous = self.effective_settings();
        self.runtime_override =
            normalized_workspace_configuration_settings(self.section.as_deref(), runtime_override);
        self.effective_settings() != previous
    }

    pub(crate) fn did_change_configuration_payload(
        &self,
        include_null_section: bool,
    ) -> Option<Value> {
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

    pub(crate) fn effective_settings(&self) -> Option<Value> {
        effective_workspace_configuration_settings(
            self.base_settings.as_ref(),
            self.runtime_override.as_ref(),
        )
    }
}

pub(crate) struct LspReaderSession {
    pub(crate) server_id: String,
    pub(crate) root: Option<PathBuf>,
    pub(crate) writer: Arc<Mutex<ChildStdin>>,
    pub(crate) pending: PendingResponseMap,
    pub(crate) diagnostics: DiagnosticsByPath,
    pub(crate) workspace_configuration: Arc<Mutex<SessionWorkspaceConfiguration>>,
    pub(crate) disconnected: Arc<AtomicBool>,
    pub(crate) transport_log: TransportLog,
    pub(crate) notifications: NotificationLog,
    pub(crate) diagnostics_generation: Arc<AtomicU64>,
    pub(crate) dirty_diagnostic_paths: Arc<Mutex<BTreeSet<PathBuf>>>,
    pub(crate) sessions_generation: Arc<AtomicU64>,
}

pub(crate) fn spawn_reader_thread(stdout: impl Read + Send + 'static, session: LspReaderSession) {
    thread::spawn(move || {
        let LspReaderSession {
            server_id,
            root,
            writer,
            pending,
            diagnostics,
            workspace_configuration,
            disconnected,
            transport_log,
            notifications,
            diagnostics_generation,
            dirty_diagnostic_paths,
            sessions_generation,
        } = session;
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
                let workspace_configuration = workspace_configuration
                    .lock()
                    .ok()
                    .map(|workspace_configuration| workspace_configuration.clone());
                let handling = server_request_response(
                    &server_id,
                    root.as_deref(),
                    object.get("method"),
                    object.get("params"),
                    workspace_configuration.as_ref(),
                );
                if let Some(notification) = handling.notification {
                    record_notification(&notifications, notification);
                }
                let id = object.get("id").cloned().unwrap_or(Value::Null);
                let response_message = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": handling.result,
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
                {
                    record_published_diagnostics(
                        &diagnostics,
                        &dirty_diagnostic_paths,
                        &diagnostics_generation,
                        path,
                        parsed,
                    );
                    continue;
                }
                if method == "$/progress"
                    && let Some(params) = object.get("params")
                    && let Some(notification) = parse_progress_notification(
                        &server_id,
                        root.as_deref(),
                        params,
                        &mut progress_tracks,
                    )
                {
                    record_notification(&notifications, notification);
                    continue;
                }
                if matches!(method, "window/showMessage" | "window/logMessage")
                    && let Some(params) = object.get("params")
                    && let Some(notification) = parse_window_message_notification(
                        method,
                        &server_id,
                        root.as_deref(),
                        params,
                    )
                {
                    record_notification(&notifications, notification);
                    continue;
                }
                if method == "didChangeStatus"
                    && let Some(params) = object.get("params")
                    && let Some(notification) =
                        parse_copilot_status_notification(&server_id, root.as_deref(), params)
                {
                    record_notification(&notifications, notification);
                    continue;
                }
            }
        }
        disconnected.store(true, Ordering::Release);
        note_session_disconnect_diagnostics(&diagnostics, &dirty_diagnostic_paths);
        diagnostics_generation.fetch_add(1, Ordering::Release);
        sessions_generation.fetch_add(1, Ordering::Release);
        record_transport_event(&transport_log, &server_id, "marked session disconnected");
        if let Ok(mut pending) = pending.lock() {
            for sender in pending.values() {
                let _ = sender.send(Err(LspClientError::Disconnected(server_id.clone())));
            }
            pending.clear();
        }
    });
}

pub(crate) fn configure_lsp_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

pub(crate) fn spawn_lsp_command(
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
        let should_retry_with_nvm = matches!(
            &spawn_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry_with_nvm && let Some(nvm_env) = windows_nvm_environment(cwd, env) {
            for candidate in windows_nvm_launch_program_candidates(program, &nvm_env) {
                spawn_result =
                    build_lsp_command(&candidate, args, cwd, env, Some(&nvm_env)).spawn();
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

pub(crate) fn build_lsp_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    #[cfg(windows)] runtime_env: Option<&[(String, String)]>,
    #[cfg(not(windows))] _runtime_env: Option<&[(String, String)]>,
) -> Command {
    let (program, args) = supervised_command_if_resolved(
        program,
        args,
        env,
        #[cfg(windows)]
        runtime_env,
        #[cfg(not(windows))]
        None,
        ProcessSupervisionMode::Background,
    );
    let mut command = Command::new(&program);
    configure_lsp_command(&mut command);
    command
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let mut env = env.to_vec();
    editor_tool_install::merge_effective_path(&mut env);
    #[cfg(windows)]
    if let Some(runtime_env) = runtime_env {
        apply_windows_runtime_environment(&mut command, &env, runtime_env);
    } else {
        apply_command_environment(&mut command, &env);
    }
    #[cfg(not(windows))]
    apply_command_environment(&mut command, &env);
    command
}

pub(crate) fn apply_command_environment(command: &mut Command, env: &[(String, String)]) {
    for (key, value) in env {
        command.env(key, value);
    }
}

#[cfg(windows)]
pub(crate) fn windows_launch_program_candidates(program: &str) -> Vec<String> {
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
pub(crate) fn windows_should_retry_spawn_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(193)
}

#[cfg(windows)]
pub(crate) fn windows_command_extensions() -> Vec<String> {
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
pub(crate) fn windows_fnm_environment(
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
pub(crate) fn windows_fnm_launch_program_candidates(
    program: &str,
    fnm_env: &[(String, String)],
) -> Vec<String> {
    windows_runtime_launch_program_candidates(program, fnm_env)
}

#[cfg(windows)]
pub(crate) fn windows_nvm_environment(
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> Option<Vec<(String, String)>> {
    let nvm_home = windows_nvm_home(env)?;
    let nvm_exe = nvm_home.join("nvm.exe");
    nvm_exe.is_file().then_some(())?;

    let mut command = Command::new(&nvm_exe);
    configure_lsp_command(&mut command);
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
    if let Some(nvm_symlink) = windows_effective_environment_value(env, "NVM_SYMLINK") {
        runtime_env.push(("NVM_SYMLINK".to_owned(), nvm_symlink));
    }
    Some(runtime_env)
}

#[cfg(windows)]
pub(crate) fn windows_nvm_launch_program_candidates(
    program: &str,
    nvm_env: &[(String, String)],
) -> Vec<String> {
    windows_runtime_launch_program_candidates(program, nvm_env)
}

#[cfg(windows)]
pub(crate) fn windows_runtime_launch_program_candidates(
    program: &str,
    runtime_env: &[(String, String)],
) -> Vec<String> {
    if Path::new(program).components().count() != 1 {
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
pub(crate) fn windows_nvm_home(env: &[(String, String)]) -> Option<PathBuf> {
    windows_effective_environment_value(env, "NVM_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            windows_effective_environment_value(env, "APPDATA")
                .map(|appdata| Path::new(&appdata).join("nvm"))
        })
}

#[cfg(windows)]
pub(crate) fn parse_windows_nvm_current_version(output: &str) -> Option<String> {
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
pub(crate) fn windows_nvm_node_dir(nvm_home: &Path, version: &str) -> Option<PathBuf> {
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
pub(crate) fn parse_windows_cmd_environment(output: &str) -> Option<Vec<(String, String)>> {
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
pub(crate) fn apply_windows_runtime_environment(
    command: &mut Command,
    env: &[(String, String)],
    runtime_env: &[(String, String)],
) {
    let explicit_path = explicit_windows_env_value(env, "PATH");
    let mut applied_path = false;
    for (key, value) in runtime_env {
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
pub(crate) fn explicit_windows_env_value<'a>(
    env: &'a [(String, String)],
    key: &str,
) -> Option<&'a String> {
    env.iter()
        .find_map(|(entry_key, value)| entry_key.eq_ignore_ascii_case(key).then_some(value))
}

#[cfg(windows)]
pub(crate) fn windows_effective_environment_value(
    env: &[(String, String)],
    key: &str,
) -> Option<String> {
    explicit_windows_env_value(env, key)
        .map(String::to_owned)
        .or_else(|| std::env::var(key).ok())
}

pub(crate) fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, LspClientError> {
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

pub(crate) fn write_response(
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

pub(crate) fn launch_summary(
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

pub(crate) fn format_transport_message(message: &Value) -> String {
    let sanitized = sanitize_transport_message(message);
    serde_json::to_string_pretty(&sanitized).unwrap_or_else(|_| sanitized.to_string())
}

pub(crate) fn sanitize_transport_message(message: &Value) -> Value {
    match message {
        Value::Array(items) => Value::Array(items.iter().map(sanitize_transport_message).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| {
                    let value = if transport_key_is_sensitive(key) {
                        Value::String("[redacted]".to_owned())
                    } else {
                        sanitize_transport_message(value)
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        _ => message.clone(),
    }
}

pub(crate) fn transport_key_is_sensitive(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect::<String>();
    matches!(
        normalized.as_str(),
        "apikey"
            | "authorization"
            | "connectionstring"
            | "datasourcename"
            | "passphrase"
            | "passwd"
            | "password"
            | "privatekey"
            | "secret"
    )
}

pub(crate) fn record_transport_message(
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

pub(crate) fn record_transport_event(
    transport_log: &TransportLog,
    server_id: &str,
    message: impl Into<String>,
) {
    record_transport_entry(
        transport_log,
        LspLogEntry::new(LspLogDirection::Event, server_id, message),
    );
}

pub(crate) fn record_transport_entry(transport_log: &TransportLog, entry: LspLogEntry) {
    if let Ok(mut log) = transport_log.lock() {
        log.record(entry);
    }
}

pub(crate) fn record_notification(notifications: &NotificationLog, notification: LspNotification) {
    if let Ok(mut log) = notifications.lock() {
        log.record(notification);
    }
}

pub(crate) fn record_published_diagnostics(
    diagnostics: &DiagnosticsByPath,
    dirty_paths: &Arc<Mutex<BTreeSet<PathBuf>>>,
    diagnostics_generation: &AtomicU64,
    path: PathBuf,
    parsed: Vec<Diagnostic>,
) {
    if let Ok(mut guard) = diagnostics.lock() {
        guard.insert(path.clone(), parsed);
    }
    if let Ok(mut dirty) = dirty_paths.lock() {
        dirty.insert(path);
    }
    diagnostics_generation.fetch_add(1, Ordering::Release);
}

pub(crate) fn note_session_disconnect_diagnostics(
    diagnostics: &DiagnosticsByPath,
    dirty_paths: &Arc<Mutex<BTreeSet<PathBuf>>>,
) {
    if let Ok(guard) = diagnostics.lock()
        && let Ok(mut dirty) = dirty_paths.lock()
    {
        dirty.extend(guard.keys().cloned());
    }
}

pub(crate) fn spawn_inert_child() -> std::io::Result<(Child, ChildStdin)> {
    #[cfg(windows)]
    let mut child = {
        use std::os::windows::process::CommandExt as _;

        Command::new("cmd")
            .args(["/C", "more"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?
    };
    #[cfg(not(windows))]
    let mut child = Command::new("sh")
        .args(["-c", "cat >/dev/null"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let stdin = child.stdin.take().ok_or_else(|| {
        std::io::Error::other("inert language server child is missing stdin pipe")
    })?;
    Ok((child, stdin))
}

pub(crate) fn session_notification_key(server_id: &str, root: Option<&Path>) -> String {
    format!("session:{server_id}:{}", notification_root_key(root))
}

pub(crate) fn notification_root_key(root: Option<&Path>) -> String {
    match root {
        Some(root) => root.display().to_string(),
        None => "global".to_owned(),
    }
}

pub(crate) fn client_capabilities() -> Result<ClientCapabilities, LspClientError> {
    serde_json::from_value::<ClientCapabilities>(json!({
        "workspace": {
            "workspaceEdit": {
                "documentChanges": true
            },
            "configuration": true,
            "workspaceFolders": true
        },
        "window": {
            "workDoneProgress": true,
            "showDocument": {
                "support": true
            }
        },
        "textDocument": {
            "hover": {
                "contentFormat": ["markdown"]
            },
            "signatureHelp": {
                "signatureInformation": {
                    "documentationFormat": ["markdown"],
                    "parameterInformation": {
                        "labelOffsetSupport": true
                    },
                    "activeParameterSupport": true
                }
            },
            "completion": {
                "completionItem": {
                    "documentationFormat": ["markdown"],
                    "resolveSupport": {
                        "properties": ["documentation", "detail"]
                    },
                    "snippetSupport": true
                }
            },
            "inlineCompletion": {
                "dynamicRegistration": false
            },
            "codeAction": {
                "dynamicRegistration": false,
                "isPreferredSupport": true,
                "disabledSupport": true,
                "codeActionLiteralSupport": {
                    "codeActionKind": {
                        "valueSet": [
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

pub(crate) fn work_done_progress_params(
    next_progress_token: &AtomicU64,
    method: &str,
) -> WorkDoneProgressParams {
    let token = next_progress_token.fetch_add(1, Ordering::AcqRel);
    WorkDoneProgressParams {
        work_done_token: Some(NumberOrString::String(format!("progress:{method}:{token}"))),
    }
}

pub(crate) fn request_timeout_for_method(method: &str) -> Duration {
    if method == Initialize::METHOD {
        INITIALIZE_REQUEST_TIMEOUT
    } else if method == CodeActionRequest::METHOD {
        CODE_ACTION_REQUEST_TIMEOUT
    } else if method == INLINE_COMPLETION_METHOD {
        INLINE_COMPLETION_REQUEST_TIMEOUT
    } else {
        REQUEST_TIMEOUT
    }
}

pub(crate) fn normalize_configuration_section(section: Option<&str>) -> Option<&str> {
    section.map(str::trim).filter(|section| !section.is_empty())
}

pub(crate) fn workspace_configuration_section_for_session(
    session: &LanguageServerSession,
) -> Option<&str> {
    normalize_configuration_section(session.workspace_configuration_section())
        .or_else(|| is_csharp_server(session.server_id()).then_some(CSHARP_WORKSPACE_SECTION))
}

pub(crate) fn normalized_workspace_configuration_settings(
    section: Option<&str>,
    settings: Option<Value>,
) -> Option<Value> {
    let settings = settings?;
    let Some(section) = normalize_configuration_section(section) else {
        return Some(settings);
    };
    match settings {
        Value::Object(mut object) => object.remove(section).or(Some(Value::Object(object))),
        other => Some(other),
    }
}

pub(crate) fn effective_workspace_configuration_settings(
    base_settings: Option<&Value>,
    runtime_override: Option<&Value>,
) -> Option<Value> {
    match (base_settings, runtime_override) {
        (Some(base_settings), Some(runtime_override)) => {
            Some(merge_json_values(base_settings, runtime_override))
        }
        (Some(base_settings), None) => Some(base_settings.clone()),
        (None, Some(runtime_override)) => Some(runtime_override.clone()),
        (None, None) => None,
    }
}

pub(crate) fn settings_contains_key(settings: Option<&Value>, key: &str) -> bool {
    match settings {
        Some(Value::Object(settings)) => settings.contains_key(key),
        Some(_) => false,
        None => false,
    }
}

pub(crate) fn text_document_sync_kind(
    capability: Option<TextDocumentSyncCapability>,
) -> TextDocumentSyncKind {
    match capability {
        Some(TextDocumentSyncCapability::Kind(kind)) => kind,
        Some(TextDocumentSyncCapability::Options(options)) => {
            options.change.unwrap_or(TextDocumentSyncKind::FULL)
        }
        None => TextDocumentSyncKind::FULL,
    }
}

pub(crate) fn text_document_content_change(
    sync_kind: TextDocumentSyncKind,
    previous_text: &str,
    text: &str,
) -> TextDocumentContentChangeEvent {
    if sync_kind == TextDocumentSyncKind::INCREMENTAL {
        TextDocumentContentChangeEvent {
            range: Some(full_document_range(previous_text)),
            range_length: None,
            text: text.to_owned(),
        }
    } else {
        TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_owned(),
        }
    }
}

pub(crate) fn usable_edit_chain(
    edits: Option<&[editor_buffer::TextEdit]>,
    last_revision: Option<u64>,
    current_revision: u64,
) -> Option<&[editor_buffer::TextEdit]> {
    let edits = edits?;
    let last_revision = last_revision?;
    if last_revision == current_revision {
        return Some(&[]);
    }
    let start = edits
        .iter()
        .position(|edit| edit.before_revision == last_revision)?;
    let suffix = &edits[start..];
    if suffix.last()?.after_revision != current_revision
        || suffix
            .windows(2)
            .any(|pair| pair[0].after_revision != pair[1].before_revision)
    {
        return None;
    }
    Some(suffix)
}

pub(crate) fn incremental_content_changes(
    previous_text: &str,
    new_text: &str,
    edits: &[editor_buffer::TextEdit],
) -> Option<Vec<TextDocumentContentChangeEvent>> {
    if edits.is_empty() {
        return Some(Vec::new());
    }
    let mut working = previous_text.to_owned();
    let mut changes = Vec::with_capacity(edits.len());
    for (index, edit) in edits.iter().enumerate() {
        if edit.start_byte > edit.old_end_byte
            || edit.old_end_byte > working.len()
            || edit.start_byte > working.len()
        {
            return None;
        }
        let inserted = inserted_text_after_edit(new_text, edit, &edits[index + 1..])?;
        changes.push(TextDocumentContentChangeEvent {
            range: Some(Range::new(
                lsp_position_on_text(&working, edit.start_position),
                lsp_position_on_text(&working, edit.old_end_position),
            )),
            range_length: None,
            text: inserted.clone(),
        });
        working.replace_range(edit.start_byte..edit.old_end_byte, &inserted);
    }
    (working == new_text).then_some(changes)
}

pub(crate) fn inserted_text_after_edit(
    new_text: &str,
    edit: &editor_buffer::TextEdit,
    remaining: &[editor_buffer::TextEdit],
) -> Option<String> {
    let mut start = edit.start_byte;
    let mut end = edit.new_end_byte;
    if start > end {
        return None;
    }
    for later in remaining {
        (start, end) = map_exclusive_range_through_edit(start, end, later)?;
    }
    new_text.get(start..end).map(str::to_owned)
}

pub(crate) fn map_exclusive_range_through_edit(
    start: usize,
    end: usize,
    edit: &editor_buffer::TextEdit,
) -> Option<(usize, usize)> {
    let delta = edit.new_end_byte as isize - edit.old_end_byte as isize;
    if end <= edit.start_byte {
        Some((start, end))
    } else if start >= edit.old_end_byte {
        Some((
            usize::try_from(start as isize + delta).ok()?,
            usize::try_from(end as isize + delta).ok()?,
        ))
    } else {
        None
    }
}

pub(crate) fn lsp_position_on_text(text: &str, point: TextPoint) -> Position {
    let line = line_slice(text, point.line);
    Position::new(point.line as u32, utf16_column(line, point.column))
}

pub(crate) fn line_slice(text: &str, line_index: usize) -> &str {
    let mut remaining = text;
    for _ in 0..line_index {
        let Some(index) = remaining.find('\n') else {
            return "";
        };
        remaining = remaining.get(index.saturating_add(1)..).unwrap_or("");
    }
    remaining
        .split_once('\n')
        .map(|(line, _)| line)
        .unwrap_or(remaining)
}

pub(crate) fn utf16_column(line: &str, char_column: usize) -> u32 {
    let mut character = 0u32;
    for (index, ch) in line.chars().enumerate() {
        if index >= char_column {
            break;
        }
        if ch == '\n' {
            break;
        }
        if ch != '\r' {
            character = character.saturating_add(ch.len_utf16() as u32);
        }
    }
    character
}

pub(crate) fn full_document_range(text: &str) -> Range {
    Range::new(Position::new(0, 0), text_end_position(text))
}

pub(crate) fn text_end_position(text: &str) -> Position {
    let mut line = 0u32;
    let mut character = 0u32;
    for ch in text.chars() {
        if ch == '\n' {
            line = line.saturating_add(1);
            character = 0;
        } else if ch != '\r' {
            character = character.saturating_add(ch.len_utf16() as u32);
        }
    }
    Position::new(line, character)
}

pub(crate) fn merge_json_values(base: &Value, override_value: &Value) -> Value {
    match (base, override_value) {
        (Value::Object(base), Value::Object(override_value)) => {
            let mut merged = base.clone();
            for (key, override_value) in override_value {
                let merged_value = merged
                    .get(key)
                    .map(|base_value| merge_json_values(base_value, override_value))
                    .unwrap_or_else(|| override_value.clone());
                merged.insert(key.clone(), merged_value);
            }
            Value::Object(merged)
        }
        _ => override_value.clone(),
    }
}

pub(crate) fn workspace_configuration_null_response(params: Option<&Value>) -> Value {
    let item_count = params
        .and_then(|params| params.get("items"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    Value::Array((0..item_count).map(|_| Value::Null).collect())
}

pub(crate) fn configuration_item_section(item: &Value) -> Option<&str> {
    normalize_configuration_section(item.get("section").and_then(Value::as_str))
}

pub(crate) fn with_csharp_solution_path_override(
    current: Option<Value>,
    solution_path: &Path,
) -> Option<Value> {
    let override_value = json!({
        "solutionPathOverride": solution_path.display().to_string(),
    });
    Some(
        match normalized_workspace_configuration_settings(Some(CSHARP_WORKSPACE_SECTION), current) {
            Some(current) => merge_json_values(&current, &override_value),
            None => override_value,
        },
    )
}

pub(crate) fn without_csharp_solution_path_override(current: Option<Value>) -> Option<Value> {
    let current =
        normalized_workspace_configuration_settings(Some(CSHARP_WORKSPACE_SECTION), current)?;
    let Value::Object(mut current) = current else {
        return Some(current);
    };
    current.remove("solutionPathOverride");
    (!current.is_empty()).then_some(Value::Object(current))
}

pub(crate) fn wrap_workspace_configuration_settings(section: &str, settings: Value) -> Value {
    let mut object = serde_json::Map::new();
    object.insert(section.to_owned(), settings);
    Value::Object(object)
}

pub(crate) fn is_copilot_server(server_id: &str) -> bool {
    server_id == COPILOT_SERVER_ID
}

pub(crate) fn is_csharp_server(server_id: &str) -> bool {
    matches!(server_id, CSHARP_SERVER_ID | ROSLYN_LANGUAGE_SERVER_ID)
}

pub(crate) fn is_csharp_metadata_uri(uri: &str) -> bool {
    uri.starts_with("csharp:/")
}

pub(crate) fn initialization_options_for_server(
    server_id: &str,
    override_value: Option<&Value>,
) -> Option<Value> {
    let base = if is_copilot_server(server_id) {
        Some(json!({
            "editorInfo": {
                "name": "Volt",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "editorPluginInfo": {
                "name": "Volt Copilot",
                "version": env!("CARGO_PKG_VERSION"),
            }
        }))
    } else if is_csharp_server(server_id) {
        Some(json!({
            "experimental": {
                "csharp": {
                    "metadataUris": true,
                }
            }
        }))
    } else {
        None
    };
    match (base, override_value) {
        (Some(base), Some(override_value)) => Some(merge_json_values(&base, override_value)),
        (Some(base), None) => Some(base),
        (None, Some(override_value)) => Some(override_value.clone()),
        (None, None) => None,
    }
}

pub(crate) fn csharp_metadata_request_params(uri: &str) -> Value {
    json!({
        "textDocument": {
            "uri": uri,
        }
    })
}

pub(crate) fn parse_csharp_metadata_response(
    uri: &str,
    value: &Value,
) -> Result<Option<Value>, LspClientError> {
    if value.is_null() {
        return Ok(None);
    }
    if let Some(source) = value.as_str() {
        return Ok(Some(json!({
            "uri": uri,
            "source": source,
        })));
    }
    let Some(metadata) = value.as_object() else {
        return Err(LspClientError::Protocol(
            "failed to decode csharp metadata response: expected an object".to_owned(),
        ));
    };
    let mut metadata = metadata.clone();
    metadata
        .entry("uri".to_owned())
        .or_insert_with(|| Value::String(uri.to_owned()));
    if !metadata.contains_key("source")
        && let Some(source) = metadata.get("text").cloned()
    {
        metadata.insert("source".to_owned(), source);
    }
    Ok(Some(Value::Object(metadata)))
}

pub(crate) fn session_lifecycle_notification(
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
    LspNotification {
        key: session_notification_key(server_id, root),
        server_id: server_id.to_owned(),
        root: root.map(Path::to_path_buf),
        level,
        title: format!("LSP · {server_id}"),
        body_lines: lines,
        progress: None,
        active,
        action: None,
    }
}

pub(crate) fn progress_notification_key(
    server_id: &str,
    root: Option<&Path>,
    token: &str,
) -> String {
    format!(
        "progress:{server_id}:{}:{token}",
        notification_root_key(root)
    )
}

pub(crate) fn parse_progress_token_key(value: Option<&Value>) -> Option<String> {
    let token = value?;
    if let Some(token) = token.as_str() {
        return Some(token.to_owned());
    }
    token.as_u64().map(|token| token.to_string())
}

pub(crate) fn parse_optional_progress_text(value: Option<&Value>) -> Option<Option<String>> {
    value.map(|value| {
        value
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    })
}

pub(crate) fn parse_progress_percentage(value: Option<&Value>) -> Option<Option<u32>> {
    value.map(|value| {
        value
            .as_u64()
            .and_then(|percentage| u32::try_from(percentage.min(100)).ok())
    })
}

pub(crate) fn progress_body_lines(track: &ProgressTrack) -> Vec<String> {
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

pub(crate) fn completion_level_for_message(message: Option<&str>) -> LspNotificationLevel {
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

pub(crate) fn parse_progress_notification(
    server_id: &str,
    root: Option<&Path>,
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
            Some(LspNotification {
                key: progress_notification_key(server_id, root, &token),
                server_id: server_id.to_owned(),
                root: root.map(Path::to_path_buf),
                level: LspNotificationLevel::Info,
                title: format!("LSP · {server_id}"),
                body_lines,
                progress: Some(LspNotificationProgress::new(progress)),
                active: true,
                action: None,
            })
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
            Some(LspNotification {
                key: progress_notification_key(server_id, root, &token),
                server_id: server_id.to_owned(),
                root: root.map(Path::to_path_buf),
                level: LspNotificationLevel::Info,
                title: format!("LSP · {server_id}"),
                body_lines: progress_body_lines(track),
                progress: Some(LspNotificationProgress::new(track.percentage)),
                active: true,
                action: None,
            })
        }
        "end" => {
            let mut track = progress_tracks.remove(&token).unwrap_or_default();
            if let Some(message) = parse_optional_progress_text(value.get("message")) {
                track.message = message;
            }
            Some(LspNotification {
                key: progress_notification_key(server_id, root, &token),
                server_id: server_id.to_owned(),
                root: root.map(Path::to_path_buf),
                level: completion_level_for_message(track.message.as_deref()),
                title: format!("LSP · {server_id}"),
                body_lines: progress_body_lines(&track),
                progress: track
                    .percentage
                    .map(|percentage| LspNotificationProgress::new(Some(percentage))),
                active: false,
                action: None,
            })
        }
        _ => None,
    }
}

pub(crate) fn parse_window_message_notification(
    method: &str,
    server_id: &str,
    root: Option<&Path>,
    params: &Value,
) -> Option<LspNotification> {
    // window/logMessage is output-channel traffic only (already in the transport log).
    // Do not promote it to UI toasts — servers such as ols misuse MessageType::Error
    // for benign startup lines like "Starting Odin Language Server …".
    if method != "window/showMessage" {
        return None;
    }
    parse_show_message_notification(server_id, root, params)
}

pub(crate) fn parse_show_message_notification(
    server_id: &str,
    root: Option<&Path>,
    params: &Value,
) -> Option<LspNotification> {
    let level = match params.get("type").and_then(Value::as_u64) {
        Some(1) => LspNotificationLevel::Error,
        Some(2) => LspNotificationLevel::Warning,
        Some(3) | Some(4) => LspNotificationLevel::Info,
        _ => LspNotificationLevel::Info,
    };
    if level != LspNotificationLevel::Error {
        return None;
    }
    let message = params
        .get("message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())?
        .to_owned();
    let mut lines = vec![message.clone()];
    if let Some(root) = root {
        lines.push(root.display().to_string());
    }
    Some(LspNotification {
        key: format!(
            "message:{server_id}:{}:{level:?}:{message}",
            notification_root_key(root)
        ),
        server_id: server_id.to_owned(),
        root: root.map(Path::to_path_buf),
        level,
        title: format!("LSP · {server_id}"),
        body_lines: lines,
        progress: None,
        active: false,
        action: None,
    })
}

pub(crate) fn status_notification_key(server_id: &str, root: Option<&Path>) -> String {
    format!("status:{server_id}:{}", notification_root_key(root))
}

pub(crate) fn parse_copilot_status_notification(
    server_id: &str,
    root: Option<&Path>,
    params: &Value,
) -> Option<LspNotification> {
    let kind = params
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("Normal");
    let message = params
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim();
    let level = match kind {
        "Error" => LspNotificationLevel::Error,
        "Warning" => LspNotificationLevel::Warning,
        "Inactive" => LspNotificationLevel::Info,
        _ => LspNotificationLevel::Success,
    };
    if level != LspNotificationLevel::Error {
        return None;
    }
    let mut lines = vec![kind.to_owned()];
    if !message.is_empty() {
        lines.push(message.to_owned());
    }
    if let Some(root) = root {
        lines.push(root.display().to_string());
    }
    let action = (is_copilot_server(server_id) && kind == "Error")
        .then_some(LspNotificationAction::CopilotSignIn);
    Some(LspNotification {
        key: status_notification_key(server_id, root),
        server_id: server_id.to_owned(),
        root: root.map(Path::to_path_buf),
        level,
        title: format!("LSP · {server_id}"),
        body_lines: lines,
        progress: None,
        active: matches!(kind, "Error" | "Warning" | "Inactive"),
        action,
    })
}

pub(crate) struct ServerRequestHandling {
    pub(crate) result: Value,
    pub(crate) notification: Option<LspNotification>,
}

pub(crate) fn server_request_response(
    server_id: &str,
    root: Option<&Path>,
    method: Option<&Value>,
    params: Option<&Value>,
    workspace_configuration: Option<&SessionWorkspaceConfiguration>,
) -> ServerRequestHandling {
    let result = match method.and_then(Value::as_str) {
        Some("workspace/configuration") => workspace_configuration
            .map(|workspace_configuration| workspace_configuration.response_for_request(params))
            .unwrap_or_else(|| workspace_configuration_null_response(params)),
        Some("workspace/workspaceFolders") => Value::Array(Vec::new()),
        Some("window/showMessageRequest") => params
            .and_then(|params| params.get("actions"))
            .and_then(Value::as_array)
            .and_then(|actions| actions.first())
            .cloned()
            .unwrap_or(Value::Null),
        Some("window/showDocument") => json!({ "success": false }),
        Some("client/registerCapability")
        | Some("client/unregisterCapability")
        | Some("window/workDoneProgress/create") => Value::Null,
        _ => Value::Null,
    };
    let notification = if matches!(method.and_then(Value::as_str), Some("window/showDocument")) {
        show_document_notification(server_id, root, params)
    } else {
        None
    };
    let result = if notification.is_some() {
        json!({ "success": true })
    } else {
        result
    };
    ServerRequestHandling {
        result,
        notification,
    }
}

pub(crate) fn show_document_notification(
    server_id: &str,
    root: Option<&Path>,
    params: Option<&Value>,
) -> Option<LspNotification> {
    if !is_copilot_server(server_id) {
        return None;
    }
    let uri = params
        .and_then(|params| params.get("uri"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|uri| uri.starts_with("http://") || uri.starts_with("https://"))?
        .to_owned();
    let mut lines = vec!["Opening browser popup".to_owned(), uri.clone()];
    if let Some(root) = root {
        lines.push(root.display().to_string());
    }
    Some(LspNotification {
        key: format!(
            "show-document:{server_id}:{}:{uri}",
            notification_root_key(root)
        ),
        server_id: server_id.to_owned(),
        root: root.map(Path::to_path_buf),
        level: LspNotificationLevel::Info,
        title: format!("LSP · {server_id}"),
        body_lines: lines,
        progress: None,
        active: false,
        action: Some(LspNotificationAction::OpenBrowserPopup { url: uri }),
    })
}

pub(crate) fn parse_publish_diagnostics(params: &Value) -> Option<(PathBuf, Vec<Diagnostic>)> {
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

pub(crate) fn parse_diagnostic(value: &Value) -> Option<Diagnostic> {
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

pub(crate) fn parse_hover_response(server_id: &str, value: &Value) -> Option<LspHoverContents> {
    let contents = value.get("contents")?;
    let (text, markdown) = hover_text(contents)?;
    let lines = hover_text_lines(&text);
    (!lines.is_empty()).then(|| LspHoverContents::new(server_id, text, lines, markdown))
}

pub(crate) fn parse_signature_help_response(
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
    Ok(signature_help_markdown(&signature_help, None)
        .is_some()
        .then(|| LspSignatureHelpContents::new(server_id, signature_help)))
}

pub(crate) fn parse_definition_response(
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

pub(crate) fn parse_reference_response(
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

pub(crate) fn parse_text_edit_response(
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

pub(crate) fn parse_code_action_response(
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

pub(crate) fn parse_code_action_item(server_id: &str, value: &Value) -> Option<LspCodeAction> {
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
    Some(LspCodeAction {
        server_id: server_id.to_owned(),
        title: title.to_owned(),
        kind,
        disabled_reason,
        preferred,
        document_edits,
        command_name,
        has_resource_operations,
    })
}

pub(crate) fn parse_code_action_command_name(value: &Value) -> Option<String> {
    match value.get("command") {
        Some(Value::String(command)) => Some(command.to_owned()),
        Some(Value::Object(command)) => command
            .get("command")
            .and_then(Value::as_str)
            .map(str::to_owned),
        _ => None,
    }
}

pub(crate) fn parse_code_action_workspace_edit(
    value: Option<&Value>,
) -> (Vec<LspDocumentTextEdits>, bool) {
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

pub(crate) fn parse_code_action_document_change(value: &Value) -> Option<LspDocumentTextEdits> {
    let path = value
        .get("textDocument")
        .and_then(|text_document| text_document.get("uri"))
        .and_then(Value::as_str)
        .and_then(file_uri_to_path)?;
    let edits = parse_inline_text_edits(value.get("edits")?);
    (!edits.is_empty()).then(|| LspDocumentTextEdits::new(path, edits))
}

pub(crate) fn parse_inline_text_edits(value: &Value) -> Vec<LspTextEdit> {
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

pub(crate) fn parse_inline_text_edit(value: &Value) -> Option<LspTextEdit> {
    let range = parse_inline_text_range(value.get("range")?)?;
    let new_text = value.get("newText")?.as_str()?;
    Some(LspTextEdit::new(range, new_text))
}

pub(crate) fn parse_inline_text_range(value: &Value) -> Option<TextRange> {
    Some(TextRange::new(
        parse_inline_text_point(value.get("start")?)?,
        parse_inline_text_point(value.get("end")?)?,
    ))
}

pub(crate) fn parse_inline_text_point(value: &Value) -> Option<TextPoint> {
    let line = value.get("line").and_then(Value::as_u64)?;
    let character = value.get("character").and_then(Value::as_u64)?;
    Some(TextPoint::new(
        usize::try_from(line).ok()?,
        usize::try_from(character).ok()?,
    ))
}

pub(crate) fn diagnostic_matches_request_range(
    diagnostic_range: TextRange,
    request_range: TextRange,
) -> bool {
    let diagnostic_range = diagnostic_range.normalized();
    let request_range = request_range.normalized();
    if request_range.start() == request_range.end() {
        let point = request_range.start();
        return diagnostic_range.start() <= point && point <= diagnostic_range.end();
    }
    diagnostic_range.start() < request_range.end() && request_range.start() < diagnostic_range.end()
}

pub(crate) fn code_action_params(
    path: &Path,
    range: TextRange,
    diagnostics: &[Diagnostic],
    work_done_progress_params: WorkDoneProgressParams,
) -> Result<CodeActionParams, LspClientError> {
    Ok(CodeActionParams {
        text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
        range: lsp_range_from_text_range(range),
        context: CodeActionContext {
            diagnostics: diagnostics.iter().map(lsp_code_action_diagnostic).collect(),
            only: None,
            trigger_kind: Some(CodeActionTriggerKind::INVOKED),
        },
        work_done_progress_params,
        partial_result_params: PartialResultParams::default(),
    })
}

pub(crate) fn lsp_code_action_diagnostic(diagnostic: &Diagnostic) -> LspDiagnostic {
    LspDiagnostic::new(
        lsp_range_from_text_range(diagnostic.range()),
        Some(lsp_diagnostic_severity(diagnostic.severity())),
        None,
        Some(diagnostic.source().to_owned()),
        diagnostic.message().to_owned(),
        None,
        None,
    )
}

pub(crate) fn lsp_diagnostic_severity(severity: DiagnosticSeverity) -> LspDiagnosticSeverity {
    match severity {
        DiagnosticSeverity::Error => LspDiagnosticSeverity::ERROR,
        DiagnosticSeverity::Warning => LspDiagnosticSeverity::WARNING,
        DiagnosticSeverity::Information => LspDiagnosticSeverity::INFORMATION,
    }
}

pub(crate) fn hover_text(value: &Value) -> Option<(String, bool)> {
    let contents = serde_json::from_value::<HoverContents>(value.clone()).ok()?;
    match contents {
        HoverContents::Scalar(marked_string) => hover_marked_string(marked_string),
        HoverContents::Array(values) => {
            let parts = values
                .into_iter()
                .filter_map(hover_marked_string_markdown_text)
                .collect::<Vec<_>>();
            let text = normalize_hover_text(&parts.join("\n\n"));
            (!text.trim().is_empty()).then_some((text, true))
        }
        HoverContents::Markup(content) => {
            let text = normalize_hover_text(&content.value);
            (!text.trim().is_empty()).then_some((text, content.kind == MarkupKind::Markdown))
        }
    }
}

pub(crate) fn hover_marked_string(marked_string: MarkedString) -> Option<(String, bool)> {
    match marked_string {
        MarkedString::String(text) => {
            let text = normalize_hover_text(&text);
            (!text.trim().is_empty()).then_some((text, true))
        }
        MarkedString::LanguageString(language) => {
            let text =
                normalize_hover_text(&markdown_code_fence(&language.language, &language.value));
            (!text.trim().is_empty()).then_some((text, true))
        }
    }
}

pub(crate) fn hover_marked_string_markdown_text(marked_string: MarkedString) -> Option<String> {
    hover_marked_string(marked_string).map(|(text, _)| text)
}

pub(crate) fn markdown_code_fence(language: &str, value: &str) -> String {
    let value = normalize_hover_text(value);
    let language = language.trim();
    if language.is_empty() {
        format!("```\n{value}\n```")
    } else {
        format!("```{language}\n{value}\n```")
    }
}

pub(crate) fn hover_text_lines(text: &str) -> Vec<String> {
    let text = normalize_hover_text(text);
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut lines = text
        .split('\n')
        .map(str::trim_end)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines
}

pub(crate) fn normalize_hover_text(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

pub(crate) fn signature_help_markdown(
    signature_help: &SignatureHelp,
    language: Option<&str>,
) -> Option<String> {
    if signature_help.signatures.is_empty() {
        return None;
    }
    let active_signature_index = signature_help
        .active_signature
        .map(|index| index as usize)
        .filter(|index| *index < signature_help.signatures.len())
        .unwrap_or(0);
    let active_signature = signature_help.signatures.get(active_signature_index)?;
    let active_parameter_index = active_signature
        .active_parameter
        .or(signature_help.active_parameter)
        .map(|index| index as usize);
    let language = language.unwrap_or_default();
    let multiple_signatures = signature_help.signatures.len() > 1;
    let mut parts = Vec::new();
    for (index, signature) in signature_help.signatures.iter().enumerate() {
        if multiple_signatures {
            let active_marker = if index == active_signature_index {
                " (active)"
            } else {
                ""
            };
            parts.push(format!(
                "**Signature {}/{}{}**",
                index + 1,
                signature_help.signatures.len(),
                active_marker
            ));
        }
        parts.push(markdown_code_fence(language, &signature.label));
        if index == active_signature_index
            && let Some(parameter_documentation) = active_parameter_index
                .and_then(|parameter_index| {
                    signature
                        .parameters
                        .as_ref()
                        .and_then(|parameters| parameters.get(parameter_index))
                })
                .and_then(|parameter| parameter.documentation.as_ref())
        {
            parts.push(documentation_markdown(parameter_documentation));
        }
        if let Some(documentation) = signature.documentation.as_ref() {
            parts.push(documentation_markdown(documentation));
        }
    }
    let text = normalize_hover_text(&parts.join("\n\n"));
    (!text.trim().is_empty()).then_some(text)
}

pub(crate) fn documentation_markdown(documentation: &Documentation) -> String {
    match documentation {
        Documentation::String(text) => normalize_hover_text(text),
        Documentation::MarkupContent(content) => normalize_hover_text(&content.value),
    }
}

pub(crate) fn active_parameter_char_range(
    signature: &lsp_types::SignatureInformation,
    active_parameter_index: usize,
) -> Option<(usize, usize)> {
    let parameter = signature.parameters.as_ref()?.get(active_parameter_index)?;
    match &parameter.label {
        ParameterLabel::Simple(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return None;
            }
            find_substring_char_range(&signature.label, trimmed)
        }
        ParameterLabel::LabelOffsets([start, end]) => {
            let start = *start as usize;
            let end = *end as usize;
            let label_chars = signature.label.chars().count();
            (start <= end && end <= label_chars).then_some((start, end))
        }
    }
}

pub(crate) fn find_substring_char_range(haystack: &str, needle: &str) -> Option<(usize, usize)> {
    let needle_len = needle.chars().count();
    if needle_len == 0 {
        return None;
    }
    let haystack_chars = haystack.chars().collect::<Vec<_>>();
    for start in 0..=haystack_chars.len().saturating_sub(needle_len) {
        if haystack_chars[start..start + needle_len] == needle.chars().collect::<Vec<_>>()[..] {
            return Some((start, start + needle_len));
        }
    }
    None
}

pub(crate) fn location_from_lsp(server_id: &str, location: &Location) -> Option<LspLocation> {
    Some(LspLocation::from_uri(
        server_id,
        location.uri.to_string(),
        text_range_from_lsp_range(&location.range),
    ))
}

pub(crate) fn location_from_link(server_id: &str, link: &LocationLink) -> Option<LspLocation> {
    Some(LspLocation::from_uri(
        server_id,
        link.target_uri.to_string(),
        text_range_from_lsp_range(&link.target_selection_range),
    ))
}

pub(crate) fn lsp_text_edit_from_lsp(edit: &TextEdit) -> LspTextEdit {
    LspTextEdit::new(
        text_range_from_lsp_range(&edit.range),
        edit.new_text.clone(),
    )
}

pub(crate) fn text_range_from_lsp_range(range: &lsp_types::Range) -> TextRange {
    TextRange::new(
        text_point_from_lsp_position(range.start),
        text_point_from_lsp_position(range.end),
    )
}

pub(crate) fn text_point_from_lsp_position(position: Position) -> TextPoint {
    TextPoint::new(position.line as usize, position.character as usize)
}

pub(crate) fn lsp_range_from_text_range(range: TextRange) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_position_from_text_point(range.start()),
        end: lsp_position_from_text_point(range.end()),
    }
}

pub(crate) fn lsp_position_from_text_point(point: TextPoint) -> Position {
    Position::new(point.line as u32, point.column as u32)
}

pub(crate) fn lsp_formatting_options(options: LspFormattingOptions) -> FormattingOptions {
    FormattingOptions {
        tab_size: options.tab_size(),
        insert_spaces: options.insert_spaces(),
        ..FormattingOptions::default()
    }
}

pub(crate) fn unsupported_lsp_request(error: &LspClientError) -> bool {
    let LspClientError::Protocol(message) = error else {
        return false;
    };
    let lower = message.to_ascii_lowercase();
    lower.contains("-32601")
        || lower.contains("method not found")
        || lower.contains("method not supported")
}

pub(crate) fn sort_locations(locations: &mut Vec<LspLocation>) {
    locations.sort_by(|left, right| {
        left.uri
            .cmp(&right.uri)
            .then_with(|| left.range.start().line.cmp(&right.range.start().line))
            .then_with(|| left.range.start().column.cmp(&right.range.start().column))
            .then_with(|| left.range.end().line.cmp(&right.range.end().line))
            .then_with(|| left.range.end().column.cmp(&right.range.end().column))
    });
    locations.dedup_by(|left, right| left.uri == right.uri && left.range == right.range);
}

pub(crate) fn parse_completion_response(server_id: &str, value: &Value) -> Vec<LspCompletionItem> {
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

pub(crate) fn inline_completion_params(
    path: &Path,
    version: i32,
    position: TextPoint,
    options: LspFormattingOptions,
) -> Result<Value, LspClientError> {
    let position = text_document_position_params(path, position)?.position;
    Ok(json!({
        "textDocument": {
            "uri": path_to_uri(path)?,
            "version": version,
        },
        "position": position,
        "context": {
            "triggerKind": 2,
        },
        "formattingOptions": {
            "tabSize": options.tab_size(),
            "insertSpaces": options.insert_spaces(),
        }
    }))
}

pub(crate) fn parse_inline_completion_response(
    server_id: &str,
    root: Option<PathBuf>,
    position: TextPoint,
    value: &Value,
) -> Vec<LspInlineCompletionItem> {
    let empty = Vec::new();
    let items = match value {
        Value::Array(items) => items,
        Value::Object(map) => map.get("items").and_then(Value::as_array).unwrap_or(&empty),
        _ => return Vec::new(),
    };
    items
        .iter()
        .filter_map(|item| parse_inline_completion_item(server_id, root.clone(), position, item))
        .collect::<Vec<_>>()
}

pub(crate) fn parse_inline_completion_item(
    server_id: &str,
    root: Option<PathBuf>,
    position: TextPoint,
    value: &Value,
) -> Option<LspInlineCompletionItem> {
    let insert_text = value.get("insertText")?.as_str()?.replace("\r\n", "\n");
    if insert_text.is_empty() {
        return None;
    }
    let range = value
        .get("range")
        .and_then(parse_inline_text_range)
        .unwrap_or_else(|| TextRange::new(position, position));
    Some(LspInlineCompletionItem::new(
        server_id,
        root,
        insert_text,
        range,
        value.clone(),
    ))
}

pub(crate) fn execute_command_params_from_inline_item(value: &Value) -> Option<Value> {
    let command = parse_lsp_server_command(value.get("command"))?;
    Some(execute_command_params(&command))
}

pub(crate) fn execute_command_params(command: &LspServerCommand) -> Value {
    json!({
        "command": command.command(),
        "arguments": command.arguments(),
    })
}

pub(crate) fn parse_lsp_server_command(value: Option<&Value>) -> Option<LspServerCommand> {
    let value = value?;
    let title = value.get("title").and_then(Value::as_str)?.trim();
    let command = value.get("command").and_then(Value::as_str)?.trim();
    if title.is_empty() || command.is_empty() {
        return None;
    }
    let arguments = value
        .get("arguments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Some(LspServerCommand::new(
        title.to_owned(),
        command.to_owned(),
        arguments,
    ))
}

pub(crate) fn parse_copilot_sign_in_response(value: &Value) -> Option<CopilotDeviceCodePrompt> {
    let user_code = value.get("userCode").and_then(Value::as_str)?.trim();
    let command = parse_lsp_server_command(value.get("command"))?;
    if user_code.is_empty() {
        return None;
    }
    Some(CopilotDeviceCodePrompt::new(user_code.to_owned(), command))
}

pub(crate) fn parse_completion_item(server_id: &str, value: &Value) -> Option<LspCompletionItem> {
    let label = value.get("label")?.as_str()?.to_owned();
    let kind = value
        .get("kind")
        .and_then(Value::as_u64)
        .and_then(parse_completion_kind);
    // LSP: when textEdit is present, it owns the inserted text and range.
    // Prefer it over insertText so trigger characters (e.g. '.') are not doubled.
    let (insert_text, edit_range) = match value.get("textEdit") {
        Some(text_edit) => {
            let new_text = text_edit
                .get("newText")
                .and_then(Value::as_str)
                .or_else(|| value.get("insertText").and_then(Value::as_str))
                .unwrap_or(&label)
                .to_owned();
            let range = text_edit
                .get("replace")
                .or_else(|| text_edit.get("range"))
                .and_then(parse_inline_text_range);
            (new_text, range)
        }
        None => {
            let insert_text = value
                .get("insertText")
                .and_then(Value::as_str)
                .unwrap_or(&label)
                .to_owned();
            (insert_text, None)
        }
    };
    let detail = value
        .get("detail")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let documentation = value
        .get("documentation")
        .and_then(completion_documentation)
        .or_else(|| detail.clone());
    let has_documentation = documentation.is_some();
    Some(
        LspCompletionItem::new(
            server_id,
            kind,
            label,
            insert_text,
            edit_range,
            detail,
            documentation,
        )
        .with_raw_item(value.clone(), has_documentation),
    )
}

pub(crate) fn completion_documentation(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.to_owned()),
        Value::Object(map) => map.get("value").and_then(Value::as_str).map(str::to_owned),
        _ => None,
    }
}

pub(crate) fn parse_completion_kind(kind: u64) -> Option<LspCompletionKind> {
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

pub(crate) fn path_to_file_uri(path: &Path) -> String {
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

pub(crate) fn path_to_uri(path: &Path) -> Result<Uri, LspClientError> {
    path_to_file_uri(path).parse().map_err(|error| {
        LspClientError::Protocol(format!(
            "failed to convert `{}` into a valid file URI: {error}",
            path.display()
        ))
    })
}

pub(crate) fn text_document_position_params(
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

pub(crate) fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
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

pub(crate) fn percent_decode(raw: &str) -> String {
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
