use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
};

/// Returns the metadata for the multiple cursor package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "multicursor",
        true,
        "Multiple cursor editing commands and selections.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "multicursor.add-next-match",
            "Adds a new cursor at the next match.",
            vec![PluginAction::emit_hook(
                "editor.vim.edit",
                Some("multicursor-add-next-match"),
            )],
        ),
        PluginCommand::new(
            "multicursor.select-all-matches",
            "Adds cursors at every remaining match in the buffer.",
            vec![PluginAction::emit_hook(
                "editor.vim.edit",
                Some("multicursor-select-all-matches"),
            )],
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new(
            "g n",
            "multicursor.add-next-match",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new(
            "g n",
            "multicursor.add-next-match",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Visual),
        PluginKeyBinding::new(
            "g N",
            "multicursor.select-all-matches",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new(
            "g N",
            "multicursor.select-all-matches",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Visual),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_default_gn_bindings() {
        let package = package();
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == "g n"
                    && binding.command_name() == "multicursor.add-next-match"
                    && binding.vim_mode() == PluginVimMode::Normal)
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == "g N"
                    && binding.command_name() == "multicursor.select-all-matches"
                    && binding.vim_mode() == PluginVimMode::Normal)
        );
    }
}
