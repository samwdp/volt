use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    plugin_hooks,
};

pub const ACP_BUFFER_KIND: &str = "acp";

#[derive(Debug, Clone)]
pub struct AcpClientConfig {
    pub id: String,
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<String>,
}

impl AcpClientConfig {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        command: impl Into<String>,
        args: &[&str],
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            command: command.into(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            env: Vec::new(),
            cwd: None,
        }
    }
}

/// Returns ACP client configurations compiled into the user package.
pub fn clients() -> Vec<AcpClientConfig> {
    vec![
        AcpClientConfig::new(
            "copilot",
            "GitHub Copilot (ACP)",
            "copilot",
            &["--acp", "--stdio", "--yolo"],
        ),
        AcpClientConfig::new("opencode", "OpenCode (ACP)", "opencode", &["acp"]),
    ]
}

pub fn client_by_id(id: &str) -> Option<AcpClientConfig> {
    clients().into_iter().find(|client| client.id == id)
}

/// Returns the metadata for ACP commands.
pub fn package() -> PluginPackage {
    let commands = vec![
        hook_command(
            "acp.pick-client",
            "Opens the ACP client picker.",
            "ui.picker.open",
            Some("acp-clients"),
        ),
        hook_command(
            "acp.pick-session",
            "Opens the ACP session picker for the active client.",
            "ui.acp.pick-session",
            None,
        ),
        hook_command(
            "acp.new-session",
            "Creates a new ACP session for the active client in a new buffer.",
            "ui.acp.new-session",
            None,
        ),
        hook_command(
            "acp.pick-mode",
            "Opens the ACP mode picker for the active session.",
            "ui.acp.pick-mode",
            None,
        ),
        hook_command(
            "acp.pick-model",
            "Opens the ACP model picker for the active session.",
            "ui.acp.pick-model",
            None,
        ),
        hook_command(
            "acp.cycle-mode",
            "Cycles to the next ACP session mode.",
            "ui.acp.cycle-mode",
            None,
        ),
        hook_command(
            "acp.switch-pane",
            "Switches focus between the ACP plan and output panes.",
            plugin_hooks::SWITCH_PANE,
            None,
        ),
        hook_command(
            "acp.complete-slash",
            "Opens ACP slash command completion.",
            "ui.acp.complete-slash",
            None,
        ),
        hook_command(
            "acp.disconnect",
            "Disconnects the active ACP client.",
            "ui.acp.disconnect",
            None,
        ),
        hook_command(
            "acp.permission-approve",
            "Approves the latest ACP permission request.",
            "ui.acp.permission-approve",
            None,
        ),
        hook_command(
            "acp.permission-deny",
            "Denies the latest ACP permission request.",
            "ui.acp.permission-deny",
            None,
        ),
    ];

    let key_bindings = vec![
        PluginKeyBinding::new("Shift+Tab", "acp.cycle-mode", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new("Ctrl+Tab", "acp.switch-pane", PluginKeymapScope::Workspace),
        PluginKeyBinding::new("Ctrl+m", "acp.pick-model", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("Ctrl+s", "acp.pick-session", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
    ];

    PluginPackage::new("acp", true, "Agent Client Protocol integrations.")
        .with_commands(commands)
        .with_key_bindings(key_bindings)
}

fn hook_command(
    name: &str,
    description: &str,
    hook_name: &str,
    detail: Option<&str>,
) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, detail)],
    )
}
