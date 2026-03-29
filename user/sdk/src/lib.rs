#![doc = r#"Shared extension-facing types used by the core editor and the compiled user library."#]

pub mod abi;

use abi_stable::{
    StableAbi,
    std_types::{ROption, RString, RVec},
};

pub use editor_core::{Section, SectionAction, SectionItem, SectionTree};
pub use editor_dap::DebugAdapterSpec;
pub use editor_fs::{DirectoryEntry, DirectoryEntryKind, ProjectSearchRoot};
pub use editor_git::{GitStatusSnapshot, StatusEntry};
pub use editor_icons::{IconFontCategory, IconFontSymbol};
pub use editor_lsp::{LanguageServerRootStrategy, LanguageServerSpec, LspCompletionKind};
pub use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};
pub use editor_theme::{Color, Theme, ThemeOption};

pub use editor_icons::symbols;
pub use abi::{
    AbiAcpClient, AbiAutocompleteProvider, AbiCaptureThemeMapping, AbiColor,
    AbiDebugAdapterSpec, AbiDirectoryEntry, AbiDirectoryEntryKind, AbiGitLogEntry,
    AbiGitStashEntry, AbiGitStatusPrefix, AbiGitStatusSnapshot, AbiHoverProvider,
    AbiIconFontCategory, AbiIconFontSymbol, AbiLanguageConfiguration,
    AbiLanguageServerRootStrategy, AbiLanguageServerSpec, AbiLspDiagnosticsInfo,
    AbiOilDefaults, AbiOilKeyAction, AbiOilKeybindings, AbiOilSortMode, AbiSection,
    AbiSectionAction, AbiSectionItem, AbiSectionTree, AbiStatusEntry, AbiStatuslineContext,
    AbiStringPair, AbiTerminalConfig, AbiTheme, AbiThemeOption, AbiThemeOptionEntry,
    AbiThemeToken, AbiWorkspaceRoot, UserLibraryModule, UserLibraryModuleRef,
};

// ─── Protocol hook name constants ───────────────────────────────────────────
//
// These string identifiers form the stable "protocol" between the host editor
// and the compiled user library.  Keeping them here means both sides always
// agree on the exact identifier without the host depending on the user crate.

/// Hook name constants for the autocomplete subsystem.
pub mod autocomplete_hooks {
    pub const TRIGGER: &str = "ui.autocomplete.trigger";
    pub const NEXT: &str = "ui.autocomplete.next";
    pub const PREVIOUS: &str = "ui.autocomplete.previous";
    pub const ACCEPT: &str = "ui.autocomplete.accept";
    pub const CANCEL: &str = "ui.autocomplete.cancel";
}

/// Hook name constants for the hover subsystem.
pub mod hover_hooks {
    pub const TOGGLE: &str = "ui.hover.toggle";
    pub const FOCUS: &str = "ui.hover.focus";
    pub const NEXT: &str = "ui.hover.next";
    pub const PREVIOUS: &str = "ui.hover.previous";
}

/// Hook name constants for the LSP subsystem.
pub mod lsp_hooks {
    pub const START: &str = "lsp.server-start";
    pub const STOP: &str = "lsp.server-stop";
    pub const RESTART: &str = "lsp.server-restart";
    pub const LOG: &str = "lsp.open-log";
    pub const DEFINITION: &str = "lsp.goto-definition";
    pub const REFERENCES: &str = "lsp.goto-references";
    pub const IMPLEMENTATION: &str = "lsp.goto-implementation";
}

/// Hook name constants for the git subsystem.
pub mod git_hooks {
    pub const STATUS_OPEN_POPUP: &str = "ui.git.status-open-popup";
    pub const DIFF_OPEN: &str = "ui.git.diff-open";
    pub const LOG_OPEN: &str = "ui.git.log-open";
    pub const STASH_LIST_OPEN: &str = "ui.git.stash-list-open";
}

/// Hook name constants for the oil directory browser.
pub mod oil_hooks {
    pub const OPEN: &str = "ui.oil.open";
    pub const OPEN_PARENT: &str = "ui.oil.open-parent";
}

/// Hook name constants for the browser buffer.
pub mod browser_hooks {
    pub const URL: &str = "ui.browser.url";
}

// ─── Buffer kind string constants ────────────────────────────────────────────

/// Buffer kind strings used when creating or matching plugin buffers.
pub mod buffer_kinds {
    pub const GIT_STATUS: &str = "git-status";
    pub const GIT_COMMIT: &str = "git-commit";
    pub const GIT_DIFF: &str = "git-diff";
    pub const GIT_LOG: &str = "git-log";
    pub const GIT_STASH: &str = "git-stash";
    pub const ACP: &str = "acp";
    pub const BROWSER: &str = "browser";
    pub const CALCULATOR: &str = "calculator";
}

/// Generic split-pane metadata for plugin buffers that want an editable input
/// area plus a dedicated read-only output pane rendered by the host.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginBufferSections {
    input_title: RString,
    output_title: RString,
    output_min_rows: usize,
    output_initial_lines: RVec<RString>,
}

impl PluginBufferSections {
    /// Creates a two-pane plugin buffer configuration.
    pub fn new(
        input_title: impl Into<RString>,
        output_title: impl Into<RString>,
        output_min_rows: usize,
        output_initial_lines: Vec<impl Into<RString>>,
    ) -> Self {
        Self {
            input_title: input_title.into(),
            output_title: output_title.into(),
            output_min_rows: output_min_rows.max(1),
            output_initial_lines: output_initial_lines
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        }
    }

    /// Returns the title shown in the editable input pane.
    pub fn input_title(&self) -> &str {
        self.input_title.as_str()
    }

    /// Returns the title shown in the read-only output pane.
    pub fn output_title(&self) -> &str {
        self.output_title.as_str()
    }

    /// Returns the minimum number of wrapped rows reserved for the output pane.
    pub fn output_min_rows(&self) -> usize {
        self.output_min_rows
    }

    /// Returns the initial output lines shown before the first evaluation.
    pub fn output_initial_lines(&self) -> &[RString] {
        self.output_initial_lines.as_slice()
    }
}

/// Declares a plugin-owned buffer kind and the host behavior it needs.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginBuffer {
    kind: RString,
    initial_lines: RVec<RString>,
    sections: ROption<PluginBufferSections>,
    evaluate_handler: ROption<RString>,
}

/// Context passed to the user library when rendering the statusline.
#[derive(Debug, Clone, Copy)]
pub struct StatuslineContext<'a> {
    pub vim_mode: &'a str,
    pub recording_macro: Option<char>,
    pub workspace_name: &'a str,
    pub buffer_name: &'a str,
    pub buffer_modified: bool,
    pub language_id: Option<&'a str>,
    pub line: usize,
    pub column: usize,
    pub lsp_server: Option<&'a str>,
    pub lsp_diagnostics: Option<LspDiagnosticsInfo>,
    pub acp_connected: bool,
    pub git_branch: Option<&'a str>,
    pub git_added: usize,
    pub git_removed: usize,
}

/// Stable contract implemented by the compiled user extension library.
pub trait UserLibrary: Send + Sync {
    fn packages(&self) -> Vec<PluginPackage>;
    fn themes(&self) -> Vec<Theme>;
    fn syntax_languages(&self) -> Vec<LanguageConfiguration>;
    fn language_servers(&self) -> Vec<LanguageServerSpec>;
    fn debug_adapters(&self) -> Vec<DebugAdapterSpec>;
    fn autocomplete_providers(&self) -> Vec<AutocompleteProvider>;
    fn autocomplete_result_limit(&self) -> usize;
    fn autocomplete_token_icon(&self) -> &'static str;
    fn hover_providers(&self) -> Vec<HoverProvider>;
    fn hover_line_limit(&self) -> usize;
    fn hover_token_icon(&self) -> &'static str;
    fn hover_signature_icon(&self) -> &'static str;
    fn acp_clients(&self) -> Vec<AcpClient>;
    fn acp_client_by_id(&self, id: &str) -> Option<AcpClient>;
    fn workspace_roots(&self) -> Vec<WorkspaceRoot>;
    fn terminal_config(&self) -> TerminalConfig;
    fn oil_defaults(&self) -> OilDefaults;
    fn oil_keybindings(&self) -> OilKeybindings;
    fn oil_keydown_action(&self, chord: &str) -> Option<OilKeyAction>;
    fn oil_chord_action(&self, had_prefix: bool, chord: &str) -> Option<OilKeyAction>;
    fn oil_help_lines(&self) -> Vec<String>;
    fn oil_directory_sections(
        &self,
        root: &std::path::Path,
        entries: &[DirectoryEntry],
        show_hidden: bool,
        sort_mode: OilSortMode,
        trash_enabled: bool,
    ) -> SectionTree;
    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str;
    fn git_status_sections(&self, snapshot: &GitStatusSnapshot) -> SectionTree;
    fn git_commit_template(&self) -> Vec<String>;
    fn git_prefix_for_chord(&self, chord: &str) -> Option<GitStatusPrefix>;
    fn git_command_for_chord(
        &self,
        prefix: Option<GitStatusPrefix>,
        chord: &str,
    ) -> Option<&'static str>;
    fn browser_buffer_lines(&self, url: Option<&str>) -> Vec<String>;
    fn browser_input_hint(&self, url: Option<&str>) -> String;
    fn browser_url_prompt(&self) -> String;
    fn browser_url_placeholder(&self) -> String;
    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String;
    fn statusline_lsp_connected_icon(&self) -> &'static str;
    fn statusline_lsp_error_icon(&self) -> &'static str;
    fn statusline_lsp_warning_icon(&self) -> &'static str;
    fn lsp_diagnostic_icon(&self) -> &'static str;
    fn lsp_diagnostic_line_limit(&self) -> usize;
    fn lsp_show_buffer_diagnostics(&self) -> bool;
    fn gitfringe_token_added(&self) -> &'static str;
    fn gitfringe_token_modified(&self) -> &'static str;
    fn gitfringe_token_removed(&self) -> &'static str;
    fn gitfringe_symbol(&self) -> &'static str;
    fn icon_symbols(&self) -> &'static [IconFontSymbol];
    fn supports_plugin_evaluate(&self, kind: &str) -> bool {
        self.plugin_buffer(kind)
            .and_then(|buffer| buffer.evaluate_handler().map(str::to_owned))
            .is_some()
    }
    fn handle_plugin_evaluate(&self, kind: &str, input: &str) -> Vec<String> {
        match self
            .plugin_buffer(kind)
            .and_then(|buffer| buffer.evaluate_handler().map(str::to_owned))
        {
            Some(handler) => self.run_plugin_buffer_evaluator(&handler, input),
            None => vec![format!("no evaluator registered for plugin kind `{kind}`")],
        }
    }
    fn plugin_buffer_initial_lines(&self, kind: &str) -> Vec<String> {
        self.plugin_buffer(kind)
            .map(|buffer| {
                buffer
                    .initial_lines()
                    .iter()
                    .map(|line| line.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
    fn plugin_buffer_sections(&self, kind: &str) -> Option<PluginBufferSections> {
        self.plugin_buffer(kind)
            .and_then(|buffer| buffer.sections().cloned())
    }
    fn run_plugin_buffer_evaluator(&self, handler: &str, input: &str) -> Vec<String>;
    fn plugin_buffer(&self, kind: &str) -> Option<PluginBuffer> {
        self.packages()
            .into_iter()
            .find_map(|package| package.buffer(kind).cloned())
    }
    fn default_build_command(&self, language: &str) -> Option<String>;
}

impl PluginBuffer {
    /// Creates a new plugin buffer declaration for the given kind.
    pub fn new(kind: impl Into<RString>, initial_lines: Vec<impl Into<RString>>) -> Self {
        Self {
            kind: kind.into(),
            initial_lines: initial_lines
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            sections: ROption::RNone,
            evaluate_handler: ROption::RNone,
        }
    }

    /// Attaches split-pane metadata for the buffer.
    pub fn with_sections(mut self, sections: PluginBufferSections) -> Self {
        self.sections = ROption::RSome(sections);
        self
    }

    /// Declares the evaluator handler id used when `plugin.evaluate` fires.
    pub fn with_evaluate_handler(mut self, handler: impl Into<RString>) -> Self {
        self.evaluate_handler = ROption::RSome(handler.into());
        self
    }

    /// Returns the plugin buffer kind.
    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }

    /// Returns the initial text content for the buffer.
    pub fn initial_lines(&self) -> &[RString] {
        self.initial_lines.as_slice()
    }

    /// Returns the optional split-pane metadata for this buffer.
    pub fn sections(&self) -> Option<&PluginBufferSections> {
        match &self.sections {
            ROption::RSome(sections) => Some(sections),
            ROption::RNone => None,
        }
    }

    /// Returns the optional evaluator handler id for this buffer.
    pub fn evaluate_handler(&self) -> Option<&str> {
        match &self.evaluate_handler {
            ROption::RSome(handler) => Some(handler.as_str()),
            ROption::RNone => None,
        }
    }
}

// ─── Generic plugin hooks ─────────────────────────────────────────────────────

/// Hook names owned by the host application's generic plugin infrastructure.
/// User plugins emit these hooks; the host handles them without needing to know
/// which specific plugin fired them.
pub mod plugin_hooks {
    /// Emitted by any user plugin that wants the host to evaluate the active
    /// buffer's input section and write the result to the output section.
    /// The host calls `UserLibrary::handle_plugin_evaluate` (defined in
    /// `editor-plugin-host`) with the active buffer's kind string.
    /// The separator line that divides input from output is
    /// [`EVALUATE_SEPARATOR_PREFIX`].
    pub const EVALUATE: &str = "plugin.evaluate";

    /// A line whose text starts with this prefix is treated as the output
    /// separator in an evaluatable plugin buffer.
    pub const EVALUATE_SEPARATOR_PREFIX: &str = "─── Output";

    /// Emitted when a plugin wants the host to run a build/compile command.
    /// Detail format: `{language}` (e.g. `"rust"`).  The host looks up the
    /// default command via `UserLibrary::default_build_command`, opens a
    /// `*compile <workspace>*` popup buffer with an input field pre-filled
    /// with the default, and runs the command on Ctrl+Enter.
    pub const RUN_COMMAND: &str = "plugin.run-command";

    /// Emitted when a plugin wants the host to re-run the last build command
    /// for the active workspace.  If no command has been run yet the host
    /// falls back to [`RUN_COMMAND`].
    pub const RERUN_COMMAND: &str = "plugin.rerun-command";

    /// Emitted when a plugin wants the host to switch focus between its
    /// currently active split panes (for example between input and output).
    pub const SWITCH_PANE: &str = "plugin.switch-pane";
}

// ─── Git action / section ID constants ───────────────────────────────────────

/// Section action IDs for the git status buffer.
pub mod git_actions {
    pub const STAGE_FILE: &str = "git.stage-file";
    pub const STAGE_ALL: &str = "git.stage-all";
    pub const UNSTAGE_FILE: &str = "git.unstage-file";
    pub const COMMIT_OPEN: &str = "git.commit-open";
    pub const PUSH: &str = "git.push";
    pub const SHOW_COMMIT: &str = "git.show-commit";
    pub const SHOW_STASH: &str = "git.show-stash";
}

/// Section IDs used in the git status buffer tree.
pub mod git_sections {
    pub const HEADERS: &str = "git.status.headers";
    pub const IN_PROGRESS: &str = "git.status.in-progress";
    pub const STAGED: &str = "git.status.staged";
    pub const UNSTAGED: &str = "git.status.unstaged";
    pub const UNTRACKED: &str = "git.status.untracked";
    pub const STASHES: &str = "git.status.stashes";
    pub const UNPULLED: &str = "git.status.unpulled";
    pub const UNPUSHED: &str = "git.status.unpushed";
    pub const REMOTE: &str = "git.status.remote";
    pub const COMMIT: &str = "git.status.commit";
}

// ─── Oil directory browser constants ─────────────────────────────────────────

/// Section / action ID constants for the oil directory browser.
pub mod oil_protocol {
    pub const ACTION_OIL_ENTRY: &str = "oil.entry";
    pub const SECTION_OIL_DIRECTORY: &str = "oil.directory";
}

// ─── Shared configuration types ──────────────────────────────────────────────

/// User-configurable sort mode for oil directory buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OilSortMode {
    TypeThenName,
    TypeThenNameDesc,
}

impl OilSortMode {
    /// Returns the human-readable label shown in the oil buffer header.
    pub fn label(self) -> &'static str {
        match self {
            Self::TypeThenName => "type+name",
            Self::TypeThenNameDesc => "type+name desc",
        }
    }

    /// Returns the next mode in the cycle used by the oil UI.
    pub fn cycle(self) -> Self {
        match self {
            Self::TypeThenName => Self::TypeThenNameDesc,
            Self::TypeThenNameDesc => Self::TypeThenName,
        }
    }
}

/// An action resolved from an oil key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OilKeyAction {
    OpenEntry,
    OpenVerticalSplit,
    OpenHorizontalSplit,
    OpenNewPane,
    PreviewEntry,
    Refresh,
    Close,
    StartPrefix,
    OpenParent,
    OpenWorkspaceRoot,
    SetRoot,
    ShowHelp,
    CycleSort,
    ToggleHidden,
    ToggleTrash,
    OpenExternal,
    SetTabLocalRoot,
}

/// User-configurable default options for new oil directory buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OilDefaults {
    pub show_hidden: bool,
    pub sort_mode: OilSortMode,
    pub trash_enabled: bool,
}

/// User-configurable keybindings for the oil directory browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OilKeybindings {
    pub open_entry: &'static str,
    pub open_vertical_split: &'static str,
    pub open_horizontal_split: &'static str,
    pub open_new_pane: &'static str,
    pub preview_entry: &'static str,
    pub refresh: &'static str,
    pub close: &'static str,
    pub prefix: &'static str,
    pub open_parent: &'static str,
    pub open_workspace_root: &'static str,
    pub set_root: &'static str,
    pub show_help: &'static str,
    pub cycle_sort: &'static str,
    pub toggle_hidden: &'static str,
    pub toggle_trash: &'static str,
    pub open_external: &'static str,
    pub set_tab_local_root: &'static str,
}

/// Git key-chord action prefix kind used for file-scoped git commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatusPrefix {
    Commit,
    Push,
    Fetch,
    Pull,
    Branch,
    Diff,
    Log,
    Stash,
    Merge,
    Rebase,
    CherryPick,
    Revert,
    Reset,
}

/// Autocomplete provider configuration exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutocompleteProvider {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub item_icon: String,
    pub or_group: Option<String>,
}

/// Hover provider configuration exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverProvider {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub line_limit: usize,
}

/// ACP client configuration exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpClient {
    pub id: String,
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<String>,
}

/// Project search root exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRoot {
    pub path: String,
    pub max_depth: usize,
}

/// Terminal configuration exported by the user library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalConfig {
    pub program: String,
    pub args: Vec<String>,
}

/// LSP diagnostic counts surfaced to the statusline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspDiagnosticsInfo {
    pub errors: usize,
    pub warnings: usize,
}

/// Stable keymap scopes shared across the host and the compiled user library.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PluginKeymapScope {
    /// Binding is active globally.
    Global,
    /// Binding is active in workspace-focused contexts.
    Workspace,
    /// Binding is active while a popup is focused.
    Popup,
}

/// Modal Vim state that can activate a keybinding.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PluginVimMode {
    /// Binding is always active regardless of the current Vim mode.
    Any,
    /// Binding is active while Vim normal mode is focused.
    Normal,
    /// Binding is active while Vim insert mode is focused.
    Insert,
    /// Binding is active while Vim visual mode is focused.
    Visual,
}

/// A command exported by a user package.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginCommand {
    name: RString,
    description: RString,
    actions: RVec<PluginAction>,
}

impl PluginCommand {
    /// Creates a new exported command.
    pub fn new(
        name: impl Into<RString>,
        description: impl Into<RString>,
        actions: Vec<PluginAction>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            actions: actions.into(),
        }
    }

    /// Returns the command identifier.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the command summary.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the actions performed when the command executes.
    pub fn actions(&self) -> &[PluginAction] {
        self.actions.as_slice()
    }
}

/// Action tags supported by the stable plugin ABI.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum PluginActionKind {
    /// Write a diagnostic message through the host.
    LogMessage,
    /// Create or surface a buffer.
    OpenBuffer,
    /// Emit a hook event.
    EmitHook,
}

/// Describes how a buffer should be opened by the host runtime.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginBufferAction {
    buffer_name: RString,
    buffer_kind: RString,
    popup_title: ROption<RString>,
}

impl PluginBufferAction {
    /// Creates a new buffer-open action.
    pub fn new(
        buffer_name: impl Into<RString>,
        buffer_kind: impl Into<RString>,
        popup_title: Option<impl Into<RString>>,
    ) -> Self {
        Self {
            buffer_name: buffer_name.into(),
            buffer_kind: buffer_kind.into(),
            popup_title: match popup_title {
                Some(title) => ROption::RSome(title.into()),
                None => ROption::RNone,
            },
        }
    }

    /// Returns the target buffer name.
    pub fn buffer_name(&self) -> &str {
        self.buffer_name.as_str()
    }

    /// Returns the buffer kind tag consumed by the host.
    pub fn buffer_kind(&self) -> &str {
        self.buffer_kind.as_str()
    }

    /// Returns the popup title, if the buffer should open in a popup.
    pub fn popup_title(&self) -> Option<&str> {
        match &self.popup_title {
            ROption::RSome(title) => Some(title.as_str()),
            ROption::RNone => None,
        }
    }
}

/// Describes how a hook event should be emitted by the host runtime.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginEmitHookAction {
    hook_name: RString,
    detail: ROption<RString>,
}

impl PluginEmitHookAction {
    /// Creates a new hook emission action.
    pub fn new(hook_name: impl Into<RString>, detail: Option<impl Into<RString>>) -> Self {
        Self {
            hook_name: hook_name.into(),
            detail: match detail {
                Some(detail) => ROption::RSome(detail.into()),
                None => ROption::RNone,
            },
        }
    }

    /// Returns the hook identifier.
    pub fn hook_name(&self) -> &str {
        self.hook_name.as_str()
    }

    /// Returns the optional event detail.
    pub fn detail(&self) -> Option<&str> {
        match &self.detail {
            ROption::RSome(detail) => Some(detail.as_str()),
            ROption::RNone => None,
        }
    }
}

/// Stable action payload used by exported commands.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginAction {
    kind: PluginActionKind,
    message: ROption<RString>,
    buffer: ROption<PluginBufferAction>,
    hook: ROption<PluginEmitHookAction>,
}

impl PluginAction {
    /// Creates a logging action.
    pub fn log_message(message: impl Into<RString>) -> Self {
        Self {
            kind: PluginActionKind::LogMessage,
            message: ROption::RSome(message.into()),
            buffer: ROption::RNone,
            hook: ROption::RNone,
        }
    }

    /// Creates a buffer-opening action.
    pub fn open_buffer(
        buffer_name: impl Into<RString>,
        buffer_kind: impl Into<RString>,
        popup_title: Option<impl Into<RString>>,
    ) -> Self {
        Self {
            kind: PluginActionKind::OpenBuffer,
            message: ROption::RNone,
            buffer: ROption::RSome(PluginBufferAction::new(
                buffer_name,
                buffer_kind,
                popup_title,
            )),
            hook: ROption::RNone,
        }
    }

    /// Creates a hook-emission action.
    pub fn emit_hook(hook_name: impl Into<RString>, detail: Option<impl Into<RString>>) -> Self {
        Self {
            kind: PluginActionKind::EmitHook,
            message: ROption::RNone,
            buffer: ROption::RNone,
            hook: ROption::RSome(PluginEmitHookAction::new(hook_name, detail)),
        }
    }

    /// Returns the action kind.
    pub const fn kind(&self) -> PluginActionKind {
        self.kind
    }

    /// Returns the log message payload when present.
    pub fn message(&self) -> Option<&str> {
        match &self.message {
            ROption::RSome(message) => Some(message.as_str()),
            ROption::RNone => None,
        }
    }

    /// Returns the buffer payload when present.
    pub fn buffer(&self) -> Option<&PluginBufferAction> {
        match &self.buffer {
            ROption::RSome(buffer) => Some(buffer),
            ROption::RNone => None,
        }
    }

    /// Returns the hook payload when present.
    pub fn hook(&self) -> Option<&PluginEmitHookAction> {
        match &self.hook {
            ROption::RSome(hook) => Some(hook),
            ROption::RNone => None,
        }
    }
}

/// Metadata for a keybinding exported by a user package.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginKeyBinding {
    chord: RString,
    command_name: RString,
    scope: PluginKeymapScope,
    vim_mode: PluginVimMode,
}

impl PluginKeyBinding {
    /// Creates a new keybinding.
    pub fn new(
        chord: impl Into<RString>,
        command_name: impl Into<RString>,
        scope: PluginKeymapScope,
    ) -> Self {
        Self {
            chord: chord.into(),
            command_name: command_name.into(),
            scope,
            vim_mode: PluginVimMode::Any,
        }
    }

    /// Sets the Vim mode that activates the binding.
    pub fn with_vim_mode(mut self, vim_mode: PluginVimMode) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Returns the key chord.
    pub fn chord(&self) -> &str {
        self.chord.as_str()
    }

    /// Returns the command targeted by the keybinding.
    pub fn command_name(&self) -> &str {
        self.command_name.as_str()
    }

    /// Returns the scope that activates the keybinding.
    pub const fn scope(&self) -> PluginKeymapScope {
        self.scope
    }

    /// Returns the Vim mode that activates the binding.
    pub const fn vim_mode(&self) -> PluginVimMode {
        self.vim_mode
    }
}

/// Declares a custom hook exported by a user package.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginHookDeclaration {
    name: RString,
    description: RString,
}

impl PluginHookDeclaration {
    /// Creates a new custom hook declaration.
    pub fn new(name: impl Into<RString>, description: impl Into<RString>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }

    /// Returns the hook name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the hook description.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

/// Subscribes a package command to a hook.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginHookBinding {
    hook_name: RString,
    subscriber: RString,
    command_name: RString,
    detail_filter: ROption<RString>,
}

impl PluginHookBinding {
    /// Creates a new hook subscription.
    pub fn new(
        hook_name: impl Into<RString>,
        subscriber: impl Into<RString>,
        command_name: impl Into<RString>,
        detail_filter: Option<impl Into<RString>>,
    ) -> Self {
        Self {
            hook_name: hook_name.into(),
            subscriber: subscriber.into(),
            command_name: command_name.into(),
            detail_filter: match detail_filter {
                Some(filter) => ROption::RSome(filter.into()),
                None => ROption::RNone,
            },
        }
    }

    /// Returns the subscribed hook name.
    pub fn hook_name(&self) -> &str {
        self.hook_name.as_str()
    }

    /// Returns the subscriber identifier.
    pub fn subscriber(&self) -> &str {
        self.subscriber.as_str()
    }

    /// Returns the command that should run when the hook fires.
    pub fn command_name(&self) -> &str {
        self.command_name.as_str()
    }

    /// Returns the optional detail filter.
    pub fn detail_filter(&self) -> Option<&str> {
        match &self.detail_filter {
            ROption::RSome(filter) => Some(filter.as_str()),
            ROption::RNone => None,
        }
    }
}

/// Metadata advertised by a user package to the core host.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginPackage {
    name: RString,
    auto_load: bool,
    description: RString,
    commands: RVec<PluginCommand>,
    key_bindings: RVec<PluginKeyBinding>,
    hook_declarations: RVec<PluginHookDeclaration>,
    hook_bindings: RVec<PluginHookBinding>,
    buffers: RVec<PluginBuffer>,
}

impl PluginPackage {
    /// Creates a new package metadata record.
    pub fn new(name: impl Into<RString>, auto_load: bool, description: impl Into<RString>) -> Self {
        Self {
            name: name.into(),
            auto_load,
            description: description.into(),
            commands: RVec::new(),
            key_bindings: RVec::new(),
            hook_declarations: RVec::new(),
            hook_bindings: RVec::new(),
            buffers: RVec::new(),
        }
    }

    /// Adds exported commands to the package.
    pub fn with_commands(mut self, commands: Vec<PluginCommand>) -> Self {
        self.commands = commands.into();
        self
    }

    /// Adds exported keybindings to the package.
    pub fn with_key_bindings(mut self, key_bindings: Vec<PluginKeyBinding>) -> Self {
        self.key_bindings = key_bindings.into();
        self
    }

    /// Adds custom hook declarations to the package.
    pub fn with_hook_declarations(mut self, hook_declarations: Vec<PluginHookDeclaration>) -> Self {
        self.hook_declarations = hook_declarations.into();
        self
    }

    /// Adds hook subscriptions to the package.
    pub fn with_hook_bindings(mut self, hook_bindings: Vec<PluginHookBinding>) -> Self {
        self.hook_bindings = hook_bindings.into();
        self
    }

    /// Adds plugin-owned buffer declarations to the package.
    pub fn with_buffers(mut self, buffers: Vec<PluginBuffer>) -> Self {
        self.buffers = buffers.into();
        self
    }

    /// Returns the package identifier.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns whether the package should be loaded automatically at startup.
    pub const fn auto_load(&self) -> bool {
        self.auto_load
    }

    /// Returns the package summary.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the exported commands.
    pub fn commands(&self) -> &[PluginCommand] {
        self.commands.as_slice()
    }

    /// Returns the exported keybindings.
    pub fn key_bindings(&self) -> &[PluginKeyBinding] {
        self.key_bindings.as_slice()
    }

    /// Returns the custom hook declarations.
    pub fn hook_declarations(&self) -> &[PluginHookDeclaration] {
        self.hook_declarations.as_slice()
    }

    /// Returns the hook subscriptions.
    pub fn hook_bindings(&self) -> &[PluginHookBinding] {
        self.hook_bindings.as_slice()
    }

    /// Returns the plugin-owned buffer declarations.
    pub fn buffers(&self) -> &[PluginBuffer] {
        self.buffers.as_slice()
    }

    /// Returns the declared plugin buffer for the given kind, if any.
    pub fn buffer(&self, kind: &str) -> Option<&PluginBuffer> {
        self.buffers.iter().find(|buffer| buffer.kind() == kind)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PluginAction, PluginBuffer, PluginBufferSections, PluginCommand, PluginHookBinding,
        PluginHookDeclaration, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    };

    #[test]
    fn plugin_package_constructor_preserves_metadata_and_registrations() {
        let package = PluginPackage::new("lsp", true, "Language server integration.")
            .with_commands(vec![PluginCommand::new(
                "lsp.start",
                "Starts the language server.",
                vec![PluginAction::emit_hook("lsp.startup", Some("rust"))],
            )])
            .with_key_bindings(vec![
                PluginKeyBinding::new("M-x lsp.start", "lsp.start", PluginKeymapScope::Global)
                    .with_vim_mode(PluginVimMode::Normal),
            ])
            .with_hook_declarations(vec![PluginHookDeclaration::new(
                "lsp.startup",
                "Runs after an LSP startup command executes.",
            )])
            .with_buffers(vec![PluginBuffer::new("calculator", vec!["1 + 1"])
                .with_sections(PluginBufferSections::new(
                    "Input",
                    "Output",
                    1,
                    vec!["(press enter)".to_owned()],
                ))
                .with_evaluate_handler("calculator.evaluate")])
            .with_hook_bindings(vec![PluginHookBinding::new(
                "buffer.file-open",
                "lsp.auto-start",
                "lsp.start",
                Some(".rs"),
            )]);

        assert_eq!(package.name(), "lsp");
        assert!(package.auto_load());
        assert_eq!(package.description(), "Language server integration.");
        assert_eq!(package.commands()[0].name(), "lsp.start");
        assert_eq!(package.key_bindings()[0].chord(), "M-x lsp.start");
        assert_eq!(package.key_bindings()[0].vim_mode(), PluginVimMode::Normal);
        assert_eq!(package.hook_declarations()[0].name(), "lsp.startup");
        assert_eq!(package.hook_bindings()[0].detail_filter(), Some(".rs"));
        assert_eq!(package.buffers()[0].kind(), "calculator");
        assert_eq!(
            package.buffers()[0]
                .initial_lines()
                .iter()
                .map(|line| line.as_str())
                .collect::<Vec<_>>(),
            vec!["1 + 1"]
        );
        assert_eq!(
            package.buffers()[0].sections().map(|sections| sections.output_title()),
            Some("Output")
        );
        assert_eq!(
            package.buffers()[0].evaluate_handler(),
            Some("calculator.evaluate")
        );
    }
}
