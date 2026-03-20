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

/// One file entry in a git status listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    path: String,
    index_status: char,
    worktree_status: char,
}

impl StatusEntry {
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
    let output = Command::new("git")
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

    use super::{list_repository_files, parse_status};

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
}
