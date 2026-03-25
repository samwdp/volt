#![doc = r#"Language Server Protocol registry, session plans, diagnostics, launch metadata, and client runtime management."#]

mod client;

use std::{collections::BTreeMap, error::Error, fmt, path::PathBuf};

use editor_buffer::TextRange;
use editor_jobs::JobSpec;

pub use client::{
    LspClientError, LspClientManager, LspCompletionItem, LspCompletionKind, LspHoverContents,
    LspLogDirection, LspLogEntry, LspLogSnapshot,
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

/// Declarative language-server specification compiled into the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageServerSpec {
    id: String,
    language_id: String,
    file_extensions: Vec<String>,
    program: String,
    args: Vec<String>,
    root_markers: Vec<String>,
    env: Vec<(String, String)>,
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
        Self {
            id: id.into(),
            language_id: language_id.into(),
            file_extensions: file_extensions
                .into_iter()
                .map(|extension| normalize_extension(&extension.into()))
                .collect(),
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            root_markers: Vec::new(),
            env: Vec::new(),
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

    /// Adds an environment override.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
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
}

/// Prepared session plan for an LSP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageServerSession {
    server_id: String,
    language_id: String,
    root: Option<PathBuf>,
    launch: JobSpec,
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
            launch: spec.launch_job(root.clone()),
            root,
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
}

fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use editor_buffer::{TextPoint, TextRange};

    use super::{Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerSpec};

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

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
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
}
