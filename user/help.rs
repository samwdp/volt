//! Docs help package.
//!
//! Press the keybind (default **F1**) to ask a question. Volt opens a browser
//! split on the docs search page for that query.
//!
//! The only intended user edit in this file is the keybind chord.

use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, help_hooks,
};

/// Builds the Volt docs search URL for `question`.
pub fn docs_search_url(question: &str) -> String {
    help_hooks::docs_search_url(question)
}

/// Returns the metadata for the help package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "help",
        true,
        "Prompt for a docs question and open the search page in a browser split.",
    )
    .with_commands(vec![PluginCommand::new(
        "help",
        "Asks for a question, then opens the Volt docs search page in a browser split.",
        vec![PluginAction::emit_hook(help_hooks::OPEN, None::<&str>)],
    )])
    .with_key_bindings(vec![
        // Change this chord if you want a different help shortcut.
        PluginKeyBinding::new("F1", "help", PluginKeymapScope::Global),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_help_command_and_f1_binding() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "help")
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == "F1" && binding.command_name() == "help")
        );
    }

    #[test]
    fn docs_search_url_encodes_query() {
        assert_eq!(
            docs_search_url("split pane"),
            "https://samwdp.github.io/volt-docs/search?q=split%20pane"
        );
        assert_eq!(
            docs_search_url(" a&b "),
            "https://samwdp.github.io/volt-docs/search?q=a%26b"
        );
    }
}
