#![doc = r#"Typed Install Recipes and Volt-managed Language Server / Debug Adapter layout."#]

mod locate;
mod paths;
mod plan;
mod recipe;
mod shim;

use std::{error::Error, fmt, io};

pub use locate::{ProgramLocation, locate_program, program_is_available};
pub use paths::{
    apply_install_bins_to_process_path, bin_dir, effective_path, effective_path_env,
    ensure_install_layout, merge_effective_path, package_dir, path_with_install_bins, tool_root,
};
pub use plan::{InstallCommand, InstallPlan, prepare_install};
pub use recipe::InstallRecipe;
pub use shim::{finalize_install, write_shim};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Typed install recipes, Volt lsp/dap layout, process PATH append, and shims.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Whether the recipe installs a Language Server or a Debug Adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolKind {
    LanguageServer,
    DebugAdapter,
}

impl ToolKind {
    /// Directory name under the Volt data directory (`lsp` or `dap`).
    pub const fn dir_name(self) -> &'static str {
        match self {
            Self::LanguageServer => "lsp",
            Self::DebugAdapter => "dap",
        }
    }

    /// Failed-install key prefix.
    pub const fn key_prefix(self) -> &'static str {
        match self {
            Self::LanguageServer => "lsp",
            Self::DebugAdapter => "dap",
        }
    }
}

/// Failure while preparing or finalizing an Install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolInstallError {
    Io(String),
    MissingToolchain {
        program: String,
        recipe: String,
    },
    MissingBinary {
        program: String,
        package_dir: String,
    },
}

impl fmt::Display for ToolInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "{message}"),
            Self::MissingToolchain { program, recipe } => write!(
                formatter,
                "cannot run Install Recipe ({recipe}): `{program}` is not on PATH"
            ),
            Self::MissingBinary {
                program,
                package_dir,
            } => write!(
                formatter,
                "Install finished but `{program}` was not found under `{package_dir}`"
            ),
        }
    }
}

impl Error for ToolInstallError {}

impl From<io::Error> for ToolInstallError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// Key used to remember a Failed Install for this process.
pub fn failed_install_key(kind: ToolKind, spec_id: &str) -> String {
    format!("{}:{spec_id}", kind.key_prefix())
}
