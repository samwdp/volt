use crate::install::InstallRecipe;
use crate::path::normalize_extension;

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
