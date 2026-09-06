fn shell_buffer_watch_path(buffer: &ShellBuffer) -> Option<PathBuf> {
    (buffer.kind == BufferKind::File || buffer.is_pdf_buffer())
        .then_some(())
        .and_then(|_| buffer.path().map(Path::to_path_buf))
}

fn sync_file_reload_watch(
    worker: &mut FileReloadWorkerState,
    previous: Option<&Path>,
    current: Option<&Path>,
) {
    if previous == current {
        return;
    }
    if let Some(previous) = previous {
        worker.unwatch_path(previous);
    }
    if let Some(current) = current {
        worker.watch_path(current.to_path_buf());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorSeverity {
    Error,
}

impl ErrorSeverity {
    fn label(self) -> &'static str {
        "error"
    }
}

#[derive(Debug, Clone)]
struct ErrorEntry {
    timestamp: SystemTime,
    severity: ErrorSeverity,
    source: String,
    message: String,
}

impl ErrorEntry {
    fn new(severity: ErrorSeverity, source: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            severity,
            source: source.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug)]
struct ErrorLog {
    entries: Vec<ErrorEntry>,
    buffer_id: BufferId,
    log_file_path: PathBuf,
    file_logging_enabled: bool,
    max_entries: usize,
}

impl ErrorLog {
    fn new(buffer_id: BufferId, log_file_path: PathBuf, file_logging_enabled: bool) -> Self {
        Self {
            entries: Vec::new(),
            buffer_id,
            log_file_path,
            file_logging_enabled,
            max_entries: ERROR_LOG_MAX_ENTRIES,
        }
    }

    fn record(&mut self, entry: ErrorEntry) -> Vec<String> {
        self.push_entry(entry.clone());
        if self.file_logging_enabled
            && let Err(error) = append_error_log(&self.log_file_path, &entry)
        {
            self.file_logging_enabled = false;
            self.push_entry(ErrorEntry::new(
                ErrorSeverity::Error,
                "error-log",
                format!(
                    "failed to write error log to `{}`: {error}",
                    self.log_file_path.display()
                ),
            ));
        }
        errors_buffer_lines(&self.entries, &self.log_file_path)
    }

    fn push_entry(&mut self, entry: ErrorEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            let overflow = self.entries.len() - self.max_entries;
            self.entries.drain(0..overflow);
        }
    }
}

#[derive(Debug, Default)]
struct LspLogBufferState {
    buffer_ids: BTreeMap<WorkspaceId, BTreeMap<String, BufferId>>,
    applied_revision: u64,
}

impl LspLogBufferState {
    fn has_buffers(&self) -> bool {
        self.buffer_ids.values().any(|buffers| !buffers.is_empty())
    }

    fn buffer_id(&self, workspace_id: WorkspaceId, server_id: &str) -> Option<BufferId> {
        self.buffer_ids
            .get(&workspace_id)
            .and_then(|buffers| buffers.get(server_id))
            .copied()
    }

    fn insert_buffer(&mut self, workspace_id: WorkspaceId, server_id: String, buffer_id: BufferId) {
        self.buffer_ids
            .entry(workspace_id)
            .or_default()
            .insert(server_id, buffer_id);
    }

    fn buffers_for_workspace(&self, workspace_id: WorkspaceId) -> Vec<(String, BufferId)> {
        self.buffer_ids
            .get(&workspace_id)
            .into_iter()
            .flat_map(|buffers| buffers.iter())
            .map(|(server_id, buffer_id)| (server_id.clone(), *buffer_id))
            .collect()
    }

    fn remove_workspace(&mut self, workspace_id: WorkspaceId) {
        self.buffer_ids.remove(&workspace_id);
    }
}

#[derive(Debug, Clone)]
struct TypingFrameProfile {
    frame_index: u32,
    timestamp: SystemTime,
    frame_pacing_sleep: Duration,
    polled_events: usize,
    keydown_events: usize,
    text_input_events: usize,
    text_preview: String,
    handle_event_total: Duration,
    keydown_handle_total: Duration,
    text_input_handle_total: Duration,
    text_input_inner_total: Duration,
    layout_sync: Duration,
    picker_search_refresh: Duration,
    lsp_refresh: Duration,
    notification_refresh: Duration,
    autocomplete_refresh: Duration,
    hover_refresh: Duration,
    terminal_refresh: Duration,
    syntax_refresh: Duration,
    syntax_worker_compute: Duration,
    syntax_result_count: usize,
    syntax_highlight_spans: usize,
    git_refresh: Duration,
    acp_refresh: Duration,
    render: Duration,
    present: Duration,
    frame_total: Duration,
    first_text_to_present: Option<Duration>,
    last_text_to_present: Option<Duration>,
}

#[derive(Debug)]
struct ActiveTypingFrameProfile {
    frame_index: u32,
    timestamp: SystemTime,
    frame_pacing_sleep: Duration,
    polled_events: usize,
    keydown_events: usize,
    text_input_events: usize,
    text_preview: String,
    handle_event_total: Duration,
    keydown_handle_total: Duration,
    text_input_handle_total: Duration,
    text_input_inner_total: Duration,
    layout_sync: Duration,
    picker_search_refresh: Duration,
    lsp_refresh: Duration,
    notification_refresh: Duration,
    autocomplete_refresh: Duration,
    hover_refresh: Duration,
    terminal_refresh: Duration,
    syntax_refresh: Duration,
    syntax_worker_compute: Duration,
    syntax_result_count: usize,
    syntax_highlight_spans: usize,
    git_refresh: Duration,
    acp_refresh: Duration,
    render: Duration,
    present: Duration,
    first_text_input_started_at: Option<Instant>,
    last_text_input_started_at: Option<Instant>,
}

impl ActiveTypingFrameProfile {
    fn new(frame_index: u32, frame_pacing_sleep: Duration) -> Self {
        Self {
            frame_index,
            timestamp: SystemTime::now(),
            frame_pacing_sleep,
            polled_events: 0,
            keydown_events: 0,
            text_input_events: 0,
            text_preview: String::new(),
            handle_event_total: Duration::from_secs(0),
            keydown_handle_total: Duration::from_secs(0),
            text_input_handle_total: Duration::from_secs(0),
            text_input_inner_total: Duration::from_secs(0),
            layout_sync: Duration::from_secs(0),
            picker_search_refresh: Duration::from_secs(0),
            lsp_refresh: Duration::from_secs(0),
            notification_refresh: Duration::from_secs(0),
            autocomplete_refresh: Duration::from_secs(0),
            hover_refresh: Duration::from_secs(0),
            terminal_refresh: Duration::from_secs(0),
            syntax_refresh: Duration::from_secs(0),
            syntax_worker_compute: Duration::from_secs(0),
            syntax_result_count: 0,
            syntax_highlight_spans: 0,
            git_refresh: Duration::from_secs(0),
            acp_refresh: Duration::from_secs(0),
            render: Duration::from_secs(0),
            present: Duration::from_secs(0),
            first_text_input_started_at: None,
            last_text_input_started_at: None,
        }
    }

    fn record_event(
        &mut self,
        metadata: &TypingEventMetadata,
        handle_event_total: Duration,
        text_input_inner_total: Option<Duration>,
    ) {
        self.polled_events = self.polled_events.saturating_add(1);
        self.handle_event_total += handle_event_total;
        match metadata {
            TypingEventMetadata::KeyDown => {
                self.keydown_events = self.keydown_events.saturating_add(1);
                self.keydown_handle_total += handle_event_total;
            }
            TypingEventMetadata::TextInput { text, received_at } => {
                self.text_input_events = self.text_input_events.saturating_add(1);
                self.text_input_handle_total += handle_event_total;
                self.text_input_inner_total +=
                    text_input_inner_total.unwrap_or_else(|| Duration::from_secs(0));
                if self.first_text_input_started_at.is_none() {
                    self.first_text_input_started_at = Some(*received_at);
                }
                self.last_text_input_started_at = Some(*received_at);
                self.push_text_preview(text);
            }
            TypingEventMetadata::Other => {}
        }
    }

    fn push_text_preview(&mut self, text: &str) {
        const MAX_PREVIEW_CHARS: usize = 24;
        if self.text_preview.chars().count() >= MAX_PREVIEW_CHARS {
            return;
        }
        let sanitized = sanitize_typing_preview(text);
        if !self.text_preview.is_empty() {
            self.text_preview.push('|');
        }
        for character in sanitized.chars() {
            if self.text_preview.chars().count() >= MAX_PREVIEW_CHARS {
                self.text_preview.push('…');
                break;
            }
            self.text_preview.push(character);
        }
    }

    fn finish(self, frame_total: Duration, presented_at: Instant) -> TypingFrameProfile {
        TypingFrameProfile {
            frame_index: self.frame_index,
            timestamp: self.timestamp,
            frame_pacing_sleep: self.frame_pacing_sleep,
            polled_events: self.polled_events,
            keydown_events: self.keydown_events,
            text_input_events: self.text_input_events,
            text_preview: self.text_preview,
            handle_event_total: self.handle_event_total,
            keydown_handle_total: self.keydown_handle_total,
            text_input_handle_total: self.text_input_handle_total,
            text_input_inner_total: self.text_input_inner_total,
            layout_sync: self.layout_sync,
            picker_search_refresh: self.picker_search_refresh,
            lsp_refresh: self.lsp_refresh,
            notification_refresh: self.notification_refresh,
            autocomplete_refresh: self.autocomplete_refresh,
            hover_refresh: self.hover_refresh,
            terminal_refresh: self.terminal_refresh,
            syntax_refresh: self.syntax_refresh,
            syntax_worker_compute: self.syntax_worker_compute,
            syntax_result_count: self.syntax_result_count,
            syntax_highlight_spans: self.syntax_highlight_spans,
            git_refresh: self.git_refresh,
            acp_refresh: self.acp_refresh,
            render: self.render,
            present: self.present,
            frame_total,
            first_text_to_present: self
                .first_text_input_started_at
                .map(|received_at| presented_at.duration_since(received_at)),
            last_text_to_present: self
                .last_text_input_started_at
                .map(|received_at| presented_at.duration_since(received_at)),
        }
    }
}

#[derive(Debug)]
enum TypingEventMetadata {
    KeyDown,
    TextInput { text: String, received_at: Instant },
    Other,
}

impl TypingEventMetadata {
    fn from_event(event: &Event) -> Self {
        match event {
            Event::KeyDown { .. } => Self::KeyDown,
            Event::TextInput { text, .. } => Self::TextInput {
                text: text.clone(),
                received_at: Instant::now(),
            },
            _ => Self::Other,
        }
    }
}

#[derive(Debug)]
struct TypingProfiler {
    log_path: PathBuf,
    frames: Vec<TypingFrameProfile>,
    max_frames: usize,
    dropped_frames: usize,
}

impl TypingProfiler {
    fn new(log_path: PathBuf) -> Self {
        Self {
            log_path,
            frames: Vec::new(),
            max_frames: TYPING_PROFILE_MAX_FRAMES,
            dropped_frames: 0,
        }
    }

    fn record_frame(&mut self, frame: TypingFrameProfile) {
        if frame.text_input_events == 0
            && frame.keydown_events == 0
            && frame.syntax_result_count == 0
            && frame.frame_total < TYPING_PROFILE_SLOW_FRAME_THRESHOLD
        {
            return;
        }
        self.frames.push(frame);
        if self.frames.len() > self.max_frames {
            let overflow = self.frames.len() - self.max_frames;
            self.frames.drain(0..overflow);
            self.dropped_frames = self.dropped_frames.saturating_add(overflow);
        }
    }

    fn write_report(&self) -> Result<TypingProfileSummary, String> {
        ensure_log_directory(&self.log_path)?;
        let mut file = fs::File::create(&self.log_path).map_err(|error| {
            format!(
                "failed to create typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        let input_frames = self
            .frames
            .iter()
            .filter(|frame| frame.text_input_events > 0)
            .collect::<Vec<_>>();
        let input_frame_times = input_frames
            .iter()
            .map(|frame| frame.frame_total)
            .collect::<Vec<_>>();
        let layout_sync_times = non_zero_frame_durations(&self.frames, |frame| frame.layout_sync);
        let picker_search_times =
            non_zero_frame_durations(&self.frames, |frame| frame.picker_search_refresh);
        let lsp_refresh_times = non_zero_frame_durations(&self.frames, |frame| frame.lsp_refresh);
        let notification_refresh_times =
            non_zero_frame_durations(&self.frames, |frame| frame.notification_refresh);
        let autocomplete_refresh_times =
            non_zero_frame_durations(&self.frames, |frame| frame.autocomplete_refresh);
        let hover_refresh_times =
            non_zero_frame_durations(&self.frames, |frame| frame.hover_refresh);
        let terminal_refresh_times =
            non_zero_frame_durations(&self.frames, |frame| frame.terminal_refresh);
        let syntax_result_frames = self
            .frames
            .iter()
            .filter(|frame| frame.syntax_result_count > 0)
            .collect::<Vec<_>>();
        let syntax_worker_times = syntax_result_frames
            .iter()
            .map(|frame| frame.syntax_worker_compute)
            .collect::<Vec<_>>();
        let syntax_apply_times = syntax_result_frames
            .iter()
            .map(|frame| frame.syntax_refresh)
            .collect::<Vec<_>>();
        let slowest_frame = self
            .frames
            .iter()
            .map(|frame| frame.frame_total)
            .max()
            .unwrap_or_else(|| Duration::from_secs(0));

        writeln!(file, "Volt typing profile").map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "Frames captured: {}", self.frames.len()).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "Frames with text input: {}", input_frames.len()).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "Dropped frames: {}", self.dropped_frames).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        if !input_frame_times.is_empty() {
            writeln!(
                file,
                "Input frame total: avg={}, p50={}, p95={}, max={}",
                format_duration_ms(average_duration(&input_frame_times)),
                format_duration_ms(percentile_duration(&input_frame_times, 50)),
                format_duration_ms(percentile_duration(&input_frame_times, 95)),
                format_duration_ms(
                    *input_frame_times
                        .iter()
                        .max()
                        .unwrap_or(&Duration::from_secs(0))
                ),
            )
            .map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
            write_duration_summary(&mut file, &self.log_path, "Layout sync", &layout_sync_times)?;
            write_duration_summary(
                &mut file,
                &self.log_path,
                "Picker search",
                &picker_search_times,
            )?;
            write_duration_summary(&mut file, &self.log_path, "LSP refresh", &lsp_refresh_times)?;
            write_duration_summary(
                &mut file,
                &self.log_path,
                "Notifications",
                &notification_refresh_times,
            )?;
            write_duration_summary(
                &mut file,
                &self.log_path,
                "Autocomplete",
                &autocomplete_refresh_times,
            )?;
            write_duration_summary(&mut file, &self.log_path, "Hover", &hover_refresh_times)?;
            write_duration_summary(
                &mut file,
                &self.log_path,
                "Terminal",
                &terminal_refresh_times,
            )?;
        }
        if !syntax_result_frames.is_empty() {
            let syntax_result_total = syntax_result_frames
                .iter()
                .map(|frame| frame.syntax_result_count)
                .sum::<usize>();
            let syntax_span_total = syntax_result_frames
                .iter()
                .map(|frame| frame.syntax_highlight_spans)
                .sum::<usize>();
            writeln!(
                file,
                "Syntax worker compute: avg={}, p50={}, p95={}, max={} (frames={}, results={}, spans={})",
                format_duration_ms(average_duration(&syntax_worker_times)),
                format_duration_ms(percentile_duration(&syntax_worker_times, 50)),
                format_duration_ms(percentile_duration(&syntax_worker_times, 95)),
                format_duration_ms(
                    *syntax_worker_times
                        .iter()
                        .max()
                        .unwrap_or(&Duration::from_secs(0))
                ),
                syntax_result_frames.len(),
                syntax_result_total,
                syntax_span_total,
            )
            .map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
            writeln!(
                file,
                "Syntax UI apply: avg={}, p50={}, p95={}, max={}",
                format_duration_ms(average_duration(&syntax_apply_times)),
                format_duration_ms(percentile_duration(&syntax_apply_times, 50)),
                format_duration_ms(percentile_duration(&syntax_apply_times, 95)),
                format_duration_ms(
                    *syntax_apply_times
                        .iter()
                        .max()
                        .unwrap_or(&Duration::from_secs(0))
                ),
            )
            .map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
        }
        writeln!(file).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "Slowest captured frames").map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        let mut slowest_frames = self.frames.iter().collect::<Vec<_>>();
        slowest_frames.sort_by_key(|frame| std::cmp::Reverse(frame.frame_total));
        for frame in slowest_frames.into_iter().take(20) {
            writeln!(file, "{}", format_typing_frame_profile(frame)).map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
        }
        writeln!(file).map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        writeln!(file, "All captured frames").map_err(|error| {
            format!(
                "failed to write typing profile `{}`: {error}",
                self.log_path.display()
            )
        })?;
        for frame in &self.frames {
            writeln!(file, "{}", format_typing_frame_profile(frame)).map_err(|error| {
                format!(
                    "failed to write typing profile `{}`: {error}",
                    self.log_path.display()
                )
            })?;
        }

        Ok(TypingProfileSummary {
            log_path: self.log_path.display().to_string(),
            frames_captured: self.frames.len(),
            input_frames_captured: input_frames.len(),
            slowest_frame_micros: slowest_frame.as_micros(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ShellVisualRefreshKey {
    render_width: u32,
    render_height: u32,
    theme_settings: ThemeRuntimeSettings,
    git_summary_revision: u64,
    git_fringe_revisions: Vec<(BufferId, u64)>,
    lsp_diagnostics_revisions: Vec<(BufferId, u64)>,
    active_lsp_server: Option<String>,
    active_lsp_workspace_loaded: bool,
    notification_revision: u64,
    notification_deadline: Option<Instant>,
    yank_flash_until: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FpsOverlaySnapshot {
    latest_frame_time: Duration,
    average_frame_time: Duration,
    worst_frame_time: Duration,
}

#[derive(Debug, Default)]
struct FpsOverlayState {
    recent_frame_times: VecDeque<Duration>,
    total_recent_frame_time_nanos: u128,
    worst_frame_time: Duration,
}

impl FpsOverlayState {
    fn snapshot(&self) -> Option<FpsOverlaySnapshot> {
        if self.recent_frame_times.is_empty() {
            return None;
        }
        let average_frame_time_nanos =
            self.total_recent_frame_time_nanos / self.recent_frame_times.len() as u128;
        let average_frame_time =
            Duration::from_nanos(average_frame_time_nanos.min(u128::from(u64::MAX)) as u64);
        self.recent_frame_times
            .back()
            .copied()
            .map(|latest_frame_time| FpsOverlaySnapshot {
                latest_frame_time,
                average_frame_time,
                worst_frame_time: self.worst_frame_time,
            })
    }

    fn record_frame(&mut self, frame_time: Duration) {
        self.recent_frame_times.push_back(frame_time);
        self.total_recent_frame_time_nanos += frame_time.as_nanos();
        self.worst_frame_time = self.worst_frame_time.max(frame_time);
        while self.recent_frame_times.len() > FPS_OVERLAY_HISTORY_FRAMES {
            if let Some(removed) = self.recent_frame_times.pop_front() {
                self.total_recent_frame_time_nanos = self
                    .total_recent_frame_time_nanos
                    .saturating_sub(removed.as_nanos());
            }
        }
    }
}
