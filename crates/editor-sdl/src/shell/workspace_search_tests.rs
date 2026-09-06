use super::*;

fn picker_entry(id: &str, label: &str) -> PickerEntry {
    PickerEntry {
        item: PickerItem::new(id, label, label, None::<&str>),
        action: PickerAction::NoOp,
        quickfix: None,
    }
}

#[test]
fn lsp_code_action_kind_sorting_prefers_specific_matches_and_stays_stable() {
    let sorted = lsp_code_action_sorted_indices([
        Some("source.fixAll.eslint"),
        Some("refactor.move"),
        Some("quickfix"),
        Some("source.organizeImports.biome"),
        Some("source"),
        Some("refactor.inline.foo"),
        Some("refactor.rewrite"),
        Some("refactor"),
        Some("refactor.extract"),
        Some("custom"),
        None,
    ]);

    assert_eq!(sorted, vec![2, 1, 7, 5, 8, 6, 4, 3, 0, 9, 10]);
}

#[test]
fn lsp_code_action_picker_overlay_preserves_sorted_entry_order() {
    let overlay = lsp_code_actions_picker_overlay_from_entries(vec![
        picker_entry("z", "zeta"),
        picker_entry("a", "alpha"),
        picker_entry("m", "mu"),
    ]);

    assert_eq!(
        overlay
            .session()
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>(),
        vec!["zeta", "alpha", "mu"]
    );
}

#[test]
fn lsp_diagnostics_picker_overlay_orders_errors_first() {
    let diagnostics = vec![
        LspWorkspaceDiagnostic::new(
            "rust-analyzer",
            PathBuf::from("src").join("main.rs"),
            editor_lsp::Diagnostic::new(
                "rustc",
                "warning message",
                WorkspaceDiagnosticSeverity::Warning,
                TextRange::new(TextPoint::new(7, 2), TextPoint::new(7, 5)),
            ),
        ),
        LspWorkspaceDiagnostic::new(
            "rust-analyzer",
            PathBuf::from("src").join("lib.rs"),
            editor_lsp::Diagnostic::new(
                "rustc",
                "error message",
                WorkspaceDiagnosticSeverity::Error,
                TextRange::new(TextPoint::new(2, 4), TextPoint::new(2, 6)),
            ),
        ),
        LspWorkspaceDiagnostic::new(
            "biome",
            PathBuf::from("src").join("app.rs"),
            editor_lsp::Diagnostic::new(
                "biome",
                "info message",
                WorkspaceDiagnosticSeverity::Information,
                TextRange::new(TextPoint::new(1, 0), TextPoint::new(1, 3)),
            ),
        ),
    ];

    let entries = lsp_diagnostics_picker_entries(None, &diagnostics);
    assert!(matches!(
        &entries[0].action,
        PickerAction::OpenFileLocation { path, target }
            if path == &PathBuf::from("src").join("lib.rs")
                && *target == TextPoint::new(2, 4)
    ));

    let overlay = lsp_diagnostics_picker_overlay_from_entries(entries);
    assert_eq!(
        overlay
            .session()
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>(),
        vec![
            "Error: error message",
            "Warning: warning message",
            "Info: info message",
        ]
    );
}

#[test]
fn workspace_search_match_entry_builds_open_file_location_action() {
    let root = Path::new("C:\\workspace");
    let entry = workspace_search_match_entry(root, ".\\src\\main.rs", 7, 5, "fn main() {}");

    assert_eq!(entry.item.label(), "fn main() {}");
    assert_eq!(entry.item.detail(), "src\\main.rs | Ln 7, Col 5");
    assert_eq!(
        entry.item.preview().map(ToOwned::to_owned),
        Some(root.join("src\\main.rs").display().to_string())
    );
    assert!(matches!(
        entry.action,
        PickerAction::OpenFileLocation { path, target }
            if path == root.join("src\\main.rs")
                && target == TextPoint::new(6, 4)
    ));
}

#[test]
fn file_context_preview_marks_target_line() -> Result<(), Box<dyn std::error::Error>> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let root = std::env::temp_dir().join(format!("volt-picker-preview-{unique}"));
    std::fs::create_dir_all(&root)?;
    let path = root.join("main.rs");
    std::fs::write(&path, "one\ntwo\nthree\nfour\nfive\n")?;

    let preview =
        file_context_preview(&path, TextPoint::new(2, 0)).ok_or("missing file context preview")?;

    assert!(preview.contains(">    3 three"));
    assert!(preview.contains("     2 two"));
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn workspace_search_status_entry_is_noop() {
    let entry = workspace_search_status_entry(
        "needle",
        "No matches found",
        "No workspace results for `needle`.",
        Some("C:\\workspace".to_owned()),
    );

    assert_eq!(entry.item.id(), "workspace-search:needle");
    assert_eq!(entry.item.label(), "No matches found for needle");
    assert_eq!(entry.item.detail(), "No workspace results for `needle`.");
    assert!(matches!(entry.action, PickerAction::NoOp));
}
