use std::{collections::BTreeMap, sync::Arc};

use crate::{BufferId, EditorRuntime, PaneId, PopupId, WindowId, WorkspaceId};

type HookCallback = Arc<dyn Fn(&HookEvent, &mut EditorRuntime) -> Result<(), String> + Send + Sync>;

/// Built-in hook identifiers exposed by the runtime.
pub mod builtins {
    /// Fires when the editor is starting up.
    pub const STARTUP: &str = "core.startup";
    /// Fires after a file-backed buffer is opened.
    pub const FILE_OPEN: &str = "buffer.file-open";
    /// Fires immediately before a save operation begins.
    pub const BEFORE_SAVE: &str = "buffer.before-save";
    /// Fires after a save operation completes.
    pub const AFTER_SAVE: &str = "buffer.after-save";
    /// Fires when the active buffer changes.
    pub const BUFFER_SWITCH: &str = "buffer.switch";
    /// Fires when a workspace is opened.
    pub const WORKSPACE_OPEN: &str = "workspace.open";
    /// Fires when a workspace is closed.
    pub const WORKSPACE_CLOSE: &str = "workspace.close";
    /// Fires after a horizontal pane split.
    pub const PANE_SPLIT_HORIZONTAL: &str = "pane.split-horizontal";
    /// Fires after a vertical pane split.
    pub const PANE_SPLIT_VERTICAL: &str = "pane.split-vertical";
    /// Fires when the active pane changes.
    pub const PANE_SWITCH: &str = "pane.switch";
    /// Fires when a popup opens.
    pub const POPUP_OPEN: &str = "popup.open";
    /// Fires when a popup closes.
    pub const POPUP_CLOSE: &str = "popup.close";
    /// Fires when a compilation job finishes.
    pub const COMPILATION_FINISH: &str = "job.compilation-finish";
    /// Fires when a terminal session exits.
    pub const TERMINAL_EXIT: &str = "terminal.exit";

    pub(super) const DEFINITIONS: [(&str, &str); 14] = [
        (STARTUP, "Runs during editor startup."),
        (FILE_OPEN, "Runs after a file-backed buffer is opened."),
        (BEFORE_SAVE, "Runs immediately before a buffer is saved."),
        (AFTER_SAVE, "Runs after a buffer is saved."),
        (BUFFER_SWITCH, "Runs when the active buffer changes."),
        (WORKSPACE_OPEN, "Runs when a workspace is opened."),
        (WORKSPACE_CLOSE, "Runs when a workspace is closed."),
        (
            PANE_SPLIT_HORIZONTAL,
            "Runs after a horizontal pane split completes.",
        ),
        (
            PANE_SPLIT_VERTICAL,
            "Runs after a vertical pane split completes.",
        ),
        (PANE_SWITCH, "Runs when the active pane changes."),
        (POPUP_OPEN, "Runs when a popup is opened."),
        (POPUP_CLOSE, "Runs when a popup is closed."),
        (
            COMPILATION_FINISH,
            "Runs when a compilation or job-oriented command finishes.",
        ),
        (TERMINAL_EXIT, "Runs when a terminal session exits."),
    ];
}

/// Event payload delivered to hook subscribers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HookEvent {
    /// Related window identifier, if any.
    pub window_id: Option<WindowId>,
    /// Related workspace identifier, if any.
    pub workspace_id: Option<WorkspaceId>,
    /// Related pane identifier, if any.
    pub pane_id: Option<PaneId>,
    /// Related popup identifier, if any.
    pub popup_id: Option<PopupId>,
    /// Related buffer identifier, if any.
    pub buffer_id: Option<BufferId>,
    /// Optional string detail describing the event.
    pub detail: Option<String>,
}

impl HookEvent {
    /// Creates an empty hook event.
    pub fn new() -> Self {
        Self::default()
    }

    /// Annotates the event with a window identifier.
    pub fn with_window(mut self, window_id: WindowId) -> Self {
        self.window_id = Some(window_id);
        self
    }

    /// Annotates the event with a workspace identifier.
    pub fn with_workspace(mut self, workspace_id: WorkspaceId) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    /// Annotates the event with a pane identifier.
    pub fn with_pane(mut self, pane_id: PaneId) -> Self {
        self.pane_id = Some(pane_id);
        self
    }

    /// Annotates the event with a popup identifier.
    pub fn with_popup(mut self, popup_id: PopupId) -> Self {
        self.popup_id = Some(popup_id);
        self
    }

    /// Annotates the event with a buffer identifier.
    pub fn with_buffer(mut self, buffer_id: BufferId) -> Self {
        self.buffer_id = Some(buffer_id);
        self
    }

    /// Annotates the event with a detail string.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Metadata describing a known hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookDefinition {
    name: String,
    description: String,
    built_in: bool,
}

impl HookDefinition {
    fn new(name: String, description: String, built_in: bool) -> Self {
        Self {
            name,
            description,
            built_in,
        }
    }

    /// Returns the hook identifier.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the hook description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns whether the hook is built into the runtime.
    pub const fn is_built_in(&self) -> bool {
        self.built_in
    }
}

#[derive(Clone)]
pub(crate) struct HookSubscription {
    subscriber: String,
    callback: HookCallback,
}

impl HookSubscription {
    pub(crate) fn subscriber(&self) -> &str {
        &self.subscriber
    }

    pub(crate) fn callback(&self) -> HookCallback {
        Arc::clone(&self.callback)
    }
}

/// Registry of hook definitions and subscriptions.
pub struct HookBus {
    definitions: BTreeMap<String, HookDefinition>,
    subscriptions: BTreeMap<String, Vec<HookSubscription>>,
}

impl HookBus {
    /// Creates a hook bus seeded with built-in hooks.
    pub fn new() -> Self {
        let mut bus = Self {
            definitions: BTreeMap::new(),
            subscriptions: BTreeMap::new(),
        };

        for (name, description) in builtins::DEFINITIONS {
            let name = name.to_owned();
            bus.subscriptions.insert(name.clone(), Vec::new());
            bus.definitions.insert(
                name.clone(),
                HookDefinition::new(name, description.to_owned(), true),
            );
        }

        bus
    }

    /// Registers a new custom hook.
    pub fn register_hook(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<(), HookError> {
        let name = name.into();

        if self.definitions.contains_key(&name) {
            return Err(HookError::DuplicateHook(name));
        }

        self.subscriptions.insert(name.clone(), Vec::new());
        self.definitions.insert(
            name.clone(),
            HookDefinition::new(name, description.into(), false),
        );

        Ok(())
    }

    /// Returns whether a hook exists.
    pub fn contains(&self, hook_name: &str) -> bool {
        self.definitions.contains_key(hook_name)
    }

    /// Returns the definition for a hook.
    pub fn definition(&self, hook_name: &str) -> Option<&HookDefinition> {
        self.definitions.get(hook_name)
    }

    /// Returns the number of declared hooks.
    pub fn hook_count(&self) -> usize {
        self.definitions.len()
    }

    /// Returns the total number of hook subscriptions.
    pub fn total_subscription_count(&self) -> usize {
        self.subscriptions.values().map(Vec::len).sum()
    }

    /// Subscribes a callback to an existing hook.
    pub fn subscribe<F>(
        &mut self,
        hook_name: impl Into<String>,
        subscriber: impl Into<String>,
        callback: F,
    ) -> Result<(), HookError>
    where
        F: Fn(&HookEvent, &mut EditorRuntime) -> Result<(), String> + Send + Sync + 'static,
    {
        let hook_name = hook_name.into();
        let subscriber = subscriber.into();
        let subscriptions = self
            .subscriptions
            .get_mut(&hook_name)
            .ok_or_else(|| HookError::UnknownHook(hook_name.clone()))?;

        if subscriptions
            .iter()
            .any(|existing| existing.subscriber == subscriber)
        {
            return Err(HookError::DuplicateSubscription {
                hook: hook_name,
                subscriber,
            });
        }

        subscriptions.push(HookSubscription {
            subscriber,
            callback: Arc::new(callback),
        });

        Ok(())
    }

    pub(crate) fn subscriptions_for(
        &self,
        hook_name: &str,
    ) -> Result<Vec<HookSubscription>, HookError> {
        self.subscriptions
            .get(hook_name)
            .cloned()
            .ok_or_else(|| HookError::UnknownHook(hook_name.to_owned()))
    }
}

impl Default for HookBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors raised by hook registration or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookError {
    /// Attempted to register a hook with an existing name.
    DuplicateHook(String),
    /// Attempted to reference a hook that has not been declared.
    UnknownHook(String),
    /// Attempted to register the same subscriber twice for a hook.
    DuplicateSubscription {
        /// Hook identifier.
        hook: String,
        /// Subscriber identifier.
        subscriber: String,
    },
    /// A hook callback returned a failure message.
    HandlerFailed {
        /// Hook identifier.
        hook: String,
        /// Subscriber identifier.
        subscriber: String,
        /// User-facing failure message.
        message: String,
    },
}

impl std::fmt::Display for HookError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateHook(name) => write!(formatter, "hook `{name}` is already registered"),
            Self::UnknownHook(name) => write!(formatter, "hook `{name}` is not registered"),
            Self::DuplicateSubscription { hook, subscriber } => {
                write!(
                    formatter,
                    "subscriber `{subscriber}` is already attached to hook `{hook}`"
                )
            }
            Self::HandlerFailed {
                hook,
                subscriber,
                message,
            } => write!(
                formatter,
                "hook `{hook}` subscriber `{subscriber}` failed: {message}"
            ),
        }
    }
}

impl std::error::Error for HookError {}
