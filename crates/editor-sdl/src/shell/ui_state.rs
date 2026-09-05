#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneSplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowMoveDirection {
    Left,
    Down,
    Up,
    Right,
}

#[derive(Debug, Clone, Copy)]
enum GitBranchActionKind {
    Checkout,
    MergePlain,
    MergeEdit,
    MergeNoCommit,
    MergeSquash,
    MergePreview,
    RebaseOnto,
    RebaseInteractive,
}

#[derive(Debug, Clone, Copy)]
enum GitCommitActionKind {
    CherryPick,
    CherryPickNoCommit,
    Revert,
    RevertNoCommit,
    ResetMixed,
    ResetSoft,
    ResetHard,
    ResetKeep,
}

#[derive(Debug, Clone, Copy)]
enum GitSequenceKind {
    CherryPick,
    Revert,
}

#[derive(Debug, Clone, Copy)]
enum GitResetMode {
    Mixed,
    Soft,
    Hard,
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BufferViewState {
    cursor: TextPoint,
    scroll_row: usize,
    scroll_col: usize,
}

#[derive(Debug, Clone)]
struct ShellPane {
    pane_id: PaneId,
    buffer_id: BufferId,
    buffer_views: BTreeMap<BufferId, BufferViewState>,
}

impl ShellPane {
    fn new(pane_id: PaneId, buffer_id: BufferId, view_state: BufferViewState) -> Self {
        let mut buffer_views = BTreeMap::new();
        buffer_views.insert(buffer_id, view_state);
        Self {
            pane_id,
            buffer_id,
            buffer_views,
        }
    }

    fn view_state(&self, buffer_id: BufferId) -> Option<BufferViewState> {
        self.buffer_views.get(&buffer_id).copied()
    }

    fn store_view_state(&mut self, buffer_id: BufferId, view_state: BufferViewState) {
        self.buffer_views.insert(buffer_id, view_state);
    }

    fn remove_view_state(&mut self, buffer_id: BufferId) {
        self.buffer_views.remove(&buffer_id);
    }
}

#[derive(Debug, Clone)]
enum PickerAction {
    NoOp,
    ExecuteCommand(String),
    ExecuteCommands(Vec<String>),
    ApplyLspCodeAction {
        workspace_id: WorkspaceId,
        buffer_id: BufferId,
        path: PathBuf,
        code_action: LspCodeAction,
    },
    FocusBuffer(BufferId),
    CloseBuffer(BufferId),
    CloseBufferSave(BufferId),
    CloseBufferDiscard(BufferId),
    OpenFile(PathBuf),
    OpenFileLocation {
        path: PathBuf,
        target: TextPoint,
    },
    OpenLspLocation {
        location: LspLocation,
    },
    OpenAcpClient(String),
    CreateWorkspaceFile {
        root: PathBuf,
    },
    ActivateTheme(String),
    EmitHook {
        hook: String,
        detail: Option<String>,
    },
    UndoTreeNode {
        buffer_id: BufferId,
        node_id: usize,
    },
    VimSearch(VimSearchDirection),
    VimSearchResult {
        direction: VimSearchDirection,
        target: TextPoint,
    },
    InstallTreeSitterLanguage(String),
    InstallLanguageServer(String),
    InstallDebugAdapter(String),
    CreateWorkspace {
        name: String,
        root: PathBuf,
    },
    SwitchWorkspace(WorkspaceId),
    DeleteWorkspace(WorkspaceId),
    GitPushRemote(String),
    GitFetchRemote(String),
    GitWorktreeBranch {
        remote_branch: String,
        local_branch: String,
    },
    GitWorktreeCreate {
        remote_branch: String,
        local_branch: String,
        base_dir: PathBuf,
    },
    GitWorktreeOilBranch {
        buffer_id: BufferId,
        remote_branch: String,
        local_branch: String,
    },
    GitWorktreeOilNewBranch {
        buffer_id: BufferId,
    },
    GitWorktreeDashboardCreate {
        base_dir: PathBuf,
    },
    GitBranchAction {
        action: GitBranchActionKind,
        branch: String,
    },
    GitCommitAction {
        action: GitCommitActionKind,
        commit: String,
    },
    AcpInsertSlashCommand {
        buffer_id: BufferId,
        command: String,
    },
    AcpInsertFileMention {
        buffer_id: BufferId,
        relative_path: String,
    },
    AcpLoadSession {
        buffer_id: BufferId,
        session_id: String,
        session_title: String,
    },
    AcpSetMode {
        buffer_id: BufferId,
        mode_id: String,
    },
    AcpSetModel {
        buffer_id: BufferId,
        model_id: String,
    },
    AcpResolvePermission {
        request_id: u64,
        option_id: String,
    },
    CopyToClipboard(String),
    StopLspSession {
        server_id: String,
        root: Option<PathBuf>,
    },
    RestartLspSession {
        server_id: String,
        root: Option<PathBuf>,
    },
    StartDapSession {
        adapter_id: String,
        configuration: DebugConfiguration,
        ask_heuristic_compile: bool,
    },
    ConfirmDapCompile {
        adapter_id: String,
        configuration: DebugConfiguration,
        command: String,
    },
    SkipDapCompile {
        adapter_id: String,
        configuration: DebugConfiguration,
    },
    RemoveDapExpression {
        expression: String,
    },
    SwitchDapThread {
        thread_id: u64,
    },
    SwitchDapStackFrame {
        frame_id: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickerKind {
    Generic,
    AcpSlash { buffer_id: BufferId },
    AcpFile { buffer_id: BufferId },
    AcpPermission { request_id: u64 },
}

impl PickerKind {
    fn acp_inline_buffer_id(self) -> Option<BufferId> {
        match self {
            PickerKind::AcpSlash { buffer_id } | PickerKind::AcpFile { buffer_id } => {
                Some(buffer_id)
            }
            PickerKind::Generic | PickerKind::AcpPermission { .. } => None,
        }
    }

    fn is_acp_inline(self) -> bool {
        self.acp_inline_buffer_id().is_some()
    }
}

#[derive(Debug, Clone)]
struct PickerEntry {
    item: PickerItem,
    action: PickerAction,
    quickfix: Option<QuickfixEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuickfixEntry {
    id: String,
    path: PathBuf,
    target: TextPoint,
    label: String,
}

impl QuickfixEntry {
    fn new(
        id: impl Into<String>,
        path: PathBuf,
        target: TextPoint,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            path,
            target,
            label: label.into(),
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn target(&self) -> TextPoint {
        self.target
    }

    fn render_line(&self, marked: bool) -> String {
        let mark = if marked { "x" } else { " " };
        format!(
            "[{mark}] {}:{}:{} | {}",
            self.path.display(),
            self.target.line + 1,
            self.target.column + 1,
            self.label
        )
    }

    fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Debug, Clone, Default)]
struct QuickfixState {
    entries: Vec<QuickfixEntry>,
    selected_index: usize,
    marked_entry_ids: BTreeSet<String>,
    buffer_id: Option<BufferId>,
}

impl QuickfixState {
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn buffer_id(&self) -> Option<BufferId> {
        self.buffer_id
    }

    fn set_buffer_id(&mut self, buffer_id: BufferId) {
        self.buffer_id = Some(buffer_id);
    }

    fn set_entries(&mut self, entries: Vec<QuickfixEntry>) {
        self.entries = entries;
        self.selected_index = 0;
        self.marked_entry_ids.clear();
    }

    fn entries(&self) -> &[QuickfixEntry] {
        &self.entries
    }

    fn selected_index(&self) -> usize {
        self.selected_index
            .min(self.entries.len().saturating_sub(1))
    }

    fn set_selected_index(&mut self, index: usize) {
        if self.entries.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = index.min(self.entries.len() - 1);
        }
    }

    fn selected_entry(&self) -> Option<QuickfixEntry> {
        self.entries.get(self.selected_index()).cloned()
    }

    fn select_next(&mut self) -> Option<QuickfixEntry> {
        if self.entries.is_empty() {
            self.selected_index = 0;
            return None;
        }
        self.selected_index = (self.selected_index + 1) % self.entries.len();
        self.selected_entry()
    }

    fn select_previous(&mut self) -> Option<QuickfixEntry> {
        if self.entries.is_empty() {
            self.selected_index = 0;
            return None;
        }
        self.selected_index = self
            .selected_index
            .checked_sub(1)
            .unwrap_or(self.entries.len() - 1);
        self.selected_entry()
    }

    fn toggle_mark_at(&mut self, index: usize) -> bool {
        let Some(entry) = self.entries.get(index) else {
            return false;
        };
        if !self.marked_entry_ids.insert(entry.id().to_owned()) {
            self.marked_entry_ids.remove(entry.id());
        }
        true
    }

    fn clear_marks(&mut self) {
        self.marked_entry_ids.clear();
    }

    fn mark_all(&mut self) {
        self.marked_entry_ids = self
            .entries
            .iter()
            .map(|entry| entry.id().to_owned())
            .collect();
    }

    fn render_lines(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|entry| entry.render_line(self.marked_entry_ids.contains(entry.id())))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutocompleteProviderKind {
    Buffer,
    Database,
    Lsp,
    Manual,
}

#[derive(Debug, Clone)]
pub(crate) struct PickerOverlay {
    session: PickerSession,
    actions: BTreeMap<String, PickerAction>,
    quickfix_entries: BTreeMap<String, QuickfixEntry>,
    extra_keybinds: Vec<PickerExtraKeybind>,
    submit_action: Option<PickerAction>,
    show_preview: bool,
    preview_syntax_key: Option<String>,
    preview_syntax_lines: IndexedSyntaxLines,
    mode: PickerMode,
    kind: PickerKind,
    source: Option<PickerSource>,
    provider_id: Option<String>,
    project_discovery_revision: Option<u64>,
}

impl PickerOverlay {
    fn from_entries(title: impl Into<String>, entries: Vec<PickerEntry>) -> Self {
        Self::from_entries_with_limit(title, entries, usize::MAX)
    }

    fn from_entries_with_limit(
        title: impl Into<String>,
        entries: Vec<PickerEntry>,
        result_limit: usize,
    ) -> Self {
        let title = title.into();
        let mut actions = BTreeMap::new();
        let mut quickfix_entries = BTreeMap::new();
        let items = entries
            .into_iter()
            .map(|entry| {
                actions.insert(entry.item.id().to_owned(), entry.action);
                if let Some(quickfix) = entry.quickfix {
                    quickfix_entries.insert(entry.item.id().to_owned(), quickfix);
                }
                entry.item
            })
            .collect();

        Self {
            session: PickerSession::new_with_limit(title, items, result_limit),
            actions,
            quickfix_entries,
            extra_keybinds: Vec::new(),
            submit_action: None,
            show_preview: false,
            preview_syntax_key: None,
            preview_syntax_lines: IndexedSyntaxLines::new(),
            mode: PickerMode::Static,
            kind: PickerKind::Generic,
            source: None,
            provider_id: None,
            project_discovery_revision: None,
        }
    }

    fn with_extra_keybinds(mut self, extra_keybinds: Vec<PickerExtraKeybind>) -> Self {
        self.extra_keybinds = extra_keybinds;
        self
    }

    fn extra_keybinds(&self) -> &[PickerExtraKeybind] {
        &self.extra_keybinds
    }

    fn with_result_order(mut self, result_order: PickerResultOrder) -> Self {
        self.session = self.session.with_result_order(result_order);
        self
    }

    fn with_title(mut self, title: impl Into<String>) -> Self {
        self.session.set_title(title);
        self
    }

    fn search(
        title: impl Into<String>,
        direction: VimSearchDirection,
        entries: Vec<PickerEntry>,
    ) -> Self {
        let title = title.into();
        let mut actions = BTreeMap::new();
        let mut quickfix_entries = BTreeMap::new();
        let items = entries
            .into_iter()
            .map(|entry| {
                actions.insert(entry.item.id().to_owned(), entry.action);
                if let Some(quickfix) = entry.quickfix {
                    quickfix_entries.insert(entry.item.id().to_owned(), quickfix);
                }
                entry.item
            })
            .collect();

        Self {
            session: PickerSession::new_with_limit(title, items, 48).with_preserve_order(),
            actions,
            quickfix_entries,
            extra_keybinds: Vec::new(),
            submit_action: Some(PickerAction::VimSearch(direction)),
            show_preview: false,
            preview_syntax_key: None,
            preview_syntax_lines: IndexedSyntaxLines::new(),
            mode: PickerMode::VimSearch(direction),
            kind: PickerKind::Generic,
            source: None,
            provider_id: None,
            project_discovery_revision: None,
        }
    }

    fn workspace_search(title: impl Into<String>, root: PathBuf) -> Self {
        Self {
            session: PickerSession::new_with_limit(title.into(), Vec::new(), 48)
                .with_preserve_order(),
            actions: BTreeMap::new(),
            quickfix_entries: BTreeMap::new(),
            extra_keybinds: Vec::new(),
            submit_action: Some(PickerAction::NoOp),
            show_preview: true,
            preview_syntax_key: None,
            preview_syntax_lines: IndexedSyntaxLines::new(),
            mode: PickerMode::WorkspaceSearch { root },
            kind: PickerKind::Generic,
            source: None,
            provider_id: None,
            project_discovery_revision: None,
        }
    }

    pub(crate) fn session(&self) -> &PickerSession {
        &self.session
    }

    fn kind(&self) -> PickerKind {
        self.kind
    }

    fn with_kind(mut self, kind: PickerKind) -> Self {
        self.kind = kind;
        self
    }

    fn with_source(mut self, source: PickerSource) -> Self {
        self.source = Some(source);
        self
    }

    fn with_provider_id(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_id = Some(provider_id.into());
        self
    }

    fn source(&self) -> Option<PickerSource> {
        self.source
    }

    fn provider_id(&self) -> Option<&str> {
        self.provider_id.as_deref()
    }

    fn project_discovery_revision(&self) -> Option<u64> {
        self.project_discovery_revision
    }

    fn set_project_discovery_revision(&mut self, revision: u64) {
        self.project_discovery_revision = Some(revision);
    }

    fn with_project_discovery_revision(mut self, revision: u64) -> Self {
        self.project_discovery_revision = Some(revision);
        self
    }

    fn replace_entries_preserving_selection(&mut self, entries: Vec<PickerEntry>) {
        let selected_id = self
            .session
            .selected()
            .map(|selected| selected.item().id().to_owned());
        self.set_entries(entries, 0);
        if let Some(selected_id) = selected_id
            && let Some(index) = self
                .session
                .matches()
                .iter()
                .position(|matched| matched.item().id() == selected_id)
        {
            self.session.set_selected_index(index);
        }
    }

    fn with_preview(mut self) -> Self {
        self.show_preview = true;
        self
    }

    fn show_preview(&self) -> bool {
        self.show_preview
    }

    fn preview_syntax_lines(&self) -> &IndexedSyntaxLines {
        &self.preview_syntax_lines
    }

    fn set_preview_syntax(&mut self, key: Option<String>, lines: IndexedSyntaxLines) {
        self.preview_syntax_key = key;
        self.preview_syntax_lines = lines;
    }

    fn preview_syntax_key(&self) -> Option<&str> {
        self.preview_syntax_key.as_deref()
    }

    fn selected_action(&self) -> Option<PickerAction> {
        if let Some(selected) = self.session.selected()
            && let Some(action) = self.actions.get(selected.item().id())
        {
            return Some(action.clone());
        }
        self.submit_action.clone()
    }

    fn vim_search_direction(&self) -> Option<VimSearchDirection> {
        match self.mode {
            PickerMode::VimSearch(direction) => Some(direction),
            _ => None,
        }
    }

    fn workspace_search_root(&self) -> Option<&Path> {
        match &self.mode {
            PickerMode::WorkspaceSearch { root } => Some(root.as_path()),
            _ => None,
        }
    }

    fn exportable_quickfix_entries(&self) -> Vec<QuickfixEntry> {
        self.session
            .matches()
            .iter()
            .filter_map(|matched| self.quickfix_entries.get(matched.item().id()).cloned())
            .collect()
    }

    fn selected_row_for_extra(&self) -> Option<PickerSelectedRow> {
        self.session.selected().map(|matched| {
            let item = matched.item();
            let path = absolute_path_hint(item.id())
                .or_else(|| item.preview().and_then(absolute_path_hint))
                .map(str::to_owned);
            PickerSelectedRow::new(item.id(), item.label(), path)
        })
    }

    fn exportable_rows_for_extra(&self) -> Vec<PickerExportableRow> {
        self.exportable_quickfix_entries()
            .into_iter()
            .map(|entry| {
                PickerExportableRow::new(
                    entry.id(),
                    entry.path().display().to_string(),
                    entry.target().line,
                    entry.target().column,
                    entry.label(),
                )
            })
            .collect()
    }

    fn resolve_extra(&self, chord: &str) -> PickerExtraDispatch {
        resolve_picker_extra(
            self.extra_keybinds(),
            chord,
            self.selected_row_for_extra(),
            self.exportable_rows_for_extra(),
        )
    }
}

fn absolute_path_hint(value: &str) -> Option<&str> {
    if Path::new(value).is_absolute() {
        Some(value)
    } else {
        None
    }
}

impl PickerOverlay {
    fn set_entries(&mut self, entries: Vec<PickerEntry>, selected_index: usize) {
        let mut actions = BTreeMap::new();
        let mut quickfix_entries = BTreeMap::new();
        let items = entries
            .into_iter()
            .map(|entry| {
                actions.insert(entry.item.id().to_owned(), entry.action);
                if let Some(quickfix) = entry.quickfix {
                    quickfix_entries.insert(entry.item.id().to_owned(), quickfix);
                }
                entry.item
            })
            .collect();
        self.actions = actions;
        self.quickfix_entries = quickfix_entries;
        self.session.set_items(items);
        self.session.set_selected_index(selected_index);
    }

    fn append_query(&mut self, text: &str) {
        let mut query = self.session.query().to_owned();
        query.push_str(text);
        self.session.set_query(query);
        self.ensure_selected_workspace_file_preview();
    }

    fn backspace_query(&mut self) {
        let mut query = self.session.query().chars().collect::<Vec<_>>();
        if query.pop().is_some() {
            self.session
                .set_query(query.into_iter().collect::<String>());
            self.ensure_selected_workspace_file_preview();
        }
    }

    fn ensure_selected_workspace_file_preview(&mut self) {
        if self.source != Some(PickerSource::WorkspaceFiles) {
            return;
        }
        let Some(selected) = self.session.selected() else {
            return;
        };
        if selected.item().preview().is_some() {
            return;
        }
        let path = PathBuf::from(selected.item().id());
        if !path.is_absolute() {
            return;
        }
        let item_id = selected.item().id().to_owned();
        let preview = repository_file_preview(&path);
        self.session.set_item_preview(&item_id, preview);
    }

    fn select_next(&mut self) {
        self.session.select_next();
        self.ensure_selected_workspace_file_preview();
    }

    fn select_previous(&mut self) {
        self.session.select_previous();
        self.ensure_selected_workspace_file_preview();
    }
}

#[derive(Debug, Clone)]
struct RuntimePopupSnapshot {
    active_buffer: BufferId,
}

#[derive(Debug, Clone)]
struct DismissedPopupState {
    title: String,
    buffers: Vec<BufferId>,
    active_buffer: BufferId,
}

#[derive(Debug, Clone)]
struct DebugLayoutState {
    saved_panes: Vec<ShellPane>,
    saved_active_pane: usize,
    saved_split_direction: Option<PaneSplitDirection>,
    saved_golden_ratio_override: Option<bool>,
    saved_pane_size_weights: Option<Vec<u32>>,
    created_pane_ids: Vec<PaneId>,
}

#[derive(Debug, Clone)]
struct ShellWorkspaceView {
    buffer_ids: Vec<BufferId>,
    panes: Vec<ShellPane>,
    active_pane: usize,
    split_buffer_id: BufferId,
    split_direction: Option<PaneSplitDirection>,
    golden_ratio_override: Option<bool>,
    pane_size_weights: Option<Vec<u32>>,
    debug_layout: Option<DebugLayoutState>,
}

impl ShellWorkspaceView {
    fn new(
        primary_pane_id: PaneId,
        primary_buffer_id: BufferId,
        primary_view_state: BufferViewState,
        split_buffer_id: BufferId,
        buffer_ids: Vec<BufferId>,
    ) -> Self {
        Self {
            buffer_ids,
            panes: vec![ShellPane::new(
                primary_pane_id,
                primary_buffer_id,
                primary_view_state,
            )],
            active_pane: 0,
            split_buffer_id,
            split_direction: None,
            golden_ratio_override: None,
            pane_size_weights: None,
            debug_layout: None,
        }
    }
}

#[derive(Debug, Clone)]
struct DirectoryPrefixState {
    chord: String,
    started_at: Instant,
}

#[derive(Debug, Clone)]
struct KeySequenceState {
    scope: KeymapScope,
    vim_mode: KeymapVimMode,
    tokens: Vec<String>,
    started_at: Instant,
    ambiguous_short: Option<String>,
}

#[derive(Debug, Clone)]
struct MarkListState {
    path: PathBuf,
    list: MarkList,
}

impl MarkListState {
    fn load(path: PathBuf) -> Result<Self, String> {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
            Err(error) => {
                return Err(format!(
                    "failed to read Mark List `{}`: {error}",
                    path.display()
                ));
            }
        };
        Ok(Self {
            path,
            list: mark_list_from_persisted_text(&text),
        })
    }

    fn empty(path: PathBuf) -> Self {
        Self {
            path,
            list: MarkList::default(),
        }
    }
}

pub(crate) struct ShellUiState {
    buffers: Vec<ShellBuffer>,
    workspace_views: BTreeMap<WorkspaceId, ShellWorkspaceView>,
    active_workspace: WorkspaceId,
    previous_workspace: Option<WorkspaceId>,
    default_workspace: WorkspaceId,
    input_mode: InputMode,
    vim: VimState,
    pending_ctrl_c: Option<Instant>,
    pending_git_prefix: Option<GitPrefixState>,
    pending_directory_prefix: Option<DirectoryPrefixState>,
    pending_key_sequence: Option<KeySequenceState>,
    attached_lsp_servers: BTreeMap<WorkspaceId, String>,
    picker: Option<PickerOverlay>,
    picker_one_shot: Option<PickerOneShotContext>,
    command_line: Option<CommandLineOverlay>,
    input_prompt: Option<InputPromptOverlay>,
    autocomplete: Option<AutocompleteOverlay>,
    hover: Option<HoverOverlay>,
    notifications: NotificationCenter,
    workspace_unread: BTreeMap<WorkspaceId, u32>,
    last_lsp_notification_revision: u64,
    last_lsp_diagnostics_generation: Option<u64>,
    last_attached_lsp_label_key: Option<(WorkspaceId, Option<PathBuf>, u64)>,
    popup_focus: bool,
    popup_buffer_id: Option<BufferId>,
    workspace_dock_open: bool,
    workspace_dock_focus: bool,
    workspace_dock_branches: WorkspaceDockBranchCache,
    acp_dock_open: bool,
    acp_dock_focus: bool,
    dismissed_popups: BTreeMap<WorkspaceId, DismissedPopupState>,
    yank_flash: Option<YankFlash>,
    git_summary: GitSummaryState,
    git_head_blobs: GitHeadBlobCache,
    autocomplete_worker: AutocompleteWorkerState,
    inline_completion_worker: InlineCompletionWorkerState,
    vim_search_worker: VimSearchWorkerState,
    workspace_search_worker: WorkspaceSearchWorkerState,
    file_reload_worker: FileReloadWorkerState,
    syntax_refresh_worker: SyntaxRefreshWorkerState,
    lsp_sync_worker: LspSyncWorkerState,
    streamed_command_worker: StreamedCommandWorkerState,
    git_editor: GitEditorState,
    issues_worker: IssuesWorkerState,
    indent_parse_sessions: BTreeMap<BufferId, SyntaxParseSession>,
    /// Per-workspace last-used build command.  Set when the user runs
    /// `workspace.compile`; reused by `workspace.recompile`.
    compile_commands: BTreeMap<WorkspaceId, String>,
    pending_syntax_prewarm_roots: VecDeque<PathBuf>,
    pending_workspace_readme_opens: VecDeque<PathBuf>,
    /// Pending DAP start waiting on minibuffer hole fill.
    pending_dap_start: Option<PendingDapStartPrompt>,
    failed_tool_installs: BTreeSet<String>,
}

impl ShellUiState {
    fn new(
        default_workspace: WorkspaceId,
        primary_pane_id: PaneId,
        primary: ShellBuffer,
        secondary: ShellBuffer,
        split_buffer_id: BufferId,
    ) -> Self {
        let primary_view_state = primary.view_state();
        let primary_buffer_id = primary.id();
        let secondary_buffer_id = secondary.id();
        let mut workspace_views = BTreeMap::new();
        workspace_views.insert(
            default_workspace,
            ShellWorkspaceView::new(
                primary_pane_id,
                primary_buffer_id,
                primary_view_state,
                split_buffer_id,
                vec![primary_buffer_id, secondary_buffer_id],
            ),
        );
        Self {
            buffers: vec![primary, secondary],
            workspace_views,
            active_workspace: default_workspace,
            previous_workspace: None,
            default_workspace,
            input_mode: InputMode::Normal,
            vim: VimState::default(),
            pending_ctrl_c: None,
            pending_git_prefix: None,
            pending_directory_prefix: None,
            pending_key_sequence: None,
            attached_lsp_servers: BTreeMap::new(),
            picker: None,
            picker_one_shot: None,
            command_line: None,
            input_prompt: None,
            autocomplete: None,
            hover: None,
            notifications: NotificationCenter::default(),
            workspace_unread: BTreeMap::new(),
            last_lsp_notification_revision: 0,
            last_lsp_diagnostics_generation: None,
            last_attached_lsp_label_key: None,
            popup_focus: false,
            popup_buffer_id: None,
            workspace_dock_open: false,
            workspace_dock_focus: false,
            workspace_dock_branches: WorkspaceDockBranchCache::new(),
            acp_dock_open: false,
            acp_dock_focus: false,
            dismissed_popups: BTreeMap::new(),
            yank_flash: None,
            git_summary: GitSummaryState::new(),
            git_head_blobs: GitHeadBlobCache::new(),
            autocomplete_worker: AutocompleteWorkerState::new(),
            inline_completion_worker: InlineCompletionWorkerState::new(),
            vim_search_worker: VimSearchWorkerState::new(),
            workspace_search_worker: WorkspaceSearchWorkerState::new(),
            file_reload_worker: FileReloadWorkerState::new(),
            syntax_refresh_worker: SyntaxRefreshWorkerState::disabled(),
            lsp_sync_worker: LspSyncWorkerState::new(),
            streamed_command_worker: StreamedCommandWorkerState::new(),
            git_editor: GitEditorState::new(),
            issues_worker: IssuesWorkerState::new(),
            indent_parse_sessions: BTreeMap::new(),
            compile_commands: BTreeMap::new(),
            pending_syntax_prewarm_roots: VecDeque::new(),
            pending_workspace_readme_opens: VecDeque::new(),
            pending_dap_start: None,
            failed_tool_installs: BTreeSet::new(),
        }
    }

    fn pane_count(&self) -> usize {
        self.workspace_view()
            .map(|view| view.panes.len())
            .unwrap_or(0)
    }

    fn picker_visible(&self) -> bool {
        self.picker.is_some()
    }

    fn command_line_visible(&self) -> bool {
        self.command_line.is_some()
    }

    fn focused_buffer_id(&self) -> Option<BufferId> {
        if self.popup_focus {
            self.popup_buffer_id.or_else(|| self.active_buffer_id())
        } else {
            self.active_buffer_id()
        }
    }

    fn set_popup_buffer(&mut self, buffer_id: BufferId) {
        if self.popup_buffer_id == Some(buffer_id) {
            return;
        }
        if self.popup_focus {
            if let Some(previous_buffer_id) = self.popup_buffer_id {
                self.persist_buffer_vim_state(previous_buffer_id);
            }
            self.popup_buffer_id = Some(buffer_id);
            self.restore_buffer_vim_state(buffer_id);
        } else {
            self.popup_buffer_id = Some(buffer_id);
        }
    }

    fn clear_popup_buffer(&mut self) {
        self.popup_buffer_id = None;
    }

    fn stash_dismissed_popup(
        &mut self,
        workspace_id: WorkspaceId,
        title: String,
        buffers: Vec<BufferId>,
        active_buffer: BufferId,
    ) {
        self.dismissed_popups.insert(
            workspace_id,
            DismissedPopupState {
                title,
                buffers,
                active_buffer,
            },
        );
    }

    fn dismissed_popup(&self, workspace_id: WorkspaceId) -> Option<&DismissedPopupState> {
        self.dismissed_popups.get(&workspace_id)
    }

    fn clear_dismissed_popup(&mut self, workspace_id: WorkspaceId) {
        self.dismissed_popups.remove(&workspace_id);
    }

    fn set_popup_focus(&mut self, focus: bool) {
        if self.popup_focus == focus {
            return;
        }
        if focus {
            self.workspace_dock_focus = false;
            self.acp_dock_focus = false;
        }
        self.persist_active_buffer_vim_state();
        self.popup_focus = focus;
        self.restore_active_buffer_vim_state();
    }

    fn workspace_dock_open(&self) -> bool {
        self.workspace_dock_open
    }

    fn set_workspace_dock_open(&mut self, open: bool) {
        self.workspace_dock_open = open;
    }

    fn toggle_workspace_dock_open(&mut self) {
        self.workspace_dock_open = !self.workspace_dock_open;
    }

    fn workspace_dock_focus(&self) -> bool {
        self.workspace_dock_focus
    }

    fn set_workspace_dock_focus(&mut self, focus: bool) {
        if self.workspace_dock_focus == focus {
            return;
        }
        if focus {
            self.popup_focus = false;
            self.acp_dock_focus = false;
        }
        self.workspace_dock_focus = focus;
    }

    fn workspace_dock_focus_active(&self, user_library: &dyn UserLibrary) -> bool {
        self.workspace_dock_focus() && workspace_dock_visible(user_library, self)
    }

    fn workspace_dock_branches(&self) -> &WorkspaceDockBranchCache {
        &self.workspace_dock_branches
    }

    fn workspace_dock_branches_mut(&mut self) -> &mut WorkspaceDockBranchCache {
        &mut self.workspace_dock_branches
    }

    fn acp_dock_open(&self) -> bool {
        self.acp_dock_open
    }

    fn toggle_acp_dock_open(&mut self) {
        self.acp_dock_open = !self.acp_dock_open;
        if !self.acp_dock_open {
            self.acp_dock_focus = false;
        }
    }

    fn acp_dock_focus(&self) -> bool {
        self.acp_dock_focus
    }

    fn set_acp_dock_focus(&mut self, focus: bool) {
        if self.acp_dock_focus == focus {
            return;
        }
        if focus {
            self.popup_focus = false;
            self.workspace_dock_focus = false;
        }
        self.acp_dock_focus = focus;
    }

    fn acp_dock_focus_active(&self) -> bool {
        self.acp_dock_focus() && acp_dock_visible(self)
    }

    fn popup_focus_allowed(&self, popup: &RuntimePopupSnapshot) -> bool {
        if let Some(buffer) = self.buffer(popup.active_buffer) {
            return !buffer_is_oil_preview(&buffer.kind);
        }
        true
    }

    fn popup_focus_active(&self, popup: &RuntimePopupSnapshot) -> bool {
        self.popup_focus && self.popup_focus_allowed(popup)
    }

    fn git_summary(&self) -> Option<GitSummarySnapshot> {
        self.git_summary.snapshot()
    }

    fn git_summary_revision(&self) -> u64 {
        self.git_summary.snapshot_revision()
    }

    fn git_summary_refresh_due(&self, now: Instant) -> bool {
        self.git_summary.refresh_due(now)
    }

    fn git_summary_state(&self) -> GitSummaryState {
        self.git_summary.clone()
    }

    fn git_head_blob_cache(&self) -> GitHeadBlobCache {
        self.git_head_blobs.clone()
    }

    fn take_git_summary_changed(&self) -> bool {
        self.git_summary.take_changed()
    }

    fn mark_git_summary_refreshed(&mut self, now: Instant) {
        self.git_summary.mark_refreshed(now);
    }

    fn mark_git_summary_stale(&mut self) {
        self.git_summary.mark_stale();
    }

    fn clear_git_summary(&self) {
        self.git_summary.set_snapshot(None);
    }

    fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    fn input_mode_for_buffer(&self, buffer_id: BufferId, active: bool) -> InputMode {
        if active {
            self.input_mode
        } else {
            self.buffer(buffer_id)
                .map(|buffer| buffer.vim_buffer_state.input_mode)
                .unwrap_or(InputMode::Normal)
        }
    }

    fn vim_target_for_buffer(&self, buffer_id: BufferId, active: bool) -> VimTarget {
        if active {
            self.vim.target
        } else {
            self.buffer(buffer_id)
                .map(|buffer| buffer.vim_buffer_state.target)
                .unwrap_or(VimTarget::Buffer)
        }
    }

    fn visual_selection_for_buffer(
        &self,
        buffer: &ShellBuffer,
        active: bool,
    ) -> Option<VisualSelection> {
        let (input_mode, target, visual_anchor, visual_kind) = if active {
            (
                self.input_mode,
                self.vim.target,
                self.vim.visual_anchor,
                self.vim.visual_kind,
            )
        } else {
            (
                buffer.vim_buffer_state.input_mode,
                buffer.vim_buffer_state.target,
                buffer.vim_buffer_state.visual_anchor,
                buffer.vim_buffer_state.visual_kind,
            )
        };
        if input_mode != InputMode::Visual || target == VimTarget::Input {
            return None;
        }
        visual_anchor.and_then(|anchor| visual_selection(buffer, anchor, visual_kind))
    }

    fn multicursor_for_buffer(
        &self,
        buffer_id: BufferId,
        active: bool,
    ) -> Option<&MulticursorState> {
        if active {
            self.vim.multicursor.as_ref()
        } else {
            self.buffer(buffer_id)
                .and_then(|buffer| buffer.vim_buffer_state.multicursor.as_ref())
        }
    }

    fn persist_buffer_vim_state(&mut self, buffer_id: BufferId) {
        let state = self.vim.active_buffer_state(self.input_mode);
        if let Some(buffer) = self.buffer_mut(buffer_id) {
            buffer.vim_buffer_state = state;
        }
    }

    fn persist_active_buffer_vim_state(&mut self) {
        if let Some(buffer_id) = self.focused_buffer_id() {
            self.persist_buffer_vim_state(buffer_id);
        }
    }

    fn restore_buffer_vim_state(&mut self, buffer_id: BufferId) {
        let state = self
            .buffer(buffer_id)
            .map(|buffer| buffer.vim_buffer_state.clone())
            .unwrap_or_default();
        self.vim
            .apply_active_buffer_state(&mut self.input_mode, &state);
    }

    fn restore_active_buffer_vim_state(&mut self) {
        if let Some(buffer_id) = self.focused_buffer_id() {
            self.restore_buffer_vim_state(buffer_id);
        } else {
            self.vim
                .apply_active_buffer_state(&mut self.input_mode, &VimBufferState::default());
        }
    }

    fn enter_normal_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.vim.visual_anchor = None;
        self.vim.visual_anchor_char_offset = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.multicursor = None;
        self.vim.clear_transient();
        self.persist_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn enter_insert_mode(&mut self) {
        self.input_mode = InputMode::Insert;
        self.vim.visual_anchor = None;
        self.vim.visual_anchor_char_offset = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.clear_transient();
        self.persist_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn enter_replace_mode(&mut self) {
        self.input_mode = InputMode::Replace;
        self.vim.visual_anchor = None;
        self.vim.visual_anchor_char_offset = None;
        self.vim.visual_kind = VisualSelectionKind::Character;
        self.vim.clear_transient();
        self.persist_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn enter_visual_mode(&mut self, anchor: TextPoint, kind: VisualSelectionKind) {
        self.input_mode = InputMode::Visual;
        self.vim.visual_anchor = Some(anchor);
        self.vim.visual_anchor_char_offset = self
            .focused_buffer_id()
            .and_then(|buffer_id| self.buffer(buffer_id))
            .and_then(|buffer| buffer.char_offset_for_point(anchor));
        self.vim.visual_kind = kind;
        self.vim.clear_transient();
        self.persist_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    pub(crate) fn vim(&self) -> &VimState {
        &self.vim
    }

    fn vim_mut(&mut self) -> &mut VimState {
        &mut self.vim
    }

    fn active_buffer_targets_input(&self) -> bool {
        self.focused_buffer_id()
            .and_then(|buffer_id| self.buffer(buffer_id))
            .is_some_and(|buffer| buffer.has_input_field() && self.vim.target == VimTarget::Input)
    }

    fn set_active_vim_target(&mut self, target: VimTarget) {
        self.vim.target = target;
        self.persist_active_buffer_vim_state();
    }

    pub(crate) fn active_workspace(&self) -> WorkspaceId {
        self.active_workspace
    }

    fn previous_workspace(&self) -> Option<WorkspaceId> {
        self.previous_workspace
    }

    fn default_workspace(&self) -> WorkspaceId {
        self.default_workspace
    }

    fn has_workspace(&self, workspace_id: WorkspaceId) -> bool {
        self.workspace_views.contains_key(&workspace_id)
    }

    fn switch_workspace(&mut self, workspace_id: WorkspaceId) {
        self.persist_active_pane_view_state();
        self.persist_active_buffer_vim_state();
        if self.active_workspace != workspace_id {
            self.previous_workspace = Some(self.active_workspace);
            self.active_workspace = workspace_id;
        }
        self.workspace_unread.remove(&workspace_id);
        self.restore_active_pane_view_state();
        self.restore_active_buffer_vim_state();
        self.close_picker();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn add_workspace(
        &mut self,
        workspace_id: WorkspaceId,
        primary_pane_id: PaneId,
        primary: ShellBuffer,
        secondary: ShellBuffer,
        split_buffer_id: BufferId,
    ) {
        let primary_view_state = primary.view_state();
        let primary_buffer_id = primary.id();
        let secondary_buffer_id = secondary.id();
        self.insert_buffer(primary);
        self.insert_buffer(secondary);
        self.workspace_views.insert(
            workspace_id,
            ShellWorkspaceView::new(
                primary_pane_id,
                primary_buffer_id,
                primary_view_state,
                split_buffer_id,
                vec![primary_buffer_id, secondary_buffer_id],
            ),
        );
    }

    fn remove_workspace(&mut self, workspace_id: WorkspaceId) {
        let active_workspace_removed = self.active_workspace == workspace_id;
        if active_workspace_removed {
            self.persist_active_buffer_vim_state();
        }
        let removed = self.workspace_views.remove(&workspace_id);
        self.attached_lsp_servers.remove(&workspace_id);
        self.dismissed_popups.remove(&workspace_id);
        if let Some(removed) = removed {
            self.buffers
                .retain(|buffer| !removed.buffer_ids.contains(&buffer.id()));
        }
        if self.previous_workspace == Some(workspace_id) {
            self.previous_workspace = None;
        }
        if self.active_workspace == workspace_id {
            self.active_workspace = self.default_workspace;
            self.restore_active_pane_view_state();
            self.restore_active_buffer_vim_state();
        }
    }

    fn set_attached_lsp_server(
        &mut self,
        workspace_id: WorkspaceId,
        attached_lsp_server: Option<String>,
    ) -> bool {
        let previous = self.attached_lsp_servers.get(&workspace_id).cloned();
        match attached_lsp_server.clone() {
            Some(server) => {
                self.attached_lsp_servers.insert(workspace_id, server);
            }
            None => {
                self.attached_lsp_servers.remove(&workspace_id);
            }
        }
        previous != attached_lsp_server
    }

    fn panes(&self) -> Option<&[ShellPane]> {
        self.workspace_view().map(|view| view.panes.as_slice())
    }

    fn active_pane_index(&self) -> usize {
        self.workspace_view()
            .map(|view| view.active_pane)
            .unwrap_or(0)
    }

    fn active_pane_id(&self) -> Option<PaneId> {
        self.workspace_view()
            .and_then(|view| view.panes.get(view.active_pane))
            .map(|pane| pane.pane_id)
    }

    fn active_pane_buffer(&self) -> Option<(PaneId, BufferId)> {
        self.workspace_view()
            .and_then(|view| view.panes.get(view.active_pane))
            .map(|pane| (pane.pane_id, pane.buffer_id))
    }

    fn buffer_view_state(&self, pane_id: PaneId, buffer_id: BufferId) -> Option<BufferViewState> {
        let fallback = self.buffer(buffer_id).map(ShellBuffer::view_state);
        let Some(view) = self.workspace_view() else {
            return fallback;
        };
        let Some(pane) = view.panes.iter().find(|pane| pane.pane_id == pane_id) else {
            return fallback;
        };
        pane.view_state(buffer_id).or(fallback)
    }

    fn persist_active_pane_view_state(&mut self) {
        let Some((pane_id, buffer_id)) = self.active_pane_buffer() else {
            return;
        };
        let Some(view_state) = self.buffer(buffer_id).map(ShellBuffer::view_state) else {
            return;
        };
        if let Some(view) = self.workspace_view_mut()
            && let Some(pane) = view.panes.iter_mut().find(|pane| pane.pane_id == pane_id)
        {
            pane.store_view_state(buffer_id, view_state);
        }
    }

    fn restore_active_pane_view_state(&mut self) {
        let Some((pane_id, buffer_id)) = self.active_pane_buffer() else {
            return;
        };
        let Some(view_state) = self.buffer_view_state(pane_id, buffer_id) else {
            return;
        };
        if let Some(buffer) = self.buffer_mut(buffer_id) {
            buffer.restore_view_state(view_state);
        }
    }

    fn focus_pane(&mut self, pane_id: PaneId) {
        self.persist_active_pane_view_state();
        self.persist_active_buffer_vim_state();
        if let Some(view) = self.workspace_view_mut()
            && let Some(index) = view.panes.iter().position(|pane| pane.pane_id == pane_id)
        {
            view.active_pane = index;
        }
        self.restore_active_pane_view_state();
        self.restore_active_buffer_vim_state();
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
    }

    fn split_buffer_id(&self) -> Option<BufferId> {
        self.workspace_view().map(|view| view.split_buffer_id)
    }

    fn pane_split_direction(&self) -> PaneSplitDirection {
        self.workspace_view()
            .and_then(|view| view.split_direction)
            .unwrap_or(PaneSplitDirection::Horizontal)
    }

    fn effective_golden_ratio(&self, user_library: &dyn UserLibrary) -> bool {
        self.workspace_view()
            .and_then(|view| view.golden_ratio_override)
            .unwrap_or_else(|| user_library.pane_config().golden_ratio)
    }

    fn pane_size_weights(&self) -> Option<&[u32]> {
        self.workspace_view()
            .and_then(|view| view.pane_size_weights.as_deref())
    }

    fn set_db_multiview_layout(&mut self, enabled: bool) {
        let Some(view) = self.workspace_view_mut() else {
            return;
        };
        if enabled {
            view.golden_ratio_override = Some(false);
            view.pane_size_weights =
                Some(vec![DB_MULTIVIEW_LEFT_WEIGHT, DB_MULTIVIEW_RIGHT_WEIGHT]);
        } else {
            view.golden_ratio_override = None;
            view.pane_size_weights = None;
        }
    }

    fn is_db_multiview_active(&self) -> bool {
        self.workspace_view().is_some_and(|view| {
            view.pane_size_weights.as_deref()
                == Some([DB_MULTIVIEW_LEFT_WEIGHT, DB_MULTIVIEW_RIGHT_WEIGHT].as_slice())
        })
    }

    fn is_debug_layout_active(&self) -> bool {
        self.workspace_view()
            .is_some_and(|view| view.debug_layout.is_some())
    }

    fn begin_debug_layout(&mut self, created_pane_ids: Vec<PaneId>) -> bool {
        let Some(view) = self.workspace_view_mut() else {
            return false;
        };
        if view.debug_layout.is_some() {
            return false;
        }
        view.debug_layout = Some(DebugLayoutState {
            saved_panes: view.panes.clone(),
            saved_active_pane: view.active_pane,
            saved_split_direction: view.split_direction,
            saved_golden_ratio_override: view.golden_ratio_override,
            saved_pane_size_weights: view.pane_size_weights.clone(),
            created_pane_ids,
        });
        view.golden_ratio_override = Some(false);
        view.pane_size_weights = Some(vec![
            DEBUG_LAYOUT_BREAKPOINTS_WEIGHT,
            DEBUG_LAYOUT_EDITOR_WEIGHT,
            DEBUG_LAYOUT_LOCALS_WEIGHT,
        ]);
        view.split_direction = Some(PaneSplitDirection::Vertical);
        true
    }

    fn take_debug_layout_state(&mut self) -> Option<DebugLayoutState> {
        self.workspace_view_mut()
            .and_then(|view| view.debug_layout.take())
    }

    fn restore_debug_layout_snapshot(&mut self, state: DebugLayoutState) {
        let Some(view) = self.workspace_view_mut() else {
            return;
        };
        view.panes = state.saved_panes;
        view.active_pane = state
            .saved_active_pane
            .min(view.panes.len().saturating_sub(1));
        view.split_direction = state.saved_split_direction;
        view.golden_ratio_override = state.saved_golden_ratio_override;
        view.pane_size_weights = state.saved_pane_size_weights;
        view.debug_layout = None;
    }

    fn set_debug_layout_panes(
        &mut self,
        panes: Vec<(PaneId, BufferId)>,
        active_pane: usize,
    ) -> Result<(), String> {
        if panes.len() != 3 {
            return Err("Debug Layout requires exactly three panes".to_owned());
        }
        let mut next_panes = Vec::with_capacity(3);
        for &(pane_id, buffer_id) in &panes {
            let view_state = self
                .workspace_view()
                .and_then(|view| {
                    view.panes
                        .iter()
                        .find(|pane| pane.pane_id == pane_id)
                        .and_then(|pane| pane.view_state(buffer_id))
                })
                .or_else(|| {
                    self.buffers
                        .iter()
                        .find(|buffer| buffer.id() == buffer_id)
                        .map(ShellBuffer::view_state)
                })
                .unwrap_or(BufferViewState {
                    cursor: TextPoint::default(),
                    scroll_row: 0,
                    scroll_col: 0,
                });
            next_panes.push(ShellPane::new(pane_id, buffer_id, view_state));
        }
        let Some(view) = self.workspace_view_mut() else {
            return Err("workspace view is missing".to_owned());
        };
        for &(_, buffer_id) in &panes {
            if !view.buffer_ids.contains(&buffer_id) {
                view.buffer_ids.push(buffer_id);
            }
        }
        view.panes = next_panes;
        view.active_pane = active_pane.min(2);
        Ok(())
    }

    fn db_multiview_sidebar_pane_id(&self) -> Option<PaneId> {
        self.workspace_view().and_then(|view| {
            view.panes.iter().find_map(|pane| {
                self.buffer(pane.buffer_id)
                    .filter(|buffer| buffer_is_db_sidebar(&buffer.kind))
                    .map(|_| pane.pane_id)
            })
        })
    }

    fn active_workspace_buffer_ids(&self) -> Option<&[BufferId]> {
        self.workspace_view().map(|view| view.buffer_ids.as_slice())
    }

    pub(crate) fn attached_lsp_server(&self) -> Option<&str> {
        self.attached_lsp_servers
            .get(&self.active_workspace)
            .map(String::as_str)
    }

    fn workspace_view(&self) -> Option<&ShellWorkspaceView> {
        self.workspace_views.get(&self.active_workspace)
    }

    fn workspace_view_mut(&mut self) -> Option<&mut ShellWorkspaceView> {
        self.workspace_views.get_mut(&self.active_workspace)
    }

    fn insert_buffer(&mut self, mut buffer: ShellBuffer) {
        let new_watch_path = shell_buffer_watch_path(&buffer);
        if let Some(existing) = self
            .buffers
            .iter_mut()
            .find(|existing| existing.id() == buffer.id())
        {
            let old_watch_path = shell_buffer_watch_path(existing);
            buffer.vim_buffer_state = existing.vim_buffer_state.clone();
            *existing = buffer;
            sync_file_reload_watch(
                &mut self.file_reload_worker,
                old_watch_path.as_deref(),
                new_watch_path.as_deref(),
            );
        } else {
            self.buffers.push(buffer);
            sync_file_reload_watch(
                &mut self.file_reload_worker,
                None,
                new_watch_path.as_deref(),
            );
        }
    }

    fn remove_buffer(&mut self, buffer_id: BufferId) {
        let removed_active_buffer = self.active_buffer_id() == Some(buffer_id);
        let removed_watch_path = self
            .buffers
            .iter()
            .find(|buffer| buffer.id() == buffer_id)
            .and_then(shell_buffer_watch_path);
        if !removed_active_buffer {
            self.persist_active_pane_view_state();
            self.persist_active_buffer_vim_state();
        }
        self.buffers.retain(|buffer| buffer.id() != buffer_id);
        for dismissed in self.dismissed_popups.values_mut() {
            dismissed.buffers.retain(|id| *id != buffer_id);
            if dismissed.active_buffer == buffer_id
                && let Some(active_buffer) = dismissed.buffers.first().copied()
            {
                dismissed.active_buffer = active_buffer;
            }
        }
        self.dismissed_popups
            .retain(|_, dismissed| !dismissed.buffers.is_empty());
        if let Some(path) = removed_watch_path.as_deref() {
            self.file_reload_worker.unwatch_path(path);
            self.lsp_sync_worker.cancel_path(path);
        }
        self.indent_parse_sessions.remove(&buffer_id);
        self.streamed_command_worker.cancel_and_remove(buffer_id);
        self.git_editor.abort_if_session(buffer_id);
        for view in self.workspace_views.values_mut() {
            if view.buffer_ids.contains(&buffer_id) {
                view.buffer_ids.retain(|id| *id != buffer_id);
                if let Some(fallback) = view.buffer_ids.first().copied() {
                    if view.split_buffer_id == buffer_id {
                        view.split_buffer_id = fallback;
                    }
                    for pane in view.panes.iter_mut() {
                        pane.remove_view_state(buffer_id);
                        if pane.buffer_id == buffer_id {
                            pane.buffer_id = fallback;
                        }
                    }
                }
            }
            if view.active_pane >= view.panes.len() {
                view.active_pane = 0;
            }
        }
        if removed_active_buffer {
            self.restore_active_pane_view_state();
            self.restore_active_buffer_vim_state();
        }
    }

    fn take_indent_parse_session(&mut self, buffer_id: BufferId) -> Option<SyntaxParseSession> {
        self.indent_parse_sessions.remove(&buffer_id)
    }

    fn store_indent_parse_session(
        &mut self,
        buffer_id: BufferId,
        parse_session: Option<SyntaxParseSession>,
    ) {
        if let Some(parse_session) = parse_session {
            self.indent_parse_sessions.insert(buffer_id, parse_session);
        } else {
            self.indent_parse_sessions.remove(&buffer_id);
        }
    }

    pub(crate) fn picker(&self) -> Option<&PickerOverlay> {
        self.picker.as_ref()
    }

    fn picker_kind(&self) -> Option<PickerKind> {
        self.picker.as_ref().map(PickerOverlay::kind)
    }

    fn picker_mut(&mut self) -> Option<&mut PickerOverlay> {
        self.picker.as_mut()
    }

    fn set_picker(&mut self, picker: PickerOverlay) {
        self.close_command_line();
        self.close_autocomplete();
        self.close_hover();
        self.vim_search_worker.clear_pending();
        self.workspace_search_worker.clear_pending();
        self.picker_one_shot = None;
        self.picker = Some(picker);
    }

    fn close_picker(&mut self) {
        self.vim_search_worker.clear_pending();
        self.workspace_search_worker.clear_pending();
        self.picker = None;
    }

    fn set_picker_one_shot(&mut self, context: PickerOneShotContext) {
        self.picker_one_shot = Some(context);
    }

    /// Takes the one-shot picker context left by a Picker Extra Keybind fire.
    pub(crate) fn take_picker_one_shot(&mut self) -> Option<PickerOneShotContext> {
        self.picker_one_shot.take()
    }

    fn command_line(&self) -> Option<&CommandLineOverlay> {
        self.command_line.as_ref()
    }

    fn command_line_mut(&mut self) -> Option<&mut CommandLineOverlay> {
        self.command_line.as_mut()
    }

    fn set_command_line(&mut self, command_line: CommandLineOverlay) {
        self.close_picker();
        self.close_autocomplete();
        self.close_hover();
        self.command_line = Some(command_line);
    }

    fn close_command_line(&mut self) {
        self.command_line = None;
    }

    pub(in crate::shell) fn input_prompt(&self) -> Option<&InputPromptOverlay> {
        self.input_prompt.as_ref()
    }

    pub(in crate::shell) fn input_prompt_mut(&mut self) -> Option<&mut InputPromptOverlay> {
        self.input_prompt.as_mut()
    }

    pub(in crate::shell) fn open_input_prompt(&mut self, overlay: InputPromptOverlay) {
        self.close_picker();
        self.close_autocomplete();
        self.close_hover();
        self.close_command_line();
        self.input_prompt = Some(overlay);
    }

    pub(super) fn close_input_prompt(&mut self) {
        self.input_prompt = None;
    }

    pub(super) fn input_prompt_visible(&self) -> bool {
        self.input_prompt.is_some()
    }

    fn autocomplete(&self) -> Option<&AutocompleteOverlay> {
        self.autocomplete.as_ref()
    }

    fn autocomplete_mut(&mut self) -> Option<&mut AutocompleteOverlay> {
        self.autocomplete.as_mut()
    }

    fn set_autocomplete(&mut self, autocomplete: AutocompleteOverlay) {
        self.close_picker();
        self.close_command_line();
        self.close_hover();
        self.autocomplete_worker.clear_pending();
        self.autocomplete = Some(autocomplete);
    }

    fn close_autocomplete(&mut self) {
        self.autocomplete_worker.clear_pending();
        self.autocomplete = None;
    }

    fn hover(&self) -> Option<&HoverOverlay> {
        self.hover.as_ref()
    }

    fn hover_mut(&mut self) -> Option<&mut HoverOverlay> {
        self.hover.as_mut()
    }

    fn set_hover(&mut self, hover: HoverOverlay) {
        self.close_picker();
        self.close_command_line();
        self.close_autocomplete();
        self.hover = Some(hover);
    }

    fn close_hover(&mut self) {
        self.hover = None;
    }

    fn apply_notification(&mut self, update: NotificationUpdate, now: Instant) -> bool {
        let is_new = !self
            .notifications
            .entries
            .iter()
            .any(|entry| entry.key == update.key);
        if is_new
            && let Some(workspace_id) = update.workspace_id
            && workspace_id != self.active_workspace
        {
            let count = self.workspace_unread.entry(workspace_id).or_insert(0);
            *count = count.saturating_add(1);
        }
        self.notifications.apply(update, now)
    }

    fn workspace_unread_count(&self, workspace_id: WorkspaceId) -> u32 {
        self.workspace_unread
            .get(&workspace_id)
            .copied()
            .unwrap_or(0)
    }

    fn prune_notifications(&mut self, now: Instant) -> bool {
        self.notifications.prune_expired(now)
    }

    fn visible_notifications(&self, now: Instant) -> Vec<&ShellNotification> {
        self.notifications.visible(now)
    }

    fn notification_revision(&self) -> u64 {
        self.notifications.revision()
    }

    fn notification_deadline(&self, now: Instant) -> Option<Instant> {
        self.notifications.next_deadline(now)
    }

    fn last_lsp_notification_revision(&self) -> u64 {
        self.last_lsp_notification_revision
    }

    fn set_last_lsp_notification_revision(&mut self, revision: u64) {
        self.last_lsp_notification_revision = revision;
    }

    fn last_lsp_diagnostics_generation(&self) -> Option<u64> {
        self.last_lsp_diagnostics_generation
    }

    fn set_last_lsp_diagnostics_generation(&mut self, generation: u64) {
        self.last_lsp_diagnostics_generation = Some(generation);
    }

    fn last_attached_lsp_label_key(&self) -> Option<&(WorkspaceId, Option<PathBuf>, u64)> {
        self.last_attached_lsp_label_key.as_ref()
    }

    fn set_last_attached_lsp_label_key(&mut self, key: (WorkspaceId, Option<PathBuf>, u64)) {
        self.last_attached_lsp_label_key = Some(key);
    }

    fn configure_syntax_refresh_worker(
        &mut self,
        configs: Vec<LanguageConfiguration>,
        install_root: PathBuf,
        query_asset_root: Option<PathBuf>,
    ) {
        self.syntax_refresh_worker
            .configure(configs, install_root, query_asset_root);
    }

    #[cfg(test)]
    pub(crate) fn syntax_refresh_worker_is_live(&self) -> bool {
        self.syntax_refresh_worker.has_live_worker()
    }

    fn set_yank_flash(&mut self, buffer_id: BufferId, selection: VisualSelection) {
        const YANK_FLASH_DURATION: Duration = Duration::from_millis(140);
        self.yank_flash = Some(YankFlash {
            buffer_id,
            selection,
            until: Instant::now() + YANK_FLASH_DURATION,
        });
    }

    pub(crate) fn yank_flash(&self, buffer_id: BufferId, now: Instant) -> Option<VisualSelection> {
        self.yank_flash.and_then(|flash| {
            (flash.buffer_id == buffer_id && now <= flash.until).then_some(flash.selection)
        })
    }

    fn yank_flash_deadline(&self, now: Instant) -> Option<Instant> {
        self.yank_flash
            .and_then(|flash| (now <= flash.until).then_some(flash.until))
    }

    fn buffer(&self, buffer_id: BufferId) -> Option<&ShellBuffer> {
        self.buffers.iter().find(|buffer| buffer.id() == buffer_id)
    }

    fn buffer_mut(&mut self, buffer_id: BufferId) -> Option<&mut ShellBuffer> {
        self.buffers
            .iter_mut()
            .find(|buffer| buffer.id() == buffer_id)
    }

    fn ensure_buffer(
        &mut self,
        buffer_id: BufferId,
        name: &str,
        kind: BufferKind,
        user_library: &dyn UserLibrary,
    ) -> &mut ShellBuffer {
        if let Some(view) = self.workspace_view_mut()
            && !view.buffer_ids.contains(&buffer_id)
        {
            view.buffer_ids.push(buffer_id);
        }

        if let Some(index) = self
            .buffers
            .iter()
            .position(|buffer| buffer.id() == buffer_id)
        {
            return &mut self.buffers[index];
        }

        self.buffers.push(ShellBuffer::placeholder(
            buffer_id,
            name,
            kind,
            user_library,
        ));
        let index = self.buffers.len() - 1;
        &mut self.buffers[index]
    }

    fn ensure_popup_buffer(
        &mut self,
        buffer_id: BufferId,
        name: &str,
        kind: BufferKind,
        user_library: &dyn UserLibrary,
    ) -> &mut ShellBuffer {
        if let Some(index) = self
            .buffers
            .iter()
            .position(|buffer| buffer.id() == buffer_id)
        {
            return &mut self.buffers[index];
        }

        self.buffers.push(ShellBuffer::placeholder(
            buffer_id,
            name,
            kind,
            user_library,
        ));
        let index = self.buffers.len() - 1;
        &mut self.buffers[index]
    }

    fn active_buffer_id(&self) -> Option<BufferId> {
        self.workspace_view()?
            .panes
            .get(self.active_pane_index())
            .map(|pane| pane.buffer_id)
    }

    fn focus_buffer_in_active_pane(&mut self, buffer_id: BufferId) {
        self.persist_active_pane_view_state();
        self.persist_active_buffer_vim_state();
        if self.buffers.iter().any(|buffer| buffer.id() == buffer_id)
            && let Some(view) = self.workspace_view_mut()
            && let Some(pane) = view.panes.get_mut(view.active_pane)
        {
            if !view.buffer_ids.contains(&buffer_id) {
                view.buffer_ids.push(buffer_id);
            }
            pane.buffer_id = buffer_id;
        }
        self.restore_active_pane_view_state();
        self.restore_active_buffer_vim_state();
    }

    fn focus_buffer(&mut self, buffer_id: BufferId) {
        self.focus_buffer_in_active_pane(buffer_id);
        self.close_picker();
        self.close_autocomplete();
        self.close_hover();
    }

    fn split_pane(&mut self, pane_id: PaneId, buffer_id: BufferId, direction: PaneSplitDirection) {
        self.persist_active_pane_view_state();
        let initial_view_state = self
            .buffer(buffer_id)
            .map(ShellBuffer::view_state)
            .unwrap_or(BufferViewState {
                cursor: TextPoint::default(),
                scroll_row: 0,
                scroll_col: 0,
            });
        if let Some(view) = self.workspace_view_mut()
            && view.panes.len() == 1
        {
            if !view.buffer_ids.contains(&buffer_id) {
                view.buffer_ids.push(buffer_id);
            }
            view.panes
                .push(ShellPane::new(pane_id, buffer_id, initial_view_state));
            view.split_direction = Some(direction);
        }
    }

    fn close_pane(&mut self, pane_id: PaneId) {
        self.persist_active_pane_view_state();
        self.persist_active_buffer_vim_state();
        if let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
            && let Some(index) = view.panes.iter().position(|pane| pane.pane_id == pane_id)
        {
            view.panes.remove(index);
            if view.panes.len() == 1 {
                view.split_direction = None;
                view.golden_ratio_override = None;
                view.pane_size_weights = None;
            }
            if index < view.active_pane {
                view.active_pane = view.active_pane.saturating_sub(1);
            } else if index == view.active_pane {
                view.active_pane = view.active_pane.min(view.panes.len().saturating_sub(1));
            }
        }
        self.restore_active_pane_view_state();
        self.restore_active_buffer_vim_state();
        self.close_autocomplete();
        self.close_hover();
    }

    fn switch_split(&mut self) -> bool {
        let active_pane_id = if let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
        {
            let active_pane_id = view.panes.get(view.active_pane).map(|pane| pane.pane_id);
            view.panes.reverse();
            if let Some(active_pane_id) = active_pane_id
                && let Some(index) = view
                    .panes
                    .iter()
                    .position(|pane| pane.pane_id == active_pane_id)
            {
                view.active_pane = index;
            }
            active_pane_id
        } else {
            None
        };
        if active_pane_id.is_none() {
            return false;
        }
        self.close_autocomplete();
        self.close_hover();
        true
    }

    fn shift_active_pane(&mut self, delta: isize) -> Option<PaneId> {
        self.persist_active_pane_view_state();
        self.persist_active_buffer_vim_state();
        if !self.picker_visible()
            && let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
        {
            let pane_count = view.panes.len() as isize;
            let next = (view.active_pane as isize + delta).rem_euclid(pane_count);
            view.active_pane = next as usize;
        }
        self.restore_active_pane_view_state();
        self.restore_active_buffer_vim_state();
        self.active_pane_id()
    }

    fn cycle_active_pane(&mut self) -> Option<PaneId> {
        self.persist_active_pane_view_state();
        self.persist_active_buffer_vim_state();
        if !self.picker_visible()
            && let Some(view) = self.workspace_view_mut()
            && view.panes.len() > 1
        {
            view.active_pane = (view.active_pane + 1) % view.panes.len();
        }
        self.restore_active_pane_view_state();
        self.restore_active_buffer_vim_state();
        self.active_pane_id()
    }
}
