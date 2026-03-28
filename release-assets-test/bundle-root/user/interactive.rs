use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
};

const INTERACTIVE_READONLY_KIND: &str = "interactive-readonly";
const INTERACTIVE_INPUT_KIND: &str = "interactive-input";

/// Returns the metadata for interactive buffer commands.
pub fn package() -> PluginPackage {
    let commands = vec![
        PluginCommand::new(
            "interactive.open-readonly",
            "Opens an interactive read-only buffer.",
            vec![PluginAction::open_buffer(
                "*interactive*",
                INTERACTIVE_READONLY_KIND,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "interactive.open-input",
            "Opens an interactive input buffer.",
            vec![PluginAction::open_buffer(
                "*interactive-input*",
                INTERACTIVE_INPUT_KIND,
                None::<&str>,
            )],
        ),
        hook_command(
            "interactive.input-submit",
            "Submits the interactive input prompt.",
            "ui.input.submit",
        ),
        hook_command(
            "interactive.input-clear",
            "Clears the interactive input prompt.",
            "ui.input.clear",
        ),
    ];
    let key_bindings = vec![
        PluginKeyBinding::new(
            "Ctrl+Enter",
            "interactive.input-submit",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new(
            "Ctrl+l",
            "interactive.input-clear",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Insert),
    ];
    PluginPackage::new("interactive", true, "Interactive buffer workflows.")
        .with_commands(commands)
        .with_key_bindings(key_bindings)
}

fn hook_command(name: &str, description: &str, hook_name: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, None::<&str>)],
    )
}
