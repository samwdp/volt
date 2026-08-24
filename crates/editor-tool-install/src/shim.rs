use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    InstallPlan, InstallRecipe, ToolInstallError,
    paths::{bin_dir, package_dir},
};

/// After Command Stream commands succeed: find the binary and write a PATH shim.
pub fn finalize_install(plan: &InstallPlan) -> Result<PathBuf, ToolInstallError> {
    let package = package_dir(plan.kind(), plan.spec_id());
    let target = resolve_binary(plan, &package)?;
    ensure_unix_executable(&target)?;
    let shim = write_shim(plan.kind(), plan.program(), &target)?;
    Ok(shim)
}

/// Writes a shim named `program` in the kind's bin dir pointing at `target`.
pub fn write_shim(
    kind: crate::ToolKind,
    program: &str,
    target: &Path,
) -> Result<PathBuf, ToolInstallError> {
    let bin = bin_dir(kind);
    fs::create_dir_all(&bin)?;
    let shim_path = shim_path(&bin, program);
    let contents = shim_contents(target);
    let mut file = File::create(&shim_path)?;
    file.write_all(contents.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&shim_path, permissions)?;
    }
    Ok(shim_path)
}

fn ensure_unix_executable(path: &Path) -> Result<(), ToolInstallError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o755);
        fs::set_permissions(path, permissions)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn shim_path(bin: &Path, program: &str) -> PathBuf {
    if cfg!(windows) {
        let already_has_ext = Path::new(program).extension().is_some();
        if already_has_ext {
            bin.join(program)
        } else {
            bin.join(format!("{program}.cmd"))
        }
    } else {
        bin.join(program)
    }
}

fn shim_contents(target: &Path) -> String {
    if cfg!(windows) {
        format!("@echo off\r\n\"{}\" %*\r\n", target.display())
    } else {
        format!("#!/bin/sh\nexec \"{}\" \"$@\"\n", target.display())
    }
}

fn resolve_binary(plan: &InstallPlan, package: &Path) -> Result<PathBuf, ToolInstallError> {
    if let InstallRecipe::Archive {
        binary: Some(relative),
        ..
    } = plan.recipe()
    {
        let candidate = package.join(relative);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    find_named_file(package, plan.program(), 6).ok_or_else(|| ToolInstallError::MissingBinary {
        program: plan.program().to_owned(),
        package_dir: package.display().to_string(),
    })
}

fn find_named_file(root: &Path, program: &str, max_depth: usize) -> Option<PathBuf> {
    let names = candidate_names(program);
    find_named_file_inner(root, &names, max_depth, 0)
}

fn candidate_names(program: &str) -> Vec<String> {
    let mut names = vec![program.to_owned()];
    if cfg!(windows) && Path::new(program).extension().is_none() {
        for extension in [".exe", ".cmd", ".bat"] {
            names.push(format!("{program}{extension}"));
        }
    }
    names
}

fn find_named_file_inner(
    dir: &Path,
    names: &[String],
    max_depth: usize,
    depth: usize,
) -> Option<PathBuf> {
    if depth > max_depth {
        return None;
    }
    let entries = fs::read_dir(dir).ok()?;
    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str())
                && names
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(file_name))
            {
                return Some(path);
            }
        } else if path.is_dir() {
            dirs.push(path);
        }
    }
    for path in dirs {
        if let Some(found) = find_named_file_inner(&path, names, max_depth, depth + 1) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{shim_contents, shim_path};
    use std::path::Path;

    #[test]
    fn windows_shim_invokes_quoted_target() {
        let contents = shim_contents(Path::new(r"C:\volt\lsp\packages\x\tool.exe"));
        if cfg!(windows) {
            assert!(contents.contains("@echo off"));
            assert!(contents.contains(r"C:\volt\lsp\packages\x\tool.exe"));
        } else {
            assert!(contents.starts_with("#!/bin/sh"));
        }
    }

    #[test]
    fn shim_path_adds_cmd_on_windows() {
        let path = shim_path(Path::new("/bin"), "typescript-language-server");
        if cfg!(windows) {
            assert!(path.ends_with("typescript-language-server.cmd"));
        } else {
            assert!(path.ends_with("typescript-language-server"));
        }
    }
}
