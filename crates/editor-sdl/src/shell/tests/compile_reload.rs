use super::*;

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
fn workspace_compile_prefills_release_rebuild_for_volt_user_tree() -> Result<(), String> {
    assert_eq!(
        prompt_prefill_for_marker(
            "compile-prompt-volt-user",
            "Cargo.toml",
            "[package]\nname = \"volt-user\"\n",
        )?,
        "cargo build --release -p volt-user"
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
fn dap_heuristic_compile_opens_confirm_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-compile-confirm");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .map_err(|e| e.to_string())?;
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|e| e.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;
    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;

    let configuration = DebugConfiguration::new("Debug", DebugRequestKind::Launch)
        .with_target_program(program)
        .with_cwd(root.clone());
    continue_dap_start(&mut state.runtime, "fake-dap", configuration, true)?;
    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "expected compile confirm picker".to_owned())?;
    assert_eq!(picker.session().title(), "Compile before debug?");
    let _ = fake.join();
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

fn write_dotnet_project(
    dir: &std::path::Path,
    file_name: &str,
    output_type: &str,
) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    fs::write(
        dir.join(file_name),
        format!(
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>{output_type}</OutputType><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>"#
        ),
    )
    .map_err(|error| error.to_string())
}

fn write_entry_dll(project_dir: &std::path::Path, name: &str) -> Result<(), String> {
    let dir = project_dir.join("bin").join("Debug").join("net8.0");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    fs::write(dir.join(name), "").map_err(|error| error.to_string())
}

#[test]
fn dap_dotnet_multi_project_opens_build_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-dotnet-build");
    let solution = root.join("src").join("dir1");
    let api = solution.join("dotnet_api");
    let tests = solution.join("dotnet_test_library");
    let shared = solution.join("dotnet_shared_library");
    write_dotnet_project(&api, "api.csproj", "Exe")?;
    write_dotnet_project(&tests, "test.csproj", "Library")?;
    write_dotnet_project(&shared, "shared.csproj", "Library")?;
    fs::write(
        solution.join("a.sln"),
        "Microsoft Visual Studio Solution File, Format Version 12.00\n",
    )
    .map_err(|error| error.to_string())?;
    let program = api.join("Program.cs");
    fs::write(&program, "class Program {}\n").map_err(|error| error.to_string())?;
    let _workspace = open_workspace_from_project(&mut state.runtime, "dotnet", &root)?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;

    let configuration = DebugConfiguration::new("Debug (dotnet)", DebugRequestKind::Launch)
        .with_target_program(program)
        .with_compile_command("dotnet build")
        .with_cwd(root.clone());
    continue_dap_start(&mut state.runtime, "sharpdbg", configuration, true)?;
    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "expected dotnet build picker".to_owned())?;
    assert_eq!(picker.session().title(), "Build with dotnet");
    let labels = picker
        .session()
        .matches()
        .iter()
        .map(|row| row.item().label().to_owned())
        .collect::<Vec<_>>();
    assert!(
        labels[0].contains("api.csproj"),
        "nearest project should be first: {labels:?}"
    );
    assert!(
        labels.iter().any(|label| label.contains("a.sln")),
        "{labels:?}"
    );
    assert!(
        labels.iter().any(|label| label.contains("test.csproj")),
        "{labels:?}"
    );
    assert!(
        labels.iter().any(|label| label.contains("shared.csproj")),
        "{labels:?}"
    );
    assert_eq!(
        labels.last().map(String::as_str),
        Some("Start without compiling")
    );
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_compile_shows_working_notice_before_session_starts() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-compile-progress");
    let _workspace = open_workspace_from_project(&mut state.runtime, "dbg", &root)?;
    let program = root.join("main.rs");
    fs::write(&program, "fn main() {}\n").map_err(|error| error.to_string())?;
    let _buffer = open_workspace_file(&mut state.runtime, &program)?;
    let (_port, fake) = install_fake_tcp_dap_manager(&mut state.runtime)?;

    let configuration = DebugConfiguration::new("Debug", DebugRequestKind::Launch)
        .with_target_program(program)
        .with_cwd(root.clone());
    run_dap_prelaunch_then_start(
        &mut state.runtime,
        "fake-dap",
        configuration,
        Some("echo dap-compile-progress".to_owned()),
    )?;

    assert!(
        !shell_ui(&state.runtime)?.is_debug_layout_active(),
        "session must wait until compile finishes"
    );
    let notices = shell_ui(&state.runtime)?.visible_notifications(std::time::Instant::now());
    assert!(
        notices
            .iter()
            .any(|notice| notice.active && notice.title == "Compiling"),
        "compiling notice missing"
    );
    assert!(
        active_runtime_popup(&state.runtime)?.is_some(),
        "compile output popup missing"
    );

    let started = std::time::Instant::now();
    loop {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        if shell_ui(&state.runtime)?.is_debug_layout_active() {
            break;
        }
        if started.elapsed() > std::time::Duration::from_secs(8) {
            return Err("debug session did not start after compile".to_owned());
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    stop_dap_for_active_workspace(&mut state.runtime)?;
    let _ = fake.join();
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn dap_dotnet_assembly_picker_lists_entry_dlls_only() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("dap-dotnet-dll");
    let solution = root.join("src").join("dir1");
    let api = solution.join("dotnet_api");
    let tests = solution.join("dotnet_test_library");
    let shared = solution.join("dotnet_shared_library");
    write_dotnet_project(&api, "api.csproj", "Exe")?;
    write_dotnet_project(&tests, "test.csproj", "Library")?;
    write_dotnet_project(&shared, "shared.csproj", "Library")?;
    fs::write(
        solution.join("a.sln"),
        r#"Microsoft Visual Studio Solution File, Format Version 12.00
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "dotnet_api", "dotnet_api\api.csproj", "{11111111-1111-1111-1111-111111111111}"
EndProject
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "dotnet_test_library", "dotnet_test_library\test.csproj", "{22222222-2222-2222-2222-222222222222}"
EndProject
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "dotnet_shared_library", "dotnet_shared_library\shared.csproj", "{33333333-3333-3333-3333-333333333333}"
EndProject
"#,
    )
    .map_err(|error| error.to_string())?;
    write_entry_dll(&api, "dotnet_api.dll")?;
    write_entry_dll(&api, "Newtonsoft.Json.dll")?;
    write_entry_dll(&tests, "dotnet_test_library.dll")?;
    write_entry_dll(&shared, "dotnet_shared_library.dll")?;
    let _workspace = open_workspace_from_project(&mut state.runtime, "dotnet-dll", &root)?;
    let command = dotnet_compile_command(&root, &solution.join("a.sln"));
    let candidates = dotnet_program_candidates_for_build_command(&root, &command, None)
        .ok_or_else(|| "expected dotnet candidates".to_owned())?;
    let configuration = DebugConfiguration::new("Debug (dotnet)", DebugRequestKind::Launch)
        .with_compile_command(command)
        .with_cwd(root.clone());
    present_dotnet_program_choice(
        &mut state.runtime,
        "sharpdbg",
        configuration,
        &root,
        candidates,
        false,
    )?;
    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "expected assembly picker".to_owned())?;
    assert_eq!(picker.session().title(), "Debug assembly");
    let labels = picker
        .session()
        .matches()
        .iter()
        .map(|row| row.item().label().to_owned())
        .collect::<Vec<_>>();
    assert!(
        labels[0].contains("dotnet_api.dll"),
        "executable assembly should be first: {labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label.contains("dotnet_test_library.dll")),
        "{labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label.contains("dotnet_shared_library.dll")),
        "{labels:?}"
    );
    assert!(
        labels
            .iter()
            .all(|label| !label.contains("Newtonsoft.Json.dll")),
        "{labels:?}"
    );
    let _ = fs::remove_dir_all(&root);
    Ok(())
}
