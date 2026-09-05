use super::*;
use editor_plugin_api::{PickerActionSpec, PickerItemSpec, PickerProviderSpec};

#[test]
fn static_user_picker_provider_builds_executable_entries() -> Result<(), String> {
    let provider = PickerProviderSpec::static_items(
        "tools",
        "Tools",
        vec![
            PickerItemSpec::new(
                "open-commands",
                "Command palette",
                "Open command picker",
                PickerActionSpec::execute_command("picker.open-commands"),
            )
            .with_search_text("commands palette")
            .with_preview("Runs picker.open-commands"),
        ],
    );

    let mut runtime = EditorRuntime::new();
    runtime
        .services_mut()
        .insert(UserLibraryService(Arc::new(NullUserLibrary)));
    let context = picker_provider_context(&runtime, &provider)?;
    let picker = user_picker_overlay(&runtime, &provider, context)?;
    let selected = picker
        .session()
        .selected()
        .ok_or_else(|| "static picker has no selected item".to_owned())?;
    assert_eq!(picker.session().title(), "Tools");
    assert_eq!(selected.item().label(), "Command palette");
    assert_eq!(selected.item().preview(), Some("Runs picker.open-commands"));
    assert!(matches!(
        picker.selected_action(),
        Some(PickerAction::ExecuteCommand(command)) if command == "picker.open-commands"
    ));
    Ok(())
}

#[test]
fn static_picker_keeps_all_matches_for_large_lists() -> Result<(), String> {
    let items = (0..80)
        .map(|index| {
            PickerItemSpec::new(
                format!("file-{index:03}"),
                format!("file-{index:03}.rs"),
                "src",
                PickerActionSpec::no_op(),
            )
        })
        .collect::<Vec<_>>();
    let provider = PickerProviderSpec::static_items("files", "Files", items);

    let mut runtime = EditorRuntime::new();
    runtime
        .services_mut()
        .insert(UserLibraryService(Arc::new(NullUserLibrary)));
    let context = picker_provider_context(&runtime, &provider)?;
    let picker = user_picker_overlay(&runtime, &provider, context)?;

    assert_eq!(picker.session().item_count(), 80);
    assert_eq!(picker.session().match_count(), 80);
    Ok(())
}

struct BoundedSourceLibrary {
    source: PickerSource,
    items: Vec<PickerItemSpec>,
}

impl UserLibrary for BoundedSourceLibrary {
    fn picker_provider_items(
        &self,
        context: &PickerProviderContext,
    ) -> Option<Vec<PickerItemSpec>> {
        (context.source == self.source).then(|| self.items.clone())
    }
}

fn preview_items(count: usize, prefix: &str) -> Vec<PickerItemSpec> {
    (0..count)
        .map(|index| {
            PickerItemSpec::new(
                format!("{prefix}-{index:03}"),
                format!("{prefix}-{index:03}.rs"),
                "src",
                PickerActionSpec::no_op(),
            )
            .with_preview(format!("preview-{prefix}-{index:03}"))
        })
        .collect()
}

fn overlay_for_source(
    source: PickerSource,
    id: &str,
    title: &str,
    items: Vec<PickerItemSpec>,
) -> Result<PickerOverlay, String> {
    let provider = PickerProviderSpec::new(id, title, source);
    let mut runtime = EditorRuntime::new();
    runtime
        .services_mut()
        .insert(UserLibraryService(Arc::new(BoundedSourceLibrary {
            source,
            items,
        })));
    let context = PickerProviderContext::new(id, title, source);
    user_picker_overlay(&runtime, &provider, context)
}

#[test]
fn workspace_files_overlay_caps_matches_without_dropping_source_items() -> Result<(), String> {
    let picker = overlay_for_source(
        PickerSource::WorkspaceFiles,
        "workspace.files",
        "Workspace Files",
        preview_items(300, "file"),
    )?;

    assert_eq!(picker.session().item_count(), 300);
    assert_eq!(
        picker.session().match_count(),
        WORKSPACE_FILES_PICKER_RESULT_LIMIT
    );
    assert!(
        picker
            .session()
            .matches()
            .iter()
            .any(|matched| matched.item().preview() == Some("preview-file-000"))
    );
    Ok(())
}

#[test]
fn command_picker_overlay_uses_finite_result_limit() -> Result<(), String> {
    let picker = overlay_for_source(
        PickerSource::Commands,
        "commands",
        "Command Palette",
        preview_items(600, "cmd"),
    )?;

    assert_eq!(picker.session().item_count(), 600);
    assert_eq!(picker.session().match_count(), COMMAND_PICKER_RESULT_LIMIT);
    Ok(())
}

#[test]
fn provider_extra_keybinds_copy_onto_open_picker_instance() -> Result<(), String> {
    let provider = PickerProviderSpec::static_items(
        "tools",
        "Tools",
        vec![PickerItemSpec::new(
            "open-commands",
            "Command palette",
            "Open command picker",
            PickerActionSpec::execute_command("picker.open-commands"),
        )],
    )
    .with_extra_keybind("Ctrl+d", "workspace.worktree-remove");

    let mut runtime = EditorRuntime::new();
    runtime
        .services_mut()
        .insert(UserLibraryService(Arc::new(NullUserLibrary)));
    let context = picker_provider_context(&runtime, &provider)?;
    let picker = user_picker_overlay(&runtime, &provider, context)?;

    assert_eq!(picker.extra_keybinds().len(), 1);
    assert_eq!(picker.extra_keybinds()[0].chord(), "Ctrl+d");
    assert_eq!(
        picker.extra_keybinds()[0].command_name(),
        "workspace.worktree-remove"
    );
    Ok(())
}

#[test]
fn picker_preview_is_opt_in() {
    let picker = PickerOverlay::from_entries("Files", Vec::new());
    assert!(!picker.show_preview());
    assert!(picker.with_preview().show_preview());
}

#[test]
fn picker_preview_layout_splits_preview_to_the_right() {
    let layout = picker_preview_layout(Some("path\nline 1\nline 2"), 20, 900, 120, 300, 20)
        .expect("preview layout should exist on wide pickers");

    assert_eq!(layout.y, 120);
    assert!(layout.x > 20);
    assert!(layout.list_width < 900);
    assert_eq!(layout.lines, vec!["path", "line 1", "line 2"]);
}
