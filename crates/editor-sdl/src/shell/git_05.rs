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
        if let Ok(root) = git_root(runtime) {
            invalidate_repository_file_list_cache_for(&root);
        }
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
    let root = match active_workspace_root(runtime) {
        Ok(Some(root)) => root,
        Ok(None) | Err(_) => {
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
    let blob_cache = {
        let ui = shell_ui(runtime)?;
        ui.git_head_blob_cache()
    };
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
            let probe = git_probe_snapshot(&root);
            build_git_fringe_snapshot_with_cache(
                &root,
                &relative_path,
                &buffer_text,
                line_count,
                probe.head(),
                Some(&blob_cache),
            )
        } else {
            GitFringeSnapshot::default()
        };
        fringe_state.update_snapshot(snapshot);
        fringe_state.finish_refresh();
    });

    Ok(())
}

fn build_git_fringe_snapshot_with_cache(
    root: &Path,
    relative_path: &Path,
    buffer_text: &str,
    line_count: usize,
    head_id: Option<&str>,
    cache: Option<&GitHeadBlobCache>,
) -> GitFringeSnapshot {
    if line_count == 0 {
        return GitFringeSnapshot::default();
    }
    match head_blob_text(root, relative_path, head_id, cache) {
        HeadBlob::Missing => {
            let mut snapshot = GitFringeSnapshot::default();
            for line_index in 0..line_count {
                snapshot.lines.insert(line_index, GitFringeKind::Added);
            }
            snapshot
        }
        HeadBlob::Binary => GitFringeSnapshot::default(),
        HeadBlob::Text(head_text) => {
            git_fringe_snapshot_from_texts(&head_text, buffer_text, line_count)
        }
    }
}

enum HeadBlob {
    Missing,
    Binary,
    Text(String),
}

fn head_blob_text(
    root: &Path,
    relative_path: &Path,
    head_id: Option<&str>,
    cache: Option<&GitHeadBlobCache>,
) -> HeadBlob {
    let relative_spec = relative_path.to_string_lossy().replace('\\', "/");
    let Some(head) = head_id
        .map(str::trim)
        .filter(|head| !head.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            git_command_output_background(root, &["rev-parse", "--verify", "HEAD"], &[0])
                .map(|output| output.trim().to_owned())
                .filter(|output| !output.is_empty())
        })
    else {
        return HeadBlob::Missing;
    };
    if let Some(cache) = cache
        && let Some(text) = cache.get(root, &relative_spec, &head)
    {
        return classify_head_blob(text);
    }
    let spec = format!("{head}:{relative_spec}");
    let Some(text) = git_command_output_background(root, &["show", &spec], &[0]) else {
        return HeadBlob::Missing;
    };
    if let Some(cache) = cache {
        cache.insert(root, &relative_spec, &head, text.clone());
    }
    classify_head_blob(text)
}

fn classify_head_blob(text: String) -> HeadBlob {
    if text.as_bytes().contains(&0) {
        HeadBlob::Binary
    } else {
        HeadBlob::Text(text)
    }
}

pub(super) fn git_fringe_snapshot_from_texts(
    head_text: &str,
    buffer_text: &str,
    line_count: usize,
) -> GitFringeSnapshot {
    if line_count == 0 {
        return GitFringeSnapshot::default();
    }
    let normalized_head_text = normalize_git_fringe_text(head_text);
    let normalized_buffer_text = normalize_git_fringe_text(buffer_text);
    if normalized_head_text == normalized_buffer_text {
        return GitFringeSnapshot::default();
    }
    let old_lines = split_git_fringe_lines(&normalized_head_text);
    let new_lines = split_git_fringe_lines(&normalized_buffer_text);
    let mut snapshot = GitFringeSnapshot::default();
    for (old_count, new_start, new_count) in line_diff_hunks(&old_lines, &new_lines) {
        apply_git_fringe_hunk(&mut snapshot, line_count, old_count, new_start, new_count);
    }
    snapshot
}

fn split_git_fringe_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.lines().collect()
    }
}

fn normalize_git_fringe_text(text: &str) -> String {
    normalize_git_fringe_bytes(text.as_bytes())
}

fn normalize_git_fringe_bytes(bytes: &[u8]) -> String {
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\r' {
            normalized.push(b'\n');
            if bytes.get(index + 1) == Some(&b'\n') {
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        normalized.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(normalized)
        .unwrap_or_else(|error| String::from_utf8_lossy(&error.into_bytes()).into_owned())
}

#[derive(Clone, Copy)]
enum FringeDiffOp {
    Equal,
    Delete,
    Insert,
}

fn line_diff_hunks(old_lines: &[&str], new_lines: &[&str]) -> Vec<(usize, usize, usize)> {
    let n = old_lines.len();
    let m = new_lines.len();
    if n == 0 && m == 0 {
        return Vec::new();
    }
    const MAX_DP_CELLS: usize = 8_000_000;
    let ops = if n.saturating_mul(m) > MAX_DP_CELLS {
        myers_diff_ops(old_lines, new_lines)
    } else {
        lcs_diff_ops(old_lines, new_lines)
    };
    hunks_from_ops(&ops)
}

fn lcs_diff_ops(old_lines: &[&str], new_lines: &[&str]) -> Vec<FringeDiffOp> {
    let n = old_lines.len();
    let m = new_lines.len();
    let mut dp = vec![vec![0u32; m.saturating_add(1)]; n.saturating_add(1)];
    for i in 1..=n {
        for j in 1..=m {
            dp[i][j] = if old_lines[i - 1] == new_lines[j - 1] {
                dp[i - 1][j - 1].saturating_add(1)
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }
    let mut ops = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 && j > 0 {
        if old_lines[i - 1] == new_lines[j - 1] {
            ops.push(FringeDiffOp::Equal);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            ops.push(FringeDiffOp::Delete);
            i -= 1;
        } else {
            ops.push(FringeDiffOp::Insert);
            j -= 1;
        }
    }
    ops.extend(std::iter::repeat_n(FringeDiffOp::Delete, i));
    ops.extend(std::iter::repeat_n(FringeDiffOp::Insert, j));
    ops.reverse();
    ops
}

fn myers_diff_ops(old_lines: &[&str], new_lines: &[&str]) -> Vec<FringeDiffOp> {
    let n = old_lines.len();
    let m = new_lines.len();
    let max = n.saturating_add(m);
    let offset = max as i32;
    let mut v = vec![0i32; max.saturating_mul(2).saturating_add(1)];
    let mut trace = Vec::with_capacity(max.saturating_add(1));
    let mut done_d = 0usize;
    'search: for d in 0..=max {
        for k in (-(d as i32)..=d as i32).step_by(2) {
            let down = k == -(d as i32)
                || (k != d as i32 && v[(k - 1 + offset) as usize] < v[(k + 1 + offset) as usize]);
            let mut x = if down {
                v[(k + 1 + offset) as usize]
            } else {
                v[(k - 1 + offset) as usize] + 1
            };
            let mut y = x - k;
            while x >= 0
                && y >= 0
                && (x as usize) < n
                && (y as usize) < m
                && old_lines[x as usize] == new_lines[y as usize]
            {
                x += 1;
                y += 1;
            }
            v[(k + offset) as usize] = x;
            if x >= n as i32 && y >= m as i32 {
                trace.push(v.clone());
                done_d = d;
                break 'search;
            }
        }
        trace.push(v.clone());
    }

    let mut x = n as i32;
    let mut y = m as i32;
    let mut ops = Vec::new();
    for d in (0..=done_d).rev() {
        let v = &trace[d];
        let k = x - y;
        let down = k == -(d as i32)
            || (k != d as i32 && v[(k - 1 + offset) as usize] < v[(k + 1 + offset) as usize]);
        let prev_k = if down { k + 1 } else { k - 1 };
        let prev_x = if d == 0 {
            0
        } else {
            v[(prev_k + offset) as usize]
        };
        let prev_y = prev_x - prev_k;
        while x > prev_x && y > prev_y {
            ops.push(FringeDiffOp::Equal);
            x -= 1;
            y -= 1;
        }
        if d > 0 {
            if x == prev_x {
                ops.push(FringeDiffOp::Insert);
            } else {
                ops.push(FringeDiffOp::Delete);
            }
            x = prev_x;
            y = prev_y;
        }
    }
    ops.reverse();
    ops
}

fn hunks_from_ops(ops: &[FringeDiffOp]) -> Vec<(usize, usize, usize)> {
    let mut hunks = Vec::new();
    let mut old_count = 0usize;
    let mut new_count = 0usize;
    let mut new_start = 0usize;
    let mut consumed_new = 0usize;
    let mut in_hunk = false;

    let flush = |hunks: &mut Vec<(usize, usize, usize)>,
                 in_hunk: &mut bool,
                 old_count: &mut usize,
                 new_count: &mut usize,
                 new_start: usize| {
        if *in_hunk {
            hunks.push((*old_count, new_start, *new_count));
            *in_hunk = false;
            *old_count = 0;
            *new_count = 0;
        }
    };

    for op in ops {
        match op {
            FringeDiffOp::Equal => {
                flush(
                    &mut hunks,
                    &mut in_hunk,
                    &mut old_count,
                    &mut new_count,
                    new_start,
                );
                consumed_new = consumed_new.saturating_add(1);
            }
            FringeDiffOp::Delete => {
                if !in_hunk {
                    in_hunk = true;
                    new_start = consumed_new.saturating_add(1);
                }
                old_count = old_count.saturating_add(1);
            }
            FringeDiffOp::Insert => {
                if !in_hunk {
                    in_hunk = true;
                    new_start = consumed_new.saturating_add(1);
                } else if new_count == 0 {
                    new_start = consumed_new.saturating_add(1);
                }
                new_count = new_count.saturating_add(1);
                consumed_new = consumed_new.saturating_add(1);
            }
        }
    }
    flush(
        &mut hunks,
        &mut in_hunk,
        &mut old_count,
        &mut new_count,
        new_start,
    );
    hunks
}

#[cfg(test)]
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

#[cfg(test)]
pub(super) fn parse_diff_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
    let trimmed = line.strip_prefix("@@")?.trim();
    let mut parts = trimmed.split_whitespace();
    let old_part = parts.next()?;
    let new_part = parts.next()?;
    let (old_start, old_count) = parse_hunk_range(old_part)?;
    let (new_start, new_count) = parse_hunk_range(new_part)?;
    Some((old_start, old_count, new_start, new_count))
}

#[cfg(test)]
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
    let probe = git_probe_snapshot_with_numstat(root);
    if !probe.present() {
        return None;
    }
    let branch = probe.branch()?.trim();
    if branch.is_empty() {
        return None;
    }
    Some(GitSummarySnapshot {
        branch: Some(branch.to_owned()),
        head: probe.head().map(str::to_owned),
        added: probe.added(),
        removed: probe.removed(),
    })
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
    git_probe_snapshot(root).present()
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
