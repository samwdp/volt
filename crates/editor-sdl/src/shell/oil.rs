fn syntax_registry_mut(runtime: &mut EditorRuntime) -> Result<&mut SyntaxRegistry, String> {
    runtime
        .services_mut()
        .get_mut::<SyntaxRegistry>()
        .ok_or_else(|| "syntax registry service missing".to_owned())
}

fn formatter_registry(runtime: &EditorRuntime) -> Result<&FormatterRegistry, String> {
    runtime
        .services()
        .get::<FormatterRegistry>()
        .ok_or_else(|| "formatter registry service missing".to_owned())
}

fn formatter_registry_mut(runtime: &mut EditorRuntime) -> Result<&mut FormatterRegistry, String> {
    runtime
        .services_mut()
        .get_mut::<FormatterRegistry>()
        .ok_or_else(|| "formatter registry service missing".to_owned())
}

fn sync_active_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some((pane_id, buffer_id, buffer_name, buffer_kind)) = active_runtime_buffer(runtime)?
    else {
        return Ok(());
    };
    let is_git_commit = buffer_is_git_commit(&buffer_kind);
    let is_git_status = buffer_is_git_status(&buffer_kind);
    let is_directory = buffer_is_directory(&buffer_kind);
    let is_terminal = buffer_is_terminal(&buffer_kind);

    let (previous_pane, previous_buffer) = {
        let ui = shell_ui(runtime)?;
        (ui.active_pane_id(), ui.active_buffer_id())
    };
    let should_enter_insert = {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        if previous_pane != Some(pane_id) {
            ui.focus_pane(pane_id);
        } else if previous_buffer != Some(buffer_id) {
            ui.close_autocomplete();
            ui.close_hover();
        }
        let has_input = ui
            .ensure_buffer(buffer_id, &buffer_name, buffer_kind, &*user_library)
            .has_input_field();
        ui.focus_buffer_in_active_pane(buffer_id);
        if !is_git_commit {
            ui.pending_ctrl_c = None;
        }
        if !is_git_status {
            ui.pending_git_prefix = None;
        }
        if !is_directory {
            ui.pending_directory_prefix = None;
        }
        previous_buffer != Some(buffer_id) && has_input
    };
    let terminal_created = if is_terminal {
        ensure_terminal_session(runtime, buffer_id)?
    } else {
        false
    };
    if should_enter_insert || terminal_created {
        shell_ui_mut(runtime)?.enter_insert_mode();
    }
    if previous_buffer != Some(buffer_id) {
        let workspace_id = runtime
            .model()
            .active_workspace_id()
            .map_err(|error| error.to_string())?;
        let window_id = active_window_id(runtime)?;
        runtime
            .emit_hook(
                builtins::BUFFER_SWITCH,
                HookEvent::new()
                    .with_window(window_id)
                    .with_workspace(workspace_id)
                    .with_pane(pane_id)
                    .with_buffer(buffer_id),
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn active_runtime_surface(runtime: &EditorRuntime) -> Result<Option<(PaneId, BufferId)>, String> {
    Ok(active_runtime_buffer(runtime)?.map(|(pane_id, buffer_id, _, _)| (pane_id, buffer_id)))
}

fn ensure_shell_buffer(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let (buffer_name, buffer_kind) = {
        let workspace_id = runtime
            .model()
            .active_workspace_id()
            .map_err(|error| error.to_string())?;
        let workspace = runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?;
        let buffer = workspace
            .buffer(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        (buffer.name().to_owned(), buffer.kind().clone())
    };
    let user_library = shell_user_library(runtime);
    shell_ui_mut(runtime)?.ensure_popup_buffer(
        buffer_id,
        &buffer_name,
        buffer_kind,
        &*user_library,
    );
    Ok(())
}

fn find_shell_buffer_by_kind(ui: &ShellUiState, kind: &str) -> Option<BufferId> {
    ui.buffers.iter().find_map(|buffer| {
        if matches!(&buffer.kind, BufferKind::Plugin(plugin_kind) if plugin_kind == kind) {
            Some(buffer.id())
        } else {
            None
        }
    })
}

fn find_oil_buffer(runtime: &EditorRuntime, workspace_id: WorkspaceId) -> Option<BufferId> {
    runtime
        .model()
        .workspace(workspace_id)
        .ok()?
        .buffers()
        .find_map(|buffer| {
            if matches!(buffer.kind(), BufferKind::Directory) && buffer.name() == OIL_BUFFER_NAME {
                Some(buffer.id())
            } else {
                None
            }
        })
}

fn active_shell_buffer_path(runtime: &EditorRuntime) -> Result<Option<PathBuf>, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    Ok(shell_buffer(runtime, buffer_id)?
        .path()
        .map(Path::to_path_buf))
}

fn active_directory_root(runtime: &EditorRuntime) -> Result<Option<PathBuf>, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    Ok(buffer.directory_state().map(|state| state.root.clone()))
}

fn oil_workspace_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_workspace_root(runtime)? {
        return Ok(root);
    }
    env::current_dir().map_err(|error| format!("oil requires a workspace root: {error}"))
}

fn oil_default_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_directory_root(runtime)? {
        return Ok(root);
    }
    if let Some(path) = active_shell_buffer_path(runtime)?
        && let Some(parent) = path.parent()
    {
        return Ok(parent.to_path_buf());
    }
    oil_workspace_root(runtime)
}

fn oil_parent_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_directory_root(runtime)? {
        return Ok(root.parent().unwrap_or(root.as_path()).to_path_buf());
    }
    let root = oil_default_root(runtime)?;
    Ok(root.parent().unwrap_or(root.as_path()).to_path_buf())
}

fn open_oil_directory(runtime: &mut EditorRuntime, root: PathBuf) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let existing = find_oil_buffer(runtime, workspace_id);
    let buffer_id = if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        existing
    } else {
        runtime
            .model_mut()
            .create_buffer(workspace_id, OIL_BUFFER_NAME, BufferKind::Directory, None)
            .map_err(|error| error.to_string())?
    };
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_buffer(
            buffer_id,
            OIL_BUFFER_NAME,
            BufferKind::Directory,
            &*user_library,
        );
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    set_directory_root(runtime, buffer_id, root)?;
    Ok(())
}

fn ensure_directory_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_directory(&buffer.kind) || buffer.directory_state().is_some() {
        return Ok(());
    }
    let root = oil_default_root(runtime)?;
    set_directory_root(runtime, buffer_id, root)
}

fn refresh_directory_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let (root, show_hidden, sort_mode, trash_enabled) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let Some(state) = buffer.directory_state() else {
            let root = oil_default_root(runtime)?;
            return set_directory_root(runtime, buffer_id, root);
        };
        (
            state.root.clone(),
            state.show_hidden,
            state.sort_mode,
            state.trash_enabled,
        )
    };
    let entries = match DirectoryBuffer::read(&root) {
        Ok(buffer) => buffer.entries().to_vec(),
        Err(error) => {
            let message = format!("failed to read `{}`: {error}", root.display());
            set_directory_error(runtime, buffer_id, &message)?;
            return Err(message);
        }
    };
    let defaults = shell_user_library(runtime).oil_defaults();
    let mut state = DirectoryViewState::new(root, entries, defaults);
    state.show_hidden = show_hidden;
    state.sort_mode = sort_mode;
    state.trash_enabled = trash_enabled;
    apply_directory_state(runtime, buffer_id, state)
}

fn set_directory_root(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    root: PathBuf,
) -> Result<(), String> {
    let defaults = shell_user_library(runtime).oil_defaults();
    let (show_hidden, sort_mode, trash_enabled, previous_root) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let state = buffer.directory_state();
        (
            state
                .map(|state| state.show_hidden)
                .unwrap_or(defaults.show_hidden),
            state
                .map(|state| state.sort_mode)
                .unwrap_or(defaults.sort_mode),
            state
                .map(|state| state.trash_enabled)
                .unwrap_or(defaults.trash_enabled),
            state.map(|state| state.root.clone()),
        )
    };
    let root_for_compare = root.clone();
    let entries = match DirectoryBuffer::read(&root) {
        Ok(buffer) => buffer.entries().to_vec(),
        Err(error) => {
            let message = format!("failed to read `{}`: {error}", root.display());
            set_directory_error(runtime, buffer_id, &message)?;
            return Err(message);
        }
    };
    let mut state = DirectoryViewState::new(root, entries, defaults);
    state.show_hidden = show_hidden;
    state.sort_mode = sort_mode;
    state.trash_enabled = trash_enabled;
    apply_directory_state(runtime, buffer_id, state)?;
    if previous_root.as_ref() != Some(&root_for_compare) {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let target_line = if buffer.line_count() > 1 { 1 } else { 0 };
        buffer.goto_line(target_line);
        buffer.scroll_row = 0;
    }
    Ok(())
}

fn set_directory_error(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    message: &str,
) -> Result<(), String> {
    record_runtime_error(runtime, "oil.directory", message.to_owned());
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.clear_directory_state();
    buffer.section_state = None;
    buffer.replace_with_lines(vec![
        "Directory view unavailable.".to_owned(),
        message.to_owned(),
    ]);
    Ok(())
}

fn open_file_in_split(
    runtime: &mut EditorRuntime,
    path: &Path,
    direction: PaneSplitDirection,
    focus: bool,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let original_pane_id = shell_ui(runtime)?
        .active_pane_id()
        .ok_or_else(|| "active pane is missing".to_owned())?;
    if shell_ui(runtime)?.pane_count() < 2 {
        split_runtime_pane(runtime, direction)?;
    }
    let target_pane_id = shell_ui(runtime)?
        .panes()
        .and_then(|panes| {
            panes
                .iter()
                .find(|pane| pane.pane_id != original_pane_id)
                .map(|pane| pane.pane_id)
        })
        .ok_or_else(|| "split pane is missing".to_owned())?;
    runtime
        .model_mut()
        .focus_pane(workspace_id, target_pane_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_pane(target_pane_id);
    open_workspace_file(runtime, path)?;
    if !focus {
        runtime
            .model_mut()
            .focus_pane(workspace_id, original_pane_id)
            .map_err(|error| error.to_string())?;
        shell_ui_mut(runtime)?.focus_pane(original_pane_id);
    }
    Ok(())
}

fn open_oil_preview_popup(runtime: &mut EditorRuntime, path: &Path) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let existing = shell_ui(runtime)
        .ok()
        .and_then(|ui| find_shell_buffer_by_kind(ui, OIL_PREVIEW_KIND));
    let buffer_id = if let Some(existing) = existing {
        existing
    } else {
        runtime
            .model_mut()
            .create_popup_buffer(
                workspace_id,
                OIL_PREVIEW_BUFFER_NAME,
                BufferKind::Plugin(OIL_PREVIEW_KIND.to_owned()),
                None,
            )
            .map_err(|error| error.to_string())?
    };
    runtime
        .model_mut()
        .open_popup_buffer(workspace_id, "Preview", buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.set_popup_focus(false);
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let text = TextBuffer::load_from_path(path)
        .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    let user_library = shell_user_library(runtime);
    let shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
    shell_ui_mut(runtime)?.insert_buffer(shell_buffer);
    queue_buffer_syntax_refresh(runtime, buffer_id)?;
    Ok(())
}

fn open_oil_help_popup(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let existing = shell_ui(runtime)
        .ok()
        .and_then(|ui| find_shell_buffer_by_kind(ui, OIL_HELP_KIND));
    let buffer_id = if let Some(existing) = existing {
        existing
    } else {
        runtime
            .model_mut()
            .create_popup_buffer(
                workspace_id,
                OIL_HELP_BUFFER_NAME,
                BufferKind::Plugin(OIL_HELP_KIND.to_owned()),
                None,
            )
            .map_err(|error| error.to_string())?
    };
    runtime
        .model_mut()
        .open_popup_buffer(workspace_id, "Oil Help", buffer_id)
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui_mut(runtime)?;
        ui.set_popup_buffer(buffer_id);
        ui.set_popup_focus(true);
    }
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let shell_buffer =
        ShellBuffer::from_runtime_buffer(buffer, user_library.oil_help_lines(), &*user_library);
    shell_ui_mut(runtime)?.insert_buffer(shell_buffer);
    Ok(())
}

fn open_external_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("path does not exist: {}", path.display()));
    }
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("cmd");
        configure_background_command(&mut command);
        command
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    }
    Ok(())
}

/// Evaluate the input section of any evaluatable plugin buffer and replace the
/// output section with the result.  Called both by the generic Ctrl+c Ctrl+c
/// handler and by the `plugin.evaluate` hook subscriber.
fn evaluate_active_plugin_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    // Read the buffer kind (needed to route the evaluate call).
    let (input_lines, sep_line, kind_str, has_plugin_sections) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let kind_str = if let BufferKind::Plugin(k) = &buffer.kind {
            k.clone()
        } else {
            return Ok(()); // not a plugin buffer; nothing to do
        };
        if buffer.has_plugin_sections() {
            let line_count = buffer.text.line_count();
            let all_lines: Vec<String> = (0..line_count)
                .map(|i| buffer.text.line(i).unwrap_or_default().to_owned())
                .collect();
            (all_lines, String::new(), kind_str, true)
        } else {
            let line_count = buffer.text.line_count();
            let all_lines: Vec<String> = (0..line_count)
                .map(|i| buffer.text.line(i).unwrap_or_default().to_owned())
                .collect();
            if let Some(idx) = all_lines
                .iter()
                .position(|l| l.starts_with(PLUGIN_EVALUATE_SEPARATOR_PREFIX))
            {
                let input = all_lines[..idx].to_vec();
                let sep = all_lines[idx].clone();
                (input, sep, kind_str, false)
            } else {
                // No separator — treat everything as input; add a fresh separator.
                let sep = format!("{} {}", PLUGIN_EVALUATE_SEPARATOR_PREFIX, "─".repeat(48));
                (all_lines, sep, kind_str, false)
            }
        }
    };

    let input_text = input_lines.join("\n");

    // Call user library evaluator (no mutable borrow of runtime required).
    let output = shell_user_library(runtime).handle_plugin_evaluate(&kind_str, &input_text);

    if has_plugin_sections {
        shell_buffer_mut(runtime, buffer_id)?.set_plugin_output_lines(output);
        return Ok(());
    }

    // Rebuild: input + separator + output.
    let mut new_lines = input_lines;
    new_lines.push(sep_line);
    new_lines.extend(output);

    shell_buffer_mut(runtime, buffer_id)?.replace_with_lines(new_lines);
    Ok(())
}

fn switch_active_plugin_pane(
    runtime: &mut EditorRuntime,
    buffer_id: Option<BufferId>,
) -> Result<(), String> {
    let buffer_id = buffer_id.unwrap_or(active_shell_buffer_id(runtime)?);
    let switched_to_read_only = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if buffer.plugin_switch_pane() {
            buffer.is_read_only()
        } else {
            return acp::acp_switch_pane(runtime);
        }
    };
    if switched_to_read_only {
        shell_ui_mut(runtime)?.enter_normal_mode();
    }
    Ok(())
}

// ─── Generic compile / build infrastructure ───────────────────────────────────
