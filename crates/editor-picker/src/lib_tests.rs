use super::{PickerItem, PickerResultOrder, PickerSession};

fn item(id: &str, label: &str) -> PickerItem {
    PickerItem::new(id, label, label, None::<&str>)
}

#[test]
fn empty_query_returns_all_items_in_sorted_order() {
    let session = PickerSession::new(
        "Commands",
        vec![item("b", "buffer.save"), item("a", "terminal.open")],
    );

    assert_eq!(session.match_count(), 2);
    assert_eq!(
        session
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>(),
        vec!["buffer.save", "terminal.open"]
    );
}

#[test]
fn source_order_preserves_input_order() {
    let session = PickerSession::new(
        "Commands",
        vec![item("z", "zeta"), item("a", "alpha"), item("m", "mu")],
    )
    .with_result_order(PickerResultOrder::Source);

    assert_eq!(
        session
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>(),
        vec!["zeta", "alpha", "mu"]
    );
}

#[test]
fn fuzzy_query_prefers_prefix_and_contiguous_matches() {
    let mut session = PickerSession::new(
        "Commands",
        vec![
            item("term", "terminal.open"),
            item("term-short", "term.open"),
            item("tabs", "workspace.open-scratch"),
        ],
    );
    session.set_query("term");

    let labels = session
        .matches()
        .iter()
        .map(|matched| matched.item().label())
        .collect::<Vec<_>>();
    assert_eq!(labels[0], "term.open");
    assert!(labels.contains(&"terminal.open"));
}

#[test]
fn contiguous_substring_beats_split_path_match() {
    let mut session = PickerSession::new(
        "Files",
        vec![
            item(
                "asset-model",
                "src/AssetFusion.Shared/Model/StaticData/AssetDefinition.cs",
            ),
            item(
                "report-model",
                "src/AssetFusion.Shared/Model/StaticData/ReportDefinition.cs",
            ),
            item(
                "asset-service",
                "src/AssetFusion.Shared/Services/Sql/AssetDefinitionService.cs",
            ),
            item(
                "vehicle-model",
                "src/AssetFusion.Shared/Model/StaticData/VehicleDefinition.cs",
            ),
        ],
    );
    session.set_query("assetdefinition");

    let labels = session
        .matches()
        .iter()
        .map(|matched| matched.item().label())
        .collect::<Vec<_>>();
    let first_non_asset = labels
        .iter()
        .position(|label| !label.contains("AssetDefinition"))
        .expect("expected weaker non-asset fuzzy matches");

    assert!(
        labels[..first_non_asset]
            .iter()
            .all(|label| label.contains("AssetDefinition"))
    );
}

#[test]
fn selection_wraps_across_match_list() {
    let mut session = PickerSession::new(
        "Commands",
        vec![item("a", "alpha"), item("b", "beta"), item("c", "gamma")],
    );

    assert_eq!(session.selected().map(|item| item.item().id()), Some("a"));
    session.select_previous();
    assert_eq!(session.selected().map(|item| item.item().id()), Some("c"));
    session.select_next();
    assert_eq!(session.selected().map(|item| item.item().id()), Some("a"));
}

#[test]
fn result_limit_caps_large_match_sets() {
    let items = (0..128)
        .map(|index| item("cmd", &format!("command-{index:03}")))
        .collect::<Vec<_>>();
    let mut session = PickerSession::new("Commands", items).with_result_limit(16);
    session.set_query("command");

    assert_eq!(session.match_count(), 16);
}

#[test]
fn whitespace_query_matches_multiple_terms() {
    let items = vec![
        item("pick-mode", "acp.pick-mode"),
        item("cycle-mode", "acp.cycle-mode"),
        item("workspace", "workspace.list-files"),
    ];
    let mut session = PickerSession::new("Commands", items);

    session.set_query("acp mode");

    assert_eq!(session.match_count(), 2);
    assert!(
        session
            .matches()
            .iter()
            .any(|matched| matched.item().label() == "acp.pick-mode")
    );
    assert!(
        session
            .matches()
            .iter()
            .any(|matched| matched.item().label() == "acp.cycle-mode")
    );
}

#[test]
fn whitespace_query_requires_all_terms() {
    let items = vec![
        item("pick-mode", "acp.pick-mode"),
        item("workspace", "workspace.list-files"),
    ];
    let mut session = PickerSession::new("Commands", items);

    session.set_query("acp files");

    assert_eq!(session.match_count(), 0);
}

#[test]
fn fringe_metadata_survives_matching() {
    let session = PickerSession::new(
        "Files",
        vec![item("alpha", "src/main.rs").with_fringe("icon")],
    );

    assert_eq!(
        session
            .selected()
            .and_then(|selected| selected.item().fringe()),
        Some("icon")
    );
}

#[test]
fn divider_visible_with_empty_query_and_hidden_when_filtering() {
    let mut session = PickerSession::new(
        "Workspaces",
        vec![
            item("open", "default"),
            PickerItem::divider("divider"),
            item("project", "volt"),
        ],
    )
    .with_result_order(PickerResultOrder::Source);

    assert_eq!(session.match_count(), 3);
    assert!(
        session
            .matches()
            .iter()
            .any(|matched| matched.item().is_divider())
    );

    session.set_query("volt");
    assert_eq!(session.match_count(), 1);
    assert!(
        session
            .matches()
            .iter()
            .all(|matched| !matched.item().is_divider())
    );
}

#[test]
fn query_change_resets_selection_to_first_match() {
    let mut session = PickerSession::new(
        "Commands",
        vec![item("a", "alpha"), item("b", "beta"), item("c", "gamma")],
    );

    session.select_next();
    session.select_next();
    assert_eq!(session.selected().map(|item| item.item().id()), Some("c"));

    session.set_query("a");
    assert_eq!(session.selected().map(|item| item.item().id()), Some("a"));

    session.set_query("b");
    assert_eq!(session.selected().map(|item| item.item().id()), Some("b"));
}

#[test]
fn selection_skips_divider_rows() {
    let mut session = PickerSession::new(
        "Workspaces",
        vec![
            item("open", "default"),
            PickerItem::divider("divider"),
            item("project", "volt"),
        ],
    )
    .with_result_order(PickerResultOrder::Source);

    assert_eq!(
        session.selected().map(|item| item.item().id()),
        Some("open")
    );
    session.select_next();
    assert_eq!(
        session.selected().map(|item| item.item().id()),
        Some("project")
    );
    session.select_previous();
    assert_eq!(
        session.selected().map(|item| item.item().id()),
        Some("open")
    );
}

#[test]
fn custom_search_text_matches_hidden_path_segments() {
    let mut session = PickerSession::new(
        "Files",
        vec![item("main", "main.rs").with_search_text("src/deep/nested/main.rs")],
    );

    session.set_query("deep nested");

    assert_eq!(session.match_count(), 1);
    assert_eq!(
        session.selected().map(|matched| matched.item().label()),
        Some("main.rs")
    );
}

#[test]
fn set_item_preview_updates_selected_match_without_filling_other_rows() {
    let mut session =
        PickerSession::new("Files", vec![item("a", "alpha.rs"), item("b", "beta.rs")]);

    session.set_item_preview("a", "fn alpha() {}");
    assert_eq!(
        session
            .selected()
            .and_then(|matched| matched.item().preview()),
        Some("fn alpha() {}")
    );
    assert!(
        session
            .matches()
            .iter()
            .find(|matched| matched.item().id() == "b")
            .and_then(|matched| matched.item().preview())
            .is_none()
    );
}

#[test]
fn empty_query_with_result_limit_truncates() {
    let items = vec![
        item("c", "charlie"),
        item("a", "alpha"),
        item("b", "bravo"),
        item("d", "delta"),
    ];
    let session = PickerSession::new("Commands", items).with_result_limit(2);

    assert_eq!(session.match_count(), 2);
    assert_eq!(
        session
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>(),
        vec!["alpha", "bravo"]
    );
}

#[test]
fn source_order_result_limit_truncates_without_reordering() {
    let session = PickerSession::new(
        "Projects",
        vec![
            item("z", "zeta"),
            item("a", "alpha"),
            item("m", "mu"),
            item("b", "beta"),
        ],
    )
    .with_result_order(PickerResultOrder::Source)
    .with_result_limit(2);

    assert_eq!(session.match_count(), 2);
    assert_eq!(
        session
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>(),
        vec!["zeta", "alpha"]
    );
}

#[test]
fn capped_match_set_does_not_require_cloning_losing_row_previews() {
    let items = (0..48)
        .map(|index| {
            PickerItem::new(
                format!("f{index:03}"),
                format!("file-{index:03}.rs"),
                "src",
                Some(format!("preview-{index:03}")),
            )
        })
        .collect::<Vec<_>>();
    let mut session = PickerSession::new("Files", items).with_result_limit(8);
    session.set_query("file");

    assert_eq!(session.match_count(), 8);
    assert_eq!(session.item_count(), 48);
    assert!(
        session
            .matches()
            .iter()
            .all(|matched| matched.item().id() != "f047")
    );
    assert_eq!(
        session.source_item("f047").and_then(PickerItem::preview),
        Some("preview-047")
    );
    assert!(
        session
            .matches()
            .iter()
            .any(|matched| matched.item().preview() == Some("preview-000"))
    );
}

#[test]
fn set_items_preserves_selected_id_when_still_matched() {
    let mut session = PickerSession::new(
        "Commands",
        vec![item("a", "alpha"), item("b", "beta"), item("c", "gamma")],
    );
    session.select_next();
    assert_eq!(
        session.selected().map(|matched| matched.item().id()),
        Some("b")
    );

    session.set_items(vec![
        item("a", "alpha"),
        item("b", "beta"),
        item("c", "gamma"),
        item("d", "delta"),
    ]);

    assert_eq!(
        session.selected().map(|matched| matched.item().id()),
        Some("b")
    );
}

#[test]
fn matched_positions_follow_search_text_characters() {
    let mut session = PickerSession::new(
        "Files",
        vec![item("main", "main.rs").with_search_text("src/main.rs")],
    );
    session.set_query("main");

    assert_eq!(
        session
            .selected()
            .map(|matched| matched.matched_positions().to_vec()),
        Some(vec![4, 5, 6, 7])
    );
}

#[test]
fn select_next_walks_retained_matches() {
    let items = (0..16)
        .map(|index| item(&format!("i{index:02}"), &format!("item-{index:02}")))
        .collect::<Vec<_>>();
    let mut session = PickerSession::new("Commands", items).with_result_limit(3);

    assert_eq!(session.match_count(), 3);
    session.select_next();
    session.select_next();
    session.select_next();
    assert_eq!(
        session.selected().map(|matched| matched.item().label()),
        Some("item-00")
    );
}
