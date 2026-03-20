#![doc = r#"Debug adapter registry, session plans, and debugger-facing launch metadata."#]

use std::{collections::BTreeMap, error::Error, fmt, path::PathBuf};

use editor_jobs::JobSpec;

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Debug adapter registry, session plans, and debugger-facing launch metadata.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Supported DAP request kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugRequestKind {
    /// Launch a new debugee process.
    Launch,
    /// Attach to an existing process.
    Attach,
}

/// Adapter specification compiled into the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugAdapterSpec {
    id: String,
    language_id: String,
    file_extensions: Vec<String>,
    program: String,
    args: Vec<String>,
}

impl DebugAdapterSpec {
    /// Creates a new debug-adapter specification.
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
        }
    }

    /// Returns the adapter identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the associated language identifier.
    pub fn language_id(&self) -> &str {
        &self.language_id
    }

    /// Returns the handled file extensions.
    pub fn file_extensions(&self) -> &[String] {
        &self.file_extensions
    }

    /// Returns the adapter executable.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Returns the adapter arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }
}

/// Launch or attach configuration chosen by the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugConfiguration {
    name: String,
    request: DebugRequestKind,
    target_program: Option<PathBuf>,
    cwd: Option<PathBuf>,
    args: Vec<String>,
}

impl DebugConfiguration {
    /// Creates a new debug configuration.
    pub fn new(name: impl Into<String>, request: DebugRequestKind) -> Self {
        Self {
            name: name.into(),
            request,
            target_program: None,
            cwd: None,
            args: Vec::new(),
        }
    }

    /// Sets the target program path.
    pub fn with_target_program(mut self, target_program: impl Into<PathBuf>) -> Self {
        self.target_program = Some(target_program.into());
        self
    }

    /// Sets the working directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Sets command-line arguments for the debugee.
    pub fn with_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configuration name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the request kind.
    pub const fn request(&self) -> DebugRequestKind {
        self.request
    }

    /// Returns the target program path, if any.
    pub fn target_program(&self) -> Option<&PathBuf> {
        self.target_program.as_ref()
    }

    /// Returns the working directory, if any.
    pub fn cwd(&self) -> Option<&PathBuf> {
        self.cwd.as_ref()
    }

    /// Returns the debugee argument list.
    pub fn args(&self) -> &[String] {
        &self.args
    }
}

/// Prepared debug session plan for an adapter and configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSessionPlan {
    adapter_id: String,
    language_id: String,
    adapter_launch: JobSpec,
    configuration: DebugConfiguration,
}

impl DebugSessionPlan {
    /// Returns the adapter identifier.
    pub fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    /// Returns the language identifier.
    pub fn language_id(&self) -> &str {
        &self.language_id
    }

    /// Returns the adapter launch job.
    pub fn adapter_launch(&self) -> &JobSpec {
        &self.adapter_launch
    }

    /// Returns the user-facing debug configuration.
    pub fn configuration(&self) -> &DebugConfiguration {
        &self.configuration
    }
}

/// Errors produced by DAP registry operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DapError {
    /// Duplicate adapter id registration.
    DuplicateAdapterId(String),
    /// Duplicate extension registration.
    DuplicateExtension(String),
    /// Unknown adapter id.
    UnknownAdapter(String),
    /// Unknown extension.
    UnknownExtension(String),
}

impl fmt::Display for DapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateAdapterId(adapter_id) => {
                write!(
                    formatter,
                    "debug adapter `{adapter_id}` is already registered"
                )
            }
            Self::DuplicateExtension(extension) => {
                write!(
                    formatter,
                    "extension `{extension}` is already mapped to a debug adapter"
                )
            }
            Self::UnknownAdapter(adapter_id) => {
                write!(formatter, "debug adapter `{adapter_id}` is not registered")
            }
            Self::UnknownExtension(extension) => {
                write!(formatter, "no debug adapter registered for `{extension}`")
            }
        }
    }
}

impl Error for DapError {}

/// Registry of known debug adapters.
#[derive(Debug, Default, Clone)]
pub struct DebugAdapterRegistry {
    adapters: BTreeMap<String, DebugAdapterSpec>,
    extensions: BTreeMap<String, String>,
}

impl DebugAdapterRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of registered adapters.
    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    /// Returns whether no adapters are registered.
    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }

    /// Registers a new debug adapter specification.
    pub fn register(&mut self, spec: DebugAdapterSpec) -> Result<(), DapError> {
        let adapter_id = spec.id().to_owned();
        if self.adapters.contains_key(&adapter_id) {
            return Err(DapError::DuplicateAdapterId(adapter_id));
        }
        for extension in spec.file_extensions() {
            if self.extensions.contains_key(extension) {
                return Err(DapError::DuplicateExtension(extension.clone()));
            }
        }
        for extension in spec.file_extensions() {
            self.extensions
                .insert(extension.clone(), adapter_id.clone());
        }
        self.adapters.insert(adapter_id, spec);
        Ok(())
    }

    /// Registers multiple debug adapters.
    pub fn register_all<I>(&mut self, specs: I) -> Result<(), DapError>
    where
        I: IntoIterator<Item = DebugAdapterSpec>,
    {
        for spec in specs {
            self.register(spec)?;
        }
        Ok(())
    }

    /// Returns an adapter by identifier.
    pub fn adapter(&self, adapter_id: &str) -> Option<&DebugAdapterSpec> {
        self.adapters.get(adapter_id)
    }

    /// Returns an adapter for a file extension, if one exists.
    pub fn adapter_for_extension(&self, extension: &str) -> Option<&DebugAdapterSpec> {
        let extension = normalize_extension(extension);
        let adapter_id = self.extensions.get(&extension)?;
        self.adapters.get(adapter_id)
    }

    /// Prepares a debug session plan using the named adapter.
    pub fn prepare_session(
        &self,
        adapter_id: &str,
        configuration: DebugConfiguration,
    ) -> Result<DebugSessionPlan, DapError> {
        let adapter = self
            .adapters
            .get(adapter_id)
            .ok_or_else(|| DapError::UnknownAdapter(adapter_id.to_owned()))?;
        let launch = JobSpec::command(
            format!("dap:{}", adapter.id()),
            adapter.program.clone(),
            adapter.args.clone(),
        );
        Ok(DebugSessionPlan {
            adapter_id: adapter.id().to_owned(),
            language_id: adapter.language_id().to_owned(),
            adapter_launch: launch,
            configuration,
        })
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

    use super::{DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind};

    fn codelldb() -> DebugAdapterSpec {
        DebugAdapterSpec::new("codelldb", "rust", ["rs"], "codelldb", ["--port", "13000"])
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    #[test]
    fn registry_resolves_adapter_by_extension() {
        let mut registry = DebugAdapterRegistry::new();
        must(registry.register(codelldb()));

        let adapter = registry.adapter_for_extension("rs").expect("adapter");
        assert_eq!(adapter.id(), "codelldb");
        assert_eq!(adapter.program(), "codelldb");
    }

    #[test]
    fn prepared_session_includes_configuration_and_launch_spec() {
        let mut registry = DebugAdapterRegistry::new();
        must(registry.register(codelldb()));

        let plan = must(
            registry.prepare_session(
                "codelldb",
                DebugConfiguration::new("Debug volt", DebugRequestKind::Launch)
                    .with_target_program(PathBuf::from("target\\debug\\volt.exe"))
                    .with_cwd(PathBuf::from("P:\\volt"))
                    .with_args(["--shell-hidden"]),
            ),
        );

        assert_eq!(plan.adapter_id(), "codelldb");
        assert_eq!(plan.language_id(), "rust");
        assert_eq!(plan.adapter_launch().program(), "codelldb");
        assert_eq!(plan.configuration().name(), "Debug volt");
        assert_eq!(plan.configuration().args(), ["--shell-hidden"]);
    }
}
