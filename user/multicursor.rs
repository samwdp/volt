use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    VimActionSpec,
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
                Some(VimActionSpec::MulticursorAddNextMatch.hook_detail()),
            )],
        ),
        PluginCommand::new(
            "multicursor.add-previous-match",
            "Adds a new cursor at the previous match.",
            vec![PluginAction::emit_hook(
                "editor.vim.edit",
                Some(VimActionSpec::MulticursorAddPreviousMatch.hook_detail()),
            )],
        ),
        PluginCommand::new(
            "multicursor.select-all-matches",
            "Adds cursors at every remaining match in the buffer.",
            vec![PluginAction::emit_hook(
                "editor.vim.edit",
                Some(VimActionSpec::MulticursorSelectAllMatches.hook_detail()),
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
        // While Multicursor Mode is active, n/p add next/previous matches
        // without requiring the g prefix (overrides Workspace search/paste).
        PluginKeyBinding::new(
            "n",
            "multicursor.add-next-match",
            PluginKeymapScope::Multicursor,
        )
        .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new(
            "n",
            "multicursor.add-next-match",
            PluginKeymapScope::Multicursor,
        )
        .with_vim_mode(PluginVimMode::Visual),
        PluginKeyBinding::new(
            "p",
            "multicursor.add-previous-match",
            PluginKeymapScope::Multicursor,
        )
        .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new(
            "p",
            "multicursor.add-previous-match",
            PluginKeymapScope::Multicursor,
        )
        .with_vim_mode(PluginVimMode::Visual),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_normal_mode_bindings_for_multicursor_commands() {
        let package = package();
        assert!(
            package
                .key_bindings()
                .iter()
                .any(
                    |binding| binding.command_name() == "multicursor.add-next-match"
                        && binding.vim_mode() == PluginVimMode::Normal
                        && binding.scope() == PluginKeymapScope::Workspace
                )
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(
                    |binding| binding.command_name() == "multicursor.select-all-matches"
                        && binding.vim_mode() == PluginVimMode::Normal
                        && binding.scope() == PluginKeymapScope::Workspace
                )
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(
                    |binding| binding.command_name() == "multicursor.add-next-match"
                        && binding.chord() == "n"
                        && binding.scope() == PluginKeymapScope::Multicursor
                )
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(
                    |binding| binding.command_name() == "multicursor.add-previous-match"
                        && binding.chord() == "p"
                        && binding.scope() == PluginKeymapScope::Multicursor
                )
        );
    }
}
