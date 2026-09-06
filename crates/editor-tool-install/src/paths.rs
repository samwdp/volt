use std::{env, fs, io, path::PathBuf};

use editor_path::volt_data_dir;

use crate::ToolKind;

/// Root directory for Language Server or Debug Adapter Installs.
pub fn tool_root(kind: ToolKind) -> PathBuf {
    volt_data_dir().join(kind.dir_name())
}

/// Shim directory appended to process PATH.
pub fn bin_dir(kind: ToolKind) -> PathBuf {
    tool_root(kind).join("bin")
}

/// Per-spec package directory.
pub fn package_dir(kind: ToolKind, spec_id: &str) -> PathBuf {
    tool_root(kind).join("packages").join(spec_id)
}

/// Creates `packages/` and `bin/` for both lsp and dap.
pub fn ensure_install_layout() -> io::Result<()> {
    for kind in [ToolKind::LanguageServer, ToolKind::DebugAdapter] {
        fs::create_dir_all(tool_root(kind).join("packages"))?;
        fs::create_dir_all(bin_dir(kind))?;
    }
    Ok(())
}

/// PATH used for locate + child spawns: user PATH first, then lsp/bin and dap/bin.
pub fn effective_path() -> String {
    path_with_install_bins(&env::var("PATH").unwrap_or_default())
}

/// PATH key/value pair for Command env lists.
pub fn effective_path_env() -> (String, String) {
    ("PATH".to_owned(), effective_path())
}

/// Merges managed bin dirs into an existing env list (replaces PATH if present).
pub fn merge_effective_path(env_pairs: &mut Vec<(String, String)>) {
    let path = effective_path();
    if let Some((_, value)) = env_pairs
        .iter_mut()
        .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))
    {
        *value = path_with_install_bins(value);
        return;
    }
    env_pairs.push(("PATH".to_owned(), path));
}

/// Appends lsp/bin and dap/bin to a PATH value if missing (user PATH stays first).
pub fn path_with_install_bins(current: &str) -> String {
    let mut entries = split_path(current);
    for kind in [ToolKind::LanguageServer, ToolKind::DebugAdapter] {
        let dir = bin_dir(kind);
        let rendered = dir.to_string_lossy().into_owned();
        if !entries.iter().any(|entry| path_entry_eq(entry, &dir)) {
            entries.push(rendered);
        }
    }
    entries.join(path_separator())
}

/// Kept for boot: ensure dirs exist so shims can be written later.
pub fn apply_install_bins_to_process_path() -> io::Result<()> {
    ensure_install_layout()
}

fn split_path(current: &str) -> Vec<String> {
    current
        .split(path_separator())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect()
}

fn path_separator() -> &'static str {
    if cfg!(windows) { ";" } else { ":" }
}

fn path_entry_eq(entry: &str, dir: &std::path::Path) -> bool {
    let left = std::path::Path::new(entry);
    if cfg!(windows) {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&dir.to_string_lossy())
    } else {
        left == dir
    }
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
