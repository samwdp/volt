#![doc = r#"Language Server Protocol registry, session plans, diagnostics, launch metadata, and client runtime management."#]

mod client;

use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use editor_buffer::TextRange;
use editor_jobs::JobSpec;
use editor_path::{PathMatcher, PathPattern, normalize_extension};
use serde_json::{Number, Value};

pub use client::{
    LspClientError, LspClientManager, LspCodeAction, LspCompletionItem, LspCompletionKind,
    LspDocumentTextEdits, LspFormattingOptions, LspHoverContents, LspLocation, LspLogDirection,
    LspLogEntry, LspLogSnapshot, LspNotification, LspNotificationEntry, LspNotificationLevel,
    LspNotificationProgress, LspNotificationSnapshot, LspSignatureHelpContents, LspTextEdit,
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Language Server Protocol registry, session plans, diagnostics, launch metadata, and client runtime management.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Diagnostic severity levels surfaced through LSP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Informational note.
    Information,
    /// Warning diagnostic.
    Warning,
    /// Error diagnostic.
    Error,
}

/// Editor-facing diagnostic reported by an LSP session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    source: String,
    message: String,
    severity: DiagnosticSeverity,
    range: TextRange,
}

impl Diagnostic {
    /// Creates a diagnostic entry.
    pub fn new(
        source: impl Into<String>,
        message: impl Into<String>,
        severity: DiagnosticSeverity,
        range: TextRange,
    ) -> Self {
        Self {
            source: source.into(),
            message: message.into(),
            severity,
            range,
        }
    }

    /// Returns the diagnostic source.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the diagnostic message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the severity.
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the affected range.
    pub const fn range(&self) -> TextRange {
        self.range
    }
}

/// Workspace configuration metadata carried from declarative server specs into planned sessions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceConfiguration {
    section: Option<String>,
    settings: Option<WorkspaceConfigurationValue>,
}

impl WorkspaceConfiguration {
    /// Creates an empty workspace configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the configuration section queried for this server.
    pub fn with_section(mut self, section: impl Into<String>) -> Self {
        self.section = normalize_optional_string(section.into());
        self
    }

    /// Sets the server-specific workspace settings payload.
    pub fn with_settings(mut self, settings: impl Into<WorkspaceConfigurationValue>) -> Self {
        self.settings = Some(settings.into());
        self
    }

    /// Returns the workspace configuration section, if one is declared.
    pub fn section(&self) -> Option<&str> {
        self.section.as_deref()
    }

    /// Returns the workspace settings payload, if one is declared.
    pub fn settings(&self) -> Option<&WorkspaceConfigurationValue> {
        self.settings.as_ref()
    }

    /// Returns the workspace settings payload as a JSON value.
    pub fn settings_json(&self) -> Option<Value> {
        self.settings.as_ref().map(Value::from)
    }

    /// Returns whether both the section and settings are absent.
    pub fn is_empty(&self) -> bool {
        self.section.is_none() && self.settings.is_none()
    }
}

/// Recursive JSON-like workspace configuration value stored in declarative LSP specs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceConfigurationValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<WorkspaceConfigurationValue>),
    Object(BTreeMap<String, WorkspaceConfigurationValue>),
}

impl WorkspaceConfigurationValue {
    /// Creates a null configuration value.
    pub const fn null() -> Self {
        Self::Null
    }

    /// Creates an integer configuration value.
    pub fn integer(value: i64) -> Self {
        Self::Number(value.into())
    }

    /// Creates an unsigned integer configuration value.
    pub fn unsigned(value: u64) -> Self {
        Self::Number(value.into())
    }

    /// Creates a floating-point configuration value when the input is finite.
    pub fn float(value: f64) -> Option<Self> {
        Number::from_f64(value).map(Self::Number)
    }

    /// Parses a JSON number string into a configuration value.
    pub fn from_number_text(value: impl AsRef<str>) -> Option<Self> {
        value.as_ref().parse::<Number>().ok().map(Self::Number)
    }

    /// Creates an array configuration value.
    pub fn array<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Array(values.into_iter().collect())
    }

    /// Creates an object configuration value.
    pub fn object<I, K>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, Self)>,
        K: Into<String>,
    {
        let mut object = BTreeMap::new();
        for (key, value) in entries {
            object.insert(key.into(), value);
        }
        Self::Object(object)
    }

    /// Returns true when the value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns the inner boolean when this value is a bool.
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the inner number when this value is numeric.
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Self::Number(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the inner string when this value is textual.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the inner array when this value is an array.
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    /// Returns the inner object when this value is an object.
    pub fn as_object(&self) -> Option<&BTreeMap<String, Self>> {
        match self {
            Self::Object(values) => Some(values),
            _ => None,
        }
    }

    /// Converts this value into a JSON value.
    pub fn to_json_value(&self) -> Value {
        Value::from(self)
    }
}

impl From<bool> for WorkspaceConfigurationValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for WorkspaceConfigurationValue {
    fn from(value: i64) -> Self {
        Self::integer(value)
    }
}

impl From<u64> for WorkspaceConfigurationValue {
    fn from(value: u64) -> Self {
        Self::unsigned(value)
    }
}

impl From<Number> for WorkspaceConfigurationValue {
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

impl From<String> for WorkspaceConfigurationValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for WorkspaceConfigurationValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl<T> From<Vec<T>> for WorkspaceConfigurationValue
where
    T: Into<WorkspaceConfigurationValue>,
{
    fn from(value: Vec<T>) -> Self {
        Self::Array(value.into_iter().map(Into::into).collect())
    }
}

impl<K, V> From<BTreeMap<K, V>> for WorkspaceConfigurationValue
where
    K: Into<String> + Ord,
    V: Into<WorkspaceConfigurationValue>,
{
    fn from(value: BTreeMap<K, V>) -> Self {
        Self::Object(
            value
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }
}

impl From<&WorkspaceConfigurationValue> for Value {
    fn from(value: &WorkspaceConfigurationValue) -> Self {
        match value {
            WorkspaceConfigurationValue::Null => Self::Null,
            WorkspaceConfigurationValue::Bool(value) => Self::Bool(*value),
            WorkspaceConfigurationValue::Number(value) => Self::Number(value.clone()),
            WorkspaceConfigurationValue::String(value) => Self::String(value.clone()),
            WorkspaceConfigurationValue::Array(values) => {
                Self::Array(values.iter().map(Value::from).collect())
            }
            WorkspaceConfigurationValue::Object(values) => Self::Object(
                values
                    .iter()
                    .map(|(key, value)| (key.clone(), Value::from(value)))
                    .collect(),
            ),
        }
    }
}

impl From<WorkspaceConfigurationValue> for Value {
    fn from(value: WorkspaceConfigurationValue) -> Self {
        match value {
            WorkspaceConfigurationValue::Null => Self::Null,
            WorkspaceConfigurationValue::Bool(value) => Self::Bool(value),
            WorkspaceConfigurationValue::Number(value) => Self::Number(value),
            WorkspaceConfigurationValue::String(value) => Self::String(value),
            WorkspaceConfigurationValue::Array(values) => {
                Self::Array(values.into_iter().map(Value::from).collect())
            }
            WorkspaceConfigurationValue::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, Value::from(value)))
                    .collect(),
            ),
        }
    }
}

impl From<Value> for WorkspaceConfigurationValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(value) => Self::Bool(value),
            Value::Number(value) => Self::Number(value),
            Value::String(value) => Self::String(value),
            Value::Array(values) => Self::Array(values.into_iter().map(Into::into).collect()),
            Value::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, WorkspaceConfigurationValue::from(value)))
                    .collect(),
            ),
        }
    }
}

/// Declarative language-server specification compiled into the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageServerSpec {
    id: String,
    language_id: String,
    file_extensions: Vec<String>,
    file_names: Vec<String>,
    file_globs: Vec<String>,
    document_language_ids: BTreeMap<String, String>,
    program: String,
    args: Vec<String>,
    root_markers: Vec<String>,
    root_strategy: LanguageServerRootStrategy,
    env: Vec<(String, String)>,
    workspace_configuration: WorkspaceConfiguration,
    path_matcher: PathMatcher,
}

/// Strategy used to choose the LSP workspace root for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageServerRootStrategy {
    /// Reuse the editor workspace root as-is.
    #[default]
    Workspace,
    /// Prefer the nearest configured root marker for the current file and fall back to the editor
    /// workspace root when no marker matches.
    MarkersOrWorkspace,
}

impl LanguageServerSpec {
    /// Creates a new language-server specification.
    pub fn new(
        id: impl Into<String>,
        language_id: impl Into<String>,
        file_extensions: impl IntoIterator<Item = impl Into<String>>,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let file_extensions = file_extensions
            .into_iter()
            .map(|extension| normalize_extension(&extension.into()))
            .collect::<Vec<_>>();
        Self {
            id: id.into(),
            language_id: language_id.into(),
            file_extensions: file_extensions.clone(),
            file_names: Vec::new(),
            file_globs: Vec::new(),
            document_language_ids: BTreeMap::new(),
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            root_markers: Vec::new(),
            root_strategy: LanguageServerRootStrategy::Workspace,
            env: Vec::new(),
            workspace_configuration: WorkspaceConfiguration::default(),
            path_matcher: PathMatcher::from_parts(
                &file_extensions,
                [] as [&str; 0],
                [] as [&str; 0],
            ),
        }
    }

    /// Adds root markers used for workspace discovery.
    pub fn with_root_markers(
        mut self,
        markers: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.root_markers = markers.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the workspace-root strategy for this server.
    pub fn with_root_strategy(mut self, strategy: LanguageServerRootStrategy) -> Self {
        self.root_strategy = strategy;
        self
    }

    /// Overrides the LSP document language id for specific extensions, basenames, or globs.
    pub fn with_document_language_ids<I, E, L>(mut self, mappings: I) -> Self
    where
        I: IntoIterator<Item = (E, L)>,
        E: Into<String>,
        L: Into<String>,
    {
        for (path_matcher, language_id) in mappings {
            let path_matcher = path_matcher.into();
            let path_matcher = path_matcher.trim();
            let language_id = language_id.into();
            if path_matcher.is_empty() || language_id.is_empty() {
                continue;
            }
            self.document_language_ids
                .insert(path_matcher.to_owned(), language_id);
        }
        self
    }

    /// Adds an environment override.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Sets the workspace configuration section used for server-specific settings.
    pub fn with_workspace_configuration_section(mut self, section: impl Into<String>) -> Self {
        self.workspace_configuration = self.workspace_configuration.with_section(section);
        self
    }

    /// Sets the server-specific workspace settings payload.
    pub fn with_workspace_configuration_settings(
        mut self,
        settings: impl Into<WorkspaceConfigurationValue>,
    ) -> Self {
        self.workspace_configuration = self.workspace_configuration.with_settings(settings);
        self
    }

    /// Sets both the workspace configuration section and settings payload.
    pub fn with_workspace_configuration(
        mut self,
        section: impl Into<String>,
        settings: impl Into<WorkspaceConfigurationValue>,
    ) -> Self {
        self.workspace_configuration = self
            .workspace_configuration
            .with_section(section)
            .with_settings(settings);
        self
    }

    /// Creates a workspace settings object without importing the underlying value type.
    pub fn workspace_settings_object<I, K>(entries: I) -> WorkspaceConfigurationValue
    where
        I: IntoIterator<Item = (K, WorkspaceConfigurationValue)>,
        K: Into<String>,
    {
        WorkspaceConfigurationValue::object(entries)
    }

    /// Creates a workspace settings array without importing the underlying value type.
    pub fn workspace_settings_array<I>(values: I) -> WorkspaceConfigurationValue
    where
        I: IntoIterator<Item = WorkspaceConfigurationValue>,
    {
        WorkspaceConfigurationValue::array(values)
    }

    /// Creates a null workspace setting value.
    pub const fn workspace_settings_null() -> WorkspaceConfigurationValue {
        WorkspaceConfigurationValue::Null
    }

    /// Creates a floating-point workspace setting value when the input is finite.
    pub fn workspace_settings_float(value: f64) -> Option<WorkspaceConfigurationValue> {
        WorkspaceConfigurationValue::float(value)
    }

    /// Returns the server identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the language identifier.
    pub fn language_id(&self) -> &str {
        &self.language_id
    }

    /// Returns the file extensions handled by this server.
    pub fn file_extensions(&self) -> &[String] {
        &self.file_extensions
    }

    /// Adds exact basenames handled by this server.
    pub fn with_file_names<I, S>(mut self, file_names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_names = normalize_unique_entries(file_names);
        self.rebuild_path_matcher();
        self
    }

    /// Returns the exact basenames handled by this server.
    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }

    /// Adds basename globs handled by this server.
    pub fn with_file_globs<I, S>(mut self, file_globs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_globs = normalize_unique_entries(file_globs);
        self.rebuild_path_matcher();
        self
    }

    /// Returns the basename globs handled by this server.
    pub fn file_globs(&self) -> &[String] {
        &self.file_globs
    }

    /// Returns the LSP document language id for a file extension.
    pub fn document_language_id_for_extension(&self, extension: &str) -> &str {
        document_language_id_for_extension(
            &self.document_language_ids,
            extension,
            &self.language_id,
        )
    }

    /// Returns the program executable.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Returns the program arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns root markers for workspace discovery.
    pub fn root_markers(&self) -> &[String] {
        &self.root_markers
    }

    /// Returns the path-matcher-to-language-id overrides.
    pub fn document_language_ids(&self) -> &BTreeMap<String, String> {
        &self.document_language_ids
    }

    /// Returns the workspace-root strategy for this server.
    pub const fn root_strategy(&self) -> LanguageServerRootStrategy {
        self.root_strategy
    }

    /// Returns the environment overrides used when launching the server.
    pub fn env(&self) -> &[(String, String)] {
        &self.env
    }

    /// Returns the declared workspace configuration.
    pub fn workspace_configuration(&self) -> &WorkspaceConfiguration {
        &self.workspace_configuration
    }

    /// Returns the workspace configuration section, if one is declared.
    pub fn workspace_configuration_section(&self) -> Option<&str> {
        self.workspace_configuration.section()
    }

    /// Returns the workspace settings payload, if one is declared.
    pub fn workspace_configuration_settings(&self) -> Option<&WorkspaceConfigurationValue> {
        self.workspace_configuration.settings()
    }

    /// Returns the workspace settings payload as a JSON value.
    pub fn workspace_configuration_settings_json(&self) -> Option<Value> {
        self.workspace_configuration.settings_json()
    }

    /// Returns whether this server should attach to the provided path.
    pub fn matches_path(&self, path: &Path) -> bool {
        self.path_match_score(path).is_some()
    }

    /// Returns the launch spec used to start the server.
    pub fn launch_job(&self, root: Option<PathBuf>) -> JobSpec {
        let mut job = JobSpec::command(
            format!("lsp:{}", self.id),
            self.program.clone(),
            self.args.clone(),
        );
        if let Some(root) = root {
            job = job.with_cwd(root);
        }
        for (key, value) in &self.env {
            job = job.with_env(key.clone(), value.clone());
        }
        job
    }

    fn planned_root_for_path(&self, path: &Path, workspace_root: Option<&Path>) -> Option<PathBuf> {
        match self.root_strategy {
            LanguageServerRootStrategy::Workspace => workspace_root.map(Path::to_path_buf),
            LanguageServerRootStrategy::MarkersOrWorkspace => {
                find_root_for_path(path, workspace_root, &self.root_markers)
                    .or_else(|| workspace_root.map(Path::to_path_buf))
            }
        }
    }

    fn rebuild_path_matcher(&mut self) {
        self.path_matcher =
            PathMatcher::from_parts(&self.file_extensions, &self.file_names, &self.file_globs);
    }

    fn path_match_score(&self, path: &Path) -> Option<usize> {
        self.path_matcher.best_match_score(path)
    }
}

/// Prepared session plan for an LSP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageServerSession {
    server_id: String,
    language_id: String,
    document_language_ids: BTreeMap<String, String>,
    root: Option<PathBuf>,
    launch: JobSpec,
    workspace_configuration: WorkspaceConfiguration,
    diagnostics: Vec<Diagnostic>,
}

impl LanguageServerSession {
    /// Returns the server identifier.
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    /// Returns the language identifier.
    pub fn language_id(&self) -> &str {
        &self.language_id
    }

    /// Sets the workspace configuration section used for this planned session.
    pub fn with_workspace_configuration_section(mut self, section: impl Into<String>) -> Self {
        self.workspace_configuration = self.workspace_configuration.with_section(section);
        self
    }

    /// Sets the workspace settings payload used for this planned session.
    pub fn with_workspace_configuration_settings(
        mut self,
        settings: impl Into<WorkspaceConfigurationValue>,
    ) -> Self {
        self.workspace_configuration = self.workspace_configuration.with_settings(settings);
        self
    }

    /// Sets both the workspace configuration section and settings payload.
    pub fn with_workspace_configuration(
        mut self,
        section: impl Into<String>,
        settings: impl Into<WorkspaceConfigurationValue>,
    ) -> Self {
        self.workspace_configuration = self
            .workspace_configuration
            .with_section(section)
            .with_settings(settings);
        self
    }

    /// Returns the document language id that should be sent for a file path.
    pub fn document_language_id_for_path(&self, path: &Path) -> &str {
        let file_name = path.file_name().and_then(|name| name.to_str());
        let extension = path.extension().and_then(|value| value.to_str());
        document_language_id_for_path(
            &self.document_language_ids,
            file_name,
            extension,
            &self.language_id,
        )
    }

    /// Returns the planned workspace root.
    pub fn root(&self) -> Option<&PathBuf> {
        self.root.as_ref()
    }

    /// Returns the launch spec.
    pub fn launch(&self) -> &JobSpec {
        &self.launch
    }

    /// Returns accumulated diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns the declared workspace configuration for this planned session.
    pub fn workspace_configuration(&self) -> &WorkspaceConfiguration {
        &self.workspace_configuration
    }

    /// Returns the workspace configuration section, if one is declared.
    pub fn workspace_configuration_section(&self) -> Option<&str> {
        self.workspace_configuration.section()
    }

    /// Returns the workspace settings payload, if one is declared.
    pub fn workspace_configuration_settings(&self) -> Option<&WorkspaceConfigurationValue> {
        self.workspace_configuration.settings()
    }

    /// Returns the workspace settings payload as a JSON value.
    pub fn workspace_configuration_settings_json(&self) -> Option<Value> {
        self.workspace_configuration.settings_json()
    }

    /// Replaces the diagnostic set.
    pub fn with_diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
        self.diagnostics = diagnostics;
        self
    }
}

/// Errors produced by LSP registry operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspError {
    /// Duplicate server id registration.
    DuplicateServerId(String),
    /// Duplicate extension registration.
    DuplicateExtension(String),
    /// Unknown server id lookup.
    UnknownServer(String),
    /// Unknown extension lookup.
    UnknownExtension(String),
}

impl fmt::Display for LspError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateServerId(server_id) => {
                write!(
                    formatter,
                    "language server `{server_id}` is already registered"
                )
            }
            Self::DuplicateExtension(extension) => {
                write!(
                    formatter,
                    "extension `{extension}` is already mapped to a server"
                )
            }
            Self::UnknownServer(server_id) => {
                write!(formatter, "language server `{server_id}` is not registered")
            }
            Self::UnknownExtension(extension) => {
                write!(formatter, "no language server registered for `{extension}`")
            }
        }
    }
}

impl Error for LspError {}

/// Registry of known language-server specifications.
#[derive(Debug, Default, Clone)]
pub struct LanguageServerRegistry {
    servers: BTreeMap<String, LanguageServerSpec>,
    server_order: Vec<String>,
    extensions: BTreeMap<String, Vec<String>>,
}

impl LanguageServerRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of registered servers.
    pub fn len(&self) -> usize {
        self.servers.len()
    }

    /// Returns whether no servers are registered.
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }

    /// Registers a new language-server specification.
    pub fn register(&mut self, spec: LanguageServerSpec) -> Result<(), LspError> {
        let server_id = spec.id().to_owned();
        if self.servers.contains_key(&server_id) {
            return Err(LspError::DuplicateServerId(server_id));
        }
        for extension in spec.file_extensions() {
            self.extensions
                .entry(extension.clone())
                .or_default()
                .push(server_id.clone());
        }
        self.server_order.push(server_id.clone());
        self.servers.insert(server_id, spec);
        Ok(())
    }

    /// Registers many language servers.
    pub fn register_all<I>(&mut self, specs: I) -> Result<(), LspError>
    where
        I: IntoIterator<Item = LanguageServerSpec>,
    {
        for spec in specs {
            self.register(spec)?;
        }
        Ok(())
    }

    /// Returns a server by identifier.
    pub fn server(&self, server_id: &str) -> Option<&LanguageServerSpec> {
        self.servers.get(server_id)
    }

    /// Returns a server for a file extension, if one is registered.
    pub fn server_for_extension(&self, extension: &str) -> Option<&LanguageServerSpec> {
        self.servers_for_extension(extension).into_iter().next()
    }

    /// Returns all servers for a file extension, preserving registration order.
    pub fn servers_for_extension(&self, extension: &str) -> Vec<&LanguageServerSpec> {
        let extension = normalize_extension(extension);
        self.extensions
            .get(&extension)
            .into_iter()
            .flat_map(|server_ids| server_ids.iter())
            .filter_map(|server_id| self.servers.get(server_id))
            .collect()
    }

    /// Returns the first server whose path matchers apply to the provided path.
    pub fn server_for_path(&self, path: &Path) -> Option<&LanguageServerSpec> {
        self.servers_for_path(path).into_iter().next()
    }

    /// Returns all servers whose path matchers apply to the provided path.
    pub fn servers_for_path(&self, path: &Path) -> Vec<&LanguageServerSpec> {
        let mut best_score: Option<usize> = None;
        for server_id in &self.server_order {
            let Some(server) = self.servers.get(server_id) else {
                continue;
            };
            let Some(score) = server.path_match_score(path) else {
                continue;
            };
            best_score = Some(best_score.map_or(score, |current| current.max(score)));
        }

        let Some(best_score) = best_score else {
            return Vec::new();
        };

        self.server_order
            .iter()
            .filter_map(|server_id| {
                let server = self.servers.get(server_id)?;
                (server.path_match_score(path) == Some(best_score)).then_some(server)
            })
            .collect()
    }

    /// Prepares a session by explicit server identifier.
    pub fn prepare_session(
        &self,
        server_id: &str,
        root: Option<PathBuf>,
    ) -> Result<LanguageServerSession, LspError> {
        let spec = self
            .servers
            .get(server_id)
            .ok_or_else(|| LspError::UnknownServer(server_id.to_owned()))?;
        Ok(LanguageServerSession {
            server_id: spec.id().to_owned(),
            language_id: spec.language_id().to_owned(),
            document_language_ids: spec.document_language_ids.clone(),
            launch: spec.launch_job(root.clone()),
            root,
            workspace_configuration: spec.workspace_configuration.clone(),
            diagnostics: Vec::new(),
        })
    }

    /// Prepares a session for a file path, resolving the root from the server strategy.
    pub fn prepare_session_for_path(
        &self,
        server_id: &str,
        path: &Path,
        workspace_root: Option<&Path>,
    ) -> Result<LanguageServerSession, LspError> {
        let spec = self
            .servers
            .get(server_id)
            .ok_or_else(|| LspError::UnknownServer(server_id.to_owned()))?;
        let root = spec.planned_root_for_path(path, workspace_root);
        Ok(LanguageServerSession {
            server_id: spec.id().to_owned(),
            language_id: spec.language_id().to_owned(),
            document_language_ids: spec.document_language_ids.clone(),
            launch: spec.launch_job(root.clone()),
            root,
            workspace_configuration: spec.workspace_configuration.clone(),
            diagnostics: Vec::new(),
        })
    }

    /// Prepares a session by file extension.
    pub fn prepare_session_for_extension(
        &self,
        extension: &str,
        root: Option<PathBuf>,
    ) -> Result<LanguageServerSession, LspError> {
        let extension = normalize_extension(extension);
        let server = self
            .server_for_extension(&extension)
            .ok_or_else(|| LspError::UnknownExtension(extension.clone()))?;
        self.prepare_session(server.id(), root)
    }

    /// Prepares sessions for every server registered to an extension.
    pub fn prepare_sessions_for_extension(
        &self,
        extension: &str,
        root: Option<PathBuf>,
    ) -> Result<Vec<LanguageServerSession>, LspError> {
        let extension = normalize_extension(extension);
        let servers = self.servers_for_extension(&extension);
        if servers.is_empty() {
            return Err(LspError::UnknownExtension(extension));
        }
        let mut sessions = Vec::with_capacity(servers.len());
        for server in servers {
            sessions.push(self.prepare_session(server.id(), root.clone())?);
        }
        Ok(sessions)
    }

    /// Prepares sessions for a file path, resolving roots from each server strategy.
    pub fn prepare_sessions_for_path(
        &self,
        path: &Path,
        workspace_root: Option<&Path>,
    ) -> Result<Vec<LanguageServerSession>, LspError> {
        let servers = self.servers_for_path(path);
        if servers.is_empty() {
            return Err(LspError::UnknownExtension(path.display().to_string()));
        }
        let mut sessions = Vec::with_capacity(servers.len());
        for server in servers {
            sessions.push(self.prepare_session_for_path(server.id(), path, workspace_root)?);
        }
        Ok(sessions)
    }
}

fn normalize_unique_entries<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for value in values {
        let value = value.into();
        let value = value.trim();
        if !value.is_empty() && !normalized.iter().any(|existing| existing == value) {
            normalized.push(value.to_owned());
        }
    }
    normalized
}

fn normalize_optional_string(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn document_language_id_for_path<'a>(
    document_language_ids: &'a BTreeMap<String, String>,
    file_name: Option<&str>,
    extension: Option<&str>,
    default_language_id: &'a str,
) -> &'a str {
    if let Some(file_name) = file_name {
        if let Some(language_id) = document_language_ids.get(file_name) {
            return language_id;
        }
        if let Some(language_id) = document_language_id_for_glob(document_language_ids, file_name) {
            return language_id;
        }
    }
    if let Some(extension) = extension {
        return document_language_id_for_extension(
            document_language_ids,
            extension,
            default_language_id,
        );
    }
    default_language_id
}

fn document_language_id_for_extension<'a>(
    document_language_ids: &'a BTreeMap<String, String>,
    extension: &str,
    default_language_id: &'a str,
) -> &'a str {
    let extension = normalize_extension(extension);
    document_language_ids
        .iter()
        .find_map(|(path_matcher, language_id)| {
            (normalize_extension(path_matcher) == extension).then_some(language_id.as_str())
        })
        .unwrap_or(default_language_id)
}

fn document_language_id_for_glob<'a>(
    document_language_ids: &'a BTreeMap<String, String>,
    file_name: &str,
) -> Option<&'a str> {
    let mut best = None;
    let mut best_score = 0;
    for (path_matcher, language_id) in document_language_ids {
        let Some(path_matcher) = PathPattern::glob(path_matcher) else {
            continue;
        };
        let Some(score) = path_matcher.match_score_for_file_name(file_name) else {
            continue;
        };
        if best.is_none() || score > best_score {
            best = Some(language_id.as_str());
            best_score = score;
        }
    }
    best
}

fn find_root_for_path(
    path: &Path,
    workspace_root: Option<&Path>,
    root_markers: &[String],
) -> Option<PathBuf> {
    if root_markers.is_empty() {
        return None;
    }
    let workspace_root = workspace_root.filter(|root| path.starts_with(root));
    let mut current = path.parent();
    while let Some(directory) = current {
        if directory_matches_root_markers(directory, root_markers) {
            return Some(directory.to_path_buf());
        }
        if workspace_root.is_some_and(|root| root == directory) {
            break;
        }
        current = directory.parent();
    }
    None
}

fn directory_matches_root_markers(directory: &Path, root_markers: &[String]) -> bool {
    root_markers
        .iter()
        .any(|marker| directory_matches_root_marker(directory, marker))
}

fn directory_matches_root_marker(directory: &Path, marker: &str) -> bool {
    if let Some(extension) = marker.strip_prefix("*.") {
        return directory_contains_extension(directory, extension);
    }
    fs::metadata(directory.join(marker)).is_ok()
}

fn directory_contains_extension(directory: &Path, extension: &str) -> bool {
    let Ok(entries) = fs::read_dir(directory) else {
        return false;
    };
    let extension = extension.to_ascii_lowercase();
    entries.filter_map(Result::ok).any(|entry| {
        entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(&extension))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use editor_buffer::{TextPoint, TextRange};
    use serde_json::json;

    use super::{
        Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerRootStrategy,
        LanguageServerSpec, WorkspaceConfigurationValue,
    };

    fn rust_analyzer() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "rust-analyzer",
            ["--stdio"],
        )
        .with_root_markers(["Cargo.toml", "rust-project.json"])
    }

    fn typescript_language_server() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "typescript-language-server",
            "typescript",
            ["ts", "tsx", "js", "jsx"],
            "typescript-language-server",
            ["--stdio"],
        )
        .with_document_language_ids([
            ("tsx", "typescriptreact"),
            ("js", "javascript"),
            ("jsx", "javascriptreact"),
        ])
    }

    fn csharp_language_server() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "csharp-ls",
            "csharp",
            ["cs"],
            "csharp-ls",
            ["--features", "razor-support,metadata-uris"],
        )
        .with_root_markers(["*.sln", "*.csproj", "global.json"])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
    }

    fn dockerfile_language_server() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "dockerfile-language-server",
            "dockerfile",
            [] as [&str; 0],
            "dockerfile-language-server",
            ["--stdio"],
        )
        .with_file_names(["Dockerfile"])
        .with_file_globs(["Dockerfile.*"])
        .with_document_language_ids([("Dockerfile", "dockerfile"), ("Dockerfile.*", "dockerfile")])
    }

    fn dev_extension_server() -> LanguageServerSpec {
        LanguageServerSpec::new("dev-server", "dev", ["dev"], "dev-server", ["--stdio"])
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-editor-lsp-{unique}"))
    }

    #[test]
    fn registry_resolves_rust_server_by_extension() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(rust_analyzer()));

        let server = registry.server_for_extension(".rs").expect("server");
        assert_eq!(server.id(), "rust-analyzer");
        assert_eq!(server.root_markers(), ["Cargo.toml", "rust-project.json"]);
    }

    #[test]
    fn registry_allows_multiple_servers_for_extension() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(LanguageServerSpec::new(
            "harper",
            "markdown",
            ["md"],
            "harper-ls",
            ["--stdio"],
        )));
        must(registry.register(LanguageServerSpec::new(
            "marksman",
            "markdown",
            ["md"],
            "marksman",
            ["server"],
        )));

        let servers = registry.servers_for_extension(".md");
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].id(), "harper");
        assert_eq!(servers[1].id(), "marksman");
    }

    #[test]
    fn registry_resolves_servers_by_filename_and_glob() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(dockerfile_language_server()));
        let dev_dockerfile = Path::new("containers").join("Dockerfile.dev");

        assert_eq!(
            registry
                .server_for_path(Path::new("Dockerfile"))
                .map(|server| server.id()),
            Some("dockerfile-language-server")
        );
        assert_eq!(
            registry
                .server_for_path(&dev_dockerfile)
                .map(|server| server.id()),
            Some("dockerfile-language-server")
        );
    }

    #[test]
    fn registry_prefers_filename_globs_over_extension_matches() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(dev_extension_server()));
        must(registry.register(dockerfile_language_server()));
        let dev_dockerfile = Path::new("containers").join("Dockerfile.dev");

        let servers = registry.servers_for_path(&dev_dockerfile);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id(), "dockerfile-language-server");
    }

    #[test]
    fn prepared_session_contains_launch_spec_and_diagnostics() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(rust_analyzer()));

        let session = must(
            registry
                .prepare_session("rust-analyzer", Some(PathBuf::from("P:\\volt")))
                .map(|session| {
                    session.with_diagnostics(vec![Diagnostic::new(
                        "rust-analyzer",
                        "Example diagnostic",
                        DiagnosticSeverity::Warning,
                        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
                    )])
                }),
        );

        assert_eq!(session.server_id(), "rust-analyzer");
        assert_eq!(session.language_id(), "rust");
        assert_eq!(session.launch().program(), "rust-analyzer");
        assert_eq!(session.launch().args(), ["--stdio"]);
        assert_eq!(session.diagnostics().len(), 1);
    }

    #[test]
    fn workspace_configuration_value_round_trips_through_json() {
        let value = WorkspaceConfigurationValue::from(json!({
            "csharp": {
                "format.enable": true,
                "maxLineLength": 120.0,
                "inlayHints": ["types", null],
            }
        }));

        assert_eq!(
            value.to_json_value(),
            json!({
                "csharp": {
                    "format.enable": true,
                    "maxLineLength": 120.0,
                    "inlayHints": ["types", null],
                }
            })
        );

        let csharp = value
            .as_object()
            .and_then(|settings| settings.get("csharp"))
            .and_then(WorkspaceConfigurationValue::as_object)
            .expect("csharp object");
        assert_eq!(
            csharp
                .get("format.enable")
                .and_then(WorkspaceConfigurationValue::as_bool),
            Some(true)
        );
        assert_eq!(
            csharp
                .get("maxLineLength")
                .and_then(WorkspaceConfigurationValue::as_number)
                .and_then(serde_json::Number::as_f64),
            Some(120.0)
        );
        let hints = csharp
            .get("inlayHints")
            .and_then(WorkspaceConfigurationValue::as_array)
            .expect("hint array");
        assert_eq!(hints[0].as_str(), Some("types"));
        assert!(hints[1].is_null());
    }

    #[test]
    fn language_server_spec_exposes_workspace_configuration_builders() {
        let spec = LanguageServerSpec::new(
            "csharp-ls",
            "csharp",
            ["cs"],
            "csharp-ls",
            ["--features", "razor-support,metadata-uris"],
        )
        .with_workspace_configuration_section("csharp")
        .with_workspace_configuration_settings(
            LanguageServerSpec::workspace_settings_object([(
                "csharp",
                LanguageServerSpec::workspace_settings_object([
                    (
                        "enableAnalyzersSupport",
                        WorkspaceConfigurationValue::from(true),
                    ),
                    (
                        "inlayHints",
                        LanguageServerSpec::workspace_settings_array([
                            WorkspaceConfigurationValue::from("types"),
                            LanguageServerSpec::workspace_settings_null(),
                        ]),
                    ),
                    (
                        "maxLineLength",
                        LanguageServerSpec::workspace_settings_float(120.0).expect("finite float"),
                    ),
                ]),
            )]),
        );

        assert_eq!(spec.workspace_configuration().section(), Some("csharp"));
        assert_eq!(spec.workspace_configuration_section(), Some("csharp"));
        assert_eq!(
            spec.workspace_configuration_settings_json(),
            Some(json!({
                "csharp": {
                    "enableAnalyzersSupport": true,
                    "inlayHints": ["types", null],
                    "maxLineLength": 120.0,
                }
            }))
        );
    }

    #[test]
    fn prepared_session_carries_workspace_configuration_from_spec() {
        let mut registry = LanguageServerRegistry::new();
        must(
            registry.register(
                LanguageServerSpec::new(
                    "csharp-ls",
                    "csharp",
                    ["cs"],
                    "csharp-ls",
                    ["--features", "razor-support,metadata-uris"],
                )
                .with_workspace_configuration(
                    "csharp",
                    LanguageServerSpec::workspace_settings_object([(
                        "csharp",
                        LanguageServerSpec::workspace_settings_object([
                            (
                                "enableAnalyzersSupport",
                                WorkspaceConfigurationValue::from(true),
                            ),
                            ("sdk", WorkspaceConfigurationValue::from("dotnet")),
                        ]),
                    )]),
                ),
            ),
        );

        let session = must(registry.prepare_session("csharp-ls", Some(PathBuf::from("P:\\volt"))));

        assert_eq!(session.workspace_configuration().section(), Some("csharp"));
        assert_eq!(session.workspace_configuration_section(), Some("csharp"));
        assert_eq!(
            session.workspace_configuration_settings_json(),
            Some(json!({
                "csharp": {
                    "enableAnalyzersSupport": true,
                    "sdk": "dotnet",
                }
            }))
        );

        let overridden = session.with_workspace_configuration_settings(
            LanguageServerSpec::workspace_settings_object([("logging", true.into())]),
        );
        assert_eq!(
            overridden.workspace_configuration_settings_json(),
            Some(json!({
                "logging": true,
            }))
        );
    }

    #[test]
    fn prepared_session_resolves_document_language_ids_per_extension() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(typescript_language_server()));

        let session = must(registry.prepare_session(
            "typescript-language-server",
            Some(PathBuf::from("P:\\volt")),
        ));

        assert_eq!(session.language_id(), "typescript");
        assert_eq!(
            session.document_language_id_for_path(Path::new("app.ts")),
            "typescript"
        );
        assert_eq!(
            session.document_language_id_for_path(Path::new("app.tsx")),
            "typescriptreact"
        );
        assert_eq!(
            session.document_language_id_for_path(Path::new("app.js")),
            "javascript"
        );
        assert_eq!(
            session.document_language_id_for_path(Path::new("app.jsx")),
            "javascriptreact"
        );
    }

    #[test]
    fn prepared_session_resolves_document_language_ids_by_filename_and_glob() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(dockerfile_language_server()));
        let dev_dockerfile = Path::new("containers").join("Dockerfile.dev");

        let session = must(registry.prepare_session(
            "dockerfile-language-server",
            Some(PathBuf::from("P:\\volt")),
        ));

        assert_eq!(
            session.document_language_id_for_path(Path::new("Dockerfile")),
            "dockerfile"
        );
        assert_eq!(
            session.document_language_id_for_path(&dev_dockerfile),
            "dockerfile"
        );
    }

    #[test]
    fn prepared_session_for_path_prefers_nearest_matching_root_marker() {
        let root = temp_dir();
        let project_dir = root.join("src").join("AssetFusion.Api");
        fs::create_dir_all(&project_dir).expect("project dir");
        fs::write(root.join("af-platform-api.sln"), "").expect("solution");
        fs::write(project_dir.join("AssetFusion.Api.csproj"), "").expect("project");
        let file_path = project_dir.join("Program.cs");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session =
            must(registry.prepare_session_for_path("csharp-ls", &file_path, Some(root.as_path())));
        assert_eq!(session.root(), Some(&project_dir));
        assert_eq!(session.launch().cwd(), Some(&project_dir));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepared_session_for_path_falls_back_to_workspace_root_when_markers_do_not_match() {
        let root = temp_dir();
        let file_path = root.join("src").join("Program.cs");
        fs::create_dir_all(file_path.parent().expect("parent")).expect("dir");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session =
            must(registry.prepare_session_for_path("csharp-ls", &file_path, Some(root.as_path())));
        assert_eq!(session.root(), Some(&root));
        assert_eq!(session.launch().cwd(), Some(&root));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_sessions_for_extension_returns_all_matching_servers() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(LanguageServerSpec::new(
            "harper",
            "markdown",
            ["md"],
            "harper-ls",
            ["--stdio"],
        )));
        must(registry.register(LanguageServerSpec::new(
            "marksman",
            "markdown",
            ["md"],
            "marksman",
            ["server"],
        )));

        let sessions =
            must(registry.prepare_sessions_for_extension("md", Some(PathBuf::from("P:\\volt"))));
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].server_id(), "harper");
        assert_eq!(sessions[1].server_id(), "marksman");
    }

    #[test]
    fn prepare_sessions_for_path_returns_filename_matches_without_extensions() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(dockerfile_language_server()));

        let sessions = must(
            registry.prepare_sessions_for_path(Path::new("Dockerfile"), Some(Path::new("P:\\"))),
        );
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].server_id(), "dockerfile-language-server");
    }
}
