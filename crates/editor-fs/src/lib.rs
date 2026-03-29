#![doc = r#"Workspace file system scanning, project discovery, and editable directory buffers."#]

use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Workspace file system scanning and editable directory buffer helpers.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Root configuration used for project discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSearchRoot {
    root: PathBuf,
    max_depth: usize,
}

impl ProjectSearchRoot {
    /// Creates a new project discovery root.
    pub fn new(root: impl Into<PathBuf>, max_depth: usize) -> Self {
        Self {
            root: root.into(),
            max_depth,
        }
    }

    /// Returns the absolute discovery root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the maximum traversal depth below the root.
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }
}

/// Supported project types discovered on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// Standard git repository containing a `.git` directory.
    Git,
    /// Git worktree containing a `.git` file.
    GitWorktree,
}

impl ProjectKind {
    /// Returns the user-facing label for the project type.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::GitWorktree => "git worktree",
        }
    }
}

/// One discovered project candidate that can be opened as a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCandidate {
    name: String,
    root: PathBuf,
    kind: ProjectKind,
}

impl ProjectCandidate {
    fn new(name: String, root: PathBuf, kind: ProjectKind) -> Self {
        Self { name, root, kind }
    }

    /// Returns the project display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the absolute project root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the discovered project kind.
    pub const fn kind(&self) -> ProjectKind {
        self.kind
    }
}

/// Distinguishes file and directory entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryEntryKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
}

/// One entry surfaced in a directory buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    name: String,
    path: PathBuf,
    kind: DirectoryEntryKind,
}

impl DirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(
        name: impl Into<String>,
        path: impl Into<PathBuf>,
        kind: DirectoryEntryKind,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            kind,
        }
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the absolute path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the entry kind.
    pub const fn kind(&self) -> DirectoryEntryKind {
        self.kind
    }
}

/// Editable directory buffer model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryBuffer {
    root: PathBuf,
    entries: Vec<DirectoryEntry>,
}

impl DirectoryBuffer {
    /// Reads the direct children of a directory into a buffer model.
    pub fn read(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        let mut entries = Vec::new();
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let kind = if metadata.is_dir() {
                DirectoryEntryKind::Directory
            } else {
                DirectoryEntryKind::File
            };
            entries.push(DirectoryEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                path: entry.path(),
                kind,
            });
        }

        entries.sort_by_key(|entry| {
            (
                matches!(entry.kind, DirectoryEntryKind::File),
                entry.name.to_ascii_lowercase(),
            )
        });

        Ok(Self { root, entries })
    }

    /// Returns the root directory path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the visible entries.
    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }

    /// Renames an entry inside the backing directory and refreshes the listing.
    pub fn rename_entry(&mut self, old_name: &str, new_name: &str) -> io::Result<()> {
        let old_path = self.root.join(old_name);
        let new_path = self.root.join(new_name);
        fs::rename(old_path, new_path)?;
        *self = Self::read(&self.root)?;
        Ok(())
    }
}

/// Discovers git repositories and git worktrees under the configured search roots.
pub fn discover_projects(search_roots: &[ProjectSearchRoot]) -> io::Result<Vec<ProjectCandidate>> {
    let mut projects = BTreeMap::new();

    for search_root in search_roots {
        if !search_root.root().exists() {
            continue;
        }

        if let Err(error) = discover_projects_in(
            search_root.root(),
            0,
            search_root.max_depth(),
            &mut projects,
        ) {
            if is_skippable_scan_error(&error) {
                continue;
            }
            return Err(error);
        }
    }

    let mut projects = projects.into_values().collect::<Vec<_>>();
    projects.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.root.cmp(&right.root))
    });
    Ok(projects)
}

fn discover_projects_in(
    path: &Path,
    depth: usize,
    max_depth: usize,
    projects: &mut BTreeMap<PathBuf, ProjectCandidate>,
) -> io::Result<()> {
    if let Some(kind) = detect_project_kind(path)? {
        let root = path.to_path_buf();
        projects
            .entry(root.clone())
            .or_insert_with(|| ProjectCandidate::new(project_name(path), root, kind));
        return Ok(());
    }

    if depth >= max_depth {
        return Ok(());
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if is_skippable_scan_error(&error) => return Ok(()),
        Err(error) => return Err(error),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) if is_skippable_scan_error(&error) => continue,
            Err(error) => return Err(error),
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) if is_skippable_scan_error(&error) => continue,
            Err(error) => return Err(error),
        };
        if metadata.is_dir()
            && let Err(error) = discover_projects_in(&entry.path(), depth + 1, max_depth, projects)
        {
            if is_skippable_scan_error(&error) {
                continue;
            }
            return Err(error);
        }
    }

    Ok(())
}

fn detect_project_kind(path: &Path) -> io::Result<Option<ProjectKind>> {
    let git_marker = path.join(".git");
    match fs::metadata(git_marker) {
        Ok(metadata) if metadata.is_dir() => Ok(Some(ProjectKind::Git)),
        Ok(metadata) if metadata.is_file() => Ok(Some(ProjectKind::GitWorktree)),
        Ok(_) => Ok(None),
        Err(error) if is_skippable_scan_error(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

fn is_skippable_scan_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
    )
}

fn project_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        DirectoryBuffer, DirectoryEntryKind, ProjectKind, ProjectSearchRoot, discover_projects,
    };

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-editor-fs-{unique}"))
    }

    #[test]
    fn directory_buffer_reads_and_renames_entries() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir();
        fs::create_dir_all(root.join("subdir"))?;
        fs::write(root.join("alpha.txt"), "alpha")?;

        let mut buffer = DirectoryBuffer::read(&root)?;
        assert_eq!(buffer.entries().len(), 2);
        assert_eq!(buffer.entries()[0].kind(), DirectoryEntryKind::Directory);
        assert_eq!(buffer.entries()[1].name(), "alpha.txt");

        buffer.rename_entry("alpha.txt", "beta.txt")?;
        assert!(
            buffer
                .entries()
                .iter()
                .any(|entry| entry.name() == "beta.txt")
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn discover_projects_finds_git_repositories_and_worktrees()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir();
        let repo = root.join("repo");
        let worktree = root.join("trees").join("feature");
        fs::create_dir_all(repo.join(".git"))?;
        fs::create_dir_all(&worktree)?;
        fs::write(worktree.join(".git"), "gitdir: ../.git/worktrees/feature\n")?;

        let projects = discover_projects(&[ProjectSearchRoot::new(&root, 3)])?;
        assert_eq!(projects.len(), 2);
        assert!(
            projects
                .iter()
                .any(|project| project.root() == repo && project.kind() == ProjectKind::Git)
        );
        assert!(projects.iter().any(|project| {
            project.root() == worktree && project.kind() == ProjectKind::GitWorktree
        }));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
