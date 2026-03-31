use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, image_hooks,
};

pub const ZOOM_IN_CHORD: &str = "Ctrl+=";
pub const ZOOM_OUT_CHORD: &str = "Ctrl+-";
pub const ZOOM_RESET_CHORD: &str = "Ctrl+0";
pub const TOGGLE_MODE_CHORD: &str = "C-c C-c";

/// Returns the metadata for the native image-viewer package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "image",
        true,
        "Native image-viewer commands and keybindings.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "image.zoom-in",
            "Zoom the active native image buffer in.",
            vec![PluginAction::emit_hook(image_hooks::ZOOM_IN, None::<&str>)],
        ),
        PluginCommand::new(
            "image.zoom-out",
            "Zoom the active native image buffer out.",
            vec![PluginAction::emit_hook(image_hooks::ZOOM_OUT, None::<&str>)],
        ),
        PluginCommand::new(
            "image.zoom-reset",
            "Reset the active native image buffer to its fitted default zoom.",
            vec![PluginAction::emit_hook(
                image_hooks::ZOOM_RESET,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "image.toggle-mode",
            "Toggle the active SVG image buffer between rendered preview and source mode.",
            vec![PluginAction::emit_hook(
                image_hooks::TOGGLE_MODE,
                None::<&str>,
            )],
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new(ZOOM_IN_CHORD, "image.zoom-in", PluginKeymapScope::Workspace),
        PluginKeyBinding::new(
            ZOOM_OUT_CHORD,
            "image.zoom-out",
            PluginKeymapScope::Workspace,
        ),
        PluginKeyBinding::new(
            ZOOM_RESET_CHORD,
            "image.zoom-reset",
            PluginKeymapScope::Workspace,
        ),
        PluginKeyBinding::new(
            TOGGLE_MODE_CHORD,
            "image.toggle-mode",
            PluginKeymapScope::Workspace,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_image_commands() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "image.zoom-in")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "image.zoom-out")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "image.zoom-reset")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "image.toggle-mode")
        );
    }

    #[test]
    fn package_exports_image_keybindings() {
        let package = package();
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == ZOOM_IN_CHORD
                    && binding.command_name() == "image.zoom-in")
        );
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == ZOOM_OUT_CHORD && binding.command_name() == "image.zoom-out"
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == ZOOM_RESET_CHORD && binding.command_name() == "image.zoom-reset"
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == TOGGLE_MODE_CHORD && binding.command_name() == "image.toggle-mode"
        }));
    }
}
