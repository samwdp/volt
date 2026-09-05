pub(super) fn git_commit_message(buffer: &ShellBuffer) -> String {
    let raw = buffer.text.text();
    let mut lines = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim_start().starts_with('#') {
            continue;
        }
        lines.push(trimmed);
    }
    lines.join("\n").trim().to_owned()
}

pub(super) fn commit_git_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let message = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        git_commit_message(buffer)
    };
    if message.trim().is_empty() {
        return Err("commit message is empty".to_owned());
    }
    let temp_path = git_commit_temp_path();
    fs::write(&temp_path, &message)
        .map_err(|error| format!("failed to write commit message: {error}"))?;
    let result = git_command_output(
        runtime,
        &root,
        "commit",
        &["commit", "-F", &temp_path.to_string_lossy()],
    );
    fs::remove_file(&temp_path).ok();
    result?;
    mark_git_fringe_snapshots_stale(runtime)?;
    invalidate_git_identity_for_active_workspace(runtime);
    close_buffer_discard(runtime, buffer_id)?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(super) fn cancel_git_commit_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    close_buffer_discard(runtime, buffer_id)?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(super) fn stage_git_files(runtime: &mut EditorRuntime, paths: &[String]) -> Result<(), String> {
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

pub(super) fn stage_git_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "add -A", &["add", "-A"])?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(super) fn unstage_git_files(
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

pub(super) fn unstage_git_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "reset", &["reset", "-q"])?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(super) fn git_action_detail(meta: Option<&SectionLineMeta>, action_id: &str) -> Option<String> {
    meta.and_then(|meta| meta.action.as_ref())
        .filter(|action| action.id() == action_id)
        .and_then(|action| action.detail())
        .map(str::to_owned)
}

#[derive(Debug, Clone)]
pub(super) struct GitDeleteTarget {
    path: String,
    untracked: bool,
}

pub(super) fn git_status_delete_target_for_line(
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

pub(super) fn visual_selection_line_range(selection: VisualSelection) -> Option<(usize, usize)> {
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

pub(super) fn git_status_selected_lines(
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

pub(super) fn git_status_action_targets(
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

pub(super) fn git_status_delete_targets(
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

pub(super) fn delete_git_status_targets(
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

pub(super) fn git_line_is_untracked(meta: Option<&SectionLineMeta>) -> bool {
    meta.is_some_and(|meta| meta.section_id == GIT_SECTION_UNTRACKED)
}

pub(super) fn git_args_with_no_pager(command: &str, extra: &[&str]) -> Vec<String> {
    let mut args = Vec::with_capacity(2 + extra.len());
    args.push("--no-pager".to_owned());
    args.push(command.to_owned());
    args.extend(extra.iter().map(|arg| (*arg).to_owned()));
    args
}

pub(super) fn git_view_lines(
    runtime: &mut EditorRuntime,
    view: &GitViewState,
) -> Result<Vec<String>, String> {
    let root = git_root(runtime)?;
    let args = view.args.iter().map(String::as_str).collect::<Vec<_>>();
    let output = git_read_command_output_allow_exit_codes(
        &root,
        &view.label,
        &args,
        &view.allowed_exit_codes,
    )?;
    let mut lines = output
        .lines()
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push(view.empty_message.clone());
    }
    Ok(lines)
}

pub(super) fn git_view_lines_or_error(
    runtime: &mut EditorRuntime,
    view: &GitViewState,
) -> Vec<String> {
    match git_view_lines(runtime, view) {
        Ok(lines) => lines,
        Err(error) => {
            record_runtime_error(runtime, &format!("git.{}", view.label), error.clone());
            vec![format!("Git {} unavailable.", view.label), error]
        }
    }
}

fn git_view_language_id(kind: &str) -> Option<&'static str> {
    match kind {
        GIT_DIFF_KIND => Some("diff"),
        _ => None,
    }
}

pub(super) fn open_git_view_buffer(
    runtime: &mut EditorRuntime,
    kind: &str,
    name: &str,
    view: GitViewState,
) -> Result<(), String> {
    let lines = git_view_lines_or_error(runtime, &view);
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_kind = BufferKind::Plugin(kind.to_owned());
    let language_id = git_view_language_id(kind).map(str::to_owned);
    let existing = find_workspace_named_buffer(runtime, workspace_id, name, &buffer_kind)?;
    if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.focus_buffer_in_active_pane(existing);
        {
            let buffer = shell_buffer_mut(runtime, existing)?;
            buffer.set_git_view(view);
            buffer.replace_with_lines(lines);
            buffer.set_language_id(language_id.clone());
        }
        if language_id.is_some() {
            queue_buffer_syntax_refresh(runtime, existing)?;
        }
        return Ok(());
    }

    let buffer_id = runtime
        .model_mut()
        .create_buffer(workspace_id, name, buffer_kind, None)
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, lines, &*user_library);
    shell_buffer.set_git_view(view);
    shell_buffer.set_language_id(language_id.clone());
    let ui = shell_ui_mut(runtime)?;
    ui.insert_buffer(shell_buffer);
    ui.focus_buffer_in_active_pane(buffer_id);
    if language_id.is_some() {
        queue_buffer_syntax_refresh(runtime, buffer_id)?;
    }
    Ok(())
}

pub(super) fn apply_git_view(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    view: GitViewState,
) -> Result<(), String> {
    let lines = git_view_lines_or_error(runtime, &view);
    let refresh_syntax = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let language_id = match &buffer.kind {
            BufferKind::Plugin(kind) if kind == GIT_DIFF_KIND => Some("diff".to_owned()),
            _ => None,
        };
        buffer.set_git_view(view);
        buffer.replace_with_lines(lines);
        buffer.set_language_id(language_id.clone());
        language_id.is_some()
    };
    if refresh_syntax {
        queue_buffer_syntax_refresh(runtime, buffer_id)?;
    }
    Ok(())
}

pub(super) fn open_git_diff_buffer(
    runtime: &mut EditorRuntime,
    view: GitViewState,
) -> Result<(), String> {
    open_git_view_buffer(runtime, GIT_DIFF_KIND, "*git-diff*", view)
}

pub(super) fn open_git_log_buffer(
    runtime: &mut EditorRuntime,
    view: GitViewState,
) -> Result<(), String> {
    open_git_view_buffer(runtime, GIT_LOG_KIND, "*git-log*", view)
}

pub(super) fn open_git_stash_list_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_args_with_no_pager("stash", &["list"]);
    let view = GitViewState::new("stash", args, "No stashes.", &[0]);
    open_git_view_buffer(runtime, GIT_STASH_KIND, "*git-stash*", view)
}

pub(super) fn open_git_diff_worktree(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_args_with_no_pager("diff", &["--no-color", "HEAD"]);
    let view = GitViewState::new("diff", args, "No working tree changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(super) fn open_git_diff_staged(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_args_with_no_pager("diff", &["--no-color", "--cached"]);
    let view = GitViewState::new("diff", args, "No staged changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(super) fn open_git_diff_unstaged(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_args_with_no_pager("diff", &["--no-color"]);
    let view = GitViewState::new("diff", args, "No unstaged changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(super) fn open_git_diff_staged_file(
    runtime: &mut EditorRuntime,
    path: &str,
) -> Result<(), String> {
    let mut args = git_args_with_no_pager("diff", &["--no-color", "--cached", "--"]);
    args.push(path.to_owned());
    let view = GitViewState::new("diff", args, "No staged changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(super) fn open_git_diff_unstaged_file(
    runtime: &mut EditorRuntime,
    path: &str,
) -> Result<(), String> {
    let mut args = git_args_with_no_pager("diff", &["--no-color", "--"]);
    args.push(path.to_owned());
    let view = GitViewState::new("diff", args, "No unstaged changes.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(super) fn open_git_diff_untracked_file(
    runtime: &mut EditorRuntime,
    path: &str,
) -> Result<(), String> {
    let mut args = git_args_with_no_pager("diff", &["--no-color", "--no-index", "--", "/dev/null"]);
    args.push(path.to_owned());
    let view = GitViewState::new("diff", args, "No untracked diff.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}

pub(super) fn open_git_diff_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let args = git_args_with_no_pager("show", &["--no-color", commit]);
    let view = GitViewState::new("show", args, "No commit diff.", &[0]);
    open_git_diff_buffer(runtime, view)
}

pub(super) fn open_git_diff_stash(runtime: &mut EditorRuntime, stash: &str) -> Result<(), String> {
    let args = git_args_with_no_pager("stash", &["show", "--no-color", "-p", stash]);
    let view = GitViewState::new("stash", args, "No stash diff.", &[0]);
    open_git_diff_buffer(runtime, view)
}

pub(super) fn diff_git_dwim(
    runtime: &mut EditorRuntime,
    _buffer_id: BufferId,
    meta: Option<&SectionLineMeta>,
    _line_text: &str,
) -> Result<(), String> {
    if let Some(commit) = git_action_detail(meta, GIT_ACTION_SHOW_COMMIT) {
        return open_git_diff_commit(runtime, &commit);
    }
    if let Some(stash) = git_action_detail(meta, GIT_ACTION_SHOW_STASH) {
        return open_git_diff_stash(runtime, &stash);
    }
    if let Some(path) = git_action_detail(meta, GIT_ACTION_UNSTAGE_FILE) {
        return open_git_diff_staged_file(runtime, &path);
    }
    if let Some(path) = git_action_detail(meta, GIT_ACTION_STAGE_FILE) {
        if git_line_is_untracked(meta) {
            return open_git_diff_untracked_file(runtime, &path);
        }
        return open_git_diff_unstaged_file(runtime, &path);
    }
    if let Some(meta) = meta
        && let SectionRenderLineKind::Header { id, .. } = &meta.kind
    {
        if id == GIT_SECTION_STAGED {
            return open_git_diff_staged(runtime);
        }
        if id == GIT_SECTION_UNSTAGED || id == GIT_SECTION_UNTRACKED {
            return open_git_diff_unstaged(runtime);
        }
    }
    open_git_diff_worktree(runtime)
}

pub(super) fn diff_git_commit_at_point(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_action_detail(meta, GIT_ACTION_SHOW_COMMIT) {
        return open_git_diff_commit(runtime, &commit);
    }
    let commit = git_snapshot_for_buffer(runtime, buffer_id)
        .ok()
        .and_then(|snapshot| snapshot.head().map(|head| head.hash().to_owned()))
        .unwrap_or_else(|| "HEAD".to_owned());
    open_git_diff_commit(runtime, &commit)
}

pub(super) fn diff_git_stash_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    open_git_diff_stash(runtime, &stash)
}

pub(super) fn git_log_args(extra: &[String]) -> Vec<String> {
    let mut args = vec![
        "--no-pager".to_owned(),
        "log".to_owned(),
        "--no-color".to_owned(),
        "--oneline".to_owned(),
        "--decorate".to_owned(),
        "--graph".to_owned(),
        "-n".to_owned(),
        GIT_LOG_VIEW_LIMIT.to_string(),
    ];
    args.extend(extra.iter().cloned());
    args
}

pub(super) fn open_git_log_current(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&[]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(super) fn open_git_log_head(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&["HEAD".to_owned()]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(super) fn open_git_log_related(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let mut refs = Vec::new();
    if let Some(branch) = snapshot.branch() {
        refs.push(branch.to_owned());
    } else {
        refs.push("HEAD".to_owned());
    }
    if let Some(upstream) = snapshot.upstream() {
        refs.push(upstream.to_owned());
    }
    if let Some(push_remote) = snapshot.push_remote() {
        refs.push(push_remote.to_owned());
    }
    refs.sort();
    refs.dedup();
    let args = git_log_args(&refs);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(super) fn open_git_log_branches(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&["--branches".to_owned()]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(super) fn open_git_log_all_branches(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&["--branches".to_owned(), "--remotes".to_owned()]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(super) fn open_git_log_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_log_args(&["--all".to_owned()]);
    let view = GitViewState::new("log", args, "No commits to show.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(super) fn stash_git_both(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "stash push", &["stash", "push"])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn stash_git_index(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "stash push --staged",
        &["stash", "push", "--staged"],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn stash_git_worktree(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "stash push --keep-index",
        &["stash", "push", "--keep-index"],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn stash_git_keep_index(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "stash push --keep-index",
        &["stash", "push", "--keep-index"],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn stash_git_apply_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "stash apply", &["stash", "apply", &stash])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn stash_git_pop_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "stash pop", &["stash", "pop", &stash])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn stash_git_drop_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "stash drop", &["stash", "drop", &stash])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn stash_git_show_at_point(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    let stash = git_action_detail(meta, GIT_ACTION_SHOW_STASH)
        .ok_or_else(|| "no stash selected".to_owned())?;
    open_git_diff_stash(runtime, &stash)
}

pub(super) fn git_merge_in_progress(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let root = git_root(runtime)?;
    let Some(git_dir) = git_dir_path(runtime, &root) else {
        return Ok(false);
    };
    Ok(git_dir.join("MERGE_HEAD").is_file())
}

pub(super) fn git_rebase_in_progress(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let root = git_root(runtime)?;
    let Some(git_dir) = git_dir_path(runtime, &root) else {
        return Ok(false);
    };
    Ok(git_dir.join("rebase-apply").is_dir() || git_dir.join("rebase-merge").is_dir())
}

pub(super) fn git_sequence_in_progress(
    runtime: &mut EditorRuntime,
) -> Result<Option<GitSequenceKind>, String> {
    let root = git_root(runtime)?;
    let Some(git_dir) = git_dir_path(runtime, &root) else {
        return Ok(None);
    };
    if git_dir.join("CHERRY_PICK_HEAD").is_file() {
        return Ok(Some(GitSequenceKind::CherryPick));
    }
    if git_dir.join("REVERT_HEAD").is_file() {
        return Ok(Some(GitSequenceKind::Revert));
    }
    Ok(None)
}

pub(super) fn sequence_git_continue(
    runtime: &mut EditorRuntime,
    kind: GitSequenceKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let (title, args) = match kind {
        GitSequenceKind::CherryPick => (
            "Git Cherry-pick",
            vec!["cherry-pick".to_owned(), "--continue".to_owned()],
        ),
        GitSequenceKind::Revert => (
            "Git Revert",
            vec!["revert".to_owned(), "--continue".to_owned()],
        ),
    };
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            title,
            args,
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(super) fn sequence_git_skip(
    runtime: &mut EditorRuntime,
    kind: GitSequenceKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let (title, args) = match kind {
        GitSequenceKind::CherryPick => (
            "Git Cherry-pick",
            vec!["cherry-pick".to_owned(), "--skip".to_owned()],
        ),
        GitSequenceKind::Revert => ("Git Revert", vec!["revert".to_owned(), "--skip".to_owned()]),
    };
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            title,
            args,
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(super) fn sequence_git_abort(
    runtime: &mut EditorRuntime,
    kind: GitSequenceKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let (title, args) = match kind {
        GitSequenceKind::CherryPick => (
            "Git Cherry-pick",
            vec!["cherry-pick".to_owned(), "--abort".to_owned()],
        ),
        GitSequenceKind::Revert => (
            "Git Revert",
            vec!["revert".to_owned(), "--abort".to_owned()],
        ),
    };
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            title,
            args,
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(super) fn git_commit_at_point(meta: Option<&SectionLineMeta>) -> Option<String> {
    git_action_detail(meta, GIT_ACTION_SHOW_COMMIT)
}

pub(super) fn cherry_pick_git_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Cherry-pick",
            vec!["cherry-pick".to_owned(), commit.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(super) fn cherry_pick_git_commit_no_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Cherry-pick",
            vec![
                "cherry-pick".to_owned(),
                "--no-commit".to_owned(),
                commit.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(super) fn revert_git_commit(runtime: &mut EditorRuntime, commit: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Revert",
            vec!["revert".to_owned(), commit.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(super) fn revert_git_commit_no_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Revert",
            vec![
                "revert".to_owned(),
                "--no-commit".to_owned(),
                commit.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(super) fn cherry_pick_commit_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return cherry_pick_git_commit(runtime, &commit);
    }
    open_git_commit_picker_with_action(runtime, "Git Cherry-Pick", GitCommitActionKind::CherryPick)
}

pub(super) fn cherry_pick_apply_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return cherry_pick_git_commit_no_commit(runtime, &commit);
    }
    open_git_commit_picker_with_action(
        runtime,
        "Git Cherry-Pick (Apply)",
        GitCommitActionKind::CherryPickNoCommit,
    )
}

pub(super) fn revert_commit_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return revert_git_commit(runtime, &commit);
    }
    open_git_commit_picker_with_action(runtime, "Git Revert", GitCommitActionKind::Revert)
}

pub(super) fn revert_no_commit_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return revert_git_commit_no_commit(runtime, &commit);
    }
    open_git_commit_picker_with_action(
        runtime,
        "Git Revert (No Commit)",
        GitCommitActionKind::RevertNoCommit,
    )
}

pub(super) fn reset_git_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
    mode: GitResetMode,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let (label, args) = match mode {
        GitResetMode::Mixed => ("reset --mixed", vec!["reset", "--mixed", commit]),
        GitResetMode::Soft => ("reset --soft", vec!["reset", "--soft", commit]),
        GitResetMode::Hard => ("reset --hard", vec!["reset", "--hard", commit]),
        GitResetMode::Keep => ("reset --keep", vec!["reset", "--keep", commit]),
    };
    git_command_output(runtime, &root, label, &args)?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn reset_commit_at_point_or_picker(
    runtime: &mut EditorRuntime,
    meta: Option<&SectionLineMeta>,
    mode: GitResetMode,
) -> Result<(), String> {
    if let Some(commit) = git_commit_at_point(meta) {
        return reset_git_commit(runtime, &commit, mode);
    }
    let (title, action) = match mode {
        GitResetMode::Mixed => ("Git Reset (Mixed)", GitCommitActionKind::ResetMixed),
        GitResetMode::Soft => ("Git Reset (Soft)", GitCommitActionKind::ResetSoft),
        GitResetMode::Hard => ("Git Reset (Hard)", GitCommitActionKind::ResetHard),
        GitResetMode::Keep => ("Git Reset (Keep)", GitCommitActionKind::ResetKeep),
    };
    open_git_commit_picker_with_action(runtime, title, action)
}

pub(super) fn merge_git_plain(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec!["merge".to_owned(), branch.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(super) fn merge_git_edit(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec![
                "merge".to_owned(),
                "--no-commit".to_owned(),
                "--edit".to_owned(),
                branch.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusCloseAndOpenCommitBuffer,
        )
        .with_git_editor(true),
    )?;
    Ok(())
}

pub(super) fn merge_git_no_commit(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec![
                "merge".to_owned(),
                "--no-commit".to_owned(),
                branch.to_owned(),
            ],
            root,
            StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
        ),
    )?;
    Ok(())
}

pub(super) fn merge_git_squash(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Merge",
            vec!["merge".to_owned(), "--squash".to_owned(), branch.to_owned()],
            root,
            StreamedCommandExitAction::RefreshGitStatusCloseAndOpenCommitBuffer,
        ),
    )?;
    Ok(())
}

pub(super) fn merge_git_preview(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let args = vec![
        "--no-pager".to_owned(),
        "diff".to_owned(),
        "--no-color".to_owned(),
        format!("HEAD...{branch}"),
    ];
    let view = GitViewState::new("diff", args, "No changes to merge.", &[0, 1]);
    open_git_diff_buffer(runtime, view)
}
