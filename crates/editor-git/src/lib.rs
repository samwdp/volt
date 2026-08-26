#![doc = r#"Git status parsing, repository file discovery, identity probes, and magit-style section modeling."#]

mod probe;
mod repository_files;

use std::{error::Error, fmt, io, path::Path, process::Command};

pub use probe::{
    GitProbeSnapshot, git_probe_generation, git_probe_snapshot, git_probe_snapshot_with_numstat,
    invalidate_git_probe_cache, invalidate_git_probe_cache_for, last_probe_generation,
    parse_git_numstat,
};
pub use repository_files::{
    REPOSITORY_FILE_PREVIEW_MAX_BYTES, REPOSITORY_FILE_PREVIEW_MAX_LINES,
    invalidate_repository_file_list_cache, invalidate_repository_file_list_cache_for,
    list_repository_files, list_repository_files_uncached, repository_file_list_generation,
    repository_file_preview,
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Git status parsing, repository file discovery, identity probes, and magit-style section modeling.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) fn configure_background_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

/// One file entry in a git status listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    path: String,
    index_status: char,
    worktree_status: char,
}

impl StatusEntry {
    /// Creates a new status entry.
    pub fn new(path: impl Into<String>, index_status: char, worktree_status: char) -> Self {
        Self {
            path: path.into(),
            index_status,
            worktree_status,
        }
    }

    /// Returns the file path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the staged/index status code.
    pub const fn index_status(&self) -> char {
        self.index_status
    }

    /// Returns the worktree status code.
    pub const fn worktree_status(&self) -> char {
        self.worktree_status
    }
}

/// Parsed repository status broken into magit-style sections.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RepositoryStatus {
    branch: Option<String>,
    ahead: usize,
    behind: usize,
    staged: Vec<StatusEntry>,
    unstaged: Vec<StatusEntry>,
    untracked: Vec<String>,
}

impl RepositoryStatus {
    /// Creates a new repository status snapshot.
    pub fn new(
        branch: Option<String>,
        ahead: usize,
        behind: usize,
        staged: Vec<StatusEntry>,
        unstaged: Vec<StatusEntry>,
        untracked: Vec<String>,
    ) -> Self {
        Self {
            branch,
            ahead,
            behind,
            staged,
            unstaged,
            untracked,
        }
    }

    /// Returns the current branch head description.
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    /// Returns ahead count relative to upstream.
    pub const fn ahead(&self) -> usize {
        self.ahead
    }

    /// Returns behind count relative to upstream.
    pub const fn behind(&self) -> usize {
        self.behind
    }

    /// Returns staged entries.
    pub fn staged(&self) -> &[StatusEntry] {
        &self.staged
    }

    /// Returns unstaged entries.
    pub fn unstaged(&self) -> &[StatusEntry] {
        &self.unstaged
    }

    /// Returns untracked paths.
    pub fn untracked(&self) -> &[String] {
        &self.untracked
    }
}

/// One commit entry used in status log sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitLogEntry {
    hash: String,
    summary: String,
}

impl GitLogEntry {
    /// Creates a new git log entry.
    pub fn new(hash: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            summary: summary.into(),
        }
    }

    /// Returns the commit hash.
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Returns the summary text.
    pub fn summary(&self) -> &str {
        &self.summary
    }
}

/// One stash entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStashEntry {
    name: String,
    summary: String,
}

impl GitStashEntry {
    /// Creates a new git stash entry.
    pub fn new(name: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            summary: summary.into(),
        }
    }

    /// Returns the stash identifier.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the stash summary.
    pub fn summary(&self) -> &str {
        &self.summary
    }
}

/// Snapshot of git status data used by UI renderers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitStatusSnapshot {
    branch: Option<String>,
    upstream: Option<String>,
    push_remote: Option<String>,
    tag: Option<String>,
    ahead: usize,
    behind: usize,
    head: Option<GitLogEntry>,
    staged: Vec<StatusEntry>,
    unstaged: Vec<StatusEntry>,
    untracked: Vec<String>,
    stashes: Vec<GitStashEntry>,
    unpulled: Vec<GitLogEntry>,
    unpushed: Vec<GitLogEntry>,
    recent: Vec<GitLogEntry>,
    in_progress: Vec<String>,
}

impl GitStatusSnapshot {
    /// Returns the branch name.
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    /// Returns the upstream ref name.
    pub fn upstream(&self) -> Option<&str> {
        self.upstream.as_deref()
    }

    /// Returns the push-remote ref name.
    pub fn push_remote(&self) -> Option<&str> {
        self.push_remote.as_deref()
    }

    /// Returns nearest reachable tag name for `HEAD`.
    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    /// Returns ahead count relative to upstream.
    pub const fn ahead(&self) -> usize {
        self.ahead
    }

    /// Returns behind count relative to upstream.
    pub const fn behind(&self) -> usize {
        self.behind
    }

    /// Returns the head commit summary.
    pub fn head(&self) -> Option<&GitLogEntry> {
        self.head.as_ref()
    }

    /// Returns staged entries.
    pub fn staged(&self) -> &[StatusEntry] {
        &self.staged
    }

    /// Returns unstaged entries.
    pub fn unstaged(&self) -> &[StatusEntry] {
        &self.unstaged
    }

    /// Returns untracked paths.
    pub fn untracked(&self) -> &[String] {
        &self.untracked
    }

    /// Returns stash entries.
    pub fn stashes(&self) -> &[GitStashEntry] {
        &self.stashes
    }

    /// Returns unpulled commits.
    pub fn unpulled(&self) -> &[GitLogEntry] {
        &self.unpulled
    }

    /// Returns unpushed commits.
    pub fn unpushed(&self) -> &[GitLogEntry] {
        &self.unpushed
    }

    /// Returns recent commits.
    pub fn recent(&self) -> &[GitLogEntry] {
        &self.recent
    }

    /// Returns in-progress operation summaries.
    pub fn in_progress(&self) -> &[String] {
        &self.in_progress
    }

    /// Updates the snapshot with status data.
    pub fn with_status(mut self, status: RepositoryStatus) -> Self {
        let RepositoryStatus {
            branch,
            ahead,
            behind,
            staged,
            unstaged,
            untracked,
        } = status;
        self.branch = branch;
        self.ahead = ahead;
        self.behind = behind;
        self.staged = staged;
        self.unstaged = unstaged;
        self.untracked = untracked;
        self
    }

    /// Adds the head commit entry.
    pub fn with_head(mut self, head: Option<GitLogEntry>) -> Self {
        self.head = head;
        self
    }

    /// Adds upstream and push-remote identifiers.
    pub fn with_upstreams(mut self, upstream: Option<String>, push_remote: Option<String>) -> Self {
        self.upstream = upstream;
        self.push_remote = push_remote;
        self
    }

    /// Adds nearest reachable tag for the current `HEAD`.
    pub fn with_tag(mut self, tag: Option<String>) -> Self {
        self.tag = tag;
        self
    }

    /// Adds stash entries.
    pub fn with_stashes(mut self, stashes: Vec<GitStashEntry>) -> Self {
        self.stashes = stashes;
        self
    }

    /// Adds unpulled commits.
    pub fn with_unpulled(mut self, unpulled: Vec<GitLogEntry>) -> Self {
        self.unpulled = unpulled;
        self
    }

    /// Adds unpushed commits.
    pub fn with_unpushed(mut self, unpushed: Vec<GitLogEntry>) -> Self {
        self.unpushed = unpushed;
        self
    }

    /// Adds recent commits.
    pub fn with_recent(mut self, recent: Vec<GitLogEntry>) -> Self {
        self.recent = recent;
        self
    }

    /// Adds in-progress operation summaries.
    pub fn with_in_progress(mut self, in_progress: Vec<String>) -> Self {
        self.in_progress = in_progress;
        self
    }
}

/// Parses `git log --oneline` output into commit entries.
pub fn parse_log_oneline(text: &str) -> Vec<GitLogEntry> {
    let mut entries = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (hash, summary) = match line.split_once(' ') {
            Some((hash, summary)) => (hash, summary),
            None => (line, ""),
        };
        entries.push(GitLogEntry {
            hash: hash.to_owned(),
            summary: summary.to_owned(),
        });
    }
    entries
}

/// Parses `git stash list` output into stash entries.
pub fn parse_stash_list(text: &str) -> Vec<GitStashEntry> {
    let mut entries = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (name, summary) = match line.split_once(':') {
            Some((name, summary)) => (name.trim(), summary.trim()),
            None => (line, ""),
        };
        entries.push(GitStashEntry {
            name: name.to_owned(),
            summary: summary.to_owned(),
        });
    }
    entries
}

/// Errors raised while parsing git status output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitStatusError {
    /// A line did not match expected porcelain status syntax.
    InvalidLine(String),
}

impl fmt::Display for GitStatusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLine(line) => write!(formatter, "invalid git status line: `{line}`"),
        }
    }
}

impl Error for GitStatusError {}

/// Errors raised while querying repository files from Git.
#[derive(Debug)]
pub enum RepositoryFilesError {
    /// The git process could not be started.
    Io(io::Error),
    /// Git exited unsuccessfully while listing files.
    CommandFailed(String),
}

impl fmt::Display for RepositoryFilesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "failed to run git: {error}"),
            Self::CommandFailed(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for RepositoryFilesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::CommandFailed(_) => None,
        }
    }
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

/// Parses `git status --short --branch` output into structured sections.
pub fn parse_status(text: &str) -> Result<RepositoryStatus, GitStatusError> {
    let mut status = RepositoryStatus::default();

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }

        if let Some(header) = line.strip_prefix("## ") {
            parse_header(header, &mut status);
            continue;
        }

        if let Some(path) = line.strip_prefix("?? ") {
            status.untracked.push(path.to_owned());
            continue;
        }

        if line.len() < 3 {
            return Err(GitStatusError::InvalidLine(line.to_owned()));
        }

        let chars: Vec<char> = line.chars().collect();
        let index_status = chars[0];
        let worktree_status = chars[1];
        let path = line[3..].to_owned();
        let entry = StatusEntry {
            path,
            index_status,
            worktree_status,
        };

        if index_status != ' ' {
            status.staged.push(entry.clone());
        }
        if worktree_status != ' ' {
            status.unstaged.push(entry);
        }
    }

    Ok(status)
}

fn parse_header(header: &str, status: &mut RepositoryStatus) {
    if let Some(branch) = header.strip_prefix("No commits yet on ") {
        status.branch = Some(branch.to_owned());
        return;
    }
    if let Some(branch) = header.strip_prefix("Initial commit on ") {
        status.branch = Some(branch.to_owned());
        return;
    }

    let mut parts = header.split("...");
    status.branch = parts.next().map(str::to_owned);

    if let Some(upstream_part) = parts.next()
        && let Some(summary_start) = upstream_part.find('[')
    {
        let summary = &upstream_part[summary_start + 1..upstream_part.len().saturating_sub(1)];
        for token in summary.split(',').map(str::trim) {
            if let Some(value) = token.strip_prefix("ahead ") {
                status.ahead = value.parse().unwrap_or_default();
            } else if let Some(value) = token.strip_prefix("behind ") {
                status.behind = value.parse().unwrap_or_default();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        REPOSITORY_FILE_PREVIEW_MAX_LINES, invalidate_repository_file_list_cache,
        invalidate_repository_file_list_cache_for, list_repository_files, parse_log_oneline,
        parse_stash_list, parse_status, repository_file_list_generation, repository_file_preview,
    };
    use std::sync::{Mutex, MutexGuard};

    fn repository_file_list_test_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn configure_git_identity(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        run_git(root, &["config", "user.email", "volt@example.com"])?;
        run_git(root, &["config", "user.name", "volt"])?;
        Ok(())
    }

    fn temp_repo_root(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-editor-git-{name}-{unique}"))
    }

    fn git_available() -> bool {
        Command::new("git").arg("--version").output().is_ok()
    }

    fn run_git(root: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new("git").args(args).current_dir(root).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("git {:?} failed with status {status}", args).into())
        }
    }

    fn git_stdout(root: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(args).current_dir(root).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            Err(format!("git {:?} failed with status {}", args, output.status).into())
        }
    }

    #[test]
    fn parser_extracts_branch_and_sections() {
        let status = parse_status(
            "## main...origin/main [ahead 2, behind 1]\nM  src/main.rs\n M README.md\n?? notes.txt\n",
        )
        .expect("status");

        assert_eq!(status.branch(), Some("main"));
        assert_eq!(status.ahead(), 2);
        assert_eq!(status.behind(), 1);
        assert_eq!(status.staged().len(), 1);
        assert_eq!(status.unstaged().len(), 1);
        assert_eq!(status.untracked(), ["notes.txt"]);
    }

    #[test]
    fn parser_extracts_unborn_branch_name() {
        let status =
            parse_status("## No commits yet on master\n?? notes.txt\n").expect("unborn status");

        assert_eq!(status.branch(), Some("master"));
        assert_eq!(status.untracked(), ["notes.txt"]);
        assert!(status.staged().is_empty());
        assert!(status.unstaged().is_empty());
    }

    #[test]
    fn repository_file_listing_excludes_gitignored_paths() -> Result<(), Box<dyn std::error::Error>>
    {
        let _lock = repository_file_list_test_lock();
        if !git_available() {
            return Ok(());
        }

        let root = temp_repo_root("files");
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join(".gitignore"), "ignored.txt\n")?;
        fs::write(root.join("src").join("main.rs"), "fn main() {}\n")?;
        fs::write(root.join("notes.txt"), "notes\n")?;
        fs::write(root.join("ignored.txt"), "ignored\n")?;

        run_git(&root, &["init", "-q"])?;
        run_git(&root, &["add", ".gitignore", "src/main.rs"])?;

        let files = list_repository_files(&root)?;

        assert!(files.contains(&PathBuf::from(".gitignore")));
        assert!(files.contains(&PathBuf::from("src/main.rs")));
        assert!(files.contains(&PathBuf::from("notes.txt")));
        assert!(!files.contains(&PathBuf::from("ignored.txt")));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn cached_repository_file_listing_reuses_paths_until_identity_changes()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = repository_file_list_test_lock();
        if !git_available() {
            return Ok(());
        }

        invalidate_repository_file_list_cache();
        let root = temp_repo_root("cache-hit");
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join("src").join("main.rs"), "fn main() {}\n")?;
        run_git(&root, &["init", "-q"])?;
        run_git(&root, &["add", "src/main.rs"])?;

        let generation_before = repository_file_list_generation();
        let first = list_repository_files(&root)?;
        let generation_after_miss = repository_file_list_generation();
        let second = list_repository_files(&root)?;
        let generation_after_hit = repository_file_list_generation();

        assert!(first.contains(&PathBuf::from("src/main.rs")));
        assert_eq!(first, second);
        assert!(generation_after_miss > generation_before);
        assert_eq!(generation_after_hit, generation_after_miss);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn cached_repository_file_listing_is_keyed_by_workspace_root()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = repository_file_list_test_lock();
        if !git_available() {
            return Ok(());
        }

        invalidate_repository_file_list_cache();
        let first_root = temp_repo_root("cache-root-a");
        let second_root = temp_repo_root("cache-root-b");
        fs::create_dir_all(&first_root)?;
        fs::create_dir_all(&second_root)?;
        fs::write(first_root.join("alpha.txt"), "a\n")?;
        fs::write(second_root.join("beta.txt"), "b\n")?;
        for root in [&first_root, &second_root] {
            run_git(root, &["init", "-q"])?;
            run_git(root, &["add", "."])?;
        }

        let first = list_repository_files(&first_root)?;
        let generation_after_first = repository_file_list_generation();
        let second = list_repository_files(&second_root)?;
        let generation_after_second = repository_file_list_generation();
        let first_again = list_repository_files(&first_root)?;

        assert!(first.contains(&PathBuf::from("alpha.txt")));
        assert!(!first.contains(&PathBuf::from("beta.txt")));
        assert!(second.contains(&PathBuf::from("beta.txt")));
        assert!(!second.contains(&PathBuf::from("alpha.txt")));
        assert_eq!(first, first_again);
        assert!(generation_after_second > generation_after_first);
        assert_eq!(
            repository_file_list_generation(),
            generation_after_second,
            "listing the first root again should reuse its cache entry"
        );

        fs::remove_dir_all(first_root)?;
        fs::remove_dir_all(second_root)?;
        Ok(())
    }

    #[test]
    fn cached_repository_file_listing_refreshes_after_index_or_head_change()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = repository_file_list_test_lock();
        if !git_available() {
            return Ok(());
        }

        invalidate_repository_file_list_cache();
        let root = temp_repo_root("cache-head");
        fs::create_dir_all(&root)?;
        fs::write(root.join("main.txt"), "main\n")?;
        run_git(&root, &["init", "-q"])?;
        configure_git_identity(&root)?;
        run_git(&root, &["add", "main.txt"])?;
        run_git(&root, &["commit", "-qm", "init"])?;
        let default_branch = git_stdout(&root, &["rev-parse", "--abbrev-ref", "HEAD"])?;

        let first = list_repository_files(&root)?;
        let generation_after_first = repository_file_list_generation();
        assert!(first.contains(&PathBuf::from("main.txt")));
        assert!(!first.contains(&PathBuf::from("extra.txt")));

        fs::write(root.join("extra.txt"), "extra\n")?;
        run_git(&root, &["add", "extra.txt"])?;
        let after_index = list_repository_files(&root)?;
        let generation_after_index = repository_file_list_generation();
        assert!(after_index.contains(&PathBuf::from("extra.txt")));
        assert!(generation_after_index > generation_after_first);

        run_git(&root, &["commit", "-qm", "add extra"])?;
        run_git(&root, &["checkout", "-qb", "other"])?;
        fs::write(root.join("other.txt"), "other\n")?;
        run_git(&root, &["add", "other.txt"])?;
        run_git(&root, &["commit", "-qm", "other branch"])?;
        run_git(&root, &["checkout", "-q", &default_branch])?;
        let after_checkout = list_repository_files(&root)?;
        assert!(after_checkout.contains(&PathBuf::from("extra.txt")));
        assert!(!after_checkout.contains(&PathBuf::from("other.txt")));
        assert!(repository_file_list_generation() > generation_after_index);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn invalidate_repository_file_list_cache_forces_rescan()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = repository_file_list_test_lock();
        if !git_available() {
            return Ok(());
        }

        invalidate_repository_file_list_cache();
        let root = temp_repo_root("cache-invalidate");
        fs::create_dir_all(&root)?;
        fs::write(root.join("keep.txt"), "keep\n")?;
        run_git(&root, &["init", "-q"])?;
        run_git(&root, &["add", "keep.txt"])?;

        let _ = list_repository_files(&root)?;
        let generation_after_first = repository_file_list_generation();
        invalidate_repository_file_list_cache_for(&root);
        let _ = list_repository_files(&root)?;
        assert!(repository_file_list_generation() > generation_after_first);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn repository_file_preview_includes_path_and_caps_lines()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_repo_root("preview-caps");
        fs::create_dir_all(&root)?;
        let path = root.join("notes.txt");
        let body = (0..40)
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, body)?;

        let preview = repository_file_preview(&path);
        let lines = preview.lines().collect::<Vec<_>>();
        assert_eq!(lines[0], path.display().to_string());
        assert_eq!(lines.len(), REPOSITORY_FILE_PREVIEW_MAX_LINES + 1);
        assert_eq!(lines[1], "line-0");
        assert_eq!(
            lines[REPOSITORY_FILE_PREVIEW_MAX_LINES],
            format!("line-{}", REPOSITORY_FILE_PREVIEW_MAX_LINES - 1)
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn repository_file_preview_falls_back_to_path_for_invalid_utf8()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_repo_root("preview-binary");
        fs::create_dir_all(&root)?;
        let path = root.join("blob.bin");
        fs::write(&path, [0xff, 0xfe, 0x00, 0x01])?;

        assert_eq!(repository_file_preview(&path), path.display().to_string());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn parses_log_oneline_entries() {
        let entries = parse_log_oneline("abc123 first\nfed456 second commit\n");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash(), "abc123");
        assert_eq!(entries[0].summary(), "first");
        assert_eq!(entries[1].hash(), "fed456");
        assert_eq!(entries[1].summary(), "second commit");
    }

    #[test]
    fn parses_stash_list_entries() {
        let entries = parse_stash_list("stash@{0}: WIP on main\nstash@{1}: update\n");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name(), "stash@{0}");
        assert_eq!(entries[0].summary(), "WIP on main");
        assert_eq!(entries[1].name(), "stash@{1}");
        assert_eq!(entries[1].summary(), "update");
    }
}
