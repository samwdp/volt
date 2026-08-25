use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::SystemTime,
};

use crate::{RepositoryFilesError, configure_background_command};

/// Byte cap for Workspace Files picker previews.
pub const REPOSITORY_FILE_PREVIEW_MAX_BYTES: u64 = 16 * 1024;
/// Line cap for Workspace Files picker previews (body lines after the path header).
pub const REPOSITORY_FILE_PREVIEW_MAX_LINES: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileFingerprint {
    modified: Option<SystemTime>,
    len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepoListIdentity {
    head: Vec<u8>,
    resolved_ref: Option<Vec<u8>>,
    index: Option<FileFingerprint>,
    packed_refs: Option<FileFingerprint>,
}

struct CachedRepoFileList {
    identity: RepoListIdentity,
    files: Vec<PathBuf>,
}

fn repository_file_list_cache() -> &'static Mutex<HashMap<PathBuf, CachedRepoFileList>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, CachedRepoFileList>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_cache() -> std::sync::MutexGuard<'static, HashMap<PathBuf, CachedRepoFileList>> {
    repository_file_list_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn list_generation() -> &'static AtomicU64 {
    static GENERATION: AtomicU64 = AtomicU64::new(0);
    &GENERATION
}

/// Returns how many times `git ls-files` has been spawned for a cache miss.
pub fn repository_file_list_generation() -> u64 {
    list_generation().load(Ordering::Relaxed)
}

/// Drops every cached repository file list.
pub fn invalidate_repository_file_list_cache() {
    lock_cache().clear();
}

/// Drops the cached repository file list for one Project Workspace root.
pub fn invalidate_repository_file_list_cache_for(root: impl AsRef<Path>) {
    let root = root.as_ref();
    let mut cache = lock_cache();
    cache.remove(&cache_key(root));
    cache.remove(&root.to_path_buf());
}

/// Returns tracked and unignored files, reusing a per-root cache until HEAD/index identity changes.
pub fn list_repository_files(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, RepositoryFilesError> {
    let root = root.as_ref();
    let key = cache_key(root);
    if let Some(identity) = repository_list_identity(root) {
        let cache = lock_cache();
        if let Some(entry) = cache.get(&key)
            && entry.identity == identity
        {
            return Ok(entry.files.clone());
        }
    }

    list_generation().fetch_add(1, Ordering::Relaxed);
    let files = list_repository_files_uncached(root)?;
    if let Some(identity) = repository_list_identity(root) {
        lock_cache().insert(
            key,
            CachedRepoFileList {
                identity,
                files: files.clone(),
            },
        );
    }
    Ok(files)
}

/// Runs `git ls-files` without consulting the cache.
pub fn list_repository_files_uncached(
    root: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, RepositoryFilesError> {
    let root = root.as_ref();
    let mut command = Command::new("git");
    configure_background_command(&mut command);
    let output = command
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--full-name",
        ])
        .current_dir(root)
        .output()
        .map_err(RepositoryFilesError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if stderr.is_empty() {
            format!(
                "git ls-files failed in `{}` with status {}",
                root.display(),
                output.status
            )
        } else {
            format!("git ls-files failed in `{}`: {stderr}", root.display())
        };
        return Err(RepositoryFilesError::CommandFailed(message));
    }

    let mut files = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| PathBuf::from(String::from_utf8_lossy(entry).into_owned()))
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    Ok(files)
}

/// Builds a capped text preview for a repository file, falling back to the path for binary data.
pub fn repository_file_preview(path: &Path) -> String {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return path.display().to_string(),
    };
    let mut buffer = String::new();
    if file
        .take(REPOSITORY_FILE_PREVIEW_MAX_BYTES)
        .read_to_string(&mut buffer)
        .is_err()
    {
        return path.display().to_string();
    }
    let mut lines = Vec::new();
    lines.push(path.display().to_string());
    lines.extend(
        buffer
            .lines()
            .take(REPOSITORY_FILE_PREVIEW_MAX_LINES)
            .map(|line| line.trim_end().to_owned()),
    );
    lines.join("\n")
}

fn cache_key(root: &Path) -> PathBuf {
    fs::canonicalize(root).unwrap_or_else(|_| normalize_path(root))
}

fn repository_list_identity(root: &Path) -> Option<RepoListIdentity> {
    let (git_dir, common_dir) = resolve_git_dirs(root)?;
    let head_path = git_dir.join("HEAD");
    let head = fs::read(&head_path).ok()?;
    let resolved_ref = symbolic_ref_name(&head)
        .and_then(|ref_name| fs::read(common_dir.join(ref_name)).ok())
        .or_else(|| {
            symbolic_ref_name(&head).and_then(|ref_name| fs::read(git_dir.join(ref_name)).ok())
        });
    Some(RepoListIdentity {
        head,
        resolved_ref,
        index: file_fingerprint(&git_dir.join("index")),
        packed_refs: file_fingerprint(&common_dir.join("packed-refs"))
            .or_else(|| file_fingerprint(&git_dir.join("packed-refs"))),
    })
}

fn symbolic_ref_name(head: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(head).ok()?.trim();
    text.strip_prefix("ref:")
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

fn file_fingerprint(path: &Path) -> Option<FileFingerprint> {
    let metadata = fs::metadata(path).ok()?;
    Some(FileFingerprint {
        modified: metadata.modified().ok(),
        len: metadata.len(),
    })
}

fn resolve_git_dirs(root: &Path) -> Option<(PathBuf, PathBuf)> {
    let marker = root.join(".git");
    if marker.is_dir() {
        return Some((marker.clone(), marker));
    }
    if !marker.is_file() {
        return None;
    }
    let contents = fs::read_to_string(&marker).ok()?;
    let gitdir = parse_gitdir_reference(&contents)
        .map(|reference| resolve_git_path(root, reference))
        .filter(|gitdir| gitdir.exists())?;
    let common_dir = worktree_common_dir(&gitdir).unwrap_or_else(|| gitdir.clone());
    Some((gitdir, common_dir))
}

fn parse_gitdir_reference(contents: &str) -> Option<&str> {
    contents
        .lines()
        .find_map(|line| line.trim().strip_prefix("gitdir:").map(str::trim))
        .filter(|reference| !reference.is_empty())
}

fn worktree_common_dir(gitdir: &Path) -> Option<PathBuf> {
    let commondir_path = gitdir.join("commondir");
    let common_dir = match fs::read_to_string(&commondir_path) {
        Ok(contents) => parse_relative_git_path(gitdir, &contents)
            .unwrap_or_else(|| default_worktree_common_dir(gitdir)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            default_worktree_common_dir(gitdir)
        }
        Err(_) => return None,
    };
    common_dir.exists().then_some(common_dir)
}

fn default_worktree_common_dir(gitdir: &Path) -> PathBuf {
    gitdir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| gitdir.to_path_buf())
}

fn parse_relative_git_path(base: &Path, contents: &str) -> Option<PathBuf> {
    contents
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|reference| resolve_git_path(base, reference))
}

fn resolve_git_path(base: &Path, reference: &str) -> PathBuf {
    #[cfg(windows)]
    if let Some(path) = windows_git_absolute_path(reference) {
        return normalize_path(&path);
    }
    let reference = Path::new(reference);
    if reference.is_absolute() {
        normalize_path(reference)
    } else {
        normalize_path(&base.join(reference))
    }
}

#[cfg(windows)]
fn windows_git_absolute_path(reference: &str) -> Option<PathBuf> {
    let mut chars = reference.chars();
    let slash = chars.next()?;
    let drive = chars.next()?;
    let separator = chars.next()?;
    if slash != '/' || separator != '/' || !drive.is_ascii_alphabetic() {
        return None;
    }
    let suffix = chars.as_str();
    Some(PathBuf::from(format!(
        "{}:/{}",
        drive.to_ascii_uppercase(),
        suffix
    )))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    let mut anchored = false;
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                normalized.push(prefix.as_os_str());
                anchored = true;
            }
            Component::RootDir => {
                normalized.push(component.as_os_str());
                anchored = true;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() && !anchored {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    normalized
}
