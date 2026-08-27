//! Emacs-style show-paren highlighting for delimiters and HTML/XML tags.
//!
//! Tag matching for show-paren and `%` is limited to `html`, `xml`, `jsx`, and `tsx`.

use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    ShowParenConfig,
};

/// Hook name for toggling show-paren on the active buffer.
pub const TOGGLE_HOOK: &str = "show-paren.toggle";

/// Theme token for a matched delimiter or tag pair.
pub const TOKEN_MATCH: &str = "ui.show-paren.match";
/// Theme token for an unmatched delimiter or tag.
pub const TOKEN_MISMATCH: &str = "ui.show-paren.mismatch";

/// Returns show-paren configuration from runtime user config.
pub fn config() -> ShowParenConfig {
    ShowParenConfig {
        enabled: crate::config::load().ui.show_paren_enabled,
    }
}

/// Returns the metadata for the show-paren package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "show-paren",
        true,
        "Highlights the matching delimiter, and HTML/XML tags in html/xml/jsx/tsx buffers.",
    )
    .with_commands(vec![PluginCommand::new(
        "show-paren.toggle",
        "Toggles show-paren highlighting for the active buffer.",
        vec![PluginAction::emit_hook(TOGGLE_HOOK, None::<&str>)],
    )])
    .with_key_bindings(vec![
        PluginKeyBinding::new(
            "<leader>sp",
            "show-paren.toggle",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
    ])
}

#[cfg(test)]
mod tests {
    use super::{config, package};

    #[test]
    fn package_exports_toggle_command_and_binding() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "show-paren.toggle")
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.command_name() == "show-paren.toggle")
        );
    }

    #[test]
    fn config_defaults_to_enabled() {
        assert!(config().enabled);
    }
}
