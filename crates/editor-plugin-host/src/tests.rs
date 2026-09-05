
use editor_core::{BufferKind, EditorRuntime, HookEvent, KeymapScope, builtins};
use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginKeyBinding,
    PluginKeymapScope, PluginPackage,
};

use super::{
    auto_loaded_packages, bootstrap, clear_package_registrations, load_auto_loaded_packages,
    reload_user_packages,
};

type HookContext = (
    Option<editor_core::WindowId>,
    Option<editor_core::WorkspaceId>,
    Option<editor_core::PaneId>,
    Option<editor_core::BufferId>,
);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct HookContextLog(Option<HookContext>);

fn file_open_package(filter: &str, buffer_name: &str) -> PluginPackage {
    PluginPackage::new("tests", true, "File-open hook test package.")
        .with_commands(vec![PluginCommand::new(
            "tests.attach",
            "Attaches test behavior.",
            vec![PluginAction::open_buffer(
                buffer_name,
                "scratch",
                None::<&str>,
            )],
        )])
        .with_hook_bindings(vec![PluginHookBinding::new(
            builtins::FILE_OPEN,
            format!("tests.auto-attach-{}", filter.replace('*', "star")),
            "tests.attach",
            Some(filter),
        )])
}

#[test]
fn bootstrap_uses_the_selected_abi_strategy() {
    assert_eq!(bootstrap().plugin_abi, "abi_stable");
}

#[test]
fn auto_loaded_packages_filters_manual_packages_out() {
    let packages = vec![
        PluginPackage::new("lsp", true, "Language server integration."),
        PluginPackage::new("git", false, "Git workflows."),
    ];

    let auto_loaded = auto_loaded_packages(&packages);
    assert_eq!(auto_loaded, vec![packages[0].clone()]);
}

#[test]
fn host_loads_auto_packages_into_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)?;

    let packages = vec![
        PluginPackage::new("terminal", true, "Builtin terminal package.")
            .with_commands(vec![PluginCommand::new(
                "terminal.open",
                "Opens the builtin terminal buffer.",
                vec![PluginAction::open_buffer(
                    "*terminal*",
                    "terminal",
                    None::<&str>,
                )],
            )])
            .with_key_bindings(vec![PluginKeyBinding::new(
                "Ctrl+`",
                "terminal.open",
                PluginKeymapScope::Global,
            )]),
        PluginPackage::new("lang-rust", true, "Rust language defaults.")
            .with_hook_declarations(vec![PluginHookDeclaration::new(
                "lang.rust.attached",
                "Runs after Rust language support attaches.",
            )])
            .with_commands(vec![PluginCommand::new(
                "lang-rust.attach",
                "Attaches Rust language services.",
                vec![
                    PluginAction::open_buffer("*rust-attachments*", "diagnostics", None::<&str>),
                    PluginAction::emit_hook("lang.rust.attached", Some("rust")),
                ],
            )])
            .with_hook_bindings(vec![PluginHookBinding::new(
                builtins::FILE_OPEN,
                "lang-rust.auto-attach",
                "lang-rust.attach",
                Some(".rs"),
            )]),
        PluginPackage::new("git", false, "Git workflows."),
    ];

    let loaded = load_auto_loaded_packages(&mut runtime, &packages)?;
    assert_eq!(loaded, 2);
    assert!(runtime.commands().contains("terminal.open"));
    assert!(runtime.keymaps().contains(&KeymapScope::Global, "Ctrl+`"));
    assert!(runtime.hooks().contains("lang.rust.attached"));

    runtime.execute_key_binding(&KeymapScope::Global, "Ctrl+`")?;
    runtime.emit_hook(
        builtins::FILE_OPEN,
        HookEvent::new()
            .with_workspace(workspace_id)
            .with_detail("main.rs"),
    )?;

    let workspace = runtime.model().workspace(workspace_id)?;
    assert_eq!(workspace.buffer_count(), 2);

    Ok(())
}

#[test]
fn file_open_hook_filters_still_match_legacy_extension_details()
-> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)?;
    let baseline_buffers = runtime.model().workspace(workspace_id)?.buffer_count();

    let packages = vec![file_open_package(".rs", "*legacy-extension*")];
    load_auto_loaded_packages(&mut runtime, &packages)?;

    runtime.emit_hook(
        builtins::FILE_OPEN,
        HookEvent::new()
            .with_workspace(workspace_id)
            .with_detail(".rs"),
    )?;

    let workspace = runtime.model().workspace(workspace_id)?;
    assert_eq!(workspace.buffer_count(), baseline_buffers + 1);
    Ok(())
}

#[test]
fn file_open_hook_filters_match_exact_basenames() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)?;
    let baseline_buffers = runtime.model().workspace(workspace_id)?.buffer_count();

    let packages = vec![file_open_package("Makefile", "*makefile*")];
    load_auto_loaded_packages(&mut runtime, &packages)?;

    runtime.emit_hook(
        builtins::FILE_OPEN,
        HookEvent::new()
            .with_workspace(workspace_id)
            .with_detail("Makefile"),
    )?;

    let workspace = runtime.model().workspace(workspace_id)?;
    assert_eq!(workspace.buffer_count(), baseline_buffers + 1);
    Ok(())
}

#[test]
fn file_open_hook_filters_match_globs() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)?;
    let baseline_buffers = runtime.model().workspace(workspace_id)?.buffer_count();

    let packages = vec![file_open_package("Dockerfile.*", "*dockerfile*")];
    load_auto_loaded_packages(&mut runtime, &packages)?;

    runtime.emit_hook(
        builtins::FILE_OPEN,
        HookEvent::new()
            .with_workspace(workspace_id)
            .with_detail("Dockerfile.dev"),
    )?;

    let workspace = runtime.model().workspace(workspace_id)?;
    assert_eq!(workspace.buffer_count(), baseline_buffers + 1);
    Ok(())
}

#[test]
fn host_registers_self_contained_manual_package_commands() -> Result<(), Box<dyn std::error::Error>>
{
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)?;
    let baseline_buffers = runtime.model().workspace(workspace_id)?.buffer_count();

    let packages = vec![
        PluginPackage::new("calculator", false, "Self-contained calculator commands.")
            .with_commands(vec![PluginCommand::new(
                "calculator.open",
                "Opens the calculator buffer.",
                vec![PluginAction::open_buffer(
                    "*calculator*",
                    "scratch",
                    None::<&str>,
                )],
            )])
            .with_key_bindings(vec![PluginKeyBinding::new(
                "Ctrl+=",
                "calculator.open",
                PluginKeymapScope::Global,
            )]),
        PluginPackage::new("lang-rust", false, "Rust language defaults.")
            .with_hook_declarations(vec![PluginHookDeclaration::new(
                "lang.rust.attached",
                "Runs after Rust language support attaches.",
            )])
            .with_commands(vec![PluginCommand::new(
                "lang-rust.attach",
                "Attaches Rust language services.",
                vec![PluginAction::emit_hook("lang.rust.attached", Some("rust"))],
            )]),
    ];

    let loaded = load_auto_loaded_packages(&mut runtime, &packages)?;
    assert_eq!(loaded, 0);
    assert!(runtime.commands().contains("calculator.open"));
    assert!(!runtime.commands().contains("lang-rust.attach"));
    assert!(!runtime.keymaps().contains(&KeymapScope::Global, "Ctrl+="));

    runtime.execute_command("calculator.open")?;
    assert_eq!(
        runtime.model().workspace(workspace_id)?.buffer_count(),
        baseline_buffers + 1
    );

    Ok(())
}

#[test]
fn reload_user_packages_replaces_commands_without_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = EditorRuntime::new();
    let initial = vec![
        PluginPackage::new("terminal", true, "Builtin terminal package.").with_commands(vec![
            PluginCommand::new(
                "terminal.open",
                "Opens the builtin terminal buffer.",
                vec![PluginAction::open_buffer(
                    "*terminal*",
                    "terminal",
                    None::<&str>,
                )],
            ),
        ]),
    ];
    load_auto_loaded_packages(&mut runtime, &initial)?;

    let reloaded = vec![
        PluginPackage::new("terminal", true, "Builtin terminal package.").with_commands(vec![
            PluginCommand::new(
                "terminal.open",
                "Opens the builtin terminal buffer (reloaded).",
                vec![PluginAction::log_message("reloaded terminal command")],
            ),
        ]),
    ];
    let loaded = reload_user_packages(&mut runtime, &initial, &reloaded)?;
    assert_eq!(loaded, 1);
    assert_eq!(
        runtime
            .commands()
            .get("terminal.open")
            .map(|command| command.description()),
        Some("Opens the builtin terminal buffer (reloaded).")
    );

    Ok(())
}

#[test]
fn clear_package_registrations_removes_hook_bindings_and_declarations()
-> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = EditorRuntime::new();
    let packages = vec![
        PluginPackage::new("lang-rust", true, "Rust language defaults.")
            .with_hook_declarations(vec![PluginHookDeclaration::new(
                "lang.rust.attached",
                "Runs after Rust language support attaches.",
            )])
            .with_hook_bindings(vec![PluginHookBinding::new(
                builtins::FILE_OPEN,
                "lang-rust.auto-attach",
                "lang-rust.attach",
                Some(".rs"),
            )])
            .with_commands(vec![PluginCommand::new(
                "lang-rust.attach",
                "Attaches Rust language services.",
                vec![PluginAction::emit_hook("lang.rust.attached", Some("rust"))],
            )]),
    ];
    load_auto_loaded_packages(&mut runtime, &packages)?;
    assert!(runtime.hooks().contains("lang.rust.attached"));

    clear_package_registrations(&mut runtime, &packages)?;
    assert!(!runtime.commands().contains("lang-rust.attach"));
    assert!(!runtime.hooks().contains("lang.rust.attached"));

    Ok(())
}

#[test]
fn emitted_hook_actions_include_active_window_pane_and_buffer()
-> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = EditorRuntime::new();
    let window_id = runtime.model_mut().create_window("main");
    let workspace_id = runtime
        .model_mut()
        .open_workspace(window_id, "scratch", None)?;
    let buffer_id =
        runtime
            .model_mut()
            .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)?;
    runtime.model_mut().focus_buffer(workspace_id, buffer_id)?;
    let pane_id = runtime
        .model()
        .workspace(workspace_id)?
        .active_pane_id()
        .ok_or("active pane missing")?;

    runtime.register_hook(
        "tests.buffer-scoped",
        "Captures the active command context for emitted hooks.",
    )?;
    runtime.services_mut().insert(HookContextLog::default());
    runtime.subscribe_hook(
        "tests.buffer-scoped",
        "tests.capture-context",
        |event, runtime| {
            let log = runtime
                .services_mut()
                .get_mut::<HookContextLog>()
                .ok_or_else(|| "hook context log missing".to_owned())?;
            log.0 = Some((
                event.window_id,
                event.workspace_id,
                event.pane_id,
                event.buffer_id,
            ));
            Ok(())
        },
    )?;

    let packages = vec![
        PluginPackage::new("tests", true, "Hook context test package.").with_commands(vec![
            PluginCommand::new(
                "tests.emit-context",
                "Emits a hook with the active runtime context.",
                vec![PluginAction::emit_hook("tests.buffer-scoped", None::<&str>)],
            ),
        ]),
    ];

    let loaded = load_auto_loaded_packages(&mut runtime, &packages)?;
    assert_eq!(loaded, 1);

    runtime.execute_command("tests.emit-context")?;

    assert_eq!(
        runtime.services().get::<HookContextLog>(),
        Some(&HookContextLog(Some((
            Some(window_id),
            Some(workspace_id),
            Some(pane_id),
            Some(buffer_id),
        ))))
    );

    Ok(())
}
