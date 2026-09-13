#![doc = r#"Workspace file system scanning, project discovery, and editable directory buffers."#]

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

pub use editor_plugin_api::{
    DirectoryEntry, DirectoryEntryKind, PROJECT_DISCOVERY_TTL, ProjectCandidate,
    ProjectDiscoveryFingerprint, ProjectDiscoverySnapshot, ProjectKind, ProjectSearchRoot,
    cancel_project_discovery_scan, compact_project_path, current_project_discovery_snapshot,
    discover_projects, invalidate_project_discovery_cache, project_discovery_background_tick,
    project_discovery_for_picker, project_discovery_forget_candidate,
    project_discovery_persist_path, project_discovery_request_scan,
    project_discovery_rescan_cached_roots, project_discovery_snapshot,
    reset_project_discovery_cache, set_project_discovery_persist_path_for_test,
    set_project_discovery_ttl_for_test, set_project_discovery_worker_blocked_for_test,
    wait_for_project_discovery,
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Workspace file system scanning and editable directory buffer helpers.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
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
            entries.push(DirectoryEntry::new(
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
                kind,
            ));
        }

        sort_directory_buffer_entries(&mut entries);

        Ok(Self { root, entries })
    }

    /// Builds a buffer from an already-loaded listing without touching the disk.
    pub fn from_entries(root: impl Into<PathBuf>, entries: Vec<DirectoryEntry>) -> Self {
        Self {
            root: root.into(),
            entries,
        }
    }

    /// Returns the root directory path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the visible entries.
    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }

    /// Consumes the buffer and returns the cached entries.
    pub fn into_entries(self) -> Vec<DirectoryEntry> {
        self.entries
    }

    /// Renames an entry inside the backing directory and patches that row.
    pub fn rename_entry(&mut self, old_name: &str, new_name: &str) -> io::Result<()> {
        let old_path = self.root.join(old_name);
        let new_path = self.root.join(new_name);
        fs::rename(&old_path, &new_path)?;
        if !self.patch_renamed(&old_path, &new_path) {
            *self = Self::read(&self.root)?;
        }
        Ok(())
    }

    /// Creates a file in the backing directory and patches the listing.
    pub fn create_file(&mut self, name: &str) -> io::Result<()> {
        let path = self.root.join(name);
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        self.patch_created(&path)?;
        Ok(())
    }

    /// Creates a directory in the backing directory and patches the listing.
    pub fn create_dir(&mut self, name: &str) -> io::Result<()> {
        let path = self.root.join(name);
        fs::create_dir(&path)?;
        self.patch_created(&path)?;
        Ok(())
    }

    /// Deletes an entry from the backing directory and patches the listing.
    pub fn delete_entry(&mut self, name: &str) -> io::Result<()> {
        let path = self.root.join(name);
        let kind = self
            .entries
            .iter()
            .find(|entry| entry.name() == name)
            .map(DirectoryEntry::kind);
        match kind {
            Some(DirectoryEntryKind::Directory) => fs::remove_dir_all(&path)?,
            Some(DirectoryEntryKind::File) => fs::remove_file(&path)?,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("directory entry `{name}` is missing"),
                ));
            }
        }
        self.patch_deleted(&path);
        Ok(())
    }

    /// Patches a renamed direct child after a successful on-disk rename.
    pub fn patch_renamed(&mut self, from: &Path, to: &Path) -> bool {
        if !is_direct_child(&self.root, from) || !is_direct_child(&self.root, to) {
            return false;
        }
        let Some(name) = to
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            return false;
        };
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.path() == from) else {
            return false;
        };
        entry.set_identity(name, to.to_path_buf());
        sort_directory_buffer_entries(&mut self.entries);
        true
    }

    /// Inserts the listing child for `path` by statting that one path.
    pub fn patch_created(&mut self, path: &Path) -> io::Result<bool> {
        let Some(child) = first_child_path(&self.root, path) else {
            return Ok(false);
        };
        if self.entries.iter().any(|entry| entry.path() == child) {
            return Ok(true);
        }
        self.entries.push(DirectoryEntry::from_path(&child)?);
        sort_directory_buffer_entries(&mut self.entries);
        Ok(true)
    }

    /// Removes a direct child from the cached listing.
    pub fn patch_deleted(&mut self, path: &Path) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.path() != path);
        before != self.entries.len() || !is_direct_child(&self.root, path)
    }
}

fn sort_directory_buffer_entries(entries: &mut [DirectoryEntry]) {
    entries.sort_by_key(|entry| {
        (
            matches!(entry.kind(), DirectoryEntryKind::File),
            entry.name().to_ascii_lowercase(),
        )
    });
}

fn is_direct_child(root: &Path, path: &Path) -> bool {
    path.parent() == Some(root)
}

fn first_child_path(root: &Path, path: &Path) -> Option<PathBuf> {
    let relative = path.strip_prefix(root).ok()?;
    let first = relative.components().next()?;
    match first {
        Component::Normal(name) => Some(root.join(name)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
