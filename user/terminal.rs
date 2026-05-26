use editor_plugin_api::{
    ContextHelpEntry, ContextHelpSpec, PluginAction, PluginCommand, PluginKeyBinding,
    PluginKeymapScope, PluginPackage, TerminalFeatureSpec, terminal_hooks,
};
use std::env;

/// Returns the default terminal program for the current platform.
pub fn default_shell_program() -> String {
    if cfg!(target_os = "windows") {
        // "pwsh".to_owned()
        // "bash".to_owned()
        // "zsh".to_owned()
        "nu".to_owned()
    } else {
        env::var("SHELL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "/bin/sh".to_owned())
        // "bash".to_owned()
        // "zsh".to_owned()
        // "fish".to_owned()
        // "nu".to_owned()
    }
}

/// Returns the default argument vector for the configured terminal shell.
pub fn default_shell_args() -> Vec<String> {
    if cfg!(target_os = "windows") {
        // vec!["-NoLogo".to_owned()]
        // vec!["-i".to_owned()] // bash
        // vec!["-i".to_owned()] // zsh
        Vec::new() // nu
    } else {
        Vec::new()
        // vec!["-i".to_owned()] // bash
        // vec!["-i".to_owned()] // zsh
        // vec!["-i".to_owned()] // fish
        // Vec::new() // nu
    }
}

/// Returns the metadata for the builtin terminal package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "terminal",
        true,
        "Builtin terminal buffers and shell integration.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "terminal.open",
            "Opens the builtin terminal buffer.",
            vec![
                PluginAction::open_buffer("*terminal*", "terminal", None::<&str>),
                PluginAction::log_message("Terminal buffer opened from the user package."),
            ],
        ),
        PluginCommand::new(
            "terminal.popup",
            "Opens a popup-hosted terminal buffer.",
            vec![
                PluginAction::open_buffer("*terminal-popup*", "terminal", Some("Terminal")),
                PluginAction::emit_hook(terminal_hooks::OPEN_POPUP, None::<&str>),
            ],
        ),
    ])
    .with_key_bindings(vec![PluginKeyBinding::new(
        "Ctrl+`",
        "terminal.open",
        PluginKeymapScope::Global,
    )])
}

/// Public terminal feature contract used by first-party and third-party code.
pub fn feature_spec() -> TerminalFeatureSpec {
    TerminalFeatureSpec {
        buffer_name: "*terminal*".to_owned(),
        popup_buffer_name: "*terminal-popup*".to_owned(),
        help: ContextHelpSpec::new(
            "Terminal",
            "Terminal",
            vec![
                ContextHelpEntry::new("Ctrl+`", "open terminal", "Opens builtin terminal buffer."),
                ContextHelpEntry::new(
                    "terminal.popup",
                    "open popup terminal",
                    "Opens popup-hosted terminal buffer.",
                ),
            ],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{default_shell_args, default_shell_program, package};

    #[test]
    fn package_exports_terminal_commands_and_binding() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "terminal.open")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "terminal.popup")
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.command_name() == "terminal.open")
        );
    }

    #[test]
    fn terminal_shell_configuration_is_non_empty() {
        assert!(!default_shell_program().trim().is_empty());
        assert!(
            default_shell_args()
                .iter()
                .all(|arg| !arg.trim().is_empty())
        );
    }
}
