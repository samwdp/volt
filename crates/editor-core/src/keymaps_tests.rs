
use super::*;

#[test]
fn normalize_chord_reorders_modifiers_and_aliases() {
    assert_eq!(normalize_chord("Shift+Ctrl+I"), "Ctrl+Shift+I");
    assert_eq!(normalize_chord("Gui+Shift+Ctrl+P"), "Ctrl+Shift+Gui+P");
    assert_eq!(normalize_chord("S-F5"), "Shift+F5");
    assert_eq!(normalize_chord("M-x scratch"), "Alt+x scratch");
    assert_eq!(normalize_chord("C-c C-c"), "Ctrl+c Ctrl+c");
    assert_eq!(normalize_chord("g g"), "g g");
}

#[test]
fn registry_resolves_equivalent_sequence_spellings() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register(
        "M-x scratch",
        "workspace.open-scratch",
        KeymapScope::Global,
        CommandSource::Core,
    )?;

    assert!(registry.contains(&KeymapScope::Global, "Alt+x scratch"));
    assert!(registry.contains(&KeymapScope::Global, "M-x scratch"));
    assert_eq!(
        registry
            .get(&KeymapScope::Global, "Alt+x scratch")
            .map(KeyBinding::chord),
        Some("Alt+x scratch")
    );
    assert!(registry.has_sequence_prefix_for_mode(
        &KeymapScope::Global,
        KeymapVimMode::Any,
        &[String::from("Alt+x")],
    ));
    assert!(registry.has_sequence_prefix_for_mode(
        &KeymapScope::Global,
        KeymapVimMode::Any,
        &[String::from("M-x")],
    ));

    Ok(())
}

#[test]
fn duplicate_detection_uses_canonical_chords() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register(
        "Shift+F5",
        "workspace.compile",
        KeymapScope::Global,
        CommandSource::Core,
    )?;

    let error = registry
        .register(
            "S-F5",
            "workspace.recompile",
            KeymapScope::Global,
            CommandSource::Core,
        )
        .expect_err("legacy alias should conflict with canonical chord");

    assert_eq!(
        error,
        KeymapError::DuplicateBinding {
            scope: KeymapScope::Global,
            vim_mode: KeymapVimMode::Any,
            chord: "Shift+F5".to_owned(),
        }
    );

    Ok(())
}

#[test]
fn workspace_minor_mode_overrides_global_for_same_chord() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register(
        "Ctrl+n",
        "global.fallback",
        KeymapScope::Global,
        CommandSource::Core,
    )?;
    registry.register(
        "Ctrl+n",
        "workspace.marked-1",
        KeymapScope::Workspace,
        CommandSource::Core,
    )?;

    let binding = registry
        .resolve_with_minor_modes(&[KeymapScope::Workspace], KeymapVimMode::Any, "Ctrl+n")
        .expect("Workspace Minor Mode should claim chord");

    assert_eq!(binding.command_name(), "workspace.marked-1");
    Ok(())
}

#[test]
fn global_is_fallback_when_no_minor_mode_claims_chord() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register(
        "Ctrl+n",
        "global.fallback",
        KeymapScope::Global,
        CommandSource::Core,
    )?;
    registry.register(
        "Ctrl+e",
        "workspace.marked-2",
        KeymapScope::Workspace,
        CommandSource::Core,
    )?;

    let binding = registry
        .resolve_with_minor_modes(&[KeymapScope::Workspace], KeymapVimMode::Any, "Ctrl+n")
        .expect("Global fallback should claim unclaimed chord");

    assert_eq!(binding.command_name(), "global.fallback");
    Ok(())
}

#[test]
fn popup_overrides_workspace_and_global_while_active() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register(
        "Ctrl+n",
        "global.fallback",
        KeymapScope::Global,
        CommandSource::Core,
    )?;
    registry.register(
        "Ctrl+n",
        "workspace.marked-1",
        KeymapScope::Workspace,
        CommandSource::Core,
    )?;
    registry.register(
        "Ctrl+n",
        "popup.next",
        KeymapScope::Popup,
        CommandSource::Core,
    )?;

    let binding = registry
        .resolve_with_minor_modes(
            &[KeymapScope::Popup, KeymapScope::Workspace],
            KeymapVimMode::Any,
            "Ctrl+n",
        )
        .expect("Popup Minor Mode should win");

    assert_eq!(binding.command_name(), "popup.next");
    Ok(())
}

#[test]
fn autocomplete_overrides_workspace_while_active() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register_for_mode(
        "Ctrl+n",
        "workspace.marked-1",
        KeymapScope::Workspace,
        KeymapVimMode::Insert,
        CommandSource::Core,
    )?;
    registry.register_for_mode(
        "Ctrl+n",
        "autocomplete.next",
        KeymapScope::Autocomplete,
        KeymapVimMode::Insert,
        CommandSource::Core,
    )?;

    let binding = registry
        .resolve_with_minor_modes(
            &[KeymapScope::Autocomplete, KeymapScope::Workspace],
            KeymapVimMode::Insert,
            "Ctrl+n",
        )
        .expect("Autocomplete Minor Mode should win");

    assert_eq!(binding.command_name(), "autocomplete.next");
    Ok(())
}

#[test]
fn hover_overrides_workspace_while_active() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register_for_mode(
        "Ctrl+n",
        "workspace.marked-1",
        KeymapScope::Workspace,
        KeymapVimMode::Normal,
        CommandSource::Core,
    )?;
    registry.register_for_mode(
        "Ctrl+n",
        "hover.next",
        KeymapScope::Hover,
        KeymapVimMode::Normal,
        CommandSource::Core,
    )?;

    let binding = registry
        .resolve_with_minor_modes(
            &[KeymapScope::Hover, KeymapScope::Workspace],
            KeymapVimMode::Normal,
            "Ctrl+n",
        )
        .expect("Hover Minor Mode should win");

    assert_eq!(binding.command_name(), "hover.next");
    Ok(())
}

#[test]
fn dap_mode_overrides_global_f5_while_session_live() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register("F5", "dap.start", KeymapScope::Global, CommandSource::Core)?;
    registry.register("F5", "dap.continue", KeymapScope::Dap, CommandSource::Core)?;

    let continued = registry
        .resolve_with_minor_modes(&[KeymapScope::Dap], KeymapVimMode::Any, "F5")
        .expect("DAP Mode should claim F5");
    assert_eq!(continued.command_name(), "dap.continue");

    let started = registry
        .resolve_with_minor_modes(&[], KeymapVimMode::Any, "F5")
        .expect("Global F5 should start when DAP Mode is inactive");
    assert_eq!(started.command_name(), "dap.start");
    Ok(())
}

#[test]
fn popup_mode_does_not_claim_workspace_dock_chords() -> Result<(), KeymapError> {
    let mut registry = KeymapRegistry::new();
    registry.register(
        "j",
        "workspace.dock.next",
        KeymapScope::WorkspaceDock,
        CommandSource::Core,
    )?;
    registry.register(
        "j",
        "vim.move-down",
        KeymapScope::Workspace,
        CommandSource::Core,
    )?;

    assert!(
        registry
            .find_in_scopes(&[KeymapScope::Popup], KeymapVimMode::Any, "j")
            .is_none(),
        "Popup Minor Mode must not claim Workspace Dock j"
    );
    let dock = registry
        .resolve_with_minor_modes(&[KeymapScope::WorkspaceDock], KeymapVimMode::Any, "j")
        .expect("Workspace Dock Minor Mode should claim j");
    assert_eq!(dock.command_name(), "workspace.dock.next");
    Ok(())
}
