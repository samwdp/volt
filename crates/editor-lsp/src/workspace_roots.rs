#![allow(unused_imports)]
use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use editor_buffer::TextRange;
use editor_jobs::JobSpec;
use editor_path::{PathMatcher, PathPattern, normalize_extension};
use serde_json::{Number, Value};

pub use editor_tool_install::InstallRecipe;

#[allow(unused_imports)]
use crate::registry::*;

pub(crate) fn normalize_unique_entries<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for value in values {
        let value = value.into();
        let value = value.trim();
        if !value.is_empty() && !normalized.iter().any(|existing| existing == value) {
            normalized.push(value.to_owned());
        }
    }
    normalized
}

pub(crate) fn normalize_optional_string(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

pub(crate) fn document_language_id_for_path<'a>(
    document_language_ids: &'a BTreeMap<String, String>,
    file_name: Option<&str>,
    extension: Option<&str>,
    default_language_id: &'a str,
) -> &'a str {
    if let Some(file_name) = file_name {
        if let Some(language_id) = document_language_ids.get(file_name) {
            return language_id;
        }
        if let Some(language_id) = document_language_id_for_glob(document_language_ids, file_name) {
            return language_id;
        }
    }
    if let Some(extension) = extension {
        return document_language_id_for_extension(
            document_language_ids,
            extension,
            default_language_id,
        );
    }
    default_language_id
}

pub(crate) fn document_language_id_for_extension<'a>(
    document_language_ids: &'a BTreeMap<String, String>,
    extension: &str,
    default_language_id: &'a str,
) -> &'a str {
    let extension = normalize_extension(extension);
    document_language_ids
        .iter()
        .find_map(|(path_matcher, language_id)| {
            (normalize_extension(path_matcher) == extension).then_some(language_id.as_str())
        })
        .unwrap_or(default_language_id)
}

pub(crate) fn document_language_id_for_glob<'a>(
    document_language_ids: &'a BTreeMap<String, String>,
    file_name: &str,
) -> Option<&'a str> {
    let mut best = None;
    let mut best_score = 0;
    for (path_matcher, language_id) in document_language_ids {
        let Some(path_matcher) = PathPattern::glob(path_matcher) else {
            continue;
        };
        let Some(score) = path_matcher.match_score_for_file_name(file_name) else {
            continue;
        };
        if best.is_none() || score > best_score {
            best = Some(language_id.as_str());
            best_score = score;
        }
    }
    best
}

pub(crate) fn find_root_for_path(
    path: &Path,
    workspace_root: Option<&Path>,
    root_markers: &[String],
) -> Option<PathBuf> {
    if root_markers.is_empty() {
        return None;
    }
    // Earlier markers are preferred over later ones across the whole ancestor walk.
    for marker in root_markers {
        if let Some(root) = find_root_for_path_matching_marker(path, workspace_root, marker) {
            return Some(root);
        }
    }
    None
}

pub(crate) fn find_root_for_path_matching_marker(
    path: &Path,
    workspace_root: Option<&Path>,
    marker: &str,
) -> Option<PathBuf> {
    let bounded_workspace_root = workspace_root.filter(|root| path.starts_with(root));
    let stop_at_workspace = !is_solution_glob_marker(marker);
    let mut current = path.parent();
    while let Some(directory) = current {
        if directory.parent().is_none() {
            break;
        }
        if directory_matches_root_marker(directory, marker) {
            return Some(directory.to_path_buf());
        }
        if should_stop_root_marker_walk(directory, bounded_workspace_root, stop_at_workspace) {
            break;
        }
        current = directory.parent();
    }
    solution_glob_extension(marker)
        .and_then(|extension| unique_workspace_match_for_extension(workspace_root, extension))
}

pub(crate) fn should_stop_root_marker_walk(
    directory: &Path,
    workspace_root: Option<&Path>,
    stop_at_workspace: bool,
) -> bool {
    if stop_at_workspace {
        return workspace_root.is_some_and(|root| root == directory);
    }
    if workspace_root.is_some_and(|root| root == directory) {
        return !directory_matches_root_marker(directory, "*.csproj");
    }
    directory_is_git_root(directory)
}

pub(crate) fn is_solution_glob_marker(marker: &str) -> bool {
    solution_glob_extension(marker).is_some()
}

pub(crate) fn solution_glob_extension(marker: &str) -> Option<&str> {
    let extension = marker.strip_prefix("*.")?;
    (extension.eq_ignore_ascii_case("sln") || extension.eq_ignore_ascii_case("slnx"))
        .then_some(extension)
}

pub(crate) fn directory_is_git_root(directory: &Path) -> bool {
    fs::metadata(directory.join(".git")).is_ok()
}

pub(crate) const SOLUTION_SEARCH_MAX_DEPTH: usize = 8;

pub(crate) fn unique_workspace_match_for_extension(
    workspace_root: Option<&Path>,
    extension: &str,
) -> Option<PathBuf> {
    let workspace_root = workspace_root?;
    let mut matches = Vec::new();
    collect_files_with_extension(workspace_root, extension, 0, &mut matches);
    if matches.len() != 1 {
        return None;
    }
    matches.pop()?.parent().map(Path::to_path_buf)
}

pub(crate) fn collect_files_with_extension(
    directory: &Path,
    extension: &str,
    depth: usize,
    found: &mut Vec<PathBuf>,
) {
    if found.len() > 1 || depth > SOLUTION_SEARCH_MAX_DEPTH {
        return;
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        if found.len() > 1 {
            return;
        }
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if should_skip_solution_search_dir(&path) {
                continue;
            }
            collect_files_with_extension(&path, extension, depth + 1, found);
            continue;
        }
        if file_type.is_file()
            && path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(extension))
        {
            found.push(path);
        }
    }
}

pub(crate) fn should_skip_solution_search_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "bin" | "obj" | ".git" | ".vs" | "node_modules" | "target" | "packages" | "dist"
            )
        })
}

pub(crate) fn directory_matches_root_marker(directory: &Path, marker: &str) -> bool {
    if let Some(extension) = marker.strip_prefix("*.") {
        return directory_contains_extension(directory, extension);
    }
    fs::metadata(directory.join(marker)).is_ok()
}

pub(crate) fn directory_contains_extension(directory: &Path, extension: &str) -> bool {
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

pub(crate) fn unique_solution_file_name(root: Option<&Path>) -> Option<String> {
    resolve_single_solution_path(root)?
        .file_name()?
        .to_str()
        .map(str::to_owned)
}

pub(crate) fn resolve_single_solution_path(root: Option<&Path>) -> Option<PathBuf> {
    let root = root?;
    if path_is_solution(root) {
        return Some(root.to_path_buf());
    }
    let mut solutions = fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path_is_solution(path))
        .collect::<Vec<_>>();
    (solutions.len() == 1).then(|| solutions.pop()).flatten()
}

pub(crate) fn path_is_solution(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("sln"))
}
