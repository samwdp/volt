#![doc = r#"Git status parsing, repository file discovery, identity probes, and magit-style section modeling."#]

mod probe;

use std::path::Path;

pub use editor_plugin_api::{
    GitLogEntry, GitStashEntry, GitStatusError, GitStatusSnapshot,
    REPOSITORY_FILE_PREVIEW_MAX_BYTES, REPOSITORY_FILE_PREVIEW_MAX_LINES, RepositoryFilesError,
    RepositoryStatus, StatusEntry, invalidate_repository_file_list_cache,
    invalidate_repository_file_list_cache_for, list_repository_files,
    list_repository_files_uncached, parse_log_oneline, parse_stash_list, parse_status,
    repository_file_list_generation, repository_file_preview,
};
pub use probe::{
    GitProbeSnapshot, git_probe_generation, git_probe_snapshot, git_probe_snapshot_with_numstat,
    invalidate_git_probe_cache, invalidate_git_probe_cache_for, last_probe_generation,
    parse_git_numstat,
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Git status parsing, repository file discovery, identity probes, and magit-style section modeling.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Detects in-progress operations by inspecting the git directory.
pub fn detect_in_progress(git_dir: impl AsRef<Path>) -> Vec<String> {
    let git_dir = git_dir.as_ref();
    let mut entries = Vec::new();
    let merge_head = git_dir.join("MERGE_HEAD");
    if merge_head.is_file() {
        entries.push("Merge in progress".to_owned());
    }
    let cherry_pick_head = git_dir.join("CHERRY_PICK_HEAD");
    if cherry_pick_head.is_file() {
        entries.push("Cherry-pick in progress".to_owned());
    }
    let revert_head = git_dir.join("REVERT_HEAD");
    if revert_head.is_file() {
        entries.push("Revert in progress".to_owned());
    }
    let rebase_apply = git_dir.join("rebase-apply");
    let rebase_merge = git_dir.join("rebase-merge");
    if rebase_apply.is_dir() || rebase_merge.is_dir() {
        entries.push("Rebase in progress".to_owned());
    }
    let bisect_log = git_dir.join("BISECT_LOG");
    if bisect_log.is_file() {
        entries.push("Bisect in progress".to_owned());
    }
    let sequencer = git_dir.join("sequencer");
    if sequencer.is_dir() {
        entries.push("Sequencer in progress".to_owned());
    }
    entries
}

#[cfg(test)]
mod tests;
