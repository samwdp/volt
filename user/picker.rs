use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
};

/// Returns the metadata for the generic picker UI package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "picker",
        true,
        "Generic searchable list UI with keyboard navigation.",
    )
    .with_commands(vec![
        hook_command(
            "picker.open-commands",
            "Opens the command picker popup.",
            "ui.picker.open",
            Some("commands"),
        ),
        hook_command(
            "picker.open-buffers",
            "Opens the buffer picker popup.",
            "ui.picker.open",
            Some("buffers"),
        ),
        hook_command(
            "picker.open-keybindings",
            "Opens the keybinding picker popup.",
            "ui.picker.open",
            Some("keybindings"),
        ),
        hook_command(
            "picker.open-themes",
            "Opens the theme picker popup.",
            "ui.picker.open",
            Some("themes"),
        ),
        hook_command(
            "picker.open-icon-fonts",
            "Opens the bundled icon font picker popup.",
            "ui.picker.open",
            Some("icon-fonts"),
        ),
        hook_command(
            "picker.select-next",
            "Moves to the next picker result.",
            "ui.picker.next",
            None,
        ),
        hook_command(
            "picker.select-previous",
            "Moves to the previous picker result.",
            "ui.picker.previous",
            None,
        ),
        hook_command(
            "picker.submit",
            "Runs the selected picker action.",
            "ui.picker.submit",
            None,
        ),
        hook_command(
            "picker.cancel",
            "Closes the active picker popup.",
            "ui.picker.cancel",
            None,
        ),
        hook_command(
            "quickfix.open",
            "Opens current quickfix popup buffer.",
            "ui.quickfix.open",
            None,
        ),
        hook_command(
            "quickfix.next",
            "Opens next quickfix entry.",
            "ui.quickfix.next",
            None,
        ),
        hook_command(
            "quickfix.previous",
            "Opens previous quickfix entry.",
            "ui.quickfix.previous",
            None,
        ),
        hook_command(
            "quickfix.toggle-mark",
            "Toggles mark on current quickfix row.",
            "ui.quickfix.toggle-mark",
            None,
        ),
        hook_command(
            "quickfix.clear-marks",
            "Clears quickfix marks.",
            "ui.quickfix.clear-marks",
            None,
        ),
        hook_command(
            "quickfix.mark-all",
            "Marks all quickfix rows.",
            "ui.quickfix.mark-all",
            None,
        ),
        hook_command(
            "picker.toggle-popup-window",
            "Shows or closes the docked popup window.",
            "ui.popup.toggle",
            None,
        ),
        hook_command(
            "popup.next",
            "Cycles to the next popup buffer.",
            "ui.popup.next",
            None,
        ),
        hook_command(
            "popup.previous",
            "Cycles to the previous popup buffer.",
            "ui.popup.previous",
            None,
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new("F3", "picker.open-commands", PluginKeymapScope::Global),
        PluginKeyBinding::new("F4", "picker.open-buffers", PluginKeymapScope::Global),
        PluginKeyBinding::new("F7", "picker.open-keybindings", PluginKeymapScope::Global),
        PluginKeyBinding::new("F6", "picker.open-themes", PluginKeymapScope::Global),
        PluginKeyBinding::new("Ctrl+n", "popup.next", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("Ctrl+p", "popup.previous", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("Ctrl+n", "picker.select-next", PluginKeymapScope::Popup),
        PluginKeyBinding::new("Ctrl+p", "picker.select-previous", PluginKeymapScope::Popup),
        PluginKeyBinding::new("Ctrl+q", "quickfix.open", PluginKeymapScope::Popup),
        PluginKeyBinding::new("Enter", "picker.submit", PluginKeymapScope::Popup),
        PluginKeyBinding::new("Escape", "picker.cancel", PluginKeymapScope::Popup),
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

    #[test]
    fn package_binds_ctrl_q_to_quickfix_open_in_popups() {
        let package = package();
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+q"
                && binding.command_name() == "quickfix.open"
                && binding.scope() == PluginKeymapScope::Popup
                && binding.vim_mode() == PluginVimMode::Any
        }));
    }
}
