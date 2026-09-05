#[test]
fn input_prompt_overlay_confirm_delivers_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-confirm");
    open_workspace_from_project(&mut state.runtime, "input-prompt-confirm", &root)?;
    let marker = "volt-input-prompt-confirm";
    let popup_buffer = start_workspace_compile(&mut state, &shell_echo_command(marker))?;
    wait_for_streamed_command_output_line(&mut state, popup_buffer, marker)?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt should close after Enter with text"
    );
    assert!(
        shell_ui(&state.runtime)?
            .buffer(popup_buffer)
            .is_some_and(|buffer| buffer.text.text().contains(marker)),
        "confirmed prompt text should reach streamed compile command"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_escape_cancels() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-escape");
    open_workspace_from_project(&mut state.runtime, "input-prompt-escape", &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;

    state
        .try_runtime_keybinding(Keycode::Escape, Mod::empty())
        .map_err(|e| e.to_string())?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt should close on Escape"
    );
    assert!(
        active_runtime_popup(&state.runtime)?.is_none(),
        "Escape should discard the compile prompt without opening a popup"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_enter_with_empty_text_is_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-empty");
    open_workspace_from_project(&mut state.runtime, "input-prompt-empty", &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;

    state
        .try_runtime_keybinding(Keycode::Return, Mod::empty())
        .map_err(|e| e.to_string())?;

    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt must stay open when Enter pressed with empty text"
    );
    assert!(
        active_runtime_popup(&state.runtime)?.is_none(),
        "empty Enter should not open the compile popup"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_prefill_appears_in_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let overlay = InputPromptOverlay::new("test.prompt", "Build: ", "cargo build");
    shell_ui_mut(&mut state.runtime)?.open_input_prompt(overlay);
    assert_eq!(
        shell_ui(&state.runtime)?.input_prompt().map(|p| p.text()),
        Some("cargo build")
    );
    Ok(())
}

#[test]
fn render_shell_state_draws_input_prompt_overlay_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let overlay = InputPromptOverlay::new(COMPILE_PROMPT_ID, "Build command: ", "cargo build");
    shell_ui_mut(&mut state.runtime)?.open_input_prompt(overlay);

    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        ShellDockEntries {
            workspace: &[],
            acp: &[],
        },
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 640,
                height: 360,
            },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { text, .. } if text.contains("Build command: cargo build")
        )),
        "InputPromptOverlay must draw into the command-line footer row"
    );
    Ok(())
}

// ─── workspace.compile prompt tests ──────────────────────────────────────────

#[test]
fn workspace_compile_opens_input_prompt_overlay() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-prompt-opens");
    open_workspace_from_project(&mut state.runtime, "compile-prompt-opens", &root)?;

    execute_shell_command(&mut state, "workspace.compile")?;

    let prompt = shell_ui(&state.runtime)?.input_prompt();
    assert!(prompt.is_some(), "InputPromptOverlay should be open");
    assert_eq!(
        prompt.map(|p| p.id.as_str()),
        Some(COMPILE_PROMPT_ID),
        "overlay id must be COMPILE_PROMPT_ID"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_cargo_toml() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker(
            "compile-prompt-cargo",
            "Cargo.toml",
            "[package]\nname = \"test\"\n",
        )?,
        "cargo build"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_sln() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-sln", "App.sln", "")?,
        "dotnet build"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_csproj() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-csproj", "App.csproj", "")?,
        "dotnet build"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_package_json() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-package-json", "package.json", "{}")?,
        "npm run build"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_detected_command_for_makefile() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-makefile", "Makefile", "")?,
        "make"
    );
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_empty_command_for_empty_directory() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker("compile-prompt-empty", "", "")?,
        ""
    );
    Ok(())
}

#[test]
fn workspace_compile_escape_does_not_store_command() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-escape");
    open_workspace_from_project(&mut state.runtime, "compile-escape", &root)?;

    execute_shell_command(&mut state, "workspace.compile")?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::empty())
        .map_err(|e| e.to_string())?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt should close on Escape"
    );
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    let stored = shell_ui(&state.runtime)?
        .compile_commands
        .get(&workspace_id)
        .cloned();
    assert!(stored.is_none(), "Escape must not store a command");
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_compile_prefills_with_stored_command_over_detected() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-stored");
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n")
        .map_err(|e| e.to_string())?;
    open_workspace_from_project(&mut state.runtime, "compile-stored", &root)?;

    // Pre-store a custom command.
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    shell_ui_mut(&mut state.runtime)?
        .compile_commands
        .insert(workspace_id, "cargo build --release".to_owned());

    execute_shell_command(&mut state, "workspace.compile")?;

    let text = shell_ui(&state.runtime)?
        .input_prompt()
        .map(|p| p.text().to_owned())
        .unwrap_or_default();
    assert_eq!(
        text, "cargo build --release",
        "stored command should take priority over auto-detection"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

// ─── workspace.recompile tests ────────────────────────────────────────────────

#[test]
fn workspace_recompile_with_stored_command_does_not_open_input_prompt() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("recompile-stored");
    open_workspace_from_project(&mut state.runtime, "recompile-stored", &root)?;

    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    shell_ui_mut(&mut state.runtime)?
        .compile_commands
        .insert(workspace_id, shell_echo_command("recompile-ok"));

    execute_shell_command(&mut state, "workspace.recompile")?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "recompile with stored command must not open InputPromptOverlay"
    );
    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "recompile with stored command should open streamed popup".to_owned())?;
    wait_for_streamed_command_output_line(&mut state, popup.active_buffer, "recompile-ok")?;
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_recompile_without_stored_command_falls_back_to_compile_prompt() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("recompile-fallback");
    open_workspace_from_project(&mut state.runtime, "recompile-fallback", &root)?;

    execute_shell_command(&mut state, "workspace.recompile")?;

    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "recompile without stored command must open InputPromptOverlay"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_recompile_uses_workspace_scoped_stored_command() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root_a = unique_temp_dir("recompile-scope-a");
    let root_b = unique_temp_dir("recompile-scope-b");

    open_workspace_from_project(&mut state.runtime, "recompile-scope-a", &root_a)?;
    let workspace_a = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    shell_ui_mut(&mut state.runtime)?
        .compile_commands
        .insert(workspace_a, shell_echo_command("workspace-a"));

    open_workspace_from_project(&mut state.runtime, "recompile-scope-b", &root_b)?;
    let workspace_b = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    assert_ne!(workspace_a, workspace_b);

    execute_shell_command(&mut state, "workspace.recompile")?;

    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "recompile in workspace B must open prompt when only workspace A has a stored command"
    );
    std::fs::remove_dir_all(&root_a).ok();
    std::fs::remove_dir_all(&root_b).ok();
    Ok(())
}

#[test]
fn workspace_compile_confirm_reuses_existing_streamed_popup() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-reuse-popup");
    open_workspace_from_project(&mut state.runtime, "compile-reuse-popup", &root)?;

    let first_buffer = start_workspace_compile(&mut state, &shell_echo_command("compile-one"))?;
    wait_for_streamed_command_output_line(&mut state, first_buffer, "compile-one")?;
    wait_for_streamed_command_worker_done(&mut state, first_buffer)?;

    let second_buffer = start_workspace_compile(&mut state, &shell_echo_command("compile-two"))?;
    assert_eq!(
        first_buffer, second_buffer,
        "workspace.compile should reuse the existing streamed popup buffer"
    );
    wait_for_streamed_command_output_line(&mut state, second_buffer, "compile-two")?;

    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_compile_closing_popup_mid_build_stops_tracking_worker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-close-popup");
    open_workspace_from_project(&mut state.runtime, "compile-close-popup", &root)?;

    let buffer_id = start_workspace_compile(
        &mut state,
        &shell_sleep_then_echo_command(60, "compile-stop"),
    )?;
    assert!(
        shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "compile worker should be tracked before popup close"
    );

    close_buffer_immediate(&mut state.runtime, buffer_id).map_err(|error| error.to_string())?;
    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;
    assert!(
        !shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "closing compile popup should stop tracking the worker within the poll timeout"
    );

    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn lsp_stop_with_no_live_sessions_returns_error_without_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("lsp-stop-empty");
    open_workspace_from_project(&mut state.runtime, "lsp-stop-empty", &root)?;
    let path = root.join("main.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_file(&mut state.runtime, &path)?;

    let error = state
        .runtime
        .execute_command("lsp.stop")
        .expect_err("lsp.stop should fail when no Sessions are live");
    assert!(
        error
            .to_string()
            .contains("no running Language Server Sessions"),
        "unexpected error: {error}"
    );
    assert!(shell_ui(&state.runtime)?.picker().is_none());

    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn lsp_restart_with_no_live_sessions_returns_error_without_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let error = state
        .runtime
        .execute_command("lsp.restart")
        .expect_err("lsp.restart should fail when no Sessions are live");
    assert!(
        error
            .to_string()
            .contains("no running Language Server Sessions"),
        "unexpected error: {error}"
    );
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    Ok(())
}

#[test]
fn lsp_install_server_opens_recipe_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state
        .runtime
        .execute_command("lsp.install-server")
        .map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let picker = ui
        .picker()
        .ok_or_else(|| "install picker missing".to_owned())?;
    assert_eq!(picker.session().title(), "Install Language Server");
    assert!(picker.session().item_count() > 0);
    let selected = picker.session().selected().expect("one row");
    assert!(
        selected
            .item()
            .label()
            .contains("typescript-language-server")
            || selected.item().label().contains("rust-analyzer")
            || !selected.item().label().is_empty()
    );
    Ok(())
}

#[test]
fn dap_install_server_opens_recipe_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state
        .runtime
        .execute_command("dap.install-server")
        .map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let picker = ui
        .picker()
        .ok_or_else(|| "install picker missing".to_owned())?;
    assert_eq!(picker.session().title(), "Install Debug Adapter");
    assert!(picker.session().item_count() > 0);
    Ok(())
}

#[test]
fn install_picker_label_prefixes_status_icon() {
    let plus = tool_install::install_picker_label(false, "rust-analyzer");
    let check = tool_install::install_picker_label(true, "rust-analyzer");
    assert!(plus.ends_with(" rust-analyzer"));
    assert!(check.ends_with(" rust-analyzer"));
    assert_ne!(plus, check);
}

#[test]
fn lsp_install_unknown_id_returns_error() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let error = tool_install::install_language_server_by_id(&mut state.runtime, "not-a-server")
        .expect_err("unknown spec must fail");
    assert!(
        error.contains("not registered"),
        "unexpected error: {error}"
    );
    Ok(())
}

#[test]
fn lsp_session_lifecycle_picker_labels_sessions_and_wires_stop_action() {
    let root = {
        #[cfg(windows)]
        {
            PathBuf::from(r"p:\volt")
        }
        #[cfg(not(windows))]
        {
            PathBuf::from("/volt")
        }
    };
    let session = LspLiveSession::new("rust-analyzer", Some(root.clone()));
    let picker = lsp_session_lifecycle_picker_overlay(LspSessionPickerAction::Stop, &[session]);
    assert_eq!(picker.session().title(), "Stop Language Server Session");
    assert_eq!(picker.session().item_count(), 1);
    let selected = picker.session().selected().expect("one row");
    assert_eq!(
        selected.item().label(),
        format!("rust-analyzer — {}", root.display())
    );
    let action = picker
        .actions
        .get(selected.item().id())
        .expect("stop action");
    assert!(matches!(
        action,
        PickerAction::StopLspSession {
            server_id,
            root: action_root
        } if server_id == "rust-analyzer" && action_root.as_deref() == Some(root.as_path())
    ));
}

#[derive(Debug, Clone, Copy)]
struct WorkspaceDockTestUserLibrary {
    config: WorkspaceDockConfig,
}

impl UserLibrary for WorkspaceDockTestUserLibrary {
    fn workspace_dock_config(&self) -> WorkspaceDockConfig {
        self.config
    }
}

fn state_with_workspace_dock_config(config: WorkspaceDockConfig) -> Result<ShellState, String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(WorkspaceDockTestUserLibrary { config });
    ShellState::new_with_user_library(default_error_log_path(), false, user_library)
        .map_err(|error| error.to_string())
}

#[test]
fn workspace_dock_config_defaults_left_undocked() {
    let config = WorkspaceDockConfig::default();
    assert_eq!(config.side, WorkspaceDockSide::Left);
    assert!(!config.docked);
}

#[test]
fn workspace_dock_toggle_shows_and_hides_when_not_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: false,
    })?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_open());
    assert!(!workspace_dock_visible(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?
    ));

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(shell_ui(&state.runtime)?.workspace_dock_open());
    assert!(workspace_dock_visible(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?
    ));

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_open());
    Ok(())
}

#[test]
fn workspace_dock_docked_stays_visible_across_toggle() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    assert!(workspace_dock_visible(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?
    ));
    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(workspace_dock_visible(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?
    ));
    assert!(shell_ui(&state.runtime)?.workspace_dock_open());
    Ok(())
}

#[test]
fn workspace_dock_entries_include_default_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-dock-default");
    let project = open_workspace_from_project(&mut state.runtime, "dock-project", &root)?;
    let default_workspace = shell_ui(&state.runtime)?.default_workspace();
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    assert!(
        entries
            .iter()
            .any(|entry| entry.workspace_id == default_workspace)
    );
    assert!(entries.iter().any(|entry| entry.workspace_id == project));
    assert_eq!(entries[0].workspace_id, default_workspace);
    Ok(())
}

#[test]
fn workspace_dock_unread_badge_tracks_other_workspace_notifications() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-unread-a");
    let second_root = unique_temp_dir("workspace-dock-unread-b");
    let first = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    let now = Instant::now();
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "other-ws".to_owned(),
            severity: NotificationSeverity::Info,
            title: "Agent finished".to_owned(),
            body_lines: vec!["done".to_owned()],
            progress: None,
            active: true,
            action: None,
            workspace_id: Some(second),
        },
        now,
    );
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let second_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == second)
        .ok_or_else(|| "second workspace missing from dock".to_owned())?;
    assert!(second_entry.unread >= 1);
    let first_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == first)
        .ok_or_else(|| "first workspace missing from dock".to_owned())?;
    assert_eq!(first_entry.unread, 0);
    switch_runtime_workspace(&mut state.runtime, second)?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let second_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == second)
        .ok_or_else(|| "second workspace missing after switch".to_owned())?;
    assert_eq!(second_entry.unread, 0);
    Ok(())
}

#[test]
fn workspace_dock_highlight_tracks_active_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-highlight-a");
    let second_root = unique_temp_dir("workspace-dock-highlight-b");
    let first = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == first)
            .is_some_and(|entry| entry.active)
    );
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == second)
            .is_some_and(|entry| !entry.active)
    );

    state
        .runtime
        .execute_command("workspace.next")
        .map_err(|error| error.to_string())?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == second)
            .is_some_and(|entry| entry.active)
    );
    Ok(())
}

#[test]
fn workspace_dock_click_switches_workspace() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    let first_root = unique_temp_dir("workspace-dock-click-a");
    let second_root = unique_temp_dir("workspace-dock-click-b");
    let first = open_workspace_from_project(&mut state.runtime, "click-a", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "click-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;

    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let second_index = entries
        .iter()
        .position(|entry| entry.workspace_id == second)
        .ok_or_else(|| "missing second workspace in dock".to_owned())?;
    let cell_width = 8;
    let line_height = 16;
    let render_width = 800;
    let render_height = 600;
    let layout = workspace_dock_layout(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?,
        render_width,
        render_height,
        cell_width,
    );
    assert!(layout.visible);
    let card_height = workspace_dock_card_height(line_height) as i32;
    let click_x = layout.dock_rect.x + 8;
    let click_y = layout.dock_rect.y + second_index as i32 * card_height + 4;

    state
        .handle_event(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: click_x as f32,
                y: click_y as f32,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    Ok(())
}

#[test]
fn workspace_dock_layout_shrinks_content_for_left_dock() -> Result<(), String> {
    let state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    let layout = workspace_dock_layout(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?,
        800,
        600,
        8,
    );
    assert!(layout.visible);
    assert!(layout.dock_width > 0);
    assert_eq!(layout.content_x, layout.dock_width as i32);
    assert_eq!(layout.content_width, 800 - layout.dock_width);
    assert_eq!(layout.dock_rect.x, 0);
    Ok(())
}

#[test]
fn workspace_dock_render_marks_active_row() -> Result<(), String> {
    let state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &*shell_user_library(&state.runtime),
    )
    .map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        ShellDockEntries {
            workspace: &entries,
            acp: &[],
        },
        ShellChrome {
            user_library: &*shell_user_library(&state.runtime),
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 640,
                height: 360,
            },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;
    let layout = workspace_dock_layout(&*shell_user_library(&state.runtime), ui, 640, 360, 8);
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == layout.dock_rect.x && rect.width == layout.dock_rect.width
    )));
    assert!(entries.iter().any(|entry| entry.active));
    Ok(())
}

#[test]
fn workspace_dock_ctrl_h_enters_focus_from_panes_when_left_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_LEFT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(
        shell_ui(&state.runtime)?.workspace_dock_focus_active(&*shell_user_library(&state.runtime))
    );
    Ok(())
}

#[test]
fn workspace_dock_ctrl_l_exits_focus_back_to_panes_when_left_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    shell_ui_mut(&mut state.runtime)?.set_workspace_dock_focus(true);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_RIGHT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());
    Ok(())
}

#[test]
fn workspace_dock_h_j_cycles_workspaces_when_focused() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Left,
        docked: true,
    })?;
    let first_root = unique_temp_dir("workspace-dock-keys-a");
    let second_root = unique_temp_dir("workspace-dock-keys-b");
    let first = open_workspace_from_project(&mut state.runtime, "keys-a", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "keys-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    shell_ui_mut(&mut state.runtime)?.set_workspace_dock_focus(true);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_NEXT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_PREVIOUS, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), first);
    Ok(())
}

#[test]
fn workspace_dock_focus_j_k_cycle_workspaces() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-jk-a");
    let second_root = unique_temp_dir("workspace-dock-jk-b");
    let first = open_workspace_from_project(&mut state.runtime, "dock-jk-a", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "dock-jk-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_DOCK_TOGGLE, HookEvent::new())
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_workspace_dock_focus(true);
        ui.enter_normal_mode();
    }

    let modes = state
        .overlay_minor_modes()
        .map_err(|error| error.to_string())?;
    assert!(
        modes.contains(&KeymapScope::WorkspaceDock),
        "dock focus must activate Workspace Dock Minor Mode: {modes:?}"
    );
    assert!(
        !modes.contains(&KeymapScope::Popup),
        "dock focus must not activate Popup Minor Mode: {modes:?}"
    );

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), first);
    Ok(())
}

#[test]
fn workspace_dock_ctrl_l_enters_focus_when_right_docked() -> Result<(), String> {
    let mut state = state_with_workspace_dock_config(WorkspaceDockConfig {
        side: WorkspaceDockSide::Right,
        docked: true,
    })?;
    assert!(!shell_ui(&state.runtime)?.workspace_dock_focus());

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_RIGHT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(
        shell_ui(&state.runtime)?.workspace_dock_focus_active(&*shell_user_library(&state.runtime))
    );
    Ok(())
}

#[test]
fn acp_dock_toggle_shows_and_hides() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_open());
    assert!(!acp_dock_visible(shell_ui(&state.runtime)?));

    toggle_acp_dock(&mut state.runtime)?;
    assert!(shell_ui(&state.runtime)?.acp_dock_open());
    assert!(acp_dock_visible(shell_ui(&state.runtime)?));

    toggle_acp_dock(&mut state.runtime)?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_open());
    assert!(!acp_dock_visible(shell_ui(&state.runtime)?));
    Ok(())
}

#[test]
fn acp_dock_ctrl_l_enters_focus_when_open() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    toggle_acp_dock(&mut state.runtime)?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_focus());

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_RIGHT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(shell_ui(&state.runtime)?.acp_dock_focus_active());
    Ok(())
}

#[test]
fn acp_dock_ctrl_h_exits_focus_back_to_panes() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    toggle_acp_dock(&mut state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.set_acp_dock_focus(true);

    state
        .runtime
        .emit_hook(HOOK_WORKSPACE_WINDOW_LEFT, HookEvent::new())
        .map_err(|error| error.to_string())?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_focus());
    Ok(())
}

#[test]
fn acp_dock_focus_j_k_cycle_buffers() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first = install_user_plugin_buffer(&mut state, "*acp Claude*", user::acp::ACP_BUFFER_KIND)?;
    let second = install_user_plugin_buffer(&mut state, "*acp Codex*", user::acp::ACP_BUFFER_KIND)?;
    shell_buffer_mut(&mut state.runtime, first)?.init_acp_view("Claude");
    shell_buffer_mut(&mut state.runtime, second)?.init_acp_view("Codex");
    acp::focus_acp_buffer(&mut state.runtime, first)?;
    toggle_acp_dock(&mut state.runtime)?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_acp_dock_focus(true);
        ui.enter_normal_mode();
    }

    let modes = state
        .overlay_minor_modes()
        .map_err(|error| error.to_string())?;
    assert!(
        modes.contains(&KeymapScope::AcpDock),
        "dock focus must activate ACP Dock Minor Mode: {modes:?}"
    );
    assert!(
        !modes.contains(&KeymapScope::WorkspaceDock),
        "ACP dock focus must not activate Workspace Dock Minor Mode: {modes:?}"
    );

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_buffer_id(), Some(second));
    assert!(shell_ui(&state.runtime)?.acp_dock_focus_active());
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.active_buffer_id(), Some(first));
    assert!(shell_ui(&state.runtime)?.acp_dock_focus_active());
    Ok(())
}

#[test]
fn acp_dock_entries_list_active_workspace_buffers() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first = install_user_plugin_buffer(&mut state, "*acp Claude*", user::acp::ACP_BUFFER_KIND)?;
    let second = install_user_plugin_buffer(&mut state, "*acp Codex*", user::acp::ACP_BUFFER_KIND)?;
    shell_buffer_mut(&mut state.runtime, first)?.init_acp_view("Claude");
    shell_buffer_mut(&mut state.runtime, second)?.init_acp_view("Codex");
    shell_buffer_mut(&mut state.runtime, first)?
        .acp_set_session_title(Some("Refactor dock".to_owned()));

    let entries = collect_acp_dock_entries(&state.runtime)?;
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].buffer_id, first);
    assert_eq!(entries[0].name, "Claude");
    assert_eq!(entries[0].session, "Refactor dock");
    assert_eq!(entries[1].buffer_id, second);
    assert_eq!(entries[1].name, "Codex");
    assert_eq!(entries[1].session, "New session");
    assert!(entries.iter().any(|entry| entry.active));
    Ok(())
}

#[test]
fn acp_dock_layout_shrinks_content_on_the_right() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    assert!(!shell_ui(&state.runtime)?.acp_dock_open());
    toggle_acp_dock(&mut state.runtime)?;
    let docks = shell_docks_layout(
        &*shell_user_library(&state.runtime),
        shell_ui(&state.runtime)?,
        640,
        360,
        8,
    );
    assert!(docks.acp.visible);
    assert_eq!(docks.acp.side, WorkspaceDockSide::Right);
    assert!(docks.acp.dock_width > 0);
    assert_eq!(docks.content_width + docks.acp.dock_width, 640);
    assert_eq!(docks.content_x, 0);
    Ok(())
}

#[test]
fn debug_fringe_is_one_cell_when_idle_and_two_when_live() {
    assert_eq!(debug_fringe_cell_count(false), 1);
    assert_eq!(debug_fringe_cell_count(true), 2);
    assert_eq!(editor_fringe_width_px(8, false), 8);
    assert_eq!(editor_fringe_width_px(8, true), 16);
}

#[test]
fn toggle_breakpoint_without_session_shows_idle_fringe_marker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-idle-fringe");
    let workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("Program.cs");
    fs::write(&program, "class Program { static void Main() {} }\n").map_err(|e| e.to_string())?;
    let buffer_id = open_workspace_file(&mut state.runtime, &program)?;

    toggle_dap_breakpoint_at_cursor(&mut state.runtime)?;
    sync_active_buffer(&mut state.runtime)?;

    let focused = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        focused, buffer_id,
        "toggling a Breakpoint must not switch away from the editor buffer"
    );
    let focused_name = shell_ui(&state.runtime)?
        .buffer(focused)
        .ok_or_else(|| "focused buffer missing".to_owned())?
        .display_name()
        .to_owned();
    assert_ne!(
        focused_name, DAP_BREAKPOINTS_BUFFER_NAME,
        "toggle must not open `{DAP_BREAKPOINTS_BUFFER_NAME}`"
    );

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "buffer missing".to_owned())?;
    assert!(
        !buffer.dap_fringe_live(),
        "idle Workspace must keep one-cell fringe (no live Session)"
    );
    assert_eq!(
        buffer.dap_fringe_marker(0),
        Some(BreakpointState::Pending),
        "Breakpoint must appear in Debug Fringe before a Session starts"
    );

    let workspace_id = workspace.get();
    let listed = state
        .runtime
        .services()
        .get::<Arc<DapClientManager>>()
        .ok_or_else(|| "dap manager missing".to_owned())?
        .list_breakpoints(workspace_id)
        .map_err(|e| e.to_string())?;
    assert_eq!(listed.len(), 1);
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn wrap_columns_shrink_when_debug_fringe_widens() {
    let idle = wrap_columns_for_width_with_fringe(320, 8, 1);
    let live = wrap_columns_for_width_with_fringe(320, 8, 2);
    assert!(live < idle);
}

fn install_fake_tcp_dap_manager(
    runtime: &mut EditorRuntime,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    use editor_dap::{DebugAdapterRegistry, DebugAdapterSpec, DebugAdapterTransport};
    use std::io::{BufRead, Read, Write};
    use std::net::TcpListener;

    fn write_raw(writer: &mut impl Write, body: &str) {
        write!(writer, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("write");
        writer.flush().expect("flush");
    }

    fn read_body(reader: &mut impl BufRead) -> Result<String, String> {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            let read = reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if read == 0 {
                return Err("adapter closed".to_owned());
            }
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                break;
            }
            let Some((key, value)) = trimmed.split_once(':') else {
                continue;
            };
            if key.eq_ignore_ascii_case("Content-Length") {
                content_length = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?);
            }
        }
        let len = content_length.ok_or_else(|| "missing Content-Length".to_owned())?;
        let mut buf = vec![0_u8; len];
        reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
        String::from_utf8(buf).map_err(|e| e.to_string())
    }

    fn extract_field(body: &str, key: &str) -> Option<String> {
        let needle = format!("\"{key}\"");
        let start = body.find(&needle)?;
        let after = &body[start + needle.len()..];
        let after = after.trim_start_matches([' ', ':', '\t']);
        if let Some(rest) = after.strip_prefix('"') {
            let end = rest.find('"')?;
            return Some(rest[..end].to_owned());
        }
        let end = after
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after.len());
        Some(after[..end].to_owned())
    }

    fn fake_adapter_loop(reader: impl Read, mut writer: impl Write) {
        let mut reader = std::io::BufReader::new(reader);
        let mut seq = 1_u64;
        let mut stopped_line = 1_u64;
        let mut program_path = "main.rs".to_owned();
        while let Ok(body) = read_body(&mut reader) {
            let command = extract_field(&body, "command").unwrap_or_default();
            let request_seq = extract_field(&body, "seq").unwrap_or_else(|| "0".to_owned());
            match command.as_str() {
                "initialize" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"initialize","body":{{"supportsConfigurationDoneRequest":true,"supportTerminateDebuggee":true,"supportsRestartRequest":true}}}}"#
                        ),
                    );
                    seq += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"initialized","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "configurationDone" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"configurationDone","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "launch" | "attach" => {
                    if let Some(program) = extract_field(&body, "program") {
                        program_path = program;
                    }
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"{command}","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line = 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"entry","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "setBreakpoints" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"setBreakpoints","body":{{"breakpoints":[]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "continue" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"continue"}}"#
                        ),
                    );
                    seq += 1;
                    if program_path.contains("exit-on-continue") {
                        write_raw(
                            &mut writer,
                            &format!(
                                r#"{{"seq":{seq},"type":"event","event":"exited","body":{{"exitCode":0}}}}"#
                            ),
                        );
                        seq += 1;
                        write_raw(
                            &mut writer,
                            &format!(
                                r#"{{"seq":{seq},"type":"event","event":"terminated","body":{{}}}}"#
                            ),
                        );
                        break;
                    }
                }
                "next" | "stepIn" | "stepOut" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"{command}","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"step","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "pause" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"pause","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"pause","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "restart" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"restart","body":{{}}}}"#
                        ),
                    );
                    seq += 1;
                    stopped_line = 1;
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"event","event":"stopped","body":{{"reason":"entry","threadId":1,"allThreadsStopped":true}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "stackTrace" => {
                    let path_json = program_path.replace('\\', "\\\\");
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"stackTrace","body":{{"stackFrames":[{{"id":1,"name":"main","source":{{"path":"{path_json}"}},"line":{stopped_line},"column":1}},{{"id":2,"name":"caller","source":{{"path":"{path_json}"}},"line":{},"column":1}}],"totalFrames":2}}}}"#,
                            stopped_line + 10
                        ),
                    );
                    seq += 1;
                }
                "threads" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"threads","body":{{"threads":[{{"id":1,"name":"main"}},{{"id":2,"name":"worker"}}]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "evaluate" => {
                    let expression = extract_field(&body, "expression").unwrap_or_default();
                    let frame_id =
                        extract_field(&body, "frameId").unwrap_or_else(|| "1".to_owned());
                    let eval_body = if expression == "person" {
                        r#"{"result":"Person { ... }","type":"Person","variablesReference":2}"#
                            .to_owned()
                    } else {
                        format!(
                            r#"{{"result":"{expression}@{frame_id}={stopped_line}","variablesReference":0}}"#
                        )
                    };
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"evaluate","body":{eval_body}}}"#
                        ),
                    );
                    seq += 1;
                }
                "scopes" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"scopes","body":{{"scopes":[{{"name":"Locals","variablesReference":1,"expensive":false}}]}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "variables" => {
                    let reference = extract_field(&body, "variablesReference")
                        .unwrap_or_else(|| "1".to_owned());
                    let variables = match reference.as_str() {
                        "2" => {
                            r#"[{"name":"Name","value":"\"Ada\"","type":"string","variablesReference":0},{"name":"Address","value":"Address { ... }","type":"Address","variablesReference":3}]"#
                        }
                        "3" => {
                            r#"[{"name":"City","value":"\"London\"","type":"string","variablesReference":0}]"#
                        }
                        _ => {
                            r#"[{"name":"x","value":"42","type":"i32","variablesReference":0},{"name":"person","value":"Person { ... }","type":"Person","variablesReference":2}]"#
                        }
                    };
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"variables","body":{{"variables":{variables}}}}}"#
                        ),
                    );
                    seq += 1;
                }
                "disconnect" => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":true,"command":"disconnect","body":{{}}}}"#
                        ),
                    );
                    break;
                }
                _ => {
                    write_raw(
                        &mut writer,
                        &format!(
                            r#"{{"seq":{seq},"type":"response","request_seq":{request_seq},"success":false,"command":"{command}","message":"unsupported"}}"#
                        ),
                    );
                    seq += 1;
                }
            }
        }
    }

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut sessions = 0_u8;
        while sessions < 4 && Instant::now() < deadline {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let reader = stream.try_clone().expect("clone");
                    fake_adapter_loop(reader, stream);
                    sessions += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(_) => break,
            }
        }
    });

    let mut registry = DebugAdapterRegistry::new();
    registry
        .register(
            DebugAdapterSpec::new("fake-dap", "rust", ["rs"], "", [] as [&str; 0])
                .with_transport(DebugAdapterTransport::Tcp {
                    host: "127.0.0.1".to_owned(),
                    port,
                })
                .with_preference(10),
        )
        .map_err(|e| e.to_string())?;
    runtime
        .services_mut()
        .insert(Arc::new(DapClientManager::new(registry)));
    Ok((port, handle))
}
