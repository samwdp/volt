fn default_volt_state_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        let base = env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        base.join("volt")
    } else {
        let base = env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("state"))
            })
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        base.join("volt")
    }
}

fn default_error_log_path() -> PathBuf {
    default_volt_state_dir().join(ERROR_LOG_FILE_NAME)
}

fn active_theme_state_path() -> PathBuf {
    default_volt_state_dir().join(ACTIVE_THEME_STATE_FILE_NAME)
}

fn default_mark_list_path() -> PathBuf {
    default_volt_state_dir().join(MARK_LIST_FILE_NAME)
}

fn default_typing_profile_log_path() -> PathBuf {
    default_volt_state_dir().join(TYPING_PROFILE_LOG_FILE_NAME)
}

fn read_saved_theme_selection(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let theme_id = contents.trim();
            if theme_id.is_empty() {
                Ok(None)
            } else {
                Ok(Some(theme_id.to_owned()))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "failed to read saved theme from `{}`: {error}",
            path.display()
        )),
    }
}

fn write_saved_theme_selection(path: &Path, theme_id: &str) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err(format!(
            "saved theme path `{}` does not have a parent directory",
            path.display()
        ));
    };
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create theme state directory `{}`: {error}",
            parent.display()
        )
    })?;
    fs::write(path, format!("{theme_id}\n")).map_err(|error| {
        format!(
            "failed to write saved theme to `{}`: {error}",
            path.display()
        )
    })
}

fn clear_saved_theme_selection(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to clear saved theme `{}`: {error}",
            path.display()
        )),
    }
}

fn restore_saved_theme_selection(
    theme_registry: &mut ThemeRegistry,
    path: &Path,
) -> Result<(), String> {
    let Some(theme_id) = read_saved_theme_selection(path)? else {
        return Ok(());
    };
    if let Err(error) = theme_registry.activate(&theme_id) {
        clear_saved_theme_selection(path)?;
        return Err(format!(
            "saved theme `{theme_id}` is no longer available and was removed from `{}`: {error}",
            path.display()
        ));
    }
    Ok(())
}

fn ensure_log_directory(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create log directory `{}`: {error}",
            parent.display()
        )
    })
}

fn install_panic_hook(log_file_path: PathBuf) {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let entry = ErrorEntry::new(ErrorSeverity::Error, "panic", info.to_string());
        if let Err(error) = append_error_log(&log_file_path, &entry) {
            eprintln!("Failed to write panic log: {error}");
        }
        default_hook(info);
    }));
}

fn panic_payload_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic payload is not a string".to_owned()
    }
}

fn format_timestamp(timestamp: SystemTime) -> String {
    match timestamp.duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}.{:03}", duration.as_secs(), duration.subsec_millis()),
        Err(_) => "0.000".to_owned(),
    }
}

fn format_duration_ms(duration: Duration) -> String {
    let micros = duration.as_micros();
    format!("{}.{:03}ms", micros / 1_000, micros % 1_000)
}

fn average_duration(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    let total_micros = durations
        .iter()
        .map(|duration| duration.as_micros())
        .sum::<u128>();
    let average_micros = total_micros / durations.len() as u128;
    let seconds = average_micros / 1_000_000;
    let nanos = ((average_micros % 1_000_000) * 1_000) as u32;
    Duration::new(seconds.min(u64::MAX as u128) as u64, nanos)
}

fn percentile_duration(durations: &[Duration], percentile: usize) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    let mut sorted = durations.to_vec();
    sorted.sort();
    let clamped = percentile.min(100);
    let index = ((sorted.len().saturating_sub(1)) * clamped) / 100;
    sorted[index]
}

fn non_zero_frame_durations(
    frames: &[TypingFrameProfile],
    extract: impl Fn(&TypingFrameProfile) -> Duration,
) -> Vec<Duration> {
    frames
        .iter()
        .map(extract)
        .filter(|duration| !duration.is_zero())
        .collect()
}

fn write_duration_summary(
    file: &mut fs::File,
    log_path: &Path,
    label: &str,
    durations: &[Duration],
) -> Result<(), String> {
    if durations.is_empty() {
        return Ok(());
    }
    writeln!(
        file,
        "{label}: avg={}, p50={}, p95={}, max={} (frames={})",
        format_duration_ms(average_duration(durations)),
        format_duration_ms(percentile_duration(durations, 50)),
        format_duration_ms(percentile_duration(durations, 95)),
        format_duration_ms(*durations.iter().max().unwrap_or(&Duration::from_secs(0))),
        durations.len(),
    )
    .map_err(|error| {
        format!(
            "failed to write typing profile `{}`: {error}",
            log_path.display()
        )
    })
}

fn format_typing_frame_profile(frame: &TypingFrameProfile) -> String {
    let timestamp = format_timestamp(frame.timestamp);
    let preview = if frame.text_preview.is_empty() {
        "<none>".to_owned()
    } else {
        frame.text_preview.clone()
    };
    let first_to_present = frame
        .first_text_to_present
        .map(format_duration_ms)
        .unwrap_or_else(|| "-".to_owned());
    let last_to_present = frame
        .last_text_to_present
        .map(format_duration_ms)
        .unwrap_or_else(|| "-".to_owned());
    format!(
        "[{timestamp}] frame={} pacing_sleep={} events={} keydowns={} text_inputs={} preview=\"{}\" handle={} keydown_handle={} text_handle={} text_inner={} layout_sync={} picker_search={} lsp={} notifications={} autocomplete={} hover={} terminal={} syntax_apply={} syntax_worker={} syntax_results={} syntax_spans={} git={} acp={} render={} present={} total={} first_text_to_present={} last_text_to_present={}",
        frame.frame_index,
        format_duration_ms(frame.frame_pacing_sleep),
        frame.polled_events,
        frame.keydown_events,
        frame.text_input_events,
        preview,
        format_duration_ms(frame.handle_event_total),
        format_duration_ms(frame.keydown_handle_total),
        format_duration_ms(frame.text_input_handle_total),
        format_duration_ms(frame.text_input_inner_total),
        format_duration_ms(frame.layout_sync),
        format_duration_ms(frame.picker_search_refresh),
        format_duration_ms(frame.lsp_refresh),
        format_duration_ms(frame.notification_refresh),
        format_duration_ms(frame.autocomplete_refresh),
        format_duration_ms(frame.hover_refresh),
        format_duration_ms(frame.terminal_refresh),
        format_duration_ms(frame.syntax_refresh),
        format_duration_ms(frame.syntax_worker_compute),
        frame.syntax_result_count,
        frame.syntax_highlight_spans,
        format_duration_ms(frame.git_refresh),
        format_duration_ms(frame.acp_refresh),
        format_duration_ms(frame.render),
        format_duration_ms(frame.present),
        format_duration_ms(frame.frame_total),
        first_to_present,
        last_to_present,
    )
}

fn sanitize_typing_preview(text: &str) -> String {
    let mut sanitized = String::new();
    for character in text.chars() {
        match character {
            '\n' => sanitized.push_str("\\n"),
            '\r' => sanitized.push_str("\\r"),
            '\t' => sanitized.push_str("\\t"),
            other => sanitized.push(other),
        }
    }
    sanitized
}

fn format_error_entry_lines(entry: &ErrorEntry) -> Vec<String> {
    let timestamp = format_timestamp(entry.timestamp);
    let mut lines = Vec::new();
    let mut message_lines = entry.message.lines();
    if let Some(first) = message_lines.next() {
        lines.push(format!(
            "[{timestamp}] {} {}: {first}",
            entry.severity.label(),
            entry.source
        ));
        for line in message_lines {
            lines.push(format!("    {line}"));
        }
    } else {
        lines.push(format!(
            "[{timestamp}] {} {}: <empty>",
            entry.severity.label(),
            entry.source
        ));
    }
    lines
}

fn append_error_log(path: &Path, entry: &ErrorEntry) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to open log `{}`: {error}", path.display()))?;
    for line in format_error_entry_lines(entry) {
        writeln!(file, "{line}")
            .map_err(|error| format!("failed to write log `{}`: {error}", path.display()))?;
    }
    Ok(())
}

fn errors_buffer_lines(entries: &[ErrorEntry], log_path: &Path) -> Vec<String> {
    let mut lines = initial_errors_lines(Some(log_path));
    if entries.is_empty() {
        return lines;
    }
    lines.push(String::new());
    lines.push(format!("Recent errors ({})", entries.len()));
    for entry in entries {
        lines.extend(format_error_entry_lines(entry));
    }
    lines
}

fn record_runtime_error(runtime: &mut EditorRuntime, source: &str, message: impl Into<String>) {
    let entry = ErrorEntry::new(ErrorSeverity::Error, source, message);
    let (buffer_id, lines) = {
        let Some(log) = runtime.services_mut().get_mut::<ErrorLog>() else {
            eprintln!("Error log service missing for: {}.", entry.message);
            return;
        };
        let lines = log.record(entry);
        (log.buffer_id, lines)
    };
    if let Err(error) = update_error_buffer(runtime, buffer_id, lines) {
        eprintln!("Failed to update errors buffer: {error}");
    }
}

fn update_error_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    lines: Vec<String>,
) -> Result<(), String> {
    let user_library = shell_user_library(runtime);
    let ui = shell_ui_mut(runtime)?;
    let buffer = ui.ensure_buffer(
        buffer_id,
        "*errors*",
        BufferKind::Diagnostics,
        &*user_library,
    );
    buffer.replace_with_lines(lines);
    Ok(())
}

fn format_lsp_log_entry_lines(entry: &LspLogEntry) -> Vec<String> {
    let timestamp = format_timestamp(entry.timestamp());
    let mut body_lines = entry
        .body()
        .lines()
        .map(str::trim_end)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let prefix = format!(
        "[{timestamp}] {} {}",
        entry.direction().label(),
        entry.server_id()
    );
    let mut lines = Vec::with_capacity(body_lines.len().saturating_add(2));
    if let Some(first) = body_lines.first() {
        lines.push(format!("{prefix}: {first}"));
        for line in body_lines.drain(1..) {
            lines.push(format!("    {line}"));
        }
    } else {
        lines.push(format!("{prefix}: <empty>"));
    }
    lines.push(String::new());
    lines
}

fn lsp_log_entries_for_server(entries: &[LspLogEntry], server_id: &str) -> Vec<LspLogEntry> {
    entries
        .iter()
        .filter(|entry| entry.server_id() == server_id)
        .cloned()
        .collect()
}

fn lsp_log_buffer_lines(server_id: &str, entries: &[LspLogEntry]) -> Vec<String> {
    let mut lines = initial_lsp_log_lines(server_id);
    if entries.is_empty() {
        return lines;
    }
    lines.push(String::new());
    lines.push(format!("Recent transport entries ({})", entries.len()));
    for entry in entries {
        lines.extend(format_lsp_log_entry_lines(entry));
    }
    lines
}

fn build_shell_summary(
    state: &mut ShellState,
    frames_rendered: u32,
    renderer_name: String,
    font_path: &Path,
) -> ShellSummary {
    let typing_profile = match state.finish_typing_profile() {
        Ok(profile) => profile,
        Err(error) => {
            state.record_error("typing-profile", error);
            None
        }
    };
    let pane_count = match state.pane_count() {
        Ok(count) => count,
        Err(error) => {
            state.record_shell_error("shell.summary.pane-count", error);
            0
        }
    };
    let popup_visible = match state.popup_visible() {
        Ok(visible) => visible,
        Err(error) => {
            state.record_shell_error("shell.summary.popup-visible", error);
            false
        }
    };
    ShellSummary {
        frames_rendered,
        pane_count,
        popup_visible,
        render_backend: RenderBackend::SdlCanvas,
        renderer_name,
        font_path: font_path.display().to_string(),
        typing_profile,
    }
}

fn initial_errors_lines(log_path: Option<&Path>) -> Vec<String> {
    let mut lines = vec![
        "*errors* captures runtime failures and panics.".to_owned(),
        "The shell continues running while logging errors here.".to_owned(),
        "Open the buffer picker (F4) to revisit this buffer.".to_owned(),
    ];
    if let Some(path) = log_path {
        lines.push(format!("Log file: {}", path.display()));
    } else {
        lines.push("Log file: <pending>".to_owned());
    }
    lines
}

fn initial_lsp_log_lines(server_id: &str) -> Vec<String> {
    vec![
        format!(
            "{} captures live JSON-RPC traffic for `{server_id}`.",
            lsp_log_buffer_name(server_id)
        ),
        "Run `lsp.log` from a buffer using that server, or open the buffer picker (F4) to focus this buffer.".to_owned(),
        "Requests, notifications, responses, and disconnect events are appended here.".to_owned(),
    ]
}

fn initial_scratch_lines() -> Vec<String> {
    vec![
        "This buffer is for text that is not saved".to_owned(),
        "To create a file, visit it with ‘CTRL .’ and enter text in its buffer.".to_owned(),
    ]
}

fn workspace_scratch_lines(name: &str, root: Option<&std::path::Path>) -> Vec<String> {
    if name == "default" && root.is_none() {
        return initial_scratch_lines();
    }

    let mut lines = vec![format!("Workspace `{name}` is now active.")];
    if let Some(root) = root {
        lines.push(format!("Root: {}", root.display()));
    }
    lines.push("This workspace was opened from the project picker.".to_owned());
    lines.push(
        "Run `workspace.switch` to change workspaces or `workspace.delete` to close one."
            .to_owned(),
    );
    lines
}

fn initial_notes_lines() -> Vec<String> {
    vec![
        "Second pane notes.".to_owned(),
        "Use F2 to split horizontally and Tab to move between panes.".to_owned(),
        "The buffer picker opened by F4 reuses the same searchable popup surface as F3.".to_owned(),
    ]
}

fn workspace_notes_lines(name: &str, root: Option<&std::path::Path>) -> Vec<String> {
    if name == "default" && root.is_none() {
        return initial_notes_lines();
    }

    let mut lines = vec![format!("Notes for workspace `{name}`.")];
    if let Some(root) = root {
        lines.push(format!("Project root: {}", root.display()));
    }
    lines.push("Use this buffer for project-specific notes or scratch edits.".to_owned());
    lines
}

fn buffer_interaction(
    kind: &BufferKind,
    _user_library: &dyn UserLibrary,
) -> (bool, Option<InputField>) {
    match kind {
        BufferKind::Image => (false, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_READONLY_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_INPUT_KIND => {
            (true, Some(InputField::new("Ask > ")))
        }
        BufferKind::Plugin(plugin_kind) if plugin_kind == DB_CONNECT_KIND => {
            let mut input = InputField::new("DB > ");
            input.set_placeholder(Some(
                "sqlite://C:/data/app.db or remember prod :: postgres://user:pass@host/db"
                    .to_owned(),
            ));
            (true, Some(input))
        }
        BufferKind::Plugin(plugin_kind) if plugin_kind == DB_CONNECTIONS_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == DB_SCHEMA_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == DB_HISTORY_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == DB_SNIPPETS_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == DB_RESULTS_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == DB_DASHBOARD_KIND => (false, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == DB_SIDEBAR_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == BROWSER_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == PDF_BUFFER_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == LSP_METADATA_BUFFER_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STATUS_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_DIFF_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_LOG_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STASH_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_COMMIT_KIND => (false, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_EDITOR_KIND => (false, None),
        BufferKind::Plugin(plugin_kind) if is_issues_board_kind(plugin_kind) => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == OIL_PREVIEW_KIND => (true, None),
        BufferKind::Plugin(plugin_kind) if plugin_kind == OIL_HELP_KIND => (true, None),
        BufferKind::Terminal => (true, None),
        BufferKind::Directory => (false, None),
        BufferKind::Quickfix => (true, None),
        _ => (false, None),
    }
}

fn plugin_section_state_for_kind(
    kind: &BufferKind,
    user_library: &dyn UserLibrary,
) -> Option<PluginSectionBufferState> {
    let BufferKind::Plugin(plugin_kind) = kind else {
        return None;
    };
    let buffer = user_library.plugin_buffer(plugin_kind)?;
    let sections = buffer.sections()?.clone();
    PluginSectionBufferState::new(sections, buffer.evaluate_target_section())
}

fn plugin_buffer_line_wrap(kind: &BufferKind, user_library: &dyn UserLibrary) -> bool {
    match kind {
        BufferKind::Plugin(plugin_kind) => user_library.plugin_buffer_line_wrap(plugin_kind),
        _ => true,
    }
}

fn placeholder_lines(name: &str, kind: &BufferKind, user_library: &dyn UserLibrary) -> Vec<String> {
    match name {
        "*scratch*" => initial_scratch_lines(),
        "*notes*" => initial_notes_lines(),
        "*errors*" => initial_errors_lines(None),
        _ => match kind {
            BufferKind::Image => vec![
                format!("{name} is a native image buffer."),
                "Supported image files open directly into a centered preview.".to_owned(),
            ],
            BufferKind::Scratch => vec![
                format!("{name} is a scratch buffer created by the runtime."),
                "This buffer can be focused from the generic buffer picker.".to_owned(),
            ],
            BufferKind::Picker => vec![
                format!("{name} is a picker-backed buffer."),
                "The SDL shell renders picker state through the popup search UI.".to_owned(),
            ],
            BufferKind::Terminal => vec![
                format!("{name} is launching the configured shell."),
                "Press i to enter terminal input mode, or stay in Normal mode to navigate scrollback."
                    .to_owned(),
            ],
            BufferKind::Git => vec![
                format!("{name} is reserved for git workflows."),
                "The next iteration can wire real magit-style status content here.".to_owned(),
            ],
            BufferKind::Directory => vec![
                format!("{name} is a directory buffer."),
                "Oil-style editing surfaces can be rendered through the same shell.".to_owned(),
            ],
            BufferKind::Diagnostics => vec![
                format!("{name} is a diagnostics-oriented buffer."),
                "LSP and DAP packages can surface structured status here.".to_owned(),
            ],
            BufferKind::Quickfix => vec![
                format!("{name} is a quickfix result list."),
                "Press Enter to open target and return focus to workspace.".to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_READONLY_KIND => vec![
                format!("{name} is an interactive read-only buffer."),
                "Keybindings still run, but edits are blocked.".to_owned(),
                "Use this as a starting point for magit-style interfaces.".to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == INTERACTIVE_INPUT_KIND => vec![
                format!("{name} is an interactive input buffer."),
                "Type into the prompt to submit commands or text.".to_owned(),
                "Use Ctrl+Enter to submit or Ctrl+l to clear.".to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == BROWSER_KIND => {
                user_library.browser_buffer_lines(None)
            }
            BufferKind::Plugin(plugin_kind) if plugin_kind == PDF_BUFFER_KIND => vec![
                format!("{name} is a native PDF buffer."),
                "Open a .pdf file to inspect its metadata, page text, and structural state."
                    .to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND => vec![
                format!("{name} is an ACP session buffer."),
                "Use acp.pick-client to start an ACP agent.".to_owned(),
                "Type into the prompt and press Ctrl+Enter to send.".to_owned(),
                "Use / for slash commands, @ to link git files, Ctrl+Shift+V to paste images, Ctrl+Space/Tab for completion, Shift+Tab to cycle modes, Ctrl+Tab to switch ACP panes, acp.pick-mode to choose a mode, acp.pick-model to choose a model, and Ctrl+j for a newline."
                    .to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STATUS_KIND => Vec::new(),
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_DIFF_KIND => Vec::new(),
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_LOG_KIND => Vec::new(),
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_STASH_KIND => Vec::new(),
            BufferKind::Plugin(plugin_kind) if plugin_kind == GIT_COMMIT_KIND => {
                user_library.git_commit_template(&editor_git::GitStatusSnapshot::default())
            }
            BufferKind::File => vec![
                format!("{name} is a file-backed buffer placeholder."),
                "File loading is not yet wired into the SDL shell event loop.".to_owned(),
            ],
            BufferKind::Plugin(plugin_kind) => {
                // Ask the user library for initial content.  If none is provided,
                // fall back to the generic plugin placeholder message.
                let initial = user_library.plugin_buffer_initial_lines(plugin_kind);
                if initial.is_empty() {
                    vec![
                        format!("{name} was opened for plugin kind `{plugin_kind}`."),
                        "Users can change this behavior by editing the matching user package and recompiling.".to_owned(),
                    ]
                } else {
                    initial
                }
            }
        },
    }
}

fn buffer_kind_label(kind: &BufferKind) -> String {
    match kind {
        BufferKind::File => "file".to_owned(),
        BufferKind::Image => "image".to_owned(),
        BufferKind::Scratch => "scratch".to_owned(),
        BufferKind::Picker => "picker".to_owned(),
        BufferKind::Terminal => "terminal".to_owned(),
        BufferKind::Git => "git".to_owned(),
        BufferKind::Directory => "directory".to_owned(),
        BufferKind::Diagnostics => "diagnostics".to_owned(),
        BufferKind::Quickfix => "quickfix".to_owned(),
        BufferKind::Plugin(plugin_kind) => plugin_kind.clone(),
    }
}

fn popup_window_height(content_height: u32, line_height: i32) -> u32 {
    let row_height = line_height.max(1) as u32;
    if content_height <= row_height {
        return content_height;
    }

    let desired = (content_height.saturating_mul(2) / 5).max(row_height * 4);
    let max_height = content_height.saturating_sub(row_height).max(row_height);
    let clamped = desired.min(max_height);
    (clamped / row_height).max(1) * row_height
}

fn pixel_rect_contains_point(rect: PixelRect, x: i32, y: i32) -> bool {
    let right = rect.x.saturating_add(rect.width as i32);
    let bottom = rect.y.saturating_add(rect.height as i32);
    x >= rect.x && x < right && y >= rect.y && y < bottom
}
