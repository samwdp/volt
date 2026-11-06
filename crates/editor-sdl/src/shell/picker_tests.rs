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
    let layout = picker_preview_layout(Some("path\nline 1\nline 2"), 20, 900, 120, 300, 20, 0.5)
        .expect("preview layout should exist on wide pickers");

    assert_eq!(layout.y, 120);
    assert!(layout.x > 20);
    assert!(layout.list_width < 900);
    assert_eq!(layout.height, 300);
    assert!(layout.width > 0);
    assert_eq!(layout.lines, vec!["path", "line 1", "line 2"]);
}

#[test]
fn picker_preview_layout_fills_available_height_with_file_lines() {
    let body = (0..80)
        .map(|index| format!("line-{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let preview = format!("C:\\repo\\file.rs\n{body}");
    let list_height = 400;
    let line_height = 20;
    let layout = picker_preview_layout(Some(&preview), 0, 900, 0, list_height, line_height, 0.5)
        .expect("preview layout should exist");

    let expected_rows = (list_height / line_height) as usize;
    assert_eq!(layout.height, list_height as u32);
    assert_eq!(layout.lines.len(), expected_rows);
    assert_eq!(layout.lines[0], "C:\\repo\\file.rs");
    assert_eq!(
        layout.lines[expected_rows - 1],
        format!("line-{}", expected_rows - 2)
    );
}

#[test]
fn picker_fringe_path_for_color_accepts_filesystem_paths_only() {
    assert!(picker_fringe_path_for_color(r"P:\volt\src\main.rs").is_some());
    assert!(picker_fringe_path_for_color("src/main.rs").is_some());
    assert!(picker_fringe_path_for_color(r"src\main.rs").is_some());
    assert!(picker_fringe_path_for_color("workspace.files").is_none());
    assert!(picker_fringe_path_for_color("").is_none());
}

#[test]
fn picker_fringe_color_uses_oil_filetype_token() {
    let rust = picker_fringe_color(None, r"P:\volt\src\main.rs", Color::RGB(1, 2, 3));
    let lock = picker_fringe_color(None, r"P:\volt\Cargo.lock", Color::RGB(1, 2, 3));
    // Without a theme registry both fall back, but path classification still runs.
    assert_eq!(rust, Color::RGB(1, 2, 3));
    assert_eq!(lock, Color::RGB(1, 2, 3));
    assert_eq!(
        oil_entry_theme_token(Path::new(r"P:\volt\src\main.rs"), false),
        "oil.file.code"
    );
    assert_eq!(
        oil_entry_theme_token(Path::new(r"P:\volt\Cargo.lock"), false),
        "oil.file.lock"
    );
}

#[test]
fn picker_preview_layout_honors_split_fraction() {
    let middle =
        picker_preview_layout(Some("preview"), 0, 900, 0, 300, 20, 0.5).expect("middle split");
    let left = picker_preview_layout(Some("preview"), 0, 900, 0, 300, 20, 0.3).expect("left split");
    let right =
        picker_preview_layout(Some("preview"), 0, 900, 0, 300, 20, 0.7).expect("right split");

    assert!(left.list_width < middle.list_width);
    assert!(middle.list_width < right.list_width);
    assert!(left.x < middle.x);
    assert!(middle.x < right.x);
    assert!(!middle.stub);
}

#[test]
fn picker_preview_layout_shows_stub_when_narrow() {
    let layout = picker_preview_layout(Some("path\nline"), 0, 400, 0, 300, 20, 0.5)
        .expect("narrow preview should still reserve a pane");
    assert!(layout.stub);
    assert_eq!(
        layout.lines,
        vec!["Preview unavailable (narrow window)".to_owned()]
    );
}

#[test]
fn picker_loading_label_animates_known_rows() {
    let samples = [
        "Loading code actions",
        "Loading Definitions",
        "Loading References",
        "Loading Implementations",
        "Loading branches…",
        "Loading…",
        "Loading...",
    ];
    for label in samples {
        let first = picker_loading_label(label, 0).expect(label);
        let second = picker_loading_label(label, 1).expect(label);
        assert_ne!(first.spinner, second.spinner, "{label}");
        assert_ne!(first.text, second.text, "{label}");
        assert!(first.text.starts_with("Loading"), "{label}");
        assert!(!first.text.contains('…'), "{label}");
        assert_eq!(
            first.text.chars().count(),
            second.text.chars().count(),
            "{label}"
        );
        assert!(
            super::super::render::SPINNER_FRAMES.contains(&first.spinner),
            "{label}"
        );
    }

    let width = picker_loading_label("Loading code actions", 0)
        .expect("loading label")
        .text
        .chars()
        .count();
    for frame in 0..12 {
        let animated = picker_loading_label("Loading code actions", frame).expect("frame");
        assert_eq!(animated.text.chars().count(), width);
    }

    assert!(picker_loading_label("Save and Close", 0).is_none());
    assert!(picker_loading_label("LoadingScreen", 0).is_none());
    assert!(picker_loading_label("No diagnostics available", 0).is_none());
}

#[test]
fn picker_loading_animation_frame_tracks_pending_rows() {
    let loading = PickerOverlay::from_entries(
        "Code Actions",
        vec![PickerEntry {
            item: PickerItem::new(
                "lsp-code-action-status",
                "Loading code actions",
                "Waiting for language server response.",
                None::<String>,
            ),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    );
    assert!(picker_loading_animation_frame(Some(&loading)).is_some());

    let idle = PickerOverlay::from_entries(
        "Code Actions",
        vec![PickerEntry {
            item: PickerItem::new("save", "Save and Close", "write", None::<String>),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    );
    assert!(picker_loading_animation_frame(Some(&idle)).is_none());
    assert!(picker_loading_animation_frame(None).is_none());
}

#[test]
fn picker_body_layout_extends_list_to_the_footer() {
    let popup = PixelRect {
        x: 10,
        y: 20,
        width: 800,
        height: 700,
    };
    let line_height = 22;
    let layout = picker_body_layout(popup, line_height);
    let footer_y = popup.y + popup.height as i32 - line_height - 12;

    assert_eq!(layout.footer_y, footer_y);
    assert_eq!(layout.list_top + layout.list_height, footer_y - 12);
    let old_height = popup.height as i32 - ((line_height * 5) + 82);
    assert!(layout.list_height > old_height);
}

#[test]
fn picker_preview_lines_fills_height_around_target_marker() {
    let mut lines = vec!["C:\\repo\\file.rs".to_owned()];
    for index in 0..40 {
        let marker = if index == 30 { ">" } else { " " };
        lines.push(format!("{marker} {:>4} line-{index}", index + 1));
    }
    let window = picker_preview_lines(&lines.join("\n"), 11);

    assert_eq!(window.len(), 11);
    assert_eq!(window[0], "C:\\repo\\file.rs");
    assert!(window.iter().any(|line| line.contains("line-30")));
    assert!(!window[1].contains("line-0"));
}

#[test]
fn picker_preview_lines_keeps_file_prefix_when_unmarked() {
    let body = (0..30)
        .map(|index| format!("line-{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let window = picker_preview_lines(&format!("C:\\repo\\file.rs\n{body}"), 5);

    assert_eq!(
        window,
        vec![
            "C:\\repo\\file.rs".to_owned(),
            "line-0".to_owned(),
            "line-1".to_owned(),
            "line-2".to_owned(),
            "line-3".to_owned(),
        ]
    );
}

#[test]
fn picker_footer_hint_appends_extra_chords() {
    assert_eq!(
        picker_footer_hint(&[]),
        "↑↓ navigate · Enter accept · Esc close"
    );
    let extras = [PickerExtraKeybind::new("Ctrl+d", "workspace.delete")];
    assert_eq!(
        picker_footer_hint(&extras),
        "↑↓ navigate · Enter accept · Esc close · Ctrl+d"
    );
}
