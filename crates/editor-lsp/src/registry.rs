use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use editor_buffer::TextRange;
use editor_jobs::JobSpec;
use editor_path::normalize_extension;
use serde_json::Value;

pub use editor_plugin_api::{
    InstallRecipe, LanguageServerRootStrategy, LanguageServerSpec, WorkspaceConfiguration,
    WorkspaceConfigurationValue,
};

use crate::workspace_roots::*;

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Language Server Protocol registry, session plans, diagnostics, launch metadata, and client runtime management.";

pub(crate) const CSHARP_LS_SERVER_ID: &str = "csharp-ls";

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
    pub(crate) source: String,
    pub(crate) message: String,
    pub(crate) severity: DiagnosticSeverity,
    pub(crate) range: TextRange,
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

/// Diagnostic paired with source server and file path for workspace-level listings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspWorkspaceDiagnostic {
    pub(crate) server_id: String,
    pub(crate) path: PathBuf,
    pub(crate) diagnostic: Diagnostic,
}

impl LspWorkspaceDiagnostic {
    /// Creates a workspace-level diagnostic entry.
    pub fn new(
        server_id: impl Into<String>,
        path: impl Into<PathBuf>,
        diagnostic: Diagnostic,
    ) -> Self {
        Self {
            server_id: server_id.into(),
            path: path.into(),
            diagnostic,
        }
    }

    /// Returns the language server identifier that reported this diagnostic.
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    /// Returns the file path for this diagnostic.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the diagnostic payload.
    pub fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

pub fn language_server_launch_job(spec: &LanguageServerSpec, root: Option<PathBuf>) -> JobSpec {
    let mut args = spec.args().to_vec();
    if spec.id() == CSHARP_LS_SERVER_ID
        && let Some(solution_name) = unique_solution_file_name(root.as_deref())
    {
        args.push("--solution".to_owned());
        args.push(solution_name);
    }
    let mut job = JobSpec::command(
        format!("lsp:{}", spec.id()),
        spec.program().to_owned(),
        args,
    );
    if let Some(root) = root {
        job = job.with_cwd(root);
    }
    for (key, value) in spec.env() {
        job = job.with_env(key.clone(), value.clone());
    }
    job
}

pub(crate) fn language_server_planned_root(
    spec: &LanguageServerSpec,
    path: &Path,
    workspace_root: Option<&Path>,
) -> Option<PathBuf> {
    match spec.root_strategy() {
        LanguageServerRootStrategy::Workspace => workspace_root.map(Path::to_path_buf),
        LanguageServerRootStrategy::MarkersOrWorkspace => {
            find_root_for_path(path, workspace_root, spec.root_markers())
                .or_else(|| workspace_root.map(Path::to_path_buf))
        }
    }
}

pub(crate) fn language_server_activation_matches_path(
    spec: &LanguageServerSpec,
    path: &Path,
    workspace_root: Option<&Path>,
) -> bool {
    spec.activation_markers().is_empty()
        || find_root_for_path(path, workspace_root, spec.activation_markers()).is_some()
}

/// Prepared session plan for an LSP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageServerSession {
    pub(crate) server_id: String,
    pub(crate) language_id: String,
    pub(crate) document_language_ids: BTreeMap<String, String>,
    pub(crate) root: Option<PathBuf>,
    pub(crate) launch: JobSpec,
    pub(crate) workspace_configuration: WorkspaceConfiguration,
    pub(crate) diagnostics: Vec<Diagnostic>,
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
    /// Matching server exists, but required project markers were not found.
    ActivationMarkersNotFound { server_id: String, path: String },
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
            Self::ActivationMarkersNotFound { server_id, path } => write!(
                formatter,
                "language server `{server_id}` is not available for `{path}` because required project markers were not found"
            ),
        }
    }
}

impl Error for LspError {}

/// Registry of known language-server specifications.
#[derive(Debug, Default, Clone)]
pub struct LanguageServerRegistry {
    pub(crate) servers: BTreeMap<String, LanguageServerSpec>,
    pub(crate) server_order: Vec<String>,
    pub(crate) extensions: BTreeMap<String, Vec<String>>,
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

    /// Returns registered servers in registration order.
    pub fn servers(&self) -> impl Iterator<Item = &LanguageServerSpec> {
        self.server_order
            .iter()
            .filter_map(|server_id| self.servers.get(server_id))
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
        self.matching_servers_for_path(path, true, None)
    }

    /// Returns all default-enabled servers whose path matchers apply to the provided path.
    pub fn default_enabled_servers_for_path(&self, path: &Path) -> Vec<&LanguageServerSpec> {
        self.default_enabled_servers_for_path_in_workspace(path, None)
    }

    /// Returns all default-enabled servers whose path matchers and activation markers apply.
    pub fn default_enabled_servers_for_path_in_workspace(
        &self,
        path: &Path,
        workspace_root: Option<&Path>,
    ) -> Vec<&LanguageServerSpec> {
        self.matching_servers_for_path(path, false, workspace_root)
    }

    fn matching_servers_for_path(
        &self,
        path: &Path,
        include_default_disabled: bool,
        workspace_root: Option<&Path>,
    ) -> Vec<&LanguageServerSpec> {
        let mut best_score: Option<usize> = None;
        for server_id in &self.server_order {
            let Some(server) = self.servers.get(server_id) else {
                continue;
            };
            if !include_default_disabled && !server.enabled_by_default() {
                continue;
            }
            if !include_default_disabled
                && !language_server_activation_matches_path(server, path, workspace_root)
            {
                continue;
            }
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
                if !include_default_disabled && !server.enabled_by_default() {
                    return None;
                }
                if !include_default_disabled
                    && !language_server_activation_matches_path(server, path, workspace_root)
                {
                    return None;
                }
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
            document_language_ids: spec.document_language_ids().clone(),
            launch: language_server_launch_job(spec, root.clone()),
            root,
            workspace_configuration: spec.workspace_configuration().clone(),
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
        if !language_server_activation_matches_path(spec, path, workspace_root) {
            return Err(LspError::ActivationMarkersNotFound {
                server_id: server_id.to_owned(),
                path: path.display().to_string(),
            });
        }
        let root = language_server_planned_root(spec, path, workspace_root);
        Ok(LanguageServerSession {
            server_id: spec.id().to_owned(),
            language_id: spec.language_id().to_owned(),
            document_language_ids: spec.document_language_ids().clone(),
            launch: language_server_launch_job(spec, root.clone()),
            root,
            workspace_configuration: spec.workspace_configuration().clone(),
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
        let servers = self.default_enabled_servers_for_path_in_workspace(path, workspace_root);
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
