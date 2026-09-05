pub(crate) struct ShellState {
    pub(crate) runtime: EditorRuntime,
    #[cfg(test)]
    pub(crate) user_library: Arc<dyn UserLibrary>,
    deferred_startup_pending: bool,
    typing_profiler: Option<TypingProfiler>,
    last_text_input_profile: Option<Duration>,
    last_text_input_at: Option<Instant>,
    pending_suppressed_text_input: Option<SuppressedTextInput>,
    mouse_drag: Option<MouseDragState>,
    browser_host: BrowserHostService,
}

#[derive(Debug)]
struct SuppressedTextInput {
    text: String,
    expires_at: Instant,
}

#[derive(Debug, Clone, Copy)]
struct MouseDragState {
    buffer_id: BufferId,
    rect: PixelRect,
    anchor: TextPoint,
    kind: VisualSelectionKind,
}

#[cfg(test)]
struct ShellTestUserLibrary;

#[cfg(test)]
impl UserLibrary for ShellTestUserLibrary {
    fn picker_providers(&self) -> Vec<PickerProviderSpec> {
        vec![
            PickerProviderSpec::new(
                "workspace.switch",
                "Switch Workspace",
                PickerSource::WorkspaceSwitch,
            ),
            PickerProviderSpec::new(
                "workspace.delete",
                "Delete Workspace",
                PickerSource::WorkspaceDelete,
            ),
        ]
    }

    fn picker_provider_items(
        &self,
        context: &PickerProviderContext,
    ) -> Option<Vec<editor_plugin_api::PickerItemSpec>> {
        match context.source {
            PickerSource::WorkspaceSwitch => Some(
                context
                    .workspaces
                    .iter()
                    .map(workspace_picker_item)
                    .collect(),
            ),
            PickerSource::WorkspaceDelete => Some(
                context
                    .workspaces
                    .iter()
                    .filter(|workspace| !workspace.is_default)
                    .map(workspace_picker_item)
                    .collect(),
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
fn workspace_picker_item(workspace: &PickerWorkspaceContext) -> editor_plugin_api::PickerItemSpec {
    let detail = workspace
        .root
        .as_ref()
        .into_option()
        .map(|root| root.to_string())
        .unwrap_or_else(|| "default workspace".to_owned());
    editor_plugin_api::PickerItemSpec::new(
        workspace.id.to_string(),
        workspace.name.clone(),
        detail.clone(),
        PickerActionSpec::switch_workspace(workspace.id),
    )
    .with_preview(detail)
}

impl ShellState {
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, ShellError> {
        let user_library: Arc<dyn UserLibrary> = Arc::new(ShellTestUserLibrary);
        Self::new_with_user_library(default_error_log_path(), false, user_library)
    }

    #[cfg(test)]
    pub(crate) fn new_with_user_library(
        log_file_path: PathBuf,
        profile_input_latency: bool,
        user_library: Arc<dyn UserLibrary>,
    ) -> Result<Self, ShellError> {
        Self::new_with_user_library_inner(log_file_path, profile_input_latency, user_library, false)
    }

    fn new_with_user_library_fast_start(
        log_file_path: PathBuf,
        profile_input_latency: bool,
        user_library: Arc<dyn UserLibrary>,
    ) -> Result<Self, ShellError> {
        Self::new_with_user_library_inner(log_file_path, profile_input_latency, user_library, true)
    }

    fn new_with_user_library_inner(
        log_file_path: PathBuf,
        profile_input_latency: bool,
        user_library: Arc<dyn UserLibrary>,
        defer_optional_services: bool,
    ) -> Result<Self, ShellError> {
        let mut startup_trace = StartupTrace::new();
        let mut runtime = EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("volt");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "default", default_workspace_root())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        if let Some(trace) = startup_trace.as_mut() {
            trace.mark("shell.runtime-foundation");
        }

        register_shell_hooks(&mut runtime).map_err(ShellError::Runtime)?;
        register_git_status_commands(&mut runtime).map_err(ShellError::Runtime)?;
        if let Some(trace) = startup_trace.as_mut() {
            trace.mark("shell.register-hooks");
        }

        let notes_id = runtime
            .model_mut()
            .create_buffer(workspace_id, "*notes*", BufferKind::Scratch, None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let scratch_id = runtime
            .model_mut()
            .create_buffer(workspace_id, "*scratch*", BufferKind::Scratch, None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let errors_id = runtime
            .model_mut()
            .create_popup_buffer(workspace_id, "*errors*", BufferKind::Diagnostics, None)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let (scratch, notes, primary_pane_id) = {
            let workspace = runtime
                .model()
                .workspace(workspace_id)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            let pane_id = workspace.active_pane_id().ok_or_else(|| {
                ShellError::Runtime("default workspace has no active pane".to_owned())
            })?;
            let scratch = workspace.buffer(scratch_id).ok_or_else(|| {
                ShellError::Runtime("scratch buffer missing after bootstrap".to_owned())
            })?;
            let notes = workspace.buffer(notes_id).ok_or_else(|| {
                ShellError::Runtime("notes buffer missing after bootstrap".to_owned())
            })?;
            (
                ShellBuffer::from_runtime_buffer(scratch, initial_scratch_lines(), &*user_library),
                ShellBuffer::from_runtime_buffer(notes, initial_notes_lines(), &*user_library),
                pane_id,
            )
        };

        let mut ui_state =
            ShellUiState::new(workspace_id, primary_pane_id, scratch, notes, notes_id);
        ui_state
            .ensure_buffer(
                errors_id,
                "*errors*",
                BufferKind::Diagnostics,
                &*user_library,
            )
            .replace_with_lines(initial_errors_lines(Some(&log_file_path)));
        runtime.services_mut().insert(ui_state);
        let mark_list_path = default_mark_list_path();
        let (mark_list_state, mark_list_error) = match MarkListState::load(mark_list_path.clone()) {
            Ok(state) => (state, None),
            Err(error) => (MarkListState::empty(mark_list_path), Some(error)),
        };
        runtime.services_mut().insert(mark_list_state);
        if let Some(trace) = startup_trace.as_mut() {
            trace.mark("shell.bootstrap-buffers");
        }

        let log_dir_error = ensure_log_directory(&log_file_path).err();
        runtime.services_mut().insert(ErrorLog::new(
            errors_id,
            log_file_path,
            log_dir_error.is_none(),
        ));
        if let Some(error) = log_dir_error {
            record_runtime_error(&mut runtime, "error-log", error);
        }
        if let Some(error) = mark_list_error {
            record_runtime_error(&mut runtime, "workspace.mark-list", error);
        }
        runtime.services_mut().insert(LspLogBufferState::default());
        runtime.services_mut().insert(QuickfixState::default());
        runtime
            .services_mut()
            .insert(Mutex::new(TerminalBufferState::default()));
        runtime.services_mut().insert(FormatterRegistry::default());
        runtime.services_mut().insert(Mutex::new(JobManager::new()));
        let mut theme_registry = ThemeRegistry::new();
        theme_registry
            .register_all(user_library.themes())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        if let Err(error) =
            restore_saved_theme_selection(&mut theme_registry, &active_theme_state_path())
        {
            record_runtime_error(&mut runtime, "theme.restore", error);
        }
        runtime.services_mut().insert(theme_registry);
        if let Some(trace) = startup_trace.as_mut() {
            trace.mark("shell.theme-registry");
        }
        runtime
            .services_mut()
            .insert(UserLibraryService(Arc::clone(&user_library)));
        runtime
            .services_mut()
            .insert(UserLibraryReloadState::default());
        load_auto_loaded_packages(&mut runtime, &user_library.packages())
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        warm_project_discovery(user_library.as_ref());
        picker::ensure_picker_keybindings(&mut runtime).map_err(ShellError::Runtime)?;
        if let Some(trace) = startup_trace.as_mut() {
            trace.mark("shell.user-packages");
        }
        if !defer_optional_services {
            install_optional_runtime_services(&mut runtime, &*user_library)?;
            if let Some(trace) = startup_trace.as_mut() {
                trace.mark("shell.optional-services");
            }
        }

        Ok(Self {
            runtime,
            #[cfg(test)]
            user_library,
            deferred_startup_pending: defer_optional_services,
            typing_profiler: profile_input_latency
                .then(|| TypingProfiler::new(default_typing_profile_log_path())),
            last_text_input_profile: None,
            last_text_input_at: None,
            pending_suppressed_text_input: None,
            mouse_drag: None,
            browser_host: BrowserHostService::new(),
        })
    }

    fn deferred_startup_pending(&self) -> bool {
        self.deferred_startup_pending
    }

    fn finish_deferred_startup(&mut self) -> Result<(), ShellError> {
        if !self.deferred_startup_pending {
            return Ok(());
        }
        let user_library = shell_user_library(&self.runtime);
        install_optional_runtime_services(&mut self.runtime, &*user_library)?;
        warm_project_discovery(&*user_library);
        self.deferred_startup_pending = false;
        Ok(())
    }

    fn record_error(&mut self, source: &str, message: impl Into<String>) {
        record_runtime_error(&mut self.runtime, source, message);
    }

    fn record_shell_error(&mut self, source: &str, error: ShellError) {
        self.record_error(source, error.to_string());
    }

    fn begin_typing_frame(
        &self,
        frame_index: u32,
        frame_pacing_sleep: Duration,
    ) -> Option<ActiveTypingFrameProfile> {
        self.typing_profiler
            .as_ref()
            .map(|_| ActiveTypingFrameProfile::new(frame_index, frame_pacing_sleep))
    }

    fn record_typing_frame(&mut self, frame: TypingFrameProfile) {
        if let Some(profiler) = self.typing_profiler.as_mut() {
            profiler.record_frame(frame);
        }
    }

    fn take_last_text_input_profile(&mut self) -> Option<Duration> {
        self.last_text_input_profile.take()
    }

    fn note_text_edit_activity(&mut self) {
        self.last_text_input_at = Some(Instant::now());
    }

    fn secondary_refresh_deferred_for_typing(&self, now: Instant) -> bool {
        secondary_refresh_deferred_for_typing(self.last_text_input_at, now)
    }

    fn typing_refresh_budget_active(&self, now: Instant) -> bool {
        frame_pacing_deferred_for_typing(self.last_text_input_at, now)
    }

    fn frame_pacing_deferred_for_typing(&self, now: Instant) -> bool {
        frame_pacing_deferred_for_typing(self.last_text_input_at, now)
    }

    fn idle_wait_deadlines(
        &self,
        now: Instant,
        extras: impl IntoIterator<Item = Instant>,
    ) -> Vec<Instant> {
        const GIT_DIRECTORY_PREFIX_TIMEOUT: Duration = Duration::from_millis(1200);
        let mut deadlines: Vec<Instant> = extras.into_iter().collect();
        if self.deferred_startup_pending() {
            deadlines.push(now);
        }
        if let Some(last) = self.last_text_input_at {
            deadlines.push(last + FRAME_PACING_TYPING_IDLE_THRESHOLD);
            deadlines.push(last + GIT_REFRESH_TYPING_IDLE_THRESHOLD);
            deadlines.push(last + LSP_SYNC_TYPING_IDLE_THRESHOLD);
        }
        if let Some(pending) = self.pending_suppressed_text_input.as_ref() {
            deadlines.push(pending.expires_at);
        }
        let Ok(ui) = self.ui() else {
            return deadlines;
        };
        if let Some(until) = ui.yank_flash_deadline(now) {
            deadlines.push(until);
        }
        if let Some(until) = ui.notification_deadline(now) {
            deadlines.push(until);
        }
        deadlines.push(ui.git_summary.next_refresh_at());
        if let Some(prefix) = ui.pending_git_prefix.as_ref() {
            deadlines.push(prefix.expires_at());
        }
        if let Some(prefix) = ui.pending_directory_prefix.as_ref() {
            deadlines.push(prefix.started_at + GIT_DIRECTORY_PREFIX_TIMEOUT);
        }
        if let Some(sequence) = ui.pending_key_sequence.as_ref() {
            let options = key_sequence_options(&*shell_user_library(&self.runtime));
            let timeout_ms = if sequence.ambiguous_short.is_some() {
                options.ambiguous_prefix_timeout_ms
            } else {
                options.sequence_idle_timeout_ms
            };
            deadlines.push(sequence.started_at + Duration::from_millis(timeout_ms));
        }
        for buffer in &ui.buffers {
            if buffer.git_fringe_dirty {
                deadlines.push(
                    buffer
                        .git_fringe_last_edit_at
                        .map(|last| last + GIT_FRINGE_REFRESH_DEBOUNCE)
                        .unwrap_or(now),
                );
            }
        }
        if let Some(due_at) = ui.autocomplete_worker.next_due_at() {
            deadlines.push(due_at);
        }
        if let Some(due_at) = ui.inline_completion_worker.next_due_at() {
            deadlines.push(due_at);
        }
        if let Some(due_at) = ui.lsp_sync_worker.next_due_at() {
            deadlines.push(due_at);
        }
        if let Some(due_at) = ui.vim_search_worker.next_due_at() {
            deadlines.push(due_at);
        }
        if let Some(due_at) = ui.workspace_search_worker.next_due_at() {
            deadlines.push(due_at);
        }
        deadlines
    }

    fn active_buffer_is_terminal(&self) -> bool {
        active_shell_buffer_id(&self.runtime)
            .ok()
            .and_then(|buffer_id| shell_buffer(&self.runtime, buffer_id).ok())
            .map(|buffer| buffer_is_terminal(&buffer.kind))
            .unwrap_or(false)
    }

    fn finish_typing_profile(&mut self) -> Result<Option<TypingProfileSummary>, String> {
        self.typing_profiler
            .as_ref()
            .map(TypingProfiler::write_report)
            .transpose()
    }
}
