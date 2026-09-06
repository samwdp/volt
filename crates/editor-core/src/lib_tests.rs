
use std::path::PathBuf;

use super::{
    BufferKind, CommandSource, EditorRuntime, HookEvent, KeymapScope, KeymapVimMode, ModelError,
    builtins, runtime_descriptor,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThemeService(&'static str);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct EventLog(Vec<String>);

#[test]
fn runtime_descriptor_matches_expected_foundation_values() {
    let descriptor = runtime_descriptor();
    assert_eq!(descriptor.application_name, "volt");
    assert_eq!(descriptor.plugin_abi, "abi_stable");
}

#[test]
fn runtime_bootstrap_tracks_editor_graph_and_services() -> Result<(), ModelError> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)?;
    let scratch_buffer =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)?;
    let command_buffer =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*commands*", BufferKind::Picker, None)?;
    let popup_id = runtime.model_mut().open_popup(
        workspace_id,
        "Command Palette",
        vec![scratch_buffer, command_buffer],
        command_buffer,
    )?;

    runtime.services_mut().insert(ThemeService("default"));

    let workspace = runtime.model().workspace(workspace_id)?;

    assert_eq!(runtime.model().window_count(), 1);
    assert_eq!(workspace.pane_count(), 1);
    assert_eq!(workspace.buffer_count(), 2);
    assert_eq!(workspace.popup_count(), 1);
    assert_eq!(
        workspace
            .active_pane()
            .and_then(|pane| pane.active_buffer()),
        Some(command_buffer)
    );
    assert_eq!(
        workspace.popup(popup_id).map(|popup| popup.active_buffer()),
        Some(command_buffer)
    );
    assert_eq!(
        runtime.services().get::<ThemeService>(),
        Some(&ThemeService("default"))
    );

    Ok(())
}

#[test]
fn command_registry_executes_commands_and_hooks_dispatch_events() -> Result<(), String> {
    let mut runtime = EditorRuntime::new();
    runtime.services_mut().insert(EventLog::default());

    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)
        .map_err(|error| error.to_string())?;

    runtime
        .register_hook(
            "user.after-open-scratch",
            "Runs after the scratch buffer command completes.",
        )
        .map_err(|error| error.to_string())?;

    runtime
        .subscribe_hook(
            builtins::WORKSPACE_OPEN,
            "core.workspace-open-log",
            |event, runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<EventLog>()
                    .ok_or_else(|| "event log service missing".to_owned())?;
                let workspace = event
                    .workspace_id
                    .map(|workspace_id| workspace_id.get())
                    .unwrap_or_default();
                log.0.push(format!("workspace-open:{workspace}"));
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;

    runtime
        .subscribe_hook(
            "user.after-open-scratch",
            "user.after-open-scratch-log",
            |event, runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<EventLog>()
                    .ok_or_else(|| "event log service missing".to_owned())?;
                let detail = event.detail.as_deref().unwrap_or("unknown");
                log.0.push(format!("after-open-scratch:{detail}"));
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;

    runtime
        .register_command(
            "workspace.open-scratch",
            "Create a scratch buffer and emit a follow-up hook.",
            CommandSource::Core,
            move |runtime| {
                let buffer_id = runtime
                    .model_mut()
                    .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)
                    .map_err(|error| error.to_string())?;

                runtime
                    .emit_hook(
                        "user.after-open-scratch",
                        HookEvent::new()
                            .with_workspace(workspace_id)
                            .with_buffer(buffer_id)
                            .with_detail("scratch"),
                    )
                    .map_err(|error| error.to_string())?;

                let log = runtime
                    .services_mut()
                    .get_mut::<EventLog>()
                    .ok_or_else(|| "event log service missing".to_owned())?;
                log.0.push("command:workspace.open-scratch".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;

    runtime
        .register_key_binding(
            "Alt+x scratch",
            "workspace.open-scratch",
            KeymapScope::Global,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    runtime
        .emit_hook(
            builtins::WORKSPACE_OPEN,
            HookEvent::new()
                .with_window(window_id)
                .with_workspace(workspace_id),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .execute_key_binding(&KeymapScope::Global, "Alt+x scratch")
        .map_err(|error| error.to_string())?;

    let log = runtime
        .services()
        .get::<EventLog>()
        .ok_or_else(|| "event log service missing".to_owned())?;

    assert_eq!(
        log.0,
        vec![
            "workspace-open:1".to_owned(),
            "after-open-scratch:scratch".to_owned(),
            "command:workspace.open-scratch".to_owned(),
        ]
    );
    assert!(runtime.commands().contains("workspace.open-scratch"));
    assert!(runtime.hooks().contains("user.after-open-scratch"));
    assert!(
        runtime
            .keymaps()
            .contains(&KeymapScope::Global, "Alt+x scratch")
    );

    Ok(())
}

#[test]
fn runtime_executes_keybindings_registered_with_legacy_aliases() -> Result<(), String> {
    let mut runtime = EditorRuntime::new();
    runtime.services_mut().insert(EventLog::default());

    runtime
        .register_command(
            "workspace.open-scratch",
            "Open a scratch buffer.",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<EventLog>()
                    .ok_or_else(|| "event log service missing".to_owned())?;
                log.0.push("command:workspace.open-scratch".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .register_key_binding(
            "M-x scratch",
            "workspace.open-scratch",
            KeymapScope::Global,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    runtime
        .execute_key_binding(&KeymapScope::Global, "Alt+x scratch")
        .map_err(|error| error.to_string())?;

    let log = runtime
        .services()
        .get::<EventLog>()
        .ok_or_else(|| "event log service missing".to_owned())?;

    assert_eq!(log.0, vec!["command:workspace.open-scratch".to_owned()]);
    assert!(
        runtime
            .keymaps()
            .contains(&KeymapScope::Global, "Alt+x scratch")
    );
    assert!(
        runtime
            .keymaps()
            .contains(&KeymapScope::Global, "M-x scratch")
    );
    assert_eq!(
        runtime
            .keymaps()
            .get(&KeymapScope::Global, "Alt+x scratch")
            .map(|binding| binding.chord()),
        Some("Alt+x scratch")
    );

    Ok(())
}

#[test]
fn runtime_resolves_mode_specific_keybindings() -> Result<(), String> {
    let mut runtime = EditorRuntime::new();
    runtime.services_mut().insert(EventLog::default());

    runtime
        .register_command(
            "vim.normal-x",
            "Normal mode x",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<EventLog>()
                    .ok_or_else(|| "event log service missing".to_owned())?;
                log.0.push("normal-x".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .register_command(
            "vim.visual-x",
            "Visual mode x",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<EventLog>()
                    .ok_or_else(|| "event log service missing".to_owned())?;
                log.0.push("visual-x".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .register_command("vim.undo", "Undo", CommandSource::Core, |runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<EventLog>()
                .ok_or_else(|| "event log service missing".to_owned())?;
            log.0.push("undo".to_owned());
            Ok(())
        })
        .map_err(|error| error.to_string())?;

    runtime
        .register_key_binding_for_mode(
            "x",
            "vim.normal-x",
            KeymapScope::Workspace,
            KeymapVimMode::Normal,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;
    runtime
        .register_key_binding_for_mode(
            "x",
            "vim.visual-x",
            KeymapScope::Workspace,
            KeymapVimMode::Visual,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;
    runtime
        .register_key_binding("u", "vim.undo", KeymapScope::Workspace, CommandSource::Core)
        .map_err(|error| error.to_string())?;

    runtime
        .execute_key_binding_for_mode(&KeymapScope::Workspace, KeymapVimMode::Normal, "x")
        .map_err(|error| error.to_string())?;
    runtime
        .execute_key_binding_for_mode(&KeymapScope::Workspace, KeymapVimMode::Visual, "x")
        .map_err(|error| error.to_string())?;
    runtime
        .execute_key_binding_for_mode(&KeymapScope::Workspace, KeymapVimMode::Visual, "u")
        .map_err(|error| error.to_string())?;

    let log = runtime
        .services()
        .get::<EventLog>()
        .ok_or_else(|| "event log service missing".to_owned())?;
    assert_eq!(
        log.0,
        vec![
            "normal-x".to_owned(),
            "visual-x".to_owned(),
            "undo".to_owned(),
        ]
    );

    Ok(())
}

#[test]
fn runtime_executes_stacked_keybinding_commands_in_order() -> Result<(), String> {
    let mut runtime = EditorRuntime::new();
    runtime.services_mut().insert(EventLog::default());

    for (command_name, entry) in [
        ("vim.scroll-half-page-down", "half-page-down"),
        ("vim.center-current-line", "center-current-line"),
    ] {
        runtime
            .register_command(
                command_name,
                command_name,
                CommandSource::Core,
                move |runtime| {
                    let log = runtime
                        .services_mut()
                        .get_mut::<EventLog>()
                        .ok_or_else(|| "event log service missing".to_owned())?;
                    log.0.push(entry.to_owned());
                    Ok(())
                },
            )
            .map_err(|error| error.to_string())?;
    }

    runtime
        .register_key_binding_for_mode_many(
            "Ctrl+d",
            vec![
                "vim.scroll-half-page-down".to_owned(),
                "vim.center-current-line".to_owned(),
            ],
            KeymapScope::Workspace,
            KeymapVimMode::Normal,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    runtime
        .execute_key_binding_for_mode(&KeymapScope::Workspace, KeymapVimMode::Normal, "Ctrl+d")
        .map_err(|error| error.to_string())?;

    let log = runtime
        .services()
        .get::<EventLog>()
        .ok_or_else(|| "event log service missing".to_owned())?;
    assert_eq!(
        log.0,
        vec![
            "half-page-down".to_owned(),
            "center-current-line".to_owned(),
        ]
    );

    Ok(())
}

#[test]
fn model_switches_and_closes_workspaces() -> Result<(), ModelError> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let default_workspace = runtime
        .model_mut()
        .open_workspace(window_id, "default", None)?;
    let project_root = PathBuf::from("C:\\projects\\demo");
    let project_workspace =
        runtime
            .model_mut()
            .open_workspace(window_id, "project", Some(project_root.clone()))?;

    let workspace_names = runtime
        .model()
        .active_window()?
        .workspaces()
        .map(|workspace| workspace.name().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        workspace_names,
        vec!["default".to_owned(), "project".to_owned()]
    );
    assert_eq!(runtime.model().active_workspace_id()?, project_workspace);
    assert_eq!(runtime.model().workspace(default_workspace)?.root(), None);
    assert_eq!(
        runtime.model().workspace(project_workspace)?.root(),
        Some(project_root.as_path())
    );

    runtime.model_mut().switch_workspace(default_workspace)?;
    assert_eq!(runtime.model().active_workspace_id()?, default_workspace);

    let removed = runtime.model_mut().close_workspace(project_workspace)?;
    assert_eq!(removed.name(), "project");
    assert_eq!(runtime.model().active_window()?.workspace_count(), 1);
    assert_eq!(runtime.model().active_workspace_id()?, default_workspace);

    Ok(())
}

#[test]
fn model_focuses_existing_buffer_in_active_pane() -> Result<(), ModelError> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "default", None)?;
    let scratch_id =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)?;
    let notes_id =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*notes*", BufferKind::Scratch, None)?;

    runtime.model_mut().focus_buffer(workspace_id, scratch_id)?;

    let active_buffer = runtime
        .model()
        .workspace(workspace_id)?
        .active_pane()
        .and_then(|pane| pane.active_buffer());
    assert_eq!(active_buffer, Some(scratch_id));
    assert!(
        runtime
            .model()
            .workspace(workspace_id)?
            .active_pane()
            .map(|pane| pane.buffer_ids().contains(&notes_id))
            .unwrap_or(false)
    );

    Ok(())
}

#[test]
fn model_splits_pane_and_focuses() -> Result<(), ModelError> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "default", None)?;
    let scratch_id =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)?;
    let notes_id =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*notes*", BufferKind::Scratch, None)?;

    let initial_pane = runtime
        .model()
        .workspace(workspace_id)?
        .active_pane_id()
        .ok_or(ModelError::NoActivePane(workspace_id))?;
    runtime.model_mut().focus_buffer(workspace_id, scratch_id)?;

    let new_pane_id = runtime.model_mut().split_pane(workspace_id, notes_id)?;
    let workspace = runtime.model().workspace(workspace_id)?;
    assert_eq!(workspace.pane_count(), 2);
    assert_eq!(workspace.active_pane_id(), Some(new_pane_id));
    assert_eq!(
        workspace
            .pane(new_pane_id)
            .and_then(|pane| pane.active_buffer()),
        Some(notes_id)
    );

    runtime.model_mut().focus_pane(workspace_id, initial_pane)?;
    assert_eq!(
        runtime.model().workspace(workspace_id)?.active_pane_id(),
        Some(initial_pane)
    );

    Ok(())
}

#[test]
fn model_closes_active_pane_without_closing_buffers() -> Result<(), ModelError> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "default", None)?;
    let scratch_id =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)?;
    let notes_id =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*notes*", BufferKind::Scratch, None)?;

    let initial_pane = runtime
        .model()
        .workspace(workspace_id)?
        .active_pane_id()
        .ok_or(ModelError::NoActivePane(workspace_id))?;
    runtime.model_mut().focus_buffer(workspace_id, scratch_id)?;
    let split_pane_id = runtime.model_mut().split_pane(workspace_id, notes_id)?;
    runtime
        .model_mut()
        .focus_pane(workspace_id, split_pane_id)?;
    runtime
        .model_mut()
        .close_pane(workspace_id, split_pane_id)?;

    let workspace = runtime.model().workspace(workspace_id)?;
    assert_eq!(workspace.pane_count(), 1);
    assert_eq!(workspace.active_pane_id(), Some(initial_pane));
    assert!(workspace.buffer(notes_id).is_some());
    assert_eq!(
        runtime.model_mut().close_pane(workspace_id, initial_pane),
        Err(ModelError::CannotCloseLastPane(workspace_id))
    );

    Ok(())
}
