use editor_plugin_api::{
    AcpActionSpec, AcpPickerContext, AcpPickerItemSpec, AcpPickerKind, PluginAction, PluginBuffer,
    PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode, plugin_hooks,
};

pub const ACP_BUFFER_KIND: &str = "acp";

#[cfg(target_os = "windows")]
pub const PI_ACP_LOCATION: &str = "";

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
pub const PI_ACP_LOCATION: &str = "";

#[cfg(target_os = "macos")]
pub const PI_ACP_LOCATION: &str = "";

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
    crate::config::load()
        .acp
        .clients
        .into_iter()
        .map(|client| AcpClientConfig {
            id: client.id,
            label: client.label,
            command: client.command,
            args: client.args,
            env: client
                .env
                .into_iter()
                .map(|pair| (pair.key, pair.value))
                .collect(),
            cwd: client.cwd,
        })
        .collect()
}

pub fn client_by_id(id: &str) -> Option<AcpClientConfig> {
    clients().into_iter().find(|client| client.id == id)
}

pub fn picker_items(context: &AcpPickerContext) -> Vec<AcpPickerItemSpec> {
    context
        .options
        .iter()
        .map(|option| {
            let detail = acp_picker_detail(option.detail.as_str(), option.current);
            let action = match context.kind {
                AcpPickerKind::Modes => AcpActionSpec::set_mode(option.id.clone()),
                AcpPickerKind::Models => AcpActionSpec::set_model(option.id.clone()),
                AcpPickerKind::Sessions => AcpActionSpec::load_session(option.id.clone()),
                AcpPickerKind::SlashCommands => {
                    AcpActionSpec::insert_slash_command(option.id.clone())
                }
                AcpPickerKind::FileMentions => {
                    AcpActionSpec::insert_file_mention(option.id.clone())
                }
            };
            AcpPickerItemSpec::new(option.id.clone(), option.label.clone(), detail, action)
        })
        .collect()
}

fn acp_picker_detail(detail: &str, current: bool) -> String {
    match (detail.is_empty(), current) {
        (true, true) => "current".to_owned(),
        (false, true) => format!("{detail} | current"),
        (false, false) => detail.to_owned(),
        (true, false) => String::new(),
    }
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
            "acp.focus-input",
            "Focuses the ACP input section and enters insert mode.",
            "ui.acp.focus-input",
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
        .with_buffers(vec![
            PluginBuffer::new(ACP_BUFFER_KIND, Vec::<String>::new()).with_key_bindings(vec![
                PluginKeyBinding::new("I", "acp.focus-input", PluginKeymapScope::Workspace)
                    .with_vim_mode(PluginVimMode::Normal),
            ]),
        ])
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

#[cfg(test)]
mod tests {
    use super::*;
    use editor_plugin_api::AcpPickerOption;

    #[test]
    fn picker_items_mark_current_models() {
        let context =
            AcpPickerContext::new(AcpPickerKind::Models, "ACP Models").with_options(vec![
                AcpPickerOption::new("sonnet", "Claude Sonnet")
                    .with_detail("fast")
                    .with_current(true),
            ]);

        let items = picker_items(&context);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label(), "Claude Sonnet");
        assert_eq!(items[0].detail(), "fast | current");
        assert_eq!(
            items[0].action(),
            &AcpActionSpec::set_model("sonnet".to_owned())
        );
    }

    #[test]
    fn picker_items_preserve_slash_command_labels() {
        let context = AcpPickerContext::new(AcpPickerKind::SlashCommands, "ACP Slash Commands")
            .with_options(vec![
                AcpPickerOption::new("fix", "/fix").with_detail("Fix selected issue"),
            ]);

        let items = picker_items(&context);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id(), "fix");
        assert_eq!(items[0].label(), "/fix");
        assert_eq!(items[0].detail(), "Fix selected issue");
        assert_eq!(
            items[0].action(),
            &AcpActionSpec::insert_slash_command("fix".to_owned())
        );
    }
}
