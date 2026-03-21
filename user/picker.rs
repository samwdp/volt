use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage,
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
            "picker.toggle-popup-window",
            "Shows or closes the docked popup window.",
            "ui.popup.toggle",
            None,
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new("F3", "picker.open-commands", PluginKeymapScope::Global),
        PluginKeyBinding::new("F4", "picker.open-buffers", PluginKeymapScope::Global),
        PluginKeyBinding::new(
            "F5",
            "picker.toggle-popup-window",
            PluginKeymapScope::Global,
        ),
        PluginKeyBinding::new("F6", "picker.open-keybindings", PluginKeymapScope::Global),
        PluginKeyBinding::new("F7", "picker.open-themes", PluginKeymapScope::Global),
        PluginKeyBinding::new("Ctrl+n", "picker.select-next", PluginKeymapScope::Popup),
        PluginKeyBinding::new("Ctrl+p", "picker.select-previous", PluginKeymapScope::Popup),
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
