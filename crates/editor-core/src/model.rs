use std::{
    collections::BTreeMap,
    fmt,
    path::{Path, PathBuf},
};

macro_rules! id_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            /// Returns the numeric identifier.
            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

id_type!(WindowId, "Unique identifier for a window.");
id_type!(WorkspaceId, "Unique identifier for a workspace.");
id_type!(PaneId, "Unique identifier for a pane.");
id_type!(PopupId, "Unique identifier for a popup.");
id_type!(BufferId, "Unique identifier for a buffer.");

/// Identifies the primary role of a buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferKind {
    /// Editable file-backed content.
    File,
    /// Native image viewing content.
    Image,
    /// Internal scratch content not backed by a file.
    Scratch,
    /// Generic list or picker content.
    Picker,
    /// Terminal-backed content.
    Terminal,
    /// Git-oriented workflows.
    Git,
    /// Directory browsing and manipulation.
    Directory,
    /// Compilation results and command output.
    Compilation,
    /// Diagnostics, messages, and structured status buffers.
    Diagnostics,
    /// Extension-defined buffer categories.
    Plugin(String),
}

/// Describes a logical editor buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    id: BufferId,
    name: String,
    kind: BufferKind,
    path: Option<PathBuf>,
    dirty: bool,
}

impl Buffer {
    fn new(id: BufferId, name: String, kind: BufferKind, path: Option<PathBuf>) -> Self {
        Self {
            id,
            name,
            kind,
            path,
            dirty: false,
        }
    }

    /// Returns the buffer identifier.
    pub const fn id(&self) -> BufferId {
        self.id
    }

    /// Returns the display name of the buffer.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Updates the display name of the buffer.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Returns the buffer kind.
    pub const fn kind(&self) -> &BufferKind {
        &self.kind
    }

    /// Returns the file system path, when the buffer is file-backed.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Reports whether the buffer contains unsaved changes.
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the buffer as dirty.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Marks the buffer as clean.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

/// Visible container for one or more buffers in the workspace layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    id: PaneId,
    buffers: Vec<BufferId>,
    active_buffer: Option<BufferId>,
}

impl Pane {
    fn new(id: PaneId) -> Self {
        Self {
            id,
            buffers: Vec::new(),
            active_buffer: None,
        }
    }

    fn add_buffer(&mut self, buffer_id: BufferId) {
        if !self.buffers.contains(&buffer_id) {
            self.buffers.push(buffer_id);
        }
        self.active_buffer = Some(buffer_id);
    }

    fn remove_buffer(&mut self, buffer_id: BufferId) -> bool {
        let Some(index) = self.buffers.iter().position(|id| *id == buffer_id) else {
            return false;
        };
        self.buffers.remove(index);
        if self.active_buffer == Some(buffer_id) {
            let next = if self.buffers.is_empty() {
                None
            } else if index > 0 {
                self.buffers.get(index - 1).copied()
            } else {
                self.buffers.first().copied()
            };
            self.active_buffer = next;
        }
        true
    }

    /// Returns the pane identifier.
    pub const fn id(&self) -> PaneId {
        self.id
    }

    /// Returns the buffers currently visible or accessible from this pane.
    pub fn buffer_ids(&self) -> &[BufferId] {
        &self.buffers
    }

    /// Returns the active buffer for this pane, if one is selected.
    pub const fn active_buffer(&self) -> Option<BufferId> {
        self.active_buffer
    }
}

/// Transient container capable of surfacing multiple buffers at once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Popup {
    id: PopupId,
    title: String,
    buffers: Vec<BufferId>,
    active_buffer: BufferId,
}

impl Popup {
    fn new(id: PopupId, title: String, buffers: Vec<BufferId>, active_buffer: BufferId) -> Self {
        Self {
            id,
            title,
            buffers,
            active_buffer,
        }
    }

    /// Returns the popup identifier.
    pub const fn id(&self) -> PopupId {
        self.id
    }

    /// Returns the popup title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the buffers hosted by the popup.
    pub fn buffer_ids(&self) -> &[BufferId] {
        &self.buffers
    }

    /// Returns the active buffer for the popup.
    pub const fn active_buffer(&self) -> BufferId {
        self.active_buffer
    }

    fn contains_buffer(&self, buffer_id: BufferId) -> bool {
        self.buffers.contains(&buffer_id)
    }

    fn add_buffer(&mut self, buffer_id: BufferId) -> bool {
        if self.contains_buffer(buffer_id) {
            return false;
        }
        self.buffers.push(buffer_id);
        true
    }

    fn set_active_buffer(&mut self, buffer_id: BufferId) -> bool {
        if !self.contains_buffer(buffer_id) {
            return false;
        }
        self.active_buffer = buffer_id;
        true
    }

    fn cycle_buffer(&mut self, forward: bool) -> Option<BufferId> {
        let len = self.buffers.len();
        if len == 0 {
            return None;
        }
        let current = self
            .buffers
            .iter()
            .position(|id| *id == self.active_buffer)
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % len
        } else {
            (current + len - 1) % len
        };
        self.active_buffer = self.buffers[next];
        Some(self.active_buffer)
    }

    fn remove_buffer(&mut self, buffer_id: BufferId) -> bool {
        let Some(index) = self.buffers.iter().position(|id| *id == buffer_id) else {
            return false;
        };
        self.buffers.remove(index);
        if self.buffers.is_empty() {
            return true;
        }
        if self.active_buffer == buffer_id {
            let next = if index > 0 {
                self.buffers.get(index - 1).copied()
            } else {
                self.buffers.first().copied()
            };
            if let Some(next) = next {
                self.active_buffer = next;
            }
        }
        false
    }
}

/// Window-scoped collection of panes, popups, and buffers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    id: WorkspaceId,
    name: String,
    root: Option<PathBuf>,
    buffers: BTreeMap<BufferId, Buffer>,
    panes: BTreeMap<PaneId, Pane>,
    popups: BTreeMap<PopupId, Popup>,
    active_pane: Option<PaneId>,
}

impl Workspace {
    fn new(id: WorkspaceId, name: String, root: Option<PathBuf>, initial_pane: Pane) -> Self {
        let initial_pane_id = initial_pane.id();
        let mut panes = BTreeMap::new();
        panes.insert(initial_pane_id, initial_pane);

        Self {
            id,
            name,
            root,
            buffers: BTreeMap::new(),
            panes,
            popups: BTreeMap::new(),
            active_pane: Some(initial_pane_id),
        }
    }

    fn active_pane_mut(&mut self) -> Option<&mut Pane> {
        let active_pane_id = self.active_pane?;
        self.panes.get_mut(&active_pane_id)
    }

    /// Returns the workspace identifier.
    pub const fn id(&self) -> WorkspaceId {
        self.id
    }

    /// Returns the workspace display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the workspace root path, if one is configured.
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Returns the active pane identifier, if one is selected.
    pub const fn active_pane_id(&self) -> Option<PaneId> {
        self.active_pane
    }

    /// Returns the active pane.
    pub fn active_pane(&self) -> Option<&Pane> {
        self.active_pane
            .and_then(|pane_id| self.panes.get(&pane_id))
    }

    /// Returns the number of buffers registered with the workspace.
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    /// Returns the number of panes registered with the workspace.
    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }

    /// Returns the number of popups currently open in the workspace.
    pub fn popup_count(&self) -> usize {
        self.popups.len()
    }

    /// Returns a buffer by identifier.
    pub fn buffer(&self, buffer_id: BufferId) -> Option<&Buffer> {
        self.buffers.get(&buffer_id)
    }

    /// Returns the buffers currently registered with the workspace.
    pub fn buffers(&self) -> impl Iterator<Item = &Buffer> {
        self.buffers.values()
    }

    /// Returns a pane by identifier.
    pub fn pane(&self, pane_id: PaneId) -> Option<&Pane> {
        self.panes.get(&pane_id)
    }

    /// Returns a popup by identifier.
    pub fn popup(&self, popup_id: PopupId) -> Option<&Popup> {
        self.popups.get(&popup_id)
    }

    /// Returns the popups currently registered with the workspace.
    pub fn popups(&self) -> impl Iterator<Item = &Popup> {
        self.popups.values()
    }
}

/// Top-level container for workspaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    id: WindowId,
    title: String,
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    active_workspace: Option<WorkspaceId>,
}

impl Window {
    fn new(id: WindowId, title: String) -> Self {
        Self {
            id,
            title,
            workspaces: BTreeMap::new(),
            active_workspace: None,
        }
    }

    /// Returns the window identifier.
    pub const fn id(&self) -> WindowId {
        self.id
    }

    /// Returns the window title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the active workspace identifier, if one is selected.
    pub const fn active_workspace_id(&self) -> Option<WorkspaceId> {
        self.active_workspace
    }

    /// Returns the number of workspaces attached to the window.
    pub fn workspace_count(&self) -> usize {
        self.workspaces.len()
    }

    /// Returns the workspaces currently attached to the window.
    pub fn workspaces(&self) -> impl Iterator<Item = &Workspace> {
        self.workspaces.values()
    }
}

/// Mutable editor state describing the active UI graph.
#[derive(Debug, Default)]
pub struct EditorModel {
    windows: BTreeMap<WindowId, Window>,
    active_window: Option<WindowId>,
    next_window_id: u64,
    next_workspace_id: u64,
    next_pane_id: u64,
    next_popup_id: u64,
    next_buffer_id: u64,
}

impl EditorModel {
    /// Creates an empty editor model with no windows.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of windows tracked by the model.
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Returns the active window identifier, if one is selected.
    pub const fn active_window_id(&self) -> Option<WindowId> {
        self.active_window
    }

    /// Creates a new window and marks it as active.
    pub fn create_window(&mut self, title: impl Into<String>) -> WindowId {
        let window_id = self.next_window_id();
        let window = Window::new(window_id, title.into());

        self.windows.insert(window_id, window);
        self.active_window = Some(window_id);

        window_id
    }

    /// Opens a workspace under the specified window and seeds it with an initial pane.
    pub fn open_workspace(
        &mut self,
        window_id: WindowId,
        name: impl Into<String>,
        root: Option<PathBuf>,
    ) -> Result<WorkspaceId, ModelError> {
        let workspace_id = self.next_workspace_id();
        let pane_id = self.next_pane_id();
        let window = self
            .windows
            .get_mut(&window_id)
            .ok_or(ModelError::WindowNotFound(window_id))?;
        let workspace = Workspace::new(workspace_id, name.into(), root, Pane::new(pane_id));

        window.workspaces.insert(workspace_id, workspace);
        window.active_workspace = Some(workspace_id);
        self.active_window = Some(window_id);

        Ok(workspace_id)
    }

    /// Switches the active window to the window containing the workspace and focuses it.
    pub fn switch_workspace(&mut self, workspace_id: WorkspaceId) -> Result<(), ModelError> {
        let window_id = self
            .windows
            .iter()
            .find_map(|(window_id, window)| {
                window
                    .workspaces
                    .contains_key(&workspace_id)
                    .then_some(*window_id)
            })
            .ok_or(ModelError::WorkspaceNotFound(workspace_id))?;
        let window = self
            .windows
            .get_mut(&window_id)
            .ok_or(ModelError::WindowNotFound(window_id))?;
        window.active_workspace = Some(workspace_id);
        self.active_window = Some(window_id);
        Ok(())
    }

    /// Closes and removes a workspace from its window, returning the removed workspace.
    pub fn close_workspace(&mut self, workspace_id: WorkspaceId) -> Result<Workspace, ModelError> {
        let (window_id, was_active) = self
            .windows
            .iter()
            .find_map(|(window_id, window)| {
                window
                    .workspaces
                    .contains_key(&workspace_id)
                    .then_some((*window_id, window.active_workspace == Some(workspace_id)))
            })
            .ok_or(ModelError::WorkspaceNotFound(workspace_id))?;
        let window = self
            .windows
            .get_mut(&window_id)
            .ok_or(ModelError::WindowNotFound(window_id))?;
        let workspace = window
            .workspaces
            .remove(&workspace_id)
            .ok_or(ModelError::WorkspaceNotFound(workspace_id))?;

        if was_active {
            window.active_workspace = window.workspaces.keys().next().copied();
        }

        Ok(workspace)
    }

    /// Creates a buffer in the specified workspace and attaches it to the active pane.
    pub fn create_buffer(
        &mut self,
        workspace_id: WorkspaceId,
        name: impl Into<String>,
        kind: BufferKind,
        path: Option<PathBuf>,
    ) -> Result<BufferId, ModelError> {
        let buffer_id = self.next_buffer_id();
        let workspace = self.workspace_mut(workspace_id)?;
        let buffer = Buffer::new(buffer_id, name.into(), kind, path);

        workspace.buffers.insert(buffer_id, buffer);
        workspace
            .active_pane_mut()
            .ok_or(ModelError::NoActivePane(workspace_id))?
            .add_buffer(buffer_id);

        Ok(buffer_id)
    }

    /// Creates a buffer in the specified workspace without attaching it to a pane.
    pub fn create_popup_buffer(
        &mut self,
        workspace_id: WorkspaceId,
        name: impl Into<String>,
        kind: BufferKind,
        path: Option<PathBuf>,
    ) -> Result<BufferId, ModelError> {
        let buffer_id = self.next_buffer_id();
        let workspace = self.workspace_mut(workspace_id)?;
        let buffer = Buffer::new(buffer_id, name.into(), kind, path);
        workspace.buffers.insert(buffer_id, buffer);
        Ok(buffer_id)
    }

    /// Focuses an existing buffer in the workspace's active pane.
    pub fn focus_buffer(
        &mut self,
        workspace_id: WorkspaceId,
        buffer_id: BufferId,
    ) -> Result<(), ModelError> {
        let workspace = self.workspace_mut(workspace_id)?;
        if !workspace.buffers.contains_key(&buffer_id) {
            return Err(ModelError::BufferNotFound(buffer_id));
        }

        workspace
            .active_pane_mut()
            .ok_or(ModelError::NoActivePane(workspace_id))?
            .add_buffer(buffer_id);

        Ok(())
    }

    /// Updates the display name of a buffer.
    pub fn set_buffer_name(
        &mut self,
        workspace_id: WorkspaceId,
        buffer_id: BufferId,
        name: String,
    ) -> Result<(), ModelError> {
        let workspace = self.workspace_mut(workspace_id)?;
        let buffer = workspace
            .buffers
            .get_mut(&buffer_id)
            .ok_or(ModelError::BufferNotFound(buffer_id))?;
        buffer.set_name(name);
        Ok(())
    }

    /// Creates a new pane in the workspace containing the provided buffer.
    pub fn split_pane(
        &mut self,
        workspace_id: WorkspaceId,
        buffer_id: BufferId,
    ) -> Result<PaneId, ModelError> {
        let pane_id = self.next_pane_id();
        let workspace = self.workspace_mut(workspace_id)?;
        if !workspace.buffers.contains_key(&buffer_id) {
            return Err(ModelError::BufferNotFound(buffer_id));
        }
        let mut pane = Pane::new(pane_id);
        pane.add_buffer(buffer_id);
        workspace.panes.insert(pane_id, pane);
        Ok(pane_id)
    }

    /// Sets the active pane for the workspace.
    pub fn focus_pane(
        &mut self,
        workspace_id: WorkspaceId,
        pane_id: PaneId,
    ) -> Result<(), ModelError> {
        let workspace = self.workspace_mut(workspace_id)?;
        if !workspace.panes.contains_key(&pane_id) {
            return Err(ModelError::PaneNotFound(pane_id));
        }
        workspace.active_pane = Some(pane_id);
        Ok(())
    }

    /// Closes a pane in the specified workspace.
    pub fn close_pane(
        &mut self,
        workspace_id: WorkspaceId,
        pane_id: PaneId,
    ) -> Result<(), ModelError> {
        let workspace = self.workspace_mut(workspace_id)?;
        if !workspace.panes.contains_key(&pane_id) {
            return Err(ModelError::PaneNotFound(pane_id));
        }
        if workspace.panes.len() <= 1 {
            return Err(ModelError::CannotCloseLastPane(workspace_id));
        }
        let next_active_pane =
            if workspace.active_pane == Some(pane_id) || workspace.active_pane.is_none() {
                workspace
                    .panes
                    .keys()
                    .copied()
                    .find(|candidate| *candidate != pane_id)
            } else {
                workspace.active_pane
            };
        workspace.panes.remove(&pane_id);
        workspace.active_pane = next_active_pane;
        Ok(())
    }

    /// Closes a buffer in the specified workspace.
    pub fn close_buffer(
        &mut self,
        workspace_id: WorkspaceId,
        buffer_id: BufferId,
    ) -> Result<(), ModelError> {
        let workspace = self.workspace_mut(workspace_id)?;
        if !workspace.buffers.contains_key(&buffer_id) {
            return Err(ModelError::BufferNotFound(buffer_id));
        }
        if workspace.buffers.len() <= 1 {
            return Err(ModelError::CannotCloseLastBuffer(workspace_id));
        }
        workspace.buffers.remove(&buffer_id);

        for pane in workspace.panes.values_mut() {
            pane.remove_buffer(buffer_id);
        }

        let mut popups_to_close = Vec::new();
        for (popup_id, popup) in workspace.popups.iter_mut() {
            if popup.remove_buffer(buffer_id) {
                popups_to_close.push(*popup_id);
            }
        }
        for popup_id in popups_to_close {
            workspace.popups.remove(&popup_id);
        }

        if let Some(fallback) = workspace.buffers.keys().next().copied() {
            for pane in workspace.panes.values_mut() {
                if pane.active_buffer.is_none() {
                    pane.active_buffer = Some(fallback);
                    if !pane.buffers.contains(&fallback) {
                        pane.buffers.push(fallback);
                    }
                }
            }
        }

        Ok(())
    }

    /// Opens a popup in the specified workspace with multiple buffers.
    pub fn open_popup(
        &mut self,
        workspace_id: WorkspaceId,
        title: impl Into<String>,
        buffers: Vec<BufferId>,
        active_buffer: BufferId,
    ) -> Result<PopupId, ModelError> {
        if buffers.is_empty() {
            return Err(ModelError::PopupRequiresBuffers);
        }

        if !buffers.contains(&active_buffer) {
            return Err(ModelError::PopupActiveBufferNotPresent(active_buffer));
        }

        let popup_id = self.next_popup_id();
        let workspace = self.workspace_mut(workspace_id)?;

        for buffer_id in &buffers {
            if !workspace.buffers.contains_key(buffer_id) {
                return Err(ModelError::BufferNotFound(*buffer_id));
            }
        }

        let popup = Popup::new(popup_id, title.into(), buffers, active_buffer);
        workspace.popups.insert(popup_id, popup);

        Ok(popup_id)
    }

    /// Opens a popup buffer, creating a popup if needed or appending to the existing popup.
    pub fn open_popup_buffer(
        &mut self,
        workspace_id: WorkspaceId,
        title: impl Into<String>,
        buffer_id: BufferId,
    ) -> Result<PopupId, ModelError> {
        let popup_id = {
            let workspace = self.workspace(workspace_id)?;
            if !workspace.buffers.contains_key(&buffer_id) {
                return Err(ModelError::BufferNotFound(buffer_id));
            }
            workspace.popups.keys().next().copied()
        };

        if let Some(popup_id) = popup_id {
            let workspace = self.workspace_mut(workspace_id)?;
            let popup = workspace
                .popups
                .get_mut(&popup_id)
                .ok_or(ModelError::PopupNotFound(popup_id))?;
            popup.add_buffer(buffer_id);
            popup.set_active_buffer(buffer_id);
            return Ok(popup_id);
        }

        self.open_popup(workspace_id, title, vec![buffer_id], buffer_id)
    }

    /// Cycles to the next or previous popup buffer, returning the new active buffer.
    pub fn cycle_popup_buffer(
        &mut self,
        workspace_id: WorkspaceId,
        forward: bool,
    ) -> Result<Option<BufferId>, ModelError> {
        let popup_id = self.workspace(workspace_id)?.popups.keys().next().copied();
        let Some(popup_id) = popup_id else {
            return Ok(None);
        };
        let workspace = self.workspace_mut(workspace_id)?;
        let popup = workspace
            .popups
            .get_mut(&popup_id)
            .ok_or(ModelError::PopupNotFound(popup_id))?;
        Ok(popup.cycle_buffer(forward))
    }

    /// Closes a popup in the specified workspace.
    pub fn close_popup(
        &mut self,
        workspace_id: WorkspaceId,
        popup_id: PopupId,
    ) -> Result<(), ModelError> {
        let workspace = self.workspace_mut(workspace_id)?;
        workspace
            .popups
            .remove(&popup_id)
            .ok_or(ModelError::PopupNotFound(popup_id))?;
        Ok(())
    }

    /// Returns a window by identifier.
    pub fn window(&self, window_id: WindowId) -> Result<&Window, ModelError> {
        self.windows
            .get(&window_id)
            .ok_or(ModelError::WindowNotFound(window_id))
    }

    /// Returns the active window.
    pub fn active_window(&self) -> Result<&Window, ModelError> {
        let active_window_id = self.active_window.ok_or(ModelError::NoActiveWindow)?;
        self.window(active_window_id)
    }

    /// Returns the active workspace identifier for the active window.
    pub fn active_workspace_id(&self) -> Result<WorkspaceId, ModelError> {
        let window = self.active_window()?;
        window
            .active_workspace_id()
            .ok_or(ModelError::NoActiveWorkspace(window.id()))
    }

    /// Returns the active workspace for the active window.
    pub fn active_workspace(&self) -> Result<&Workspace, ModelError> {
        let workspace_id = self.active_workspace_id()?;
        self.workspace(workspace_id)
    }

    /// Returns a workspace by identifier.
    pub fn workspace(&self, workspace_id: WorkspaceId) -> Result<&Workspace, ModelError> {
        self.windows
            .values()
            .find_map(|window| window.workspaces.get(&workspace_id))
            .ok_or(ModelError::WorkspaceNotFound(workspace_id))
    }

    fn workspace_mut(&mut self, workspace_id: WorkspaceId) -> Result<&mut Workspace, ModelError> {
        self.windows
            .values_mut()
            .find_map(|window| window.workspaces.get_mut(&workspace_id))
            .ok_or(ModelError::WorkspaceNotFound(workspace_id))
    }

    fn next_window_id(&mut self) -> WindowId {
        self.next_window_id += 1;
        WindowId(self.next_window_id)
    }

    fn next_workspace_id(&mut self) -> WorkspaceId {
        self.next_workspace_id += 1;
        WorkspaceId(self.next_workspace_id)
    }

    fn next_pane_id(&mut self) -> PaneId {
        self.next_pane_id += 1;
        PaneId(self.next_pane_id)
    }

    fn next_popup_id(&mut self) -> PopupId {
        self.next_popup_id += 1;
        PopupId(self.next_popup_id)
    }

    fn next_buffer_id(&mut self) -> BufferId {
        self.next_buffer_id += 1;
        BufferId(self.next_buffer_id)
    }
}

/// Errors raised by model operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    /// No active window is currently selected.
    NoActiveWindow,
    /// The active window has no active workspace selected.
    NoActiveWorkspace(WindowId),
    /// The requested window identifier does not exist.
    WindowNotFound(WindowId),
    /// The requested workspace identifier does not exist.
    WorkspaceNotFound(WorkspaceId),
    /// The requested buffer identifier does not exist.
    BufferNotFound(BufferId),
    /// The requested pane identifier does not exist.
    PaneNotFound(PaneId),
    /// The requested popup identifier does not exist.
    PopupNotFound(PopupId),
    /// The workspace has no active pane available.
    NoActivePane(WorkspaceId),
    /// The workspace must keep at least one pane.
    CannotCloseLastPane(WorkspaceId),
    /// The workspace must keep at least one buffer.
    CannotCloseLastBuffer(WorkspaceId),
    /// Popups must be created with at least one buffer.
    PopupRequiresBuffers,
    /// The popup active buffer must be included in the popup buffer list.
    PopupActiveBufferNotPresent(BufferId),
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveWindow => formatter.write_str("no active window is selected"),
            Self::NoActiveWorkspace(window_id) => {
                write!(formatter, "window {window_id} has no active workspace")
            }
            Self::WindowNotFound(window_id) => {
                write!(formatter, "window {window_id} was not found")
            }
            Self::WorkspaceNotFound(workspace_id) => {
                write!(formatter, "workspace {workspace_id} was not found")
            }
            Self::BufferNotFound(buffer_id) => {
                write!(formatter, "buffer {buffer_id} was not found")
            }
            Self::PaneNotFound(pane_id) => {
                write!(formatter, "pane {pane_id} was not found")
            }
            Self::PopupNotFound(popup_id) => {
                write!(formatter, "popup {popup_id} was not found")
            }
            Self::NoActivePane(workspace_id) => {
                write!(formatter, "workspace {workspace_id} has no active pane")
            }
            Self::CannotCloseLastPane(workspace_id) => write!(
                formatter,
                "workspace {workspace_id} must retain at least one pane"
            ),
            Self::CannotCloseLastBuffer(workspace_id) => write!(
                formatter,
                "workspace {workspace_id} must retain at least one buffer"
            ),
            Self::PopupRequiresBuffers => {
                formatter.write_str("a popup requires at least one buffer")
            }
            Self::PopupActiveBufferNotPresent(buffer_id) => write!(
                formatter,
                "popup active buffer {buffer_id} must be included in the popup buffer list"
            ),
        }
    }
}

impl std::error::Error for ModelError {}
