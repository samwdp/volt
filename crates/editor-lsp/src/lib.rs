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

pub use client::{
    LspClientError, LspClientManager, LspCompletionItem, LspCompletionKind, LspFormattingOptions,
    LspHoverContents, LspLocation, LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotification,
    LspNotificationEntry, LspNotificationLevel, LspNotificationProgress, LspNotificationSnapshot,
    LspSignatureHelpContents, LspTextEdit,
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
    document_language_ids: BTreeMap<String, String>,
    program: String,
    args: Vec<String>,
    root_markers: Vec<String>,
    root_strategy: LanguageServerRootStrategy,
    env: Vec<(String, String)>,
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
        Self {
            id: id.into(),
            language_id: language_id.into(),
            file_extensions: file_extensions
                .into_iter()
                .map(|extension| normalize_extension(&extension.into()))
                .collect(),
            document_language_ids: BTreeMap::new(),
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            root_markers: Vec::new(),
            root_strategy: LanguageServerRootStrategy::Workspace,
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

    /// Sets the workspace-root strategy for this server.
    pub fn with_root_strategy(mut self, strategy: LanguageServerRootStrategy) -> Self {
        self.root_strategy = strategy;
        self
    }

    /// Overrides the LSP document language id for specific file extensions.
    pub fn with_document_language_ids<I, E, L>(mut self, mappings: I) -> Self
    where
        I: IntoIterator<Item = (E, L)>,
        E: Into<String>,
        L: Into<String>,
    {
        for (extension, language_id) in mappings {
            let extension = normalize_extension(&extension.into());
            let language_id = language_id.into();
            if extension.is_empty() || language_id.is_empty() {
                continue;
            }
            self.document_language_ids.insert(extension, language_id);
        }
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

    /// Returns the LSP document language id for a file extension.
    pub fn document_language_id_for_extension(&self, extension: &str) -> &str {
        let extension = normalize_extension(extension);
        self.document_language_ids
            .get(&extension)
            .map(String::as_str)
            .unwrap_or(&self.language_id)
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

    /// Returns the workspace-root strategy for this server.
    pub const fn root_strategy(&self) -> LanguageServerRootStrategy {
        self.root_strategy
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
}

/// Prepared session plan for an LSP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageServerSession {
    server_id: String,
    language_id: String,
    document_language_ids: BTreeMap<String, String>,
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

    /// Returns the document language id that should be sent for a file path.
    pub fn document_language_id_for_path(&self, path: &Path) -> &str {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| {
                let extension = normalize_extension(extension);
                self.document_language_ids
                    .get(&extension)
                    .map(String::as_str)
                    .unwrap_or(&self.language_id)
            })
            .unwrap_or(&self.language_id)
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
            document_language_ids: spec.document_language_ids.clone(),
            launch: spec.launch_job(root.clone()),
            root,
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
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| LspError::UnknownExtension(path.display().to_string()))?;
        let extension = normalize_extension(extension);
        let servers = self.servers_for_extension(&extension);
        if servers.is_empty() {
            return Err(LspError::UnknownExtension(extension));
        }
        let mut sessions = Vec::with_capacity(servers.len());
        for server in servers {
            sessions.push(self.prepare_session_for_path(server.id(), path, workspace_root)?);
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

    use super::{
        Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerRootStrategy,
        LanguageServerSpec,
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
}
