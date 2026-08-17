//! Rainbow delimiter theme tokens and package metadata.

use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    RainbowParensConfig,
};

/// Hook name for toggling rainbow parens on the active buffer.
pub const TOGGLE_HOOK: &str = "rainbow.parens.toggle";

/// Theme token prefix for depth-colored delimiters.
pub const DEPTH_TOKEN_PREFIX: &str = "rainbow.paren.depth.";
/// Theme token for unmatched closing delimiters.
pub const TOKEN_UNMATCHED: &str = "rainbow.paren.unmatched";
/// Theme token for mismatched closing delimiters.
pub const TOKEN_MISMATCHED: &str = "rainbow.paren.mismatched";

/// Returns rainbow-parens configuration from runtime user config.
pub fn config() -> RainbowParensConfig {
    RainbowParensConfig {
        enabled: crate::config::load().ui.rainbow_parens_enabled,
    }
}

/// Returns the metadata for the rainbow parens package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "rainbow-parens",
        true,
        "Tree-sitter rainbow delimiter highlighting for brackets.",
    )
    .with_commands(vec![PluginCommand::new(
        "rainbow.parens.toggle",
        "Toggles rainbow delimiter highlighting for the active buffer.",
        vec![PluginAction::emit_hook(TOGGLE_HOOK, None::<&str>)],
    )])
    .with_key_bindings(vec![
        PluginKeyBinding::new(
            "<leader>rp",
            "rainbow.parens.toggle",
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
                .any(|command| command.name() == "rainbow.parens.toggle")
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.command_name() == "rainbow.parens.toggle")
        );
    }

    #[test]
    fn config_defaults_to_enabled() {
        assert!(config().enabled);
    }
}
