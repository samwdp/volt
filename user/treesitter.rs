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
    .with_commands(vec![PluginCommand::new(
        "treesitter.install",
        "Installs a registered Tree-sitter grammar from the picker.",
        vec![PluginAction::emit_hook(
            "ui.picker.open",
            Some("treesitter.languages"),
        )],
    )])
}

pub fn picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    context
        .syntax_languages
        .iter()
        .map(|language| {
            let mut item = PickerItemSpec::new(
                language.id.clone(),
                language.id.clone(),
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
