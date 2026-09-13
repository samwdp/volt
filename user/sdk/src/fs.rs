use std::{
    collections::BTreeMap,
    fs, io,
    path::{Component, Path, PathBuf},
};

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
    repository_name: String,
    repository_root: PathBuf,
}

impl ProjectCandidate {
    fn new(name: String, root: PathBuf, kind: ProjectKind) -> Self {
        let (repository_name, repository_root) = project_repository_details(&root, &name, kind);
        Self {
            name,
            root,
            kind,
            repository_name,
            repository_root,
        }
    }

    /// Reconstructs a candidate from persisted fields without probing disk.
    pub fn from_persisted(
        name: impl Into<String>,
        root: impl Into<PathBuf>,
        kind: ProjectKind,
        repository_name: impl Into<String>,
        repository_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            name: name.into(),
            root: root.into(),
            kind,
            repository_name: repository_name.into(),
            repository_root: repository_root.into(),
        }
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

    /// Returns the repository name that owns this project candidate.
    pub fn repository_name(&self) -> &str {
        &self.repository_name
    }

    /// Returns a compact repository label using the repo name and its parent directory.
    pub fn repository_display_name(&self) -> String {
        compact_project_path(&self.repository_root, 2)
    }

    /// Returns the repository root that owns this project candidate.
    pub fn repository_root(&self) -> &Path {
        &self.repository_root
    }

    /// Returns the parent directory name for a worktree, when available.
    pub fn worktree_parent_name(&self) -> Option<String> {
        (self.kind == ProjectKind::GitWorktree)
            .then(|| worktree_parent_name(&self.root))
            .flatten()
    }

    /// Returns a picker-friendly display name.
    pub fn display_name(&self) -> String {
        if self.kind == ProjectKind::GitWorktree && self.repository_root != self.root {
            let project_name = self
                .worktree_parent_name()
                .unwrap_or_else(|| self.repository_display_name());
            return format!("{project_name} [{}]", self.name);
        }
        self.name.clone()
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

    /// Reads a single filesystem entry without listing its siblings.
    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?;
        let kind = if metadata.is_dir() {
            DirectoryEntryKind::Directory
        } else {
            DirectoryEntryKind::File
        };
        Ok(Self::new(name, path, kind))
    }

    /// Updates the cached name and path after a rename in a directory buffer.
    pub fn set_identity(&mut self, name: impl Into<String>, path: impl Into<PathBuf>) {
        self.name = name.into();
        self.path = path.into();
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

fn project_repository_details(path: &Path, name: &str, kind: ProjectKind) -> (String, PathBuf) {
    match kind {
        ProjectKind::Git => (name.to_owned(), path.to_path_buf()),
        ProjectKind::GitWorktree => resolve_worktree_repository_root(path)
            .ok()
            .flatten()
            .map(|repository_root| (project_name(&repository_root), repository_root))
            .unwrap_or_else(|| (name.to_owned(), path.to_path_buf())),
    }
}

fn resolve_worktree_repository_root(path: &Path) -> io::Result<Option<PathBuf>> {
    let gitdir = worktree_gitdir(path)?;
    let common_dir = worktree_common_dir(&gitdir)?;
    Ok(common_dir.parent().map(Path::to_path_buf))
}

fn worktree_gitdir(path: &Path) -> io::Result<PathBuf> {
    let marker = path.join(".git");
    let contents = fs::read_to_string(&marker)?;
    let gitdir = parse_gitdir_reference(&contents)
        .map(|reference| resolve_git_path(path, reference))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "`{}` does not contain a `gitdir:` reference",
                    marker.display()
                ),
            )
        })?;
    if !gitdir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("resolved gitdir `{}` does not exist", gitdir.display()),
        ));
    }
    Ok(gitdir)
}

fn worktree_common_dir(gitdir: &Path) -> io::Result<PathBuf> {
    let commondir_path = gitdir.join("commondir");
    let common_dir = match fs::read_to_string(&commondir_path) {
        Ok(contents) => parse_relative_git_path(gitdir, &contents)
            .unwrap_or_else(|| default_worktree_common_dir(gitdir)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            default_worktree_common_dir(gitdir)
        }
        Err(error) => return Err(error),
    };
    if !common_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "resolved common dir `{}` does not exist",
                common_dir.display()
            ),
        ));
    }
    Ok(common_dir)
}

fn default_worktree_common_dir(gitdir: &Path) -> PathBuf {
    gitdir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| gitdir.to_path_buf())
}

fn parse_gitdir_reference(contents: &str) -> Option<&str> {
    contents
        .lines()
        .find_map(|line| line.trim().strip_prefix("gitdir:").map(str::trim))
        .filter(|reference| !reference.is_empty())
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

/// Returns a compact path label using the last `component_count` path segments.
pub fn compact_project_path(path: &Path, component_count: usize) -> String {
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(segment) => Some(segment.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if components.is_empty() {
        return path.display().to_string();
    }

    let keep = component_count.max(1).min(components.len());
    let separator = std::path::MAIN_SEPARATOR.to_string();
    components[components.len() - keep..].join(&separator)
}

fn worktree_parent_name(path: &Path) -> Option<String> {
    path.parent()
        .and_then(Path::file_name)
        .map(|segment| segment.to_string_lossy().into_owned())
        .filter(|segment| !segment.is_empty())
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
