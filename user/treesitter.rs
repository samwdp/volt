use crate::icon_font::symbols::fa;
use editor_plugin_api::{
    PickerActionSpec, PickerItemSpec, PickerProviderContext, PluginAction, PluginCommand,
    PluginPackage,
};

/// Returns the metadata for the tree-sitter installer package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "treesitter",
        true,
        "Tree-sitter grammar installation and picker commands.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "treesitter.install",
            "Installs a registered Tree-sitter grammar from the picker.",
            vec![PluginAction::emit_hook(
                "ui.picker.open",
                Some("treesitter.languages"),
            )],
        ),
        PluginCommand::new(
            "treesitter.recompile-installed",
            "Recompiles all installed Tree-sitter grammars.",
            vec![PluginAction::emit_hook(
                "treesitter.recompile-installed",
                None::<&str>,
            )],
        ),
    ])
}

pub fn picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .syntax_languages
        .iter()
        .map(|language| {
            let icon = if language.is_installed {
                fa::FA_CHECK
            } else {
                fa::FA_PLUS
            };
            let label = format!("{icon} {}", language.id);
            let mut item = PickerItemSpec::new(
                language.id.clone(),
                label,
                language.detail.clone(),
                PickerActionSpec::install_tree_sitter_language(language.id.clone()),
            );
            if let Some(preview) = language.preview.as_ref().into_option() {
                item = item.with_preview(preview.clone());
            }
            item
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_recompile_installed_command() {
        let package = package();
        let command = package
            .commands()
            .iter()
            .find(|command| command.name() == "treesitter.recompile-installed")
            .expect("treesitter.recompile-installed command");

        assert!(
            command.actions().iter().any(|action| action
                .hook()
                .is_some_and(|hook| hook.hook_name() == "treesitter.recompile-installed"
                    && hook.detail().is_none())),
            "command must emit treesitter.recompile-installed"
        );
    }
}
