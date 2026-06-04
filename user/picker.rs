use editor_plugin_api::{
    PickerActionSpec, PickerItemSpec, PickerProviderContext, PickerProviderSpec, PickerSource,
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    picker_hooks,
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
            picker_hooks::OPEN,
            Some("commands"),
        ),
        hook_command(
            "picker.open-buffers",
            "Opens the buffer picker popup.",
            picker_hooks::OPEN,
            Some("buffers"),
        ),
        hook_command(
            "picker.open-keybindings",
            "Opens the keybinding picker popup.",
            picker_hooks::OPEN,
            Some("keybindings"),
        ),
        hook_command(
            "picker.open-themes",
            "Opens the theme picker popup.",
            picker_hooks::OPEN,
            Some("themes"),
        ),
        hook_command(
            "picker.open-icon-fonts",
            "Opens the bundled icon font picker popup.",
            picker_hooks::OPEN,
            Some("icon-fonts"),
        ),
        hook_command(
            "picker.select-next",
            "Moves to the next picker result.",
            picker_hooks::NEXT,
            None,
        ),
        hook_command(
            "picker.select-previous",
            "Moves to the previous picker result.",
            picker_hooks::PREVIOUS,
            None,
        ),
        hook_command(
            "picker.submit",
            "Runs the selected picker action.",
            picker_hooks::SUBMIT,
            None,
        ),
        hook_command(
            "picker.cancel",
            "Closes the active picker popup.",
            picker_hooks::CANCEL,
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

/// Returns picker providers exposed to the shell. Add custom pickers here.
pub fn providers() -> Vec<PickerProviderSpec> {
    vec![
        PickerProviderSpec::new("commands", "Command Palette", PickerSource::Commands),
        PickerProviderSpec::new("buffers", "Buffers", PickerSource::Buffers),
        PickerProviderSpec::new("buffers.close", "Close Buffers", PickerSource::BufferClose),
        PickerProviderSpec::new("keybindings", "Keybindings", PickerSource::Keybindings),
        PickerProviderSpec::new("themes", "Themes", PickerSource::Themes),
        PickerProviderSpec::new("icon-fonts", "Bundled Icon Fonts", PickerSource::IconFonts),
        PickerProviderSpec::new("acp-clients", "ACP Clients", PickerSource::AcpClients),
        PickerProviderSpec::new(
            "treesitter.languages",
            "Tree-sitter Install",
            PickerSource::TreesitterLanguages,
        ),
        PickerProviderSpec::new(
            "workspace.projects",
            "Projects",
            PickerSource::WorkspaceProjects,
        ),
        PickerProviderSpec::new(
            "workspace.dashboard",
            "Worktrees",
            PickerSource::WorkspaceDashboard,
        ),
        PickerProviderSpec::new(
            "workspace.switch",
            "Workspaces",
            PickerSource::WorkspaceSwitch,
        ),
        PickerProviderSpec::new(
            "workspace.delete",
            "Delete Workspace",
            PickerSource::WorkspaceDelete,
        ),
        PickerProviderSpec::new(
            "workspace.files",
            "Workspace Files",
            PickerSource::WorkspaceFiles,
        ),
        PickerProviderSpec::new(
            "workspace.search",
            "Workspace Search",
            PickerSource::WorkspaceSearch,
        ),
        PickerProviderSpec::new("undo-tree", "Undo Tree", PickerSource::UndoTree),
    ]
}

/// Returns entries for custom user-owned pickers.
///
/// Add a `PickerProviderSpec::user("my.picker", "My Picker")` in `providers()`,
/// then return its entries here. Built-in `PickerSource::*` providers do not need
/// an entry function because the shell supplies their runtime data.
pub fn provider_items(context: &PickerProviderContext) -> Option<Vec<PickerItemSpec>> {
    match context.source {
        PickerSource::User | PickerSource::Static => providers()
            .into_iter()
            .find(|provider| {
                provider.id() == context.provider_id.as_str()
                    && provider.source() == PickerSource::Static
            })
            .map(|provider| provider.items().to_vec()),
        PickerSource::Commands => Some(command_picker_items(context)),
        PickerSource::Buffers => Some(buffer_picker_items(context)),
        PickerSource::BufferClose => Some(buffer_close_picker_items(context)),
        PickerSource::Keybindings => Some(keybinding_picker_items(context)),
        PickerSource::Themes => Some(theme_picker_items(context)),
        PickerSource::IconFonts => Some(icon_font_picker_items(context)),
        PickerSource::AcpClients => Some(acp_client_picker_items(context)),
        PickerSource::TreesitterLanguages => Some(crate::treesitter::picker_items(context)),
        PickerSource::WorkspaceProjects
        | PickerSource::WorkspaceDashboard
        | PickerSource::WorkspaceSwitch
        | PickerSource::WorkspaceDelete
        | PickerSource::WorkspaceFiles
        | PickerSource::WorkspaceSearch => crate::workspace::picker_items(context),
        PickerSource::UndoTree => Some(crate::undotree::picker_items(context)),
    }
}

fn command_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .commands
        .iter()
        .map(|command| {
            PickerItemSpec::new(
                command.name.clone(),
                command.name.clone(),
                command.description.clone(),
                PickerActionSpec::execute_command(command.name.clone()),
            )
            .with_preview(command.description.clone())
        })
        .collect()
}

fn buffer_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .buffers
        .iter()
        .map(|buffer| {
            PickerItemSpec::new(
                buffer.id.to_string(),
                buffer.display_name.clone(),
                buffer.kind_label.clone(),
                PickerActionSpec::focus_buffer(buffer.id),
            )
            .with_preview(format!(
                "{} | row {}, col {}",
                buffer.kind_label,
                buffer.cursor_row + 1,
                buffer.cursor_col + 1
            ))
        })
        .collect()
}

fn buffer_close_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .buffers
        .iter()
        .map(|buffer| {
            let dirty = if buffer.dirty { "modified" } else { "clean" };
            PickerItemSpec::new(
                buffer.id.to_string(),
                buffer.display_name.clone(),
                format!("{} | {dirty}", buffer.kind_label),
                PickerActionSpec::close_buffer(buffer.id),
            )
            .with_preview(format!(
                "{} | row {}, col {}",
                buffer.kind_label,
                buffer.cursor_row + 1,
                buffer.cursor_col + 1
            ))
        })
        .collect()
}

fn keybinding_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .keybindings
        .iter()
        .map(|binding| {
            let command_names = binding
                .command_names
                .iter()
                .map(|name| name.as_str().to_owned())
                .collect::<Vec<_>>();
            let command_label = command_names.join(" -> ");
            let action = if command_names.len() == 1 {
                PickerActionSpec::execute_command(command_names[0].clone())
            } else {
                PickerActionSpec::execute_commands(command_names)
            };
            PickerItemSpec::new(
                binding.id.clone(),
                format!("{} {}", binding.chord, command_label),
                format!(
                    "{} [{}] -> {}",
                    binding.scope, binding.vim_mode, command_label
                ),
                action,
            )
            .with_preview(binding.description.clone())
        })
        .collect()
}

fn theme_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .themes
        .iter()
        .map(|theme| {
            PickerItemSpec::new(
                theme.id.clone(),
                theme.name.clone(),
                "Theme",
                PickerActionSpec::activate_theme(theme.id.clone()),
            )
            .with_preview(theme.id.clone())
        })
        .collect()
}

fn icon_font_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .icons
        .iter()
        .map(|icon| {
            PickerItemSpec::new(
                icon.id.clone(),
                icon.label.clone(),
                icon.detail.clone(),
                PickerActionSpec::copy_to_clipboard(icon.glyph.clone()),
            )
            .with_preview(icon.glyph.clone())
        })
        .collect()
}

fn acp_client_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .acp_clients
        .iter()
        .map(|client| {
            PickerItemSpec::new(
                client.id.clone(),
                client.label.clone(),
                client.detail.clone(),
                PickerActionSpec::open_acp_client(client.id.clone()),
            )
        })
        .collect()
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
