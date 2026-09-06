fn file_open_detail(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|file_name| file_name.to_str())
        .map(str::to_owned)
}

fn image_format_for_path(path: &Path) -> Option<ImageBufferFormat> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "svg" => Some(ImageBufferFormat::Svg),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "bmp" | "tif" | "tiff" => {
            Some(ImageBufferFormat::Raster)
        }
        _ => None,
    }
}

fn open_image_workspace_file(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    display_name: &str,
    path: &Path,
    format: ImageBufferFormat,
) -> Result<BufferId, String> {
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            display_name,
            BufferKind::Image,
            Some(path.to_path_buf()),
        )
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("new image buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let shell_buffer = match format {
        ImageBufferFormat::Raster => {
            let decoded = decode_raster_image_path(path)?;
            let mut text = TextBuffer::new();
            text.set_path(path.to_path_buf());
            let mut shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
            shell_buffer.set_image_state(ImageBufferState {
                format,
                mode: ImageBufferMode::Rendered,
                decoded,
                zoom: 1.0,
            });
            shell_buffer
        }
        ImageBufferFormat::Svg => {
            let text = TextBuffer::load_from_path(path)
                .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
            let decoded = rasterize_svg_text(&text.text(), Some(path))?;
            let mut shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
            shell_buffer.set_image_state(ImageBufferState {
                format,
                mode: ImageBufferMode::Rendered,
                decoded,
                zoom: 1.0,
            });
            shell_buffer.set_language_id(language_id_for_path(runtime, path).ok());
            shell_buffer
        }
    };

    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
    }

    if let Some(detail) = file_open_detail(path) {
        runtime
            .emit_hook(
                builtins::FILE_OPEN,
                HookEvent::new()
                    .with_workspace(workspace_id)
                    .with_buffer(buffer_id)
                    .with_detail(detail),
            )
            .map_err(|error| error.to_string())?;
    }

    if format == ImageBufferFormat::Svg {
        queue_buffer_syntax_refresh(runtime, buffer_id)?;
    }

    Ok(buffer_id)
}

fn open_workspace_file(runtime: &mut EditorRuntime, path: &Path) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if let Some(existing) = find_workspace_file_buffer(runtime, workspace_id, path)? {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        shell_ui_mut(runtime)?.focus_buffer_in_active_pane(existing);
        return Ok(existing);
    }

    let workspace_root = active_workspace_root(runtime)?;
    let display_name = workspace_relative_path(workspace_root.as_deref(), path);
    if let Some(format) = image_format_for_path(path) {
        return open_image_workspace_file(
            runtime,
            workspace_id,
            display_name.as_str(),
            path,
            format,
        );
    }
    if is_pdf_path(path) {
        return open_pdf_workspace_file(runtime, workspace_id, display_name.as_str(), path);
    }
    if let Some(dashboard_id) = active_dashboard_editor_buffer(runtime) {
        return load_workspace_file_into_db_editor(runtime, dashboard_id, path, &display_name);
    }
    let text = TextBuffer::load_from_path(path)
        .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            display_name.as_str(),
            BufferKind::File,
            Some(path.to_path_buf()),
        )
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("new file buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);

    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
    }

    if let Some(detail) = file_open_detail(path) {
        runtime
            .emit_hook(
                builtins::FILE_OPEN,
                HookEvent::new()
                    .with_workspace(workspace_id)
                    .with_buffer(buffer_id)
                    .with_detail(detail),
            )
            .map_err(|error| error.to_string())?;
    }
    queue_buffer_syntax_refresh(runtime, buffer_id)?;

    Ok(buffer_id)
}

fn active_dashboard_editor_buffer(runtime: &EditorRuntime) -> Option<BufferId> {
    let buffer_id = active_shell_buffer_id(runtime).ok()?;
    let buffer = shell_buffer(runtime, buffer_id).ok()?;
    buffer_is_db_dashboard(&buffer.kind).then_some(buffer_id)
}

fn load_workspace_file_into_db_editor(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    path: &Path,
    display_name: &str,
) -> Result<BufferId, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    let language_id = language_id_for_path(runtime, path).ok();
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .set_buffer_name(workspace_id, buffer_id, display_name.to_owned())
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .set_buffer_path(workspace_id, buffer_id, Some(path.to_path_buf()))
        .map_err(|error| error.to_string())?;
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.replace_with_lines(contents.lines().map(str::to_owned).collect());
        buffer.text.set_path(path.to_path_buf());
        buffer.text.mark_clean();
        buffer.plugin_focus_section_named(DB_EDITOR_SECTION);
        if let Some(language_id) = language_id {
            buffer.set_language_id(Some(language_id));
        }
        buffer.set_lsp_path(Some(path.to_path_buf()));
        buffer.force_syntax_refresh();
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_buffer_in_active_pane(buffer_id);
    Ok(buffer_id)
}

fn open_workspace_file_at(
    runtime: &mut EditorRuntime,
    path: &Path,
    target: TextPoint,
) -> Result<(), String> {
    let buffer_id = open_workspace_file(runtime, path)?;
    if let Some(buffer) = shell_ui_mut(runtime)?.buffer_mut(buffer_id) {
        buffer.set_cursor(target);
    }
    Ok(())
}

fn create_workspace_file_from_query(
    runtime: &mut EditorRuntime,
    root: &Path,
    query: &str,
) -> Result<(), String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let input_path = PathBuf::from(trimmed);
    if input_path.is_absolute() {
        return Err(format!(
            "workspace file path must be relative: {}",
            input_path.display()
        ));
    }

    if input_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("workspace file path must not contain `..`".to_owned());
    }

    if input_path.file_name().is_none() {
        return Err("workspace file path must include a file name".to_owned());
    }

    let path = root.join(input_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
    }

    if !path.exists() {
        fs::File::create(&path)
            .map_err(|error| format!("failed to create `{}`: {error}", path.display()))?;
    }

    open_workspace_file(runtime, &path)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn workspace_switch_picker_overlay(
    runtime: &EditorRuntime,
) -> Result<PickerOverlay, String> {
    picker::picker_overlay(runtime, "workspace.switch")
}

#[cfg(test)]
pub(crate) fn workspace_delete_picker_overlay(
    runtime: &EditorRuntime,
) -> Result<PickerOverlay, String> {
    picker::picker_overlay(runtime, "workspace.delete")
}

fn apply_undo_tree_node(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    node_id: usize,
) -> Result<(), String> {
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    if buffer.undo_tree_select(node_id) {
        buffer.mark_syntax_dirty();
        Ok(())
    } else {
        Err("undo tree node is missing".to_owned())
    }
}

fn workspace_relative_path(root: Option<&Path>, path: &Path) -> String {
    root.and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path)
        .display()
        .to_string()
}
