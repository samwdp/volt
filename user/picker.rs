//! Picker providers, commands, and label truncation configuration.
//!
//! Long path labels are clipped at render time using [`truncate_strategy`]. Change
//! that function to pick a [`PickerTruncateStrategy`]; the shell applies it to
//! every picker row label.
//!
//! Examples use `src/dir1/dir2/test.rs` unless noted. Ellipsis variants clip to
//! the viewport width after any path transform.
//!
//! | Strategy | Example output |
//! |---|---|
//! | [`PickerTruncateStrategy::Auto`] | full path, else `s/d/d/test.rs`, else `...ir2/test.rs` |
//! | [`PickerTruncateStrategy::StartEllipsis`] | `...ir2/test.rs` |
//! | [`PickerTruncateStrategy::MiddleEllipsis`] | `src...test.rs` |
//! | [`PickerTruncateStrategy::EndEllipsis`] | `src/dir1/dir2/te...` |
//! | [`PickerTruncateStrategy::ShrinkDirectories`] | `s/d/d/test.rs` |
//! | [`PickerTruncateStrategy::ShrinkAll`] | `s/d/d/t.rs` |
//! | [`PickerTruncateStrategy::FileName`] | `test.rs` |
//! | [`PickerTruncateStrategy::FileNameWithParent`] | `dir2/test.rs` |
//! | [`PickerTruncateStrategy::ParentInitialFileName`] | `d/test.rs` |
//! | [`PickerTruncateStrategy::ShrinkLeadingKeepTail`] | shrink leading dirs, keep last 3 segments |
//! | [`PickerTruncateStrategy::Full`] | full path when it fits, else head clip |

use std::path::Path;

pub use editor_plugin_api::PickerTruncateStrategy;

use editor_plugin_api::{
    PickerActionSpec, PickerItemSpec, PickerProviderContext, PickerProviderSpec, PickerSource,
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    picker_hooks,
};

/// Picker label truncation when a row is narrower than the label.
///
/// Default is [`PickerTruncateStrategy::Auto`]: show the full label when it fits,
/// shrink parent directories fish-shell style, then clip from the start so the
/// filename stays visible. For doom-modeline `truncate-all`, use
/// [`PickerTruncateStrategy::ShrinkAll`].
pub fn truncate_strategy() -> PickerTruncateStrategy {
    crate::config::load().ui.picker_truncate_strategy.into()
}

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
            "Workspaces and Projects",
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
            let label = buffer_picker_label(buffer);
            let detail = buffer_picker_detail(buffer);
            let preview = buffer
                .preview
                .as_ref()
                .into_option()
                .map(|preview| preview.to_string())
                .unwrap_or_else(|| {
                    format!(
                        "{} | row {}, col {}",
                        buffer.kind_label,
                        buffer.cursor_row + 1,
                        buffer.cursor_col + 1
                    )
                });
            PickerItemSpec::new(
                buffer.id.to_string(),
                label,
                detail,
                PickerActionSpec::focus_buffer(buffer.id),
            )
            .with_preview(preview)
            .with_search_text(buffer.display_name.clone())
        })
        .collect()
}

fn buffer_close_picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .buffers
        .iter()
        .map(|buffer| {
            let dirty = if buffer.dirty { "modified" } else { "clean" };
            let label = buffer_picker_label(buffer);
            let detail = format!("{} | {dirty}", buffer_picker_detail(buffer));
            let preview = buffer
                .preview
                .as_ref()
                .into_option()
                .map(|preview| preview.to_string())
                .unwrap_or_else(|| {
                    format!(
                        "{} | row {}, col {}",
                        buffer.kind_label,
                        buffer.cursor_row + 1,
                        buffer.cursor_col + 1
                    )
                });
            PickerItemSpec::new(
                buffer.id.to_string(),
                label,
                detail,
                PickerActionSpec::close_buffer(buffer.id),
            )
            .with_preview(preview)
            .with_search_text(buffer.display_name.clone())
        })
        .collect()
}

fn buffer_picker_label(buffer: &editor_plugin_api::PickerBufferContext) -> String {
    buffer
        .path
        .as_ref()
        .into_option()
        .and_then(|path| path_file_name(path.as_str()))
        .unwrap_or_else(|| buffer.display_name.to_string())
}

/// Extracts the final path component, treating both `/` and `\` as separators so
/// Windows-style buffer paths render a clean file name on every host platform.
fn path_file_name(path: &str) -> Option<String> {
    let name = path
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default();
    (!name.is_empty()).then(|| name.to_owned())
}

fn buffer_picker_detail(buffer: &editor_plugin_api::PickerBufferContext) -> String {
    let mut parts = vec![buffer.kind_label.to_string()];
    if let Some(path) = buffer.path.as_ref().into_option() {
        let parent = Path::new(path.as_str())
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(|parent| parent.display().to_string())
            .unwrap_or_else(|| path.to_string());
        parts.push(parent);
    }
    parts.push(format!(
        "row {}, col {}",
        buffer.cursor_row + 1,
        buffer.cursor_col + 1
    ));
    parts.join(" | ")
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

    #[test]
    fn truncate_strategy_defaults_to_auto() {
        assert_eq!(truncate_strategy(), PickerTruncateStrategy::Auto);
    }

    #[test]
    fn buffer_picker_shows_file_name_first_and_keeps_path_search() {
        let mut context = PickerProviderContext::new("buffers", "Buffers", PickerSource::Buffers);
        context.buffers = vec![editor_plugin_api::PickerBufferContext {
            id: 42,
            display_name: r"P:\volt\src\main.rs".into(),
            path: Some(r"P:\volt\src\main.rs".into()).into(),
            kind_label: "file".into(),
            preview: Some("P:\\volt\\src\\main.rs\nfn main() {}".into()).into(),
            cursor_row: 3,
            cursor_col: 4,
            dirty: false,
        }]
        .into();

        let items = buffer_picker_items(&context);
        assert_eq!(items[0].label(), "main.rs");
        assert!(items[0].detail().contains("file"));
        assert!(items[0].detail().contains(r"P:\volt\src"));
        assert_eq!(items[0].search_text(), Some(r"P:\volt\src\main.rs"));
        assert!(
            items[0]
                .preview()
                .is_some_and(|preview| preview.contains("fn main()"))
        );
    }
}
