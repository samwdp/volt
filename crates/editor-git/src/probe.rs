//! Cached git identity snapshot for one Project Workspace root.
//!
//! Reads `.git/HEAD` and gitdir files when that matches branch/present semantics.
//! Falls back to one `git rev-parse` only when packed/worktree layout is ambiguous.
//! Numstat is a separate spawn, keyed by HEAD/index identity.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::{
    configure_background_command,
    repository_files::{
        FileFingerprint, cache_key, file_fingerprint, resolve_git_dirs, resolve_git_path,
        worktree_common_dir,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProbeIdentity {
    marker: Option<FileFingerprint>,
    head: Option<Vec<u8>>,
    resolved_ref: Option<Vec<u8>>,
    index: Option<FileFingerprint>,
    packed_refs: Option<FileFingerprint>,
}

#[derive(Debug, Clone)]
struct CachedProbe {
    identity: ProbeIdentity,
    snapshot: GitProbeSnapshot,
    numstat: Option<(usize, usize)>,
    numstat_identity: Option<ProbeIdentity>,
}

/// Cached HEAD/branch/present (and optional numstat) for one Project Workspace root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitProbeSnapshot {
    present: bool,
    branch: Option<String>,
    head: Option<String>,
    added: usize,
    removed: usize,
    git_dir: Option<PathBuf>,
    revision: u64,
}

impl GitProbeSnapshot {
    fn absent(revision: u64) -> Self {
        Self {
            present: false,
            branch: None,
            head: None,
            added: 0,
            removed: 0,
            git_dir: None,
            revision,
        }
    }

    /// Whether this root is a git repository (including a worktree `.git` file).
    #[must_use]
    pub const fn present(&self) -> bool {
        self.present
    }

    /// Branch from `HEAD`, or `"HEAD"` when detached. Empty filtering is the caller's job.
    #[must_use]
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    /// Resolved HEAD object id, when one exists (unborn branches have none).
    #[must_use]
    pub fn head(&self) -> Option<&str> {
        self.head.as_deref()
    }

    /// Added line count from `git diff --numstat HEAD`, or `0` if numstat was not probed.
    #[must_use]
    pub const fn added(&self) -> usize {
        self.added
    }

    /// Removed line count from `git diff --numstat HEAD`, or `0` if numstat was not probed.
    #[must_use]
    pub const fn removed(&self) -> usize {
        self.removed
    }

    /// Resolved git directory for this root, when present.
    #[must_use]
    pub fn git_dir(&self) -> Option<&Path> {
        self.git_dir.as_deref()
    }

    /// Identity revision for this cache entry. Unchanged across cache hits.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Workspace Dock branch label: hides empty names and detached `HEAD`.
    #[must_use]
    pub fn dock_branch(&self) -> Option<&str> {
        self.branch
            .as_deref()
            .map(str::trim)
            .filter(|branch| !branch.is_empty() && *branch != "HEAD")
    }
}

fn probe_cache() -> &'static Mutex<HashMap<PathBuf, CachedProbe>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, CachedProbe>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_cache() -> std::sync::MutexGuard<'static, HashMap<PathBuf, CachedProbe>> {
    probe_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn spawn_generation() -> &'static AtomicU64 {
    static GENERATION: AtomicU64 = AtomicU64::new(0);
    &GENERATION
}

fn identity_revision() -> &'static AtomicU64 {
    static REVISION: AtomicU64 = AtomicU64::new(1);
    &REVISION
}

/// How many git processes the probe cache has spawned (rev-parse fallback or numstat).
#[must_use]
pub fn git_probe_generation() -> u64 {
    spawn_generation().load(Ordering::Relaxed)
}

/// Alias matching the issue's spawn-counter name.
#[must_use]
pub fn last_probe_generation() -> u64 {
    git_probe_generation()
}

/// Drops every cached identity snapshot.
pub fn invalidate_git_probe_cache() {
    lock_cache().clear();
}

/// Drops the cached identity snapshot for one Project Workspace root.
pub fn invalidate_git_probe_cache_for(root: impl AsRef<Path>) {
    let root = root.as_ref();
    let mut cache = lock_cache();
    cache.remove(&cache_key(root));
    cache.remove(&root.to_path_buf());
}

/// Returns HEAD, branch, and present for `root`, spawning git only on identity miss fallbacks.
#[must_use]
pub fn git_probe_snapshot(root: impl AsRef<Path>) -> GitProbeSnapshot {
    load_snapshot(root.as_ref(), false)
}

/// Same as [`git_probe_snapshot`], plus numstat when HEAD or the index identity changed.
#[must_use]
pub fn git_probe_snapshot_with_numstat(root: impl AsRef<Path>) -> GitProbeSnapshot {
    load_snapshot(root.as_ref(), true)
}

/// Parses `git diff --numstat` stdout into `(added, removed)` line counts.
#[must_use]
pub fn parse_git_numstat(output: &str) -> (usize, usize) {
    let mut added = 0usize;
    let mut removed = 0usize;
    for line in output.lines() {
        let mut parts = line.split('\t');
        let add_raw = parts.next().unwrap_or_default();
        let remove_raw = parts.next().unwrap_or_default();
        added = added.saturating_add(add_raw.parse::<usize>().unwrap_or(0));
        removed = removed.saturating_add(remove_raw.parse::<usize>().unwrap_or(0));
    }
    (added, removed)
}

fn load_snapshot(root: &Path, want_numstat: bool) -> GitProbeSnapshot {
    let key = cache_key(root);
    let identity = probe_identity(root);
    let cached = {
        let cache = lock_cache();
        cache.get(&key).cloned()
    };
    if let Some(entry) = cached.as_ref()
        && entry.identity == identity
    {
        if !want_numstat || entry.numstat_identity.as_ref() == Some(&identity) {
            return snapshot_from_entry(entry, want_numstat);
        }
        return fill_numstat(root, &key, identity, entry.snapshot.clone());
    }

    let snapshot = compute_identity_snapshot(root, &identity);
    if want_numstat {
        fill_numstat(root, &key, identity, snapshot)
    } else {
        lock_cache().insert(
            key,
            CachedProbe {
                identity,
                snapshot: snapshot.clone(),
                numstat: None,
                numstat_identity: None,
            },
        );
        snapshot
    }
}

fn snapshot_from_entry(entry: &CachedProbe, want_numstat: bool) -> GitProbeSnapshot {
    let mut snapshot = entry.snapshot.clone();
    if want_numstat && let Some((added, removed)) = entry.numstat {
        snapshot.added = added;
        snapshot.removed = removed;
    }
    snapshot
}

fn fill_numstat(
    root: &Path,
    key: &Path,
    identity: ProbeIdentity,
    mut snapshot: GitProbeSnapshot,
) -> GitProbeSnapshot {
    let value = if snapshot.present && snapshot.head.is_some() {
        probe_numstat(root)
    } else {
        (0, 0)
    };
    snapshot.added = value.0;
    snapshot.removed = value.1;
    let mut cache = lock_cache();
    match cache.get_mut(key) {
        Some(entry) if entry.identity == identity => {
            entry.numstat = Some(value);
            entry.numstat_identity = Some(identity);
            entry.snapshot.added = value.0;
            entry.snapshot.removed = value.1;
            snapshot.revision = entry.snapshot.revision;
        }
        _ => {
            let numstat_identity = Some(identity.clone());
            cache.insert(
                key.to_path_buf(),
                CachedProbe {
                    identity,
                    snapshot: snapshot.clone(),
                    numstat: Some(value),
                    numstat_identity,
                },
            );
        }
    }
    snapshot
}

fn compute_identity_snapshot(root: &Path, identity: &ProbeIdentity) -> GitProbeSnapshot {
    let Some(dirs) = resolve_probe_dirs(root) else {
        return GitProbeSnapshot::absent(next_identity_revision());
    };
    let (git_dir, common_dir) = dirs;
    let head_bytes = identity
        .head
        .clone()
        .or_else(|| fs::read(git_dir.join("HEAD")).ok());
    let parsed = head_bytes
        .as_deref()
        .map(parse_head)
        .unwrap_or(HeadParse::Missing);

    let (branch, head) = match parsed {
        HeadParse::Detached(sha) => (Some("HEAD".to_owned()), Some(sha)),
        HeadParse::Symbolic(ref_name) => {
            let branch = short_branch_name(&ref_name);
            let sha = resolve_ref_sha(&git_dir, &common_dir, &ref_name)
                .or_else(|| packed_ref_sha(&git_dir, &common_dir, &ref_name));
            match branch {
                Some(branch) => (Some(branch), sha),
                None => fallback_rev_parse(root, sha),
            }
        }
        HeadParse::Missing | HeadParse::Ambiguous => fallback_rev_parse(root, None),
    };

    GitProbeSnapshot {
        present: true,
        branch,
        head,
        added: 0,
        removed: 0,
        git_dir: Some(git_dir),
        revision: next_identity_revision(),
    }
}

fn next_identity_revision() -> u64 {
    identity_revision().fetch_add(1, Ordering::Relaxed)
}

fn resolve_probe_dirs(root: &Path) -> Option<(PathBuf, PathBuf)> {
    if let Some(dirs) = resolve_git_dirs(root) {
        return Some(dirs);
    }
    let marker = root.join(".git");
    if !marker.is_file() {
        return None;
    }
    let git_dir = rev_parse_git_dir(root)?;
    let common_dir = worktree_common_dir(&git_dir).unwrap_or_else(|| git_dir.clone());
    Some((git_dir, common_dir))
}

fn probe_identity(root: &Path) -> ProbeIdentity {
    let marker = file_fingerprint(&root.join(".git"));
    let Some((git_dir, common_dir)) = resolve_git_dirs(root) else {
        return ProbeIdentity {
            marker,
            head: None,
            resolved_ref: None,
            index: None,
            packed_refs: None,
        };
    };
    let head = fs::read(git_dir.join("HEAD")).ok();
    let resolved_ref = head
        .as_deref()
        .and_then(symbolic_ref_name)
        .and_then(|ref_name| read_ref_bytes(&git_dir, &common_dir, ref_name));
    ProbeIdentity {
        marker,
        head,
        resolved_ref,
        index: file_fingerprint(&git_dir.join("index")),
        packed_refs: file_fingerprint(&common_dir.join("packed-refs"))
            .or_else(|| file_fingerprint(&git_dir.join("packed-refs"))),
    }
}

fn symbolic_ref_name(head: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(head).ok()?.trim();
    text.strip_prefix("ref:")
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

fn read_ref_bytes(git_dir: &Path, common_dir: &Path, ref_name: &str) -> Option<Vec<u8>> {
    fs::read(ref_path(common_dir, ref_name))
        .ok()
        .or_else(|| fs::read(ref_path(git_dir, ref_name)).ok())
}

fn ref_path(base: &Path, ref_name: &str) -> PathBuf {
    let mut path = base.to_path_buf();
    for part in ref_name.split('/') {
        if !part.is_empty() {
            path.push(part);
        }
    }
    path
}

fn resolve_ref_sha(git_dir: &Path, common_dir: &Path, ref_name: &str) -> Option<String> {
    let bytes = read_ref_bytes(git_dir, common_dir, ref_name)?;
    parse_sha(std::str::from_utf8(&bytes).ok()?)
}

fn packed_ref_sha(git_dir: &Path, common_dir: &Path, ref_name: &str) -> Option<String> {
    let packed = fs::read_to_string(common_dir.join("packed-refs"))
        .ok()
        .or_else(|| fs::read_to_string(git_dir.join("packed-refs")).ok())?;
    parse_packed_ref(&packed, ref_name)
}

fn parse_packed_ref(contents: &str, ref_name: &str) -> Option<String> {
    let suffix = format!(" {ref_name}");
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        if let Some(sha) = line.strip_suffix(suffix.as_str()) {
            return parse_sha(sha);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HeadParse {
    Detached(String),
    Symbolic(String),
    Missing,
    Ambiguous,
}

fn parse_head(head: &[u8]) -> HeadParse {
    let Ok(text) = std::str::from_utf8(head) else {
        return HeadParse::Ambiguous;
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return HeadParse::Missing;
    }
    if let Some(ref_name) = trimmed.strip_prefix("ref:") {
        let ref_name = ref_name.trim();
        if ref_name.is_empty() {
            return HeadParse::Ambiguous;
        }
        return HeadParse::Symbolic(ref_name.to_owned());
    }
    match parse_sha(trimmed) {
        Some(sha) => HeadParse::Detached(sha),
        None => HeadParse::Ambiguous,
    }
}

fn parse_sha(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let len = trimmed.len();
    if (len == 40 || len == 64) && trimmed.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Some(trimmed.to_owned())
    } else {
        None
    }
}

fn short_branch_name(ref_name: &str) -> Option<String> {
    ref_name
        .strip_prefix("refs/heads/")
        .or_else(|| ref_name.strip_prefix("refs/remotes/"))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

fn fallback_rev_parse(root: &Path, known_sha: Option<String>) -> (Option<String>, Option<String>) {
    let branch = run_git(root, &["rev-parse", "--abbrev-ref", "HEAD"], &[0])
        .map(|output| output.trim().to_owned())
        .filter(|branch| !branch.is_empty());
    let head = known_sha.or_else(|| {
        run_git(root, &["rev-parse", "--verify", "HEAD"], &[0])
            .map(|output| output.trim().to_owned())
            .filter(|head| !head.is_empty())
    });
    (branch, head)
}

fn rev_parse_git_dir(root: &Path) -> Option<PathBuf> {
    let output = run_git(root, &["rev-parse", "--git-dir"], &[0])?;
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = resolve_git_path(root, trimmed);
    path.exists().then_some(path)
}

fn probe_numstat(root: &Path) -> (usize, usize) {
    let output = run_git(root, &["diff", "--numstat", "HEAD"], &[0, 1]).unwrap_or_default();
    parse_git_numstat(&output)
}

fn run_git(root: &Path, args: &[&str], allowed_exit_codes: &[i32]) -> Option<String> {
    spawn_generation().fetch_add(1, Ordering::Relaxed);
    let mut command = Command::new("git");
    configure_background_command(&mut command);
    let output = command.args(args).current_dir(root).output().ok()?;
    let exit_code = output.status.code()?;
    if exit_code != 0 && !allowed_exit_codes.contains(&exit_code) {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
#[path = "probe_tests.rs"]
mod tests;
