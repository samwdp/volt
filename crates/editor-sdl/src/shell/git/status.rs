use super::super::*;

use super::diff::*;
use super::fringe::*;
use super::merge_rebase::*;
use super::process::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ActiveBufferEventContext {
    pub(crate) buffer_id: BufferId,
    pub(crate) has_input: bool,
    pub(crate) vim_targets_input: bool,
    pub(crate) is_read_only: bool,
    pub(crate) is_git_status: bool,
    pub(crate) is_git_commit: bool,
    pub(crate) is_git_editor: bool,
    pub(crate) is_acp: bool,
    pub(crate) is_directory: bool,
    pub(crate) is_browser: bool,
    pub(crate) is_terminal: bool,
    pub(crate) is_db_query: bool,
    pub(crate) is_plugin_evaluatable: bool,
}

pub(crate) fn default_vim_target(has_input: bool) -> VimTarget {
    if has_input {
        VimTarget::Input
    } else {
        VimTarget::Buffer
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveLspBufferContext {
    pub(crate) workspace_id: WorkspaceId,
    pub(crate) buffer_id: BufferId,
    pub(crate) path: PathBuf,
    pub(crate) text: String,
    pub(crate) revision: u64,
    pub(crate) root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct GitViewState {
    pub(crate) label: String,
    pub(crate) args: Vec<String>,
    pub(crate) empty_message: String,
    pub(crate) allowed_exit_codes: Vec<i32>,
}

impl GitViewState {
    pub(crate) fn new(
        label: impl Into<String>,
        args: Vec<String>,
        empty_message: impl Into<String>,
        allowed_exit_codes: &[i32],
    ) -> Self {
        Self {
            label: label.into(),
            args,
            empty_message: empty_message.into(),
            allowed_exit_codes: allowed_exit_codes.to_vec(),
        }
    }
}

pub(crate) fn format_section_line(line: &SectionRenderLine) -> String {
    let indent = "  ".repeat(line.depth);
    match &line.kind {
        SectionRenderLineKind::Header { .. } => format!("{indent}{}", line.text),
        SectionRenderLineKind::Item => format!("{indent}{}", line.text),
        SectionRenderLineKind::Spacer => String::new(),
    }
}

pub(crate) fn git_status_line_spans(
    line: &SectionRenderLine,
    formatted_line: &str,
) -> Vec<LineSyntaxSpan> {
    let mut spans = Vec::new();
    let indent_bytes = leading_indent_bytes(formatted_line);
    let trimmed = &formatted_line[indent_bytes..];
    if trimmed.is_empty() {
        return spans;
    }
    match &line.kind {
        SectionRenderLineKind::Header { .. } => {
            push_span_bytes(
                &mut spans,
                formatted_line,
                indent_bytes,
                indent_bytes + trimmed.len(),
                TOKEN_GIT_STATUS_SECTION_HEADER,
            );
            if let Some((start, end)) = find_paren_number_range(trimmed) {
                push_span_bytes(
                    &mut spans,
                    formatted_line,
                    indent_bytes + start,
                    indent_bytes + end,
                    TOKEN_GIT_STATUS_SECTION_COUNT,
                );
            }
        }
        SectionRenderLineKind::Item => match line.section_id.as_str() {
            GIT_SECTION_HEADERS => {
                git_status_header_item_spans(formatted_line, indent_bytes, trimmed, &mut spans);
            }
            GIT_SECTION_IN_PROGRESS => {
                push_span_bytes(
                    &mut spans,
                    formatted_line,
                    indent_bytes,
                    indent_bytes + trimmed.len(),
                    TOKEN_GIT_STATUS_IN_PROGRESS,
                );
            }
            GIT_SECTION_STAGED | GIT_SECTION_UNSTAGED | GIT_SECTION_UNTRACKED => {
                git_status_entry_item_spans(formatted_line, indent_bytes, trimmed, &mut spans);
            }
            GIT_SECTION_STASHES => {
                git_status_stash_item_spans(formatted_line, indent_bytes, trimmed, &mut spans);
            }
            GIT_SECTION_UNPULLED | GIT_SECTION_UNPUSHED => {
                git_status_commit_item_spans(formatted_line, indent_bytes, trimmed, &mut spans);
            }
            GIT_SECTION_COMMIT => {
                git_status_commit_message_spans(formatted_line, indent_bytes, trimmed, &mut spans);
            }
            _ => {}
        },
        SectionRenderLineKind::Spacer => {}
    }
    spans
}

pub(crate) fn git_status_header_item_spans(
    line: &str,
    indent_bytes: usize,
    trimmed: &str,
    spans: &mut Vec<LineSyntaxSpan>,
) {
    let (icon_bounds, content_start, content) =
        split_icon_prefixed_content(trimmed).unwrap_or((None, 0, trimmed));
    let Some(colon_index) = content.find(':') else {
        return;
    };
    let label_end = colon_index + 1;
    if let Some((icon_start, icon_end)) = icon_bounds {
        push_span_bytes(
            spans,
            line,
            indent_bytes + icon_start,
            indent_bytes + icon_end,
            TOKEN_GIT_STATUS_HEADER_LABEL,
        );
    }
    push_span_bytes(
        spans,
        line,
        indent_bytes + content_start,
        indent_bytes + content_start + label_end,
        TOKEN_GIT_STATUS_HEADER_LABEL,
    );
    let rest_start = skip_whitespace_bytes(content, label_end);
    if rest_start >= content.len() {
        return;
    }
    let label = content[..colon_index].trim();
    let rest = &content[rest_start..];
    let rest_offset = indent_bytes + content_start + rest_start;
    match label {
        "Head" => git_status_head_spans(line, rest, rest_offset, spans),
        "Merge" => git_status_upstream_spans(line, rest, rest_offset, spans),
        _ => {
            push_span_bytes(
                spans,
                line,
                rest_offset,
                rest_offset + rest.len(),
                TOKEN_GIT_STATUS_HEADER_VALUE,
            );
        }
    }
}

pub(crate) fn git_status_head_spans(
    line: &str,
    rest: &str,
    rest_offset: usize,
    spans: &mut Vec<LineSyntaxSpan>,
) {
    let Some((first_start, first_end)) = next_word_bounds(rest, 0) else {
        return;
    };
    let first = &rest[first_start..first_end];
    let summary_start = if is_git_hash(first) {
        push_span_bytes(
            spans,
            line,
            rest_offset + first_start,
            rest_offset + first_end,
            TOKEN_GIT_STATUS_HEADER_HASH,
        );
        Some(first_end)
    } else {
        push_span_bytes(
            spans,
            line,
            rest_offset + first_start,
            rest_offset + first_end,
            TOKEN_GIT_STATUS_HEADER_VALUE,
        );
        let Some((second_start, second_end)) = next_word_bounds(rest, first_end) else {
            return;
        };
        let second = &rest[second_start..second_end];
        if is_git_hash(second) {
            push_span_bytes(
                spans,
                line,
                rest_offset + second_start,
                rest_offset + second_end,
                TOKEN_GIT_STATUS_HEADER_HASH,
            );
            Some(second_end)
        } else {
            Some(second_start)
        }
    };
    if let Some(summary_start) = summary_start {
        let summary_start = skip_whitespace_bytes(rest, summary_start);
        if summary_start < rest.len() {
            push_span_bytes(
                spans,
                line,
                rest_offset + summary_start,
                rest_offset + rest.len(),
                TOKEN_GIT_STATUS_HEADER_SUMMARY,
            );
        }
    }
}

pub(crate) fn git_status_upstream_spans(
    line: &str,
    rest: &str,
    rest_offset: usize,
    spans: &mut Vec<LineSyntaxSpan>,
) {
    let value_end = rest.find('(').unwrap_or(rest.len());
    let value_end = rest[..value_end].trim_end().len();
    if value_end > 0 {
        push_span_bytes(
            spans,
            line,
            rest_offset,
            rest_offset + value_end,
            TOKEN_GIT_STATUS_HEADER_VALUE,
        );
    }
    push_number_after_keyword(
        spans,
        line,
        rest_offset,
        rest,
        "ahead",
        TOKEN_GIT_STATUS_SECTION_COUNT,
    );
    push_number_after_keyword(
        spans,
        line,
        rest_offset,
        rest,
        "behind",
        TOKEN_GIT_STATUS_SECTION_COUNT,
    );
}

pub(crate) fn git_head_tag(root: &Path) -> Option<String> {
    git_read_command_output_optional(
        root,
        "describe --tags --abbrev=0",
        &["describe", "--tags", "--abbrev=0"],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
}

pub(crate) fn git_status_entry_item_spans(
    line: &str,
    indent_bytes: usize,
    trimmed: &str,
    spans: &mut Vec<LineSyntaxSpan>,
) {
    let (icon_bounds, content_start, content) =
        split_icon_prefixed_content(trimmed).unwrap_or((None, 0, trimmed));
    if let Some((icon_start, icon_end)) = icon_bounds {
        let icon = &trimmed[icon_start..icon_end];
        let token = git_status_entry_token_from_icon(icon);
        push_span_bytes(
            spans,
            line,
            indent_bytes + icon_start,
            indent_bytes + icon_end,
            token,
        );
        push_span_bytes(
            spans,
            line,
            indent_bytes + content_start,
            indent_bytes + content_start + content.len(),
            TOKEN_GIT_STATUS_ENTRY_PATH,
        );
        return;
    }
    let Some((status_start, status_end)) = next_word_bounds(content, 0) else {
        return;
    };
    let status = &content[status_start..status_end];
    let token = git_status_entry_token(status);
    push_span_bytes(
        spans,
        line,
        indent_bytes + content_start + status_start,
        indent_bytes + content_start + status_end,
        token,
    );
    let path_start = skip_whitespace_bytes(content, status_end);
    if path_start < content.len() {
        push_span_bytes(
            spans,
            line,
            indent_bytes + content_start + path_start,
            indent_bytes + content_start + content.len(),
            TOKEN_GIT_STATUS_ENTRY_PATH,
        );
    }
}

pub(crate) fn git_status_entry_token(label: &str) -> &'static str {
    match label {
        "added" => TOKEN_GIT_STATUS_ENTRY_ADDED,
        "modified" => TOKEN_GIT_STATUS_ENTRY_MODIFIED,
        "deleted" => TOKEN_GIT_STATUS_ENTRY_DELETED,
        "renamed" => TOKEN_GIT_STATUS_ENTRY_RENAMED,
        "copied" => TOKEN_GIT_STATUS_ENTRY_COPIED,
        "updated" => TOKEN_GIT_STATUS_ENTRY_UPDATED,
        "untracked" => TOKEN_GIT_STATUS_ENTRY_UNTRACKED,
        "changed" => TOKEN_GIT_STATUS_ENTRY_CHANGED,
        _ => TOKEN_GIT_STATUS_ENTRY_CHANGED,
    }
}

pub(crate) fn git_status_entry_token_from_icon(icon: &str) -> &'static str {
    match icon {
        editor_icons::symbols::cod::COD_DIFF_ADDED => TOKEN_GIT_STATUS_ENTRY_ADDED,
        editor_icons::symbols::cod::COD_DIFF_MODIFIED => TOKEN_GIT_STATUS_ENTRY_MODIFIED,
        editor_icons::symbols::cod::COD_DIFF_REMOVED => TOKEN_GIT_STATUS_ENTRY_DELETED,
        editor_icons::symbols::cod::COD_DIFF_RENAMED => TOKEN_GIT_STATUS_ENTRY_RENAMED,
        editor_icons::symbols::cod::COD_ARROW_SWAP => TOKEN_GIT_STATUS_ENTRY_COPIED,
        editor_icons::symbols::cod::COD_SYNC => TOKEN_GIT_STATUS_ENTRY_UPDATED,
        editor_icons::symbols::cod::COD_SYMBOL_FILE => TOKEN_GIT_STATUS_ENTRY_UNTRACKED,
        _ => TOKEN_GIT_STATUS_ENTRY_CHANGED,
    }
}

pub(crate) fn git_status_stash_item_spans(
    line: &str,
    indent_bytes: usize,
    trimmed: &str,
    spans: &mut Vec<LineSyntaxSpan>,
) {
    let (icon_bounds, content_start, content) =
        split_icon_prefixed_content(trimmed).unwrap_or((None, 0, trimmed));
    let Some((name_start, name_end)) = next_word_bounds(content, 0) else {
        return;
    };
    if let Some((icon_start, icon_end)) = icon_bounds {
        push_span_bytes(
            spans,
            line,
            indent_bytes + icon_start,
            indent_bytes + icon_end,
            TOKEN_GIT_STATUS_STASH_NAME,
        );
    }
    push_span_bytes(
        spans,
        line,
        indent_bytes + content_start + name_start,
        indent_bytes + content_start + name_end,
        TOKEN_GIT_STATUS_STASH_NAME,
    );
    let summary_start = skip_whitespace_bytes(content, name_end);
    if summary_start < content.len() {
        push_span_bytes(
            spans,
            line,
            indent_bytes + content_start + summary_start,
            indent_bytes + content_start + content.len(),
            TOKEN_GIT_STATUS_STASH_SUMMARY,
        );
    }
}

pub(crate) fn git_status_commit_item_spans(
    line: &str,
    indent_bytes: usize,
    trimmed: &str,
    spans: &mut Vec<LineSyntaxSpan>,
) {
    let (icon_bounds, content_start, content) =
        split_icon_prefixed_content(trimmed).unwrap_or((None, 0, trimmed));
    let Some((hash_start, hash_end)) = next_word_bounds(content, 0) else {
        return;
    };
    if let Some((icon_start, icon_end)) = icon_bounds {
        push_span_bytes(
            spans,
            line,
            indent_bytes + icon_start,
            indent_bytes + icon_end,
            TOKEN_GIT_STATUS_COMMIT_HASH,
        );
    }
    push_span_bytes(
        spans,
        line,
        indent_bytes + content_start + hash_start,
        indent_bytes + content_start + hash_end,
        TOKEN_GIT_STATUS_COMMIT_HASH,
    );
    let summary_start = skip_whitespace_bytes(content, hash_end);
    if summary_start < content.len() {
        push_span_bytes(
            spans,
            line,
            indent_bytes + content_start + summary_start,
            indent_bytes + content_start + content.len(),
            TOKEN_GIT_STATUS_COMMIT_SUMMARY,
        );
    }
}

pub(crate) fn git_status_commit_message_spans(
    line: &str,
    indent_bytes: usize,
    trimmed: &str,
    spans: &mut Vec<LineSyntaxSpan>,
) {
    let (_, _, content) = split_icon_prefixed_content(trimmed).unwrap_or((None, 0, trimmed));
    let token = if content.starts_with("Press ") {
        TOKEN_GIT_STATUS_COMMAND
    } else {
        TOKEN_GIT_STATUS_MESSAGE
    };
    push_span_bytes(
        spans,
        line,
        indent_bytes,
        indent_bytes + trimmed.len(),
        token,
    );
}

pub(crate) type IconPrefixedContent<'a> = (Option<(usize, usize)>, usize, &'a str);

pub(crate) fn split_icon_prefixed_content(text: &str) -> Option<IconPrefixedContent<'_>> {
    let (icon_start, icon_end) = next_word_bounds(text, 0)?;
    let content_start = skip_whitespace_bytes(text, icon_end);
    if content_start >= text.len() {
        return Some((Some((icon_start, icon_end)), text.len(), ""));
    }
    Some((
        Some((icon_start, icon_end)),
        content_start,
        &text[content_start..],
    ))
}

pub(crate) fn leading_indent_bytes(line: &str) -> usize {
    line.char_indices()
        .find(|(_, character)| *character != ' ')
        .map(|(index, _)| index)
        .unwrap_or_else(|| line.len())
}

pub(crate) fn push_span_bytes(
    spans: &mut Vec<LineSyntaxSpan>,
    line: &str,
    start_byte: usize,
    end_byte: usize,
    token: &str,
) {
    if start_byte >= end_byte {
        return;
    }
    let start = clamp_to_char_boundary(line, start_byte);
    let end = clamp_to_char_boundary(line, end_byte.min(line.len()));
    if start >= end {
        return;
    }
    let start_col = line[..start].chars().count();
    let end_col = line[..end].chars().count();
    if start_col < end_col {
        spans.push(LineSyntaxSpan {
            start: start_col,
            end: end_col,
            capture_name: Arc::from(token),
            theme_token: Arc::from(token),
        });
    }
}

pub(crate) fn next_word_bounds(text: &str, start: usize) -> Option<(usize, usize)> {
    let start = skip_whitespace_bytes(text, start);
    if start >= text.len() {
        return None;
    }
    let mut end = text.len();
    for (offset, character) in text[start..].char_indices() {
        if character.is_whitespace() {
            end = start + offset;
            break;
        }
    }
    Some((start, end))
}

pub(crate) fn skip_whitespace_bytes(text: &str, start: usize) -> usize {
    let start = start.min(text.len());
    for (offset, character) in text[start..].char_indices() {
        if !character.is_whitespace() {
            return start + offset;
        }
    }
    text.len()
}

pub(crate) fn is_git_hash(text: &str) -> bool {
    (7..=40).contains(&text.len()) && text.chars().all(|character| character.is_ascii_hexdigit())
}

pub(crate) fn find_paren_number_range(text: &str) -> Option<(usize, usize)> {
    let open = text.rfind('(')?;
    let close = text[open..].find(')')? + open;
    let inner = &text[open + 1..close];
    let digit_offset = inner.find(|character: char| character.is_ascii_digit())?;
    let digit_start = open + 1 + digit_offset;
    let digit_end = inner[digit_offset..]
        .find(|character: char| !character.is_ascii_digit())
        .map(|index| digit_start + index)
        .unwrap_or(close);
    (digit_start < digit_end).then_some((digit_start, digit_end))
}

pub(crate) fn push_number_after_keyword(
    spans: &mut Vec<LineSyntaxSpan>,
    line: &str,
    base_offset: usize,
    text: &str,
    keyword: &str,
    token: &str,
) {
    let Some(keyword_index) = text.find(keyword) else {
        return;
    };
    let number_start = skip_whitespace_bytes(text, keyword_index + keyword.len());
    if number_start >= text.len() {
        return;
    }
    let number_end = text[number_start..]
        .find(|character: char| !character.is_ascii_digit())
        .map(|index| number_start + index)
        .unwrap_or(text.len());
    if number_start < number_end {
        push_span_bytes(
            spans,
            line,
            base_offset + number_start,
            base_offset + number_end,
            token,
        );
    }
}

pub(crate) type GitPrefix = editor_plugin_api::GitStatusPrefix;

#[derive(Debug, Clone)]
pub(crate) struct GitPrefixState {
    pub(crate) prefix: GitPrefix,
    pub(crate) started_at: Instant,
}

impl GitPrefixState {
    pub(crate) fn prefix(&self) -> GitPrefix {
        self.prefix
    }

    pub(crate) fn expires_at(&self) -> Instant {
        self.started_at + Duration::from_millis(1200)
    }
}

pub(crate) fn refresh_git_status_if_active(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    if !buffer_is_git_status(&shell_buffer(runtime, buffer_id)?.kind) {
        return Ok(());
    }
    refresh_git_status_buffer(runtime, buffer_id)
}

pub(crate) fn refresh_git_status_if_active_if_due(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    if !buffer_is_git_status(&shell_buffer(runtime, buffer_id)?.kind) {
        return Ok(());
    }
    let root = match git_root(runtime) {
        Ok(root) => root,
        Err(error) => {
            set_git_status_error(runtime, buffer_id, &error)?;
            return Err(error);
        }
    };
    let now = Instant::now();
    if !shell_buffer(runtime, buffer_id)?.git_status_refresh_due(&root, now) {
        return Ok(());
    }
    refresh_git_status_buffer_for_root(runtime, buffer_id, &root, now)
}

/// Marks fringe/summary stale after a disk mutation without blocking on `git status`.
pub(crate) fn invalidate_git_state_after_save(runtime: &mut EditorRuntime) -> Result<(), String> {
    mark_git_fringe_snapshots_stale(runtime)?;
    invalidate_git_identity_for_active_workspace(runtime);
    Ok(())
}

pub(crate) fn refresh_git_status_buffers(runtime: &mut EditorRuntime) -> Result<(), String> {
    invalidate_git_state_after_save(runtime)?;
    let buffer_ids = {
        let ui = shell_ui(runtime)?;
        ui.buffers
            .iter()
            .filter(|buffer| buffer_is_git_status(&buffer.kind))
            .map(ShellBuffer::id)
            .collect::<Vec<_>>()
    };
    for buffer_id in buffer_ids {
        let _ = refresh_git_status_buffer(runtime, buffer_id);
    }
    Ok(())
}

pub(crate) fn refresh_git_status_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = match git_root(runtime) {
        Ok(root) => root,
        Err(error) => {
            set_git_status_error(runtime, buffer_id, &error)?;
            return Err(error);
        }
    };
    refresh_git_status_buffer_for_root(runtime, buffer_id, &root, Instant::now())
}

pub(crate) fn refresh_git_status_buffer_for_root(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    root: &Path,
    now: Instant,
) -> Result<(), String> {
    let snapshot = match git_status_snapshot(runtime, root) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            set_git_status_error(runtime, buffer_id, &error)?;
            return Err(error);
        }
    };
    apply_git_status_snapshot(runtime, buffer_id, snapshot)?;
    shell_buffer_mut(runtime, buffer_id)?.mark_git_status_refreshed(root, now);
    Ok(())
}

pub(crate) fn set_git_status_error(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    message: &str,
) -> Result<(), String> {
    record_runtime_error(runtime, "git.status", message.to_owned());
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.replace_with_lines(vec![
        "Git status unavailable.".to_owned(),
        message.to_owned(),
    ]);
    Ok(())
}

pub(crate) fn apply_git_status_snapshot(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    snapshot: GitStatusSnapshot,
) -> Result<(), String> {
    let user_library = shell_user_library(runtime);
    let sections = user_library.git_status_sections(&snapshot);
    let collapsed = shell_buffer(runtime, buffer_id)?
        .section_state()
        .map(|state| state.collapsed.clone())
        .unwrap_or_default();
    let lines = sections.render_lines(&collapsed);
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    {
        let state = buffer.ensure_section_state();
        state.collapsed = collapsed;
    }
    buffer.set_git_snapshot(snapshot);
    buffer.set_section_lines(lines);
    Ok(())
}

pub(crate) fn open_git_status_popup(runtime: &mut EditorRuntime) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = runtime
        .model_mut()
        .create_popup_buffer(
            workspace_id,
            "*git-status*",
            BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .open_popup(workspace_id, "Git Status", vec![buffer_id], buffer_id)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_popup_buffer(
            buffer_id,
            "*git-status*",
            BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
            &*user_library,
        );
        ui.set_popup_buffer(buffer_id);
    }
    shell_ui_mut(runtime)?.set_popup_focus(true);
    refresh_git_status_buffer(runtime, buffer_id)
}

pub(crate) fn open_git_stash_list_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let args = git_args_with_no_pager("stash", &["list"]);
    let view = GitViewState::new("stash", args, "No stashes.", &[0]);
    open_git_view_buffer(runtime, GIT_STASH_KIND, "*git-stash*", view)
}

pub(crate) fn take_git_prefix(runtime: &mut EditorRuntime) -> Result<Option<GitPrefix>, String> {
    const PREFIX_TIMEOUT: Duration = Duration::from_millis(1200);
    let now = Instant::now();
    let ui = shell_ui_mut(runtime)?;
    let prefix = match ui.pending_git_prefix.take() {
        Some(state) if now.duration_since(state.started_at) <= PREFIX_TIMEOUT => {
            Some(state.prefix())
        }
        _ => None,
    };
    Ok(prefix)
}

pub(crate) fn set_git_prefix(runtime: &mut EditorRuntime, prefix: GitPrefix) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    ui.pending_git_prefix = Some(GitPrefixState {
        prefix,
        started_at: Instant::now(),
    });
    Ok(())
}

pub(crate) type ShellCommandHandler = fn(&mut EditorRuntime) -> Result<(), String>;

pub(crate) fn git_status_sequence_kind(
    runtime: &mut EditorRuntime,
    message: &str,
) -> Result<GitSequenceKind, String> {
    git_sequence_in_progress(runtime)?.ok_or_else(|| message.to_owned())
}

pub(crate) fn git_status_command_name(
    user_library: &dyn UserLibrary,
    prefix: Option<GitPrefix>,
    chord: &str,
) -> Option<&'static str> {
    user_library.git_command_for_chord(prefix, chord)
}
