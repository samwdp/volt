use editor_plugin_api::{
    PickerActionSpec, PickerItemSpec, PickerProviderContext, PluginAction, PluginCommand,
    PluginPackage,
};

/// Returns the metadata for the undo tree picker package.
pub fn package() -> PluginPackage {
    PluginPackage::new("undotree", true, "Undo tree history navigation.").with_commands(vec![
        PluginCommand::new(
            "undo-tree.open",
            "Opens the undo tree picker.",
            vec![PluginAction::emit_hook("ui.picker.open", Some("undo-tree"))],
        ),
    ])
}

pub fn picker_items(context: &PickerProviderContext) -> Vec<PickerItemSpec> {
    if context.undo_tree.is_empty() {
        return vec![
            PickerItemSpec::new(
                "No undo history",
                "No undo history",
                "Make an edit to populate the undo tree.",
                PickerActionSpec::no_op(),
            )
            .with_preview("Make an edit to populate the undo tree."),
        ];
    }
    context
        .undo_tree
        .iter()
        .map(|entry| {
            let mut item = PickerItemSpec::new(
                format!("undo:{}", entry.node_id),
                entry.label.clone(),
                entry.detail.clone(),
                PickerActionSpec::undo_tree_node(entry.buffer_id, entry.node_id),
            );
            if let Some(preview) = entry.preview.as_ref().into_option() {
                item = item.with_preview(preview.clone());
            }
            item
        })
        .collect()
}
