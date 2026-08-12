use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage,
    WorkspaceDockConfig,
};

/// Returns workspace dock settings from user config.
pub fn config() -> WorkspaceDockConfig {
    crate::config::load()
        .ui
        .workspace_dock
        .workspace_dock_config()
}

/// Returns the metadata for the workspace dock package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "workspace-dock",
        true,
        "Vertical workspace dock that lists open workspaces.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "workspace.dock.toggle",
            "Shows or hides the workspace dock when it is not permanently docked.",
            vec![PluginAction::emit_hook(
                "ui.workspace-dock.toggle",
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "workspace.dock.previous",
            "Moves to the previous workspace in the dock list.",
            vec![PluginAction::emit_hook(
                "ui.workspace-dock.previous",
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "workspace.dock.next",
            "Moves to the next workspace in the dock list.",
            vec![PluginAction::emit_hook(
                "ui.workspace-dock.next",
                None::<&str>,
            )],
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new("j", "workspace.dock.next", PluginKeymapScope::Popup),
        PluginKeyBinding::new("k", "workspace.dock.previous", PluginKeymapScope::Popup),
    ])
}

#[cfg(test)]
mod tests {
    use super::{config, package};
    use editor_plugin_api::{PluginKeymapScope, WorkspaceDockSide};

    #[test]
    fn package_exports_toggle_command() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "workspace.dock.toggle")
        );
    }

    #[test]
    fn package_exports_dock_navigation_commands() {
        let package = package();
        let names: Vec<_> = package
            .commands()
            .iter()
            .map(|command| command.name())
            .collect();
        assert!(names.contains(&"workspace.dock.previous"));
        assert!(names.contains(&"workspace.dock.next"));
    }

    #[test]
    fn package_binds_j_and_k_in_popup_scope() {
        let package = package();
        for (chord, command) in [
            ("j", "workspace.dock.next"),
            ("k", "workspace.dock.previous"),
        ] {
            assert!(
                package.key_bindings().iter().any(|binding| {
                    binding.chord() == chord
                        && binding.command_name() == command
                        && binding.scope() == PluginKeymapScope::Popup
                }),
                "missing binding for {chord} -> {command}"
            );
        }
    }

    #[test]
    fn config_defaults_to_left_undocked() {
        let config = config();
        assert_eq!(config.side, WorkspaceDockSide::Left);
        assert!(!config.docked);
    }
}
