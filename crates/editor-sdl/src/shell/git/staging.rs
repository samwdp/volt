use super::super::*;

use super::process::*;
use super::status::*;

pub(crate) fn stage_git_files(runtime: &mut EditorRuntime, paths: &[String]) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let root = git_root(runtime)?;
    let mut args = vec!["add".to_owned(), "--".to_owned()];
    args.extend(paths.iter().cloned());
    git_command_output_owned(runtime, &root, "add", &args)?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(crate) fn stage_git_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "add -A", &["add", "-A"])?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(crate) fn unstage_git_files(
    runtime: &mut EditorRuntime,
    paths: &[String],
) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let root = git_root(runtime)?;
    let mut args = vec!["reset".to_owned(), "-q".to_owned(), "--".to_owned()];
    args.extend(paths.iter().cloned());
    git_command_output_owned(runtime, &root, "reset --", &args)?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(crate) fn unstage_git_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "reset", &["reset", "-q"])?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(crate) fn git_action_detail(meta: Option<&SectionLineMeta>, action_id: &str) -> Option<String> {
    meta.and_then(|meta| meta.action.as_ref())
        .filter(|action| action.id() == action_id)
        .and_then(|action| action.detail())
        .map(str::to_owned)
}

#[derive(Debug, Clone)]
pub(crate) struct GitDeleteTarget {
    pub(crate) path: String,
    pub(crate) untracked: bool,
}

pub(crate) fn git_status_delete_target_for_line(
    buffer: &ShellBuffer,
    line_index: usize,
) -> Option<GitDeleteTarget> {
    let meta = buffer.section_line_meta(line_index)?;
    let action = meta.action.as_ref()?;
    let path = action.detail()?;
    if action.id() == GIT_ACTION_UNSTAGE_FILE {
        return Some(GitDeleteTarget {
            path: path.to_owned(),
            untracked: false,
        });
    }
    if action.id() == GIT_ACTION_STAGE_FILE {
        return Some(GitDeleteTarget {
            path: path.to_owned(),
            untracked: git_line_is_untracked(Some(meta)),
        });
    }
    None
}

pub(crate) fn visual_selection_line_range(selection: VisualSelection) -> Option<(usize, usize)> {
    match selection {
        VisualSelection::Range(range) => {
            let start_line = range.start().line;
            let mut end_line = range.end().line;
            if range.end().column == 0 && end_line > start_line {
                end_line = end_line.saturating_sub(1);
            }
            Some((start_line, end_line.max(start_line)))
        }
        VisualSelection::Block(selection) => Some((selection.start_line, selection.end_line)),
    }
}

pub(crate) fn git_status_selected_lines(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<(Vec<usize>, bool), String> {
    let ui = shell_ui(runtime)?;
    let is_visual = matches!(ui.input_mode(), InputMode::Visual);
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !is_visual {
        return Ok((vec![buffer.cursor_point().line], false));
    }

    let anchor = ui
        .vim()
        .visual_anchor
        .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
    let selection = visual_selection(buffer, anchor, ui.vim().visual_kind)
        .ok_or_else(|| "visual selection is empty".to_owned())?;
    let (start_line, end_line) = visual_selection_line_range(selection).unwrap_or((0, 0));
    let end_line = end_line.min(buffer.line_count().saturating_sub(1));
    Ok(((start_line..=end_line).collect(), true))
}

pub(crate) fn git_status_action_targets(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    action_id: &str,
) -> Result<(Vec<String>, bool), String> {
    let (selected_lines, is_visual) = git_status_selected_lines(runtime, buffer_id)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    let mut targets = BTreeSet::new();
    for line_index in selected_lines {
        if let Some(path) = git_action_detail(buffer.section_line_meta(line_index), action_id) {
            targets.insert(path);
        }
    }
    Ok((targets.into_iter().collect(), is_visual))
}

pub(crate) fn git_status_delete_targets(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<(Vec<GitDeleteTarget>, bool), String> {
    let (selected_lines, is_visual) = git_status_selected_lines(runtime, buffer_id)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    let mut targets = BTreeMap::new();
    for line_index in selected_lines {
        if let Some(target) = git_status_delete_target_for_line(buffer, line_index) {
            targets.entry(target.path.clone()).or_insert(target);
        }
    }
    Ok((targets.into_values().collect(), is_visual))
}

pub(crate) fn delete_git_status_targets(
    runtime: &mut EditorRuntime,
    targets: &[GitDeleteTarget],
) -> Result<(), String> {
    if targets.is_empty() {
        return Ok(());
    }
    let root = git_root(runtime)?;
    for target in targets {
        if target.untracked {
            let path = root.join(&target.path);
            let metadata = fs::metadata(&path)
                .map_err(|error| format!("failed to stat `{}`: {error}", path.display()))?;
            if metadata.is_dir() {
                fs::remove_dir_all(&path)
                    .map_err(|error| format!("failed to remove `{}`: {error}", path.display()))?;
            } else {
                fs::remove_file(&path)
                    .map_err(|error| format!("failed to remove `{}`: {error}", path.display()))?;
            }
        } else {
            let args = ["rm", "-f", "--ignore-unmatch", "--", target.path.as_str()];
            git_command_output(runtime, &root, "rm -f", &args)?;
        }
    }
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(crate) fn git_line_is_untracked(meta: Option<&SectionLineMeta>) -> bool {
    meta.is_some_and(|meta| meta.section_id == GIT_SECTION_UNTRACKED)
}
