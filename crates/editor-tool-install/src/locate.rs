use std::path::{Path, PathBuf};

use editor_jobs::resolve_command_path;

use crate::{
    ToolKind,
    paths::{bin_dir, effective_path_env, tool_root},
};

/// Where Volt would find a Spec program today.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramLocation {
    /// Found on user PATH (not under Volt lsp/dap).
    UserPath(PathBuf),
    /// Found under a Volt Language Server or Debug Adapter Install.
    VoltInstall(PathBuf),
    /// Not found.
    Missing,
}

/// True when spawn would find `program` using Volt's effective PATH.
pub fn program_is_available(program: &str) -> bool {
    !matches!(locate_program(program), ProgramLocation::Missing)
}

/// Resolves `program` with user PATH first, then Volt bin dirs.
pub fn locate_program(program: &str) -> ProgramLocation {
    let path_env = effective_path_env();
    if let Some(resolved) = resolve_command_path(program, std::slice::from_ref(&path_env), None) {
        let path = PathBuf::from(&resolved);
        if is_volt_install_path(&path) {
            return ProgramLocation::VoltInstall(path);
        }
        return ProgramLocation::UserPath(path);
    }
    ProgramLocation::Missing
}

fn is_volt_install_path(path: &Path) -> bool {
    for kind in [ToolKind::LanguageServer, ToolKind::DebugAdapter] {
        if path.starts_with(tool_root(kind)) || path.starts_with(bin_dir(kind)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{ProgramLocation, locate_program};

    #[test]
    fn missing_program_is_missing() {
        assert_eq!(
            locate_program("volt-definitely-not-a-real-program-xyz"),
            ProgramLocation::Missing
        );
    }
}
