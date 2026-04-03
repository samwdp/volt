use super::*;

#[derive(Debug, Clone)]
pub(super) struct SearchPickerData {
    pub(super) entries: Vec<PickerEntry>,
    pub(super) selected_index: usize,
}

pub(super) fn workspace_search_entries(root: &Path, query: &str) -> SearchPickerData {
    let query = query.trim();
    if query.is_empty() {
        return SearchPickerData {
            entries: Vec::new(),
            selected_index: 0,
        };
    }

    let entries = match workspace_search_output(root, query) {
        Ok(output) => {
            let parsed = parse_workspace_search_entries(root, query, &output);
            if parsed.is_empty() {
                vec![workspace_search_status_entry(
                    query,
                    "No matches found",
                    format!("No workspace results for `{query}`."),
                    Some(root.display().to_string()),
                )]
            } else {
                parsed
            }
        }
        Err(error) => vec![workspace_search_status_entry(
            query,
            "Search unavailable",
            error,
            Some(root.display().to_string()),
        )],
    };

    SearchPickerData {
        entries,
        selected_index: 0,
    }
}

pub(super) fn workspace_search_output(root: &Path, query: &str) -> Result<String, String> {
    match workspace_search_rg_output(root, query) {
        Ok(output) => Ok(output),
        Err(rg_error) => workspace_search_grep_output(root, query).map_err(|grep_error| {
            format!("workspace search requires `rg` or `grep`: {rg_error}; {grep_error}")
        }),
    }
}

pub(super) fn workspace_search_rg_output(root: &Path, query: &str) -> Result<String, String> {
    let mut args = vec![
        "--vimgrep".to_owned(),
        "--no-heading".to_owned(),
        "--color".to_owned(),
        "never".to_owned(),
        "--fixed-strings".to_owned(),
    ];
    if !search_is_case_sensitive(query) {
        args.push("--ignore-case".to_owned());
    }
    args.push("--".to_owned());
    args.push(query.to_owned());
    args.push(".".to_owned());
    run_search_command(root, "rg", &args)
}

pub(super) fn workspace_search_grep_output(root: &Path, query: &str) -> Result<String, String> {
    let mut args = vec![
        "-R".to_owned(),
        "-n".to_owned(),
        "-F".to_owned(),
        "--binary-files=without-match".to_owned(),
        "--exclude-dir=.git".to_owned(),
    ];
    if !search_is_case_sensitive(query) {
        args.push("-i".to_owned());
    }
    args.push("--".to_owned());
    args.push(query.to_owned());
    args.push(".".to_owned());
    run_search_command(root, "grep", &args)
}

pub(super) fn run_search_command(
    root: &Path,
    command: &str,
    args: &[String],
) -> Result<String, String> {
    let mut process = Command::new(command);
    configure_background_command(&mut process);
    let output = process
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run `{command}`: {error}"))?;
    let exit_code = output
        .status
        .code()
        .ok_or_else(|| format!("`{command}` terminated unexpectedly"))?;
    if exit_code != 0 && exit_code != 1 {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if stderr.is_empty() {
            format!("`{command}` exited with status {exit_code}")
        } else {
            format!("`{command}` exited with status {exit_code}: {stderr}")
        };
        return Err(message);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(super) fn parse_workspace_search_entries(
    root: &Path,
    query: &str,
    output: &str,
) -> Vec<PickerEntry> {
    output
        .lines()
        .filter_map(|line| {
            parse_rg_workspace_search_line(line)
                .or_else(|| parse_grep_workspace_search_line(line, query))
                .map(|(relative_path, line_number, column, line_text)| {
                    workspace_search_match_entry(
                        root,
                        &relative_path,
                        line_number,
                        column,
                        &line_text,
                    )
                })
        })
        .take(SEARCH_PICKER_ITEM_LIMIT)
        .collect()
}

pub(super) fn parse_rg_workspace_search_line(line: &str) -> Option<(String, usize, usize, String)> {
    let mut parts = line.splitn(4, ':');
    let relative_path = parts.next()?.to_owned();
    let line_number = parts.next()?.parse::<usize>().ok()?;
    let column = parts.next()?.parse::<usize>().ok()?;
    let line_text = parts.next()?.to_owned();
    Some((relative_path, line_number, column, line_text))
}

pub(super) fn parse_grep_workspace_search_line(
    line: &str,
    query: &str,
) -> Option<(String, usize, usize, String)> {
    let mut parts = line.splitn(3, ':');
    let relative_path = parts.next()?.to_owned();
    let line_number = parts.next()?.parse::<usize>().ok()?;
    let line_text = parts.next()?.to_owned();
    let column = workspace_search_grep_column(&line_text, query);
    Some((relative_path, line_number, column, line_text))
}

pub(super) fn workspace_search_grep_column(line_text: &str, query: &str) -> usize {
    if query.is_empty() {
        return 1;
    }
    let case_sensitive = search_is_case_sensitive(query);
    if case_sensitive {
        return line_text.find(query).map(|offset| offset + 1).unwrap_or(1);
    }
    let line_lower = line_text.to_lowercase();
    let query_lower = query.to_lowercase();
    line_lower
        .find(&query_lower)
        .map(|offset| offset + 1)
        .unwrap_or(1)
}

pub(super) fn workspace_search_match_entry(
    root: &Path,
    relative_path: &str,
    line_number: usize,
    byte_column: usize,
    line_text: &str,
) -> PickerEntry {
    let relative_path = relative_path
        .strip_prefix(".\\")
        .or_else(|| relative_path.strip_prefix("./"))
        .unwrap_or(relative_path);
    let path = root.join(relative_path);
    let column = workspace_search_char_column(line_text, byte_column.saturating_sub(1));
    let target = TextPoint::new(line_number.saturating_sub(1), column);
    let preview = line_text.trim();
    let label = if preview.is_empty() {
        format!("{relative_path}:{}", line_number)
    } else {
        preview.to_owned()
    };
    let detail = format!("{} | Ln {}, Col {}", relative_path, line_number, column + 1);
    PickerEntry {
        item: PickerItem::new(
            format!("{}:{}:{}", path.display(), line_number, column + 1),
            label,
            detail,
            Some(path.display().to_string()),
        ),
        action: PickerAction::OpenFileLocation { path, target },
    }
}

pub(super) fn workspace_search_status_entry(
    query: &str,
    title: &str,
    detail: impl Into<String>,
    preview: Option<String>,
) -> PickerEntry {
    PickerEntry {
        item: PickerItem::new(
            format!("workspace-search:{query}"),
            format!("{title} for {query}"),
            detail,
            preview,
        ),
        action: PickerAction::NoOp,
    }
}

pub(super) fn lsp_locations_picker_overlay(
    runtime: &EditorRuntime,
    title: &str,
    locations: &[LspLocation],
) -> PickerOverlay {
    let workspace_root = active_workspace_root(runtime).ok().flatten();
    let entries = locations
        .iter()
        .take(SEARCH_PICKER_ITEM_LIMIT)
        .map(|location| lsp_location_picker_entry(workspace_root.as_deref(), location))
        .collect();
    PickerOverlay::from_entries(title, entries)
}

pub(super) fn lsp_location_picker_entry(
    workspace_root: Option<&Path>,
    location: &LspLocation,
) -> PickerEntry {
    let path = location.path().to_path_buf();
    let target = location.range().start();
    let relative_path = workspace_relative_path(workspace_root, &path);
    let line_number = target.line + 1;
    let column = target.column + 1;
    let preview = TextBuffer::load_from_path(&path)
        .ok()
        .and_then(|buffer| buffer.line(target.line))
        .map(|line| line.trim().to_owned())
        .filter(|line| !line.is_empty());
    let label = preview
        .clone()
        .unwrap_or_else(|| format!("{relative_path}:{line_number}"));
    let detail = format!(
        "{relative_path} | Ln {line_number}, Col {column} | {}",
        location.server_id()
    );
    PickerEntry {
        item: PickerItem::new(
            format!("lsp:{}:{}:{}", path.display(), line_number, column),
            label,
            detail,
            Some(path.display().to_string()),
        ),
        action: PickerAction::OpenFileLocation { path, target },
    }
}

pub(super) fn lsp_code_actions_picker_overlay(
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    code_actions: &[LspCodeAction],
) -> PickerOverlay {
    let entries = code_actions
        .iter()
        .take(SEARCH_PICKER_ITEM_LIMIT)
        .enumerate()
        .map(|(index, code_action)| {
            lsp_code_action_picker_entry(workspace_id, buffer_id, path, index, code_action)
        })
        .collect();
    PickerOverlay::from_entries("Code Actions", entries)
}

pub(super) fn lsp_code_action_picker_entry(
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    index: usize,
    code_action: &LspCodeAction,
) -> PickerEntry {
    let status = if code_action.disabled_reason().is_some() {
        "disabled"
    } else if lsp_code_action_supported_edits(code_action, path).is_ok() {
        "edit"
    } else {
        "unsupported"
    };
    let mut detail_parts = vec![code_action.server_id().to_owned()];
    if let Some(kind) = code_action.kind() {
        detail_parts.push(kind.to_owned());
    }
    if code_action.is_preferred() {
        detail_parts.push("preferred".to_owned());
    }
    detail_parts.push(status.to_owned());
    PickerEntry {
        item: PickerItem::new(
            format!("lsp-code-action:{index}"),
            code_action.title(),
            detail_parts.join(" | "),
            lsp_code_action_picker_preview(code_action, path),
        ),
        action: PickerAction::ApplyLspCodeAction {
            workspace_id,
            buffer_id,
            path: path.to_path_buf(),
            code_action: code_action.clone(),
        },
    }
}

pub(super) fn lsp_code_action_picker_preview(
    code_action: &LspCodeAction,
    path: &Path,
) -> Option<String> {
    match lsp_code_action_supported_edits(code_action, path) {
        Ok(edits) => {
            let suffix = if edits.len() == 1 { "" } else { "s" };
            Some(format!(
                "Apply {} text edit{suffix} to the active file.",
                edits.len()
            ))
        }
        Err(message) => Some(message),
    }
}

pub(super) fn lsp_code_action_supported_edits<'a>(
    code_action: &'a LspCodeAction,
    path: &Path,
) -> Result<&'a [LspTextEdit], String> {
    if let Some(reason) = code_action.disabled_reason() {
        return Err(format!("Disabled: {reason}"));
    }
    if code_action.has_resource_operations() {
        return Err("Unsupported: resource operations are not supported yet.".to_owned());
    }
    if let Some(command_name) = code_action.command_name() {
        return Err(format!(
            "Unsupported: command `{command_name}` code actions are not supported yet."
        ));
    }
    if code_action.document_edits().is_empty() {
        return Err("Unsupported: this code action does not include text edits.".to_owned());
    }
    if code_action.document_edits().len() != 1 {
        return Err("Unsupported: multi-file code actions are not supported yet.".to_owned());
    }
    let document_edit = &code_action.document_edits()[0];
    if document_edit.path() != path {
        return Err(format!(
            "Unsupported: this code action edits `{}` instead of the active file.",
            document_edit.path().display()
        ));
    }
    Ok(document_edit.edits())
}

pub(super) fn apply_lsp_code_action(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    code_action: &LspCodeAction,
) -> Result<(), String> {
    let edits = lsp_code_action_supported_edits(code_action, path)?;
    let original_cursor = shell_buffer(runtime, buffer_id)?.cursor_point();
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if buffer.path() != Some(path) {
            return Err(format!(
                "buffer `{}` is no longer available for `{}`",
                path.display(),
                code_action.title()
            ));
        }
        apply_lsp_text_edits(buffer, edits);
        buffer.set_cursor(original_cursor);
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(runtime)?;
    Ok(())
}

pub(super) fn workspace_search_char_column(line: &str, byte_offset: usize) -> usize {
    let mut byte_offset = byte_offset.min(line.len());
    while byte_offset > 0 && !line.is_char_boundary(byte_offset) {
        byte_offset = byte_offset.saturating_sub(1);
    }
    line[..byte_offset].chars().count()
}
