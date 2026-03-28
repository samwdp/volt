#![doc = r#"Shared extension-facing types used by the core editor and the compiled user library."#]

use abi_stable::{
    StableAbi,
    std_types::{ROption, RString, RVec},
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
}

#[cfg(test)]
mod tests {
    use super::{
        PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginKeyBinding,
        PluginKeymapScope, PluginPackage, PluginVimMode,
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
    }
}
