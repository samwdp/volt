#![doc = r#"Debug adapter registry, session plans, and DAP client host."#]

mod breakpoints;
mod client;
mod config;

use std::{collections::BTreeMap, error::Error, fmt, path::PathBuf};

use editor_jobs::JobSpec;

pub use breakpoints::{
    BreakpointState, BreakpointStore, BreakpointToggle, StoredBreakpoint, debug_source_paths_eq,
};
pub use client::{
    DapClientError, DapClientManager, DapEvaluateContext, DapExecutionPosition, DapLocalVariable,
    DapLogDirection, DapLogEntry, DapLogSnapshot, DapSessionEvent, DapSessionInfo,
    DapStackFrameInfo, DapStoppedSnapshot, DapThreadInfo, DapTransportLog, DapVariableNode,
    DapVariablePath, DapVariableRow, DapWatchExpression,
};
pub use config::{
    DapConfigError, DebugConfigurationCandidate, DebugConfigurationSource, DebugInferContext,
    DebugStartHistory, DebugStartRecord, PROJECT_DEBUG_CONFIG_PATH,
    collect_configuration_candidates, configuration_holes, infer_compile_heuristic,
    infer_configurations, load_project_configurations,
};
pub use editor_tool_install::InstallRecipe;

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Debug adapter registry, session plans, and DAP client host.";

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

/// How Volt talks to a Debug Adapter process.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DebugAdapterTransport {
    /// JSON-RPC frames over the adapter's stdin/stdout.
    #[default]
    Stdio,
    /// JSON-RPC frames over a TCP socket the adapter listens on.
    Tcp {
        /// Host to connect to after the adapter is ready.
        host: String,
        /// Port the adapter listens on.
        port: u16,
    },
}

/// Strategy used to choose a debug project root for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugAdapterRootStrategy {
    /// Reuse the editor workspace root as-is.
    #[default]
    Workspace,
    /// Prefer the nearest configured root marker for the current file and fall back to the editor
    /// workspace root when no marker matches.
    MarkersOrWorkspace,
}

/// Adapter specification compiled into the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugAdapterSpec {
    id: String,
    language_id: String,
    file_extensions: Vec<String>,
    program: String,
    args: Vec<String>,
    transport: DebugAdapterTransport,
    preference: i32,
    root_markers: Vec<String>,
    root_strategy: DebugAdapterRootStrategy,
    enabled_by_default: bool,
    install_recipe: Option<InstallRecipe>,
}

impl DebugAdapterSpec {
    /// Creates a new debug-adapter specification with stdio transport defaults.
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
            transport: DebugAdapterTransport::Stdio,
            preference: 0,
            root_markers: Vec::new(),
            root_strategy: DebugAdapterRootStrategy::Workspace,
            enabled_by_default: true,
            install_recipe: None,
        }
    }

    /// Sets the transport used to speak DAP with this adapter.
    pub fn with_transport(mut self, transport: DebugAdapterTransport) -> Self {
        self.transport = transport;
        self
    }

    /// Sets preference for multi-adapter resolution. Higher values win.
    pub fn with_preference(mut self, preference: i32) -> Self {
        self.preference = preference;
        self
    }

    /// Adds root markers used for project discovery.
    pub fn with_root_markers(
        mut self,
        markers: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.root_markers = markers.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the workspace-root strategy for this adapter.
    pub fn with_root_strategy(mut self, strategy: DebugAdapterRootStrategy) -> Self {
        self.root_strategy = strategy;
        self
    }

    /// Controls whether generic DAP start should include this adapter by default.
    pub fn with_enabled_by_default(mut self, enabled_by_default: bool) -> Self {
        self.enabled_by_default = enabled_by_default;
        self
    }

    /// Sets the optional Install Recipe used when the program is not on PATH.
    pub fn with_install_recipe(mut self, recipe: InstallRecipe) -> Self {
        self.install_recipe = Some(recipe);
        self
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

    /// Returns the DAP transport.
    pub fn transport(&self) -> &DebugAdapterTransport {
        &self.transport
    }

    /// Returns preference used when multiple adapters match.
    pub const fn preference(&self) -> i32 {
        self.preference
    }

    /// Returns root markers used for project discovery.
    pub fn root_markers(&self) -> &[String] {
        &self.root_markers
    }

    /// Returns the workspace-root strategy.
    pub const fn root_strategy(&self) -> DebugAdapterRootStrategy {
        self.root_strategy
    }

    /// Returns whether generic DAP start should include this adapter by default.
    pub const fn enabled_by_default(&self) -> bool {
        self.enabled_by_default
    }

    /// Returns the Install Recipe, if this adapter is Volt-installable.
    pub fn install_recipe(&self) -> Option<&InstallRecipe> {
        self.install_recipe.as_ref()
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
    adapter_id: Option<String>,
    compile_command: Option<String>,
    process_id: Option<u32>,
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
            adapter_id: None,
            compile_command: None,
            process_id: None,
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

    /// Pins this configuration to a Debug Adapter id.
    pub fn with_adapter_id(mut self, adapter_id: impl Into<String>) -> Self {
        self.adapter_id = Some(adapter_id.into());
        self
    }

    /// Sets an explicit compile-before-debug shell command.
    pub fn with_compile_command(mut self, compile_command: impl Into<String>) -> Self {
        self.compile_command = Some(compile_command.into());
        self
    }

    /// Sets a process id for attach configurations.
    pub fn with_process_id(mut self, process_id: u32) -> Self {
        self.process_id = Some(process_id);
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

    /// Returns the pinned adapter id, if any.
    pub fn adapter_id(&self) -> Option<&str> {
        self.adapter_id.as_deref()
    }

    /// Returns the explicit compile-before-debug command, if any.
    pub fn compile_command(&self) -> Option<&str> {
        self.compile_command.as_deref()
    }

    /// Returns the attach process id, if any.
    pub const fn process_id(&self) -> Option<u32> {
        self.process_id
    }
}

/// Prepared debug session plan for an adapter and configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSessionPlan {
    adapter_id: String,
    language_id: String,
    adapter_launch: JobSpec,
    configuration: DebugConfiguration,
    transport: DebugAdapterTransport,
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

    /// Returns the transport for this plan.
    pub fn transport(&self) -> &DebugAdapterTransport {
        &self.transport
    }
}

/// Errors produced by DAP registry operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DapError {
    /// Duplicate adapter id registration.
    DuplicateAdapterId(String),
    /// Unknown adapter id.
    UnknownAdapter(String),
    /// Unknown extension.
    UnknownExtension(String),
    /// No enabled adapter matched the extension.
    NoEnabledAdapter(String),
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
            Self::UnknownAdapter(adapter_id) => {
                write!(formatter, "debug adapter `{adapter_id}` is not registered")
            }
            Self::UnknownExtension(extension) => {
                write!(formatter, "no debug adapter registered for `{extension}`")
            }
            Self::NoEnabledAdapter(extension) => {
                write!(
                    formatter,
                    "no enabled debug adapter registered for `{extension}`"
                )
            }
        }
    }
}

impl Error for DapError {}

/// Registry of known debug adapters.
#[derive(Debug, Default, Clone)]
pub struct DebugAdapterRegistry {
    adapters: BTreeMap<String, DebugAdapterSpec>,
    extensions: BTreeMap<String, Vec<String>>,
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
            self.extensions
                .entry(extension.clone())
                .or_default()
                .push(adapter_id.clone());
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

    /// Returns registered adapters sorted by id.
    pub fn adapters(&self) -> impl Iterator<Item = &DebugAdapterSpec> {
        self.adapters.values()
    }

    /// Returns adapters for a file extension, highest preference first.
    pub fn adapters_for_extension(&self, extension: &str) -> Vec<&DebugAdapterSpec> {
        let extension = normalize_extension(extension);
        let Some(adapter_ids) = self.extensions.get(&extension) else {
            return Vec::new();
        };
        let mut adapters = adapter_ids
            .iter()
            .filter_map(|adapter_id| self.adapters.get(adapter_id))
            .collect::<Vec<_>>();
        adapters.sort_by(|left, right| {
            right
                .preference()
                .cmp(&left.preference())
                .then_with(|| left.id().cmp(right.id()))
        });
        adapters
    }

    /// Returns the preferred enabled adapter for a file extension, if one exists.
    pub fn adapter_for_extension(&self, extension: &str) -> Option<&DebugAdapterSpec> {
        self.adapters_for_extension(extension)
            .into_iter()
            .find(|adapter| adapter.enabled_by_default())
    }

    /// Returns enabled adapters for a file extension, highest preference first.
    pub fn enabled_adapters_for_extension(&self, extension: &str) -> Vec<&DebugAdapterSpec> {
        self.adapters_for_extension(extension)
            .into_iter()
            .filter(|adapter| adapter.enabled_by_default())
            .collect()
    }

    /// Resolves a preferred adapter for an extension, failing when none are enabled.
    pub fn resolve_adapter_for_extension(
        &self,
        extension: &str,
    ) -> Result<&DebugAdapterSpec, DapError> {
        let extension = normalize_extension(extension);
        let adapters = self.enabled_adapters_for_extension(&extension);
        adapters.into_iter().next().ok_or_else(|| {
            if self.extensions.contains_key(&extension) {
                DapError::NoEnabledAdapter(extension)
            } else {
                DapError::UnknownExtension(extension)
            }
        })
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
            transport: adapter.transport.clone(),
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
mod tests;
