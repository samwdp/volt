use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

const INTERACTIVE_READONLY_KIND: &str = "interactive-readonly";

/// Returns the metadata for interactive buffer commands.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "interactive",
        true,
        "Interactive read-only buffer workflows.",
    )
    .with_commands(vec![PluginCommand::new(
        "interactive.open-readonly",
        "Opens an interactive read-only buffer.",
        vec![PluginAction::open_buffer(
            "*interactive*",
            INTERACTIVE_READONLY_KIND,
            None::<&str>,
        )],
    )])
}
