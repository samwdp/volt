#![allow(unused_imports)]
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fmt, fs,
    mem::ManuallyDrop,
    ops::ControlFlow,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use editor_buffer::{SyntaxText, TextBuffer, TextByteChunks, TextEdit, TextPoint};
use editor_path::PathMatcher;
use tree_sitter::Language;
use tree_sitter::{
    InputEdit, Node, Parser, Point, Query, QueryCursor, QueryCursorOptions, QueryPredicateArg,
    QueryProperty, Range, StreamingIterator, TextProvider, Tree,
};
use tree_sitter_language::LanguageFn;

#[allow(unused_imports)]
use crate::highlight::*;
#[allow(unused_imports)]
use crate::language::*;
#[allow(unused_imports)]
use crate::query::*;
#[allow(unused_imports)]
use crate::registry::*;

pub(crate) fn configure_background_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

#[cfg(windows)]
pub(crate) fn windows_msvc_target_triple() -> &'static str {
    match env::consts::ARCH {
        "aarch64" => "aarch64-pc-windows-msvc",
        "x86" => "i686-pc-windows-msvc",
        _ => "x86_64-pc-windows-msvc",
    }
}

pub(crate) fn remove_legacy_grammar_install_directory(
    grammar: &GrammarSource,
    install_root: &Path,
) -> Result<(), SyntaxError> {
    let legacy_dir = grammar.legacy_install_directory(install_root);
    if legacy_dir.exists() {
        fs::remove_dir_all(&legacy_dir).map_err(|error| {
            io_error(
                "remove legacy grammar install directory",
                &legacy_dir,
                error,
            )
        })?;
    }
    Ok(())
}

pub(crate) fn ensure_cloned_grammar_dir_exists(grammar_dir: &Path) -> Result<(), SyntaxError> {
    if grammar_dir.exists() {
        return Ok(());
    }
    Err(SyntaxError::Io {
        operation: "locate cloned grammar directory".to_owned(),
        path: grammar_dir.to_path_buf(),
        message: "configured grammar directory does not exist in the cloned repository".to_owned(),
    })
}

pub(crate) fn run_install_command(
    language_id: &str,
    command_spec: &InstallCommandSpec,
) -> Result<(), SyntaxError> {
    let mut command = Command::new(command_spec.program());
    configure_background_command(&mut command);
    command.envs(command_spec.env().iter().cloned());
    let output = command
        .args(command_spec.args())
        .current_dir(command_spec.cwd())
        .output()
        .map_err(|error| {
            io_error(
                &format!("run {}", command_spec.program()),
                command_spec.cwd(),
                error,
            )
        })?;
    if !output.status.success() {
        return Err(SyntaxError::InstallCommand {
            language_id: language_id.to_owned(),
            message: command_failure_message(command_spec.label(), &output),
        });
    }
    Ok(())
}

pub(crate) fn remove_compiler_sidecar_artifacts(library_path: &Path) -> Result<(), SyntaxError> {
    for extension in ["exp", "lib"] {
        let artifact_path = library_path.with_extension(extension);
        if artifact_path.exists() {
            fs::remove_file(&artifact_path).map_err(|error| {
                io_error("remove compiler sidecar artifact", &artifact_path, error)
            })?;
        }
    }
    Ok(())
}

pub(crate) fn io_error(operation: &str, path: &Path, error: std::io::Error) -> SyntaxError {
    SyntaxError::Io {
        operation: operation.to_owned(),
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

pub(crate) fn command_failure_message(command_name: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !stdout.is_empty() {
        return stdout;
    }

    format!("{command_name} exited with status {}", output.status)
}

pub(crate) fn normalize_unique_entries<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for value in values {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() || normalized.iter().any(|entry| entry == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_owned());
    }
    normalized
}

pub(crate) fn default_install_root() -> PathBuf {
    editor_path::grammar_install_root()
}

pub(crate) fn default_query_asset_root() -> Option<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(exe_path) = env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        roots.extend(
            exe_dir
                .ancestors()
                .take(DEFAULT_QUERY_ASSET_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }
    if let Ok(current_dir) = env::current_dir() {
        roots.extend(
            current_dir
                .ancestors()
                .take(DEFAULT_QUERY_ASSET_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }

    resolve_query_asset_root_from_roots(roots)
}

pub(crate) fn asset_path_from_parts(base: &Path, parts: &[&str]) -> PathBuf {
    parts
        .iter()
        .fold(base.to_path_buf(), |candidate, part| candidate.join(part))
}

pub(crate) fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

pub(crate) fn shared_library_file_name(install_dir_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("lib{install_dir_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{install_dir_name}.dylib")
    } else {
        format!("lib{install_dir_name}.so")
    }
}

pub(crate) fn temp_guid_like_directory_name() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let value = duration.as_nanos() ^ ((std::process::id() as u128) << 32);
    let part1 = ((value >> 96) & 0xffff_ffff) as u32;
    let part2 = ((value >> 80) & 0xffff) as u16;
    let part3 = ((value >> 64) & 0xffff) as u16;
    let part4 = ((value >> 48) & 0xffff) as u16;
    let part5 = (value & 0xffff_ffff_ffff) as u64;
    format!("{part1:08x}-{part2:04x}-{part3:04x}-{part4:04x}-{part5:012x}")
}
