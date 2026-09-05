/// [`InputPromptOverlay`] id used for the `workspace.compile` prompt.
const COMPILE_PROMPT_ID: &str = "compile";

/// Buffer name pattern for the compilation popup.
fn compile_buffer_name(workspace_name: &str) -> String {
    format!("*compile {workspace_name}*")
}

fn command_output_buffer_name(workspace_name: &str) -> String {
    format!("*command {workspace_name}*")
}

fn buffer_is_command_output(kind: &BufferKind, name: &str) -> bool {
    matches!(kind, BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_INPUT_KIND)
        && name.starts_with("*command ")
}

/// Open an [`InputPromptOverlay`] pre-filled with the auto-detected (or last
/// stored) build command for the active workspace.
///
/// Called by the `plugin.run-command` hook subscriber.  On confirmation the
/// overlay dispatches to [`dispatch_input_prompt_confirm`] with id
/// `COMPILE_PROMPT_ID`.
fn open_compile_prompt(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;

    // Prefer the last stored command for this workspace; fall back to
    // auto-detection from the workspace root.
    let prefill = shell_ui(runtime)
        .ok()
        .and_then(|ui| ui.compile_commands.get(&workspace_id).cloned())
        .unwrap_or_else(|| {
            active_workspace_root(runtime)
                .ok()
                .flatten()
                .map(|root| detect_build_command(&root))
                .unwrap_or_default()
        });

    let overlay = InputPromptOverlay::new(COMPILE_PROMPT_ID, "Build command: ", &prefill);
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

/// Run `command` in the `*compile <workspace>*` streamed popup.
///
/// Stores the command per-workspace, opens (or reuses) the popup, and streams
/// stdout+stderr into it.  On success, triggers a user-library hot-reload if
/// the command targets `volt-user`.
fn run_compile_command_streamed(runtime: &mut EditorRuntime, command: &str) -> Result<(), String> {
    let command = command.trim().to_owned();
    if command.is_empty() {
        return Ok(());
    }

    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|e| e.to_string())?;
    let workspace_name = runtime
        .model()
        .active_workspace()
        .map_err(|e| e.to_string())?
        .name()
        .to_owned();

    // Store the confirmed command for this workspace.
    if let Ok(ui) = shell_ui_mut(runtime) {
        ui.compile_commands.insert(workspace_id, command.clone());
    }

    let cwd = active_workspace_root(runtime)
        .ok()
        .flatten()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let buf_name = compile_buffer_name(&workspace_name);
    let popup_title = buf_name.clone();

    let on_exit = StreamedCommandExitAction::LeaveOpenAndMaybeReloadUserLibrary {
        command: command.clone(),
    };

    let spec = ExternalCommandSpec {
        popup_title: popup_title.clone(),
        buffer_name: buf_name.clone(),
        command_label: Some(command.clone()),
        invocation: ExternalCommandInvocation::Shell(command.clone()),
        env: Vec::new(),
        cwd,
        stream: true,
        on_exit,
        notify_on_success: true,
        notify_on_failure: true,
        use_git_editor: false,
    };

    // Reuse the existing popup buffer if one with this name is already open.
    let existing_id = shell_ui(runtime).ok().and_then(|ui| {
        ui.buffers
            .iter()
            .find(|b| b.display_name() == buf_name)
            .map(|b| b.id())
    });

    if let Some(buffer_id) = existing_id {
        let (program, args, label) = resolve_external_invocation(runtime, &spec.invocation)?;
        let streamed = StreamedCommandSpec {
            popup_title: spec.popup_title,
            buffer_name: spec.buffer_name,
            command_label: spec.command_label.unwrap_or(label),
            program,
            args,
            env: spec.env,
            cwd: spec.cwd,
            on_exit: spec.on_exit,
            notify_on_success: spec.notify_on_success,
            notify_on_failure: spec.notify_on_failure,
        };
        continue_streamed_command_popup(runtime, buffer_id, streamed)?;
    } else {
        run_command(runtime, spec)?;
    }
    Ok(())
}

fn open_command_output_buffer(runtime: &mut EditorRuntime) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace_name = runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?
        .name()
        .to_owned();
    let buf_name = command_output_buffer_name(&workspace_name);
    let existing = shell_ui(runtime).ok().and_then(|ui| {
        ui.buffers
            .iter()
            .find(|buffer| buffer.display_name() == buf_name)
            .map(ShellBuffer::id)
    });
    if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.focus_buffer_in_active_pane(existing);
        ui.enter_normal_mode();
        return Ok(existing);
    }
    let id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            &buf_name,
            BufferKind::Plugin(INTERACTIVE_INPUT_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(id)
        .ok_or_else(|| format!("buffer `{id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let initial = vec![format!("# {workspace_name} — command output")];
    let shell_buf = ShellBuffer::from_runtime_buffer(buffer, initial, &*user_library);
    let ui = shell_ui_mut(runtime)?;
    ui.insert_buffer(shell_buf);
    ui.focus_buffer_in_active_pane(id);
    ui.enter_normal_mode();
    Ok(id)
}

fn run_shell_command_from_vim_command_line(
    runtime: &mut EditorRuntime,
    command: &str,
) -> Result<(), String> {
    let command = command.trim();
    if command.is_empty() {
        return Err(":! requires a shell command".to_owned());
    }
    let buffer_id = open_command_output_buffer(runtime)?;
    run_shell_command_in_buffer(runtime, buffer_id, command)
}

fn run_shell_command_in_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    command: &str,
) -> Result<(), String> {
    let command = command.trim().to_owned();
    if command.is_empty() {
        return Ok(());
    }
    let terminal_config = shell_user_library(runtime).terminal_config();
    let cwd = active_workspace_root(runtime)
        .ok()
        .flatten()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let mut args = terminal_config.args;
    let shell_program = terminal_config.program;
    args.extend(
        shell_command_eval_args(&shell_program)
            .into_iter()
            .map(str::to_owned),
    );
    args.push(command.clone());
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!("$ {command}"), String::new()]);
        buffer.clear_input();
    }
    let spec = JobSpec::command("command", shell_program, args).with_cwd(cwd);
    let manager = runtime
        .services()
        .get::<Mutex<JobManager>>()
        .ok_or_else(|| "job manager service missing".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "job manager lock poisoned".to_owned())?;
    let handle = manager.spawn(spec).map_err(|error| error.to_string())?;
    drop(manager);
    let result = handle.wait().map_err(|error| error.to_string())?;
    let transcript = result.transcript();
    let output_lines: Vec<String> = transcript.lines().map(str::to_owned).collect();
    let status_line = if result.succeeded() {
        "── ✓ Command succeeded ────────────────────────────────────────────────".to_owned()
    } else {
        format!(
            "── ✗ Command failed (exit {}) ──────────────────────────────────────",
            result.exit_code().unwrap_or(-1)
        )
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.append_output_lines(&output_lines);
    buffer.append_output_lines(&[status_line]);
    Ok(())
}

/// Flags that make `program` evaluate a one-shot shell command string.
///
/// Nushell's bare `-c` skips `config.nu` (where users often wire fnm/nvm). Login
/// (`-l -c`) loads that config so tools like `node` resolve the same way as an
/// interactive nu / profile-backed pwsh session.
pub(super) fn shell_command_eval_args(program: &str) -> Vec<&'static str> {
    let shell = Path::new(program)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    if cfg!(target_os = "windows") {
        if shell.eq_ignore_ascii_case("cmd") {
            vec!["/C"]
        } else if shell.eq_ignore_ascii_case("powershell") || shell.eq_ignore_ascii_case("pwsh") {
            vec!["-Command"]
        } else if shell.eq_ignore_ascii_case("nu") {
            vec!["-l", "-c"]
        } else {
            vec!["-c"]
        }
    } else if shell.eq_ignore_ascii_case("nu") {
        vec!["-l", "-c"]
    } else {
        vec!["-c"]
    }
}

/// Re-run the last stored build command for the active workspace.
/// If no command has been stored yet, falls back to opening the compile prompt.
fn rerun_compile_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let stored = shell_ui(runtime)
        .ok()
        .and_then(|ui| ui.compile_commands.get(&workspace_id).cloned());
    if let Some(cmd) = stored {
        run_compile_command_streamed(runtime, &cmd)
    } else {
        open_compile_prompt(runtime)
    }
}

fn command_builds_user_library(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    if !lower.contains("cargo") || !lower.contains("volt-user") {
        return false;
    }
    lower.contains("build")
        || lower.contains("check")
        || lower.contains("clippy")
        || lower.contains("test")
        || lower.contains("run")
}

fn current_runtime_user_library_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    if let Ok(env_path) = std::env::var("VOLT_USER_LIBRARY") {
        let path = PathBuf::from(env_path);
        if seen.insert(path.clone()) {
            candidates.push(path);
        }
    }
    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
    {
        let path = UserLibraryModuleRef::get_library_path(exe_dir);
        if seen.insert(path.clone()) {
            candidates.push(path);
        }
    }
    candidates
}

fn user_library_filename() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "user.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "libuser.dylib"
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        "libuser.so"
    }
}

fn built_user_library_path_for_command(runtime: &EditorRuntime, command: &str) -> Option<PathBuf> {
    let cwd = active_workspace_root(runtime)
        .ok()
        .flatten()
        .or_else(|| std::env::current_dir().ok())?;
    let profile = if command.contains("--release") {
        "release"
    } else {
        "debug"
    };
    Some(
        cwd.join("target")
            .join(profile)
            .join(user_library_filename()),
    )
}

fn stage_user_library_for_reload(built_path: &Path) -> Result<PathBuf, String> {
    if !built_path.is_file() {
        return Err(format!(
            "built user library `{}` does not exist",
            built_path.display()
        ));
    }
    let parent = built_path.parent().ok_or_else(|| {
        format!(
            "built user library `{}` has no parent directory",
            built_path.display()
        )
    })?;
    let stage_dir = parent.join("volt-user-hot");
    fs::create_dir_all(&stage_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", stage_dir.display()))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system time error: {error}"))?
        .as_millis();
    let staged_path = stage_dir.join(format!(
        "user-{}-{}-{}",
        std::process::id(),
        stamp,
        user_library_filename()
    ));
    fs::copy(built_path, &staged_path).map_err(|error| {
        format!(
            "failed to stage user library from `{}` to `{}`: {error}",
            built_path.display(),
            staged_path.display()
        )
    })?;
    Ok(staged_path)
}

fn catch_unwind_silently<F, T>(operation: F) -> Result<T, String>
where
    F: FnOnce() -> T,
{
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(AssertUnwindSafe(operation));
    panic::set_hook(hook);
    result.map_err(|payload| match payload.downcast::<String>() {
        Ok(message) => *message,
        Err(payload) => match payload.downcast::<&'static str>() {
            Ok(message) => (*message).to_owned(),
            Err(_) => "panic without message".to_owned(),
        },
    })
}

fn validate_runtime_user_library(library: &dyn UserLibrary) -> Result<(), String> {
    catch_unwind_silently(|| {
        let _ = library.syntax_languages();
    })
    .map(|_| ())
    .map_err(|message| format!("syntax language export panicked: {message}"))
}

fn replace_runtime_user_library(
    runtime: &mut EditorRuntime,
    user_library: Arc<dyn UserLibrary>,
) -> Result<Vec<String>, String> {
    let previous_packages = shell_user_library(runtime).packages();
    let next_packages = user_library.packages();

    runtime
        .services_mut()
        .insert(UserLibraryService(Arc::clone(&user_library)));

    let active_theme_id = runtime
        .services()
        .get::<ThemeRegistry>()
        .and_then(|registry| registry.active_theme().map(|theme| theme.id().to_owned()));
    let theme_registry = rebuild_theme_registry(user_library.themes(), active_theme_id.as_deref())?;
    runtime.services_mut().insert(theme_registry);
    runtime
        .services_mut()
        .insert(AutocompleteRegistry::from_user_config(&*user_library));
    runtime
        .services_mut()
        .insert(HoverRegistry::from_user_config(&*user_library));

    let mut lsp_registry = LanguageServerRegistry::new();
    lsp_registry
        .register_all(user_library.language_servers())
        .map_err(|error| error.to_string())?;
    runtime
        .services_mut()
        .insert(Arc::new(LspClientManager::new(lsp_registry)));

    let mut dap_registry = DebugAdapterRegistry::new();
    dap_registry
        .register_all(user_library.debug_adapters())
        .map_err(|error| error.to_string())?;
    runtime
        .services_mut()
        .insert(Arc::new(DapClientManager::new(dap_registry)));

    let mut syntax_registry = SyntaxRegistry::new();
    syntax_registry
        .register_all(user_library.syntax_languages())
        .map_err(|error| error.to_string())?;
    runtime.services_mut().insert(syntax_registry);
    configure_syntax_refresh_worker(runtime)?;

    let refresh_ids = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .filter(|buffer| {
                buffer.path().is_some()
                    || buffer
                        .language_id()
                        .is_some_and(|language_id| !language_id.is_empty())
            })
            .map(ShellBuffer::id)
            .collect::<Vec<_>>()
    };
    for buffer_id in refresh_ids {
        let _ = refresh_buffer_syntax(runtime, buffer_id);
    }

    let loaded_packages = reload_user_packages(runtime, &previous_packages, &next_packages)
        .map_err(|error| error.to_string())?;
    picker::ensure_picker_keybindings(runtime).map_err(|error| error.to_string())?;

    Ok(vec![
        "── ✓ User library reload requested ───────────────────────────────────".to_owned(),
        "Refreshed theme, autocomplete, hover, LSP, and syntax registries.".to_owned(),
        format!("Re-registered {loaded_packages} auto-loaded user packages."),
    ])
}

#[cfg(test)]
mod compile_reload_tests;

fn reload_user_library(runtime: &mut EditorRuntime) -> Result<Vec<String>, String> {
    let last_staged = runtime
        .services()
        .get::<UserLibraryReloadState>()
        .and_then(|state| state.last_staged_path.clone());
    let candidate_paths = last_staged
        .into_iter()
        .chain(current_runtime_user_library_candidates())
        .collect::<Vec<_>>();
    let mut last_error = None;
    for path in candidate_paths {
        if !path.is_file() {
            continue;
        }
        match DynamicUserLibrary::load_from_file(&path) {
            Ok(user_library) => {
                validate_runtime_user_library(user_library.as_ref())?;
                let mut lines = replace_runtime_user_library(runtime, user_library)?;
                lines.push(format!("Loaded runtime library from `{}`.", path.display()));
                return Ok(lines);
            }
            Err(error) => last_error = Some(format!("{}: {error}", path.display())),
        }
    }
    Err(last_error.unwrap_or_else(|| "no runtime user library candidate found".to_owned()))
}

fn execute_oil_action(runtime: &mut EditorRuntime, action: OilKeyAction) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_directory(&buffer.kind) {
        return Ok(false);
    }
    shell_ui_mut(runtime)?.pending_directory_prefix = None;
    match action {
        OilKeyAction::OpenEntry => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(runtime, buffer_id, entry, DirectoryOpenMode::Current)?;
            Ok(true)
        }
        OilKeyAction::OpenVerticalSplit => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(runtime, buffer_id, entry, DirectoryOpenMode::SplitVertical)?;
            Ok(true)
        }
        OilKeyAction::OpenHorizontalSplit => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(
                runtime,
                buffer_id,
                entry,
                DirectoryOpenMode::SplitHorizontal,
            )?;
            Ok(true)
        }
        OilKeyAction::OpenNewPane => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(runtime, buffer_id, entry, DirectoryOpenMode::NewPane)?;
            Ok(true)
        }
        OilKeyAction::PreviewEntry => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_directory_entry(runtime, buffer_id, entry, DirectoryOpenMode::Preview)?;
            Ok(true)
        }
        OilKeyAction::Refresh => {
            refresh_directory_buffer(runtime, buffer_id)?;
            Ok(true)
        }
        OilKeyAction::Close => {
            close_buffer_discard(runtime, buffer_id)?;
            Ok(true)
        }
        OilKeyAction::ShowHelp => {
            open_oil_help_popup(runtime)?;
            Ok(true)
        }
        OilKeyAction::ToggleHidden => {
            update_directory_state(runtime, buffer_id, |state| {
                state.show_hidden = !state.show_hidden;
            })?;
            Ok(true)
        }
        OilKeyAction::ToggleTrash => {
            update_directory_state(runtime, buffer_id, |state| {
                state.trash_enabled = !state.trash_enabled;
            })?;
            Ok(true)
        }
        OilKeyAction::CycleSort => {
            update_directory_state(runtime, buffer_id, |state| {
                state.sort_mode = state.sort_mode.cycle();
            })?;
            Ok(true)
        }
        OilKeyAction::OpenExternal => {
            let entry = directory_entry_at_cursor(runtime, buffer_id)?;
            open_external_path(entry.path())?;
            Ok(true)
        }
        OilKeyAction::SetTabLocalRoot | OilKeyAction::SetRoot => {
            directory_cd_from_cursor(runtime, buffer_id)?;
            Ok(true)
        }
        OilKeyAction::CreateGitWorktree => {
            eprintln!("[oil.git-worktree] oil action executing oil.git-worktree");
            record_runtime_error(
                runtime,
                "oil.git-worktree.trace",
                "oil action executing oil.git-worktree",
            );
            runtime
                .execute_command("oil.git-worktree")
                .map_err(|error| error.to_string())?;
            Ok(true)
        }
        OilKeyAction::StartPrefix => {
            set_directory_prefix(runtime, "")?;
            Ok(true)
        }
        OilKeyAction::OpenParent => {
            let root = oil_parent_root(runtime)?;
            set_directory_root(runtime, buffer_id, root)?;
            Ok(true)
        }
        OilKeyAction::OpenWorkspaceRoot => {
            let root = oil_workspace_root(runtime)?;
            set_directory_root(runtime, buffer_id, root)?;
            Ok(true)
        }
    }
}

fn handle_directory_keydown_chord(
    runtime: &mut EditorRuntime,
    chord: &str,
) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_directory(&buffer.kind) {
        return Ok(false);
    }
    shell_ui_mut(runtime)?.pending_directory_prefix = None;
    let user_library = shell_user_library(runtime);
    if let Some(action) = user_library.oil_keydown_action(chord) {
        execute_oil_action(runtime, action)
    } else {
        Ok(false)
    }
}

fn handle_directory_chord(runtime: &mut EditorRuntime, chord: &str) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_directory(&buffer.kind) {
        return Ok(false);
    }
    let prefix = take_directory_prefix(runtime)?;
    let had_prefix = prefix.is_some();
    let chord = match prefix {
        Some(prefix) => format!("{prefix}{chord}"),
        None => chord.to_owned(),
    };
    let user_library = shell_user_library(runtime);
    match user_library.oil_chord_action(had_prefix, &chord) {
        Some(action) => execute_oil_action(runtime, action),
        None if chord == "w" => {
            set_directory_prefix(runtime, "w")?;
            Ok(true)
        }
        None if had_prefix => {
            let prefix = user_library.oil_keybindings().prefix;
            record_runtime_error(
                runtime,
                "oil.directory",
                format!("unknown oil {prefix}{chord} action"),
            );
            Ok(true)
        }
        _ => Ok(false),
    }
}
