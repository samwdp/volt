use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

/// Returns the metadata for the tree-sitter installer package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "treesitter",
        true,
        "Tree-sitter grammar installation and picker commands.",
    )
    .with_commands(vec![PluginCommand::new(
        "treesitter.install",
        "Installs a registered Tree-sitter grammar from the picker.",
        vec![PluginAction::emit_hook(
            "ui.picker.open",
            Some("treesitter.languages"),
        )],
    )])
}
