use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

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
            &["--acp", "--stdio"],
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

    PluginPackage::new("acp", true, "Agent Client Protocol integrations.").with_commands(commands)
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
