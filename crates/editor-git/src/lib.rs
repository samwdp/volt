#![doc = r#"Git status parsing, repository file discovery, and magit-style section modeling."#]

use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
    process::Command,
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Git status parsing, repository file discovery, and magit-style section modeling.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn configure_background_command(_command: &mut Command) {
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

/// Returns the tracked and unignored files visible from a repository root.
pub fn list_repository_files(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, RepositoryFilesError> {
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

    use super::{list_repository_files, parse_log_oneline, parse_stash_list, parse_status};

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
