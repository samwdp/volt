use super::*;
use std::process::Stdio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GitFringeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Default)]
pub(super) struct GitFringeSnapshot {
    lines: BTreeMap<usize, GitFringeKind>,
}

impl GitFringeSnapshot {
    fn line_kind(&self, line_index: usize) -> Option<GitFringeKind> {
        self.lines.get(&line_index).copied()
    }
}

#[derive(Debug, Clone)]
pub(super) struct GitFringeState {
    snapshot: Arc<Mutex<GitFringeSnapshot>>,
    inflight: Arc<AtomicBool>,
    revision: Arc<AtomicU64>,
}

impl GitFringeState {
    pub(super) fn new() -> Self {
        Self {
            snapshot: Arc::new(Mutex::new(GitFringeSnapshot::default())),
            inflight: Arc::new(AtomicBool::new(false)),
            revision: Arc::new(AtomicU64::new(0)),
        }
    }

    pub(super) fn try_begin_refresh(&self) -> bool {
        self.inflight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(super) fn finish_refresh(&self) {
        self.inflight.store(false, Ordering::Release);
    }

    pub(super) fn try_line_kind(&self, line_index: usize) -> Option<GitFringeKind> {
        let guard = self.snapshot.try_lock().ok()?;
        guard.line_kind(line_index)
    }

    pub(super) fn update_snapshot(&self, snapshot: GitFringeSnapshot) {
        if let Ok(mut guard) = self.snapshot.lock() {
            *guard = snapshot;
            self.revision.fetch_add(1, Ordering::AcqRel);
        }
    }

    pub(super) fn snapshot_revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct GitSummarySnapshot {
    pub(super) branch: Option<String>,
    pub(super) head: Option<String>,
    pub(super) added: usize,
    pub(super) removed: usize,
}

#[derive(Debug, Clone)]
pub(super) struct GitSummaryState {
    snapshot: Arc<Mutex<Option<GitSummarySnapshot>>>,
    inflight: Arc<AtomicBool>,
    revision: Arc<AtomicU64>,
    changed: Arc<AtomicBool>,
    last_refresh_at: Option<Instant>,
}

impl GitSummaryState {
    pub(super) fn new() -> Self {
        Self {
            snapshot: Arc::new(Mutex::new(None)),
            inflight: Arc::new(AtomicBool::new(false)),
            revision: Arc::new(AtomicU64::new(0)),
            changed: Arc::new(AtomicBool::new(false)),
            last_refresh_at: None,
        }
    }

    pub(super) fn snapshot(&self) -> Option<GitSummarySnapshot> {
        let guard = self.snapshot.lock().ok()?;
        guard.clone()
    }

    pub(super) fn set_snapshot(&self, snapshot: Option<GitSummarySnapshot>) {
        if let Ok(mut guard) = self.snapshot.lock() {
            if *guard == snapshot {
                return;
            }
            *guard = snapshot;
            self.revision.fetch_add(1, Ordering::AcqRel);
            self.changed.store(true, Ordering::Release);
        }
    }

    pub(super) fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::AcqRel)
    }

    pub(super) fn refresh_due(&self, now: Instant) -> bool {
        self.last_refresh_at
            .map(|last| now.duration_since(last) >= GIT_SUMMARY_REFRESH_INTERVAL)
            .unwrap_or(true)
    }

    pub(super) fn mark_refreshed(&mut self, now: Instant) {
        self.last_refresh_at = Some(now);
    }

    pub(super) fn try_begin_refresh(&self) -> bool {
        self.inflight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(super) fn finish_refresh(&self) {
        self.inflight.store(false, Ordering::Release);
    }

    pub(super) fn snapshot_revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ActiveBufferEventContext {
    pub(super) buffer_id: BufferId,
    pub(super) has_input: bool,
    pub(super) vim_targets_input: bool,
    pub(super) is_read_only: bool,
    pub(super) is_git_status: bool,
    pub(super) is_git_commit: bool,
    pub(super) is_acp: bool,
    pub(super) is_directory: bool,
    pub(super) is_browser: bool,
    pub(super) is_terminal: bool,
    pub(super) is_db_query: bool,
    pub(super) is_plugin_evaluatable: bool,
}

pub(super) fn default_vim_target(has_input: bool) -> VimTarget {
    if has_input {
        VimTarget::Input
    } else {
        VimTarget::Buffer
    }
}

#[derive(Debug, Clone)]
pub(super) struct ActiveLspBufferContext {
    pub(super) workspace_id: WorkspaceId,
    pub(super) buffer_id: BufferId,
    pub(super) path: PathBuf,
    pub(super) text: String,
    pub(super) revision: u64,
    pub(super) root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(super) struct GitViewState {
    label: String,
    args: Vec<String>,
    empty_message: String,
    allowed_exit_codes: Vec<i32>,
}

impl GitViewState {
    fn new(
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

pub(super) fn format_section_line(line: &SectionRenderLine) -> String {
    let indent = "  ".repeat(line.depth);
    match &line.kind {
        SectionRenderLineKind::Header { .. } => format!("{indent}{}", line.text),
        SectionRenderLineKind::Item => format!("{indent}{}", line.text),
        SectionRenderLineKind::Spacer => String::new(),
    }
}

pub(super) fn git_status_line_spans(
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

pub(super) fn git_status_header_item_spans(
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

pub(super) fn git_status_head_spans(
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

pub(super) fn git_status_upstream_spans(
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

fn git_head_tag(root: &Path) -> Option<String> {
    git_read_command_output_optional(
        root,
        "describe --tags --abbrev=0",
        &["describe", "--tags", "--abbrev=0"],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
}

pub(super) fn git_status_entry_item_spans(
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

pub(super) fn git_status_entry_token(label: &str) -> &'static str {
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

pub(super) fn git_status_entry_token_from_icon(icon: &str) -> &'static str {
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

pub(super) fn git_status_stash_item_spans(
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

pub(super) fn git_status_commit_item_spans(
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

pub(super) fn git_status_commit_message_spans(
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

type IconPrefixedContent<'a> = (Option<(usize, usize)>, usize, &'a str);

pub(super) fn split_icon_prefixed_content(text: &str) -> Option<IconPrefixedContent<'_>> {
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

pub(super) fn leading_indent_bytes(line: &str) -> usize {
    line.char_indices()
        .find(|(_, character)| *character != ' ')
        .map(|(index, _)| index)
        .unwrap_or_else(|| line.len())
}

pub(super) fn push_span_bytes(
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
            capture_name: token.to_owned(),
            theme_token: token.to_owned(),
        });
    }
}

pub(super) fn next_word_bounds(text: &str, start: usize) -> Option<(usize, usize)> {
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

pub(super) fn skip_whitespace_bytes(text: &str, start: usize) -> usize {
    let start = start.min(text.len());
    for (offset, character) in text[start..].char_indices() {
        if !character.is_whitespace() {
            return start + offset;
        }
    }
    text.len()
}

pub(super) fn is_git_hash(text: &str) -> bool {
    (7..=40).contains(&text.len()) && text.chars().all(|character| character.is_ascii_hexdigit())
}

pub(super) fn find_paren_number_range(text: &str) -> Option<(usize, usize)> {
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

pub(super) fn push_number_after_keyword(
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

pub(super) type GitPrefix = editor_plugin_api::GitStatusPrefix;

#[derive(Debug, Clone)]
pub(super) struct GitPrefixState {
    prefix: GitPrefix,
    started_at: Instant,
}

pub(super) fn refresh_git_status_if_active(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    if !buffer_is_git_status(&shell_buffer(runtime, buffer_id)?.kind) {
        return Ok(());
    }
    refresh_git_status_buffer(runtime, buffer_id)
}

pub(super) fn refresh_git_status_if_active_if_due(
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

pub(super) fn refresh_git_status_buffers(runtime: &mut EditorRuntime) -> Result<(), String> {
    mark_git_fringe_snapshots_stale(runtime)?;
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

pub(super) fn refresh_git_status_buffer(
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

fn refresh_git_status_buffer_for_root(
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

pub(super) fn set_git_status_error(
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

pub(super) fn apply_git_status_snapshot(
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

pub(super) fn open_git_status_popup(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn open_git_commit_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let existing = shell_ui(runtime)
        .ok()
        .and_then(|ui| find_shell_buffer_by_kind(ui, GIT_COMMIT_KIND));
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.focus_buffer_in_active_pane(existing);
        ui.enter_normal_mode();
        return Ok(());
    }
    let buffer_id = {
        runtime
            .model_mut()
            .create_buffer(
                workspace_id,
                "*git-commit*",
                BufferKind::Plugin(GIT_COMMIT_KIND.to_owned()),
                None,
            )
            .map_err(|error| error.to_string())?
    };
    let root = git_root(runtime)?;
    let snapshot = git_status_snapshot(runtime, &root)?;
    let user_library = shell_user_library(runtime);
    let template = user_library.git_commit_template(&snapshot);
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, template, &*user_library);
    shell_buffer.set_language_id(Some("gitcommit".to_owned()));
    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    queue_buffer_syntax_refresh(runtime, buffer_id)
}

pub(super) fn git_commit_temp_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    env::temp_dir().join(format!(
        "volt-git-commit-{}-{unique}.txt",
        std::process::id()
    ))
}

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
    match kind {
        GitSequenceKind::CherryPick => {
            git_command_output(
                runtime,
                &root,
                "cherry-pick --continue",
                &["cherry-pick", "--continue"],
            )?;
        }
        GitSequenceKind::Revert => {
            git_command_output(
                runtime,
                &root,
                "revert --continue",
                &["revert", "--continue"],
            )?;
        }
    }
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn sequence_git_skip(
    runtime: &mut EditorRuntime,
    kind: GitSequenceKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    match kind {
        GitSequenceKind::CherryPick => {
            git_command_output(
                runtime,
                &root,
                "cherry-pick --skip",
                &["cherry-pick", "--skip"],
            )?;
        }
        GitSequenceKind::Revert => {
            git_command_output(runtime, &root, "revert --skip", &["revert", "--skip"])?;
        }
    }
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn sequence_git_abort(
    runtime: &mut EditorRuntime,
    kind: GitSequenceKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    match kind {
        GitSequenceKind::CherryPick => {
            git_command_output(
                runtime,
                &root,
                "cherry-pick --abort",
                &["cherry-pick", "--abort"],
            )?;
        }
        GitSequenceKind::Revert => {
            git_command_output(runtime, &root, "revert --abort", &["revert", "--abort"])?;
        }
    }
    refresh_git_status_buffers(runtime)?;
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
    git_command_output(runtime, &root, "cherry-pick", &["cherry-pick", commit])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn cherry_pick_git_commit_no_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "cherry-pick --no-commit",
        &["cherry-pick", "--no-commit", commit],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn revert_git_commit(runtime: &mut EditorRuntime, commit: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "revert", &["revert", commit])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn revert_git_commit_no_commit(
    runtime: &mut EditorRuntime,
    commit: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "revert --no-commit",
        &["revert", "--no-commit", commit],
    )?;
    refresh_git_status_buffers(runtime)?;
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
    git_command_output(runtime, &root, "merge", &["merge", branch])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn merge_git_edit(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "merge --no-commit --edit",
        &["merge", "--no-commit", "--edit", branch],
    )?;
    refresh_git_status_buffers(runtime)?;
    open_git_commit_buffer(runtime)
}

pub(super) fn merge_git_no_commit(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "merge --no-commit",
        &["merge", "--no-commit", branch],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn merge_git_squash(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "merge --squash",
        &["merge", "--squash", branch],
    )?;
    refresh_git_status_buffers(runtime)?;
    open_git_commit_buffer(runtime)
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

pub(super) fn merge_git_continue(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "merge --continue", &["merge", "--continue"])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn merge_git_abort(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "merge --abort", &["merge", "--abort"])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn rebase_git_onto(runtime: &mut EditorRuntime, target: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "rebase", &["rebase", target])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn rebase_git_interactive_onto(
    runtime: &mut EditorRuntime,
    target: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "rebase -i", &["rebase", "-i", target])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn rebase_git_onto_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let upstream = snapshot
        .upstream()
        .ok_or_else(|| "no upstream configured for rebase".to_owned())?;
    rebase_git_onto(runtime, upstream)
}

pub(super) fn rebase_git_onto_pushremote(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let push_remote = snapshot
        .push_remote()
        .ok_or_else(|| "no push-remote configured for rebase".to_owned())?;
    rebase_git_onto(runtime, push_remote)
}

pub(super) fn rebase_git_continue(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "rebase --continue",
        &["rebase", "--continue"],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn rebase_git_skip(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "rebase --skip", &["rebase", "--skip"])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn rebase_git_edit_todo(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(
        runtime,
        &root,
        "rebase --edit-todo",
        &["rebase", "--edit-todo"],
    )?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn rebase_git_abort(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "rebase --abort", &["rebase", "--abort"])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn open_git_cherry_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let upstream = snapshot
        .upstream()
        .ok_or_else(|| "no upstream configured for cherry".to_owned())?;
    let args = git_args_with_no_pager("cherry", &["-v", upstream]);
    let view = GitViewState::new("cherry", args, "No cherry commits.", &[0]);
    open_git_log_buffer(runtime, view)
}

pub(super) fn push_git_remote(runtime: &mut EditorRuntime, remote: &str) -> Result<(), String> {
    let branch = {
        let buffer_id = active_shell_buffer_id(runtime)?;
        shell_buffer(runtime, buffer_id)?
            .git_snapshot()
            .and_then(|snapshot| snapshot.branch())
            .map(str::to_owned)
            .ok_or_else(|| "git push requires a current branch".to_owned())?
    };
    run_git_push_in_popup_buffer(
        runtime,
        vec![
            "push".to_owned(),
            "--progress".to_owned(),
            "--set-upstream".to_owned(),
            remote.to_owned(),
            branch,
        ],
    )
}

pub(super) fn open_git_remote_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    let remotes = git_remote_list(runtime, &root)?;
    if remotes.is_empty() {
        return Err("no git remotes found".to_owned());
    }
    let entries = remotes
        .into_iter()
        .map(|remote| {
            let item_id = format!("git-remote:{remote}");
            let action = PickerAction::GitPushRemote(remote.clone());
            PickerEntry {
                item: PickerItem::new(item_id, remote.clone(), "remote", None::<String>),
                action,
                quickfix: None,
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries("Git Push", entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(super) fn open_git_fetch_remote_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    let remotes = git_remote_list(runtime, &root)?;
    if remotes.is_empty() {
        return Err("no git remotes found".to_owned());
    }
    let entries = remotes
        .into_iter()
        .map(|remote| {
            let item_id = format!("git-fetch-remote:{remote}");
            let action = PickerAction::GitFetchRemote(remote.clone());
            PickerEntry {
                item: PickerItem::new(item_id, remote.clone(), "remote", None::<String>),
                action,
                quickfix: None,
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries("Git Fetch", entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(super) fn git_snapshot_for_buffer(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<GitStatusSnapshot, String> {
    shell_buffer(runtime, buffer_id)?
        .git_snapshot()
        .cloned()
        .ok_or_else(|| "git status snapshot is missing".to_owned())
}

pub(super) fn remote_name_from_ref(reference: &str) -> Option<String> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.split('/').next().unwrap_or(trimmed).to_owned())
}

pub(super) fn remote_and_branch_from_ref(reference: &str) -> Option<(String, String)> {
    let trimmed = reference.trim();
    let (remote, branch) = trimmed.split_once('/')?;
    if remote.is_empty() || branch.is_empty() {
        return None;
    }
    Some((remote.to_owned(), branch.to_owned()))
}

fn git_config_get(root: &Path, key: &str) -> Option<String> {
    git_read_command_output_optional(
        root,
        &format!("config --get {key}"),
        &["config", "--get", key],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
}

fn git_branch_push_remote(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.pushRemote"))
}

fn git_branch_remote(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.remote")).filter(|remote| remote != ".")
}

fn git_branch_merge(root: &Path, branch: &str) -> Option<String> {
    git_config_get(root, &format!("branch.{branch}.merge"))
}

fn local_branch_name_from_merge_ref(reference: &str) -> Option<String> {
    reference
        .strip_prefix("refs/heads/")
        .map(str::to_owned)
        .filter(|branch| !branch.is_empty())
}

fn git_push_remote_name(root: &Path, snapshot: &GitStatusSnapshot) -> Option<String> {
    let branch = snapshot.branch()?;
    let upstream_branch = git_branch_merge(root, branch)
        .as_deref()
        .and_then(local_branch_name_from_merge_ref);
    git_branch_push_remote(root, branch)
        .or_else(|| git_config_get(root, "remote.pushDefault"))
        .or_else(|| git_branch_remote(root, branch))
        .or_else(|| {
            upstream_branch
                .as_deref()
                .and_then(|upstream_branch| git_branch_push_remote(root, upstream_branch))
        })
        .or_else(|| {
            upstream_branch
                .as_deref()
                .and_then(|upstream_branch| git_branch_remote(root, upstream_branch))
        })
        .or_else(|| snapshot.upstream().and_then(remote_name_from_ref))
        .or_else(|| snapshot.push_remote().and_then(remote_name_from_ref))
}

fn status_output_upstream(status_output: &str) -> Option<String> {
    status_output.lines().find_map(|line| {
        let line = line.strip_prefix("## ")?;
        let (_, tracking) = line.split_once("...")?;
        let upstream = tracking
            .split_once(" [")
            .map(|(upstream, _)| upstream)
            .unwrap_or(tracking)
            .trim();
        (!upstream.is_empty()).then(|| upstream.to_owned())
    })
}

pub(super) fn fetch_git_remote(runtime: &mut EditorRuntime, remote: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "fetch", &["fetch", remote])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn fetch_git_all(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "fetch --all", &["fetch", "--all"])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn fetch_git_prune(runtime: &mut EditorRuntime, root: &Path) -> Result<(), String> {
    let remotes = git_remote_list(runtime, root)?;
    if remotes.is_empty() {
        return Err("no git remotes found".to_owned());
    }
    for remote in remotes {
        let refspec = format!("+refs/heads/*:refs/remotes/{remote}/*");
        git_read_command_output(
            root,
            "fetch --prune remote-tracking refs",
            &["fetch", "--prune", &remote, &refspec],
        )?;
    }
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn fetch_git_pushremote(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    if let Some(remote) = git_push_remote_name(&root, &snapshot) {
        fetch_git_remote(runtime, &remote)?;
        return Ok(());
    }
    open_git_fetch_remote_picker(runtime)
}

pub(super) fn fetch_git_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let remote = snapshot
        .upstream()
        .and_then(remote_name_from_ref)
        .ok_or_else(|| "no upstream configured for fetch".to_owned())?;
    fetch_git_remote(runtime, &remote)
}

pub(super) fn pull_git_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let (remote, branch) = snapshot
        .upstream()
        .and_then(remote_and_branch_from_ref)
        .ok_or_else(|| "no upstream configured for pull".to_owned())?;
    pull_git_remote_branch(runtime, &remote, &branch)
}

pub(super) fn push_git_to_pushremote(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    if let Some(remote) = git_push_remote_name(&root, &snapshot) {
        push_git_remote(runtime, &remote)?;
        return Ok(());
    }
    open_git_remote_picker(runtime)
}

pub(super) fn push_git_to_upstream(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let snapshot = git_snapshot_for_buffer(runtime, buffer_id)?;
    let (remote, branch) = snapshot
        .upstream()
        .and_then(remote_and_branch_from_ref)
        .ok_or_else(|| "no upstream configured for push".to_owned())?;
    push_git_remote_branch(runtime, &remote, &branch)
}

pub(super) fn push_git_remote_branch(
    runtime: &mut EditorRuntime,
    remote: &str,
    branch: &str,
) -> Result<(), String> {
    run_git_push_in_popup_buffer(
        runtime,
        vec![
            "push".to_owned(),
            "--progress".to_owned(),
            remote.to_owned(),
            branch.to_owned(),
        ],
    )
}

fn run_git_push_in_popup_buffer(
    runtime: &mut EditorRuntime,
    args: Vec<String>,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let command_label = format!("git {}", args.join(" "));
    let buffer_name = format!("*{command_label}*");
    open_streamed_command_popup(
        runtime,
        StreamedCommandSpec {
            popup_title: "Git Push".to_owned(),
            buffer_name,
            command_label,
            program: "git".to_owned(),
            args,
            env: Vec::new(),
            cwd: root,
            on_exit: StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
            notify_on_success: true,
            notify_on_failure: true,
        },
    )?;
    Ok(())
}

pub(super) fn pull_git_remote_branch(
    runtime: &mut EditorRuntime,
    remote: &str,
    branch: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "pull", &["pull", remote, branch])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn git_branch_list(
    _runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<Vec<String>, String> {
    let output = git_read_command_output(
        root,
        "branch --format",
        &["branch", "--format=%(refname:short)"],
    )?;
    let mut branches = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();
    branches.sort();
    branches.dedup();
    Ok(branches)
}

pub(super) fn git_remote_worktree_branch_list(
    runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<Vec<(String, String)>, String> {
    trace_oil_worktree(
        runtime,
        format!("listing remote branches from `{}`", root.display()),
    );
    let fetch_error = fetch_git_prune(runtime, root).err();
    match &fetch_error {
        Some(error) => trace_oil_worktree(runtime, format!("git fetch --prune failed: {error}")),
        None => trace_oil_worktree(runtime, "git fetch --prune succeeded"),
    }
    let local = git_branch_list(runtime, root)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    trace_oil_worktree(runtime, format!("found {} local branches", local.len()));
    let output = match git_read_command_output(
        root,
        "branch -r --format",
        &["branch", "-r", "--format=%(refname:short)"],
    ) {
        Ok(output) => output,
        Err(error) => return Err(fetch_error.unwrap_or(error)),
    };
    let mut branches = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.ends_with("/HEAD"))
        .filter_map(|remote_branch| {
            let (_, local_branch) = remote_and_branch_from_ref(remote_branch)?;
            let local_branch = if local.contains(&local_branch) {
                remote_branch.replace('/', "-")
            } else {
                local_branch
            };
            Some((remote_branch.to_owned(), local_branch))
        })
        .collect::<Vec<_>>();
    if branches.is_empty() {
        trace_oil_worktree(
            runtime,
            "no remote branches found; falling back to local branch refs",
        );
        branches = local
            .iter()
            .filter(|branch| branch.as_str() != "HEAD")
            .map(|branch| (branch.clone(), branch.clone()))
            .collect::<Vec<_>>();
    }
    branches.sort();
    branches.dedup();
    if branches.is_empty() {
        if let Some(error) = fetch_error {
            return Err(error);
        }
    } else if let Some(error) = fetch_error {
        record_runtime_error(runtime, "git.worktree.fetch", error);
    }
    trace_oil_worktree(
        runtime,
        format!(
            "found {} remote branches for worktree picker",
            branches.len()
        ),
    );
    Ok(branches)
}

pub(super) fn git_commit_list(
    _runtime: &mut EditorRuntime,
    root: &Path,
    limit: usize,
) -> Result<Vec<GitLogEntry>, String> {
    let output = git_read_command_output(
        root,
        "log --oneline",
        &["log", "-n", &limit.to_string(), "--oneline"],
    )?;
    Ok(parse_log_oneline(&output))
}

pub(super) fn open_git_commit_picker_with_action(
    runtime: &mut EditorRuntime,
    title: &str,
    action: GitCommitActionKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let commits = git_commit_list(runtime, &root, GIT_LOG_VIEW_LIMIT)?;
    if commits.is_empty() {
        return Err("no commits found".to_owned());
    }
    let entries = commits
        .into_iter()
        .map(|commit| {
            let label = format!("{} {}", commit.hash(), commit.summary());
            let item_id = format!("git-commit:{}", commit.hash());
            let action = PickerAction::GitCommitAction {
                action,
                commit: commit.hash().to_owned(),
            };
            PickerEntry {
                item: PickerItem::new(item_id, label, "commit", None::<String>),
                action,
                quickfix: None,
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries(title, entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(super) fn open_git_branch_picker_with_action(
    runtime: &mut EditorRuntime,
    title: &str,
    action: GitBranchActionKind,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let branches = git_branch_list(runtime, &root)?;
    if branches.is_empty() {
        return Err("no git branches found".to_owned());
    }
    let entries = branches
        .into_iter()
        .map(|branch| {
            let item_id = format!("git-branch:{branch}");
            let action = PickerAction::GitBranchAction {
                action,
                branch: branch.clone(),
            };
            PickerEntry {
                item: PickerItem::new(item_id, branch.clone(), "branch", None::<String>),
                action,
                quickfix: None,
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries(title, entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(super) fn open_git_branch_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(runtime, "Git Branches", GitBranchActionKind::Checkout)
}

pub(super) fn open_git_worktree_branch_picker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = git_root(runtime)?;
    let entries = git_remote_worktree_branch_list(runtime, &root)?
        .into_iter()
        .map(|(remote_branch, local_branch)| {
            let item_id = format!("git-worktree-branch:{remote_branch}");
            let action = PickerAction::GitWorktreeBranch {
                remote_branch: remote_branch.clone(),
                local_branch: local_branch.clone(),
            };
            PickerEntry {
                item: PickerItem::new(
                    item_id,
                    remote_branch,
                    format!("create local branch {local_branch}"),
                    None::<String>,
                ),
                action,
                quickfix: None,
            }
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Err("no remote-only branches found".to_owned());
    }
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries("Git Worktree Branch", entries));
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GitWorktreeListEntry {
    /// Exact `worktree` path from `git worktree list --porcelain` (Git identity).
    raw_path: String,
    /// Normalized filesystem path for workspace / picker use.
    path: PathBuf,
    branch: Option<String>,
    head: Option<String>,
    bare: bool,
    /// `prunable` in porcelain — checkout/admin already broken; `remove` cannot succeed.
    prunable: bool,
}

/// Git invocation for Worktree Remove after resolving the registered worktree entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WorktreeRemoveGitInvocation {
    /// `git worktree remove <raw_path> --force` using the porcelain path spelling.
    Remove { cli_path: String },
    /// `git worktree prune -v` when the entry is already prunable.
    Prune,
}

/// Picks remove vs prune and the CLI path Git will recognize.
///
/// Git for Windows may register worktrees as `/w/...` while Volt normalizes to `W:\...`.
/// `git worktree remove` matches the registered spelling, so args must use [`GitWorktreeListEntry::raw_path`].
pub(super) fn worktree_remove_git_invocation_for_entries(
    entries: &[GitWorktreeListEntry],
    selected: &Path,
) -> WorktreeRemoveGitInvocation {
    let Some(entry) = entries
        .iter()
        .find(|entry| !entry.bare && project_roots_equal(&entry.path, selected))
    else {
        return WorktreeRemoveGitInvocation::Remove {
            cli_path: selected.display().to_string(),
        };
    };
    if entry.prunable {
        WorktreeRemoveGitInvocation::Prune
    } else {
        WorktreeRemoveGitInvocation::Remove {
            cli_path: entry.raw_path.clone(),
        }
    }
}

pub(super) fn worktree_remove_git_args(invocation: &WorktreeRemoveGitInvocation) -> Vec<String> {
    match invocation {
        WorktreeRemoveGitInvocation::Remove { cli_path } => vec![
            "worktree".to_owned(),
            "remove".to_owned(),
            cli_path.clone(),
            "--force".to_owned(),
        ],
        WorktreeRemoveGitInvocation::Prune => {
            vec!["worktree".to_owned(), "prune".to_owned(), "-v".to_owned()]
        }
    }
}

impl GitWorktreeListEntry {
    fn display_name(&self, base_dir: &Path) -> String {
        if let Some(path) = self
            .path
            .strip_prefix(base_dir)
            .ok()
            .and_then(|path| path.to_str())
            .filter(|path| !path.is_empty())
        {
            return path.to_owned();
        }
        if let Some(name) = self.path.file_name().and_then(|name| name.to_str()) {
            return name.to_owned();
        }
        self.path.to_string_lossy().into_owned()
    }

    fn detail(&self) -> String {
        let mut parts = Vec::new();
        if let Some(branch) = &self.branch {
            parts.push(branch.clone());
        }
        if let Some(head) = &self.head {
            parts.push(head.chars().take(12).collect::<String>());
        }
        if self.bare {
            parts.push("bare".to_owned());
        }
        parts.join(" | ")
    }
}

pub(super) fn git_worktree_dashboard_picker_overlay(
    runtime: &EditorRuntime,
) -> Result<PickerOverlay, String> {
    let base_dir = worktree_dashboard_base_dir(runtime)?;
    let worktrees = git_worktree_list(&base_dir)?;
    let mut entries = worktrees
        .into_iter()
        .filter(|entry| !entry.bare)
        .map(|entry| {
            let existing_workspace = find_workspace_by_root(runtime, &entry.path)?;
            let name = entry.display_name(&base_dir);
            let detail = {
                let mut detail = entry.detail();
                if existing_workspace.is_some() {
                    if !detail.is_empty() {
                        detail.push_str(" | ");
                    }
                    detail.push_str("open workspace");
                }
                detail
            };
            let action = existing_workspace.map_or(
                PickerAction::CreateWorkspace {
                    name: name.clone(),
                    root: entry.path.clone(),
                },
                PickerAction::SwitchWorkspace,
            );
            Ok(PickerEntry {
                item: PickerItem::new(
                    entry.path.display().to_string(),
                    name,
                    detail,
                    Some(entry.path.display().to_string()),
                ),
                action,
                quickfix: None,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    entries.insert(
        0,
        PickerEntry {
            item: PickerItem::new(
                "git-worktree-dashboard:create",
                "+ new worktree",
                base_dir.display().to_string(),
                Some("Open oil at the bare repo and choose a branch.".to_owned()),
            ),
            action: PickerAction::GitWorktreeDashboardCreate { base_dir },
            quickfix: None,
        },
    );

    Ok(PickerOverlay::from_entries("Workspace Dashboard", entries))
}

pub(super) fn open_git_worktree_dashboard_create(
    runtime: &mut EditorRuntime,
    base_dir: &Path,
) -> Result<(), String> {
    split_runtime_pane(runtime, PaneSplitDirection::Vertical)?;
    open_oil_directory(runtime, base_dir.to_path_buf())?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    begin_oil_worktree_request(runtime, buffer_id)
}

/// Force-removes a Worktree from disk using one-shot picker context.
///
/// Closes matching Project Workspaces first, then streams
/// `git worktree remove <path> --force`. Missing path / create affordance → no-op.
pub(super) fn worktree_remove_from_one_shot(runtime: &mut EditorRuntime) -> Result<(), String> {
    let Some(context) = shell_ui_mut(runtime)?.take_picker_one_shot() else {
        return Ok(());
    };
    let selected_path = context
        .selected()
        .and_then(PickerSelectedRow::path)
        .map(PathBuf::from);
    let open = open_project_workspaces_with_roots(runtime)?;
    let Some(plan) = plan_worktree_remove(selected_path.as_deref(), &open) else {
        return Ok(());
    };

    for workspace_id in plan.workspace_ids_to_close {
        delete_runtime_workspace(runtime, workspace_id)?;
    }

    let cwd = worktree_remove_repo_cwd(runtime, &plan.request.path)?;
    let entries = git_worktree_list(&cwd)?;
    let invocation = worktree_remove_git_invocation_for_entries(&entries, &plan.request.path);
    if matches!(invocation, WorktreeRemoveGitInvocation::Prune)
        && plan.request.path.exists()
        && let Err(error) = std::fs::remove_dir_all(&plan.request.path)
    {
        return Err(format!(
            "failed to delete leftover worktree `{}`: {error}",
            plan.request.path.display()
        ));
    }
    let args = worktree_remove_git_args(&invocation);
    let command_label = format!("git {}", args.join(" "));
    let buffer_name = format!("*{command_label}*");
    open_streamed_command_popup(
        runtime,
        StreamedCommandSpec {
            popup_title: "Worktree Remove".to_owned(),
            buffer_name,
            command_label,
            program: "git".to_owned(),
            args,
            env: Vec::new(),
            cwd,
            on_exit: StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer,
            notify_on_success: true,
            notify_on_failure: true,
        },
    )?;
    Ok(())
}

fn worktree_dashboard_base_dir(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    if let Some(root) = active_directory_root(runtime)? {
        return git_common_dir(&root).or(Ok(root));
    }
    if let Some(root) = active_workspace_root(runtime)? {
        return git_common_dir(&root).or_else(|_| {
            root.parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| format!("workspace root `{}` has no parent", root.display()))
        });
    }
    env::current_dir().map_err(|error| format!("workspace dashboard requires a root: {error}"))
}

/// Resolves a git cwd that can list/remove worktrees for `selected`.
///
/// Broken/prunable checkouts may lack `.git`, so fall back to open Project
/// Workspace roots and parent directories before giving up.
fn worktree_remove_repo_cwd(runtime: &EditorRuntime, selected: &Path) -> Result<PathBuf, String> {
    if let Ok(cwd) = git_common_dir(selected) {
        return Ok(cwd);
    }

    for (_, root) in open_project_workspaces_with_roots(runtime)? {
        if let Ok(cwd) = git_common_dir(&root) {
            return Ok(cwd);
        }
    }

    let mut cursor = selected.parent();
    while let Some(dir) = cursor {
        if let Ok(cwd) = git_common_dir(dir) {
            return Ok(cwd);
        }
        cursor = dir.parent();
    }

    selected.parent().map(Path::to_path_buf).ok_or_else(|| {
        format!(
            "cannot resolve git repository for worktree `{}`",
            selected.display()
        )
    })
}

fn git_common_dir(root: &Path) -> Result<PathBuf, String> {
    let output = git_read_command_output(
        root,
        "rev-parse --git-common-dir",
        &["rev-parse", "--git-common-dir"],
    )?;
    let common_dir = output.trim();
    if common_dir.is_empty() {
        return Err(format!(
            "git rev-parse --git-common-dir returned no path for {}",
            root.display()
        ));
    }
    let path = normalize_git_output_path(common_dir);
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    Ok(path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| *name == ".git")
        .and_then(|_| path.parent().map(Path::to_path_buf))
        .unwrap_or(path))
}

fn git_worktree_list(root: &Path) -> Result<Vec<GitWorktreeListEntry>, String> {
    parse_git_worktree_list(&git_read_command_output(
        root,
        "worktree list",
        &["worktree", "list", "--porcelain"],
    )?)
}

pub(super) fn parse_git_worktree_list(output: &str) -> Result<Vec<GitWorktreeListEntry>, String> {
    let mut entries = Vec::new();
    let mut current: Option<GitWorktreeListEntry> = None;
    for line in output.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            continue;
        }
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(GitWorktreeListEntry {
                raw_path: path.to_owned(),
                path: normalize_git_output_path(path),
                branch: None,
                head: None,
                bare: false,
                prunable: false,
            });
        } else if let Some(entry) = current.as_mut() {
            if let Some(branch) = line.strip_prefix("branch ") {
                entry.branch = Some(branch.trim_start_matches("refs/heads/").to_owned());
            } else if let Some(head) = line.strip_prefix("HEAD ") {
                entry.head = Some(head.to_owned());
            } else if line == "bare" {
                entry.bare = true;
            } else if line.starts_with("prunable") {
                entry.prunable = true;
            }
        } else {
            return Err(format!(
                "git worktree list returned unexpected line `{line}`"
            ));
        }
    }
    if let Some(entry) = current {
        entries.push(entry);
    }
    Ok(entries)
}

pub(super) fn begin_oil_worktree_request(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    trace_oil_worktree(
        runtime,
        format!("begin oil worktree request for buffer `{buffer_id}`"),
    );
    let root = git_root(runtime)?;
    trace_oil_worktree(runtime, format!("resolved git root `{}`", root.display()));
    let branches = git_remote_worktree_branch_list(runtime, &root)?;
    let mut entries = branches
        .into_iter()
        .map(|(remote_branch, local_branch)| {
            let item_id = format!("git-worktree-oil-branch:{remote_branch}");
            let action = PickerAction::GitWorktreeOilBranch {
                buffer_id,
                remote_branch: remote_branch.clone(),
                local_branch: local_branch.clone(),
            };
            PickerEntry {
                item: PickerItem::new(
                    item_id,
                    remote_branch,
                    format!("create local branch {local_branch}"),
                    None::<String>,
                ),
                action,
                quickfix: None,
            }
        })
        .collect::<Vec<_>>();
    entries.insert(
        0,
        PickerEntry {
            item: PickerItem::new(
                "git-worktree-oil-branch:new",
                "New Branch",
                "create a new local branch and worktree",
                Some("Enter a branch name, then choose the worktree directory in oil.".to_owned()),
            ),
            action: PickerAction::GitWorktreeOilNewBranch { buffer_id },
            quickfix: None,
        },
    );
    let entry_count = entries.len();
    shell_ui_mut(runtime)?.set_picker(
        PickerOverlay::from_entries("Git Worktree Branch", entries)
            .with_result_order(PickerResultOrder::Source),
    );
    trace_oil_worktree(
        runtime,
        format!("set git worktree picker with {entry_count} entries"),
    );
    Ok(())
}

pub(super) fn open_git_worktree_new_branch_prompt(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.set_command_line(CommandLineOverlay::for_worktree_new_branch(buffer_id));
    Ok(())
}

pub(super) fn submit_git_worktree_new_branch_name(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    branch: &str,
) -> Result<(), String> {
    let branch = branch.trim();
    if branch.is_empty() {
        return Err("branch name is required".to_owned());
    }
    if branch.chars().any(char::is_whitespace) {
        return Err("branch name must not contain whitespace".to_owned());
    }
    finish_oil_worktree_branch_selection(runtime, buffer_id, branch, branch, true)?;
    sync_active_buffer(runtime)
}

pub(super) fn open_git_worktree_path_picker(
    runtime: &mut EditorRuntime,
    remote_branch: &str,
    local_branch: &str,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let base_dir = root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.clone());
    let mut picker = PickerOverlay::from_entries(
        format!("Worktree directory for {remote_branch}"),
        Vec::new(),
    );
    picker.submit_action = Some(PickerAction::GitWorktreeCreate {
        remote_branch: remote_branch.to_owned(),
        local_branch: local_branch.to_owned(),
        base_dir,
    });
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(super) fn create_git_worktree_from_query(
    runtime: &mut EditorRuntime,
    remote_branch: &str,
    local_branch: &str,
    base_dir: &Path,
    query: &str,
) -> Result<(), String> {
    let name = query.trim();
    if name.is_empty() {
        return Err("worktree directory name is required".to_owned());
    }
    let worktree_path = worktree_path_from_name(base_dir, name)?;
    create_git_worktree(runtime, remote_branch, local_branch, &worktree_path, false)
}

pub(super) fn finish_oil_worktree_branch_selection(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    remote_branch: &str,
    local_branch: &str,
    create_new_branch: bool,
) -> Result<(), String> {
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.move_line_end();
    buffer.insert_text("\n");
    let line = buffer.cursor_point().line;
    buffer.set_cursor(TextPoint::new(line, buffer.line_len_chars(line)));
    let state = buffer
        .directory_state_mut()
        .ok_or_else(|| "directory state is missing".to_owned())?;
    state.pending_worktree = Some(PendingWorktreeRequest {
        line,
        remote_branch: remote_branch.to_owned(),
        local_branch: local_branch.to_owned(),
        create_new_branch,
    });
    shell_ui_mut(runtime)?.enter_insert_mode();
    Ok(())
}

pub(super) fn create_git_worktree(
    runtime: &mut EditorRuntime,
    remote_branch: &str,
    local_branch: &str,
    worktree_path: &Path,
    create_new_branch: bool,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    if worktree_path.exists() {
        return Err(format!(
            "worktree path already exists: {}",
            worktree_path.display()
        ));
    }
    let path_arg = worktree_path.display().to_string();
    let args = if create_new_branch {
        vec!["worktree", "add", "-b", local_branch, &path_arg]
    } else if remote_branch == local_branch {
        vec!["worktree", "add", &path_arg, local_branch]
    } else {
        vec![
            "worktree",
            "add",
            "--track",
            "-b",
            local_branch,
            &path_arg,
            remote_branch,
        ]
    };
    git_command_output(runtime, &root, "worktree add", &args)?;
    let name = worktree_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(local_branch);
    open_workspace_from_project(runtime, name, worktree_path)?;
    Ok(())
}

pub(super) fn worktree_path_from_name(base_dir: &Path, name: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(name);
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("worktree path must not contain `..`".to_owned());
    }
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(base_dir.join(path))
    }
}

pub(super) fn checkout_git_branch(runtime: &mut EditorRuntime, branch: &str) -> Result<(), String> {
    let root = git_root(runtime)?;
    git_command_output(runtime, &root, "checkout", &["checkout", branch])?;
    refresh_git_status_buffers(runtime)?;
    Ok(())
}

pub(super) fn take_git_prefix(runtime: &mut EditorRuntime) -> Result<Option<GitPrefix>, String> {
    const PREFIX_TIMEOUT: Duration = Duration::from_millis(1200);
    let now = Instant::now();
    let ui = shell_ui_mut(runtime)?;
    let prefix = match ui.pending_git_prefix.take() {
        Some(state) if now.duration_since(state.started_at) <= PREFIX_TIMEOUT => Some(state.prefix),
        _ => None,
    };
    Ok(prefix)
}

pub(super) fn set_git_prefix(runtime: &mut EditorRuntime, prefix: GitPrefix) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    ui.pending_git_prefix = Some(GitPrefixState {
        prefix,
        started_at: Instant::now(),
    });
    Ok(())
}

pub(super) type ShellCommandHandler = fn(&mut EditorRuntime) -> Result<(), String>;

#[derive(Debug, Clone)]
pub(super) struct GitStatusCommandContext {
    buffer_id: BufferId,
    meta: Option<SectionLineMeta>,
    staged_empty: bool,
    has_stage_candidates: bool,
}

pub(super) const GIT_STATUS_COMMANDS: &[(&str, &str, ShellCommandHandler)] = &[
    (
        "git.status.refresh",
        "Refresh the active git status buffer.",
        git_status_refresh_command,
    ),
    (
        "git.status.next-section",
        "Move to the next git status section.",
        git_status_next_section_command,
    ),
    (
        "git.status.previous-section",
        "Move to the previous git status section.",
        git_status_previous_section_command,
    ),
    (
        "git.status.stage",
        "Stage the selected file or all unstaged changes.",
        git_status_stage_command,
    ),
    (
        "git.status.stage-all",
        "Stage all unstaged changes.",
        git_status_stage_all_command,
    ),
    (
        "git.status.unstage",
        "Unstage the selected file.",
        git_status_unstage_command,
    ),
    (
        "git.status.unstage-all",
        "Unstage all staged changes.",
        git_status_unstage_all_command,
    ),
    (
        "git.status.commit",
        "Open the git commit buffer for staged changes.",
        git_status_commit_command,
    ),
    (
        "git.status.push-pushremote",
        "Push to the configured push-remote from the git status buffer.",
        git_status_push_pushremote_command,
    ),
    (
        "git.status.push-upstream",
        "Push to the configured upstream from the git status buffer.",
        git_status_push_upstream_command,
    ),
    (
        "git.status.fetch-pushremote",
        "Fetch from the configured push-remote from the git status buffer.",
        git_status_fetch_pushremote_command,
    ),
    (
        "git.status.fetch-upstream",
        "Fetch from the configured upstream from the git status buffer.",
        git_status_fetch_upstream_command,
    ),
    (
        "git.status.fetch-all",
        "Fetch all remotes.",
        git_status_fetch_all_command,
    ),
    (
        "git.status.pull-upstream",
        "Pull from the configured upstream from the git status buffer.",
        git_status_pull_upstream_command,
    ),
    (
        "git.status.branches",
        "Open the git branch picker.",
        git_status_branches_command,
    ),
    (
        "git.worktree.create",
        "Create a worktree from a remote-only branch.",
        git_worktree_create_command,
    ),
    (
        "git.status.merge",
        "Merge a branch from the git status buffer.",
        git_status_merge_command,
    ),
    (
        "git.status.merge-edit",
        "Merge a branch and edit the merge message.",
        git_status_merge_edit_command,
    ),
    (
        "git.status.merge-no-commit",
        "Merge a branch without committing.",
        git_status_merge_no_commit_command,
    ),
    (
        "git.status.merge-squash",
        "Squash-merge a branch from the git status buffer.",
        git_status_merge_squash_command,
    ),
    (
        "git.status.merge-preview",
        "Preview the diff for a branch merge.",
        git_status_merge_preview_command,
    ),
    (
        "git.status.merge-abort",
        "Abort the current git merge.",
        git_status_merge_abort_command,
    ),
    (
        "git.status.rebase-pushremote",
        "Rebase onto the configured push-remote from the git status buffer.",
        git_status_rebase_pushremote_command,
    ),
    (
        "git.status.rebase-upstream",
        "Rebase onto the configured upstream from the git status buffer.",
        git_status_rebase_upstream_command,
    ),
    (
        "git.status.rebase-onto",
        "Rebase onto a selected branch or edit the current rebase todo.",
        git_status_rebase_onto_command,
    ),
    (
        "git.status.rebase-interactive",
        "Start an interactive rebase from the git status buffer.",
        git_status_rebase_interactive_command,
    ),
    (
        "git.status.rebase-continue",
        "Continue the current git rebase.",
        git_status_rebase_continue_command,
    ),
    (
        "git.status.rebase-skip",
        "Skip the current git rebase commit.",
        git_status_rebase_skip_command,
    ),
    (
        "git.status.rebase-abort",
        "Abort the current git rebase.",
        git_status_rebase_abort_command,
    ),
    (
        "git.status.rebase-autosquash",
        "Autosquash the current git rebase.",
        git_status_rebase_autosquash_command,
    ),
    (
        "git.status.rebase-edit-commit",
        "Edit a commit during the current git rebase.",
        git_status_rebase_edit_commit_command,
    ),
    (
        "git.status.rebase-reword",
        "Reword a commit during the current git rebase.",
        git_status_rebase_reword_command,
    ),
    (
        "git.status.rebase-remove-commit",
        "Remove a commit during the current git rebase.",
        git_status_rebase_remove_commit_command,
    ),
    (
        "git.status.diff-dwim",
        "Open the most relevant git diff for the current status line.",
        git_status_diff_dwim_command,
    ),
    (
        "git.status.diff-staged",
        "Open the staged git diff buffer.",
        git_status_diff_staged_command,
    ),
    (
        "git.status.diff-unstaged",
        "Open the unstaged git diff buffer.",
        git_status_diff_unstaged_command,
    ),
    (
        "git.status.diff-commit",
        "Open a git diff for the commit at point or HEAD.",
        git_status_diff_commit_command,
    ),
    (
        "git.status.diff-stash",
        "Open a git diff for the stash at point.",
        git_status_diff_stash_command,
    ),
    (
        "git.status.diff-range",
        "Diff a git range from the git status buffer.",
        git_status_diff_range_command,
    ),
    (
        "git.status.diff-paths",
        "Diff selected paths from the git status buffer.",
        git_status_diff_paths_command,
    ),
    (
        "git.status.log-head",
        "Open the git log for HEAD.",
        git_status_log_head_command,
    ),
    (
        "git.status.log-related",
        "Open the git log for the branch, upstream, and push-remote related to the status buffer.",
        git_status_log_related_command,
    ),
    (
        "git.status.log-other",
        "Open another git log view from the git status buffer.",
        git_status_log_other_command,
    ),
    (
        "git.status.log-branches",
        "Open the git log for local branches.",
        git_status_log_branches_command,
    ),
    (
        "git.status.log-all-branches",
        "Open the git log for local and remote branches.",
        git_status_log_all_branches_command,
    ),
    (
        "git.status.log-all",
        "Open the git log for all refs.",
        git_status_log_all_command,
    ),
    (
        "git.status.stash-both",
        "Stash both index and worktree changes.",
        git_status_stash_both_command,
    ),
    (
        "git.status.stash-index",
        "Stash staged changes.",
        git_status_stash_index_command,
    ),
    (
        "git.status.stash-worktree",
        "Stash worktree changes.",
        git_status_stash_worktree_command,
    ),
    (
        "git.status.stash-keep-index",
        "Stash changes while keeping the index.",
        git_status_stash_keep_index_command,
    ),
    (
        "git.status.stash-apply",
        "Apply the stash at point.",
        git_status_stash_apply_command,
    ),
    (
        "git.status.stash-pop",
        "Pop the stash at point.",
        git_status_stash_pop_command,
    ),
    (
        "git.status.stash-drop",
        "Drop the stash at point.",
        git_status_stash_drop_command,
    ),
    (
        "git.status.stash-show",
        "Show the stash diff at point.",
        git_status_stash_show_command,
    ),
    (
        "git.status.cherry-open",
        "Open the git cherry buffer for the current upstream.",
        git_status_cherry_open_command,
    ),
    (
        "git.status.cherry-pick",
        "Cherry-pick the commit at point or continue an active sequence.",
        git_status_cherry_pick_command,
    ),
    (
        "git.status.cherry-pick-apply",
        "Apply the commit at point without committing, or abort an active sequence.",
        git_status_cherry_pick_apply_command,
    ),
    (
        "git.status.cherry-pick-skip",
        "Skip the current cherry-pick sequence step.",
        git_status_cherry_pick_skip_command,
    ),
    (
        "git.status.revert",
        "Revert the commit at point or continue an active revert sequence.",
        git_status_revert_command,
    ),
    (
        "git.status.revert-no-commit",
        "Revert the commit at point without committing, or abort an active revert sequence.",
        git_status_revert_no_commit_command,
    ),
    (
        "git.status.revert-skip",
        "Skip the current cherry-pick or revert sequence step.",
        git_status_revert_skip_command,
    ),
    (
        "git.status.revert-abort",
        "Abort the current cherry-pick or revert sequence.",
        git_status_revert_abort_command,
    ),
    (
        "git.status.apply-commit",
        "Apply the commit at point without committing, or open the commit picker.",
        git_status_apply_commit_command,
    ),
    (
        "git.status.reset-mixed",
        "Reset to the selected commit with --mixed.",
        git_status_reset_mixed_command,
    ),
    (
        "git.status.reset-soft",
        "Reset to the selected commit with --soft.",
        git_status_reset_soft_command,
    ),
    (
        "git.status.reset-hard",
        "Reset to the selected commit with --hard.",
        git_status_reset_hard_command,
    ),
    (
        "git.status.reset-keep",
        "Reset to the selected commit with --keep.",
        git_status_reset_keep_command,
    ),
    (
        "git.status.reset-index",
        "Reset the git index from the git status buffer.",
        git_status_reset_index_command,
    ),
    (
        "git.status.reset-worktree",
        "Reset the git worktree from the git status buffer.",
        git_status_reset_worktree_command,
    ),
    (
        "git.status.checkout-file",
        "Check out a file from the git status buffer.",
        git_status_checkout_file_command,
    ),
    (
        "git.status.discard-or-reset",
        "Delete selected git status targets or reset the commit at point.",
        git_status_discard_or_reset_command,
    ),
];

pub(super) fn register_git_status_commands(runtime: &mut EditorRuntime) -> Result<(), String> {
    for &(name, description, handler) in GIT_STATUS_COMMANDS {
        runtime
            .register_command(name, description, CommandSource::Core, handler)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(super) fn active_git_status_command_context(
    runtime: &EditorRuntime,
) -> Result<GitStatusCommandContext, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_git_status(&buffer.kind) {
        return Err("git status buffer is not active".to_owned());
    }
    let snapshot = buffer.git_snapshot();
    Ok(GitStatusCommandContext {
        buffer_id,
        meta: buffer
            .section_line_meta(buffer.cursor_point().line)
            .cloned(),
        staged_empty: snapshot
            .map(|snapshot| snapshot.staged().is_empty())
            .unwrap_or(true),
        has_stage_candidates: snapshot
            .map(|snapshot| !(snapshot.unstaged().is_empty() && snapshot.untracked().is_empty()))
            .unwrap_or(false),
    })
}

pub(super) fn ensure_no_rebase_in_progress(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_rebase_in_progress(runtime)? {
        return Err("rebase already in progress".to_owned());
    }
    Ok(())
}

pub(super) fn ensure_rebase_in_progress(
    runtime: &mut EditorRuntime,
    message: &str,
) -> Result<(), String> {
    if !git_rebase_in_progress(runtime)? {
        return Err(message.to_owned());
    }
    Ok(())
}

pub(super) fn git_status_sequence_kind(
    runtime: &mut EditorRuntime,
    message: &str,
) -> Result<GitSequenceKind, String> {
    git_sequence_in_progress(runtime)?.ok_or_else(|| message.to_owned())
}

pub(super) fn unsupported_git_status_command(message: &str) -> Result<(), String> {
    Err(message.to_owned())
}

pub(super) fn git_status_refresh_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    refresh_git_status_buffer(runtime, context.buffer_id)
}

pub(super) fn git_status_next_section_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    move_git_section(runtime, true).map(|_| ())
}

pub(super) fn git_status_previous_section_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    move_git_section(runtime, false).map(|_| ())
}

pub(super) fn git_status_stage_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    let (targets, is_visual) =
        git_status_action_targets(runtime, context.buffer_id, GIT_ACTION_STAGE_FILE)?;
    if !targets.is_empty() {
        stage_git_files(runtime, &targets)?;
        if is_visual {
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        return Ok(());
    }
    if is_visual {
        return Err("no stageable files selected".to_owned());
    }
    if !context.has_stage_candidates {
        return Err("no unstaged changes to stage".to_owned());
    }
    stage_git_all(runtime)
}

pub(super) fn git_status_stage_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if !context.has_stage_candidates {
        return Err("no unstaged changes to stage".to_owned());
    }
    stage_git_all(runtime)
}

pub(super) fn git_status_unstage_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    let (targets, is_visual) =
        git_status_action_targets(runtime, context.buffer_id, GIT_ACTION_UNSTAGE_FILE)?;
    if !targets.is_empty() {
        unstage_git_files(runtime, &targets)?;
        if is_visual {
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        return Ok(());
    }
    if is_visual {
        return Err("no staged files selected".to_owned());
    }
    Ok(())
}

pub(super) fn git_status_unstage_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if context.staged_empty {
        return Err("no staged changes to unstage".to_owned());
    }
    unstage_git_all(runtime)
}

pub(super) fn git_status_commit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if context.staged_empty {
        return Err("no staged changes to commit".to_owned());
    }
    open_git_commit_buffer(runtime)
}

pub(super) fn git_status_push_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    push_git_to_pushremote(runtime, context.buffer_id)
}

pub(super) fn git_status_push_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    push_git_to_upstream(runtime, context.buffer_id)
}

pub(super) fn git_status_fetch_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    fetch_git_pushremote(runtime, context.buffer_id)
}

pub(super) fn git_status_fetch_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    fetch_git_upstream(runtime, context.buffer_id)
}

pub(super) fn git_status_fetch_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    fetch_git_all(runtime)
}

pub(super) fn git_status_pull_upstream_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    pull_git_upstream(runtime, context.buffer_id)
}

pub(super) fn git_status_branches_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker(runtime)
}

pub(super) fn git_worktree_create_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_worktree_branch_picker(runtime)
}

fn trace_oil_worktree(runtime: &mut EditorRuntime, message: impl Into<String>) {
    let message = message.into();
    eprintln!("[oil.git-worktree] {message}");
    record_runtime_error(runtime, "oil.git-worktree.trace", message);
}

pub(super) fn oil_git_worktree_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    trace_oil_worktree(runtime, "oil.git-worktree command invoked");
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (buffer_name, buffer_kind, is_directory) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        (
            buffer.name.clone(),
            buffer_kind_label(&buffer.kind),
            buffer_is_directory(&buffer.kind),
        )
    };
    trace_oil_worktree(
        runtime,
        format!("active buffer `{buffer_name}` kind `{buffer_kind}`"),
    );
    if !is_directory {
        return Err("oil.git-worktree requires an oil buffer".to_owned());
    }
    begin_oil_worktree_request(runtime, buffer_id)
}

pub(super) fn git_status_merge_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_merge_in_progress(runtime)? {
        return merge_git_continue(runtime);
    }
    open_git_branch_picker_with_action(runtime, "Git Merge", GitBranchActionKind::MergePlain)
}

pub(super) fn git_status_merge_edit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Edit Message)",
        GitBranchActionKind::MergeEdit,
    )
}

pub(super) fn git_status_merge_no_commit_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (No Commit)",
        GitBranchActionKind::MergeNoCommit,
    )
}

pub(super) fn git_status_merge_squash_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Squash)",
        GitBranchActionKind::MergeSquash,
    )
}

pub(super) fn git_status_merge_preview_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_branch_picker_with_action(
        runtime,
        "Git Merge (Preview)",
        GitBranchActionKind::MergePreview,
    )
}

pub(super) fn git_status_merge_abort_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    merge_git_abort(runtime)
}

pub(super) fn git_status_rebase_pushremote_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    let context = active_git_status_command_context(runtime)?;
    rebase_git_onto_pushremote(runtime, context.buffer_id)
}

pub(super) fn git_status_rebase_upstream_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    let context = active_git_status_command_context(runtime)?;
    rebase_git_onto_upstream(runtime, context.buffer_id)
}

pub(super) fn git_status_rebase_onto_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    if git_rebase_in_progress(runtime)? {
        return rebase_git_edit_todo(runtime);
    }
    open_git_branch_picker_with_action(runtime, "Git Rebase", GitBranchActionKind::RebaseOnto)
}

pub(super) fn git_status_rebase_interactive_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_no_rebase_in_progress(runtime)?;
    open_git_branch_picker_with_action(
        runtime,
        "Git Rebase (Interactive)",
        GitBranchActionKind::RebaseInteractive,
    )
}

pub(super) fn git_status_rebase_continue_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "no rebase in progress")?;
    rebase_git_continue(runtime)
}

pub(super) fn git_status_rebase_skip_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "rebase subset is not supported yet")?;
    rebase_git_skip(runtime)
}

pub(super) fn git_status_rebase_abort_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    ensure_rebase_in_progress(runtime, "no rebase in progress")?;
    rebase_git_abort(runtime)
}

pub(super) fn git_status_rebase_autosquash_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase autosquash is not supported yet")
}

pub(super) fn git_status_rebase_edit_commit_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase edit-commit is not supported yet")
}

pub(super) fn git_status_rebase_reword_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase reword is not supported yet")
}

pub(super) fn git_status_rebase_remove_commit_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("rebase remove-commit is not supported yet")
}

pub(super) fn git_status_diff_dwim_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_dwim(runtime, context.buffer_id, context.meta.as_ref(), "")
}

pub(super) fn git_status_diff_staged_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_diff_staged(runtime)
}

pub(super) fn git_status_diff_unstaged_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_diff_unstaged(runtime)
}

pub(super) fn git_status_diff_commit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_commit_at_point(runtime, context.buffer_id, context.meta.as_ref())
}

pub(super) fn git_status_diff_stash_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    diff_git_stash_at_point(runtime, context.meta.as_ref())
}

pub(super) fn git_status_diff_range_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git diff range is not supported yet")
}

pub(super) fn git_status_diff_paths_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git diff paths is not supported yet")
}

pub(super) fn git_status_log_head_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_head(runtime)
}

pub(super) fn git_status_log_related_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    open_git_log_related(runtime, context.buffer_id)
}

pub(super) fn git_status_log_other_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("git log other is not supported yet")
}

pub(super) fn git_status_log_branches_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_branches(runtime)
}

pub(super) fn git_status_log_all_branches_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    open_git_log_all_branches(runtime)
}

pub(super) fn git_status_log_all_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    open_git_log_all(runtime)
}

pub(super) fn git_status_stash_both_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_both(runtime)
}

pub(super) fn git_status_stash_index_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_index(runtime)
}

pub(super) fn git_status_stash_worktree_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    stash_git_worktree(runtime)
}

pub(super) fn git_status_stash_keep_index_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    stash_git_keep_index(runtime)
}

pub(super) fn git_status_stash_apply_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    stash_git_apply_at_point(runtime, context.meta.as_ref())
}

pub(super) fn git_status_stash_pop_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    stash_git_pop_at_point(runtime, context.meta.as_ref())
}

pub(super) fn git_status_stash_drop_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    stash_git_drop_at_point(runtime, context.meta.as_ref())
}

pub(super) fn git_status_stash_show_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    stash_git_show_at_point(runtime, context.meta.as_ref())
}

pub(super) fn git_status_cherry_open_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    open_git_cherry_buffer(runtime, context.buffer_id)
}

pub(super) fn git_status_cherry_pick_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if let Some(kind) = git_sequence_in_progress(runtime)? {
        return sequence_git_continue(runtime, kind);
    }
    cherry_pick_commit_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(super) fn git_status_cherry_pick_apply_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if let Some(kind) = git_sequence_in_progress(runtime)? {
        return sequence_git_abort(runtime, kind);
    }
    cherry_pick_apply_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(super) fn git_status_cherry_pick_skip_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let kind =
        git_status_sequence_kind(runtime, "cherry-pick move commands are not supported yet")?;
    sequence_git_skip(runtime, kind)
}

pub(super) fn git_status_revert_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if let Some(kind) = git_sequence_in_progress(runtime)? {
        return sequence_git_continue(runtime, kind);
    }
    revert_commit_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(super) fn git_status_revert_no_commit_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    if let Some(kind) = git_sequence_in_progress(runtime)? {
        return sequence_git_abort(runtime, kind);
    }
    revert_no_commit_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(super) fn git_status_revert_skip_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let kind = git_status_sequence_kind(runtime, "no cherry-pick or revert in progress")?;
    sequence_git_skip(runtime, kind)
}

pub(super) fn git_status_revert_abort_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let kind = git_status_sequence_kind(runtime, "no cherry-pick or revert in progress")?;
    sequence_git_abort(runtime, kind)
}

pub(super) fn git_status_apply_commit_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    cherry_pick_apply_at_point_or_picker(runtime, context.meta.as_ref())
}

pub(super) fn git_status_reset_mixed_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Mixed)
}

pub(super) fn git_status_reset_soft_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Soft)
}

pub(super) fn git_status_reset_hard_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Hard)
}

pub(super) fn git_status_reset_keep_command(runtime: &mut EditorRuntime) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Keep)
}

pub(super) fn git_status_reset_index_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("reset index is not supported yet")
}

pub(super) fn git_status_reset_worktree_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("reset worktree is not supported yet")
}

pub(super) fn git_status_checkout_file_command(_: &mut EditorRuntime) -> Result<(), String> {
    unsupported_git_status_command("file checkout is not supported yet")
}

pub(super) fn git_status_discard_or_reset_command(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let context = active_git_status_command_context(runtime)?;
    let (targets, is_visual) = git_status_delete_targets(runtime, context.buffer_id)?;
    if !targets.is_empty() {
        delete_git_status_targets(runtime, &targets)?;
        if is_visual {
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        return Ok(());
    }
    if is_visual {
        return Err("no deletable files selected".to_owned());
    }
    reset_commit_at_point_or_picker(runtime, context.meta.as_ref(), GitResetMode::Mixed)
}

pub(super) fn git_status_command_name(
    user_library: &dyn UserLibrary,
    prefix: Option<GitPrefix>,
    chord: &str,
) -> Option<&'static str> {
    user_library.git_command_for_chord(prefix, chord)
}

pub(super) fn take_directory_prefix(runtime: &mut EditorRuntime) -> Result<Option<String>, String> {
    const PREFIX_TIMEOUT: Duration = Duration::from_millis(1200);
    let now = Instant::now();
    let ui = shell_ui_mut(runtime)?;
    let pending = match ui.pending_directory_prefix.take() {
        Some(state) if now.duration_since(state.started_at) <= PREFIX_TIMEOUT => Some(state.chord),
        _ => None,
    };
    Ok(pending)
}

pub(super) fn set_directory_prefix(runtime: &mut EditorRuntime, chord: &str) -> Result<(), String> {
    shell_ui_mut(runtime)?.pending_directory_prefix = Some(DirectoryPrefixState {
        chord: chord.to_owned(),
        started_at: Instant::now(),
    });
    Ok(())
}

pub(super) enum TakeKeySequence {
    /// Live pending tokens for this scope.
    Live(PendingKeySequence),
    /// Ambiguous short already timed out — fire it before handling the new token.
    FireShort {
        chord: String,
        vim_mode: KeymapVimMode,
    },
    /// No pending sequence for this scope.
    None,
}

pub(super) fn take_key_sequence(
    runtime: &mut EditorRuntime,
    scope: &KeymapScope,
    options: &KeySequenceOptions,
) -> Result<TakeKeySequence, String> {
    let now = Instant::now();
    let ui = shell_ui_mut(runtime)?;
    let state = ui.pending_key_sequence.take();
    match state {
        Some(state) if &state.scope == scope => {
            let elapsed_ms =
                u64::try_from(now.duration_since(state.started_at).as_millis()).unwrap_or(u64::MAX);
            let pending = PendingKeySequence {
                tokens: state.tokens,
                started_at_ms: 0,
                ambiguous_short: state.ambiguous_short,
            };
            match tick_key_sequence(&pending, elapsed_ms, options) {
                KeySequenceTick::Pending => Ok(TakeKeySequence::Live(pending)),
                KeySequenceTick::Execute { chord } => Ok(TakeKeySequence::FireShort {
                    chord,
                    vim_mode: state.vim_mode,
                }),
                KeySequenceTick::Expired => Ok(TakeKeySequence::None),
            }
        }
        Some(state) => {
            ui.pending_key_sequence = Some(state);
            Ok(TakeKeySequence::None)
        }
        None => Ok(TakeKeySequence::None),
    }
}

pub(super) fn set_key_sequence(
    runtime: &mut EditorRuntime,
    scope: KeymapScope,
    vim_mode: KeymapVimMode,
    pending: PendingKeySequence,
) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    ui.pending_key_sequence = Some(KeySequenceState {
        scope,
        vim_mode,
        tokens: pending.tokens,
        started_at: Instant::now(),
        ambiguous_short: pending.ambiguous_short,
    });
    Ok(())
}

pub(super) fn clear_key_sequence(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.pending_key_sequence = None;
    Ok(())
}

pub(super) fn peek_key_sequence_tick(
    runtime: &EditorRuntime,
    options: &KeySequenceOptions,
) -> Result<Option<(KeymapScope, KeymapVimMode, KeySequenceTick)>, String> {
    let ui = shell_ui(runtime)?;
    let Some(state) = ui.pending_key_sequence.as_ref() else {
        return Ok(None);
    };
    let now = Instant::now();
    let elapsed_ms =
        u64::try_from(now.duration_since(state.started_at).as_millis()).unwrap_or(u64::MAX);
    let pending = PendingKeySequence {
        tokens: state.tokens.clone(),
        started_at_ms: 0,
        ambiguous_short: state.ambiguous_short.clone(),
    };
    Ok(Some((
        state.scope.clone(),
        state.vim_mode,
        tick_key_sequence(&pending, elapsed_ms, options),
    )))
}

pub(super) fn move_git_section(runtime: &mut EditorRuntime, forward: bool) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (start_line, line_count) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_git_status(&buffer.kind) {
            return Ok(false);
        }
        (buffer.cursor_point().line, buffer.line_count())
    };
    if line_count == 0 {
        return Ok(false);
    }
    if forward {
        for line in start_line.saturating_add(1)..line_count {
            if let Some(meta) = shell_buffer(runtime, buffer_id)?.section_line_meta(line)
                && matches!(meta.kind, SectionRenderLineKind::Header { .. })
            {
                shell_buffer_mut(runtime, buffer_id)?.goto_line(line);
                return Ok(true);
            }
        }
    } else {
        let mut line = start_line;
        while line > 0 {
            line = line.saturating_sub(1);
            if let Some(meta) = shell_buffer(runtime, buffer_id)?.section_line_meta(line)
                && matches!(meta.kind, SectionRenderLineKind::Header { .. })
            {
                shell_buffer_mut(runtime, buffer_id)?.goto_line(line);
                return Ok(true);
            }
            if line == 0 {
                break;
            }
        }
    }
    Ok(false)
}

pub(super) fn toggle_git_section(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (section_id, snapshot) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_git_status(&buffer.kind) {
            return Ok(false);
        }
        let meta = buffer
            .section_line_meta(buffer.cursor_point().line)
            .cloned();
        let section_id = match meta.as_ref().map(|meta| &meta.kind) {
            Some(SectionRenderLineKind::Header { id, .. }) => id.clone(),
            _ => return Ok(false),
        };
        let snapshot = buffer
            .git_snapshot()
            .cloned()
            .ok_or_else(|| "git status snapshot is missing".to_owned())?;
        (section_id, snapshot)
    };
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let state = buffer.ensure_section_state();
        state.collapsed.toggle(&section_id);
    }
    apply_git_status_snapshot(runtime, buffer_id, snapshot)?;
    Ok(true)
}

pub(super) fn handle_git_status_tab(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let meta = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_git_status(&buffer.kind) {
            return Ok(false);
        }
        buffer
            .section_line_meta(buffer.cursor_point().line)
            .cloned()
    };
    if matches!(
        meta.as_ref().map(|meta| &meta.kind),
        Some(SectionRenderLineKind::Header { .. })
    ) {
        return toggle_git_section(runtime);
    }
    diff_git_dwim(runtime, buffer_id, meta.as_ref(), "")?;
    Ok(true)
}

pub(super) fn handle_git_status_chord(
    runtime: &mut EditorRuntime,
    chord: &str,
) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_git_status(&buffer.kind) {
            return Ok(false);
        }
    }

    let prefix = take_git_prefix(runtime)?;
    let user_library = shell_user_library(runtime);
    if let Some(command_name) = git_status_command_name(&*user_library, prefix, chord)
        .or_else(|| git_status_command_name(&*user_library, None, chord))
    {
        runtime
            .execute_command(command_name)
            .map_err(|error| error.to_string())?;
        return Ok(true);
    }

    if let Some(prefix) = user_library.git_prefix_for_chord(chord) {
        set_git_prefix(runtime, prefix)?;
        return Ok(true);
    }
    Ok(false)
}

pub(super) fn handle_git_view_chord(
    runtime: &mut EditorRuntime,
    chord: &str,
) -> Result<bool, String> {
    if chord != "g" {
        return Ok(false);
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let view = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let is_git_view = matches!(
            &buffer.kind,
            BufferKind::Plugin(plugin_kind)
                if plugin_kind == GIT_DIFF_KIND
                    || plugin_kind == GIT_LOG_KIND
                    || plugin_kind == GIT_STASH_KIND
        );
        if !is_git_view {
            return Ok(false);
        }
        buffer
            .git_view()
            .cloned()
            .ok_or_else(|| "git view state is missing".to_owned())?
    };
    apply_git_view(runtime, buffer_id, view)?;
    Ok(true)
}

pub(super) fn refresh_pending_git_summary(
    runtime: &mut EditorRuntime,
    now: Instant,
    typing_active: bool,
) -> Result<(), String> {
    if shell_ui(runtime)?.take_git_summary_changed() {
        mark_git_fringe_snapshots_stale(runtime)?;
    }
    if typing_active {
        return Ok(());
    }
    let summary_state = {
        let ui = shell_ui_mut(runtime)?;
        if !ui.git_summary_refresh_due(now) {
            return Ok(());
        }
        let summary_state = ui.git_summary_state();
        if !summary_state.try_begin_refresh() {
            return Ok(());
        }
        ui.mark_git_summary_refreshed(now);
        summary_state
    };
    let root = match git_root(runtime) {
        Ok(root) => root,
        Err(_) => {
            if let Ok(ui) = shell_ui(runtime) {
                ui.clear_git_summary();
            }
            summary_state.finish_refresh();
            return Ok(());
        }
    };

    std::thread::spawn(move || {
        let snapshot = build_git_summary_snapshot(&root);
        summary_state.set_snapshot(snapshot);
        summary_state.finish_refresh();
    });

    Ok(())
}

pub(super) fn mark_git_fringe_snapshots_stale(runtime: &mut EditorRuntime) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    for buffer in &mut ui.buffers {
        buffer.mark_git_fringe_stale();
    }
    Ok(())
}

pub(super) fn refresh_git_fringe(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = match git_root(runtime) {
        Ok(root) => root,
        Err(_) => {
            if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                buffer.clear_git_fringe_dirty();
            }
            return Ok(());
        }
    };
    let (path, line_count, fringe_state) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let Some(path) = buffer.path() else {
            return Ok(());
        };
        let Some(fringe_state) = buffer.git_fringe_state().cloned() else {
            return Ok(());
        };
        (path.to_path_buf(), buffer.line_count(), fringe_state)
    };
    let relative_path = match path.strip_prefix(&root) {
        Ok(relative) => relative.to_path_buf(),
        Err(_) => {
            fringe_state.update_snapshot(GitFringeSnapshot::default());
            if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                buffer.clear_git_fringe_dirty();
            }
            return Ok(());
        }
    };
    if !fringe_state.try_begin_refresh() {
        return Ok(());
    }
    let text_snapshot = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer.text.snapshot()
    };
    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
        buffer.clear_git_fringe_dirty();
    }

    std::thread::spawn(move || {
        let buffer_text = text_snapshot.text();
        let snapshot = if git_repository_present(&root) {
            build_git_fringe_snapshot(&root, &relative_path, &buffer_text, line_count)
        } else {
            GitFringeSnapshot::default()
        };
        fringe_state.update_snapshot(snapshot);
        fringe_state.finish_refresh();
    });

    Ok(())
}

pub(super) fn build_git_fringe_snapshot(
    root: &Path,
    relative_path: &Path,
    buffer_text: &str,
    line_count: usize,
) -> GitFringeSnapshot {
    if line_count == 0 {
        return GitFringeSnapshot::default();
    }
    let relative_spec = relative_path.to_string_lossy().replace('\\', "/");
    let head_spec = format!("HEAD:{relative_spec}");
    let head_text = git_command_output_background(root, &["show", &head_spec], &[0]);
    let Some(head_text) = head_text else {
        let mut snapshot = GitFringeSnapshot::default();
        for line_index in 0..line_count {
            snapshot.lines.insert(line_index, GitFringeKind::Added);
        }
        return snapshot;
    };
    let normalized_head_text = normalize_git_fringe_text(&head_text);
    let normalized_buffer_text = normalize_git_fringe_text(buffer_text);
    if normalized_head_text == normalized_buffer_text {
        return GitFringeSnapshot::default();
    }
    let head_path = git_fringe_temp_path("head");
    let buffer_path = git_fringe_temp_path("buffer");
    if fs::write(&head_path, normalized_head_text).is_err()
        || fs::write(&buffer_path, normalized_buffer_text).is_err()
    {
        let _ = fs::remove_file(&head_path);
        let _ = fs::remove_file(&buffer_path);
        return GitFringeSnapshot::default();
    }
    let head_path_str = head_path.to_string_lossy().to_string();
    let buffer_path_str = buffer_path.to_string_lossy().to_string();
    let diff_output = git_command_output_background(
        root,
        &[
            "diff",
            "--no-index",
            "--unified=0",
            "--no-color",
            head_path_str.as_str(),
            buffer_path_str.as_str(),
        ],
        &[0, 1],
    )
    .unwrap_or_default();
    let _ = fs::remove_file(&head_path);
    let _ = fs::remove_file(&buffer_path);
    parse_git_fringe_diff(&diff_output, line_count)
}

fn normalize_git_fringe_text(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            normalized.push('\n');
        } else {
            normalized.push(ch);
        }
    }
    normalized
}

pub(super) fn parse_git_fringe_diff(diff_output: &str, line_count: usize) -> GitFringeSnapshot {
    let mut snapshot = GitFringeSnapshot::default();
    if line_count == 0 {
        return snapshot;
    }
    for line in diff_output.lines() {
        let Some((_old_start, old_count, new_start, new_count)) = parse_diff_hunk_header(line)
        else {
            continue;
        };
        apply_git_fringe_hunk(&mut snapshot, line_count, old_count, new_start, new_count);
    }
    snapshot
}

pub(super) fn apply_git_fringe_hunk(
    snapshot: &mut GitFringeSnapshot,
    line_count: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
) {
    if line_count == 0 {
        return;
    }
    let start_index = new_start.saturating_sub(1);
    if old_count == 0 {
        let end = start_index.saturating_add(new_count).min(line_count);
        for line_index in start_index..end {
            snapshot.lines.insert(line_index, GitFringeKind::Added);
        }
    } else if new_count == 0 {
        let line_index = start_index.min(line_count.saturating_sub(1));
        snapshot.lines.insert(line_index, GitFringeKind::Removed);
    } else {
        let end = start_index.saturating_add(new_count).min(line_count);
        for line_index in start_index..end {
            snapshot.lines.insert(line_index, GitFringeKind::Modified);
        }
    }
}

pub(super) fn parse_diff_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
    let trimmed = line.strip_prefix("@@")?.trim();
    let mut parts = trimmed.split_whitespace();
    let old_part = parts.next()?;
    let new_part = parts.next()?;
    let (old_start, old_count) = parse_hunk_range(old_part)?;
    let (new_start, new_count) = parse_hunk_range(new_part)?;
    Some((old_start, old_count, new_start, new_count))
}

pub(super) fn parse_hunk_range(part: &str) -> Option<(usize, usize)> {
    let part = part.strip_prefix('-').or_else(|| part.strip_prefix('+'))?;
    let mut pieces = part.split(',');
    let start = pieces.next()?.parse::<usize>().ok()?;
    let count = match pieces.next() {
        Some(raw) => raw.parse::<usize>().ok()?,
        None => 1,
    };
    Some((start, count))
}

pub(super) fn build_git_summary_snapshot(root: &Path) -> Option<GitSummarySnapshot> {
    let branch_output =
        git_command_output_background(root, &["rev-parse", "--abbrev-ref", "HEAD"], &[0])?;
    let head = git_command_output_background(root, &["rev-parse", "--verify", "HEAD"], &[0])
        .map(|head| head.trim().to_owned())
        .filter(|head| !head.is_empty());
    let branch = branch_output.trim();
    if branch.is_empty() {
        return None;
    }
    let diff_output = git_command_output_background(root, &["diff", "--numstat", "HEAD"], &[0, 1])
        .unwrap_or_default();
    let (added, removed) = parse_git_numstat(&diff_output);
    Some(GitSummarySnapshot {
        branch: Some(branch.to_owned()),
        head,
        added,
        removed,
    })
}

pub(super) fn parse_git_numstat(output: &str) -> (usize, usize) {
    let mut added = 0usize;
    let mut removed = 0usize;
    for line in output.lines() {
        let mut parts = line.split('\t');
        let add_raw = parts.next().unwrap_or_default();
        let remove_raw = parts.next().unwrap_or_default();
        added = added.saturating_add(add_raw.parse::<usize>().unwrap_or(0));
        removed = removed.saturating_add(remove_raw.parse::<usize>().unwrap_or(0));
    }
    (added, removed)
}

pub(super) fn git_command_output_background(
    root: &Path,
    args: &[&str],
    allowed_exit_codes: &[i32],
) -> Option<String> {
    let output = run_direct_git_command(root, args).ok()?;
    let exit_code = output.status.code()?;
    if exit_code != 0 && !allowed_exit_codes.contains(&exit_code) {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(super) fn git_repository_present(root: &Path) -> bool {
    git_command_output_background(root, &["rev-parse", "--git-dir"], &[0]).is_some()
}

pub(super) fn git_fringe_temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    env::temp_dir().join(format!(
        "volt-git-fringe-{label}-{}-{unique}.tmp",
        std::process::id()
    ))
}

pub(super) fn git_command_output(
    runtime: &mut EditorRuntime,
    root: &Path,
    label: &str,
    args: &[&str],
) -> Result<String, String> {
    let spec = JobSpec::command(
        label,
        "git",
        args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
    )
    .with_cwd(root.to_path_buf());
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
    if !result.succeeded() {
        return Err(format!("git {label} failed: {}", result.transcript()));
    }
    Ok(result.stdout().to_owned())
}

pub(super) fn git_command_output_owned(
    runtime: &mut EditorRuntime,
    root: &Path,
    label: &str,
    args: &[String],
) -> Result<String, String> {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    git_command_output(runtime, root, label, &refs)
}

fn git_read_command_output(root: &Path, label: &str, args: &[&str]) -> Result<String, String> {
    git_read_command_output_allow_exit_codes(root, label, args, &[0])
}

fn git_read_command_output_optional(root: &Path, label: &str, args: &[&str]) -> Option<String> {
    git_read_command_output(root, label, args).ok()
}

fn git_read_log_oneline_optional(root: &Path, label: &str, revision: &str) -> Vec<GitLogEntry> {
    let limit = GIT_LOG_LIMIT.to_string();
    git_read_command_output_optional(root, label, &["log", "-n", &limit, "--oneline", revision])
        .map(|output| parse_log_oneline(&output))
        .unwrap_or_default()
}

fn git_read_command_output_allow_exit_codes(
    root: &Path,
    label: &str,
    args: &[&str],
    allowed_exit_codes: &[i32],
) -> Result<String, String> {
    let output = run_direct_git_command(root, args)?;
    let exit_code = output.status.code().ok_or_else(|| {
        format!(
            "git {label} failed to return an exit code: {}",
            command_output_transcript(&output)
        )
    })?;
    if exit_code != 0 && !allowed_exit_codes.contains(&exit_code) {
        return Err(format!(
            "git {label} failed: {}",
            command_output_transcript(&output)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_direct_git_command(root: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    let mut command = Command::new("git");
    configure_background_command(&mut command);
    command
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            format!(
                "failed to run git {:?} in {}: {error}",
                args,
                root.display()
            )
        })
}

fn command_output_transcript(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.is_empty() {
        stdout.into_owned()
    } else if stdout.is_empty() {
        stderr.into_owned()
    } else {
        format!("{stdout}{stderr}")
    }
}

pub(super) fn git_dir_path(_runtime: &mut EditorRuntime, root: &Path) -> Option<PathBuf> {
    let output =
        git_read_command_output_optional(root, "rev-parse --git-dir", &["rev-parse", "--git-dir"])?;
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = normalize_git_output_path(trimmed);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(root.join(path))
    }
}

pub(super) fn git_status_snapshot(
    runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<GitStatusSnapshot, String> {
    let status_output = git_read_command_output(
        root,
        "status --short --branch",
        &["status", "--short", "--branch"],
    )?;
    let status = parse_status(&status_output).map_err(|error| error.to_string())?;

    let recent_output = git_read_command_output_optional(
        root,
        "log --oneline",
        &["log", "-n", &GIT_LOG_LIMIT.to_string(), "--oneline"],
    )
    .unwrap_or_default();
    let recent = parse_log_oneline(&recent_output);
    let head = recent.first().cloned();
    let head_exists = head.is_some();

    let upstream = git_read_command_output_optional(
        root,
        "rev-parse --abbrev-ref @{upstream}",
        &["rev-parse", "--abbrev-ref", "@{upstream}"],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
    .or_else(|| status_output_upstream(&status_output));
    let push_remote = git_read_command_output_optional(
        root,
        "rev-parse --abbrev-ref @{push}",
        &["rev-parse", "--abbrev-ref", "@{push}"],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
    .or_else(|| upstream.clone());
    let tag = git_head_tag(root);

    let stash_output = git_read_command_output_optional(root, "stash list", &["stash", "list"])
        .unwrap_or_default();
    let stashes = parse_stash_list(&stash_output);

    let unpulled = if head_exists && upstream.is_some() {
        git_read_log_oneline_optional(root, "log --oneline ..@{upstream}", "..@{upstream}")
    } else {
        Vec::new()
    };
    let unpushed = if head_exists && upstream.is_some() {
        git_read_log_oneline_optional(root, "log --oneline @{upstream}..", "@{upstream}..")
    } else {
        Vec::new()
    };

    let in_progress = git_dir_path(runtime, root)
        .map(detect_in_progress)
        .unwrap_or_default();

    Ok(GitStatusSnapshot::default()
        .with_status(status)
        .with_head(head)
        .with_upstreams(upstream, push_remote)
        .with_tag(tag)
        .with_stashes(stashes)
        .with_unpulled(unpulled)
        .with_unpushed(unpushed)
        .with_recent(recent)
        .with_in_progress(in_progress))
}

pub(super) fn git_remote_list(
    _runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<Vec<String>, String> {
    let output = git_read_command_output(root, "remote", &["remote"])?;
    let mut remotes = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();
    remotes.sort();
    remotes.dedup();
    Ok(remotes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_git::GitStatusSnapshot;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn summary(branch: &str, head: &str, added: usize, removed: usize) -> GitSummarySnapshot {
        GitSummarySnapshot {
            branch: Some(branch.to_owned()),
            head: Some(head.to_owned()),
            added,
            removed,
        }
    }

    #[test]
    fn git_summary_changed_tracks_head_updates() {
        let state = GitSummaryState::new();
        assert!(!state.take_changed());

        let first = Some(summary("main", "abc123", 1, 0));
        state.set_snapshot(first.clone());
        assert!(state.take_changed());
        assert!(!state.take_changed());

        state.set_snapshot(first);
        assert!(!state.take_changed());

        state.set_snapshot(Some(summary("main", "def456", 0, 0)));
        assert!(state.take_changed());
    }

    #[cfg(windows)]
    #[test]
    fn git_worktree_list_parser_normalizes_windows_drive_paths() {
        let entries = parse_git_worktree_list(
            "worktree w:/w/ftc-ui-web\nHEAD abc123\nbranch refs/heads/main\n\n",
        )
        .expect("worktree list parses");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, PathBuf::from(r"W:\w\ftc-ui-web"));
    }

    fn temp_dir() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-shell-git-{unique}"))
    }

    #[test]
    fn git_push_remote_name_prefers_branch_push_remote_for_slashy_branch_names() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("temp dir");
        git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
        git_read_command_output(
            &root,
            "config user.email",
            &["config", "user.email", "volt-tests@example.com"],
        )
        .expect("user email");
        git_read_command_output(
            &root,
            "config user.name",
            &["config", "user.name", "Volt Tests"],
        )
        .expect("user name");
        git_read_command_output(
            &root,
            "checkout branch",
            &["checkout", "-qb", "feature/TASK-1234-abc"],
        )
        .expect("branch");
        git_read_command_output(
            &root,
            "config pushRemote",
            &[
                "config",
                "branch.feature/TASK-1234-abc.pushRemote",
                "origin",
            ],
        )
        .expect("pushRemote");

        let snapshot = GitStatusSnapshot::default()
            .with_upstreams(
                Some("origin/feature/TASK-1234-abc".to_owned()),
                Some("feature/TASK-1234-abc".to_owned()),
            )
            .with_status(editor_git::RepositoryStatus::new(
                Some("feature/TASK-1234-abc".to_owned()),
                0,
                0,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));

        assert_eq!(
            git_push_remote_name(&root, &snapshot),
            Some("origin".to_owned())
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn git_push_remote_name_uses_upstream_branch_remote_for_local_tracking_worktree_branch() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("temp dir");
        git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
        git_read_command_output(
            &root,
            "config branch feature remote",
            &["config", "branch.feature/TASK-1234-abc.remote", "origin"],
        )
        .expect("feature remote");
        git_read_command_output(
            &root,
            "config branch feature merge",
            &[
                "config",
                "branch.feature/TASK-1234-abc.merge",
                "refs/heads/feature/TASK-1234-abc",
            ],
        )
        .expect("feature merge");
        git_read_command_output(
            &root,
            "config branch abc remote",
            &["config", "branch.abc.remote", "."],
        )
        .expect("abc remote");
        git_read_command_output(
            &root,
            "config branch abc merge",
            &[
                "config",
                "branch.abc.merge",
                "refs/heads/feature/TASK-1234-abc",
            ],
        )
        .expect("abc merge");

        let snapshot = GitStatusSnapshot::default()
            .with_upstreams(Some("feature/TASK-1234-abc".to_owned()), None)
            .with_status(editor_git::RepositoryStatus::new(
                Some("abc".to_owned()),
                0,
                0,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));

        assert_eq!(
            git_push_remote_name(&root, &snapshot),
            Some("origin".to_owned())
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn status_output_upstream_reads_short_branch_tracking_ref() {
        assert_eq!(
            status_output_upstream("## master...origin/master [ahead 2, behind 1]\n M file.rs\n"),
            Some("origin/master".to_owned())
        );
        assert_eq!(status_output_upstream("## master\n"), None);
    }

    #[test]
    fn git_read_log_oneline_optional_returns_empty_for_unknown_revision() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("temp dir");
        git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");

        let entries =
            git_read_log_oneline_optional(&root, "log --oneline ..@{upstream}", "..@{upstream}");

        assert!(entries.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn git_fringe_snapshot_is_empty_when_buffer_matches_head() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("temp dir");
        git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
        git_read_command_output(
            &root,
            "config user email",
            &["config", "user.email", "volt@example.com"],
        )
        .expect("user email");
        git_read_command_output(
            &root,
            "config user name",
            &["config", "user.name", "Volt Tests"],
        )
        .expect("user name");
        let path = root.join("main.rs");
        fs::write(&path, "fn main() {}").expect("write file");
        git_read_command_output(&root, "add", &["add", "main.rs"]).expect("git add");
        git_read_command_output(&root, "commit", &["commit", "-qm", "init"]).expect("git commit");

        let snapshot = build_git_fringe_snapshot(&root, Path::new("main.rs"), "fn main() {}", 1);
        assert!(snapshot.lines.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn git_fringe_snapshot_ignores_crlf_only_difference() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("temp dir");
        git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
        git_read_command_output(
            &root,
            "config user email",
            &["config", "user.email", "volt@example.com"],
        )
        .expect("user email");
        git_read_command_output(
            &root,
            "config user name",
            &["config", "user.name", "Volt Tests"],
        )
        .expect("user name");
        let path = root.join("main.rs");
        fs::write(&path, "fn main() {\r\n    println!(\"hi\");\r\n}\r\n").expect("write file");
        git_read_command_output(&root, "add", &["add", "main.rs"]).expect("git add");
        git_read_command_output(&root, "commit", &["commit", "-qm", "init"]).expect("git commit");

        let snapshot = build_git_fringe_snapshot(
            &root,
            Path::new("main.rs"),
            "fn main() {\n    println!(\"hi\");\n}\n",
            3,
        );
        assert!(snapshot.lines.is_empty());

        let _ = fs::remove_dir_all(root);
    }
}
