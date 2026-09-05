
use super::*;

#[test]
fn matching_extra_resolves_command_close_and_selected_context() {
    let extras = [PickerExtraKeybind::new(
        "Ctrl+d",
        "workspace.worktree-remove",
    )];
    let selected = PickerSelectedRow::new(r"P:\repo\feature", "feature", Some(r"P:\repo\feature"));

    let outcome = resolve_picker_extra(&extras, "Ctrl+d", Some(selected.clone()), Vec::new());

    assert_eq!(
        outcome,
        PickerExtraDispatch::Fire {
            command_name: "workspace.worktree-remove".to_owned(),
            context: PickerOneShotContext::new(Some(selected), Vec::new()),
            close_picker: true,
        }
    );
}

#[test]
fn non_extra_chord_falls_through_for_shared_popup_bindings() {
    let extras = [PickerExtraKeybind::new(
        "Ctrl+d",
        "workspace.worktree-remove",
    )];
    let selected = PickerSelectedRow::new(r"P:\repo\feature", "feature", Some(r"P:\repo\feature"));

    let outcome = resolve_picker_extra(&extras, "Ctrl+n", Some(selected), Vec::new());

    assert_eq!(outcome, PickerExtraDispatch::Fallthrough);
}

#[test]
fn empty_selection_still_yields_defined_context_for_command_noop() {
    let extras = [PickerExtraKeybind::new("Ctrl+q", "quickfix.open")];

    let outcome = resolve_picker_extra(&extras, "Ctrl+q", None, Vec::new());

    assert_eq!(
        outcome,
        PickerExtraDispatch::Fire {
            command_name: "quickfix.open".to_owned(),
            context: PickerOneShotContext::new(None, Vec::new()),
            close_picker: true,
        }
    );
}

#[test]
fn create_row_selection_still_yields_defined_context_for_command_noop() {
    let extras = [PickerExtraKeybind::new(
        "Ctrl+d",
        "workspace.worktree-remove",
    )];
    let selected = PickerSelectedRow::new(
        "git-worktree-dashboard:create",
        "+ new worktree",
        None::<String>,
    );

    let outcome = resolve_picker_extra(&extras, "Ctrl+d", Some(selected.clone()), Vec::new());

    assert_eq!(
        outcome,
        PickerExtraDispatch::Fire {
            command_name: "workspace.worktree-remove".to_owned(),
            context: PickerOneShotContext::new(Some(selected), Vec::new()),
            close_picker: true,
        }
    );
}

#[test]
fn matching_extra_snapshots_exportable_quickfix_rows() {
    let extras = [PickerExtraKeybind::new("Ctrl+q", "quickfix.open")];
    let selected = PickerSelectedRow::new("hit:0", "main.rs:1:4", Some("main.rs"));
    let rows = vec![
        PickerExportableRow::new("hit:0", "main.rs", 0, 3, "fn alpha() {}"),
        PickerExportableRow::new("hit:1", "lib.rs", 0, 3, "fn beta() {}"),
    ];

    let outcome = resolve_picker_extra(&extras, "Ctrl+q", Some(selected.clone()), rows.clone());

    assert_eq!(
        outcome,
        PickerExtraDispatch::Fire {
            command_name: "quickfix.open".to_owned(),
            context: PickerOneShotContext::new(Some(selected), rows),
            close_picker: true,
        }
    );
}
