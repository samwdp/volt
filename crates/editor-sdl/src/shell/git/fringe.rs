#![allow(unused_imports)]
use super::super::*;

#[allow(unused_imports)]
use super::commands::*;
#[allow(unused_imports)]
use super::commit::*;
#[allow(unused_imports)]
use super::diff::*;
#[allow(unused_imports)]
use super::log::*;
#[allow(unused_imports)]
use super::merge_rebase::*;
#[allow(unused_imports)]
use super::pickers::*;
#[allow(unused_imports)]
use super::process::*;
#[allow(unused_imports)]
use super::remote::*;
#[allow(unused_imports)]
use super::staging::*;
#[allow(unused_imports)]
use super::stash::*;
#[allow(unused_imports)]
use super::status::*;
#[allow(unused_imports)]
use super::worktree::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitFringeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GitFringeSnapshot {
    pub(crate) lines: BTreeMap<usize, GitFringeKind>,
}

impl GitFringeSnapshot {
    pub(crate) fn line_kind(&self, line_index: usize) -> Option<GitFringeKind> {
        self.lines.get(&line_index).copied()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GitFringeState {
    pub(crate) snapshot: Arc<Mutex<GitFringeSnapshot>>,
    pub(crate) inflight: Arc<AtomicBool>,
    pub(crate) revision: Arc<AtomicU64>,
}

impl GitFringeState {
    pub(crate) fn new() -> Self {
        Self {
            snapshot: Arc::new(Mutex::new(GitFringeSnapshot::default())),
            inflight: Arc::new(AtomicBool::new(false)),
            revision: Arc::new(AtomicU64::new(0)),
        }
    }

    pub(crate) fn try_begin_refresh(&self) -> bool {
        self.inflight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(crate) fn finish_refresh(&self) {
        self.inflight.store(false, Ordering::Release);
        ping_shell_wakeup();
    }

    pub(crate) fn try_line_kind(&self, line_index: usize) -> Option<GitFringeKind> {
        let guard = self.snapshot.try_lock().ok()?;
        guard.line_kind(line_index)
    }

    pub(crate) fn update_snapshot(&self, snapshot: GitFringeSnapshot) {
        if let Ok(mut guard) = self.snapshot.lock() {
            *guard = snapshot;
            self.revision.fetch_add(1, Ordering::AcqRel);
        }
    }

    pub(crate) fn snapshot_revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct GitSummarySnapshot {
    pub(crate) branch: Option<String>,
    pub(crate) head: Option<String>,
    pub(crate) added: usize,
    pub(crate) removed: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct GitSummaryState {
    pub(crate) snapshot: Arc<Mutex<Option<GitSummarySnapshot>>>,
    pub(crate) inflight: Arc<AtomicBool>,
    pub(crate) revision: Arc<AtomicU64>,
    pub(crate) changed: Arc<AtomicBool>,
    pub(crate) last_refresh_at: Option<Instant>,
}

impl GitSummaryState {
    pub(crate) fn new() -> Self {
        Self {
            snapshot: Arc::new(Mutex::new(None)),
            inflight: Arc::new(AtomicBool::new(false)),
            revision: Arc::new(AtomicU64::new(0)),
            changed: Arc::new(AtomicBool::new(false)),
            last_refresh_at: None,
        }
    }

    pub(crate) fn snapshot(&self) -> Option<GitSummarySnapshot> {
        let guard = self.snapshot.lock().ok()?;
        guard.clone()
    }

    pub(crate) fn set_snapshot(&self, snapshot: Option<GitSummarySnapshot>) {
        if let Ok(mut guard) = self.snapshot.lock() {
            if *guard == snapshot {
                return;
            }
            *guard = snapshot;
            self.revision.fetch_add(1, Ordering::AcqRel);
            self.changed.store(true, Ordering::Release);
        }
    }

    pub(crate) fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn refresh_due(&self, now: Instant) -> bool {
        self.last_refresh_at
            .map(|last| now.duration_since(last) >= GIT_SUMMARY_REFRESH_INTERVAL)
            .unwrap_or(true)
    }

    pub(crate) fn mark_refreshed(&mut self, now: Instant) {
        self.last_refresh_at = Some(now);
    }

    pub(crate) fn mark_stale(&mut self) {
        self.last_refresh_at = None;
    }

    pub(crate) fn next_refresh_at(&self) -> Instant {
        self.last_refresh_at
            .map(|last| last + GIT_SUMMARY_REFRESH_INTERVAL)
            .unwrap_or_else(Instant::now)
    }

    pub(crate) fn try_begin_refresh(&self) -> bool {
        self.inflight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(crate) fn finish_refresh(&self) {
        self.inflight.store(false, Ordering::Release);
        ping_shell_wakeup();
    }

    pub(crate) fn snapshot_revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GitHeadBlobKey {
    pub(crate) root: PathBuf,
    pub(crate) relative: String,
    pub(crate) head: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GitHeadBlobCache {
    pub(crate) entries: Arc<Mutex<HashMap<GitHeadBlobKey, String>>>,
}

impl GitHeadBlobCache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get(&self, root: &Path, relative: &str, head: &str) -> Option<String> {
        let key = GitHeadBlobKey {
            root: root.to_path_buf(),
            relative: relative.to_owned(),
            head: head.to_owned(),
        };
        self.entries.lock().ok()?.get(&key).cloned()
    }

    pub(crate) fn insert(&self, root: &Path, relative: &str, head: &str, text: String) {
        let Ok(mut entries) = self.entries.lock() else {
            return;
        };
        entries.retain(|key, _| key.root != root || key.head == head);
        entries.insert(
            GitHeadBlobKey {
                root: root.to_path_buf(),
                relative: relative.to_owned(),
                head: head.to_owned(),
            },
            text,
        );
    }
}

pub(crate) fn refresh_pending_git_summary(
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

pub(crate) fn mark_git_fringe_snapshots_stale(runtime: &mut EditorRuntime) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    for buffer in &mut ui.buffers {
        buffer.mark_git_fringe_stale();
    }
    Ok(())
}

pub(crate) fn refresh_git_fringe(
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

pub(crate) fn build_git_fringe_snapshot_with_cache(
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

pub(crate) enum HeadBlob {
    Missing,
    Binary,
    Text(String),
}

pub(crate) fn head_blob_text(
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

pub(crate) fn classify_head_blob(text: String) -> HeadBlob {
    if text.as_bytes().contains(&0) {
        HeadBlob::Binary
    } else {
        HeadBlob::Text(text)
    }
}

pub(crate) fn git_fringe_snapshot_from_texts(
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

pub(crate) fn split_git_fringe_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.lines().collect()
    }
}

pub(crate) fn normalize_git_fringe_text(text: &str) -> String {
    normalize_git_fringe_bytes(text.as_bytes())
}

pub(crate) fn normalize_git_fringe_bytes(bytes: &[u8]) -> String {
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
pub(crate) enum FringeDiffOp {
    Equal,
    Delete,
    Insert,
}

pub(crate) fn line_diff_hunks(
    old_lines: &[&str],
    new_lines: &[&str],
) -> Vec<(usize, usize, usize)> {
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

pub(crate) fn lcs_diff_ops(old_lines: &[&str], new_lines: &[&str]) -> Vec<FringeDiffOp> {
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

pub(crate) fn myers_diff_ops(old_lines: &[&str], new_lines: &[&str]) -> Vec<FringeDiffOp> {
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

pub(crate) fn hunks_from_ops(ops: &[FringeDiffOp]) -> Vec<(usize, usize, usize)> {
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
pub(crate) fn parse_git_fringe_diff(diff_output: &str, line_count: usize) -> GitFringeSnapshot {
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

pub(crate) fn apply_git_fringe_hunk(
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
pub(crate) fn parse_diff_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
    let trimmed = line.strip_prefix("@@")?.trim();
    let mut parts = trimmed.split_whitespace();
    let old_part = parts.next()?;
    let new_part = parts.next()?;
    let (old_start, old_count) = parse_hunk_range(old_part)?;
    let (new_start, new_count) = parse_hunk_range(new_part)?;
    Some((old_start, old_count, new_start, new_count))
}

#[cfg(test)]
pub(crate) fn parse_hunk_range(part: &str) -> Option<(usize, usize)> {
    let part = part.strip_prefix('-').or_else(|| part.strip_prefix('+'))?;
    let mut pieces = part.split(',');
    let start = pieces.next()?.parse::<usize>().ok()?;
    let count = match pieces.next() {
        Some(raw) => raw.parse::<usize>().ok()?,
        None => 1,
    };
    Some((start, count))
}

pub(crate) fn build_git_summary_snapshot(root: &Path) -> Option<GitSummarySnapshot> {
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
