use std::io::{BufRead as _, BufReader, Read as _};
use std::process::Stdio;

use editor_jobs::{ProcessSupervisionMode, supervised_command_if_resolved};
use editor_lsp::LspDocumentTextEdits;
use url::Url;

use super::*;

const WORKSPACE_SEARCH_OUTPUT_LIMIT: usize = 48;
const LSP_CODE_ACTION_KIND_ORDER: [&str; 8] = [
    "quickfix",
    "refactor",
    "refactor.inline",
    "refactor.extract",
    "refactor.rewrite",
    "source",
    "source.organizeImports",
    "source.fixAll",
];

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
    let (program, args) = supervised_command_if_resolved(
        command,
        args,
        &[],
        None,
        ProcessSupervisionMode::Background,
    );
    let mut process = Command::new(&program);
    configure_background_command(&mut process);
    let mut child = process
        .args(&args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to run `{command}`: {error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("failed to capture `{command}` stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("failed to capture `{command}` stderr"))?;
    let stderr_reader = std::thread::spawn(move || -> std::io::Result<Vec<u8>> {
        let mut stderr = BufReader::new(stderr);
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes)?;
        Ok(bytes)
    });

    let (stdout, reached_limit) = collect_search_output(stdout, WORKSPACE_SEARCH_OUTPUT_LIMIT)
        .map_err(|error| format!("failed to read `{command}` output: {error}"))?;
    if reached_limit {
        let _ = child.kill();
        let _ = child.wait();
        let _ = stderr_reader.join();
        return Ok(stdout);
    }

    let status = child
        .wait()
        .map_err(|error| format!("failed to wait for `{command}`: {error}"))?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| format!("failed to read `{command}` stderr"))?
        .map_err(|error| format!("failed to read `{command}` stderr: {error}"))?;
    let exit_code = status
        .code()
        .ok_or_else(|| format!("`{command}` terminated unexpectedly"))?;
    if exit_code != 0 && exit_code != 1 {
        let stderr = String::from_utf8_lossy(&stderr).trim().to_owned();
        let message = if stderr.is_empty() {
            format!("`{command}` exited with status {exit_code}")
        } else {
            format!("`{command}` exited with status {exit_code}: {stderr}")
        };
        return Err(message);
    }
    Ok(stdout)
}

pub(super) fn collect_search_output(
    stdout: impl std::io::Read,
    limit: usize,
) -> std::io::Result<(String, bool)> {
    let mut reader = BufReader::new(stdout);
    let mut output = String::new();
    let mut line = String::new();
    let mut count = 0;
    let limit = limit.max(1);

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok((output, false));
        }

        output.push_str(&line);
        count += 1;
        if count >= limit {
            return Ok((output, true));
        }
    }
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
        .take(WORKSPACE_SEARCH_OUTPUT_LIMIT)
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
    let target = location.range().start();
    let line_number = target.line + 1;
    let column = target.column + 1;
    let (label, detail, preview) = if let Some(path) = location.file_path() {
        let relative_path = workspace_relative_path(workspace_root, path);
        let preview = TextBuffer::load_from_path(path)
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
        let preview = preview.or_else(|| Some(path.display().to_string()));
        (label, detail, preview)
    } else {
        let uri_label = lsp_location_uri_label(location.uri());
        let detail = format!(
            "{} | Ln {line_number}, Col {column} | {}",
            lsp_location_uri_detail(location.uri()),
            location.server_id()
        );
        (uri_label, detail, Some(location.uri().to_owned()))
    };
    PickerEntry {
        item: PickerItem::new(
            format!("lsp:{}:{}:{}", location.uri(), line_number, column),
            label,
            detail,
            preview,
        ),
        action: PickerAction::OpenLspLocation {
            location: location.clone(),
        },
    }
}

fn lsp_location_uri_label(uri: &str) -> String {
    Url::parse(uri)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()
                .and_then(|mut segments| segments.rfind(|segment| !segment.is_empty()))
                .map(str::to_owned)
        })
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| uri.to_owned())
}

fn lsp_location_uri_detail(uri: &str) -> String {
    Url::parse(uri)
        .ok()
        .map(|parsed| {
            let path = parsed.path().trim_start_matches('/');
            if path.is_empty() {
                format!("{}:/", parsed.scheme())
            } else {
                format!("{}:/{}", parsed.scheme(), path)
            }
        })
        .unwrap_or_else(|| uri.to_owned())
}

pub(super) fn lsp_code_actions_picker_overlay(
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    code_actions: &[LspCodeAction],
) -> PickerOverlay {
    let entries = lsp_code_action_sorted_indices(code_actions.iter().map(LspCodeAction::kind))
        .into_iter()
        .take(SEARCH_PICKER_ITEM_LIMIT)
        .enumerate()
        .map(|(index, action_index)| {
            lsp_code_action_picker_entry(
                workspace_id,
                buffer_id,
                path,
                index,
                &code_actions[action_index],
            )
        })
        .collect();
    lsp_code_actions_picker_overlay_from_entries(entries)
}

fn lsp_code_actions_picker_overlay_from_entries(entries: Vec<PickerEntry>) -> PickerOverlay {
    PickerOverlay::from_entries("Code Actions", entries)
        .with_result_order(PickerResultOrder::Source)
}

fn lsp_code_action_sorted_indices<'a>(
    kinds: impl IntoIterator<Item = Option<&'a str>>,
) -> Vec<usize> {
    let mut indexed_kinds = kinds.into_iter().enumerate().collect::<Vec<_>>();
    indexed_kinds.sort_by_key(|(index, kind)| (lsp_code_action_kind_rank(*kind), *index));
    indexed_kinds.into_iter().map(|(index, _)| index).collect()
}

fn lsp_code_action_kind_rank(kind: Option<&str>) -> usize {
    kind.and_then(lsp_code_action_explicit_kind_rank)
        .unwrap_or(LSP_CODE_ACTION_KIND_ORDER.len())
}

fn lsp_code_action_explicit_kind_rank(kind: &str) -> Option<usize> {
    LSP_CODE_ACTION_KIND_ORDER
        .iter()
        .enumerate()
        .filter(|(_, explicit_kind)| lsp_code_action_kind_matches(kind, explicit_kind))
        .max_by_key(|(_, explicit_kind)| explicit_kind.len())
        .map(|(rank, _)| rank)
}

fn lsp_code_action_kind_matches(kind: &str, explicit_kind: &str) -> bool {
    kind == explicit_kind
        || kind
            .strip_prefix(explicit_kind)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

pub(super) fn lsp_code_actions_status_picker_overlay(
    label: &str,
    detail: &str,
    preview: Option<String>,
) -> PickerOverlay {
    PickerOverlay::from_entries(
        "Code Actions",
        vec![PickerEntry {
            item: PickerItem::new("lsp-code-action-status", label, detail, preview),
            action: PickerAction::NoOp,
        }],
    )
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
    } else if lsp_code_action_supported_edits(code_action).is_ok() {
        if code_action.command_name().is_some() {
            "edit + command"
        } else {
            "edit"
        }
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
            lsp_code_action_picker_preview(code_action),
        ),
        action: PickerAction::ApplyLspCodeAction {
            workspace_id,
            buffer_id,
            path: path.to_path_buf(),
            code_action: code_action.clone(),
        },
    }
}

pub(super) fn lsp_code_action_picker_preview(code_action: &LspCodeAction) -> Option<String> {
    match lsp_code_action_supported_edits(code_action) {
        Ok(document_edits) => {
            let edit_count = document_edits
                .iter()
                .map(|document_edit| document_edit.edits().len())
                .sum::<usize>();
            let file_count = document_edits.len();
            let edit_suffix = if edit_count == 1 { "" } else { "s" };
            let file_suffix = if file_count == 1 { "" } else { "s" };
            let command_suffix = if code_action.command_name().is_some() {
                " The action also carries a follow-up server command."
            } else {
                ""
            };
            Some(format!(
                "Apply {edit_count} text edit{edit_suffix} across {file_count} file{file_suffix}.{command_suffix}"
            ))
        }
        Err(message) => Some(message),
    }
}

pub(super) fn lsp_code_action_supported_edits(
    code_action: &LspCodeAction,
) -> Result<&[LspDocumentTextEdits], String> {
    if let Some(reason) = code_action.disabled_reason() {
        return Err(format!("Disabled: {reason}"));
    }
    if code_action.has_resource_operations() {
        return Err("Unsupported: resource operations are not supported yet.".to_owned());
    }
    if code_action.document_edits().is_empty() {
        if let Some(command_name) = code_action.command_name() {
            return Err(format!(
                "Unsupported: command-only code action `{command_name}` is not supported yet."
            ));
        }
        return Err("Unsupported: this code action does not include text edits.".to_owned());
    }
    Ok(code_action.document_edits())
}

pub(super) fn apply_lsp_code_action(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    path: &Path,
    code_action: &LspCodeAction,
) -> Result<(), String> {
    let document_edits = lsp_code_action_supported_edits(code_action)?;
    let original_cursor = shell_buffer(runtime, buffer_id)?.cursor_point();
    for document_edit in document_edits {
        apply_lsp_document_edit(
            runtime,
            workspace_id,
            buffer_id,
            path,
            original_cursor,
            document_edit,
            code_action.title(),
        )?;
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(runtime)?;
    Ok(())
}

fn apply_lsp_document_edit(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    active_buffer_id: BufferId,
    active_path: &Path,
    original_cursor: TextPoint,
    document_edit: &LspDocumentTextEdits,
    action_title: &str,
) -> Result<(), String> {
    let target_buffer_id = if document_edit.path() == active_path {
        active_buffer_id
    } else if let Some(existing) =
        find_workspace_file_buffer(runtime, workspace_id, document_edit.path())?
    {
        existing
    } else {
        open_workspace_file(runtime, document_edit.path())?
    };
    let buffer = shell_buffer_mut(runtime, target_buffer_id)?;
    if buffer.path() != Some(document_edit.path()) {
        return Err(format!(
            "buffer `{}` is no longer available for `{action_title}`",
            document_edit.path().display()
        ));
    }
    apply_lsp_text_edits(buffer, document_edit.edits());
    if target_buffer_id == active_buffer_id {
        buffer.set_cursor(original_cursor);
    }
    Ok(())
}

pub(super) fn workspace_search_char_column(line: &str, byte_offset: usize) -> usize {
    let mut byte_offset = byte_offset.min(line.len());
    while byte_offset > 0 && !line.is_char_boundary(byte_offset) {
        byte_offset = byte_offset.saturating_sub(1);
    }
    line[..byte_offset].chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picker_entry(id: &str, label: &str) -> PickerEntry {
        PickerEntry {
            item: PickerItem::new(id, label, label, None::<&str>),
            action: PickerAction::NoOp,
        }
    }

    #[test]
    fn lsp_code_action_kind_sorting_prefers_specific_matches_and_stays_stable() {
        let sorted = lsp_code_action_sorted_indices([
            Some("source.fixAll.eslint"),
            Some("refactor.move"),
            Some("quickfix"),
            Some("source.organizeImports.biome"),
            Some("source"),
            Some("refactor.inline.foo"),
            Some("refactor.rewrite"),
            Some("refactor"),
            Some("refactor.extract"),
            Some("custom"),
            None,
        ]);

        assert_eq!(sorted, vec![2, 1, 7, 5, 8, 6, 4, 3, 0, 9, 10]);
    }

    #[test]
    fn lsp_code_action_picker_overlay_preserves_sorted_entry_order() {
        let overlay = lsp_code_actions_picker_overlay_from_entries(vec![
            picker_entry("z", "zeta"),
            picker_entry("a", "alpha"),
            picker_entry("m", "mu"),
        ]);

        assert_eq!(
            overlay
                .session()
                .matches()
                .iter()
                .map(|matched| matched.item().label())
                .collect::<Vec<_>>(),
            vec!["zeta", "alpha", "mu"]
        );
    }
}
