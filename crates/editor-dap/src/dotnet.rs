//! .NET build-target and runnable-assembly discovery for Debug start.
//!
//! `dotnet build` at the workspace root misses a solution or project that lives
//! in a subdirectory, and a `bin` folder contains dependency DLLs as well as the
//! program. Candidates are the assembly named by `AssemblyName`, the project
//! directory, or the project file stem.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const WALK_MAX_DEPTH: usize = 12;
const BIN_WALK_MAX_DEPTH: usize = 5;

/// A `.sln`, `.slnx`, or `.csproj` that `dotnet build` can target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotnetBuildTarget {
    path: PathBuf,
    kind: DotnetBuildTargetKind,
}

impl DotnetBuildTarget {
    /// Absolute path of the solution or project file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether this target is a solution or a project.
    pub const fn kind(&self) -> DotnetBuildTargetKind {
        self.kind
    }
}

/// Solution file versus project file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotnetBuildTargetKind {
    /// `.sln` or `.slnx`.
    Solution,
    /// `.csproj`.
    Project,
}

/// One runnable assembly for a selected .NET build target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotnetProgramCandidate {
    path: PathBuf,
    exists: bool,
    executable: bool,
}

impl DotnetProgramCandidate {
    /// Absolute path of the assembly.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether the assembly is already on disk.
    pub const fn exists(&self) -> bool {
        self.exists
    }

    /// Path relative to the workspace when it sits under `root`.
    pub fn display_path(&self, root: &Path) -> String {
        display_under_root(root, &self.path)
    }

    /// Picker detail: built or expected, and executable or library.
    pub fn picker_detail(&self) -> String {
        let presence = if self.exists { "built" } else { "expected" };
        let kind = if self.executable {
            "executable"
        } else {
            "library"
        };
        format!("{presence} · {kind}")
    }
}

/// Lists solution and project files under `root`, nearest project to `active_file` first.
pub fn discover_dotnet_build_targets(
    root: &Path,
    active_file: Option<&Path>,
) -> Vec<DotnetBuildTarget> {
    let mut files = Vec::new();
    walk_build_files(root, 0, &mut files);
    let nearest = active_file.and_then(|active| nearest_project(&files, active).cloned());
    files.sort_by(|left, right| {
        target_rank(left, nearest.as_deref())
            .cmp(&target_rank(right, nearest.as_deref()))
            .then_with(|| {
                left.to_string_lossy()
                    .to_lowercase()
                    .cmp(&right.to_string_lossy().to_lowercase())
            })
    });
    files
        .into_iter()
        .filter_map(|path| {
            let kind = build_target_kind(&path)?;
            Some(DotnetBuildTarget { path, kind })
        })
        .collect()
}

/// Shell command that builds one solution or project.
pub fn dotnet_compile_command(root: &Path, target: &Path) -> String {
    let rendered = display_under_root(root, target);
    if rendered.contains(' ') || rendered.contains('"') {
        format!("dotnet build \"{}\"", rendered.replace('"', "\\\""))
    } else {
        format!("dotnet build {rendered}")
    }
}

/// Resolves the project or solution argument of a `dotnet build` command.
pub fn dotnet_build_target_from_command(root: &Path, command: &str) -> Option<PathBuf> {
    let arg = dotnet_build_argument(command)?;
    let path = PathBuf::from(arg);
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    path.is_file().then_some(path)
}

/// Runnable assemblies for one solution or project.
///
/// When any matching DLL already exists, only those files are returned.
/// Otherwise each project contributes its expected Debug output path.
pub fn dotnet_program_candidates(root: &Path, build_target: &Path) -> Vec<DotnetProgramCandidate> {
    let _ = root;
    candidates_for_projects(&projects_for_target(build_target))
}

/// Runnable assemblies for every project under the workspace.
pub fn dotnet_workspace_program_candidates(root: &Path) -> Vec<DotnetProgramCandidate> {
    let projects = discover_dotnet_build_targets(root, None)
        .into_iter()
        .filter(|target| target.kind == DotnetBuildTargetKind::Project)
        .map(|target| target.path)
        .collect::<Vec<_>>();
    candidates_for_projects(&projects)
}

/// Assemblies produced by a `dotnet build` command.
///
/// Returns `None` when `command` is not `dotnet build`.
pub fn dotnet_program_candidates_for_build_command(
    root: &Path,
    command: &str,
    active_file: Option<&Path>,
) -> Option<Vec<DotnetProgramCandidate>> {
    if !is_dotnet_build_command(command) {
        return None;
    }
    if let Some(target) = dotnet_build_target_from_command(root, command) {
        return Some(dotnet_program_candidates(root, &target));
    }
    let targets = discover_dotnet_build_targets(root, active_file);
    let projects = targets
        .iter()
        .filter(|target| target.kind == DotnetBuildTargetKind::Project)
        .map(|target| target.path.clone())
        .collect::<Vec<_>>();
    if !projects.is_empty() {
        return Some(candidates_for_projects(&projects));
    }
    let solution = targets
        .iter()
        .find(|target| target.kind == DotnetBuildTargetKind::Solution)?;
    Some(dotnet_program_candidates(root, solution.path()))
}

/// Existing runnable DLL for `csproj`, preferring the project directory name.
pub fn preferred_existing_dotnet_dll(csproj: &Path) -> Option<PathBuf> {
    let project = csproj.to_path_buf();
    let existing = candidates_for_projects(std::slice::from_ref(&project))
        .into_iter()
        .filter(|candidate| candidate.exists)
        .collect::<Vec<_>>();
    let directory_name = csproj.parent().and_then(|dir| dir.file_name());
    if let Some(directory_name) = directory_name
        && let Some(matched) = existing.iter().find(|candidate| {
            candidate
                .path
                .file_stem()
                .is_some_and(|stem| stem == directory_name)
        })
    {
        return Some(matched.path.clone());
    }
    existing.into_iter().next().map(|candidate| candidate.path)
}

fn candidates_for_projects(projects: &[PathBuf]) -> Vec<DotnetProgramCandidate> {
    let mut existing = Vec::new();
    let mut expected = Vec::new();
    let mut seen = BTreeSet::new();
    for project in projects {
        let contents = fs::read_to_string(project).unwrap_or_default();
        let names = assembly_names(project, &contents);
        let executable = is_executable_project(&contents);
        let project_dir = project.parent().unwrap_or(project.as_path());
        for path in existing_assembly_dlls(project_dir, &contents, &names) {
            push_candidate(
                &mut existing,
                &mut seen,
                DotnetProgramCandidate {
                    path,
                    exists: true,
                    executable,
                },
            );
        }
        for path in expected_assembly_dlls(project_dir, &contents, &names) {
            push_candidate(
                &mut expected,
                &mut seen,
                DotnetProgramCandidate {
                    path,
                    exists: false,
                    executable,
                },
            );
        }
    }
    let mut chosen = if existing.is_empty() {
        expected
    } else {
        existing
    };
    chosen.sort_by(|left, right| {
        left.executable
            .cmp(&right.executable)
            .reverse()
            .then_with(|| {
                left.path
                    .to_string_lossy()
                    .to_lowercase()
                    .cmp(&right.path.to_string_lossy().to_lowercase())
            })
    });
    chosen
}

fn push_candidate(
    out: &mut Vec<DotnetProgramCandidate>,
    seen: &mut BTreeSet<String>,
    candidate: DotnetProgramCandidate,
) {
    let key = candidate.path.to_string_lossy().to_lowercase();
    if seen.insert(key) {
        out.push(candidate);
    }
}

fn projects_for_target(target: &Path) -> Vec<PathBuf> {
    let Some(extension) = target.extension().and_then(|ext| ext.to_str()) else {
        return Vec::new();
    };
    if extension.eq_ignore_ascii_case("csproj") {
        return vec![target.to_path_buf()];
    }
    if extension.eq_ignore_ascii_case("sln") || extension.eq_ignore_ascii_case("slnx") {
        let listed = if extension.eq_ignore_ascii_case("slnx") {
            slnx_projects(target)
        } else {
            sln_projects(target)
        };
        if !listed.is_empty() {
            return listed;
        }
        if let Some(dir) = target.parent() {
            return csproj_files_under(dir);
        }
    }
    Vec::new()
}

fn sln_projects(sln: &Path) -> Vec<PathBuf> {
    let Ok(text) = fs::read_to_string(sln) else {
        return Vec::new();
    };
    let Some(parent) = sln.parent() else {
        return Vec::new();
    };
    let mut projects = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("Project(") {
            continue;
        }
        let Some((_, rest)) = line.split_once('=') else {
            continue;
        };
        let mut parts = rest.split(',').map(str::trim);
        let _name = parts.next();
        let Some(raw_path) = parts.next() else {
            continue;
        };
        let rel = raw_path.trim().trim_matches('"').replace('\\', "/");
        if !rel.to_ascii_lowercase().ends_with(".csproj") {
            continue;
        }
        let path = parent.join(&rel);
        if path.is_file() {
            projects.push(path);
        }
    }
    projects
}

fn slnx_projects(slnx: &Path) -> Vec<PathBuf> {
    let Ok(text) = fs::read_to_string(slnx) else {
        return Vec::new();
    };
    let Some(parent) = slnx.parent() else {
        return Vec::new();
    };
    let mut projects = Vec::new();
    let mut rest = text.as_str();
    while let Some(index) = rest.find("Path=\"") {
        rest = &rest[index + 6..];
        let Some(end) = rest.find('"') else {
            break;
        };
        let rel = rest[..end].replace('\\', "/");
        rest = &rest[end..];
        if !rel.to_ascii_lowercase().ends_with(".csproj") {
            continue;
        }
        let path = parent.join(&rel);
        if path.is_file() {
            projects.push(path);
        }
    }
    projects
}

fn csproj_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_build_files(dir, 0, &mut files);
    files.retain(|path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("csproj"))
    });
    files
}

fn assembly_names(csproj: &Path, contents: &str) -> Vec<String> {
    if let Some(name) = parse_property(contents, "AssemblyName") {
        return vec![name];
    }
    let mut names = Vec::new();
    if let Some(dir_name) = csproj
        .parent()
        .and_then(|dir| dir.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        && !dir_name.is_empty()
    {
        names.push(dir_name);
    }
    if let Some(stem) = csproj
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        && !names.iter().any(|name| name.eq_ignore_ascii_case(&stem))
    {
        names.push(stem);
    }
    if names.is_empty() {
        names.push("App".to_owned());
    }
    names
}

fn expected_assembly_dlls(project_dir: &Path, contents: &str, names: &[String]) -> Vec<PathBuf> {
    let tfm = target_framework(contents);
    let dir = expected_output_dir(project_dir, contents, &tfm);
    names
        .iter()
        .map(|name| dir.join(format!("{name}.dll")))
        .collect()
}

fn expected_output_dir(project_dir: &Path, contents: &str, tfm: &str) -> PathBuf {
    if let Some(output) = parse_property(contents, "OutputPath") {
        let output = output.replace('\\', "/");
        let path = PathBuf::from(&output);
        let base = if path.is_absolute() {
            path
        } else {
            project_dir.join(output)
        };
        let append = parse_property(contents, "AppendTargetFrameworkToOutputPath")
            .is_none_or(|value| !value.eq_ignore_ascii_case("false"));
        if append && base.file_name().and_then(|name| name.to_str()) != Some(tfm) {
            return base.join(tfm);
        }
        return base;
    }
    project_dir.join("bin").join("Debug").join(tfm)
}

fn target_framework(contents: &str) -> String {
    if let Some(tfm) = parse_property(contents, "TargetFramework") {
        return tfm;
    }
    if let Some(frameworks) = parse_property(contents, "TargetFrameworks")
        && let Some(first) = frameworks
            .split([';', ',', ' '])
            .map(str::trim)
            .find(|part| !part.is_empty())
    {
        return first.to_owned();
    }
    "net8.0".to_owned()
}

fn existing_assembly_dlls(project_dir: &Path, contents: &str, names: &[String]) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_named_dlls(&project_dir.join("bin"), names, 0, &mut found);
    if parse_property(contents, "OutputPath").is_some() {
        let dir = expected_output_dir(project_dir, contents, &target_framework(contents));
        collect_named_dlls(&dir, names, 0, &mut found);
        if let Some(parent) = dir.parent() {
            collect_named_dlls(parent, names, 0, &mut found);
        }
    }
    found.sort();
    found.dedup();
    found
}

fn collect_named_dlls(dir: &Path, names: &[String], depth: usize, out: &mut Vec<PathBuf>) {
    if depth > BIN_WALK_MAX_DEPTH || !dir.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if entry.file_name().to_str().is_some_and(|name| {
                name.eq_ignore_ascii_case("ref") || name.eq_ignore_ascii_case("refs")
            }) {
                continue;
            }
            collect_named_dlls(&path, names, depth + 1, out);
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let lower = file_name.to_ascii_lowercase();
        let Some(stem) = lower.strip_suffix(".dll") else {
            continue;
        };
        if names.iter().any(|name| name.eq_ignore_ascii_case(stem)) {
            out.push(path);
        }
    }
}

fn is_executable_project(contents: &str) -> bool {
    parse_property(contents, "OutputType").is_some_and(|value| {
        value.eq_ignore_ascii_case("Exe")
            || value.eq_ignore_ascii_case("WinExe")
            || value.eq_ignore_ascii_case("AppContainerExe")
    })
}

fn parse_property(contents: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let start = contents.find(&open)? + open.len();
    let rest = contents.get(start..)?;
    let end = rest.find(&close)?;
    let value = rest[..end].trim();
    (!value.is_empty()).then(|| value.to_owned())
}

pub fn is_dotnet_build_command(command: &str) -> bool {
    let command = command.trim();
    command == "dotnet build" || command.starts_with("dotnet build ")
}

fn dotnet_build_argument(command: &str) -> Option<&str> {
    let command = command.trim();
    let rest = command.strip_prefix("dotnet build")?.trim();
    if rest.is_empty() {
        return None;
    }
    if let Some(rest) = rest.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(&rest[..end]);
    }
    rest.split_whitespace().next()
}

fn walk_build_files(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > WALK_MAX_DEPTH {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if is_skipped_dir(&entry.file_name()) {
                continue;
            }
            walk_build_files(&path, depth + 1, out);
            continue;
        }
        if build_target_kind(&path).is_some() {
            out.push(path);
        }
    }
}

fn is_skipped_dir(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|name| {
        matches!(
            name.to_ascii_lowercase().as_str(),
            "bin" | "obj" | ".git" | ".vs" | "node_modules" | "target" | "testresults"
        )
    })
}

fn build_target_kind(path: &Path) -> Option<DotnetBuildTargetKind> {
    let extension = path.extension()?.to_str()?;
    if extension.eq_ignore_ascii_case("csproj") {
        Some(DotnetBuildTargetKind::Project)
    } else if extension.eq_ignore_ascii_case("sln") || extension.eq_ignore_ascii_case("slnx") {
        Some(DotnetBuildTargetKind::Solution)
    } else {
        None
    }
}

fn nearest_project<'a>(files: &'a [PathBuf], active_file: &Path) -> Option<&'a PathBuf> {
    files
        .iter()
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("csproj"))
        })
        .filter(|path| {
            path.parent()
                .is_some_and(|dir| active_file.starts_with(dir))
        })
        .max_by_key(|path| {
            path.parent()
                .map(|dir| dir.components().count())
                .unwrap_or(0)
        })
}

fn target_rank(path: &Path, nearest: Option<&Path>) -> u8 {
    if nearest.is_some_and(|nearest| nearest == path) {
        0
    } else if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("csproj"))
    {
        1
    } else {
        2
    }
}

fn display_under_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
#[path = "dotnet_tests.rs"]
mod tests;
